//! Fresh regular-file metadata. Directory hard-link entries may contain stale
//! sizes/times; they supply enumeration identity, never publication metadata.
//! Basic-by-name is optional (Windows 11 24H2); older systems query one owned
//! no-reparse HANDLE. Path queries are snapshots, not file lifetime pins.
use super::{NativeFileIdentity, bad, file_identity};
use std::{
    ffi::c_void,
    io,
    mem::{align_of, size_of},
    os::windows::{ffi::OsStrExt, io::AsRawHandle},
    path::{Component, Path, Prefix},
};
use windows_sys::Win32::Storage::FileSystem::*;

type Module = *mut c_void;
type Query = unsafe extern "system" fn(*const u16, i32, *mut c_void, u32) -> i32;
#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryExW(name: *const u16, file: *mut c_void, flags: u32) -> Module;
    fn GetProcAddress(module: Module, name: *const u8) -> *mut c_void;
    fn FreeLibrary(module: Module) -> i32;
}
#[repr(C)]
#[derive(Default)]
struct Basic {
    file_id: i64,
    creation: i64,
    access: i64,
    write: i64,
    change: i64,
    allocation: i64,
    end: i64,
    attrs: u32,
    reparse: u32,
    links: u32,
    device: u32,
    characteristics: u32,
    reserved: u32,
    volume: i64,
    id: [u8; 16],
}
const _: () = assert!(size_of::<Basic>() == 104 && align_of::<Basic>() == 8);

pub(super) struct Metadata {
    pub(super) size: u64,
    pub(super) modified: i64,
}
pub(super) struct Api {
    module: Module,
    query: Query,
    path: Vec<u16>,
}
pub(super) fn unsupported(error: &io::Error) -> bool {
    matches!(error.raw_os_error(), Some(1 | 50 | 87 | 120))
}
fn wide_into(path: &Path, result: &mut Vec<u16>) -> io::Result<()> {
    result.clear();
    let Some(Component::Prefix(prefix)) = path.components().next() else {
        return Err(bad());
    };
    let (lead, skip) = match prefix.kind() {
        Prefix::Disk(_) if path.is_absolute() => ("\\\\?\\", 0),
        Prefix::UNC(_, _) => ("\\\\?\\UNC\\", 2),
        Prefix::VerbatimDisk(_) | Prefix::VerbatimUNC(_, _) => ("", 0),
        _ => return Err(bad()),
    };
    result.extend(lead.encode_utf16());
    // Check before push; never allocate an unbounded path buffer.
    for unit in path.as_os_str().encode_wide().skip(skip) {
        if unit == 0 || result.len() >= 32766 {
            return Err(bad());
        }
        result.push(unit);
    }
    result.push(0);
    Ok(())
}
#[cfg(test)]
fn wide(path: &Path) -> io::Result<Vec<u16>> {
    let mut result = Vec::with_capacity(32767);
    wide_into(path, &mut result)?;
    Ok(result)
}

