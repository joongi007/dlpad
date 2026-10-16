//! Windows named-copy export candidate; source remains unchanged.
//! Requires Root-owned typed created_relative_file facade (see SHARED_BOUNDARY.md).
use crate::{file_identity::NativeFileIdentity, library::LibraryEntry};
#[cfg(any(windows, test))]
use sha2::{Digest, Sha256};
#[cfg(any(windows, test))]
use std::sync::atomic::Ordering;
use std::{
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, atomic::AtomicBool},
    time::Instant,
};
#[cfg(any(windows, test))]
use std::{
    io::{Read, Write},
    time::Duration,
};
#[cfg(any(windows, test))]
const CHUNK: usize = 64 * 1024;
#[cfg(any(windows, test))]
const MAX_BYTES: u64 = 1024 * 1024 * 1024 * 1024;
#[cfg(windows)]
const MAX_TIMEOUT: Duration = Duration::from_secs(300);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Copy,
    VerifyStage,
    VerifySource,
    VerifyFinal,
    Complete,
}
#[derive(Clone, Debug)]
pub struct Progress {
    pub phase: Phase,
    pub bytes: u64,
    pub total: u64,
}
#[derive(Clone, Default, Debug)]
pub struct ProgressFeed(Arc<Mutex<Option<Progress>>>);
impl ProgressFeed {
    pub fn latest(&self) -> Option<Progress> {
        self.0.lock().ok().and_then(|p| p.clone())
    }
    #[cfg(any(windows, test))]
    fn publish(&self, phase: Phase, bytes: u64, total: u64) {
        if let Ok(mut latest) = self.0.lock() {
            *latest = Some(Progress {
                phase,
                bytes,
                total,
            });
        }
    }
}
pub struct Request {
    pub source: LibraryEntry,
    pub output_parent: PathBuf,
    pub output_name: String,
}
#[derive(Debug)]
pub enum Publication {
    NotAttempted,
    Unconfirmed,
    PublishedUnverified,
    Verified,
}
#[derive(Debug)]
pub enum Creation {
    NotAttempted,
    AttemptedUnconfirmed,
    OwnedCreated,
}
#[derive(Debug)]
pub enum Cleanup {
    /// No cleanup was attempted. This does NOT prove path/file absence.
    NotAttempted,
    OriginalRemoved,
    OriginalRetained,
    Unconfirmed,
}
#[derive(Debug)]
pub struct Failure {
    pub error: io::Error,
    pub creation: Creation,
    pub publication: Publication,
    pub cleanup: Cleanup,
    /// Attempted stage name or retained owned name; facts only, never deletion authority.
    pub stage_path: Option<PathBuf>,
    pub target_path: Option<PathBuf>,
    pub created_identity: Option<NativeFileIdentity>,
}
#[derive(Debug)]
pub struct Exported {
    pub path: PathBuf,
    pub bytes: u64,
    pub sha256: String,
    pub source_identity: NativeFileIdentity,
    pub target_identity: NativeFileIdentity,
}
fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
#[cfg(any(windows, test))]
fn check(cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    if cancel.load(Ordering::Acquire) {
        return Err(io::ErrorKind::Interrupted.into());
    }
    if Instant::now() >= deadline {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(())
}
/// Same component policy as proposed download naming, but explicit user input is
/// rejected rather than silently sanitizing/truncating it to another destination.
pub fn validate_name(name: &str, extension: &str) -> io::Result<()> {
    let base = name
        .split('.')
        .next()
        .unwrap_or("")
        .trim_end()
        .to_ascii_uppercase();
    let reserved = matches!(
        base.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$" | "CONIN$" | "CONOUT$"
    ) || ["COM", "LPT"].iter().any(|p| {
        base.strip_prefix(p).is_some_and(|n| {
            matches!(
                n,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    });
    if name.is_empty()||name.len()>240||name.encode_utf16().count()>240||name.ends_with([' ','.'])||reserved||name.chars().any(|c|c.is_control()||matches!(c,'<'|'>'|':'|'"'|'/'|'\\'|'|'|'?'|'*'|'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}'))||extension.is_empty()||extension.len()>12||!extension.bytes().all(|b|b.is_ascii_alphanumeric())||!Path::new(name).extension().and_then(|s|s.to_str()).is_some_and(|actual|actual.eq_ignore_ascii_case(extension)){return Err(invalid("invalid export name or changed extension"));}
    Ok(())
}
#[cfg(any(windows, test))]
fn windows_identity(value: &str) -> io::Result<NativeFileIdentity> {
    let fields: Vec<_> = value.split(':').collect();
    let ["windows", "v1", volume, hex] = fields.as_slice() else {
        return Err(invalid("missing original library file identity"));
    };
    if hex.len() != 32 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid("invalid original library identity"));
    }
    let mut file_id = [0u8; 16];
    for (index, byte) in file_id.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)
            .map_err(|_| invalid("invalid identity bytes"))?;
    }
    if file_id == [0; 16] {
        return Err(invalid("zero original library identity"));
    }
    Ok(NativeFileIdentity::Windows {
        volume_serial: volume
            .parse()
            .map_err(|_| invalid("invalid identity volume"))?,
        file_id,
    })
}
#[cfg(any(windows, test))]
fn hash<R: Read>(
    reader: &mut R,
    expected: u64,
    cancel: &AtomicBool,
    deadline: Instant,
    feed: &ProgressFeed,
    phase: Phase,
) -> io::Result<String> {
    let mut block = [0u8; CHUNK];
    let mut total = 0u64;
    let mut digest = Sha256::new();
    feed.publish(phase, 0, expected);
    loop {
        check(cancel, deadline)?;
        let n = reader.read(&mut block)?;
        check(cancel, deadline)?;
        if n == 0 {
            break;
        }
        total = total
            .checked_add(n as u64)
            .ok_or_else(|| invalid("export byte overflow"))?;
        if total > expected {
            return Err(invalid("export file grew"));
        }
        digest.update(&block[..n]);
        feed.publish(phase, total, expected);
    }
    if total != expected {
        return Err(invalid("export file truncated"));
    }
    Ok(format!("{:x}", digest.finalize()))
}
#[cfg(any(windows, test))]
fn copy<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    expected: u64,
    cancel: &AtomicBool,
    deadline: Instant,
    feed: &ProgressFeed,
) -> io::Result<String> {
    if expected == 0 || expected > MAX_BYTES {
        return Err(invalid("export size limit"));
    }
    let mut block = [0u8; CHUNK];
    let mut total = 0u64;
    let mut digest = Sha256::new();
    feed.publish(Phase::Copy, 0, expected);
    loop {
        check(cancel, deadline)?;
        let n = reader.read(&mut block)?;
        check(cancel, deadline)?;
        if n == 0 {
            break;
        }
        total = total
            .checked_add(n as u64)
            .ok_or_else(|| invalid("export byte overflow"))?;
        if total > expected {
            return Err(invalid("export source grew"));
        }
        writer.write_all(&block[..n])?;
        check(cancel, deadline)?;
        digest.update(&block[..n]);
        feed.publish(Phase::Copy, total, expected);
    }
    if total != expected {
        return Err(invalid("export source truncated"));
    }
    Ok(format!("{:x}", digest.finalize()))
}
#[cfg(any(windows, test))]
fn preparation_failure(
    error: io::Error,
    creation: Creation,
    stage_path: Option<PathBuf>,
    target_path: Option<PathBuf>,
    created_identity: Option<NativeFileIdentity>,
) -> Failure {
    let cleanup = if matches!(creation, Creation::AttemptedUnconfirmed) {
        Cleanup::Unconfirmed
    } else {
        Cleanup::NotAttempted
    };
    Failure {
        error,
        creation,
        publication: Publication::NotAttempted,
        cleanup,
        stage_path,
        target_path,
        created_identity,
    }
}
/// Root library worker supplies an exact selected row snapshot; not a raw UI path.
/// Four sequential full reads are visible as progress phases, within one 300s budget.
pub fn export(
    request: Request,
    cancel: &AtomicBool,
    deadline: Instant,
    feed: &ProgressFeed,
) -> Result<Exported, Failure> {
    #[cfg(windows)]
    {
        native::export(request, cancel, deadline, feed)
    }
    #[cfg(not(windows))]
    {
        let _ = (request, cancel, deadline, feed);
        Err(Failure {
            error: io::Error::new(io::ErrorKind::Unsupported, "Windows copy export only"),
            creation: Creation::NotAttempted,
            publication: Publication::NotAttempted,
            cleanup: Cleanup::NotAttempted,
            stage_path: None,
            target_path: None,
            created_identity: None,
        })
    }
}
#[cfg(windows)]
mod native {
    use super::*;
    use crate::created_relative_file::{CleanupOutcome, CreatedRelativeFile, PublishError};
    use std::{
        fs::{self, File, OpenOptions},
        io::{Seek, SeekFrom},
        os::windows::{ffi::OsStrExt, fs::OpenOptionsExt},
        path::{Component, Prefix},
        sync::atomic::AtomicU64,
        time::{SystemTime, UNIX_EPOCH},
    };
    use windows_sys::Win32::Storage::FileSystem::{FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ};
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    fn boundary(e: crate::tools::ToolError) -> io::Error {
        match e {
            crate::tools::ToolError::Io(e) => e,
            e => io::Error::other(e.to_string()),
        }
    }
    fn path_valid(path: &Path) -> io::Result<()> {
        if !path.is_absolute()
            || path.components().count() > 256
            || path.as_os_str().encode_wide().count() > 32766
            || path
                .components()
                .any(|p| matches!(p, Component::CurDir | Component::ParentDir))
            || !matches!(path.components().next(),Some(Component::Prefix(p))if matches!(p.kind(),Prefix::Disk(_)|Prefix::VerbatimDisk(_)))
        {
            return Err(invalid("local absolute export path required"));
        }
        Ok(())
    }
    fn parents(
        path: &Path,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> io::Result<Vec<(PathBuf, File)>> {
        path_valid(path)?;
        let mut pins = vec![];
        for parent in path.ancestors().collect::<Vec<_>>().into_iter().rev() {
            check(cancel, deadline)?;
            pins.push((
                parent.to_owned(),
                crate::tools::native_files::open_directory(parent).map_err(boundary)?,
            ));
        }
        Ok(pins)
    }
    fn verify_parents(
        pins: &[(PathBuf, File)],
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> io::Result<()> {
        for (path, pin) in pins {
            check(cancel, deadline)?;
            if !crate::tools::native_files::same_directory(path, pin) {
                return Err(invalid("export parent identity changed"));
            }
        }
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn test_parent_pins(path: &Path) -> io::Result<Vec<(PathBuf, File)>> {
        parents(
            path,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(5),
        )
    }
    #[cfg(test)]
    pub(super) fn test_verify_parents(pins: &[(PathBuf, File)]) -> io::Result<()> {
        verify_parents(
            pins,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(5),
        )
    }
    fn modified_ms(time: SystemTime) -> io::Result<i64> {
        i64::try_from(
            time.duration_since(UNIX_EPOCH)
                .map_err(|_| invalid("export source timestamp"))?
                .as_millis(),
        )
        .map_err(|_| invalid("export source timestamp overflow"))
    }
    fn verify_source(
        source: &File,
        path: &Path,
        id: &NativeFileIdentity,
        bytes: u64,
        modified: SystemTime,
    ) -> io::Result<()> {
        let metadata = source.metadata()?;
        if !metadata.is_file()
            || metadata.len() != bytes
            || metadata.modified()? != modified
            || crate::file_identity::from_file(source)? != *id
            || crate::file_identity::final_path(source)? != path
            || !crate::tools::native_files::same_file(path, source)
        {
            return Err(invalid("export source changed"));
        }
        Ok(())
    }
    fn cleanup(created: CreatedRelativeFile) -> Cleanup {
        match created.cancel_cleanup() {
            CleanupOutcome::RemovedOriginal => Cleanup::OriginalRemoved,
            CleanupOutcome::RetainedOriginal => Cleanup::OriginalRetained,
            CleanupOutcome::Unconfirmed => Cleanup::Unconfirmed,
        }
    }
    pub(super) fn export(
        request: Request,
        cancel: &AtomicBool,
        deadline: Instant,
        feed: &ProgressFeed,
    ) -> Result<Exported, Failure> {
        let deadline = deadline.min(Instant::now() + MAX_TIMEOUT);
        let mut creation = Creation::NotAttempted;
        let mut stage_path = None;
        let mut target_path = None;
        let mut created_identity = None;
        let prepared = (|| -> io::Result<_> {
            check(cancel, deadline)?;
            path_valid(&request.source.path)?;
            path_valid(&request.output_parent)?;
            let extension = request
                .source
                .path
                .extension()
                .and_then(|s| s.to_str())
                .ok_or_else(|| invalid("source extension missing"))?;
            validate_name(&request.output_name, extension)?;
            let expected = windows_identity(
                request
                    .source
                    .file_identity
                    .as_deref()
                    .ok_or_else(|| invalid("selected source identity missing"))?,
            )?;
            let source_parent = request
                .source
                .path
                .parent()
                .ok_or_else(|| invalid("source parent missing"))?;
            let source_parents = parents(source_parent, cancel, deadline)?;
            let output_parents = parents(&request.output_parent, cancel, deadline)?;
            let observed =
                crate::tools::native_files::open_regular(&request.source.path).map_err(boundary)?;
            let source_path = crate::file_identity::final_path(&observed)?;
            // Strong read pin rejects already-open/new writers and DELETE. No immutable
            // claim for the broad-sharing native_files observation handle itself.
            let source = OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&source_path)?;
            let actual = crate::file_identity::from_file(&source)?;
            let metadata = source.metadata()?;
            let modified = metadata.modified()?;
            if actual != expected
                || crate::file_identity::from_file(&observed)? != expected
                || metadata.len() != request.source.size
                || request.source.modified_ms != Some(modified_ms(modified)?)
                || request.source.size == 0
                || request.source.size > MAX_BYTES
            {
                return Err(invalid("selected library source changed"));
            }
            let parent_pin = &output_parents
                .last()
                .ok_or_else(|| invalid("output parent missing"))?
                .1;
            let parent_path = crate::file_identity::final_path(parent_pin)?;
            let target = parent_path.join(&request.output_name);
            path_valid(&target)?;
            if target == source_path {
                return Err(invalid("source and export target are the same"));
            }
            match fs::symlink_metadata(&target) {
                Ok(_) => return Err(io::ErrorKind::AlreadyExists.into()),
                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                Err(e) => return Err(e),
            }
            verify_parents(&source_parents, cancel, deadline)?;
            verify_parents(&output_parents, cancel, deadline)?;
            verify_source(&source, &source_path, &expected, metadata.len(), modified)?;
            let leaf = format!(
                ".dlpad-export-{}-{}.partial",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            );
            // Record attempted namespace BEFORE factory: it may create the original
            // file then fail post-create validation. The pathname is not authority.
            stage_path = Some(parent_path.join(&leaf));
            target_path = Some(target);
            creation = Creation::AttemptedUnconfirmed;
            let created = CreatedRelativeFile::create(parent_pin, &leaf)?;
            creation = Creation::OwnedCreated;
            created_identity = Some(created.identity().clone());
            Ok((
                source,
                source_path,
                expected,
                metadata.len(),
                modified,
                source_parents,
                output_parents,
                parent_path,
                created,
            ))
        })();
        let (
            mut source,
            source_path,
            expected,
            bytes,
            modified,
            source_parents,
            output_parents,
            parent_path,
            mut created,
        ) = match prepared {
            Ok(v) => v,
            Err(error) => {
                return Err(preparation_failure(
                    error,
                    creation,
                    stage_path,
                    target_path,
                    created_identity,
                ));
            }
        };
        let copied = (|| -> io::Result<String> {
            let digest = copy(
                &mut source,
                created.file_mut(),
                bytes,
                cancel,
                deadline,
                feed,
            )?;
            created.file_mut().sync_all()?;
            verify_source(&source, &source_path, &expected, bytes, modified)?;
            verify_parents(&source_parents, cancel, deadline)?;
            verify_parents(&output_parents, cancel, deadline)?;
            created.verify_current(&parent_path)?;
            created.file_mut().seek(SeekFrom::Start(0))?;
            if hash(
                created.file_mut(),
                bytes,
                cancel,
                deadline,
                feed,
                Phase::VerifyStage,
            )? != digest
            {
                return Err(invalid("copied stage digest mismatch"));
            }
            source.seek(SeekFrom::Start(0))?;
            if hash(
                &mut source,
                bytes,
                cancel,
                deadline,
                feed,
                Phase::VerifySource,
            )? != digest
            {
                return Err(invalid("source digest changed during export"));
            }
            verify_source(&source, &source_path, &expected, bytes, modified)?;
            verify_parents(&source_parents, cancel, deadline)?;
            verify_parents(&output_parents, cancel, deadline)?;
            created.verify_current(&parent_path)?;
            check(cancel, deadline)?;
            Ok(digest)
        })();
        let digest = match copied {
            Ok(d) => d,
            Err(error) => {
                return Err(Failure {
                    error,
                    creation: Creation::OwnedCreated,
                    publication: Publication::NotAttempted,
                    cleanup: cleanup(created),
                    stage_path,
                    target_path,
                    created_identity,
                });
            }
        };
        let mut published = match created.publish_no_replace(&request.output_name) {
            Ok(p) => p,
            Err(PublishError::BeforePublish { created, error }) => {
                return Err(Failure {
                    error,
                    creation: Creation::OwnedCreated,
                    publication: Publication::NotAttempted,
                    cleanup: cleanup(created),
                    stage_path,
                    target_path,
                    created_identity,
                });
            }
            Err(PublishError::Unconfirmed {
                source_path,
                target_path: native_target,
                identity,
                error,
            }) => {
                // Native facts preserve evidence only; they never authorize path cleanup.
                if !source_path.as_os_str().is_empty() {
                    stage_path = Some(source_path);
                }
                if !native_target.as_os_str().is_empty() {
                    target_path = Some(native_target);
                }
                created_identity = Some(identity);
                return Err(Failure {
                    error,
                    creation: Creation::OwnedCreated,
                    publication: Publication::Unconfirmed,
                    cleanup: Cleanup::Unconfirmed,
                    stage_path,
                    target_path,
                    created_identity,
                });
            }
        };
        let verified = (|| -> io::Result<()> {
            published.reader_mut().seek(SeekFrom::Start(0))?;
            if hash(
                published.reader_mut(),
                bytes,
                cancel,
                deadline,
                feed,
                Phase::VerifyFinal,
            )? != digest
            {
                return Err(invalid("published export digest mismatch"));
            }
            published.verify_current(&parent_path)?;
            verify_parents(&source_parents, cancel, deadline)?;
            verify_parents(&output_parents, cancel, deadline)?;
            verify_source(&source, &source_path, &expected, bytes, modified)?;
            check(cancel, deadline)?;
            Ok(())
        })();
        if let Err(error) = verified {
            return Err(Failure {
                error,
                creation: Creation::OwnedCreated,
                publication: Publication::PublishedUnverified,
                cleanup: Cleanup::OriginalRetained,
                stage_path: None,
                target_path: Some(published.path().to_owned()),
                created_identity: Some(published.identity().clone()),
            });
        }
        feed.publish(Phase::Complete, bytes, bytes);
        Ok(Exported {
            path: published.path().to_owned(),
            bytes,
            sha256: digest,
            source_identity: expected,
            target_identity: published.identity().clone(),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    fn deadline() -> Instant {
        Instant::now() + Duration::from_secs(2)
    }
    #[test]
    fn user_name_hangul_emoji_extension_and_reserved_policy() {
        validate_name("한글 강의 🎬.mp4", "mp4").unwrap();
        validate_name("copy.MP4", "mp4").unwrap();
        for bad in [
            "CON.mp4",
            "nul.mp4",
            "LPT².mp4",
            "bad:stream.mp4",
            "../foreign.mp4",
            "bad.mp4.",
            "bad.mp4 ",
            "bad.avi",
            "\u{202e}hide.mp4",
        ] {
            assert!(validate_name(bad, "mp4").is_err());
        }
        assert!(validate_name(&format!("{}.mp4", "🎬".repeat(100)), "mp4").is_err());
    }
    #[test]
    fn bounded_stream_copy_four_phase_hashes_match_without_progress_queue_growth() {
        struct Bounded {
            inner: Cursor<Vec<u8>>,
            calls: usize,
        }
        impl Read for Bounded {
            fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
                assert!(out.len() <= CHUNK);
                self.calls += 1;
                self.inner.read(out)
            }
        }
        let raw = vec![37u8; CHUNK * 3 + 17];
        let mut source = Bounded {
            inner: Cursor::new(raw.clone()),
            calls: 0,
        };
        let mut destination = vec![];
        let cancel = AtomicBool::new(false);
        let feed = ProgressFeed::default();
        let digest = copy(
            &mut source,
            &mut destination,
            raw.len() as u64,
            &cancel,
            deadline(),
            &feed,
        )
        .unwrap();
        assert!(source.calls >= 4);
        assert_eq!(destination, raw);
        assert_eq!(feed.latest().unwrap().phase, Phase::Copy);
        for phase in [Phase::VerifyStage, Phase::VerifySource, Phase::VerifyFinal] {
            assert_eq!(
                hash(
                    &mut Cursor::new(raw.clone()),
                    raw.len() as u64,
                    &cancel,
                    deadline(),
                    &feed,
                    phase
                )
                .unwrap(),
                digest
            );
            assert_eq!(feed.latest().unwrap().phase, phase);
            assert_eq!(feed.latest().unwrap().bytes, raw.len() as u64);
        }
    }
    #[test]
    fn cancel_partial_write_read_failure_and_size_drift_are_distinct() {
        struct CancelWriter<'a> {
            cancel: &'a AtomicBool,
            bytes: usize,
        }
        impl Write for CancelWriter<'_> {
            fn write(&mut self, raw: &[u8]) -> io::Result<usize> {
                self.bytes += raw.len();
                self.cancel.store(true, Ordering::Release);
                Ok(raw.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let cancel = AtomicBool::new(false);
        let mut writer = CancelWriter {
            cancel: &cancel,
            bytes: 0,
        };
        let feed = ProgressFeed::default();
        assert_eq!(
            copy(
                &mut Cursor::new(vec![1u8; CHUNK * 2]),
                &mut writer,
                (CHUNK * 2) as u64,
                &cancel,
                deadline(),
                &feed
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::Interrupted
        );
        assert_eq!(writer.bytes, CHUNK);
        struct NoSpace;
        impl Write for NoSpace {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::ErrorKind::StorageFull.into())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        assert_eq!(
            copy(
                &mut Cursor::new(vec![1u8; 8]),
                &mut NoSpace,
                8,
                &AtomicBool::new(false),
                deadline(),
                &feed
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::StorageFull
        );
        struct ReadFail;
        impl Read for ReadFail {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::ErrorKind::PermissionDenied.into())
            }
        }
        assert_eq!(
            copy(
                &mut ReadFail,
                &mut vec![],
                8,
                &AtomicBool::new(false),
                deadline(),
                &feed
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::PermissionDenied
        );
        for (raw, expected) in [(vec![1u8; 7], 8), (vec![1u8; 9], 8)] {
            assert_eq!(
                copy(
                    &mut Cursor::new(raw),
                    &mut vec![],
                    expected,
                    &AtomicBool::new(false),
                    deadline(),
                    &feed
                )
                .unwrap_err()
                .kind(),
                io::ErrorKind::InvalidInput
            );
        }
        assert_eq!(
            copy(
                &mut Cursor::new(vec![1u8; 8]),
                &mut vec![],
                8,
                &AtomicBool::new(false),
                Instant::now(),
                &feed
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::TimedOut
        );
    }
    #[test]
    fn factory_postcreate_validation_failure_retains_attempted_name_without_cleanup_authority() {
        let root = tempfile::tempdir().unwrap();
        let stage = root.path().join(".dlpad-export-fixture.partial");
        let target = root.path().join("copy.mp4");
        // Simulated factory has created its namespace, then native identity validation failed.
        std::fs::write(&stage, b"retained postcreate fixture").unwrap();
        let failure = preparation_failure(
            io::ErrorKind::Unsupported.into(),
            Creation::AttemptedUnconfirmed,
            Some(stage.clone()),
            Some(target),
            None,
        );
        assert!(matches!(failure.creation, Creation::AttemptedUnconfirmed));
        assert!(matches!(failure.cleanup, Cleanup::Unconfirmed));
        assert!(matches!(failure.publication, Publication::NotAttempted));
        assert_eq!(failure.stage_path.as_ref(), Some(&stage));
        assert!(failure.created_identity.is_none());
        assert_eq!(
            std::fs::read(&stage).unwrap(),
            b"retained postcreate fixture"
        );
        let before = preparation_failure(
            io::ErrorKind::InvalidInput.into(),
            Creation::NotAttempted,
            None,
            None,
            None,
        );
        assert!(matches!(before.cleanup, Cleanup::NotAttempted));
        assert!(before.stage_path.is_none());
    }
    #[test]
    fn full_identity_parser_never_truncates_file_id_or_accepts_missing_selection() {
        let original = "windows:v1:18446744073709551615:0102030405060708090a0b0c0d0e0f10";
        assert_eq!(
            windows_identity(original).unwrap(),
            NativeFileIdentity::Windows {
                volume_serial: u64::MAX,
                file_id: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
            }
        );
        for bad in [
            "windows:1:1:ff",
            "windows:v1:1:00000000000000000000000000000000",
            "unix:1:2",
            "windows:v1:1:0102030405060708090a0b0c0d0e0f10:extra",
        ] {
            assert!(windows_identity(bad).is_err());
        }
    }
}
#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;
    use std::{fs, sync::Mutex, time::UNIX_EPOCH};
    static SERIAL: Mutex<()> = Mutex::new(());
    fn selected(path: &Path) -> LibraryEntry {
        let file = crate::file_identity::open_regular_no_reparse(path).unwrap();
        let metadata = file.metadata().unwrap();
        let NativeFileIdentity::Windows {
            volume_serial,
            file_id,
        } = crate::file_identity::from_file(&file).unwrap()
        else {
            panic!("actual Windows full native identity required")
        };
        LibraryEntry {
            path: fs::canonicalize(path).unwrap(),
            size: metadata.len(),
            modified_ms: Some(
                i64::try_from(
                    metadata
                        .modified()
                        .unwrap()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis(),
                )
                .unwrap(),
            ),
            file_identity: Some(format!(
                "windows:v1:{volume_serial}:{}",
                file_id
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            )),
        }
    }
    fn fixture() -> (tempfile::TempDir, PathBuf, LibraryEntry, Vec<u8>) {
        let dir = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(dir.path()).unwrap();
        let path = root.join("원본.mp4");
        let bytes = vec![77u8; CHUNK * 2 + 17];
        fs::write(&path, &bytes).unwrap();
        let source = selected(&path);
        (dir, root, source, bytes)
    }
    #[test]
    fn same_folder_new_name_copy_is_verified_and_original_unchanged() {
        let _serial = SERIAL.lock().unwrap();
        let (_dir, root, source, bytes) = fixture();
        let original = source.clone();
        let feed = ProgressFeed::default();
        let result = export(
            Request {
                source,
                output_parent: root,
                output_name: "새 이름 🎬.mp4".into(),
            },
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(20),
            &feed,
        )
        .unwrap();
        assert_eq!(fs::read(&result.path).unwrap(), bytes);
        assert_eq!(fs::read(&original.path).unwrap(), bytes);
        assert_ne!(result.source_identity, result.target_identity);
        assert_eq!(result.sha256, format!("{:x}", Sha256::digest(&bytes)));
        assert_eq!(feed.latest().unwrap().phase, Phase::Complete);
    }
    #[test]
    fn collision_exact_source_and_hardlink_never_create_or_overwrite() {
        let _serial = SERIAL.lock().unwrap();
        let (_dir, root, source, bytes) = fixture();
        fs::write(root.join("existing.mp4"), b"user data").unwrap();
        fs::hard_link(&source.path, root.join("alias.mp4")).unwrap();
        for name in ["existing.mp4", "原本-placeholder.mp4", "alias.mp4"] {
            let name = if name == "原本-placeholder.mp4" {
                source.path.file_name().unwrap().to_str().unwrap()
            } else {
                name
            };
            let failure = export(
                Request {
                    source: source.clone(),
                    output_parent: root.clone(),
                    output_name: name.into(),
                },
                &AtomicBool::new(false),
                Instant::now() + Duration::from_secs(20),
                &ProgressFeed::default(),
            )
            .unwrap_err();
            assert!(matches!(failure.publication, Publication::NotAttempted));
            assert!(matches!(failure.cleanup, Cleanup::NotAttempted));
        }
        assert_eq!(fs::read(root.join("existing.mp4")).unwrap(), b"user data");
        assert_eq!(fs::read(&source.path).unwrap(), bytes);
        assert_eq!(fs::read(root.join("alias.mp4")).unwrap(), bytes);
    }
    #[test]
    fn original_source_replacement_and_original_parent_replacement_are_detected() {
        let _serial = SERIAL.lock().unwrap();
        let (_dir, root, source, bytes) = fixture();
        fs::rename(&source.path, root.join("retained-original.mp4")).unwrap();
        fs::write(&source.path, &bytes).unwrap();
        let failure = export(
            Request {
                source,
                output_parent: root.clone(),
                output_name: "new.mp4".into(),
            },
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(20),
            &ProgressFeed::default(),
        )
        .unwrap_err();
        assert!(matches!(failure.cleanup, Cleanup::NotAttempted));
        assert!(!root.join("new.mp4").exists());
        assert_eq!(fs::read(root.join("retained-original.mp4")).unwrap(), bytes);
        let parent = root.join("output");
        fs::create_dir(&parent).unwrap();
        let pins = native::test_parent_pins(&parent).unwrap();
        fs::rename(&parent, root.join("retained-parent")).unwrap();
        fs::create_dir(&parent).unwrap();
        assert!(native::test_verify_parents(&pins).is_err());
        assert!(parent.is_dir());
        assert!(root.join("retained-parent").is_dir());
    }
}
