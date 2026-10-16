//! Bounded Windows kernel directory snapshots; NOT per-file HANDLE observations.
//! Enumeration→later mutation remains a namespace race; batch root checks are
//! detection boundaries, not atomic file publication authority.
//! https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_id_extd_dir_info
use super::LibraryError;
use crate::file_identity::{self, NativeFileIdentity};
use std::{
    fs::{self, File, OpenOptions},
    io,
    mem::{offset_of, size_of},
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    path::{Path, PathBuf},
};
use windows_sys::Win32::{Foundation::ERROR_NO_MORE_FILES, Storage::FileSystem::*};
mod fresh;
const BUFFER: usize = 65536;
const HEADER: usize = offset_of!(FILE_ID_EXTD_DIR_INFO, FileName);
fn bad() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "bounded directory observation rejected",
    )
}
fn read<T: Copy>(bytes: &[u8], at: usize) -> io::Result<T> {
    if at
        .checked_add(size_of::<T>())
        .is_none_or(|end| end > bytes.len())
    {
        return Err(bad());
    }
    // SAFETY: validated byte range, integer/array types only and unaligned copied value.
    Ok(unsafe { std::ptr::read_unaligned(bytes.as_ptr().add(at).cast::<T>()) })
}
struct Entry {
    name: String,
    id: [u8; 16],
    attributes: u32,
    next: usize,
}
fn parse(bytes: &[u8]) -> io::Result<Entry> {
    if bytes.len() < HEADER {
        return Err(bad());
    }
    let next = read::<u32>(bytes, offset_of!(FILE_ID_EXTD_DIR_INFO, NextEntryOffset))? as usize;
    let length = read::<u32>(bytes, offset_of!(FILE_ID_EXTD_DIR_INFO, FileNameLength))? as usize;
    if length == 0 || !length.is_multiple_of(2) || length > 510 {
        return Err(bad());
    }
    let end = HEADER.checked_add(length).ok_or_else(bad)?;
    if end > bytes.len()
        || next != 0 && (!next.is_multiple_of(8) || next < end || next >= bytes.len())
    {
        return Err(bad());
    }
    let mut units = Vec::with_capacity(length / 2);
    for at in (HEADER..end).step_by(2) {
        units.push(read::<u16>(bytes, at)?);
    }
    if units
        .iter()
        .any(|u| matches!(*u, 0 | 47 | 92 | 58) || *u < 32)
    {
        return Err(bad());
    }
    let name = String::from_utf16(&units).map_err(|_| bad())?;
    let size = read::<i64>(bytes, offset_of!(FILE_ID_EXTD_DIR_INFO, EndOfFile))?;
    if size < 0 {
        return Err(bad());
    }
    Ok(Entry {
        name,
        id: read(bytes, offset_of!(FILE_ID_EXTD_DIR_INFO, FileId))?,
        attributes: read(bytes, offset_of!(FILE_ID_EXTD_DIR_INFO, FileAttributes))?,
        next,
    })
}
pub(super) struct NativeObservation {
    pub(super) path: PathBuf,
    pub(super) size: u64,
    pub(super) modified_ms: Option<i64>,
    pub(super) file_identity: String,
}
enum Step {
    PreflightFile,
    Observation(NativeObservation),
}
struct Frame {
    path: PathBuf,
    file: File,
    identity: NativeFileIdentity,
    buffer: Vec<u64>,
    offset: Option<usize>,
    started: bool,
    ended: bool,
}
impl Frame {
    fn open(path: PathBuf, expected: Option<(u64, [u8; 16])>) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .access_mode(FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
            .open(&path)?;
        let mut attrs = FILE_ATTRIBUTE_TAG_INFO::default();
        // SAFETY: original live directory HANDLE and exact initialized output.
        if unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileAttributeTagInfo,
                (&mut attrs as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
                size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if attrs.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
            || attrs.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        {
            return Err(bad());
        }
        let identity = file_identity::from_file(&file)?;
        if let Some((volume, id)) = expected
            && identity
                != (NativeFileIdentity::Windows {
                    volume_serial: volume,
                    file_id: id,
                })
        {
            return Err(bad());
        }
        let frame = Self {
            path,
            file,
            identity,
            buffer: vec![0; BUFFER / 8],
            offset: None,
            started: false,
            ended: false,
        };
        frame.check()?;
        Ok(frame)
    }
    fn check(&self) -> io::Result<()> {
        if file_identity::from_file(&self.file)? != self.identity
            || file_identity::final_path(&self.file)? != fs::canonicalize(&self.path)?
        {
            return Err(bad());
        }
        Ok(())
    }
    fn volume(&self) -> io::Result<u64> {
        if let NativeFileIdentity::Windows { volume_serial, .. } = self.identity {
            Ok(volume_serial)
        } else {
            Err(bad())
        }
    }
    fn batch(&mut self) -> io::Result<()> {
        self.check()?;
        self.buffer.fill(0);
        // SAFETY: retained directory HANDLE and aligned initialized fixed output.
        let result = unsafe {
            GetFileInformationByHandleEx(
                self.file.as_raw_handle(),
                if self.started {
                    FileIdExtdDirectoryInfo
                } else {
                    FileIdExtdDirectoryRestartInfo
                },
                self.buffer.as_mut_ptr().cast(),
                BUFFER as u32,
            )
        };
        let error = (result == 0).then(io::Error::last_os_error);
        self.check()?;
        self.started = true;
        if let Some(error) = error {
            if error.raw_os_error() == Some(ERROR_NO_MORE_FILES as i32) {
                self.ended = true;
                self.offset = None;
                return Ok(());
            }
            if matches!(error.raw_os_error(), Some(1 | 50 | 87)) {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    ClassUnsupported(error),
                ));
            }
            return Err(error);
        }
        self.offset = Some(0);
        Ok(())
    }
    fn entry(&mut self) -> io::Result<Option<Entry>> {
        if self.offset.is_none() && !self.ended {
            self.batch()?;
        }
        let Some(offset) = self.offset else {
            return Ok(None);
        };
        // SAFETY: initialized fixed buffer, view bounded by original Vec capacity.
        let bytes =
            unsafe { std::slice::from_raw_parts(self.buffer.as_ptr().cast::<u8>(), BUFFER) };
        let record = parse(&bytes[offset..])?;
        self.offset = if record.next == 0 {
            None
        } else {
            Some(offset.checked_add(record.next).ok_or_else(bad)?)
        };
        Ok(Some(record))
    }
}
struct RootPin {
    path: PathBuf,
    file: File,
    identity: NativeFileIdentity,
}
impl RootPin {
    fn verify(&self) -> io::Result<()> {
        if file_identity::from_file(&self.file)? != self.identity
            || file_identity::final_path(&self.file)? != fs::canonicalize(&self.path)?
        {
            return Err(bad());
        }
        let mut attrs = FILE_ATTRIBUTE_TAG_INFO::default();
        // SAFETY: retained original root HANDLE and initialized exact output.
        if unsafe {
            GetFileInformationByHandleEx(
                self.file.as_raw_handle(),
                FileAttributeTagInfo,
                (&mut attrs as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
                size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if attrs.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
            || attrs.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        {
            return Err(bad());
        }
        Ok(())
    }
}
#[derive(Debug)]
struct ClassUnsupported(io::Error);
impl std::fmt::Display for ClassUnsupported {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "directory information class unsupported")
    }
}
impl std::error::Error for ClassUnsupported {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}
pub(super) struct NativeWalker {
    root: RootPin,
    frames: Vec<Frame>,
    fresh_api: Option<fresh::Api>,
    fresh_capability_checked: bool,
    publishing: bool,
}
impl NativeWalker {
    pub(super) fn new(root: &Path) -> Result<Option<Self>, LibraryError> {
        if !root.is_absolute()
            || root
                .components()
                .any(|c| c == std::path::Component::ParentDir)
        {
            return Err(bad().into());
        }
        let mut frame = Frame::open(root.to_owned(), None)?;
        let root_pin = RootPin {
            path: frame.path.clone(),
            file: frame.file.try_clone()?,
            identity: frame.identity.clone(),
        };
        root_pin.verify()?;
        // Only the first information-class query may choose fallback, before any observation.
        // ERROR_INVALID_FUNCTION/NOT_SUPPORTED/INVALID_PARAMETER: old OS/filesystem class rejection.
        match frame.batch() {
            Ok(()) => Ok(Some(Self {
                root: root_pin,
                frames: vec![frame],
                fresh_api: fresh::Api::load()?,
                fresh_capability_checked: false,
                publishing: false,
            })),
            Err(e)
                if e.get_ref()
                    .is_some_and(|inner| inner.is::<ClassUnsupported>()) =>
            {
                root_pin.verify()?;
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }
    pub(super) fn verify_root(&self) -> Result<(), LibraryError> {
        self.root.verify()?;
        Ok(())
    }
    /// Sole consumer: restart shares original directory description, never parallel enumeration.
    pub(super) fn restart(&mut self) -> Result<(), LibraryError> {
        self.verify_root()?;
        self.frames.clear();
        let mut frame = Frame {
            path: self.root.path.clone(),
            file: self.root.file.try_clone()?,
            identity: self.root.identity.clone(),
            buffer: vec![0; BUFFER / 8],
            offset: None,
            started: false,
            ended: false,
        };
        frame.batch()?;
        self.frames.push(frame);
        self.publishing = true;
        self.verify_root()?;
        Ok(())
    }
    pub(super) fn next(
        &mut self,
        cancelled: &mut dyn FnMut() -> bool,
    ) -> Result<Option<NativeObservation>, LibraryError> {
        self.step(cancelled, true)?
            .map(|step| match step {
                Step::Observation(row) => Ok(row),
                Step::PreflightFile => Err(bad().into()),
            })
            .transpose()
    }
    /// Capability/namespace traversal only. No metadata can escape this seam.
    /// Probe the optional fresh class once; transactional next() queries every file.
    pub(super) fn preflight_next(
        &mut self,
        cancelled: &mut dyn FnMut() -> bool,
    ) -> Result<Option<()>, LibraryError> {
        self.step(cancelled, false).map(|step| step.map(|_| ()))
    }
    fn step(
        &mut self,
        cancelled: &mut dyn FnMut() -> bool,
        materialize: bool,
    ) -> Result<Option<Step>, LibraryError> {
        loop {
            if cancelled() {
                return Err(LibraryError::Cancelled);
            }
            if self.frames.is_empty() {
                return Ok(None);
            }
            let needs_batch = self
                .frames
                .last()
                .is_some_and(|frame| frame.offset.is_none());
            if needs_batch {
                self.verify_root()?;
            }
            let frame = self.frames.last_mut().ok_or_else(bad)?;
            let record = frame.entry()?;
            if needs_batch {
                self.verify_root()?;
            }
            let frame = self.frames.last_mut().ok_or_else(bad)?;
            let Some(record) = record else {
                frame.check()?;
                self.frames.pop();
                continue;
            };
            if cancelled() {
                return Err(LibraryError::Cancelled);
            }
            if record.name == "." || record.name == ".." {
                continue;
            }
            let path = frame.path.join(&record.name);
            if super::partial(&path) {
                continue;
            }
            // Unknown/reparse records fail the entire transaction, never mid-scan fallback.
            if record.attributes & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DEVICE) != 0
                || record.id == [0; 16]
            {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "unsupported directory record capability",
                )
                .into());
            }
            let volume = frame.volume()?;
            if record.attributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
                if self.frames.len() >= super::MAX_SCAN_DEPTH {
                    return Err(LibraryError::DepthLimit);
                }
                if cancelled() {
                    return Err(LibraryError::Cancelled);
                }
                self.verify_root()?;
                let child = Frame::open(path, Some((volume, record.id)))?;
                self.frames.push(child);
                continue;
            }
            // Per-link directory EOF/time may remain stale after writes via
            // another hard link (including aliases outside the selected root).
            // Preserve enumeration identity, but publish fresh file metadata.
            if cancelled() {
                return Err(LibraryError::Cancelled);
            }
            if !materialize && (self.fresh_capability_checked || self.fresh_api.is_none()) {
                return Ok(Some(Step::PreflightFile));
            }
            let metadata = if let Some(api) = &mut self.fresh_api {
                match api.observe(&path, volume, record.id) {
                    Ok(metadata) => metadata,
                    Err(error) if fresh::unsupported(&error) && !self.publishing => {
                        self.fresh_api = None;
                        fresh::from_handle(&path, volume, record.id)?
                    }
                    Err(error) => return Err(error.into()),
                }
            } else {
                fresh::from_handle(&path, volume, record.id)?
            };
            if cancelled() {
                return Err(LibraryError::Cancelled);
            }
            self.fresh_capability_checked = true;
            if !materialize {
                return Ok(Some(Step::PreflightFile));
            }
            let modified_ms = modified_ms(metadata.modified);
            // The volume and 16 opaque bytes fit in 64 ASCII bytes. Build the
            // same identity directly, avoiding sixteen temporary Strings per file.
            use std::fmt::Write as _;
            let mut file_identity = String::with_capacity(64);
            write!(&mut file_identity, "windows:v1:{volume}:").map_err(|_| bad())?;
            const HEX: &[u8; 16] = b"0123456789abcdef";
            for byte in record.id {
                file_identity.push(char::from(HEX[usize::from(byte >> 4)]));
                file_identity.push(char::from(HEX[usize::from(byte & 15)]));
            }
            if file_identity.len() > 128 {
                return Err(bad().into());
            }
            return Ok(Some(Step::Observation(NativeObservation {
                path,
                size: metadata.size,
                modified_ms,
                file_identity,
            })));
        }
    }
}
fn modified_ms(filetime: i64) -> Option<i64> {
    let ticks = u64::try_from(filetime).ok()?;
    let unix = ticks.checked_sub(116_444_736_000_000_000)?;
    i64::try_from(unix / 10000).ok()
}
#[cfg(test)]
mod tests {
    use super::*;
    fn valid() -> Vec<u8> {
        let mut b = vec![0; HEADER + 16];
        b[offset_of!(FILE_ID_EXTD_DIR_INFO, FileNameLength)..][..4]
            .copy_from_slice(&2u32.to_ne_bytes());
        b[HEADER..][..2].copy_from_slice(&(b'a' as u16).to_ne_bytes());
        b
    }
    #[test]
    fn parser_bounds_reject_malformed_native_records() {
        assert_eq!(parse(&valid()).unwrap().name, "a");
        assert!(parse(&[0; HEADER - 1]).is_err());
        for n in [0u32, 1, 512, u32::MAX] {
            let mut b = valid();
            b[offset_of!(FILE_ID_EXTD_DIR_INFO, FileNameLength)..][..4]
                .copy_from_slice(&n.to_ne_bytes());
            assert!(parse(&b).is_err());
        }
        for offset in [1u32, 8, u32::MAX] {
            let mut b = valid();
            b[..4].copy_from_slice(&offset.to_ne_bytes());
            assert!(parse(&b).is_err());
        }
        for unit in [0u16, 47, 92, 58, 0xd800] {
            let mut b = valid();
            b[HEADER..][..2].copy_from_slice(&unit.to_ne_bytes());
            assert!(parse(&b).is_err());
        }
        let mut b = valid();
        b[offset_of!(FILE_ID_EXTD_DIR_INFO, EndOfFile)..][..8]
            .copy_from_slice(&(-1i64).to_ne_bytes());
        assert!(parse(&b).is_err());
    }
    #[test]
    fn filetime_conversion_matches_unix_nonnegative_policy() {
        assert_eq!(modified_ms(116_444_736_000_000_000), Some(0));
        assert_eq!(modified_ms(116_444_736_000_019_999), Some(1));
        assert_eq!(modified_ms(116_444_735_999_999_999), None);
        assert_eq!(modified_ms(-1), None);
    }
}

