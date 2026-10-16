//! Windows real-tool synthetic local-media fixture. No user media or credentials.
#[cfg(windows)]
mod native {
    use dlpad_core::{
        jobs::JobState,
        options::{DownloadOptions, Quality, SourceUrl, TimeRange},
    };
    use dlpad_engine::{
        download::DownloadRepository,
        process::{self, ProcessOutcome, ProcessSpec},
    };
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use std::{
        ffi::OsString,
        fs::{self, File, OpenOptions},
        io::{self, Read, Seek, SeekFrom, Write},
        net::{TcpListener, TcpStream},
        os::windows::fs::{MetadataExt, OpenOptionsExt},
        path::{Path, PathBuf},
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
            mpsc,
        },
        thread,
        time::{Duration, Instant},
    };
    type Error = Box<dyn std::error::Error>;
    const FFMPEG_SHA: &str = "0e482b853f2fd4821055d20cdce510f0eea6027037c04b994f454a2136e8868e";
    const PROBE_SHA: &str = "b9627185ffbad1d4fc6ef953868ca9ad8df75fc8dd035fcab58aca78caa2d23e";
    const VERSION: &str = "n9.0.2-17-g2a571b6068-20260930";
    const SENTINEL: &[u8] = b"dlpad-real-ffmpeg-owned-sentinel-v1";
    #[derive(Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    struct Input {
        schema: u8,
        yt_dlp: Tool,
        ffmpeg: Tool,
        ffprobe: Tool,
        license: Tool,
    }
    #[derive(Deserialize, Serialize)]
    #[serde(deny_unknown_fields)]
    struct Tool {
        path: PathBuf,
        sha256: String,
        bytes: u64,
    }
    struct Pin {
        file: File,
        expected: String,
        identity: dlpad_engine::file_identity::NativeFileIdentity,
    }
    fn plain(path: &Path) -> Result<(), Error> {
        if !path.is_absolute()
            || path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err("absolute plain path required".into());
        }
        for ancestor in path.ancestors() {
            let m = fs::symlink_metadata(ancestor)?;
            if m.file_attributes() & 0x400 != 0 {
                return Err("reparse path rejected".into());
            }
        }
        Ok(())
    }
    fn hash(file: &mut File) -> Result<String, Error> {
        file.seek(SeekFrom::Start(0))?;
        let mut hash = Sha256::new();
        let mut b = [0; 65536];
        let mut bytes = 0u64;
        loop {
            let n = file.read(&mut b)?;
            if n == 0 {
                break;
            }
            bytes += n as u64;
            if bytes > 256 * 1024 * 1024 {
                return Err("bounded hash rejected".into());
            }
            hash.update(&b[..n])
        }
        Ok(format!("{:x}", hash.finalize()))
    }
    fn pin(tool: &Tool) -> Result<Pin, Error> {
        plain(&tool.path)?;
        if tool.bytes == 0
            || tool.bytes > 256 * 1024 * 1024
            || tool.sha256.len() != 64
            || !tool
                .sha256
                .bytes()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        {
            return Err("tool manifest rejected".into());
        }
        let mut file = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .custom_flags(0x00200000)
            .open(&tool.path)?;
        if !file.metadata()?.is_file()
            || file.metadata()?.len() != tool.bytes
            || hash(&mut file)? != tool.sha256
        {
            return Err("original tool hash rejected".into());
        }
        let identity = dlpad_engine::file_identity::from_file(&file)?;
        Ok(Pin {
            file,
            expected: tool.sha256.clone(),
            identity,
        })
    }
    fn spec(executable: &Path, args: Vec<OsString>, timeout: Duration) -> ProcessSpec {
        ProcessSpec {
            executable: executable.into(),
            args,
            timeout,
            max_log_bytes: 64 * 1024,
            runtime_environment: None,
        }
    }
    fn temporal_pixel(
        report: &mut Value,
        ffmpeg: &Path,
        source: &Path,
        owned: &Path,
        request: (&str, &str, &str),
        cancel: &Arc<AtomicBool>,
    ) -> Result<[u8; 3], Error> {
        let (label, time, expected) = request;
        let args = |a: &[&str]| a.iter().map(OsString::from).collect::<Vec<_>>();
        let output = owned.join(format!("pixel-{label}.rgb"));
        // Create-only original fixture file; -y is authorized ONLY for this exact
        // retained empty capability, never any prior/user source or artifact.
        let mut original = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&output)?;
        let identity = dlpad_engine::file_identity::from_file(&original)?;
        report["current_phase"] = json!(format!("temporal-pixel-{label}"));
        let mut argv = args(&["-nostdin", "-hide_banner", "-loglevel", "error", "-i"]);
        argv.push(source.as_os_str().into());
        argv.extend(args(&[
            "-ss",
            time,
            "-frames:v",
            "1",
            "-vf",
            "scale=1:1:flags=area",
            "-pix_fmt",
            "rgb24",
            "-f",
            "rawvideo",
            "-y",
        ]));
        argv.push(output.as_os_str().into());
        let execution = execute(
            report,
            "temporal-pixel",
            ffmpeg,
            argv,
            Duration::from_secs(10),
            cancel,
        )?;
        if original.metadata()?.len() != 3
            || dlpad_engine::file_identity::from_file(&File::open(&output)?)? != identity
        {
            return Err("temporal pixel size/identity rejected".into());
        }
        original.seek(SeekFrom::Start(0))?;
        let mut pixel = [0u8; 3];
        original.read_exact(&mut pixel)?;
        let mut extra = [0];
        if original.read(&mut extra)? != 0 {
            return Err("pixel EOF bound".into());
        }
        report["current_temporal_pixel"] = json!({"label":label,"time":time,"expected":expected,"rgb":pixel,"execution":execution});
        // MPEG4/YUV conversion tolerance: dominant channels >=150, absent<=90;
        // green lavfi is RGB(0,128,0), so green threshold>=80, red/blue<=70.
        let [r, g, b] = pixel;
        let matches = match expected {
            "red" => r >= 150 && g <= 90 && b <= 90,
            "green" => g >= 80 && r <= 70 && b <= 70,
            "blue" => b >= 150 && r <= 90 && g <= 90,
            "yellow" => r >= 150 && g >= 150 && b <= 90,
            _ => false,
        };
        if !matches {
            return Err("actual temporal color rejected".into());
        }
        Ok(pixel)
    }
    fn execute(
        report: &mut Value,
        phase: &str,
        executable: &Path,
        args: Vec<OsString>,
        timeout: Duration,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Value, Error> {
        report["current_phase"] = json!(phase);
        let started = Instant::now();
        let result = match process::run(&spec(executable, args.clone(), timeout), cancel) {
            Ok(result) => result,
            Err(error) => {
                report["last_process_error"] = json!({"phase":phase,"kind":format!("{:?}",error.kind()),"cleanup_unconfirmed":process::cleanup_is_unconfirmed(&error),"error":error.to_string()});
                return Err(error.into());
            }
        };
        let evidence = json!({"phase":phase,"argv":args,"elapsed_ms":started.elapsed().as_millis(),"outcome":format!("{:?}",result.outcome),"exit_code":result.exit_code,"logs":result.logs,"truncated":result.truncated});
        report["last_execution"] = evidence.clone();
        if result.outcome != ProcessOutcome::Exited || result.exit_code != Some(0) {
            return Err("real tool did not exit successfully; structured evidence retained".into());
        }
        Ok(evidence)
    }

    fn output_new(path: &Path, value: &Value) -> Result<(), Error> {
        let bytes = serde_json::to_vec_pretty(value)?;
        if bytes.len() > 1024 * 1024 {
            return Err("result bound".into());
        }
        let mut f = OpenOptions::new().create_new(true).write(true).open(path)?;
        f.write_all(&bytes)?;
        f.sync_all()?;
        Ok(())
    }
    struct Server {
        stop: Arc<AtomicBool>,
        done: mpsc::Receiver<Result<u32, String>>,
        worker: Option<thread::JoinHandle<()>>,
        port: u16,
        waited: bool,
    }
    impl Server {
        fn start(source: &Path, deadline: Instant) -> Result<Self, Error> {
            let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
            listener.set_nonblocking(true)?;
            let port = listener.local_addr()?.port();
            let source = File::open(source)?;
            let stop = Arc::new(AtomicBool::new(false));
            let flag = stop.clone();
            let (tx, done) = mpsc::channel();
            let worker = thread::spawn(move || {
                let result = (|| -> Result<u32, String> {
                    let mut requests = 0;
                    while !flag.load(Ordering::Acquire) && Instant::now() < deadline {
                        match listener.accept() {
                            Ok((mut stream, address)) => {
                                if !address.ip().is_loopback() {
                                    return Err("nonloopback peer".into());
                                }
                                requests += 1;
                                if requests > 128 {
                                    return Err("request count bound".into());
                                }
                                if let Err(error) = serve(&mut stream, &source, deadline)
                                    && !matches!(
                                        error.kind(),
                                        io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset
                                    )
                                {
                                    return Err(error.to_string());
                                }
                            }
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                thread::sleep(Duration::from_millis(5))
                            }
                            Err(e) => return Err(e.to_string()),
                        }
                    }
                    Ok(requests)
                })();
                let _ = tx.send(result);
            });
            Ok(Self {
                stop,
                done,
                worker: Some(worker),
                port,
                waited: false,
            })
        }
        fn finish(&mut self) -> Result<u32, Error> {
            self.stop.store(true, Ordering::Release);
            self.waited = true;
            let result = self.done.recv_timeout(Duration::from_secs(5))?;
            if let Some(worker) = self.worker.take() {
                worker.join().map_err(|_| "owned server panic")?;
            }
            result.map_err(Into::into)
        }
    }
    impl Drop for Server {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Release);
            if self.worker.is_some()
                && !self.waited
                && self.done.recv_timeout(Duration::from_secs(5)).is_ok()
                && let Some(worker) = self.worker.take()
            {
                let _ = worker.join();
            }
        }
    }
    fn remaining(deadline: Instant) -> io::Result<Duration> {
        deadline
            .checked_duration_since(Instant::now())
            .filter(|v| !v.is_zero())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "owned HTTP absolute deadline"))
    }
    fn send(stream: &mut TcpStream, mut bytes: &[u8], deadline: Instant) -> io::Result<()> {
        while !bytes.is_empty() {
            stream.set_write_timeout(Some(remaining(deadline)?))?;
            let n = stream.write(bytes)?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "owned HTTP short write",
                ));
            }
            bytes = &bytes[n..];
        }
        Ok(())
    }
    fn serve(stream: &mut TcpStream, source: &File, total_deadline: Instant) -> io::Result<()> {
        // Windows accepted sockets can retain the listener's nonblocking mode.
        // Per-request timeout only bounds blocking I/O after this transition.
        stream.set_nonblocking(false)?;
        let deadline = total_deadline.min(Instant::now() + Duration::from_secs(2));
        let mut header = Vec::new();
        let mut b = [0; 1];
        while !header.ends_with(b"\r\n\r\n") {
            if header.len() >= 8192 {
                return Err(io::Error::other("header bound"));
            }
            stream.set_read_timeout(Some(remaining(deadline)?))?;
            if stream.read(&mut b)? == 0 {
                return Err(io::Error::other("partial request"));
            }
            header.push(b[0]);
        }
        let text =
            std::str::from_utf8(&header).map_err(|_| io::Error::other("request encoding"))?;
        let first = text
            .lines()
            .next()
            .ok_or_else(|| io::Error::other("request missing"))?;
        let fields: Vec<_> = first.split_ascii_whitespace().collect();
        if fields.len() != 3
            || fields[1] != "/synthetic.mp4"
            || !["GET", "HEAD"].contains(&fields[0])
        {
            send(
                stream,
                b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                deadline,
            )?;
            return Ok(());
        }
        let size = source.metadata()?.len();
        if size == 0 || size > 1024 * 1024 {
            return Err(io::Error::other("media size bound"));
        }
        let mut start = 0u64;
        let mut end = size - 1;
        let mut ranged = false;
        for line in text.lines().skip(1) {
            if let Some((key, value)) = line.split_once(':')
                && key.eq_ignore_ascii_case("range")
            {
                if ranged {
                    return Err(io::Error::other("duplicate range"));
                }
                let range = value
                    .trim()
                    .strip_prefix("bytes=")
                    .ok_or_else(|| io::Error::other("range syntax"))?;
                let (a, b) = range
                    .split_once('-')
                    .ok_or_else(|| io::Error::other("range syntax"))?;
                start = a.parse().map_err(|_| io::Error::other("range start"))?;
                if !b.is_empty() {
                    end = b.parse().map_err(|_| io::Error::other("range end"))?
                }
                ranged = true;
            }
        }
        if start > end || end >= size {
            return Err(io::Error::other("range bounds"));
        }
        let bytes = end - start + 1;
        let status = if ranged {
            "206 Partial Content"
        } else {
            "200 OK"
        };
        let range = if ranged {
            format!("Content-Range: bytes {start}-{end}/{size}\r\n")
        } else {
            String::new()
        };
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: video/mp4\r\nContent-Length: {bytes}\r\nAccept-Ranges: bytes\r\n{range}Connection: close\r\n\r\n"
        );
        send(stream, response.as_bytes(), deadline)?;
        if fields[0] == "GET" {
            let mut source = source.try_clone()?;
            source.seek(SeekFrom::Start(start))?;
            let mut buffer = [0; 16384];
            let mut left = bytes;
            while left > 0 {
                remaining(deadline)?;
                let n = source.read(&mut buffer[..left.min(16384) as usize])?;
                if n == 0 {
                    return Err(io::Error::other("short original media"));
                }
                send(stream, &buffer[..n], deadline)?;
                left -= n as u64;
            }
        }
        Ok(())
    }
    pub fn run(manifest: &Path, result_path: &Path) -> Result<(), Error> {
        plain(manifest)?;
        let mut bytes = Vec::new();
        File::open(manifest)?
            .take(64 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > 64 * 1024 {
            return Err("manifest bound".into());
        }
        let input: Input = serde_json::from_slice(&bytes)?;
        if input.schema != 1
            || input.ffmpeg.sha256 != FFMPEG_SHA
            || input.ffmpeg.bytes != 134497280
            || input.ffprobe.sha256 != PROBE_SHA
            || input.ffprobe.bytes != 134296064
            || input.license.bytes != 7651
            || input.license.sha256
                != "da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768"
        {
            return Err("pinned original component rejected".into());
        }
        if !result_path.is_absolute() || result_path.exists() {
            return Err("fresh absolute evidence required".into());
        }
        plain(result_path.parent().ok_or("result parent")?)?;
        let mut pins = [
            pin(&input.yt_dlp)?,
            pin(&input.ffmpeg)?,
            pin(&input.ffprobe)?,
            pin(&input.license)?,
        ];
        let owned = tempfile::Builder::new()
            .prefix("dlpad-real-ffmpeg-owned-")
            .tempdir()?
            .keep();
        let cancel = Arc::new(AtomicBool::new(false));
        let budget = Instant::now() + Duration::from_secs(150);
        let (cancel_tx, cancel_rx) = mpsc::channel();
        let watchdog_cancel = cancel.clone();
        let watchdog = thread::spawn(move || {
            if cancel_rx.recv_timeout(Duration::from_secs(150)).is_err() {
                watchdog_cancel.store(true, Ordering::Release)
            }
        });
        let mut report = json!({"schema":1,"phase":"windows-real-ffmpeg-cut","status":"FAIL","owned_root":owned,"scope":"actual synthetic local media; external executable selection; not supply adoption or managed receipt acceptance","physical_desktop_input_used":false});
        report["original_tools"] = json!({"yt_dlp":input.yt_dlp,"ffmpeg":input.ffmpeg,"ffprobe":input.ffprobe,"license":input.license});
        let tested = (|| -> Result<(), Error> {
            let sentinel = owned.join("foreign-sentinel");
            fs::write(&sentinel, SENTINEL)?;
            let tool_dir = owned.join("외부 FFmpeg 한글 공백 경로");
            fs::create_dir(&tool_dir)?;
            let selected = tool_dir.join("ffmpeg.exe");
            fs::copy(&input.ffmpeg.path, &selected)?;
            let selected_probe = tool_dir.join("ffprobe.exe");
            fs::copy(&input.ffprobe.path, &selected_probe)?;
            let mut probe_pin = pin(&Tool {
                path: selected_probe,
                sha256: PROBE_SHA.into(),
                bytes: 134296064,
            })?;
            let copy_tool = Tool {
                path: selected.clone(),
                sha256: FFMPEG_SHA.into(),
                bytes: 134497280,
            };
            let mut copy_pin = pin(&copy_tool)?;
            let args = |a: &[&str]| a.iter().map(OsString::from).collect::<Vec<_>>();
            let version = execute(
                &mut report,
                "ffmpeg-version",
                &selected,
                args(&["-version"]),
                Duration::from_secs(10),
                &cancel,
            )?;
            if !version["logs"]
                .as_str()
                .is_some_and(|s| s.starts_with(&format!("ffmpeg version {VERSION} ")))
            {
                return Err("fixed FFmpeg version mismatch".into());
            }
            report["ffmpeg_version"] = version;
            let yt_version = execute(
                &mut report,
                "yt-dlp-version",
                &input.yt_dlp.path,
                args(&["--version"]),
                Duration::from_secs(10),
                &cancel,
            )?;
            report["yt_dlp_version"] = yt_version;
            let probe_version = execute(
                &mut report,
                "ffprobe-version",
                &input.ffprobe.path,
                args(&["-version"]),
                Duration::from_secs(10),
                &cancel,
            )?;
            report["ffprobe_version"] = probe_version;
            let source = owned.join("synthetic.mp4");
            let mut generate = args(&[
                "-nostdin",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=c=red:s=160x90:r=30:d=0.5[r];color=c=green:s=160x90:r=30:d=0.5[g];color=c=blue:s=160x90:r=30:d=0.5[b];color=c=yellow:s=160x90:r=30:d=0.5[y];[r][g][b][y]concat=n=4:v=1:a=0",
                "-t",
                "2",
                "-c:v",
                "mpeg4",
                "-g",
                "15",
            ]);
            generate.push(source.clone().into_os_string());
            let generated = execute(
                &mut report,
                "generation",
                &selected,
                generate,
                Duration::from_secs(20),
                &cancel,
            )?;
            report["generation"] = generated;
            let mut original_source = File::open(&source)?;
            let source_sha = hash(&mut original_source)?;
            report["source_sha256_before"] = json!(source_sha);
            report["source_native_identity"] =
                json!(dlpad_engine::file_identity::from_file(&original_source)?);
            let mut source_colors = Vec::new();
            for (label, time, color) in [
                ("source-red", "0.20", "red"),
                ("source-green", "0.70", "green"),
                ("source-blue", "1.20", "blue"),
                ("source-yellow", "1.70", "yellow"),
            ] {
                let rgb = temporal_pixel(
                    &mut report,
                    &selected,
                    &source,
                    &owned,
                    (label, time, color),
                    &cancel,
                )?;
                source_colors.push(json!({"time":time,"expected":color,"rgb":rgb}));
                report["source_reference_colors"] = json!(source_colors);
            }
            let mut server = Server::start(&source, budget)?;
            let db = owned.join("jobs.sqlite");
            let repository = DownloadRepository::open(&db)?;
            let base = DownloadOptions {
                url: SourceUrl::parse(&format!("http://127.0.0.1:{}/synthetic.mp4", server.port))
                    .map_err(|e| format!("{e:?}"))?,
                output_dir: owned.join("한글 저장 폴더"),
                quality: Quality::Best,
                time_range: None,
                advanced: vec![],
                ffmpeg: Some(selected.clone()),
                deno: None,
                final_name_style: None,
                subtitles: None,
            };
            let mut cuts = Vec::new();
            for (id, accurate, begin, end, first_color) in [
                ("windows-fast-cut", false, 500, 1500, "green"),
                ("windows-exact-cut", true, 500, 1500, "green"),
                ("windows-exact-off-gop-cut", true, 350, 1350, "red"),
            ] {
                if Instant::now() >= budget {
                    return Err("fixture total deadline".into());
                }
                let options = DownloadOptions {
                    time_range: Some(
                        TimeRange::new(begin, end, accurate).map_err(|e| format!("{e:?}"))?,
                    ),
                    ..base.clone()
                };
                let plan = options.plan().map_err(|e| format!("{e:?}"))?;
                repository.create_job(id, &options)?;
                let started = Instant::now();
                report["current_cut_id"] = json!(id);
                report["current_phase"] = json!("repository-cut");
                report["current_cut_argv"] = json!(plan.args);
                let completed = match repository.execute(id, &options, &input.yt_dlp.path, &cancel)
                {
                    Ok(completed) => completed,
                    Err(error) => {
                        if let dlpad_engine::download::DownloadError::Execution { stage, result } =
                            &error
                        {
                            report["failed_download"] = json!({"stage":stage,"outcome":format!("{:?}",result.outcome),"exit_code":result.exit_code,"logs":result.logs,"truncated":result.truncated});
                        }
                        report["failed_job"] = json!(repository.job(id)?);
                        report["loopback_server_terminal"] = match server.finish() {
                            Ok(requests) => json!({"status":"STOPPED","requests":requests}),
                            Err(server_error) => {
                                json!({"status":"FAIL","error":server_error.to_string()})
                            }
                        };
                        return Err(error.into());
                    }
                };
                report["current_cut_download"] = json!({"elapsed_ms":started.elapsed().as_millis(),"outcome":format!("{:?}",completed.execution.outcome),"exit_code":completed.execution.exit_code,"logs":completed.execution.logs,"truncated":completed.execution.truncated,"tool_version":completed.tool_version});
                if completed.job.state != JobState::Downloaded
                    || completed.job.artifacts.len() != 1
                    || completed.ffmpeg_version.as_deref() != Some(VERSION)
                {
                    return Err("download completion/version evidence rejected".into());
                }
                let artifact = &completed.job.artifacts[0];
                plain(&artifact.path)?;
                if artifact.bytes == 0 {
                    return Err("empty artifact".into());
                }
                let mut probe = args(&[
                    "-v",
                    "error",
                    "-show_entries",
                    "format=duration:stream=codec_name,width,height",
                    "-of",
                    "json",
                ]);
                probe.push(artifact.path.clone().into_os_string());
                let probed = execute(
                    &mut report,
                    "ffprobe-cut",
                    &input.ffprobe.path,
                    probe,
                    Duration::from_secs(20),
                    &cancel,
                )?;
                report["current_cut_probe"] = probed.clone();
                let probe_json: Value =
                    serde_json::from_str(probed["logs"].as_str().ok_or("probe logs")?)?;
                let duration = probe_json["format"]["duration"]
                    .as_str()
                    .ok_or("actual duration missing")?
                    .parse::<f64>()?;
                report["current_cut_actual_duration"] = json!(duration);
                if !duration.is_finite()
                    || !(0.90..=1.15).contains(&duration)
                    || (accurate && (duration - 1.0).abs() > 0.067)
                {
                    return Err("actual cut duration rejected".into());
                }
                if !probe_json["streams"]
                    .as_array()
                    .is_some_and(|a| a.iter().any(|s| s["width"] == 160 && s["height"] == 90))
                {
                    return Err("actual stream dimensions".into());
                }
                let mut decode = args(&["-nostdin", "-hide_banner", "-loglevel", "error", "-i"]);
                decode.push(artifact.path.clone().into_os_string());
                decode.extend(args(&["-f", "null", "-"]));
                let decoded = execute(
                    &mut report,
                    "decode-cut",
                    &selected,
                    decode,
                    Duration::from_secs(20),
                    &cancel,
                )?;
                let first_pixel = temporal_pixel(
                    &mut report,
                    &selected,
                    &artifact.path,
                    &owned,
                    (&format!("{id}-first"), "0.05", first_color),
                    &cancel,
                )?;
                let last_pixel = temporal_pixel(
                    &mut report,
                    &selected,
                    &artifact.path,
                    &owned,
                    (&format!("{id}-last"), "0.80", "blue"),
                    &cancel,
                )?;
                let reopened = DownloadRepository::open(&db)?;
                if reopened.job(id)?.state != JobState::Downloaded
                    || reopened.ffmpeg_version(id)? != completed.ffmpeg_version
                    || reopened.stored_options(id)?.ffmpeg != Some(selected.clone())
                {
                    return Err("reopened original attempt/options rejected".into());
                }
                let mut output = File::open(&artifact.path)?;
                cuts.push(json!({"id":id,"accurate":accurate,"typed_plan_argv":plan.args,"download_elapsed_ms":started.elapsed().as_millis(),"ffmpeg_version":completed.ffmpeg_version,"artifact":artifact,"artifact_sha256":hash(&mut output)?,"duration":duration,"begin_ms":begin,"end_ms":end,"first_expected":first_color,"first_rgb":first_pixel,"last_expected":"blue","last_rgb":last_pixel,"ffprobe":probed,"decode":decoded,"db_reopen_verified":true}));
                report["cuts"] = json!(cuts);
            }
            // Real process cancellation/deadline are separate from the repository's 24h download timeout.
            let mut controls = Vec::new();
            for interrupted in [true, false] {
                let local_cancel = Arc::new(AtomicBool::new(false));
                let feed = dlpad_engine::metrics::ProgressFeed::default();
                let active = Arc::new(AtomicBool::new(false));
                let (done_tx, done_rx) = mpsc::channel();
                let mut controller = None;
                if interrupted {
                    let observed = feed.clone();
                    let flag = local_cancel.clone();
                    let started = active.clone();
                    controller = Some(thread::spawn(move || {
                        let until = Instant::now() + Duration::from_secs(10);
                        while Instant::now() < until {
                            if observed.processing_snapshot().is_some() {
                                started.store(true, Ordering::Release);
                                flag.store(true, Ordering::Release);
                                return;
                            }
                            if done_rx.recv_timeout(Duration::from_millis(10)).is_ok() {
                                return;
                            }
                        }
                        flag.store(true, Ordering::Release);
                    }));
                }
                let control_args = args(&[
                    "-nostdin",
                    "-hide_banner",
                    "-loglevel",
                    "error",
                    "-re",
                    "-f",
                    "lavfi",
                    "-i",
                    "color=c=blue:s=160x90:r=30",
                    "-t",
                    "30",
                    "-progress",
                    "pipe:2",
                    "-f",
                    "null",
                    "-",
                ]);
                let result = process::run_with_progress(
                    &spec(
                        &selected,
                        control_args.clone(),
                        Duration::from_secs(if interrupted { 15 } else { 3 }),
                    ),
                    &local_cancel,
                    feed.clone(),
                );
                let _ = done_tx.send(());
                if let Some(controller) = controller {
                    controller
                        .join()
                        .map_err(|_| "owned cancellation controller panic")?;
                }
                let result = result?;
                let expected = if interrupted {
                    ProcessOutcome::Cancelled
                } else {
                    ProcessOutcome::TimedOut
                };
                let seen = if interrupted {
                    active.load(Ordering::Acquire)
                } else {
                    feed.processing_snapshot().is_some()
                };
                if !seen || result.outcome != expected {
                    return Err("actualstarted control outcome rejected".into());
                }
                controls.push(json!({"scope":"real FFmpeg ProcessSpec only; not repository cut cancellation", "argv":control_args, "outcome":format!("{:?}",result.outcome),"actual_processing_seen":seen,"exit_code":result.exit_code,"logs":result.logs,"truncated":result.truncated}));
            }
            let missing = DownloadOptions {
                ffmpeg: Some(owned.join("missing-ffmpeg.exe")),
                ..base.clone()
            };
            repository.create_job("missing-external-ffmpeg", &missing)?;
            let failed = repository.execute(
                "missing-external-ffmpeg",
                &missing,
                &input.yt_dlp.path,
                &cancel,
            );
            if failed.is_ok()
                || repository.job("missing-external-ffmpeg")?.state != JobState::Failed
                || !repository
                    .job("missing-external-ffmpeg")?
                    .artifacts
                    .is_empty()
                || repository
                    .ffmpeg_version("missing-external-ffmpeg")?
                    .is_some()
            {
                return Err("repository invalid external selection acceptance failed".into());
            }
            report["invalid_external_selection_failed_without_artifact"] = json!(true);
            report["real_process_control"] = json!(controls);
            report["cuts"] = json!(cuts);
            report["loopback_requests"] = json!(server.finish()?);
            if hash(&mut original_source)? != source_sha
                || hash(&mut copy_pin.file)? != FFMPEG_SHA
                || hash(&mut probe_pin.file)? != PROBE_SHA
                || fs::read(&sentinel)? != SENTINEL
            {
                return Err("owned source/tool/sentinel changed".into());
            }
            report["source_sha256_before"] = json!(source_sha);
            report["source_sha256_after"] = json!(hash(&mut original_source)?);
            report["external_copy_sha_preserved"] = json!(true);
            report["foreign_sentinel_preserved"] = json!(true);
            report["repository_recovery_count"] = json!(repository.recover()?);
            Ok(())
        })();
        let _ = cancel_tx.send(());
        watchdog.join().map_err(|_| "owned watchdog panic")?;
        if owned.join("synthetic.mp4").is_file() {
            match File::open(owned.join("synthetic.mp4"))
                .map_err(|e| -> Error { e.into() })
                .and_then(|mut file| hash(&mut file))
            {
                Ok(digest) => report["source_sha256_after"] = json!(digest),
                Err(error) => report["source_postflight_error"] = json!(error.to_string()),
            }
        }
        report["foreign_sentinel_preserved"] =
            json!(fs::read(owned.join("foreign-sentinel")).is_ok_and(|v| v == SENTINEL));
        let pins_preserved = pins
            .iter_mut()
            .try_fold(true, |ok, p| -> Result<bool, Error> {
                Ok(ok
                    && hash(&mut p.file)? == p.expected
                    && dlpad_engine::file_identity::from_file(&p.file)? == p.identity)
            })?;
        report["original_tool_hashes_preserved"] = json!(pins_preserved);
        report["original_native_identities"] =
            json!(pins.iter().map(|p| &p.identity).collect::<Vec<_>>());
        match tested {
            Ok(()) if pins_preserved => report["status"] = json!("PASS"),
            Ok(()) => report["error"] = json!("original tool changed"),
            Err(error) => report["error"] = json!(error.to_string()),
        }
        output_new(result_path, &report)?;
        println!(
            "{}",
            serde_json::to_string(
                &json!({"schema":1,"status":report["status"],"evidence":result_path})
            )?
        );
        if report["status"] != "PASS" {
            return Err("real Windows media acceptance failed; evidence retained".into());
        }
        Ok(())
    }
    // Include inside the Windows `native` module of the actual example.
    #[cfg(test)]
    mod transport_tests {
        use super::*;
        use std::io::{Read, Write};
        fn source() -> (tempfile::TempDir, PathBuf, Vec<u8>) {
            let root = tempfile::tempdir().unwrap();
            let path = root.path().join("owned-synthetic.mp4");
            let bytes: Vec<u8> = (0..4096).map(|i| (i % 251) as u8).collect();
            fs::write(&path, &bytes).unwrap();
            (root, path, bytes)
        }
        fn response(server: &Server, request: &[u8]) -> Vec<u8> {
            let mut client =
                TcpStream::connect((std::net::Ipv4Addr::LOCALHOST, server.port)).unwrap();
            client
                .set_read_timeout(Some(Duration::from_secs(4)))
                .unwrap();
            client
                .set_write_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            client.write_all(request).unwrap();
            let mut bytes = Vec::new();
            client.take(16385).read_to_end(&mut bytes).unwrap();
            assert!(bytes.len() <= 16384);
            bytes
        }
        fn parts(bytes: &[u8]) -> (&str, &[u8]) {
            let split = bytes.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
            (
                std::str::from_utf8(&bytes[..split]).unwrap(),
                &bytes[split..],
            )
        }
        #[test]
        fn nonblocking_listener_actual_accept_head_get_range_preserve_original_bytes() {
            let (_root, path, original) = source();
            // Exercise the REAL Server worker, including conversion of the accepted
            // nonblocking Windows socket to blocking BEFORE read/write timeouts.
            let mut server = Server::start(&path, Instant::now() + Duration::from_secs(9)).unwrap();
            let head = response(
                &server,
                b"HEAD /synthetic.mp4 HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
            );
            let (headers, body) = parts(&head);
            assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"));
            assert!(headers.contains("Content-Length: 4096\r\n"));
            assert!(body.is_empty());
            let get = response(
                &server,
                b"GET /synthetic.mp4 HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
            );
            let (headers, body) = parts(&get);
            assert!(headers.starts_with("HTTP/1.1 200 OK\r\n"));
            assert_eq!(body, original);
            let ranged = response(
                &server,
                b"GET /synthetic.mp4 HTTP/1.1\r\nHost: 127.0.0.1\r\nRange: bytes=17-48\r\n\r\n",
            );
            let (headers, body) = parts(&ranged);
            assert!(headers.starts_with("HTTP/1.1 206 Partial Content\r\n"));
            assert!(headers.contains("Content-Range: bytes 17-48/4096\r\n"));
            assert_eq!(body, &original[17..49]);
            assert_eq!(server.finish().unwrap(), 3);
            assert_eq!(fs::read(path).unwrap(), original);
        }
        fn failing_header(write: Option<&[u8]>, close_send: bool) -> (io::Error, Duration) {
            let (_root, path, original) = source();
            let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
            listener.set_nonblocking(true).unwrap();
            let address = listener.local_addr().unwrap();
            let (accepted_tx, accepted_rx) = mpsc::sync_channel(1);
            let (done_tx, done_rx) = mpsc::sync_channel(1);
            let source_file = File::open(&path).unwrap();
            let worker = thread::spawn(move || {
                let accept_deadline = Instant::now() + Duration::from_secs(2);
                let mut stream = loop {
                    match listener.accept() {
                        Ok((stream, peer)) => {
                            assert!(peer.ip().is_loopback());
                            break stream;
                        }
                        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                            assert!(Instant::now() < accept_deadline);
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(error) => panic!("owned accept error: {error}"),
                    }
                };
                // Leave the accepted socket mode untouched: the production serve
                // must normalize it, so removing that fix fails the idle regression.
                let start = Instant::now();
                accepted_tx.send(()).unwrap();
                let result = serve(&mut stream, &source_file, start + Duration::from_secs(3));
                done_tx.send((result, start.elapsed())).unwrap();
            });
            let mut client = TcpStream::connect(address).unwrap();
            accepted_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            if let Some(bytes) = write {
                client.write_all(bytes).unwrap();
            }
            if close_send {
                client.shutdown(std::net::Shutdown::Write).unwrap();
            }
            let (result, elapsed) = done_rx.recv_timeout(Duration::from_secs(4)).unwrap();
            drop(client);
            worker.join().unwrap();
            assert_eq!(fs::read(path).unwrap(), original);
            (result.unwrap_err(), elapsed)
        }
        #[test]
        fn idle_header_has_absolute_two_second_budget_not_wouldblock_10035() {
            let (error, elapsed) = failing_header(None, false);
            assert!(matches!(
                error.kind(),
                io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
            ));
            // Windows SO_RCVTIMEO may map to WouldBlock, but must actually wait for
            // the configured deadline rather than immediately fail with WSA10035.
            assert!(
                elapsed >= Duration::from_millis(1500),
                "premature error {error:?}: {elapsed:?}"
            );
            assert!(elapsed < Duration::from_secs(4));
        }
        #[test]
        fn partial_header_eof_is_explicit_failure_and_original_source_is_unchanged() {
            let (error, elapsed) = failing_header(Some(b"GET /synthetic.mp4 HTTP/1.1\r\n"), true);
            assert_eq!(error.kind(), io::ErrorKind::Other);
            assert_eq!(error.to_string(), "partial request");
            assert!(elapsed < Duration::from_secs(2));
        }
        #[test]
        fn partial_but_open_header_uses_same_absolute_budget() {
            let (error, elapsed) = failing_header(Some(b"GET /synthetic.mp4 HTTP/1.1\r\n"), false);
            assert!(matches!(
                error.kind(),
                io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
            ));
            assert!(elapsed >= Duration::from_millis(1500));
            assert!(elapsed < Duration::from_secs(4));
        }
    }
}
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if dlpad_engine::process::guardian_entry()? {
        return Ok(());
    }
    dlpad_engine::process::register_self_guardian()?;
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: absolute-tools-manifest.json fresh-result.json".into());
    }
    native::run(
        std::path::Path::new(&args[0]),
        std::path::Path::new(&args[1]),
    )
}
#[cfg(not(windows))]
fn main() {
    println!("Windows only; native fixture not executed");
}
