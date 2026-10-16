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
fn real_tool_downloads_synthetic_media_and_persists_completion() {
    let tool = PathBuf::from(std::env::var_os("DLPAD_SMOKE_YTDLP").expect("explicit yt-dlp path"));
    let ffmpeg =
        PathBuf::from(std::env::var_os("DLPAD_SMOKE_FFMPEG").expect("explicit ffmpeg path"));
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("synthetic.mp4");
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
                "color=c=blue:s=160x90:r=10",
                "-t",
                "2",
                "-c:v",
                "mpeg4",
            ]
            .into_iter()
            .map(Into::into)
            .chain([source.clone().into_os_string()])
            .collect(),
            timeout: Duration::from_secs(20),
            max_log_bytes: process::MAX_LOG_BYTES,
        },
        &cancel,
    )
    .unwrap();
    assert_eq!(generated.outcome, ProcessOutcome::Exited);
    assert_eq!(generated.exit_code, Some(0), "{}", generated.logs);
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
        std::fs::read(source).unwrap()
    );
    drop(repo);
    let reopened = DownloadRepository::open(&database).unwrap();
    assert_eq!(
        reopened.job("real-local-media").unwrap().state,
        JobState::Downloaded
    );
    assert_eq!(reopened.recover().unwrap(), 0);
    let explicit_dir = directory.path().join("외부 FFmpeg 공백 경로");
    std::fs::create_dir(&explicit_dir).unwrap();
    let explicit_ffmpeg = explicit_dir.join("ffmpeg");
    std::fs::copy(&ffmpeg, &explicit_ffmpeg).unwrap();
    for (id, accurate) in [("real-fast-cut", false), ("real-accurate-cut", true)] {
        let cut_options = DownloadOptions {
            time_range: Some(TimeRange::new(200, 1200, accurate).unwrap()),
            ffmpeg: Some(explicit_ffmpeg.clone()),
            ..options.clone()
        };
        reopened.create_job(id, &cut_options).unwrap();
        let processing = dlpad_engine::metrics::ProgressFeed::default();
        let cut = reopened
            .execute_with_progress(id, &cut_options, &tool, &cancel, processing.clone())
            .unwrap_or_else(|error| {
                if let dlpad_engine::download::DownloadError::Execution { result, .. } = &error {
                    eprintln!("{}", result.logs);
                }
                panic!("{id}: {error}");
            });
        assert_eq!(cut.job.state, JobState::Downloaded);
        #[cfg(target_os = "linux")]
        {
            let sample = processing
                .processing_snapshot()
                .expect("FFmpeg FIFO progress end record");
            assert!(sample.ended, "{}", cut.execution.logs);
            assert!(sample.output_time_us.is_some());
            let partial = cut_options.output_dir.join(".dlpad-partials").join(id);
            if partial.exists() {
                assert!(std::fs::read_dir(partial).unwrap().all(|entry| {
                    !entry
                        .unwrap()
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".progress-")
                }));
            }
        }
        assert!(cut.ffmpeg_version.is_some());
        assert_eq!(reopened.ffmpeg_version(id).unwrap(), cut.ffmpeg_version);
        let evidence = DownloadRepository::open(&database).unwrap();
        assert_eq!(evidence.ffmpeg_version(id).unwrap(), cut.ffmpeg_version);
        let decoded = process::run(
            &ProcessSpec {
                #[cfg(windows)]
                runtime_environment: None,
                executable: PathBuf::from(std::env::var_os("DLPAD_SMOKE_FFMPEG").unwrap()),
                args: ["-nostdin", "-hide_banner", "-loglevel", "error", "-i"]
                    .into_iter()
                    .map(Into::into)
                    .chain([cut.job.artifacts[0].path.clone().into_os_string()])
                    .chain(["-f", "null", "-"].into_iter().map(Into::into))
                    .collect(),
                timeout: Duration::from_secs(20),
                max_log_bytes: process::MAX_LOG_BYTES,
            },
            &cancel,
        )
        .unwrap();
        assert_eq!(decoded.outcome, ProcessOutcome::Exited);
        assert_eq!(decoded.exit_code, Some(0), "{id}: {}", decoded.logs);
    }
    // A test-only argv adapter supplies synthetic extractor metadata, while real
    // yt-dlp performs the media/subtitle requests and emits production markers.
    use std::os::unix::fs::PermissionsExt;
    let subtitle = b"WEBVTT\n\n00:00:00.000 --> 00:00:01.000\nSynthetic subtitle\n";
    std::fs::write(directory.path().join("manual.vtt"), subtitle).unwrap();
    let metadata_path = directory.path().join("subtitle.info.json");
    let origin = format!("http://127.0.0.1:{port}");
    let info = serde_json::json!({"id":"synthetic-subtitles", "title":"Synthetic subtitles", "extractor":"generic",
        "webpage_url":options.url.as_str(), "url":options.url.as_str(), "ext":"mp4",
        "subtitles":{"ko":[{"url":format!("{origin}/manual.vtt"),"ext":"vtt"}]}});
    std::fs::write(&metadata_path, serde_json::to_vec(&info).unwrap()).unwrap();
    let wrapper = directory.path().join("yt-dlp-subtitle-fixture");
    let code = format!(
        "#!/usr/bin/python3\nimport os,sys\na=sys.argv[1:]\nif '--version' not in a:\n i=a.index('--');a=a[:i]+['--load-info-json',{}]\nos.execv({},[{}]+a)\n",
        serde_json::to_string(&metadata_path).unwrap(),
        serde_json::to_string(&tool).unwrap(),
        serde_json::to_string(&tool).unwrap()
    );
    std::fs::write(&wrapper, code).unwrap();
    std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o700)).unwrap();
    let selected = DownloadOptions {
        final_name_style: None,

        subtitles: Some(dlpad_core::options::SubtitleOptions {
            language: dlpad_core::options::SubtitleLanguage::Korean,
            auto_generated: false,
        }),
        ..options.clone()
    };
    reopened
        .create_job("real-media-subtitle", &selected)
        .unwrap();
    let result = reopened
        .execute("real-media-subtitle", &selected, &wrapper, &cancel)
        .unwrap();
    assert_eq!(result.job.state, JobState::Downloaded);
    assert_eq!(result.job.artifacts.len(), 2);
    assert_eq!(
        std::fs::read(selected.output_dir.join("real-media-subtitle.ko.vtt")).unwrap(),
        subtitle
    );
    let persisted = DownloadRepository::open(&database)
        .unwrap()
        .job("real-media-subtitle")
        .unwrap();
    assert_eq!(persisted.artifacts, result.job.artifacts);
}
