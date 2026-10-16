//! Original-FD content snapshots, not immutable/trusted/execution/deletion authority.
//! Caller retains namespace and deny-write pins and revalidates before publication.
use super::manifest::BundleManifest;
use crate::file_identity::{NativeFileIdentity, from_file};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::File,
    io,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
#[derive(Debug, Clone)]
pub struct PayloadObservation {
    pub name: String,
    pub bytes: u64,
    pub sha256: String,
    pub identity: NativeFileIdentity,
}
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid app payload")
}
fn checkpoint(cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    if cancel.load(Ordering::Acquire) {
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "app payload cancelled",
        ));
    }
    if Instant::now() >= deadline {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "app payload deadline",
        ));
    }
    Ok(())
}
// Windows seek_read changes that File object's cursor, so reopen the original
// capability rather than cloning its shared file description. No pathname open.
pub(super) fn reading_pin(original: &File) -> io::Result<File> {
    #[cfg(windows)]
    {
        use std::os::windows::io::{AsRawHandle, FromRawHandle};
        use windows_sys::Win32::{
            Foundation::{GENERIC_READ, INVALID_HANDLE_VALUE},
            Storage::FileSystem::{
                FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, ReOpenFile,
            },
        };
        // SAFETY: original remains borrowed/live; returned noninvalid handle is
        // transferred exactly once to File, including failure-check RAII paths.
        let handle = unsafe {
            ReOpenFile(
                original.as_raw_handle(),
                GENERIC_READ,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                0,
            )
        };
        if handle == INVALID_HANDLE_VALUE || handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let file = unsafe { File::from_raw_handle(handle) };
        if from_file(original)? != from_file(&file)? {
            return Err(invalid());
        }
        Ok(file)
    }
    #[cfg(unix)]
    {
        original.try_clone()
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = original;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "payload inspection unavailable",
        ))
    }
}
fn read_at(f: &File, b: &mut [u8], offset: u64) -> io::Result<usize> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileExt;
        f.read_at(b, offset)
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::FileExt;
        f.seek_read(b, offset)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (f, b, offset);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "payload inspection unavailable",
        ))
    }
}
fn pe(header: &[u8]) -> io::Result<()> {
    if header.len() < 64 || header.get(..2) != Some(b"MZ") {
        return Err(invalid());
    }
    let offset = u32::from_le_bytes(header[60..64].try_into().map_err(|_| invalid())?) as usize;
    let end = offset.checked_add(26).ok_or_else(invalid)?;
    if offset < 64
        || end > header.len()
        || header.get(offset..offset + 4) != Some(b"PE\0\0")
        || u16::from_le_bytes([header[offset + 4], header[offset + 5]]) != 0x8664
    {
        return Err(invalid());
    }
    let optional = u16::from_le_bytes([header[offset + 20], header[offset + 21]]) as usize;
    if optional < 112
        || offset
            .checked_add(24)
            .and_then(|n| n.checked_add(optional))
            .is_none_or(|n| n > header.len())
        || u16::from_le_bytes([header[offset + 24], header[offset + 25]]) != 0x20b
    {
        return Err(invalid());
    }
    Ok(())
}
pub fn inspect_payload(
    manifest: &BundleManifest,
    files: &[(&str, &File)],
    cancel: &AtomicBool,
    deadline: Instant,
) -> io::Result<Vec<PayloadObservation>> {
    manifest.validate()?;
    checkpoint(cancel, deadline)?;
    if files.len() != manifest.entries.len() {
        return Err(invalid());
    }
    let mut names = BTreeSet::new();
    for (name, _) in files {
        if !names.insert(*name) || !manifest.entries.iter().any(|e| e.name == *name) {
            return Err(invalid());
        }
    }
    let mut observed: Vec<PayloadObservation> = Vec::with_capacity(files.len());
    for entry in &manifest.entries {
        checkpoint(cancel, deadline)?;
        let file = files
            .iter()
            .find(|(n, _)| *n == entry.name)
            .ok_or_else(invalid)?
            .1;
        let before = file.metadata()?;
        let id = from_file(file)?;
        if !before.is_file()
            || before.len() != entry.bytes
            || observed.iter().any(|a| a.identity == id)
        {
            return Err(invalid());
        }
        let reader = reading_pin(file)?;
        let mut hash = Sha256::new();
        let mut buffer = [0u8; 65536];
        let mut offset = 0u64;
        let mut header = Vec::new();
        loop {
            checkpoint(cancel, deadline)?;
            let remaining = entry.bytes.saturating_sub(offset);
            let wanted = usize::try_from(remaining.min(65536)).map_err(|_| invalid())?;
            if wanted == 0 {
                let mut extra = [0];
                if read_at(&reader, &mut extra, offset)? != 0 {
                    return Err(invalid());
                }
                break;
            }
            let count = match read_at(&reader, &mut buffer[..wanted], offset) {
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                r => r?,
            };
            if count == 0 {
                return Err(invalid());
            }
            if entry.name == "dlpad.exe" && header.len() < 65536 {
                let n = count.min(65536 - header.len());
                header.extend_from_slice(&buffer[..n]);
            }
            hash.update(&buffer[..count]);
            offset = offset.checked_add(count as u64).ok_or_else(invalid)?;
        }
        let digest = format!("{:x}", hash.finalize());
        if digest != entry.sha256 {
            return Err(invalid());
        }
        if entry.name == "dlpad.exe" {
            pe(&header)?;
        }
        checkpoint(cancel, deadline)?;
        let after = file.metadata()?;
        if !after.is_file() || after.len() != entry.bytes || from_file(file)? != id {
            return Err(invalid());
        }
        observed.push(PayloadObservation {
            name: entry.name.clone(),
            bytes: offset,
            sha256: digest,
            identity: id,
        });
    }
    checkpoint(cancel, deadline)?;
    Ok(observed)
}
#[cfg(test)]
mod tests {
    use super::super::manifest::BundleEntry;
    use super::*;
    use std::{
        io::{Seek, SeekFrom},
        time::Duration,
    };
    fn bytes() -> Vec<u8> {
        let mut b = vec![0; 256];
        b[..2].copy_from_slice(b"MZ");
        b[60..64].copy_from_slice(&64u32.to_le_bytes());
        b[64..68].copy_from_slice(b"PE\0\0");
        b[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        b[84..86].copy_from_slice(&112u16.to_le_bytes());
        b[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        b
    }
    fn manifest(b: &[u8]) -> BundleManifest {
        BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-gnu".into(),
            development_only: true,
            entries: vec![BundleEntry {
                name: "dlpad.exe".into(),
                bytes: b.len() as u64,
                sha256: format!("{:x}", Sha256::digest(b)),
            }],
        }
    }
    #[test]
    fn actual_file_snapshot_keeps_cursor_and_bytes() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("tool");
        let b = bytes();
        std::fs::write(&p, &b).unwrap();
        let mut f = File::open(&p).unwrap();
        f.seek(SeekFrom::Start(13)).unwrap();
        let r = inspect_payload(
            &manifest(&b),
            &[("dlpad.exe", &f)],
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(10),
        )
        .unwrap();
        assert_eq!(r[0].bytes, 256);
        assert_eq!(f.stream_position().unwrap(), 13);
        assert_eq!(std::fs::read(p).unwrap(), b);
    }
    #[test]
    fn bad_sha_size_set_and_alias_rejected() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("tool");
        let b = bytes();
        std::fs::write(&p, &b).unwrap();
        let f = File::open(p).unwrap();
        let check = |m: &BundleManifest, fs: &[(&str, &File)]| {
            inspect_payload(
                m,
                fs,
                &AtomicBool::new(false),
                Instant::now() + Duration::from_secs(10),
            )
        };
        let mut m = manifest(&b);
        m.entries[0].sha256 = "a".repeat(64);
        assert!(check(&m, &[("dlpad.exe", &f)]).is_err());
        m = manifest(&b);
        m.entries[0].bytes += 1;
        assert!(check(&m, &[("dlpad.exe", &f)]).is_err());
        assert!(check(&manifest(&b), &[("extra.txt", &f)]).is_err());
        m = manifest(&b);
        m.entries.push(BundleEntry {
            name: "notice.txt".into(),
            bytes: 256,
            sha256: m.entries[0].sha256.clone(),
        });
        assert!(check(&m, &[("dlpad.exe", &f), ("notice.txt", &f)]).is_err());
    }
    #[test]
    fn cancel_and_deadline_are_typed() {
        let f = tempfile::tempfile().unwrap();
        let b = bytes();
        let m = manifest(&b);
        assert_eq!(
            inspect_payload(
                &m,
                &[("dlpad.exe", &f)],
                &AtomicBool::new(true),
                Instant::now() + Duration::from_secs(1)
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::Interrupted
        );
        assert_eq!(
            inspect_payload(
                &m,
                &[("dlpad.exe", &f)],
                &AtomicBool::new(false),
                Instant::now()
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::TimedOut
        );
    }
    #[test]
    fn pe_boundary_wrong_machine_magic_and_truncation() {
        assert!(pe(&bytes()).is_ok());
        for index in [0, 64, 68, 88] {
            let mut b = bytes();
            b[index] ^= 1;
            assert!(pe(&b).is_err());
        }
        let mut b = bytes();
        b[60..64].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(pe(&b).is_err());
        assert!(pe(&bytes()[..90]).is_err());
    }
}
