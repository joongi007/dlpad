//! Bounded metadata/frame service; owned processes run outside the UI thread.
#[cfg(windows)]
pub mod local_export;
use crate::process::{self, ProcessOutcome, ProcessSpec};
use dlpad_core::options::SourceUrl;
use serde_json::Value;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
const WIDTH: usize = 320;
const HEIGHT: usize = 180;
const FRAME_BYTES: usize = WIDTH * HEIGHT * 3;
const TITLE_BYTES: usize = 512;
const MAX_DURATION_MS: u64 = 7 * 24 * 60 * 60 * 1000;
#[derive(Clone)]
pub struct Tools {
    pub ytdlp: PathBuf,
    pub deno: Option<PathBuf>,
    pub ffprobe: PathBuf,
    pub ffmpeg: PathBuf,
}
/// Execution-only borrowed management authority. The caller owns cleanup and finish.
pub struct PreviewSession<'a> {
    pub managed: Option<&'a crate::tools::ManagedExecution>,
    #[cfg(windows)]
    pub runtime_environment: Option<&'a crate::runtime_environment::WindowsManagedDenoEnvironment>,
}
impl PreviewSession<'_> {
    fn external() -> Self {
        Self {
            managed: None,
            #[cfg(windows)]
            runtime_environment: None,
        }
    }
}
// Tools must be current verified immutable original tools, pinned by caller for worker lifetime.
// Do not expose these paths or remote URLs through Debug/log/status serializers.
#[derive(Clone)]
pub enum Source {
    Url(String),
    Local(PathBuf),
}
pub struct Prepared {
    pub title: String,
    pub duration_ms: u64,
    input: Source,
    local_pin: Option<Arc<LocalPin>>,
}
struct LocalPin {
    file: std::fs::File,
    identity: crate::file_identity::NativeFileIdentity,
    ancestors: Vec<std::fs::File>,
}
pub struct Frame {
    pub seek_ms: u64,
    pub rgb: Vec<u8>,
}
impl Frame {
    pub fn width(&self) -> usize {
        WIDTH
    }
    pub fn height(&self) -> usize {
        HEIGHT
    }
}
pub struct FrameBatch {
    pub start_ms: u64,
    pub rgb: Vec<u8>,
}
impl FrameBatch {
    pub fn width(&self) -> usize {
        WIDTH
    }
    pub fn height(&self) -> usize {
        HEIGHT
    }
    pub fn frames(&self) -> usize {
        self.rgb.len() / FRAME_BYTES
    }
    pub fn fps(&self) -> u32 {
        6
    }
}
impl Prepared {
    /// Worker-only borrowed original local capability. The caller retains this Prepared
    /// through handoff and must revalidate immediately before dispatch. A URL is never a file.
    pub fn verified_local_file(&self) -> Result<(&Path, &std::fs::File), Error> {
        let Source::Local(path) = &self.input else {
            return Err(Error::InvalidInput);
        };
        let pin = self.local_pin.as_ref().ok_or(Error::InvalidInput)?;
        verify_local(path, pin)?;
        Ok((path, &pin.file))
    }
    pub fn is_local(&self) -> bool {
        matches!(&self.input, Source::Local(_))
    }
}
pub enum Request {
    Prepare(Source),
    Seek {
        prepared: Arc<Prepared>,
        position_ms: u64,
    },
    PreviewFrames {
        prepared: Arc<Prepared>,
        position_ms: u64,
    },
}
pub enum Product {
    Prepared(Arc<Prepared>),
    Frame(Frame),
    Frames(FrameBatch),
    Finished,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidInput,
    InvalidMetadata,
    FrameUnavailable,
    UnsupportedMedia,
    Cancelled,
    TimedOut,
    OutputLimit,
    ToolFailed,
    OwnershipUnconfirmed,
}
fn check(cancel: &AtomicBool, deadline: Instant) -> Result<(), Error> {
    if cancel.load(Ordering::Acquire) {
        Err(Error::Cancelled)
    } else if Instant::now() >= deadline {
        Err(Error::TimedOut)
    } else {
        Ok(())
    }
}
#[derive(Clone, Copy)]
enum CaptureRole {
    ExtractorMetadata,
    ProbeMetadata,
    DirectProcessing,
}
fn capture(
    tool: &PathBuf,
    args: Vec<OsString>,
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
    maximum: Duration,
    session: &PreviewSession<'_>,
    role: CaptureRole,
) -> Result<Vec<u8>, Error> {
    check(cancel, deadline)?;
    let tool_pin = pin_local(tool, cancel, deadline)?;
    let timeout = maximum.min(deadline.saturating_duration_since(Instant::now()));
    let spec = ProcessSpec {
        executable: tool.clone(),
        args,
        timeout,
        max_log_bytes: 16384,
        #[cfg(windows)]
        runtime_environment: if matches!(role, CaptureRole::ExtractorMetadata) {
            session.runtime_environment.cloned()
        } else {
            None
        },
    };
    #[cfg(windows)]
    let result = if let Some(managed) = session.managed {
        let stage = if matches!(role, CaptureRole::DirectProcessing) {
            crate::tools::ManagedReaderStage::DirectProcessing
        } else {
            crate::tools::ManagedReaderStage::Metadata
        };
        managed.run_tool(&spec, cancel, true, None, stage)
    } else {
        process::run_metadata(&spec, cancel)
    };
    #[cfg(not(windows))]
    let result = {
        let _ = (session, role);
        process::run_metadata(&spec, cancel)
    };
    let result = result.map_err(|e| {
        if process::cleanup_is_unconfirmed(&e) {
            Error::OwnershipUnconfirmed
        } else {
            Error::ToolFailed
        }
    })?;
    verify_local(tool, &tool_pin)?;
    finished(result.execution.outcome, result.execution.exit_code)?;
    check(cancel, deadline)?;
    Ok(result.stdout)
}
fn finished(outcome: ProcessOutcome, code: Option<i32>) -> Result<(), Error> {
    match outcome {
        ProcessOutcome::Cancelled => Err(Error::Cancelled),
        ProcessOutcome::TimedOut => Err(Error::TimedOut),
        ProcessOutcome::OutputLimit => Err(Error::OutputLimit),
        ProcessOutcome::Exited if code == Some(0) => Ok(()),
        ProcessOutcome::Exited => Err(Error::ToolFailed),
    }
}
fn args(v: &[&str]) -> Vec<OsString> {
    v.iter().map(OsString::from).collect()
}
fn strict_url(s: &str) -> Result<(), Error> {
    SourceUrl::parse(s).map_err(|_| Error::InvalidInput)?;
    // SourceUrl is syntax only, not a network/content/trust validator. Tighten hostname syntax
    // here without new dependency; no credentials or fragment accepted for preview transport.
    let rest = s
        .strip_prefix("https://")
        .or_else(|| s.strip_prefix("http://"))
        .ok_or(Error::InvalidInput)?;
    let authority = rest
        .split(['/', '?', '#'])
        .next()
        .ok_or(Error::InvalidInput)?;
    if authority.is_empty()
        || authority
            .chars()
            .any(|c| !c.is_ascii() || c.is_control() || matches!(c, '\\' | '@'))
        || s.contains('#')
    {
        return Err(Error::InvalidInput);
    }
    Ok(())
}
fn milliseconds(v: &Value) -> Result<u64, Error> {
    let seconds = v
        .as_f64()
        .or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))
        .ok_or(Error::InvalidMetadata)?;
    if !seconds.is_finite() || seconds <= 0.0 || seconds * 1000.0 > MAX_DURATION_MS as f64 {
        return Err(Error::InvalidMetadata);
    }
    let ms = (seconds * 1000.0).round() as u64;
    if ms == 0 {
        Err(Error::InvalidMetadata)
    } else {
        Ok(ms)
    }
}
fn title(v: Option<&Value>) -> String {
    v.and_then(Value::as_str)
        .unwrap_or("Media")
        .chars()
        .filter(|c| !c.is_control())
        .scan(0usize, |n, c| {
            *n += c.len_utf8();
            (*n <= TITLE_BYTES).then_some(c)
        })
        .collect()
}
fn metadata(bytes: &[u8], remote: bool) -> Result<Prepared, Error> {
    if bytes.len() > process::MAX_METADATA_BYTES {
        return Err(Error::OutputLimit);
    }
    let v: Value = serde_json::from_slice(bytes).map_err(|_| Error::InvalidMetadata)?;
    if remote {
        if v.get("entries").is_some() || v.get("is_live").and_then(Value::as_bool) == Some(true) {
            return Err(Error::UnsupportedMedia);
        }
        let direct = v
            .get("url")
            .and_then(Value::as_str)
            .ok_or(Error::UnsupportedMedia)?;
        strict_url(direct)?;
        // One direct video source, including video-only DASH. No implicit separate-stream joining.
        if v.get("vcodec")
            .and_then(Value::as_str)
            .is_none_or(|s| s == "none")
        {
            return Err(Error::UnsupportedMedia);
        }
        Ok(Prepared {
            title: title(v.get("title")),
            duration_ms: milliseconds(v.get("duration").ok_or(Error::InvalidMetadata)?)?,
            input: Source::Url(direct.into()),
            local_pin: None,
        })
    } else {
        if !v.get("streams").and_then(Value::as_array).is_some_and(|a| {
            a.iter()
                .any(|s| s.get("codec_type").and_then(Value::as_str) == Some("video"))
        }) {
            return Err(Error::UnsupportedMedia);
        }
        let f = v.get("format").ok_or(Error::InvalidMetadata)?;
        Ok(Prepared {
            title: title(f.get("tags").and_then(|v| v.get("title"))),
            duration_ms: milliseconds(f.get("duration").ok_or(Error::InvalidMetadata)?)?,
            input: Source::Local(PathBuf::new()),
            local_pin: None,
        })
    }
}
// Ignore ambient runtime discovery even when no runtime was explicitly selected.
fn preview_runtime_arguments(argv: &mut Vec<OsString>, selected: Option<&Path>) {
    argv.push("--no-js-runtimes".into());
    if let Some(path) = selected {
        let mut runtime = OsString::from("deno:");
        runtime.push(path);
        argv.extend(["--js-runtimes".into(), runtime]);
    }
}
pub fn execute(
    tools: &Tools,
    request: Request,
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
) -> Result<Product, Error> {
    execute_with_session(tools, request, cancel, deadline, PreviewSession::external())
}
pub fn execute_with_session(
    tools: &Tools,
    request: Request,
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
    session: PreviewSession<'_>,
) -> Result<Product, Error> {
    check(cancel, deadline)?;
    match request {
        Request::Prepare(Source::Url(url)) => {
            strict_url(&url)?;
            let mut argv = args(&[
                "--ignore-config",
                "-J",
                "--skip-download",
                "--no-playlist",
                "--no-plugin-dirs",
                "--no-warnings",
                "--no-progress",
                "--socket-timeout",
                "10",
                "--retries",
                "0",
                "--extractor-retries",
                "0",
                "-f",
                "bestvideo*[height<=360]/worstvideo*",
            ]);
            let deno_pin = tools
                .deno
                .as_ref()
                .map(|path| pin_local(path, cancel, deadline))
                .transpose()?;
            preview_runtime_arguments(&mut argv, tools.deno.as_deref());
            #[cfg(windows)]
            if let Some(policy) = session.runtime_environment {
                let selected = tools.deno.as_ref().ok_or(Error::InvalidInput)?;
                let managed = session.managed.ok_or(Error::InvalidInput)?;
                managed
                    .validate_runtime_environment(policy, selected, cancel)
                    .map_err(|_| check(cancel, deadline).err().unwrap_or(Error::InvalidInput))?;
                check(cancel, deadline)?;
            }
            argv.extend(["--".into(), url.into()]);
            let bytes = capture(
                &tools.ytdlp,
                argv,
                cancel,
                deadline,
                Duration::from_secs(25),
                &session,
                CaptureRole::ExtractorMetadata,
            )?;
            if let (Some(path), Some(pin)) = (&tools.deno, &deno_pin) {
                verify_local(path, pin)?;
            }
            Ok(Product::Prepared(Arc::new(metadata(&bytes, true)?)))
        }
        Request::Prepare(Source::Local(path)) => {
            // Original local file handle is retained through probe. This is not file deletion permission.
            let pin = pin_local(&path, cancel, deadline)?;
            let mut argv = args(&[
                "-v",
                "error",
                "-protocol_whitelist",
                "file",
                "-show_entries",
                "format=duration:format_tags=title:stream=codec_type",
                "-of",
                "json",
                "-i",
            ]);
            argv.push(path.as_os_str().into());
            let bytes = capture(
                &tools.ffprobe,
                argv,
                cancel,
                deadline,
                Duration::from_secs(10),
                &session,
                CaptureRole::ProbeMetadata,
            )?;
            let mut prepared = metadata(&bytes, false)?;
            if prepared.title == "Media" {
                prepared.title = title(
                    path.file_name()
                        .and_then(|s| s.to_str())
                        .map(|s| serde_json::Value::String(s.into()))
                        .as_ref(),
                );
            }
            prepared.input = Source::Local(path);
            prepared.local_pin = Some(Arc::new(pin));
            Ok(Product::Prepared(Arc::new(prepared)))
        }
        Request::Seek {
            prepared,
            position_ms,
        } => decode(
            tools,
            prepared,
            position_ms,
            false,
            cancel,
            deadline,
            &session,
        ),
        Request::PreviewFrames {
            prepared,
            position_ms,
        } => decode(
            tools,
            prepared,
            position_ms,
            true,
            cancel,
            deadline,
            &session,
        ),
    }
}
fn frame_count(bytes: usize) -> Result<usize, Error> {
    if bytes == 0 || !bytes.is_multiple_of(FRAME_BYTES) || bytes > 12 * FRAME_BYTES {
        return Err(Error::InvalidMetadata);
    }
    Ok(bytes / FRAME_BYTES)
}
fn decode(
    tools: &Tools,
    prepared: Arc<Prepared>,
    position_ms: u64,
    batch: bool,
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
    session: &PreviewSession<'_>,
) -> Result<Product, Error> {
    check(cancel, deadline)?;
    if position_ms >= prepared.duration_ms {
        return Err(Error::InvalidInput);
    }
    let (_pin, input, protocol) = match &prepared.input {
        Source::Local(p) => {
            let pin = prepared.local_pin.as_ref().ok_or(Error::InvalidInput)?;
            verify_local(p, pin)?;
            (Some(pin.clone()), p.as_os_str().to_owned(), "file")
        }
        Source::Url(u) => {
            strict_url(u)?;
            (None, u.into(), "http,https,tcp,tls")
        }
    };
    let mut candidates = if batch {
        vec![position_ms]
    } else {
        vec![
            position_ms,
            position_ms.saturating_sub(167),
            position_ms.saturating_sub(1000),
        ]
    };
    candidates.dedup();
    for seek_ms in candidates {
        let stamp = format!("{}.{:03}", seek_ms / 1000, seek_ms % 1000);
        let mut argv = args(&[
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            "error",
            "-threads",
            "1",
            "-filter_threads",
            "1",
            "-max_alloc",
            "33554432",
            "-protocol_whitelist",
            protocol,
        ]);
        if matches!(&prepared.input, Source::Url(_)) {
            argv.extend(args(&["-rw_timeout", "10000000"]))
        }
        argv.extend(args(&["-ss", &stamp, "-i"]));
        argv.push(input.clone());
        argv.extend(args(&[
            "-map", "0:v:0", "-an", "-sn", "-dn", "-threads", "1",
        ]));
        let filter = if batch {
            "fps=6,scale=320:180:force_original_aspect_ratio=decrease,pad=320:180:(ow-iw)/2:(oh-ih)/2"
        } else {
            "scale=320:180:force_original_aspect_ratio=decrease,pad=320:180:(ow-iw)/2:(oh-ih)/2"
        };
        if batch {
            argv.extend(args(&["-t", "2", "-frames:v", "12"]))
        } else {
            argv.extend(args(&["-frames:v", "1"]))
        }
        argv.extend(args(&[
            "-vf", filter, "-pix_fmt", "rgb24", "-f", "rawvideo", "pipe:1",
        ]));
        let rgb = capture(
            &tools.ffmpeg,
            argv,
            cancel,
            deadline,
            Duration::from_secs(15),
            session,
            CaptureRole::DirectProcessing,
        )?;
        if let (Source::Local(p), Some(pin)) = (&prepared.input, &_pin) {
            verify_local(p, pin)?;
        }
        if rgb.is_empty() {
            if batch && prepared.duration_ms.saturating_sub(position_ms) <= 1000 {
                return Ok(Product::Finished);
            }
            if !batch {
                continue;
            }
        }
        let count = frame_count(rgb.len())?;
        return if batch {
            Ok(Product::Frames(FrameBatch {
                start_ms: position_ms,
                rgb,
            }))
        } else if count == 1 {
            Ok(Product::Frame(Frame { seek_ms, rgb }))
        } else {
            Err(Error::InvalidMetadata)
        };
    }
    Err(Error::FrameUnavailable)
}
fn pin_local(path: &PathBuf, cancel: &AtomicBool, deadline: Instant) -> Result<LocalPin, Error> {
    check(cancel, deadline)?;
    if !path.is_absolute() || path.as_os_str().len() > 32767 || path.components().count() > 256 {
        return Err(Error::InvalidInput);
    }
    #[cfg(windows)]
    let file = {
        use std::os::windows::fs::OpenOptionsExt;
        // Original read pin denies write/delete, and refuses final reparse following.
        let f = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .custom_flags(0x00200000)
            .open(path)
            .map_err(|_| Error::InvalidInput)?;
        let m = f.metadata().map_err(|_| Error::InvalidInput)?;
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err(Error::InvalidInput);
        }
        f
    };
    #[cfg(not(windows))]
    let file = {
        if std::fs::symlink_metadata(path)
            .map_err(|_| Error::InvalidInput)?
            .file_type()
            .is_symlink()
        {
            return Err(Error::InvalidInput);
        }
        std::fs::File::open(path).map_err(|_| Error::InvalidInput)?
    };
    if !file.metadata().map_err(|_| Error::InvalidInput)?.is_file() {
        return Err(Error::InvalidInput);
    }
    let identity = crate::file_identity::from_file(&file).map_err(|_| Error::InvalidInput)?;
    let mut ancestors = Vec::new();
    #[cfg(windows)]
    {
        use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
        for parent in path.ancestors().skip(1).take(256) {
            check(cancel, deadline)?;
            let cap = std::fs::OpenOptions::new()
                .access_mode(0x80)
                .share_mode(3)
                .custom_flags(0x02000000 | 0x00200000)
                .open(parent)
                .map_err(|_| Error::InvalidInput)?;
            let m = cap.metadata().map_err(|_| Error::InvalidInput)?;
            if !m.is_dir() || m.file_attributes() & 0x400 != 0 {
                return Err(Error::InvalidInput);
            }
            ancestors.push(cap);
            check(cancel, deadline)?;
        }
    }
    let pin = LocalPin {
        file,
        identity,
        ancestors,
    };
    verify_local(path, &pin)?;
    check(cancel, deadline)?;
    Ok(pin)
}
fn verify_local(path: &Path, pin: &LocalPin) -> Result<(), Error> {
    let _bounded_retained_ancestor_count = pin.ancestors.len();
    if crate::file_identity::from_file(&pin.file).map_err(|_| Error::InvalidInput)? != pin.identity
    {
        return Err(Error::InvalidInput);
    }
    #[cfg(windows)]
    let current =
        crate::file_identity::open_regular_no_reparse(path).map_err(|_| Error::InvalidInput)?;
    #[cfg(not(windows))]
    let current = std::fs::File::open(path).map_err(|_| Error::InvalidInput)?;
    if crate::file_identity::from_file(&current).map_err(|_| Error::InvalidInput)? != pin.identity {
        return Err(Error::InvalidInput);
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verified_local_file_retains_original_and_rejects_remote_missing_pin_and_wrong_path() {
        let root = std::env::temp_dir().join(format!(
            "dlpad-open-capability-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("synthetic.mp4");
        std::fs::write(&path, b"synthetic media capability").unwrap();
        let stop = AtomicBool::new(false);
        let pin = pin_local(&path, &stop, Instant::now() + Duration::from_secs(2)).unwrap();
        let mut prepared = Prepared {
            title: "Synthetic".into(),
            duration_ms: 3000,
            input: Source::Local(path.clone()),
            local_pin: Some(Arc::new(pin)),
        };
        let (original, file) = prepared.verified_local_file().unwrap();
        assert_eq!(original, path);
        assert_eq!(
            crate::file_identity::from_file(file).unwrap(),
            prepared.local_pin.as_ref().unwrap().identity
        );
        prepared.input = Source::Url("https://example.test/video".into());
        assert!(prepared.verified_local_file().is_err());
        let other = root.join("different.mp4");
        std::fs::write(&other, b"different original identity").unwrap();
        prepared.input = Source::Local(other.clone());
        assert!(prepared.verified_local_file().is_err());
        prepared.input = Source::Local(path.clone());
        prepared.local_pin = None;
        assert!(prepared.verified_local_file().is_err());
        // A missing pin is not recreated by inspecting the present path.
        drop(prepared);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(other).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn cancel_deadline_priorities() {
        let c = AtomicBool::new(true);
        assert_eq!(
            check(&c, Instant::now() - Duration::from_secs(1)),
            Err(Error::Cancelled)
        );
        c.store(false, Ordering::Release);
        assert_eq!(
            check(&c, Instant::now() - Duration::from_secs(1)),
            Err(Error::TimedOut)
        );
    }
    #[test]
    fn invalid_url_no_transport() {
        for u in [
            "file:///a",
            "https://user:secret@host/v",
            "https://host/v#fragment",
            "https://ho\\st/v",
        ] {
            assert_eq!(strict_url(u), Err(Error::InvalidInput));
        }
    }
    #[test]
    fn invalid_metadata_and_unbounded_duration() {
        assert!(metadata(b"{", true).is_err());
        assert!(milliseconds(&serde_json::json!(1e20)).is_err());
        assert!(milliseconds(&serde_json::json!(0.0001)).is_err());
        assert!(metadata(br#"{"entries":[],"duration":1}"#, true).is_err());
    }
    #[test]
    fn exact_single_direct_video() {
        let p=metadata(br#"{"url":"https://media.invalid/v.mp4","vcodec":"h264","duration":1.25,"title":"Test"}"#,true).unwrap();
        assert_eq!(p.duration_ms, 1250);
        assert_eq!(p.title, "Test");
    }
    #[test]
    fn relative_local_refused_before_tool() {
        assert!(
            pin_local(
                &PathBuf::from("relative.mp4"),
                &AtomicBool::new(false),
                Instant::now() + Duration::from_secs(1)
            )
            .is_err()
        );
    }
    #[test]
    fn process_timeout_not_success() {
        assert_eq!(
            finished(ProcessOutcome::TimedOut, Some(0)),
            Err(Error::TimedOut)
        );
        assert_eq!(
            finished(ProcessOutcome::Cancelled, None),
            Err(Error::Cancelled)
        );
        assert_eq!(
            finished(ProcessOutcome::Exited, Some(2)),
            Err(Error::ToolFailed)
        );
    }
    #[test]
    fn invalid_selected_runtime_refused_before_extractor_and_cancel_keeps_priority() {
        for deno in [PathBuf::new(), PathBuf::from("relative/deno.exe")] {
            let tools = Tools {
                ytdlp: PathBuf::from("must-not-run"),
                deno: Some(deno),
                ffmpeg: PathBuf::new(),
                ffprobe: PathBuf::new(),
            };
            let cancel = Arc::new(AtomicBool::new(false));
            let request = || Request::Prepare(Source::Url("https://video.invalid/watch".into()));
            assert!(matches!(
                execute(
                    &tools,
                    request(),
                    &cancel,
                    Instant::now() + Duration::from_secs(1)
                ),
                Err(Error::InvalidInput)
            ));
            cancel.store(true, Ordering::Release);
            assert!(matches!(
                execute(
                    &tools,
                    request(),
                    &cancel,
                    Instant::now() + Duration::from_secs(1)
                ),
                Err(Error::Cancelled)
            ));
        }
    }
    #[test]
    fn bounded_full_frame_batches() {
        for bytes in [0, FRAME_BYTES - 1, 13 * FRAME_BYTES] {
            assert!(frame_count(bytes).is_err())
        }
        assert_eq!(frame_count(FRAME_BYTES), Ok(1));
        assert_eq!(frame_count(12 * FRAME_BYTES), Ok(12));
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "root owned host supplies pinned tools and an explicitly authorized public URL"]
    fn actual_public_metadata_and_consecutive_batches() {
        let url = std::env::var("DLPAD_PUBLIC_TEST_URL").unwrap();
        assert!(matches!(
            url.as_str(),
            "https://youtu.be/lxOLdQXE2yw?si=D_OjqkJT2d5WKMl7"
                | "https://chzzk.naver.com/video/15441192"
        ));
        let ffmpeg = PathBuf::from(std::env::var_os("DLPAD_MEDIA_FFMPEG").unwrap());
        let tools = Tools {
            ffprobe: ffmpeg.parent().unwrap().join("ffprobe.exe"),
            ffmpeg,
            ytdlp: PathBuf::from(std::env::var_os("DLPAD_PUBLIC_YTDLP").unwrap()),
            deno: Some(PathBuf::from(
                std::env::var_os("DLPAD_PUBLIC_DENO").unwrap(),
            )),
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let deadline = || Instant::now() + Duration::from_secs(30);
        let prepared = match execute(
            &tools,
            Request::Prepare(Source::Url(url)),
            &cancel,
            deadline(),
        )
        .unwrap()
        {
            Product::Prepared(prepared) => prepared,
            _ => panic!("metadata product"),
        };
        for position_ms in [0, 2000, 4000] {
            let result = execute(
                &tools,
                Request::PreviewFrames {
                    prepared: prepared.clone(),
                    position_ms,
                },
                &cancel,
                deadline(),
            );
            match result {
                Ok(Product::Frames(batch)) => assert_eq!(batch.frames(), 12),
                Err(error) => panic!("public batch at {position_ms}ms: {error:?}"),
                _ => panic!("batch product"),
            }
        }
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "root private host supplies real pinned tools and synthetic video"]
    fn actual_synthetic_metadata_seek_batch_cancel_preserves_source() {
        let file = PathBuf::from(std::env::var_os("DLPAD_MEDIA_FIXTURE").unwrap());
        let ffmpeg = PathBuf::from(std::env::var_os("DLPAD_MEDIA_FFMPEG").unwrap());
        assert!(file.is_absolute() && ffmpeg.is_absolute());
        let tools = Tools {
            ffprobe: ffmpeg.parent().unwrap().join("ffprobe.exe"),
            ffmpeg,
            ytdlp: PathBuf::new(),
            deno: None,
        };
        let original = std::fs::read(&file).unwrap();
        assert!(original.len() < 1_048_576);
        let cancel = Arc::new(AtomicBool::new(false));
        let deadline = || Instant::now() + Duration::from_secs(30);
        let prepared = match execute(
            &tools,
            Request::Prepare(Source::Local(file.clone())),
            &cancel,
            deadline(),
        )
        .unwrap()
        {
            Product::Prepared(v) => v,
            _ => panic!("metadata"),
        };
        assert!((1900..=2100).contains(&prepared.duration_ms));
        for (position_ms, red) in [(0, true), (1200, false), (1999, false)] {
            let frame = match execute(
                &tools,
                Request::Seek {
                    prepared: prepared.clone(),
                    position_ms,
                },
                &cancel,
                deadline(),
            )
            .unwrap()
            {
                Product::Frame(v) => v,
                _ => panic!("frame"),
            };
            assert_eq!(frame.rgb.len(), FRAME_BYTES);
            let center = (90 * 320 + 160) * 3;
            let r = frame.rgb[center];
            let b = frame.rgb[center + 2];
            assert!(if red {
                r > 200 && b < 40
            } else {
                b > 200 && r < 40
            });
        }
        let batch = match execute(
            &tools,
            Request::PreviewFrames {
                prepared: prepared.clone(),
                position_ms: 0,
            },
            &cancel,
            deadline(),
        )
        .unwrap()
        {
            Product::Frames(v) => v,
            _ => panic!("batch"),
        };
        assert_eq!(batch.frames(), 12);
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            execute(
                &tools,
                Request::Seek {
                    prepared: prepared.clone(),
                    position_ms: 0
                },
                &cancel,
                deadline()
            ),
            Err(Error::Cancelled)
        ));
        cancel.store(false, Ordering::Release);
        assert!(matches!(
            execute(
                &tools,
                Request::Seek {
                    prepared,
                    position_ms: 0
                },
                &cancel,
                Instant::now() - Duration::from_secs(1)
            ),
            Err(Error::TimedOut)
        ));
        assert_eq!(std::fs::read(file).unwrap(), original);
    }
    #[test]
    fn preview_runtime_selection_never_enables_ambient_discovery() {
        let mut disabled = args(&["--ignore-config"]);
        preview_runtime_arguments(&mut disabled, None);
        assert_eq!(disabled, args(&["--ignore-config", "--no-js-runtimes"]));
        let selected = PathBuf::from("/explicit runtime/deno");
        let mut explicit = Vec::new();
        preview_runtime_arguments(&mut explicit, Some(&selected));
        assert_eq!(explicit.len(), 3);
        assert_eq!(explicit[0], OsString::from("--no-js-runtimes"));
        assert_eq!(explicit[1], OsString::from("--js-runtimes"));
        let mut expected = OsString::from("deno:");
        expected.push(&selected);
        assert_eq!(
            explicit[2], expected,
            "selected executable remains a single argv value"
        );
    }
    #[test]
    fn external_session_rejects_input_and_preserves_cancellation_priority_before_tools() {
        let tools = Tools {
            ytdlp: PathBuf::new(),
            deno: None,
            ffprobe: PathBuf::new(),
            ffmpeg: PathBuf::new(),
        };
        let request = || Request::Prepare(Source::Url("file:///never-run".into()));
        let cancel = Arc::new(AtomicBool::new(false));
        assert!(matches!(
            execute_with_session(
                &tools,
                request(),
                &cancel,
                Instant::now() + Duration::from_secs(1),
                PreviewSession::external()
            ),
            Err(Error::InvalidInput)
        ));
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            execute_with_session(
                &tools,
                request(),
                &cancel,
                Instant::now() + Duration::from_secs(1),
                PreviewSession::external()
            ),
            Err(Error::Cancelled)
        ));
        assert!(matches!(
            execute(
                &tools,
                request(),
                &cancel,
                Instant::now() + Duration::from_secs(1)
            ),
            Err(Error::Cancelled)
        ));
    }
}
