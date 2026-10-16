//! Explicit real-tool HTTP failure/resume proof using synthetic media only.
#![cfg(any(unix, windows))]
use dlpad_core::{
    jobs::JobState,
    options::{AdvancedOption, DownloadOptions, Quality, SourceUrl},
};
use dlpad_engine::{
    download::{DownloadError, DownloadRepository},
    process::{self, ProcessOutcome, ProcessSpec},
};
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[cfg(windows)]
struct ToolPin {
    file: std::fs::File,
    digest: String,
    path: PathBuf,
}
#[cfg(windows)]
impl ToolPin {
    fn new(path: &Path, variable: &str) -> Self {
        use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
        assert!(path.is_absolute(), "explicit absolute Windows tool");
        for ancestor in path.ancestors() {
            let metadata = std::fs::symlink_metadata(ancestor).unwrap();
            assert_eq!(
                metadata.file_attributes() & 0x400,
                0,
                "original tool reparse rejected"
            );
        }
        // Retain an original read capability denying foreign writes and deletes.
        let file = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(path)
            .unwrap();
        let expected = std::env::var(variable).expect("explicit original tool SHA256 required");
        assert_eq!(expected.len(), 64);
        assert!(
            expected
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        );
        let mut pin = Self {
            file,
            digest: expected,
            path: path.into(),
        };
        pin.verify();
        pin
    }
    fn verify(&mut self) {
        use sha2::{Digest, Sha256};
        use std::io::{Seek, SeekFrom};
        let metadata = self.file.metadata().unwrap();
        assert!(metadata.is_file() && metadata.len() <= 256 * 1024 * 1024);
        self.file.seek(SeekFrom::Start(0)).unwrap();
        let mut digest = Sha256::new();
        let mut total = 0u64;
        let mut buffer = [0u8; 65536];
        loop {
            let count = self.file.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            total += count as u64;
            assert!(total <= metadata.len());
            digest.update(&buffer[..count]);
        }
        assert_eq!(total, metadata.len());
        assert_eq!(
            format!("{:x}", digest.finalize()),
            self.digest,
            "original pinned tool {}",
            self.path.display()
        );
    }
}

const MAX_MEDIA: usize = 4 * 1024 * 1024;
const CUT_BYTES: usize = 32 * 1024;
const EXPIRED: usize = 0;
const DISCONNECT: usize = 1;
const HEALTHY: usize = 2;

struct Deadline {
    cancel: Arc<AtomicBool>,
    finished: Arc<(Mutex<bool>, Condvar)>,
    worker: Option<JoinHandle<()>>,
}
impl Deadline {
    fn new() -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let finished = Arc::new((Mutex::new(false), Condvar::new()));
        let owned = cancel.clone();
        let signal = finished.clone();
        let worker = thread::spawn(move || {
            let (lock, wake) = &*signal;
            let (done, _) = wake
                .wait_timeout_while(lock.lock().unwrap(), Duration::from_secs(30), |done| !*done)
                .unwrap();
            if !*done {
                owned.store(true, Ordering::Release);
            }
        });
        Self {
            cancel,
            finished,
            worker: Some(worker),
        }
    }
}
impl Drop for Deadline {
    fn drop(&mut self) {
        *self.finished.0.lock().unwrap() = true;
        self.finished.1.notify_all();
        self.worker.take().unwrap().join().unwrap();
    }
}

