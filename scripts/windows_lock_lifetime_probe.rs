//! Synthetic native experiment. No product recovery/deletion authority.
#[cfg(windows)]
mod native {
    use std::{
        ffi::c_void,
        fs::{self, File, OpenOptions, TryLockError},
        io::{self, Write},
        os::windows::io::{AsRawHandle, FromRawHandle},
        path::{Path, PathBuf},
        process::{Child, Command, Stdio},
        thread,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };
    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq)]
    struct Identity {
        volume: u64,
        id: [u8; 16],
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetStdHandle(which: u32) -> *mut c_void;
        fn GetFileInformationByHandleEx(
            handle: *mut c_void,
            class: u32,
            data: *mut c_void,
            size: u32,
        ) -> i32;
    }
    fn identity(file: &File) -> io::Result<Identity> {
        let mut value = Identity {
            volume: 0,
            id: [0; 16],
        };
        if unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                18,
                (&mut value as *mut Identity).cast(),
                24,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if value.id == [0; 16] {
            return Err(io::Error::other("native file identity unavailable"));
        }
        Ok(value)
    }
    fn lock(file: &File) -> io::Result<&'static str> {
        match file.try_lock() {
            Ok(()) => Ok("acquired"),
            Err(TryLockError::WouldBlock) => Ok("would-block"),
            Err(TryLockError::Error(error)) => Err(error),
        }
    }
    fn marker(path: &Path, bytes: &[u8]) -> io::Result<()> {
        let mut f = OpenOptions::new().write(true).create_new(true).open(path)?;
        f.write_all(bytes)?;
        f.sync_all()
    }
    fn wait_marker(
        path: &Path,
        expected: &[u8],
        deadline: Instant,
        child: Option<&mut Child>,
    ) -> io::Result<()> {
        let mut child = child;
        loop {
            match fs::read(path) {
                Ok(bytes) if bytes == expected => return Ok(()),
                Ok(_) => return Err(io::Error::other("invalid fixture marker")),
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            }
            if let Some(c) = child.as_mut() {
                if c.try_wait()?.is_some() {
                    return Err(io::Error::other("owned child exited before marker"));
                }
            }
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "fixture marker deadline",
                ));
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn child(path: PathBuf) -> io::Result<()> {
        let raw = unsafe { GetStdHandle((-10i32) as u32) };
        if raw.is_null() || raw as isize == -1 {
            return Err(io::Error::other("inherited handle absent"));
        }
        let held = unsafe { File::from_raw_handle(raw) };
        let original = identity(&held)?;
        let mut bytes = Vec::with_capacity(24);
        bytes.extend(original.volume.to_le_bytes());
        bytes.extend(original.id);
        marker(&path.join("ready"), &bytes)?;
        wait_marker(
            &path.join("release"),
            b"R",
            Instant::now() + Duration::from_secs(5),
            None,
        )?;
        drop(held);
        Ok(())
    }
    struct OwnedChild(Option<Child>);
    impl OwnedChild {
        fn reap(&mut self) -> io::Result<()> {
            let deadline = Instant::now() + Duration::from_secs(3);
            loop {
                let c = self
                    .0
                    .as_mut()
                    .ok_or_else(|| io::Error::other("child missing"))?;
                if let Some(status) = c.try_wait()? {
                    self.0 = None;
                    return if status.success() {
                        Ok(())
                    } else {
                        Err(io::Error::other("owned child failed"))
                    };
                }
                if Instant::now() >= deadline {
                    c.kill()?;
                    let reap_deadline = Instant::now() + Duration::from_secs(2);
                    while Instant::now() < reap_deadline {
                        if c.try_wait()?.is_some() {
                            self.0 = None;
                            break;
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "owned child deadline",
                    ));
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            if let Some(mut c) = self.0.take() {
                let _ = c.kill();
                let deadline = Instant::now() + Duration::from_secs(2);
                while Instant::now() < deadline {
                    if matches!(c.try_wait(), Ok(Some(_))) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }
    struct Fixture(PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    pub fn run() -> io::Result<()> {
        let mut args = std::env::args_os();
        args.next();
        if args.next().as_deref() == Some(std::ffi::OsStr::new("--child")) {
            return child(PathBuf::from(
                args.next()
                    .ok_or_else(|| io::Error::other("fixture path absent"))?,
            ));
        }
        let started = Instant::now();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| io::Error::other("clock unavailable"))?
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("dlpad-lock-fixture-{}-{nonce}", std::process::id()));
        fs::create_dir(&path)?;
        let fixture = Fixture(path);
        let file_path = fixture.0.join("synthetic-lock");
        let original = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&file_path)?;
        if lock(&original)? != "acquired" {
            return Err(io::Error::other("initial lock unavailable"));
        }
        let expected = identity(&original)?;
        let contender = OpenOptions::new().read(true).write(true).open(&file_path)?;
        let before = lock(&contender)?;
        println!("{{\"phase\":\"original-held\",\"lock\":\"{before}\"}}");
        if before != "would-block" {
            return Err(io::Error::other("initial exclusive lock not observed"));
        }
        drop(contender);
        let duplicate = original.try_clone()?;
        let mut command = Command::new(std::env::current_exe()?);
        command
            .arg("--child")
            .arg(&fixture.0)
            .stdin(Stdio::from(duplicate))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        let child = command.spawn()?;
        drop(command);
        let mut owned = OwnedChild(Some(child));
        let mut ready = Vec::new();
        ready.extend(expected.volume.to_le_bytes());
        ready.extend(expected.id);
        wait_marker(
            &fixture.0.join("ready"),
            &ready,
            Instant::now() + Duration::from_secs(2),
            owned.0.as_mut(),
        )?;
        println!(
            "{{\"phase\":\"child-ready\",\"native_identity_matches\":true,\"owned_child_pid\":{}}}",
            owned.0.as_ref().unwrap().id()
        );
        drop(original);
        let contender = OpenOptions::new().read(true).write(true).open(&file_path)?;
        let after = lock(&contender)?;
        let live = owned.0.as_mut().unwrap().try_wait()?.is_none();
        if !live {
            return Err(io::Error::other("child not live at lock observation"));
        }
        println!(
            "{{\"phase\":\"parent-original-closed-child-held\",\"child_live\":true,\"lock\":\"{after}\"}}"
        );
        drop(contender);
        marker(&fixture.0.join("release"), b"R")?;
        owned.reap()?;
        let contender = OpenOptions::new().read(true).write(true).open(&file_path)?;
        let terminal = lock(&contender)?;
        if terminal != "acquired" {
            return Err(io::Error::other("terminal lock unavailable"));
        }
        drop(contender);
        println!(
            "{{\"phase\":\"terminal\",\"lock\":\"{terminal}\",\"duplicate_preserves_lock\":{},\"elapsed_ms\":{},\"authority_eligible\":false}}",
            after == "would-block",
            started.elapsed().as_millis()
        );
        drop(fixture);
        Ok(())
    }
}
fn main() {
    #[cfg(windows)]
    if let Err(error) = native::run() {
        eprintln!(
            "lock-probe error kind={:?} os={:?}",
            error.kind(),
            error.raw_os_error()
        );
        std::process::exit(2)
    }
    #[cfg(not(windows))]
    std::process::exit(2);
}
