//! Owned loopback actual-tool transfer longevity; not real-site, GUI or sleep proof.
#![cfg(windows)]
use dlpad_core::{
    jobs::JobState,
    options::{DownloadOptions, Quality, SourceUrl},
};
use dlpad_engine::{download::DownloadRepository, metrics::ProgressFeed, process};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
#[cfg(windows)]
struct ToolPin {
    file: std::fs::File,
    digest: String,
    path: PathBuf,
    identity: dlpad_engine::file_identity::NativeFileIdentity,
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
            identity: dlpad_engine::file_identity::from_file(&file).unwrap(),
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
        assert_eq!(
            dlpad_engine::file_identity::from_file(&self.file).unwrap(),
            self.identity
        );
        let current = dlpad_engine::file_identity::open_regular_no_reparse(&self.path).unwrap();
        assert_eq!(
            dlpad_engine::file_identity::from_file(&current).unwrap(),
            self.identity
        );
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

const MAX_MEDIA: u64 = 128 * 1024 * 1024;
fn envpath(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).expect(name))
}
fn hash(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let mut f = File::open(path).unwrap();
    let size = f.metadata().unwrap().len();
    assert!(size <= 256 * 1024 * 1024);
    let mut sha = Sha256::new();
    let mut bytes = 0;
    let mut buffer = [0; 65536];
    loop {
        let n = f.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        bytes += n as u64;
        assert!(bytes <= size);
        sha.update(&buffer[..n]);
    }
    assert_eq!(bytes, size);
    format!("{:x}", sha.finalize())
}
fn new_json(path: &Path, value: &serde_json::Value) {
    let raw = serde_json::to_vec(value).unwrap();
    assert!(raw.len() <= 512 * 1024);
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    f.write_all(&raw).unwrap();
    f.sync_all().unwrap();
}
#[derive(Default)]
struct Observed {
    requests: u64,
    heads: u64,
    early_disconnects: u64,
    completed_bodies: u64,
    completed_transfer_seconds: f64,
    bytes_sent: u64,
}
struct Server {
    port: u16,
    stop: Arc<AtomicBool>,
    observed: Arc<Mutex<Observed>>,
    worker: Option<JoinHandle<()>>,
}
impl Server {
    fn new(media: PathBuf, seconds: u64, cancel: Arc<AtomicBool>) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let stop = Arc::new(AtomicBool::new(false));
        let observed = Arc::new(Mutex::new(Observed::default()));
        let stopping = stop.clone();
        let evidence = observed.clone();
        let worker = thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                while !stopping.load(Ordering::Acquire) {
                    match listener.accept() {
                        Ok((mut socket, _)) => {
                            socket.set_nonblocking(false).unwrap();
                            socket
                                .set_read_timeout(Some(Duration::from_millis(500)))
                                .unwrap();
                            socket
                                .set_write_timeout(Some(Duration::from_millis(500)))
                                .unwrap();
                            {
                                let mut o = evidence.lock().unwrap();
                                o.requests += 1;
                                assert!(o.requests <= 256, "request bound");
                            }
                            if respond(&mut socket, &media, seconds, &stopping, &evidence).is_err()
                            {
                                evidence.lock().unwrap().early_disconnects += 1;
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(10))
                        }
                        Err(e) => panic!("owned listener {e}"),
                    }
                }
            }));
            if let Err(error) = result {
                cancel.store(true, Ordering::Release);
                std::panic::resume_unwind(error);
            }
        });
        Self {
            port,
            stop,
            observed,
            worker: Some(worker),
        }
    }
    fn source(&self) -> String {
        format!("http://127.0.0.1:{}/source.html", self.port)
    }
    fn finish(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.finish();
    }
}
fn respond(
    socket: &mut TcpStream,
    media: &Path,
    seconds: u64,
    stop: &AtomicBool,
    evidence: &Mutex<Observed>,
) -> std::io::Result<()> {
    let mut request = Vec::with_capacity(8192);
    let mut buf = [0u8; 1024];
    while !request.windows(4).any(|b| b == b"\r\n\r\n") {
        let n = socket.read(&mut buf)?;
        if n == 0 || request.len() + n > 8192 {
            return Ok(());
        }
        request.extend_from_slice(&buf[..n]);
    }
    let text = String::from_utf8_lossy(&request);
    let mut first = text.lines().next().unwrap_or_default().split_whitespace();
    let method = first.next().unwrap_or_default();
    let path = first.next().unwrap_or_default();
    if !["GET", "HEAD"].contains(&method) {
        return Ok(());
    }
    if path == "/source.html" {
        let html=b"<html><head><title>Owned eight hour media</title></head><body><video src=\"/media.mp4\"></video></body></html>";
        write!(
            socket,
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            html.len()
        )?;
        if method == "GET" {
            socket.write_all(html)?
        }
        return Ok(());
    }
    if path != "/media.mp4" {
        socket.write_all(
            b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        )?;
        return Ok(());
    }
    let mut f = File::open(media)?;
    let size = f.metadata()?.len();
    let range = text.lines().find_map(|line| {
        line.to_ascii_lowercase()
            .strip_prefix("range: bytes=")
            .map(str::to_owned)
    });
    let mut start = 0;
    let mut end = size - 1;
    if let Some(range) = &range {
        let (a, b) = range
            .split_once('-')
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
        start = a
            .parse()
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
        if !b.is_empty() {
            end = b
                .parse::<u64>()
                .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?
                .min(size - 1);
        }
    }
    if start >= size || end < start {
        write!(
            socket,
            "HTTP/1.1 416 Range Not Satisfiable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )?;
        return Ok(());
    }
    let count = end - start + 1;
    if range.is_some() {
        write!(
            socket,
            "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes {start}-{end}/{size}\r\n"
        )?
    } else {
        socket.write_all(b"HTTP/1.1 200 OK\r\n")?
    }
    write!(
        socket,
        "Content-Type: video/mp4\r\nAccept-Ranges: bytes\r\nContent-Length: {count}\r\nConnection: close\r\n\r\n"
    )?;
    if method == "HEAD" {
        evidence.lock().unwrap().heads += 1;
        return Ok(());
    }
    f.seek(SeekFrom::Start(start))?;
    let began = Instant::now();
    let paced = count > 131072;
    let mut sent = 0u64;
    let warm = count.min(65536);
    let mut buffer = [0u8; 65536];
    while sent < count && !stop.load(Ordering::Acquire) {
        let want = if sent < warm {
            (warm - sent).min(buffer.len() as u64)
        } else {
            ((count - warm).div_ceil(seconds.max(1)))
                .clamp(1, 65536)
                .min(count - sent)
        } as usize;
        let n = f.read(&mut buffer[..want])?;
        if n == 0 {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
        if paced && sent + n as u64 > warm {
            let fraction = (sent + n as u64 - warm) as f64 / (count - warm) as f64;
            let deadline = began + Duration::from_secs_f64(seconds as f64 * fraction);
            while !stop.load(Ordering::Acquire) {
                let now = Instant::now();
                if now >= deadline {
                    break;
                }
                thread::sleep((deadline - now).min(Duration::from_millis(100)));
            }
        }
        if stop.load(Ordering::Acquire) {
            break;
        }
        socket.write_all(&buffer[..n])?;
        sent += n as u64;
        evidence.lock().unwrap().bytes_sent += n as u64;
    }
    if sent == count {
        let mut o = evidence.lock().unwrap();
        if paced {
            o.completed_bodies += 1;
            o.completed_transfer_seconds = o
                .completed_transfer_seconds
                .max(began.elapsed().as_secs_f64());
        }
    }
    Ok(())
}
// Product download.rs stages under canonical output/.dlpad-partials/job.id;
// enumerate actual bounded leaves instead of assuming a yt-dlp extension/template.
fn partial_inventory(output: &Path, id: &str) -> (bool, u64, u64) {
    use std::os::windows::fs::MetadataExt;
    let stage = output.join(".dlpad-partials").join(id);
    match fs::symlink_metadata(&stage) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (false, 0, 0),
        Err(e) => panic!("staging inspection {e}"),
        Ok(metadata) => {
            assert!(metadata.is_dir());
            assert_eq!(metadata.file_attributes() & 0x400, 0);
            let mut count = 0;
            let mut bytes = 0;
            for entry in fs::read_dir(stage).unwrap() {
                let entry = entry.unwrap();
                count += 1;
                assert!(count <= 32, "staging leaf bound");
                let m = fs::symlink_metadata(entry.path()).unwrap();
                assert!(m.is_file());
                assert_eq!(m.file_attributes() & 0x400, 0);
                bytes += m.len();
                assert!(bytes <= MAX_MEDIA, "actual staging aggregate bound");
            }
            (true, count, bytes)
        }
    }
}
fn sample(root: &Path, started: Instant, feed: &ProgressFeed) -> serde_json::Value {
    let (present, files, bytes) = partial_inventory(&root.join("output"), "wall-soak");
    let database_bytes = fs::metadata(root.join("jobs.sqlite")).unwrap().len();
    assert!(database_bytes <= 16 * 1024 * 1024);
    serde_json::json!({"elapsed_seconds":started.elapsed().as_secs_f64(),"working_set_bytes":memory(),"progress_present":feed.snapshot().is_some(),"database_bytes":database_bytes,"partial_namespace_present":present,"partial_files":files,"partial_aggregate_bytes":bytes,"partial_size_inspected":present})
}
fn checkpoint(file: &mut File, value: &serde_json::Value) {
    let raw = serde_json::to_vec(value).unwrap();
    assert!(raw.len() <= 4096);
    file.seek(SeekFrom::Start(0)).unwrap();
    file.write_all(&raw).unwrap();
    file.set_len(raw.len() as u64).unwrap();
    file.sync_all().unwrap();
}
#[repr(C)]
struct Memory {
    cb: u32,
    faults: u32,
    peak: usize,
    working: usize,
    paged_peak: usize,
    paged: usize,
    nonpaged_peak: usize,
    nonpaged: usize,
    pagefile: usize,
    pagefile_peak: usize,
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> *mut std::ffi::c_void;
}
#[link(name = "psapi")]
unsafe extern "system" {
    fn GetProcessMemoryInfo(process: *mut std::ffi::c_void, info: *mut Memory, size: u32) -> i32;
}
fn memory() -> Option<usize> {
    let mut info = Memory {
        cb: std::mem::size_of::<Memory>() as u32,
        faults: 0,
        peak: 0,
        working: 0,
        paged_peak: 0,
        paged: 0,
        nonpaged_peak: 0,
        nonpaged: 0,
        pagefile: 0,
        pagefile_peak: 0,
    };
    let size = info.cb;
    let ok = unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut info, size) };
    if ok != 0 { Some(info.working) } else { None }
}
fn run(formal: bool) {
    let seconds: u64 = std::env::var("DLPAD_WINDOWS_SOAK_SECONDS")
        .unwrap()
        .parse()
        .unwrap();
    assert!(if formal {
        (28800..=28860).contains(&seconds)
    } else {
        seconds == 72
    });
    let root = envpath("DLPAD_WINDOWS_SOAK_ROOT");
    assert!(root.is_absolute() && !root.exists());
    let temp = std::env::temp_dir();
    assert_eq!(root.parent(), Some(temp.as_path()));
    assert!(
        root.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("dlpad-windows-wall-soak-owned-")
    );
    for a in root.parent().unwrap().ancestors() {
        use std::os::windows::fs::MetadataExt;
        assert_eq!(
            fs::symlink_metadata(a).unwrap().file_attributes() & 0x400,
            0
        )
    }
    fs::create_dir(&root).unwrap();
    let sentinel = root.join("foreign-sentinel");
    fs::write(&sentinel, b"preserve").unwrap();
    let media = envpath("DLPAD_WINDOWS_SOAK_MEDIA");
    let tool = envpath("DLPAD_SMOKE_YTDLP");
    let ffmpeg = envpath("DLPAD_SMOKE_FFMPEG");
    let mut media_pin = ToolPin::new(&media, "DLPAD_WINDOWS_SOAK_MEDIA_SHA256");
    let mut tool_pin = ToolPin::new(&tool, "DLPAD_SMOKE_YTDLP_SHA256");
    let mut ffmpeg_pin = ToolPin::new(&ffmpeg, "DLPAD_SMOKE_FFMPEG_SHA256");
    assert!(
        media_pin.file.metadata().unwrap().len() > 131072
            && media_pin.file.metadata().unwrap().len() <= MAX_MEDIA
    );
    let probe_path = envpath("DLPAD_WINDOWS_SOAK_MEDIA_PROOF");
    let mut probe_pin = ToolPin::new(&probe_path, "DLPAD_WINDOWS_SOAK_MEDIA_PROOF_SHA256");
    assert!(probe_pin.file.metadata().unwrap().len() <= 65536);
    let proof: serde_json::Value =
        serde_json::from_reader(File::open(&probe_path).unwrap()).unwrap();
    assert_eq!(proof["media_sha256"].as_str().unwrap(), media_pin.digest);
    assert_eq!(
        proof["media_bytes"].as_u64().unwrap(),
        media_pin.file.metadata().unwrap().len()
    );
    let duration = proof["format"]["duration"]
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert!(duration.is_finite() && duration >= 28800.0);
    let duration_ms = (duration * 1000.0) as u64;
    let cancel = Arc::new(AtomicBool::new(false));
    let mut server = Server::new(media.clone(), seconds, cancel.clone());
    let options = DownloadOptions {
        url: SourceUrl::parse(&server.source()).unwrap(),
        output_dir: root.join("output"),
        quality: Quality::Best,
        time_range: None,
        advanced: vec![],
        ffmpeg: None,
        deno: None,
        final_name_style: None,
        subtitles: None,
    };
    let db = root.join("jobs.sqlite");
    let repo = DownloadRepository::open(&db).unwrap();
    repo.create_job("wall-soak", &options).unwrap();
    let feed = ProgressFeed::default();
    let done = Arc::new(AtomicBool::new(false));
    let finished = done.clone();
    let cancellation = cancel.clone();
    let observed_feed = feed.clone();
    let sample_root = root.clone();
    let started = Instant::now();
    new_json(
        &root.join("launch.json"),
        &serde_json::json!({"state":"running","formal_eight_hour":formal,"planned_seconds":seconds,"media_duration_ms_declared":duration_ms,"duration_original_proof_bound":true,"media_sha256":media_pin.digest,"tool_sha256":tool_pin.digest,"ffmpeg_sha256":ffmpeg_pin.digest}),
    );
    let observer = thread::spawn(move || {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut live = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(sample_root.join("latest-sample.json"))
                .unwrap();
            let mut samples = Vec::new();
            let mut next = Instant::now();
            let interval = if formal { seconds.div_ceil(478) } else { 60 };
            while !finished.load(Ordering::Acquire) {
                if started.elapsed() > Duration::from_secs(seconds + 600) {
                    cancellation.store(true, Ordering::Release);
                    break;
                }
                if Instant::now() >= next && samples.len() < 479 {
                    let value = sample(&sample_root, started, &observed_feed);
                    checkpoint(&mut live, &value);
                    samples.push(value);
                    next += Duration::from_secs(interval);
                }
                thread::sleep(Duration::from_millis(100));
            }
            let final_sample = sample(&sample_root, started, &observed_feed);
            checkpoint(&mut live, &final_sample);
            assert!(samples.len() < 480);
            samples.push(final_sample);
            samples
        }));
        if outcome.is_err() {
            cancellation.store(true, Ordering::Release);
        }
        outcome
    });
    let executed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        repo.execute_with_progress("wall-soak", &options, &tool, &cancel, feed.clone())
    }));
    done.store(true, Ordering::Release);
    let observed = observer.join();
    let server_join = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| server.finish()));
    assert!(server_join.is_ok(), "server thread must ACK join");
    let samples = observed
        .expect("observer thread joined")
        .expect("observer failed; download cancellation requested");
    let result = executed.expect("download panicked after owned threads joined");
    new_json(&root.join("samples.json"), &serde_json::json!(samples));
    let result = result.unwrap();
    assert!(
        samples.iter().any(|s| s["progress_present"] == true
            && s["partial_namespace_present"] == true
            && s["partial_files"].as_u64().unwrap() > 0
            && s["partial_aggregate_bytes"].as_u64().unwrap() > 0),
        "actual partial bytes must be observed, absent is not verified size"
    );
    assert!(
        samples.first().unwrap()["elapsed_seconds"]
            .as_f64()
            .unwrap()
            <= 5.0
    );
    if formal {
        assert!(
            samples.len() >= seconds as usize / 65,
            "formal wall sample coverage"
        )
    }
    assert!(
        started.elapsed().as_secs_f64()
            - samples.last().unwrap()["elapsed_seconds"].as_f64().unwrap()
            <= 65.0,
        "last-to-finish sample gap"
    );
    for adjacent in samples.windows(2) {
        assert!(
            adjacent[1]["elapsed_seconds"].as_f64().unwrap()
                - adjacent[0]["elapsed_seconds"].as_f64().unwrap()
                <= 65.0,
            "sampling gap/sleep is not active transfer proof"
        )
    }
    assert!(!cancel.load(Ordering::Acquire));
    assert_eq!(result.job.state, JobState::Downloaded);
    assert_eq!(result.job.attempts.len(), 1);
    assert_eq!(result.job.artifacts.len(), 1);
    assert!(result.execution.logs.len() <= process::MAX_LOG_BYTES);
    assert!(feed.snapshot().is_some());
    let o = server.observed.lock().unwrap();
    assert!(
        o.completed_bodies >= 1 && o.completed_transfer_seconds >= seconds as f64,
        "actual transfer must span selected wall duration"
    );
    assert_eq!(hash(&result.job.artifacts[0].path), media_pin.digest);
    assert!(started.elapsed() >= Duration::from_secs(seconds));
    drop(repo);
    let reopened = DownloadRepository::open(&db).unwrap();
    assert_eq!(reopened.recover().unwrap(), 0);
    let job = reopened.job("wall-soak").unwrap();
    assert_eq!(job.state, JobState::Downloaded);
    assert_eq!(job.attempts.len(), 1);
    assert_eq!(job.artifacts, result.job.artifacts);
    assert_eq!(fs::read(sentinel).unwrap(), b"preserve");
    media_pin.verify();
    tool_pin.verify();
    ffmpeg_pin.verify();
    probe_pin.verify();
    assert!(
        started.elapsed().as_secs_f64()
            - samples.last().unwrap()["elapsed_seconds"].as_f64().unwrap()
            <= 65.0,
        "final acceptance must retain bounded sample gap"
    );
    let summary = serde_json::json!({"status":"REVIEW_REQUIRED","phase":"windows-wall-download-soak","formal_eight_hour":formal,"eight_hour_wall_transfer_verified":formal,"elapsed_seconds":started.elapsed().as_secs_f64(),"completed_transfer_seconds":o.completed_transfer_seconds,"requests":o.requests,"early_disconnects":o.early_disconnects,"artifact_sha256":media_pin.digest,"db_reopen_verified":true,"physical_sleep_proven":false,"real_site_proven":false,"gui_proven":false});
    new_json(&root.join("result.json"), &summary);
    println!("{summary}");
}
#[test]
#[ignore = "explicit owned copied media/tools/whole proof;72second preflight only"]
fn windows_actual_download_wall_preflight() {
    run(false)
}
#[test]
#[ignore = "explicit immutable owned inputs and genuine28800second wall transfer"]
fn windows_actual_download_eight_hour_wall_soak() {
    run(true)
}