#[derive(Default)]
struct Observations {
    source_pages: usize,
    forbidden: usize,
    disconnected: usize,
    refreshed_media: usize,
    positive_ranges: Vec<usize>,
}
struct Server {
    port: u16,
    mode: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
    observed: Arc<Mutex<Observations>>,
    worker: Option<JoinHandle<()>>,
}
impl Server {
    fn new(body: Vec<u8>, mode: usize) -> Self {
        assert!(body.len() > 2 * CUT_BYTES && body.len() <= MAX_MEDIA);
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let mode = Arc::new(AtomicUsize::new(mode));
        let stop = Arc::new(AtomicBool::new(false));
        let observed = Arc::new(Mutex::new(Observations::default()));
        let settings = mode.clone();
        let shutdown = stop.clone();
        let evidence = observed.clone();
        let worker = thread::spawn(move || {
            let mut requests = 0;
            while !shutdown.load(Ordering::Acquire) && requests < 256 {
                match listener.accept() {
                    Ok((mut socket, _)) => {
                        // Accepted sockets may inherit the listener's nonblocking mode on Windows.
                        // This bounded request reader uses blocking I/O with explicit timeouts.
                        socket.set_nonblocking(false).unwrap();
                        requests += 1;
                        socket
                            .set_read_timeout(Some(Duration::from_millis(500)))
                            .unwrap();
                        socket
                            .set_write_timeout(Some(Duration::from_millis(500)))
                            .unwrap();
                        let _ = respond(
                            &mut socket,
                            &body,
                            settings.load(Ordering::Acquire),
                            &evidence,
                        );
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("owned loopback server: {error}"),
                }
            }
        });
        Self {
            port,
            mode,
            stop,
            observed,
            worker: Some(worker),
        }
    }
    fn source(&self) -> String {
        format!("http://127.0.0.1:{}/source.html", self.port)
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.worker.take().unwrap().join().unwrap();
    }
}

fn respond(
    socket: &mut TcpStream,
    media: &[u8],
    mode: usize,
    observed: &Mutex<Observations>,
) -> std::io::Result<()> {
    let mut bytes = Vec::with_capacity(1024);
    let mut buffer = [0; 1024];
    while !bytes.windows(4).any(|part| part == b"\r\n\r\n") {
        let count = socket.read(&mut buffer)?;
        if count == 0 || bytes.len() + count > 8192 {
            return Ok(());
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    let request = String::from_utf8_lossy(&bytes);
    let mut first = request
        .lines()
        .next()
        .unwrap_or_default()
        .split_whitespace();
    let method = first.next().unwrap_or_default();
    let path = first.next().unwrap_or_default();
    if path == "/source.html" {
        observed.lock().unwrap().source_pages += 1;
        let ticket = if mode == HEALTHY {
            "refreshed"
        } else {
            "expired"
        };
        let html = format!(
            "<html><head><title>Synthetic failure fixture</title></head><body><video controls src=\"/media.mp4?ticket={ticket}\"></video></body></html>"
        );
        write!(
            socket,
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            html.len()
        )?;
        if method != "HEAD" {
            socket.write_all(html.as_bytes())?;
        }
    } else if path.starts_with("/media.mp4?") {
        if mode == EXPIRED {
            observed.lock().unwrap().forbidden += 1;
            socket.write_all(
                b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )?;
            return Ok(());
        }
        let start = request
            .lines()
            .find_map(|line| {
                line.to_ascii_lowercase()
                    .strip_prefix("range: bytes=")?
                    .split('-')
                    .next()?
                    .parse::<usize>()
                    .ok()
            })
            .unwrap_or(0);
        if start >= media.len() {
            socket.write_all(b"HTTP/1.1 416 Range Not Satisfiable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")?;
            return Ok(());
        }
        if start > 0 {
            let mut evidence = observed.lock().unwrap();
            if evidence.positive_ranges.len() < 128 {
                evidence.positive_ranges.push(start);
            }
        }
        if path.contains("ticket=refreshed") {
            observed.lock().unwrap().refreshed_media += 1;
        }
        let remaining = &media[start..];
        if start == 0 {
            write!(socket, "HTTP/1.1 200 OK\r\n")?;
        } else {
            write!(
                socket,
                "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes {}-{}/{}\r\n",
                start,
                media.len() - 1,
                media.len()
            )?;
        }
        write!(
            socket,
            "Content-Type: video/mp4\r\nAccept-Ranges: bytes\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            remaining.len()
        )?;
        if method != "HEAD" {
            if mode == DISCONNECT {
                socket.write_all(&remaining[..remaining.len().min(CUT_BYTES)])?;
                socket.flush()?;
                observed.lock().unwrap().disconnected += 1;
                socket.shutdown(Shutdown::Both)?;
            } else {
                socket.write_all(remaining)?;
            }
        }
    } else {
        socket.write_all(
            b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        )?;
    }
    Ok(())
}

fn generate(ffmpeg: &Path, source: &Path) -> Vec<u8> {
    let cancel = Arc::new(AtomicBool::new(false));
    let generated = process::run(
        &ProcessSpec {
            #[cfg(windows)]
            runtime_environment: None,
            executable: ffmpeg.into(),
            args: [
                "-nostdin",
                "-n",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc2=s=320x180:r=20",
                "-t",
                "5",
                "-c:v",
                "mpeg4",
                "-q:v",
                "2",
                "-movflags",
                "+faststart",
            ]
            .into_iter()
            .map(Into::into)
            .chain([source.as_os_str().to_owned()])
            .collect(),
            timeout: Duration::from_secs(20),
            max_log_bytes: process::MAX_LOG_BYTES,
        },
        &cancel,
    )
    .unwrap();
    assert_eq!(generated.outcome, ProcessOutcome::Exited);
    assert_eq!(generated.exit_code, Some(0), "{}", generated.logs);
    let size = std::fs::metadata(source).unwrap().len();
    assert!(size > (2 * CUT_BYTES) as u64 && size <= MAX_MEDIA as u64);
    std::fs::read(source).unwrap()
}

#[test]
#[ignore = "requires explicit DLPAD_SMOKE_YTDLP and DLPAD_SMOKE_FFMPEG executable paths"]
fn real_http_expiry_disconnect_and_same_job_resume_are_persisted() {
    let tool = PathBuf::from(std::env::var_os("DLPAD_SMOKE_YTDLP").expect("explicit yt-dlp path"));
    let ffmpeg =
        PathBuf::from(std::env::var_os("DLPAD_SMOKE_FFMPEG").expect("explicit FFmpeg path"));
    #[cfg(windows)]
    let mut tool_pin = ToolPin::new(&tool, "DLPAD_SMOKE_YTDLP_SHA256");
    #[cfg(windows)]
    let mut ffmpeg_pin = ToolPin::new(&ffmpeg, "DLPAD_SMOKE_FFMPEG_SHA256");
    let directory = tempfile::tempdir().unwrap();
    let media = generate(&ffmpeg, &directory.path().join("synthetic.mp4"));
    for (id, mode) in [("expired-http", EXPIRED), ("disconnected-http", DISCONNECT)] {
        let server = Server::new(media.clone(), mode);
        let options = DownloadOptions {
            url: SourceUrl::parse(&server.source()).unwrap(),
            output_dir: directory.path().join(id),
            quality: Quality::Best,
            time_range: None,
            advanced: vec![AdvancedOption {
                key: "--retries".into(),
                value: "0".into(),
            }],
            ffmpeg: None,
            deno: None,
            final_name_style: None,
            subtitles: None,
        };
        let database = directory.path().join(format!("{id}.sqlite"));
        let repo = DownloadRepository::open(&database).unwrap();
        repo.create_job(id, &options).unwrap();
        let watchdog = Deadline::new();
        let started = Instant::now();
        let error = match repo.execute(id, &options, &tool, &watchdog.cancel) {
            Err(error) => error,
            Ok(_) => panic!("injected HTTP failure must not download"),
        };
        assert!(started.elapsed() < Duration::from_secs(30));
        assert!(
            !watchdog.cancel.load(Ordering::Acquire),
            "watchdog fired: {error}"
        );
        let DownloadError::Execution { result, .. } = error else {
            panic!("actual tool execution failure expected: {error}")
        };
        assert_eq!(result.outcome, ProcessOutcome::Exited, "{}", result.logs);
        assert_ne!(result.exit_code, Some(0));
        eprintln!("{id} actual failure: {}", result.logs);
        drop(watchdog);
        let failed = repo.job(id).unwrap();
        assert_eq!(failed.state, JobState::Failed);
        assert!(failed.artifacts.is_empty());
        assert_eq!(failed.attempts.len(), 1);
        let evidence = server.observed.lock().unwrap();
        assert!(evidence.source_pages > 0);
        if mode == EXPIRED {
            assert!(evidence.forbidden > 0);
        } else {
            assert!(evidence.disconnected > 0);
        }
        drop(evidence);
        let partial = options
            .output_dir
            .join(".dlpad-partials")
            .join(id)
            .join(format!("{id}.mp4.part"));
        let partial_bytes = if mode == DISCONNECT {
            let bytes = std::fs::metadata(&partial)
                .expect("real partial must be retained")
                .len();
            assert!(bytes > 0 && bytes < media.len() as u64);
            bytes
        } else {
            0
        };
        assert!(!options.output_dir.join(format!("{id}.mp4")).exists());
        let pages_before = server.observed.lock().unwrap().source_pages;
        drop(repo);
        let reopened = DownloadRepository::open(&database).unwrap();
        assert_eq!(reopened.job(id).unwrap().state, JobState::Failed);
        assert_eq!(
            reopened
                .stored_options(id)
                .unwrap()
                .to_stored_json()
                .unwrap(),
            options.to_stored_json().unwrap()
        );
        server.mode.store(HEALTHY, Ordering::Release);
        let watchdog = Deadline::new();
        let restored = reopened
            .resume(id, &tool, &watchdog.cancel)
            .unwrap_or_else(|error| {
                if let DownloadError::Execution { result, .. } = &error {
                    eprintln!("{}", result.logs);
                }
                panic!("{id} resume: {error}");
            });
        assert!(!watchdog.cancel.load(Ordering::Acquire));
        drop(watchdog);
        assert_eq!(restored.job.id, id);
        assert_eq!(restored.job.state, JobState::Downloaded);
        assert_eq!(restored.job.attempts.len(), 2);
        assert_eq!(restored.job.attempts[0], failed.attempts[0]);
        assert_ne!(restored.job.attempts[0].id, restored.job.attempts[1].id);
        assert_eq!(restored.job.attempts[1].number, 2);
        assert_eq!(restored.job.artifacts.len(), 1);
        assert_eq!(
            std::fs::read(&restored.job.artifacts[0].path).unwrap(),
            media
        );
        let evidence = server.observed.lock().unwrap();
        assert!(
            evidence.source_pages > pages_before,
            "original source must be re-extracted"
        );
        assert!(evidence.refreshed_media > 0);
        if mode == DISCONNECT {
            assert!(
                evidence.positive_ranges.contains(&(partial_bytes as usize)),
                "retained bytes={partial_bytes}, ranges={:?}",
                evidence.positive_ranges
            );
        }
        eprintln!(
            "{id}: partial={partial_bytes}, source_pages={}, refreshed={}, ranges={:?}",
            evidence.source_pages, evidence.refreshed_media, evidence.positive_ranges
        );
        drop(evidence);
        drop(reopened);
        let persisted = DownloadRepository::open(&database).unwrap();
        let final_job = persisted.job(id).unwrap();
        assert_eq!(final_job.state, JobState::Downloaded);
        assert_eq!(final_job.attempts.len(), 2);
        assert_eq!(final_job.artifacts, restored.job.artifacts);
        assert_eq!(persisted.recover().unwrap(), 0);
        assert!(!partial.exists());
    }
    #[cfg(windows)]
    {
        tool_pin.verify();
        ffmpeg_pin.verify();
    }
}