#[cfg(test)]
mod native_regressions {
    use super::super::MAX_SCAN_DEPTH;
    use super::*;
    // Draft for include! inside native scanner #[cfg(test)] module after root integration.
    // NativeWalker/LibraryError/MAX_SCAN_DEPTH accessible from parent imports.
    // No production fault hook; expected-ID/parser unit tests require agreed private APIs.
    #[test]
    fn retained_root_handle_denies_namespace_rename() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("original");
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("원본.mp4"), b"abc").unwrap();
        let walker = NativeWalker::new(&root)
            .unwrap()
            .expect("native supported fixture");
        let moved = temp.path().join("moved");
        let error = std::fs::rename(&root, &moved).unwrap_err();
        assert!(matches!(error.raw_os_error(), Some(5 | 32)));
        assert_eq!(std::fs::read(root.join("원본.mp4")).unwrap(), b"abc");
        assert!(!moved.exists());
        drop(walker);
        std::fs::rename(&root, &moved).unwrap();
        assert_eq!(std::fs::read(moved.join("원본.mp4")).unwrap(), b"abc");
    }

    #[test]
    fn native_walker_depth_limit_fails_instead_of_truncating_snapshot() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("depth");
        std::fs::create_dir(&root).unwrap();
        let mut path = root.clone();
        for _ in 0..MAX_SCAN_DEPTH {
            path.push("d");
            std::fs::create_dir(&path).unwrap();
        }
        std::fs::write(path.join("leaf.mp4"), b"abc").unwrap();
        let mut walker = NativeWalker::new(&root)
            .unwrap()
            .expect("native supported fixture");
        let mut never_cancel = || false;
        assert!(matches!(
            walker.next(&mut never_cancel),
            Err(LibraryError::DepthLimit)
        ));
        assert_eq!(std::fs::read(path.join("leaf.mp4")).unwrap(), b"abc");
    }

    // Existing public cancellation before root open / after-first rollback and
    // hardlink transfer tests remain the primary policy regressions; not duplicated.
    // Follow-up after private API handoff: batch expectedID vs deleted/replaced child,
    // malformed record parser offset/UTF16 failures, API Unsupported fallback only,
    // Unicode/hardlink exact observation checks. No synthetic unsupported error is
    // treated as an actual OS capability test; junction/symlink rights remain untested.

    #[test]
    fn child_original_id_rejects_deleted_and_replaced_namespace() {
        let temp = tempfile::tempdir().unwrap();
        let child = temp.path().join("child");
        std::fs::create_dir(&child).unwrap();
        let old = Frame::open(child.clone(), None).unwrap();
        let NativeFileIdentity::Windows {
            volume_serial,
            file_id,
        } = old.identity
        else {
            panic!("native Windows ID required")
        };
        // Create a distinct live object while the original still exists: no ID reuse assumption.
        let other = temp.path().join("different");
        std::fs::create_dir(&other).unwrap();
        let different = Frame::open(other.clone(), None).unwrap();
        let NativeFileIdentity::Windows {
            file_id: other_id, ..
        } = different.identity
        else {
            panic!("native Windows ID required")
        };
        assert_ne!(other_id, file_id);
        drop(different);
        drop(old);
        std::fs::remove_dir(&child).unwrap();
        assert!(Frame::open(child.clone(), Some((volume_serial, file_id))).is_err());
        std::fs::rename(&other, &child).unwrap();
        assert!(Frame::open(child.clone(), Some((volume_serial, file_id))).is_err());
        assert!(child.is_dir());
    }

    #[test]
    fn unicode_native_observations_equal_original_handle_metadata_and_skip_partials() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let source = root.join("한글 공백.mp4");
        std::fs::write(&source, b"abc").unwrap();
        let link = root.join("같은 원본.mp4");
        std::fs::hard_link(&source, &link).unwrap();
        for name in ["not.part", "not.ytdl", "not.tmp", "not.part-Frag1"] {
            std::fs::write(root.join(name), b"pending").unwrap();
        }
        std::fs::create_dir(root.join(".dlpad-partials")).unwrap();
        std::fs::write(root.join(".dlpad-partials/hidden.mp4"), b"pending").unwrap();
        let mut walker = NativeWalker::new(root)
            .unwrap()
            .expect("native supported fixture");
        let mut count = 0;
        let mut shared = None;
        while let Some(row) = walker.next(&mut || false).unwrap() {
            assert!(row.path == source || row.path == link);
            let file = crate::file_identity::open_regular_no_reparse(&row.path).unwrap();
            let NativeFileIdentity::Windows {
                volume_serial,
                file_id,
            } = crate::file_identity::from_file(&file).unwrap()
            else {
                panic!("native Windows ID required")
            };
            let exact = format!(
                "windows:v1:{volume_serial}:{}",
                file_id
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            );
            let metadata = std::fs::symlink_metadata(&row.path).unwrap();
            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .and_then(|d| i64::try_from(d.as_millis()).ok());
            assert_eq!(row.file_identity, exact);
            assert_eq!(row.size, metadata.len());
            assert_eq!(row.modified_ms, modified);
            if let Some(first) = &shared {
                assert_eq!(&row.file_identity, first);
            } else {
                shared = Some(row.file_identity);
            }
            count += 1;
        }
        assert_eq!(count, 2);
    }
}
