//! Worker-only selected runtime proof. No source mutation or cleanup authority.
use crate::{
    file_identity::{self, NativeFileIdentity},
    process::{self, ProcessOutcome, ProcessSpec},
};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    path::{Component, Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
const LIMIT: u64 = 256 * 1024 * 1024;
const BUDGET: Duration = Duration::from_secs(10);
struct Pin {
    path: PathBuf,
    file: File,
    id: NativeFileIdentity,
}
pub(crate) struct SelectedDeno {
    pub path: PathBuf,
    pub version: String,
    file: File,
    id: NativeFileIdentity,
    hash: [u8; 32],
    ancestors: Vec<Pin>,
}
fn error(kind: io::ErrorKind) -> io::Error {
    io::Error::new(kind, "selected Deno verification failed")
}
fn check(cancel: &Arc<AtomicBool>, deadline: Instant) -> io::Result<Duration> {
    if cancel.load(Ordering::Acquire) {
        return Err(error(io::ErrorKind::Interrupted));
    }
    deadline
        .checked_duration_since(Instant::now())
        .filter(|d| !d.is_zero())
        .ok_or_else(|| error(io::ErrorKind::TimedOut))
}
fn path_policy(path: &Path) -> io::Result<()> {
    if !path.is_absolute()
        || path.components().any(|c| matches!(c, Component::ParentDir))
        || path.as_os_str().len() > 32766
    {
        return Err(error(io::ErrorKind::InvalidInput));
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        if path.as_os_str().encode_wide().any(|u| u == 0) {
            return Err(error(io::ErrorKind::InvalidInput));
        }
    }
    let mut current = PathBuf::new();
    let mut count = 0;
    for part in path.components() {
        current.push(part.as_os_str());
        if matches!(part, Component::Prefix(_)) {
            continue;
        }
        count += 1;
        if count > 64 {
            return Err(error(io::ErrorKind::InvalidInput));
        }
        let meta = fs::symlink_metadata(&current)?;
        if meta.file_type().is_symlink() {
            return Err(error(io::ErrorKind::InvalidInput));
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if meta.file_attributes() & 0x400 != 0 {
                return Err(error(io::ErrorKind::InvalidInput));
            }
        }
    }
    Ok(())
}
fn open(path: &Path, directory: bool) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(
            libc::O_NOFOLLOW
                | libc::O_CLOEXEC
                | if directory {
                    libc::O_DIRECTORY
                } else {
                    libc::O_NONBLOCK
                },
        );
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::*;
        options.custom_flags(
            FILE_FLAG_OPEN_REPARSE_POINT
                | if directory {
                    FILE_FLAG_BACKUP_SEMANTICS
                } else {
                    0
                },
        );
        if directory {
            options.access_mode(FILE_READ_ATTRIBUTES);
        }
    }
    let file = options.open(path)?;
    let meta = file.metadata()?;
    if (directory && !meta.is_dir()) || (!directory && !meta.is_file()) {
        return Err(error(io::ErrorKind::InvalidInput));
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Storage::FileSystem::*;
        let mut tag = FILE_ATTRIBUTE_TAG_INFO::default();
        if unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileAttributeTagInfo,
                (&mut tag as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
                std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if tag.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
            || (tag.FileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0) != directory
        {
            return Err(error(io::ErrorKind::InvalidInput));
        }
    }
    Ok(file)
}
fn pins(path: &Path) -> io::Result<Vec<Pin>> {
    let parent = path
        .parent()
        .ok_or_else(|| error(io::ErrorKind::InvalidInput))?;
    let mut current = PathBuf::new();
    let mut pins = Vec::with_capacity(64);
    for part in parent.components() {
        current.push(part.as_os_str());
        if matches!(part, Component::Prefix(_)) {
            continue;
        }
        let file = open(&current, true)?;
        let id = file_identity::from_file(&file)?;
        pins.push(Pin {
            path: current.clone(),
            file,
            id,
        });
    }
    Ok(pins)
}
fn hash(file: &File, cancel: &Arc<AtomicBool>, deadline: Instant) -> io::Result<[u8; 32]> {
    if file.metadata()?.len() > LIMIT {
        return Err(error(io::ErrorKind::InvalidData));
    }
    let mut source = file;
    source.seek(SeekFrom::Start(0))?;
    let mut hasher = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        check(cancel, deadline)?;
        let n = source.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        bytes = bytes
            .checked_add(n as u64)
            .ok_or_else(|| error(io::ErrorKind::InvalidData))?;
        if bytes > LIMIT {
            return Err(error(io::ErrorKind::InvalidData));
        }
        hasher.update(&buffer[..n]);
    }
    if bytes == 0 || bytes != file.metadata()?.len() {
        return Err(error(io::ErrorKind::InvalidData));
    }
    Ok(hasher.finalize().into())
}
fn target() -> io::Result<&'static str> {
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Ok("x86_64-unknown-linux-gnu")
    } else if cfg!(all(windows, target_arch = "x86_64")) {
        Ok("x86_64-pc-windows-msvc")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Ok("x86_64-apple-darwin")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Ok("aarch64-apple-darwin")
    } else {
        Err(error(io::ErrorKind::Unsupported))
    }
}
impl SelectedDeno {
    fn verify_until(&self, cancel: &Arc<AtomicBool>, deadline: Instant) -> io::Result<()> {
        check(cancel, deadline)?;
        path_policy(&self.path)?;
        for pin in &self.ancestors {
            check(cancel, deadline)?;
            let current = open(&pin.path, true)?;
            if file_identity::from_file(&pin.file)? != pin.id
                || file_identity::from_file(&current)? != pin.id
            {
                return Err(error(io::ErrorKind::InvalidData));
            }
        }
        let current = open(&self.path, false)?;
        if file_identity::from_file(&self.file)? != self.id
            || file_identity::from_file(&current)? != self.id
            || hash(&self.file, cancel, deadline)? != self.hash
            || hash(&current, cancel, deadline)? != self.hash
        {
            return Err(error(io::ErrorKind::InvalidData));
        }
        #[cfg(windows)]
        file_identity::ensure_regular_under(
            &self
                .ancestors
                .last()
                .ok_or_else(|| error(io::ErrorKind::InvalidInput))?
                .file,
            &current,
        )?;
        check(cancel, deadline)?;
        Ok(())
    }
    pub(crate) fn verify(&self, cancel: &Arc<AtomicBool>) -> io::Result<()> {
        self.verify_until(cancel, Instant::now() + BUDGET)
    }
}
#[cfg(all(test, target_os = "linux", target_arch = "x86_64"))]
pub(crate) fn inspect(path: &Path, cancel: &Arc<AtomicBool>) -> io::Result<SelectedDeno> {
    inspect_with_managed(path, cancel, None)
}
pub(crate) fn inspect_with_managed(
    path: &Path,
    cancel: &Arc<AtomicBool>,
    managed: Option<&crate::tools::ManagedExecution>,
    #[cfg(windows)] runtime_environment: Option<
        &crate::runtime_environment::WindowsManagedDenoEnvironment,
    >,
) -> io::Result<SelectedDeno> {
    let deadline = Instant::now() + BUDGET;
    check(cancel, deadline)?;
    path_policy(path)?;
    let path = path.canonicalize()?;
    path_policy(&path)?;
    #[cfg(windows)]
    if let Some(policy) = runtime_environment {
        policy
            .validate()
            .map_err(|_| error(io::ErrorKind::InvalidInput))?;
        if managed.is_none() || policy.selected_executable() != Some(path.as_path()) {
            return Err(error(io::ErrorKind::InvalidInput));
        }
    }
    let ancestors = pins(&path)?;
    let file = open(&path, false)?;
    let id = file_identity::from_file(&file)?;
    #[cfg(windows)]
    file_identity::ensure_regular_under(
        &ancestors
            .last()
            .ok_or_else(|| error(io::ErrorKind::InvalidInput))?
            .file,
        &file,
    )?;
    let digest = hash(&file, cancel, deadline)?;
    let mut selected = SelectedDeno {
        path,
        version: String::new(),
        file,
        id,
        hash: digest,
        ancestors,
    };
    selected.verify_until(cancel, deadline)?;
    let spec = ProcessSpec {
        #[cfg(windows)]
        runtime_environment: runtime_environment.cloned(),
        executable: selected.path.clone(),
        args: vec!["--version".into()],
        timeout: check(cancel, deadline)?,
        max_log_bytes: 4096,
    };
    #[cfg(windows)]
    let result = if let Some(managed) = managed {
        managed.run_tool(
            &spec,
            cancel,
            true,
            None,
            crate::tools::ManagedReaderStage::VersionProbe,
        )?
    } else {
        process::run_metadata(&spec, cancel)?
    };
    #[cfg(not(windows))]
    let result = {
        let _ = managed;
        process::run_metadata(&spec, cancel)?
    };
    match result.execution.outcome {
        ProcessOutcome::Cancelled => return Err(error(io::ErrorKind::Interrupted)),
        ProcessOutcome::TimedOut => return Err(error(io::ErrorKind::TimedOut)),
        ProcessOutcome::OutputLimit => return Err(error(io::ErrorKind::InvalidData)),
        ProcessOutcome::Exited => {}
    }
    if result.execution.exit_code != Some(0)
        || result.execution.truncated
        || result.stdout.len() > 4096
    {
        return Err(error(io::ErrorKind::InvalidData));
    }
    let first = result
        .stdout
        .split(|b| *b == b'\n')
        .next()
        .ok_or_else(|| error(io::ErrorKind::InvalidData))?;
    let first = std::str::from_utf8(first).map_err(|_| error(io::ErrorKind::InvalidData))?;
    let version = first
        .strip_prefix("deno ")
        .and_then(|v| v.split_once(' ').map(|(token, _)| token))
        .ok_or_else(|| error(io::ErrorKind::InvalidData))?;
    if version.is_empty()
        || version.len() > 64
        || version.split('.').count() != 3
        || version
            .split('.')
            .any(|v| v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()))
        || !crate::tools::verify_deno_output(&result.stdout, version, target()?)
    {
        return Err(error(io::ErrorKind::InvalidData));
    }
    selected.verify_until(cancel, deadline)?;
    selected.version = version.to_owned();
    Ok(selected)
}

