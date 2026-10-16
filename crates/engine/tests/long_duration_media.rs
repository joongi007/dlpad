//! Opt-in real-tool integration using generated media and a loopback server only.
#![cfg(unix)]
use dlpad_core::{
    jobs::JobState,
    options::{DownloadOptions, Quality, SourceUrl, TimeRange},
};
use dlpad_engine::{
    download::DownloadRepository,
    process::{self, ProcessOutcome, ProcessSpec},
};
use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};
struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[test]
#[ignore = "requires explicit DLPAD_SMOKE_YTDLP and DLPAD_SMOKE_FFMPEG executable paths"]
fn eight_hour_timeline_downloads_and_cuts_without_eight_hour_wall_clock() {
    let tool = PathBuf::from(std::env::var_os("DLPAD_SMOKE_YTDLP").expect("explicit yt-dlp path"));
    let ffmpeg =
        PathBuf::from(std::env::var_os("DLPAD_SMOKE_FFMPEG").expect("explicit ffmpeg path"));
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("synthetic.mp4");
    let started = std::time::Instant::now();
    let cancel = Arc::new(AtomicBool::new(false));
    let generated = process::run(
        &ProcessSpec {
            #[cfg(windows)]
            runtime_environment: None,
            executable: ffmpeg.clone(),
            args: [
                "-nostdin",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=c=blue:s=160x90:r=1/60",
                "-t",
                "28800",
                "-c:v",
                "mpeg4",
            ]
            .into_iter()
            .map(Into::into)
            .chain([source.clone().into_os_string()])
            .collect(),
            timeout: Duration::from_secs(30),
            max_log_bytes: process::MAX_LOG_BYTES,
        },
        &cancel,
    )
    .unwrap();
    assert_eq!(generated.outcome, ProcessOutcome::Exited);
    assert_eq!(generated.exit_code, Some(0), "{}", generated.logs);
    assert!(std::fs::metadata(&source).unwrap().len() < 16 * 1024 * 1024);
    let ffprobe =
        PathBuf::from(std::env::var_os("DLPAD_SMOKE_FFPROBE").expect("explicit ffprobe path"));
    let original = probe(&ffprobe, &source, &cancel);
    assert_eq!(original["format"]["duration"], "28800.000000");
    assert_eq!(original["streams"][0]["nb_frames"], "480");
    let mut server = Server(
        Command::new("python3")
            .args([
                "-u",
                "-c",
                r#"
import http.server, functools, sys
handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=sys.argv[1])
server = http.server.HTTPServer(('127.0.0.1', 0), handler)
print(server.server_port, flush=True)
server.serve_forever()
"#,
            ])
            .arg(directory.path())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let mut port = String::new();
    BufReader::new(server.0.stdout.take().unwrap())
        .read_line(&mut port)
        .unwrap();
    let port: u16 = port.trim().parse().unwrap();
    let options = DownloadOptions {
        url: SourceUrl::parse(&format!("http://127.0.0.1:{port}/synthetic.mp4")).unwrap(),
        output_dir: directory.path().join("한글 저장 폴더"),
        quality: Quality::Best,
        time_range: None,
        advanced: vec![],
        ffmpeg: None,
        deno: None,
        final_name_style: None,
        subtitles: None,
    };
    let database = directory.path().join("jobs.sqlite");
    let repo = DownloadRepository::open(&database).unwrap();
    repo.create_job("real-local-media", &options).unwrap();
    let progress = dlpad_engine::metrics::ProgressFeed::default();
    let downloaded = repo
        .execute_with_progress(
            "real-local-media",
            &options,
            &tool,
            &cancel,
            progress.clone(),
        )
        .unwrap_or_else(|error| {
            if let dlpad_engine::download::DownloadError::Execution { result, .. } = &error {
                eprintln!("{}", result.logs);
            }
            panic!("{error}");
        });
    assert_eq!(downloaded.job.state, JobState::Downloaded);
    assert!(
        progress.snapshot().is_some(),
        "real tool must emit the configured machine progress template: {}",
        downloaded.execution.logs
    );
    let artifact = &downloaded.job.artifacts[0];
    assert_eq!(
        std::fs::read(&artifact.path).unwrap(),
        std::fs::read(&source).unwrap()
    );
    drop(repo);
    let reopened = DownloadRepository::open(&database).unwrap();
    assert_eq!(
        reopened.job("real-local-media").unwrap().state,
        JobState::Downloaded
    );
    assert_eq!(reopened.recover().unwrap(), 0);
    let cut_options = DownloadOptions {
        time_range: Some(TimeRange::new(28_680_000, 28_800_000, true).unwrap()),
        ffmpeg: Some(ffmpeg.clone()),
        ..options.clone()
    };
    reopened
        .create_job("eight-hour-tail", &cut_options)
        .unwrap();
    let cut = reopened
        .execute("eight-hour-tail", &cut_options, &tool, &cancel)
        .unwrap();
    assert_eq!(cut.job.state, JobState::Downloaded);
    let tail = &cut.job.artifacts[0].path;
    assert!(std::fs::metadata(tail).unwrap().len() < 16 * 1024 * 1024);
    let observed = probe(&ffprobe, tail, &cancel);
    let tail_duration: f64 = observed["format"]["duration"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!((tail_duration - 120.0).abs() < 0.2, "tail={observed}");
    let decode = process::run(
        &ProcessSpec {
            #[cfg(windows)]
            runtime_environment: None,
            executable: ffmpeg.clone(),
            args: ["-nostdin", "-v", "error", "-i"]
                .into_iter()
                .map(Into::into)
                .chain([tail.clone().into_os_string()])
                .chain(["-f", "null", "-"].into_iter().map(Into::into))
                .collect(),
            timeout: Duration::from_secs(30),
            max_log_bytes: 65536,
        },
        &cancel,
    )
    .unwrap();
    assert_eq!(decode.exit_code, Some(0), "{}", decode.logs);
    drop(reopened);
    let final_repo = DownloadRepository::open(&database).unwrap();
    assert_eq!(final_repo.recover().unwrap(), 0);
    assert_eq!(
        final_repo.job("eight-hour-tail").unwrap().state,
        JobState::Downloaded
    );
    println!(
        "{}",
        serde_json::json!({
            "scope":"eight-hour synthetic media timeline; wall-clock soak not verified",
            "duration_seconds":28800, "source_frames":480,
            "source_bytes":std::fs::metadata(&source).unwrap().len(),
            "tail_duration_seconds":tail_duration, "tail_decodes":true,
            "completion_and_reopen_verified":true, "elapsed_seconds":started.elapsed().as_secs_f64(),
            "actual_media_used":false, "server_bind":"127.0.0.1",
        })
    );
}

fn probe(
    tool: &std::path::Path,
    media: &std::path::Path,
    cancel: &Arc<AtomicBool>,
) -> serde_json::Value {
    let result = process::run_metadata(
        &ProcessSpec {
            #[cfg(windows)]
            runtime_environment: None,
            executable: tool.into(),
            args: [
                "-v",
                "error",
                "-show_entries",
                "format=duration:stream=nb_frames",
                "-of",
                "json",
            ]
            .into_iter()
            .map(Into::into)
            .chain([media.as_os_str().to_owned()])
            .collect(),
            timeout: Duration::from_secs(10),
            max_log_bytes: 65536,
        },
        cancel,
    )
    .unwrap();
    assert_eq!(
        result.execution.exit_code,
        Some(0),
        "{}",
        result.execution.logs
    );
    serde_json::from_slice(&result.stdout).unwrap()
}
