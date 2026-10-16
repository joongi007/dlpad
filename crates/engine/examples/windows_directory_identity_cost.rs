//! Native validated_identity rejects zero IDs; library observe maps errors to None.
//! No-delete-share directory does not pin all ancestors; batch canonical checks
//! detect observations of replacement, not an atomic namespace race guarantee.
//! Only first128 entries/round receive individual original-file crosschecks.
//! Prototype only: bounded native directory observation, not production race equivalence.
//! https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_id_extd_dir_info
#[cfg(windows)]
mod native {
    use dlpad_engine::file_identity::{self, NativeFileIdentity};
    use std::{
        fs::{self, File, OpenOptions},
        io::{self, Write},
        mem::{offset_of, size_of},
        os::windows::{
            fs::{MetadataExt, OpenOptionsExt},
            io::AsRawHandle,
        },
        time::Instant,
    };
    use windows_sys::Win32::{
        Foundation::{ERROR_NO_MORE_FILES, FILETIME},
        Storage::FileSystem::*,
        System::Threading::{GetCurrentProcess, GetProcessTimes},
    };
    const BUFFER: usize = 65536;
    const COUNT: usize = 10000;
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
        size: u64,
        id: [u8; 16],
        attributes: u32,
        modified: i64,
        next: usize,
    }
    fn parse(bytes: &[u8]) -> io::Result<Entry> {
        if bytes.len() < HEADER {
            return Err(bad());
        }
        let next = read::<u32>(bytes, offset_of!(FILE_ID_EXTD_DIR_INFO, NextEntryOffset))? as usize;
        let length =
            read::<u32>(bytes, offset_of!(FILE_ID_EXTD_DIR_INFO, FileNameLength))? as usize;
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
            size: size as u64,
            id: read(bytes, offset_of!(FILE_ID_EXTD_DIR_INFO, FileId))?,
            attributes: read(bytes, offset_of!(FILE_ID_EXTD_DIR_INFO, FileAttributes))?,
            modified: read(bytes, offset_of!(FILE_ID_EXTD_DIR_INFO, LastWriteTime))?,
            next,
        })
    }
    // Fixed owned fixture membership, not arbitrary directory production policy.
    fn mark_name(name: &str, seen: &mut [u8; 1250]) -> io::Result<()> {
        let bytes = name.as_bytes();
        if bytes.len() != 19
            || &bytes[..10] != b"synthetic-"
            || &bytes[15..] != b".mp4"
            || !bytes[10..15].iter().all(u8::is_ascii_digit)
        {
            return Err(bad());
        }
        let index = bytes[10..15]
            .iter()
            .fold(0usize, |n, digit| n * 10 + usize::from(digit - b'0'));
        if index >= COUNT {
            return Err(bad());
        }
        let bit = 1u8 << (index % 8);
        if seen[index / 8] & bit != 0 {
            return Err(bad());
        }
        seen[index / 8] |= bit;
        Ok(())
    }
    fn directory_current(
        file: &File,
        path: &std::path::Path,
        identity: &NativeFileIdentity,
    ) -> io::Result<()> {
        if file_identity::from_file(file)? != *identity
            || file_identity::final_path(file)? != fs::canonicalize(path)?
        {
            return Err(bad());
        }
        Ok(())
    }
    fn time(t: FILETIME) -> u64 {
        (u64::from(t.dwHighDateTime) << 32) | u64::from(t.dwLowDateTime)
    }
    fn cpu() -> io::Result<(u64, u64, u64)> {
        let (mut c, mut e, mut k, mut u) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        // SAFETY: original process pseudo handle with aligned initialized outputs.
        if unsafe { GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut k, &mut u) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok((time(c), time(k), time(u)))
    }
    fn nanos(t: Instant) -> u64 {
        u64::try_from(t.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }
    fn emit(v: serde_json::Value) -> io::Result<()> {
        let bytes = serde_json::to_vec(&v)?;
        if bytes.len() > 16384 {
            return Err(bad());
        }
        let mut out = io::stdout().lock();
        out.write_all(&bytes)?;
        out.write_all(b"\n")?;
        out.flush()
    }
    pub fn run() -> io::Result<()> {
        let mut args = std::env::args_os().skip(1);
        if args.next() != Some("--fixed-directory-cost-experiment".into()) || args.next().is_some()
        {
            return Err(bad());
        }
        let deadline = Instant::now();
        let check = || {
            if deadline.elapsed().as_secs() >= 120 {
                Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "directory experiment deadline",
                ))
            } else {
                Ok(())
            }
        };
        let root = tempfile::Builder::new()
            .prefix("dlpad-directory-cost-owned-")
            .tempdir()?
            .keep();
        let root = fs::canonicalize(root)?;
        emit(
            serde_json::json!({"phase":"created","owned_fixture_root":root,"synthetic_only":true}),
        )?;
        fs::write(root.join("sentinel"), b"preserve")?;
        let directory = root.join("synthetic");
        fs::create_dir(&directory)?;
        for i in 0..COUNT {
            check()?;
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(directory.join(format!("synthetic-{i:05}.mp4")))?;
            f.write_all(b"abc")?;
        }
        let original = OpenOptions::new()
            .read(true)
            .access_mode(FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
            .open(&directory)?;
        let root_id = file_identity::from_file(&original)?;
        if file_identity::final_path(&original)? != fs::canonicalize(&directory)? {
            return Err(bad());
        }
        let mut attrs = FILE_ATTRIBUTE_TAG_INFO::default();
        // SAFETY: retained original directory handle and exact structure output.
        if unsafe {
            GetFileInformationByHandleEx(
                original.as_raw_handle(),
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
        let mut storage = vec![0_u64; BUFFER / 8];
        let mut rounds = Vec::with_capacity(3);
        let mut previous = cpu()?;
        for round in 0..3 {
            let before = cpu()?;
            if before.0 != previous.0 || before.1 < previous.1 || before.2 < previous.2 {
                return Err(bad());
            }
            let wall = Instant::now();
            let mut seen = [0u8; 1250];
            let (mut count, mut batches, mut cross, mut enum_ns, mut cross_ns) =
                (0usize, 0usize, 0usize, 0u64, 0u64);
            loop {
                check()?;
                batches += 1;
                if batches > 1000 {
                    return Err(bad());
                }
                directory_current(&original, &directory, &root_id)?;
                storage.fill(0);
                let t = Instant::now();
                // SAFETY: aligned owned fixed64KiB output, retained directory HANDLE.
                let success = unsafe {
                    GetFileInformationByHandleEx(
                        original.as_raw_handle(),
                        if batches == 1 {
                            FileIdExtdDirectoryRestartInfo
                        } else {
                            FileIdExtdDirectoryInfo
                        },
                        storage.as_mut_ptr().cast(),
                        BUFFER as u32,
                    )
                };
                enum_ns = enum_ns.saturating_add(nanos(t));
                let failed_error = (success == 0).then(io::Error::last_os_error);
                directory_current(&original, &directory, &root_id)?;
                if let Some(error) = failed_error {
                    if error.raw_os_error() == Some(ERROR_NO_MORE_FILES as i32) {
                        break;
                    }
                    return Err(error);
                }
                // SAFETY: all fixed buffer bytes initialized above and API does not escape it.
                let bytes =
                    unsafe { std::slice::from_raw_parts(storage.as_ptr().cast::<u8>(), BUFFER) };
                let mut offset = 0;
                loop {
                    let t = Instant::now();
                    let entry = parse(&bytes[offset..])?;
                    enum_ns = enum_ns.saturating_add(nanos(t));
                    if entry.name != "." && entry.name != ".." {
                        mark_name(&entry.name, &mut seen)?;
                        count += 1;
                        if count > 20000
                            || entry.attributes
                                & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT)
                                != 0
                            || entry.size != 3
                            || entry.id == [0; 16]
                        {
                            return Err(bad());
                        }
                        if cross < 128 {
                            let t = Instant::now();
                            let path = directory.join(&entry.name);
                            let f = file_identity::open_regular_no_reparse(&path)?;
                            if file_identity::final_path(&f)? != fs::canonicalize(&path)?
                                || f.metadata()?.len() != entry.size
                                || entry.modified < 0
                                || f.metadata()?.last_write_time() != entry.modified as u64
                            {
                                return Err(bad());
                            }
                            let NativeFileIdentity::Windows {
                                volume_serial,
                                file_id,
                            } = file_identity::from_file(&f)?
                            else {
                                return Err(bad());
                            };
                            let NativeFileIdentity::Windows {
                                volume_serial: root_volume,
                                ..
                            } = root_id
                            else {
                                return Err(bad());
                            };
                            if file_id != entry.id || volume_serial != root_volume {
                                return Err(bad());
                            }
                            cross += 1;
                            cross_ns = cross_ns.saturating_add(nanos(t));
                        }
                    }
                    if entry.next == 0 {
                        break;
                    }
                    offset = offset.checked_add(entry.next).ok_or_else(bad)?;
                }
            }
            if seen.iter().any(|byte| *byte != 255)
                || count != COUNT
                || cross != 128
                || file_identity::from_file(&original)? != root_id
            {
                return Err(bad());
            }
            let after = cpu()?;
            if after.0 != before.0 {
                return Err(bad());
            }
            let k = after.1.checked_sub(before.1).ok_or_else(bad)?;
            let u = after.2.checked_sub(before.2).ok_or_else(bad)?;
            previous = after;
            rounds.push(serde_json::json!({"round":round,"entries":count,"batches":batches,"crosscheck_count":cross,"enum_api_and_parse_wall_ns":enum_ns,"crosscheck_wall_ns":cross_ns,"whole_wall_ns":nanos(wall),"kernel_100ns":k,"user_100ns":u,"whole_cpu_100ns":k.checked_add(u).ok_or_else(bad)?,"creation_filetime":before.0}));
        }
        if fs::read(root.join("sentinel"))? != b"preserve" {
            return Err(bad());
        }
        emit(
            serde_json::json!({"scope":"bounded directory native prototype; not production namespace/race equivalence; no fallback","owned_fixture_root":root,"synthetic_only":true,"rounds":rounds,"sentinel_preserved":true}),
        )
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        fn valid() -> Vec<u8> {
            let mut b = vec![0; HEADER + 8];
            b[offset_of!(FILE_ID_EXTD_DIR_INFO, FileNameLength)..][..4]
                .copy_from_slice(&2u32.to_ne_bytes());
            b[HEADER..][..2].copy_from_slice(&(b'a' as u16).to_ne_bytes());
            b
        }
        #[test]
        fn exact_membership_rejects_duplicates_and_bad_names_and_detects_missing() {
            let mut seen = [0u8; 1250];
            for bad_name in [
                "synthetic-10000.mp4",
                "synthetic-0000.mp4",
                "synthetic-00000.mp3",
                "synthetic-00a00.mp4",
                "foreign-00000.mp4",
                "synthetic-00000.mp4x",
            ] {
                assert!(mark_name(bad_name, &mut seen).is_err());
            }
            mark_name("synthetic-00000.mp4", &mut seen).unwrap();
            assert!(mark_name("synthetic-00000.mp4", &mut seen).is_err());
            assert!(seen.iter().any(|byte| *byte != 255));
            for index in 1..COUNT {
                mark_name(&format!("synthetic-{index:05}.mp4"), &mut seen).unwrap();
            }
            assert!(seen.iter().all(|byte| *byte == 255));
        }
        #[test]
        fn bounded_parser_accepts_one_name_and_rejects_malformed_records() {
            assert_eq!(parse(&valid()).unwrap().name, "a");
            assert!(parse(&vec![0; HEADER - 1]).is_err());
            for length in [0u32, 1, 512, u32::MAX] {
                let mut b = valid();
                b[offset_of!(FILE_ID_EXTD_DIR_INFO, FileNameLength)..][..4]
                    .copy_from_slice(&length.to_ne_bytes());
                assert!(parse(&b).is_err());
            }
            for offset in [1u32, 8, 1048576] {
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
    }
}
fn main() {
    #[cfg(windows)]
    if let Err(e) = native::run() {
        eprintln!("directory identity experiment failed: {:?}", e.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("directory identity experiment unsupported");
        std::process::exit(2);
    }
}