#[cfg(all(test, target_os = "linux", target_arch = "x86_64"))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    fn fixture(body: &str) -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("synthetic deno 한글");
        fs::write(&path,format!("#!/usr/bin/python3\nimport sys, pathlib, time\nassert sys.argv[1:] == ['--version']\n{body}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        (temp, path)
    }
    const OUTPUT: &str = "print('deno 2.9.7 (stable, release, x86_64-unknown-linux-gnu)\\nv8 15.0.245.2-rusty\\ntypescript 6.0.3')";
    #[test]
    fn native_version_and_original_identity_recheck() {
        let (_temp, path) = fixture(OUTPUT);
        let cancel = Arc::new(AtomicBool::new(false));
        let selected = inspect(&path, &cancel).unwrap();
        assert_eq!(selected.version, "2.9.7");
        selected.verify(&cancel).unwrap();
        let original = fs::read(&path).unwrap();
        let replacement = path.with_extension("replacement");
        fs::write(&replacement, &original).unwrap();
        fs::set_permissions(&replacement, fs::Permissions::from_mode(0o700)).unwrap();
        fs::rename(replacement, &path).unwrap();
        assert_eq!(
            selected.verify(&cancel).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(fs::read(path).unwrap(), original);
    }
    #[test]
    fn malformed_nonzero_self_mutation_and_unsafe_path_fail_closed() {
        for body in ["print('deno 2.9.bad (stable, release, x86_64-unknown-linux-gnu)\\nv8 x\\ntypescript x')".to_owned(),format!("{OUTPUT}\nsys.exit(7)"),format!("{OUTPUT}\np = pathlib.Path(__file__)\np.write_bytes(p.read_bytes() + b'\\n# owned synthetic mutation\\n')")] {
            let (_temp,path)=fixture(&body);let error=match inspect(&path,&Arc::new(AtomicBool::new(false))){Ok(_)=>panic!("invalid runtime accepted"),Err(error)=>error};assert_eq!(error.kind(),io::ErrorKind::InvalidData);
        }
        let (temp, path) = fixture(OUTPUT);
        let alias = temp.path().join("alias");
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        assert!(
            matches!(inspect(&alias,&Arc::new(AtomicBool::new(false))),Err(error) if error.kind()==io::ErrorKind::InvalidInput)
        );
        assert!(path.exists());
    }
    #[test]
    fn cancellation_during_owned_native_version_preserves_source() {
        let (_temp, path) = fixture(
            "pathlib.Path(__file__ + '.ready').write_text('synthetic ready')\ntime.sleep(30)",
        );
        let before = fs::read(&path).unwrap();
        let ready = PathBuf::from(format!("{}.ready", path.to_str().unwrap()));
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let trigger = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !ready.exists() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
            let observed = ready.exists();
            flag.store(true, Ordering::Release);
            observed
        });
        let result = inspect(&path, &cancel);
        assert!(trigger.join().unwrap());
        assert!(matches!(result,Err(error) if error.kind()==io::ErrorKind::Interrupted));
        assert_eq!(fs::read(path).unwrap(), before);
    }
}