impl Api {
    pub(super) fn load() -> io::Result<Option<Self>> {
        let name: Vec<u16> = "api-ms-win-core-file-l2-1-4.dll"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        // SAFETY: terminated constant name, null reserved HANDLE, system-only lookup.
        let module = unsafe { LoadLibraryExW(name.as_ptr(), std::ptr::null_mut(), 0x800) };
        if module.is_null() {
            let error = io::Error::last_os_error();
            return if matches!(error.raw_os_error(), Some(126 | 127)) {
                Ok(None)
            } else {
                Err(error)
            };
        }
        // SAFETY: owned live module, terminated exact public export name.
        let address =
            unsafe { GetProcAddress(module, c"GetFileInformationByName".as_ptr().cast()) };
        if address.is_null() {
            let error = io::Error::last_os_error();
            // SAFETY: release exactly the reference acquired above.
            unsafe { FreeLibrary(module) };
            return if error.raw_os_error() == Some(127) {
                Ok(None)
            } else {
                Err(error)
            };
        }
        // SAFETY: SDK WINAPI BOOL(PCWSTR, enum32, PVOID, ULONG32) ABI. Api owns
        // the DLL for the entire function lifetime; the pointer never escapes.
        let query = unsafe { std::mem::transmute::<*mut c_void, Query>(address) };
        Ok(Some(Self {
            module,
            query,
            path: Vec::with_capacity(32767),
        }))
    }
    pub(super) fn observe(
        &mut self,
        path: &Path,
        volume: u64,
        id: [u8; 16],
    ) -> io::Result<Metadata> {
        wide_into(path, &mut self.path)?;
        let mut basic = Basic::default();
        // SAFETY: bounded terminated UTF16 and exact initialized 104-byte SDK output.
        if unsafe {
            (self.query)(
                self.path.as_ptr(),
                3,
                (&mut basic as *mut Basic).cast(),
                size_of::<Basic>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if basic.end < 0
            || basic.attrs
                & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DEVICE)
                != 0
            || basic.reparse != 0
            || basic.links == 0
            || basic.id != id
            || basic.volume as u64 != volume
        {
            return Err(bad());
        }
        Ok(Metadata {
            size: basic.end as u64,
            modified: basic.write,
        })
    }
}
impl Drop for Api {
    fn drop(&mut self) {
        // SAFETY: this Api owns exactly one live loader reference.
        unsafe { FreeLibrary(self.module) };
    }
}
pub(super) fn from_handle(path: &Path, volume: u64, id: [u8; 16]) -> io::Result<Metadata> {
    let file = file_identity::open_regular_no_reparse(path)?;
    if file_identity::from_file(&file)?
        != (NativeFileIdentity::Windows {
            volume_serial: volume,
            file_id: id,
        })
    {
        return Err(bad());
    }
    let mut standard = FILE_STANDARD_INFO::default();
    let mut basic = FILE_BASIC_INFO::default();
    // SAFETY: original live no-reparse regular-file HANDLE and exact initialized outputs.
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileStandardInfo,
            (&mut standard as *mut FILE_STANDARD_INFO).cast(),
            size_of::<FILE_STANDARD_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: same original HANDLE, exact initialized FILE_BASIC_INFO output.
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileBasicInfo,
            (&mut basic as *mut FILE_BASIC_INFO).cast(),
            size_of::<FILE_BASIC_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if standard.EndOfFile < 0
        || standard.Directory
        || basic.FileAttributes
            & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DEVICE)
            != 0
    {
        return Err(bad());
    }
    Ok(Metadata {
        size: standard.EndOfFile as u64,
        modified: basic.LastWriteTime,
    })
}
// Proposed append-only tests inside fresh.rs; not published or executed.
#[cfg(test)]
mod independent_tests {
    use super::*;
    use std::{
        fs,
        io::Write,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    fn anchor(path: &Path) -> (u64, [u8; 16]) {
        let file = file_identity::open_regular_no_reparse(path).unwrap();
        let NativeFileIdentity::Windows {
            volume_serial,
            file_id,
        } = file_identity::from_file(&file).unwrap()
        else {
            panic!("Windows identity required")
        };
        (volume_serial, file_id)
    }
    #[test]
    fn original_handle_metadata_tracks_inside_and_outside_hardlink_writes() {
        for inside in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("선택-root");
            fs::create_dir(&root).unwrap();
            let ordinary = root.join("완료😀.mp4");
            fs::write(&ordinary, b"xxxxxxxxxxxx").unwrap();
            let alias_parent = if inside {
                let p = root.join(".dlpad-partials/job");
                fs::create_dir_all(&p).unwrap();
                p
            } else {
                let p = temp.path().join("outside");
                fs::create_dir(&p).unwrap();
                p
            };
            let alias = alias_parent.join("alias.part");
            fs::hard_link(&ordinary, &alias).unwrap();
            let (volume, id) = anchor(&ordinary);
            let target = SystemTime::now() + Duration::from_secs(3600);
            let mut writer = fs::OpenOptions::new().append(true).open(&alias).unwrap();
            writer.write_all(b"z").unwrap();
            writer.set_modified(target).unwrap();
            writer.sync_all().unwrap();
            drop(writer);
            let result = from_handle(&ordinary, volume, id).unwrap();
            assert_eq!(result.size, 13);
            let unix = result.modified as i128 - 116444736000000000i128;
            let wanted = target.duration_since(UNIX_EPOCH).unwrap().as_nanos() / 100;
            assert!(
                (unix - wanted as i128).abs() < 10000,
                "actual HANDLE timestamp must track future write"
            );
            let mut wrong = id;
            wrong[0] ^= 1;
            assert!(from_handle(&ordinary, volume, wrong).is_err());
            assert!(from_handle(&ordinary, volume.wrapping_add(1), id).is_err());
            // Optional ByName availability/class is separately characterized, not a silent PASS substitute.
        }
    }
    #[test]
    fn bounded_wide_rejects_relative_nul_and_oversize() {
        assert!(wide(Path::new("relative.mp4")).is_err());
        assert!(wide(Path::new("C:relative.mp4")).is_err());
        assert!(wide(Path::new("C:\\bad\0name")).is_err());
        assert!(wide(Path::new(&format!("C:\\{}", "x".repeat(32767)))).is_err());
        assert_eq!(
            *wide(Path::new("C:\\한글😀.mp4")).unwrap().last().unwrap(),
            0
        );
        assert!(from_handle(Path::new("relative.mp4"), 1, [1; 16]).is_err());
    }
}

#[cfg(test)]
mod buffer_tests {
    use super::*;

    #[test]
    fn path_prefixes_and_unicode_reuse_one_buffer() {
        let mut buffer = Vec::with_capacity(32767);
        let pointer = buffer.as_ptr();
        for (input, expected) in [
            (r"D:\한글\file.mp4", r"\\?\D:\한글\file.mp4"),
            (r"\\server\share\한글.mp4", r"\\?\UNC\server\share\한글.mp4"),
            (r"\\?\D:\file.mp4", r"\\?\D:\file.mp4"),
            (
                r"\\?\UNC\server\share\file.mp4",
                r"\\?\UNC\server\share\file.mp4",
            ),
        ] {
            wide_into(Path::new(input), &mut buffer).unwrap();
            let expected: Vec<_> = expected.encode_utf16().chain(Some(0)).collect();
            assert_eq!(buffer, expected);
            assert_eq!(buffer.as_ptr(), pointer);
        }
    }

    #[test]
    fn failed_and_long_paths_do_not_contaminate_next_query() {
        let mut buffer = Vec::with_capacity(32767);
        let valid = format!(r"D:\{}", "a".repeat(32759));
        wide_into(Path::new(&valid), &mut buffer).unwrap();
        assert_eq!(buffer.len(), 32767);
        assert!(wide_into(Path::new(&(valid + "a")), &mut buffer).is_err());
        assert!(wide_into(Path::new("relative"), &mut buffer).is_err());
        assert!(buffer.is_empty());
        assert!(wide_into(Path::new("D:\\file\0suffix"), &mut buffer).is_err());
        wide_into(Path::new(r"D:\ok.mp4"), &mut buffer).unwrap();
        assert_eq!(
            buffer,
            r"\\?\D:\ok.mp4"
                .encode_utf16()
                .chain(Some(0))
                .collect::<Vec<_>>()
        );
    }
}
