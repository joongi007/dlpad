//! Linux-only owned FIFO for FFmpeg machine progress. It never owns media or
//! starts/kills tool processes. Blocking lifetime methods belong on a worker.
use crate::metrics::ProgressFeed;
use std::{
    ffi::{CString, OsStr, OsString},
    fs::{self, DirBuilder, File, OpenOptions},
    io::{self, Read},
    os::{
        fd::AsRawFd,
        unix::{
            ffi::{OsStrExt, OsStringExt},
            fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt},
        },
    },
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

static NEXT_ENDPOINT: AtomicU64 = AtomicU64::new(0);
const POLL_MS: i32 = 20;
const DRAIN_BYTES: usize = 256 * 1024;
const DRAIN_GRACE: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Identity {
    dev: u64,
    ino: u64,
}
fn identity(path: &Path) -> io::Result<Identity> {
    let m = fs::symlink_metadata(path)?;
    Ok(Identity {
        dev: m.dev(),
        ino: m.ino(),
    })
}

/// Per-job user-private telemetry, not artifact/completion evidence.
pub struct ProcessingEndpoint {
    directory: PathBuf,
    directory_identity: Identity,
    path: PathBuf,
    fifo_identity: Identity,
    stop: Arc<AtomicBool>,
    reader: Option<JoinHandle<io::Result<()>>>,
    feed: ProgressFeed,
}
impl ProcessingEndpoint {
    pub fn new(job_owned_directory: &Path, feed: ProgressFeed) -> io::Result<Self> {
        let parent = fs::symlink_metadata(job_owned_directory)?;
        if !parent.file_type().is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "processing parent is not a real directory",
            ));
        }
        let root = fs::canonicalize(job_owned_directory)?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        let directory = root.join(format!(
            ".progress-{}-{stamp}-{}",
            std::process::id(),
            NEXT_ENDPOINT.fetch_add(1, Ordering::Relaxed)
        ));
        DirBuilder::new().mode(0o700).create(&directory)?;
        let directory_identity = identity(&directory)?;
        let path = directory.join("ffmpeg.fifo");
        let fifo_path = CString::new(path.as_os_str().as_bytes()).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "processing path contains NUL")
        })?;
        // SAFETY: NUL-terminated owned path, no replacement or existing node allowed.
        if unsafe { libc::mkfifo(fifo_path.as_ptr(), 0o600) } != 0 {
            let error = io::Error::last_os_error();
            if identity(&directory).ok() == Some(directory_identity) {
                let _ = fs::remove_dir(&directory);
            }
            return Err(error);
        }
        let fifo_identity = identity(&path)?;
        let mut endpoint = Self {
            directory,
            directory_identity,
            path,
            fifo_identity,
            stop: Arc::new(AtomicBool::new(false)),
            reader: None,
            feed,
        };
        // The provisional owner drops with identity-checked cleanup on all
        // subsequent constructor failures, preserving any foreign replacement.
        let pipe = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .open(&endpoint.path)?;
        let m = pipe.metadata()?;
        if !m.file_type().is_fifo()
            || (Identity {
                dev: m.dev(),
                ino: m.ino(),
            }) != endpoint.fifo_identity
        {
            return Err(io::Error::other("processing FIFO identity changed"));
        }
        let thread_stop = endpoint.stop.clone();
        let thread_feed = endpoint.feed.clone();
        endpoint.feed.begin_dedicated_processing();
        match thread::Builder::new()
            .name("dlpad-processing".into())
            .spawn(move || read_progress(pipe, thread_stop, thread_feed))
        {
            Ok(reader) => endpoint.reader = Some(reader),
            Err(error) => {
                endpoint.feed.finish_dedicated_processing();
                return Err(error);
            }
        }
        Ok(endpoint)
    }
    pub fn argument(&self) -> &OsStr {
        self.path.as_os_str()
    }
    /// yt-dlp parses its one argv value with shlex, without invoking a shell.
    /// Preserve Unix raw filename bytes and quote embedded apostrophes exactly.
    pub fn postprocessor_argument(&self) -> OsString {
        let mut bytes = b"ffmpeg:-progress '".to_vec();
        for byte in self.path.as_os_str().as_bytes() {
            if *byte == b'\'' {
                bytes.extend_from_slice(b"'\\''");
            } else {
                bytes.push(*byte);
            }
        }
        bytes.extend_from_slice(b"' -nostats");
        OsString::from_vec(bytes)
    }
    /// Stop after bounded pending drain and remove only matching owned nodes.
    pub fn finish(&mut self) -> io::Result<()> {
        self.stop.store(true, Ordering::Release);
        let result = if let Some(reader) = self.reader.take() {
            reader
                .join()
                .map_err(|_| io::Error::other("processing reader panicked"))
                .and_then(|r| r)
        } else {
            Ok(())
        };
        // Clear active ownership even when the reader panicked before its finalizer.
        self.feed.finish_dedicated_processing();
        let cleanup = self.cleanup();
        if result.is_err() || cleanup.is_err() {
            self.feed.invalidate_processing();
        }
        result.and(cleanup)
    }
    fn cleanup(&self) -> io::Result<()> {
        match identity(&self.directory) {
            Ok(actual) if actual == self.directory_identity => (),
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            _ => {
                return Err(io::Error::other(
                    "processing directory identity changed; preserved",
                ));
            }
        }
        match identity(&self.path) {
            Ok(actual) if actual == self.fifo_identity => fs::remove_file(&self.path)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            _ => {
                return Err(io::Error::other(
                    "processing FIFO identity changed; preserved",
                ));
            }
        }
        fs::remove_dir(&self.directory)
    }
}
impl Drop for ProcessingEndpoint {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}
fn read_progress(mut pipe: File, stop: Arc<AtomicBool>, feed: ProgressFeed) -> io::Result<()> {
    let result = (|| {
        let mut bytes = [0u8; 4096];
        let mut drain_started = None;
        let mut drained = 0usize;
        loop {
            let stopping = stop.load(Ordering::Acquire);
            if stopping && drain_started.is_none() {
                drain_started = Some(Instant::now());
            }
            if drain_started.is_some_and(|at| at.elapsed() > DRAIN_GRACE) || drained > DRAIN_BYTES {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "processing drain limit",
                ));
            }
            match pipe.read(&mut bytes) {
                Ok(n) if n > 0 => {
                    feed.observe_dedicated_processing(&bytes[..n]);
                    if stopping {
                        drained += n;
                    }
                    continue;
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) if e.kind() != io::ErrorKind::WouldBlock => return Err(e),
                _ => {
                    if stopping {
                        break;
                    }
                }
            }
            let mut poll = libc::pollfd {
                fd: pipe.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            // SAFETY: one live reader-owned FD; bounded timeout, no descriptor mutation.
            if unsafe { libc::poll(&mut poll, 1, POLL_MS) } < 0 {
                let error = io::Error::last_os_error();
                if error.kind() != io::ErrorKind::Interrupted {
                    return Err(error);
                }
            }
        }
        Ok(())
    })();
    feed.finish_dedicated_processing();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn audio_fragments_pending_drain_and_private_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("한글 ' quoted folder");
        fs::create_dir(&root).unwrap();
        let feed = ProgressFeed::default();
        feed.set_processing_duration(Some(4_000_000));
        let mut endpoint = ProcessingEndpoint::new(&root, feed.clone()).unwrap();
        let path = PathBuf::from(endpoint.argument());
        assert_eq!(
            fs::metadata(&endpoint.directory).unwrap().mode() & 0o777,
            0o700
        );
        assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o600);
        let mut writer = OpenOptions::new().write(true).open(&path).unwrap();
        writer
            .write_all(b"bitrate=N/A\nout_time_us=1500000\nspe")
            .unwrap();
        writer.write_all(b"ed=2.5x\nprogress=continue\n").unwrap();
        let start = Instant::now();
        while feed.processing_snapshot().is_none() && start.elapsed() < Duration::from_secs(1) {
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(feed.processing_snapshot().unwrap().eta_seconds, Some(1.0));
        writer
            .write_all(b"out_time_us=4000000\nspeed=2x\nprogress=end\n")
            .unwrap();
        endpoint.finish().unwrap();
        assert!(feed.processing_snapshot().unwrap().ended);
        assert!(!path.exists());
        assert!(!endpoint.directory.exists());
        assert!(root.exists());
    }
    #[test]
    fn no_writer_cancel_and_incomplete_fragment_stop_bounded() {
        let dir = tempfile::tempdir().unwrap();
        let feed = ProgressFeed::default();
        let mut endpoint = ProcessingEndpoint::new(dir.path(), feed.clone()).unwrap();
        let start = Instant::now();
        endpoint.finish().unwrap();
        assert!(start.elapsed() < Duration::from_secs(1));
        assert_eq!(feed.processing_snapshot(), None);
        let mut endpoint = ProcessingEndpoint::new(dir.path(), feed.clone()).unwrap();
        let mut writer = OpenOptions::new()
            .write(true)
            .open(endpoint.argument())
            .unwrap();
        writer.write_all(b"out_time_us=100\nspeed=").unwrap();
        endpoint.finish().unwrap();
        assert_eq!(feed.processing_snapshot(), None);
    }
    #[test]
    fn replaced_node_is_preserved_and_bad_parent_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let feed = ProgressFeed::default();
        let mut endpoint = ProcessingEndpoint::new(dir.path(), feed.clone()).unwrap();
        let path = PathBuf::from(endpoint.argument());
        fs::remove_file(&path).unwrap();
        fs::write(&path, b"foreign replacement").unwrap();
        assert!(endpoint.finish().is_err());
        assert_eq!(fs::read(&path).unwrap(), b"foreign replacement");
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(dir.path(), &link).unwrap();
        assert!(ProcessingEndpoint::new(&link, feed.clone()).is_err());
        assert!(ProcessingEndpoint::new(&path, feed).is_err());
    }
    #[test]
    fn postprocessor_path_value_roundtrips_ytdlp_shlex() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("한글 ' whitespace");
        fs::create_dir(&root).unwrap();
        let endpoint = ProcessingEndpoint::new(&root, ProgressFeed::default()).unwrap();
        let output=std::process::Command::new("python3").args(["-c","import shlex,sys,json; print(json.dumps(shlex.split(sys.argv[1].split(':',1)[1])))"]).arg(endpoint.postprocessor_argument()).output().unwrap();
        assert!(output.status.success());
        let parsed: Vec<String> = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            parsed,
            vec![
                "-progress".to_owned(),
                endpoint.argument().to_str().unwrap().to_owned(),
                "-nostats".to_owned()
            ]
        );
    }
    #[test]
    fn repository_success_failure_and_cancel_drop_endpoint() {
        use crate::download::DownloadRepository;
        use dlpad_core::{
            jobs::JobState,
            options::{DownloadOptions, Quality, SourceUrl},
        };
        use std::os::unix::fs::PermissionsExt;
        for mode in ["success", "failure", "cancel"] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().join("한글 ' media");
            let tool = directory.path().join("synthetic-ytdlp");
            let script = r#"#!/usr/bin/python3
import pathlib,sys,json,shlex,time
base=pathlib.Path(__file__).parent
a=sys.argv[1:]
if '--version' in a:print('2026.10.02');sys.exit(0)
if '--dump-single-json' in a:print(json.dumps({'id':'fixture','duration':4}));sys.exit(0)
progress=shlex.split(a[a.index('--postprocessor-args')+1].split(':',1)[1])
p=pathlib.Path(progress[progress.index('-progress')+1])
(base/'endpoint.json').write_text(json.dumps(str(p)))
root=pathlib.Path(a[a.index('--paths')+1]);partial=root/'fixture.part';partial.write_bytes(b'x'*65536)
mode=(base/'mode').read_text()
with p.open('w') as out:
 out.write('bitrate=N/A\nout_time_us=1500000\nspeed=2.5x\nprogress=continue\n');out.flush()
 if mode=='cancel':
  while True:time.sleep(.1)
 if mode=='failure':
  out.write('speed=invalid\nprogress=end\n');out.flush();sys.exit(7)
 out.write('out_time_us=4000000\nspeed=2x\nprogress=end\n');out.flush()
final=root/a[a.index('--output')+1].replace('%(ext)s','mp4');final.write_bytes(b'synthetic-final')
print('DLPAD_ARTIFACT:'+json.dumps(str(final)),flush=True)
"#;
            fs::write(&tool, script).unwrap();
            fs::set_permissions(&tool, fs::Permissions::from_mode(0o700)).unwrap();
            fs::write(directory.path().join("mode"), mode).unwrap();
            let options = DownloadOptions {
                url: SourceUrl::parse("https://example.invalid/synthetic").unwrap(),
                output_dir: root,
                quality: Quality::Best,
                time_range: None,
                advanced: vec![],
                ffmpeg: None,
                deno: None,
                final_name_style: None,
                subtitles: None,
            };
            let repository =
                DownloadRepository::open(&directory.path().join("jobs.sqlite")).unwrap();
            repository.create_job("fixture", &options).unwrap();
            let feed = ProgressFeed::default();
            let cancel = Arc::new(AtomicBool::new(false));
            let requester = if mode == "cancel" {
                let observed = feed.clone();
                let signal = cancel.clone();
                Some(thread::spawn(move || {
                    let start = Instant::now();
                    while observed.processing_snapshot().is_none()
                        && start.elapsed() < Duration::from_secs(5)
                    {
                        thread::sleep(Duration::from_millis(5));
                    }
                    let ready = observed.processing_snapshot().is_some();
                    signal.store(true, Ordering::Release);
                    ready
                }))
            } else {
                None
            };
            let result =
                repository.execute_with_progress("fixture", &options, &tool, &cancel, feed.clone());
            match mode {
                "success" => {
                    assert!(result.is_ok());
                    assert_eq!(
                        repository.job("fixture").unwrap().state,
                        JobState::Downloaded
                    );
                    assert!(feed.processing_snapshot().unwrap().ended);
                }
                "failure" => {
                    assert!(result.is_err());
                    assert_eq!(repository.job("fixture").unwrap().state, JobState::Failed);
                    assert_eq!(feed.processing_snapshot(), None);
                }
                "cancel" => {
                    assert!(requester.unwrap().join().unwrap());
                    assert!(result.is_err());
                    assert_eq!(
                        repository.job("fixture").unwrap().state,
                        JobState::Cancelled
                    );
                }
                _ => unreachable!(),
            }
            let path: String =
                serde_json::from_slice(&fs::read(directory.path().join("endpoint.json")).unwrap())
                    .unwrap();
            let path = PathBuf::from(path);
            assert!(!path.exists());
            assert!(!path.parent().unwrap().exists());
            assert!(
                options
                    .output_dir
                    .join(".dlpad-partials/fixture/fixture.part")
                    .exists()
            );
        }
    }
}
