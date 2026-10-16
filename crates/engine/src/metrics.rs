//! Bounded, on-demand observations. Interface counters include other applications' traffic.
pub mod ffmpeg;

#[cfg(windows)]
#[path = "metrics/windows_counter.rs"]
mod windows_counter;
/// Owned by one sampling worker. Existing stateless/Linux APIs remain unchanged.
#[derive(Default)]
pub struct NetworkObserver {
    #[cfg(windows)]
    windows: windows_counter::Counter,
}
impl NetworkObserver {
    pub fn reset(&mut self) {
        #[cfg(windows)]
        self.windows.reset();
    }
    pub fn sample(&mut self, interface: &str) -> io::Result<Option<NetworkSample>> {
        #[cfg(windows)]
        {
            self.windows.sample(interface)
        }
        #[cfg(not(windows))]
        {
            sample_network_interface(interface)
        }
    }
}
/// Local Windows adapter names, bounded and read only. Call from a worker.
#[cfg(windows)]
pub fn observed_interfaces() -> io::Result<Vec<String>> {
    windows_counter::aliases()
}
/// Local default-route observation; does not identify a download destination.
#[cfg(windows)]
pub fn automatic_observed_interface() -> io::Result<Option<String>> {
    windows_counter::automatic_alias()
}
/// UI policy for the stateful backend; does not alter the Linux snapshot parser.
pub fn valid_observed_interface(interface: &str) -> bool {
    #[cfg(windows)]
    {
        windows_counter::valid_alias(interface)
    }
    #[cfg(not(windows))]
    {
        valid_interface(interface)
    }
}

use std::io;
use std::path::Path;
use std::time::Instant;

pub const MAX_METRIC_LINE_BYTES: usize = 64 * 1024;
const MAX_NETWORK_SNAPSHOT_BYTES: usize = 128 * 1024;
const MAX_INTERFACE_ROWS: usize = 256;

#[derive(Debug, Clone, PartialEq)]
pub struct DownloadMetrics {
    pub bytes_per_second: Option<f64>,
    pub eta_seconds: Option<f64>,
    pub downloaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
    pub estimated_total_bytes: Option<f64>,
    /// Current-file transfer completion, never publication/job completion.
    pub transfer_finished: bool,
}
impl DownloadMetrics {
    pub fn transfer_fraction(&self) -> Option<(f32, bool)> {
        let downloaded = self.downloaded_bytes?;
        if let Some(total) = self.total_bytes.filter(|value| *value > 0) {
            if downloaded > total {
                return None;
            }
            let fraction = if downloaded == total {
                1.0
            } else {
                (downloaded as f64 / total as f64).min(0.999) as f32
            };
            return Some((fraction, false));
        }
        let total = self
            .estimated_total_bytes
            .filter(|value| value.is_finite() && *value > 0.0 && *value <= u64::MAX as f64)?;
        if downloaded as f64 > total {
            return None;
        }
        let fraction = downloaded as f64 / total;
        let fraction = if self.transfer_finished {
            fraction
        } else {
            fraction.min(0.999)
        };
        Some((fraction as f32, true))
    }
}
/// A coalesced observation, never a queue or persistent job state.
#[derive(Debug, Clone, Default)]
pub struct ProgressFeed(
    std::sync::Arc<std::sync::Mutex<Option<(Instant, DownloadMetrics)>>>,
    std::sync::Arc<std::sync::Mutex<ProcessingFeedState>>,
);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProcessingMetrics {
    pub output_time_us: Option<u64>,
    pub speed: Option<f64>,
    pub eta_seconds: Option<f64>,
    pub ended: bool,
}
#[derive(Debug, Default)]
struct ProcessingFeedState {
    parser: ffmpeg::FfmpegProgressParser,
    #[cfg(target_os = "linux")]
    dedicated_parser: ffmpeg::FfmpegProgressParser,
    #[cfg(target_os = "linux")]
    dedicated_active: bool,
    recording: bool,
    ended: bool,
    duration_us: Option<u64>,
    latest: Option<(Instant, ProcessingMetrics)>,
}
impl std::hash::Hash for ProgressFeed {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&(std::sync::Arc::as_ptr(&self.0) as usize), state);
    }
}
impl ProgressFeed {
    pub(crate) fn observe(&self, line: &str) {
        let Some(json) = line.strip_prefix("DLPAD_PROGRESS:") else {
            return;
        };
        let Ok(sample) = parse_ytdlp_progress(json) else {
            if let Ok(mut latest) = self.0.lock() {
                *latest = None;
            }
            return;
        };
        if let Ok(mut latest) = self.0.lock() {
            *latest = Some((Instant::now(), sample));
        }
    }
    /// Known output duration only. Changing it clears prior processing evidence.
    pub fn set_processing_duration(&self, duration_us: Option<u64>) {
        if let Ok(mut state) = self.1.lock() {
            *state = ProcessingFeedState {
                duration_us,
                ..Default::default()
            };
        }
    }
    /// Only the FFmpeg-bearing stderr stream may call this observer.
    /// A strict video frame key starts a record; human speed text alone is ignored.
    pub(crate) fn observe_processing(&self, line: &str) {
        let Ok(mut state) = self.1.lock() else {
            return;
        };
        #[cfg(target_os = "linux")]
        if state.dedicated_active {
            return;
        }
        if line.len() > MAX_METRIC_LINE_BYTES || line.contains(['\n', '\r']) {
            state.latest = None;
            state.recording = false;
            state.parser = Default::default();
            return;
        }
        let frame = line.strip_prefix("frame=").is_some_and(|v| {
            !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()) && v.parse::<u64>().is_ok()
        });
        if frame {
            if state.recording || state.ended {
                state.parser = Default::default();
                state.latest = None;
            }
            state.recording = true;
            state.ended = false;
        }
        if !state.recording {
            return;
        }
        let mut emitted = None;
        state
            .parser
            .push(line.as_bytes(), |sample| emitted = Some(sample));
        state.parser.push(b"\n", |sample| emitted = Some(sample));
        if let Some(sample) = emitted {
            state.recording = false;
            match sample {
                Ok(sample) => {
                    state.ended = sample.ended;
                    state.latest = Some((
                        Instant::now(),
                        ProcessingMetrics {
                            output_time_us: sample.output_time_us,
                            speed: sample.speed,
                            eta_seconds: sample.eta_seconds(state.duration_us),
                            ended: sample.ended,
                        },
                    ));
                }
                Err(_) => {
                    state.latest = None;
                    state.parser = Default::default();
                }
            }
        }
    }
    /// Dedicated machine transport only; never mixed stdout/stderr human logs.
    #[cfg(target_os = "linux")]
    pub(crate) fn begin_dedicated_processing(&self) {
        if let Ok(mut state) = self.1.lock() {
            let duration_us = state.duration_us;
            *state = ProcessingFeedState {
                duration_us,
                dedicated_active: true,
                ..Default::default()
            };
        }
    }
    #[cfg(target_os = "linux")]
    pub(crate) fn observe_dedicated_processing(&self, bytes: &[u8]) {
        let Ok(mut state) = self.1.lock() else {
            return;
        };
        if !state.dedicated_active {
            return;
        }
        // The machine pipe may carry serial FFmpeg processes. Reset after each
        // complete end record so the next audio/video record starts a new epoch.
        for byte in bytes {
            let mut emitted = None;
            state
                .dedicated_parser
                .push(std::slice::from_ref(byte), |sample| emitted = Some(sample));
            if let Some(sample) = emitted {
                match sample {
                    Ok(sample) => {
                        state.latest = Some((
                            Instant::now(),
                            ProcessingMetrics {
                                output_time_us: sample.output_time_us,
                                speed: sample.speed,
                                eta_seconds: sample.eta_seconds(state.duration_us),
                                ended: sample.ended,
                            },
                        ));
                        if sample.ended {
                            state.dedicated_parser = Default::default();
                        }
                    }
                    Err(_) => state.latest = None,
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    pub(crate) fn finish_dedicated_processing(&self) {
        if let Ok(mut state) = self.1.lock() {
            let mut invalid = false;
            state
                .dedicated_parser
                .finish(|sample| invalid |= sample.is_err());
            if invalid {
                state.latest = None;
            }
            state.dedicated_active = false;
        }
    }
    #[cfg(target_os = "linux")]
    pub(crate) fn invalidate_processing(&self) {
        if let Ok(mut state) = self.1.lock() {
            state.latest = None;
        }
    }
    pub fn processing_snapshot(&self) -> Option<ProcessingMetrics> {
        self.processing_snapshot_at(Instant::now())
    }
    fn processing_snapshot_at(&self, now: Instant) -> Option<ProcessingMetrics> {
        let state = self.1.lock().ok()?;
        let (sampled, sample) = state.latest.as_ref()?;
        (now.checked_duration_since(*sampled)? <= std::time::Duration::from_secs(3))
            .then_some(*sample)
    }
    pub fn snapshot(&self) -> Option<DownloadMetrics> {
        self.snapshot_at(Instant::now())
    }
    fn snapshot_at(&self, now: Instant) -> Option<DownloadMetrics> {
        let latest = self.0.lock().ok()?;
        let (sampled, sample) = latest.as_ref()?;
        (now.checked_duration_since(*sampled)? <= std::time::Duration::from_secs(3))
            .then(|| sample.clone())
    }
}

#[cfg(test)]
mod progress_feed_tests {
    use super::*;
    #[cfg(target_os = "linux")]
    #[test]
    fn dedicated_audio_epochs_ignore_mixed_stderr_and_keep_download_independent() {
        let feed = ProgressFeed::default();
        feed.observe(r#"DLPAD_PROGRESS:{"speed":123,"eta":2}"#);
        feed.set_processing_duration(Some(5_000_000));
        feed.begin_dedicated_processing();
        for line in ["frame=1", "out_time_us=1", "speed=9x", "progress=end"] {
            feed.observe_processing(line);
        }
        assert_eq!(feed.processing_snapshot(), None);
        for chunk in b"bitrate=N/A\nout_time_us=2000000\nspeed=2x\nprogress=continue\n".chunks(3) {
            feed.observe_dedicated_processing(chunk);
        }
        assert_eq!(feed.processing_snapshot().unwrap().eta_seconds, Some(1.5));
        feed.observe_dedicated_processing(b"out_time_us=5000000\nspeed=2x\nprogress=end\nbitrate=N/A\nout_time_us=0\nspeed=N/A\nprogress=continue\n");
        assert!(!feed.processing_snapshot().unwrap().ended);
        assert_eq!(feed.processing_snapshot().unwrap().speed, None);
        assert_eq!(feed.snapshot().unwrap().bytes_per_second, Some(123.0));
        feed.observe_dedicated_processing(b"speed=2x");
        feed.finish_dedicated_processing();
        assert_eq!(feed.processing_snapshot(), None);
    }
    #[test]
    #[ignore = "explicit DLPAD_TEST_PROCESSING_LOG from owned synthetic FFmpeg probe required"]
    fn actual_ffmpeg_stderr_records_reach_processing_feed() {
        let path =
            std::env::var_os("DLPAD_TEST_PROCESSING_LOG").expect("explicit owned stderr log");
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .unwrap()
            .take(256 * 1024 + 1)
            .read_to_end(&mut bytes)
            .unwrap();
        assert!(bytes.len() <= 256 * 1024);
        let text = std::str::from_utf8(&bytes).unwrap();
        let feed = ProgressFeed::default();
        feed.set_processing_duration(Some(6_000_000));
        feed.observe(r#"DLPAD_PROGRESS:{"speed":1000,"eta":2}"#);
        let mut eta_observations = 0;
        for line in text.lines() {
            feed.observe_processing(line.trim_end_matches('\r'));
            if line == "progress=continue"
                && let Some(sample) = feed.processing_snapshot()
                && let Some(eta) = sample.eta_seconds
            {
                assert!(eta.is_finite() && eta >= 0.0);
                eta_observations += 1;
            }
        }
        println!("actual non-final duration-based ETA observations={eta_observations}");
        let processing = feed
            .processing_snapshot()
            .expect("real machine frame/end record");
        assert!(processing.ended);
        assert!(processing.output_time_us.unwrap() > 0);
        assert!(processing.speed.is_some());
        assert_eq!(processing.eta_seconds, None);
        assert_eq!(feed.snapshot().unwrap().bytes_per_second, Some(1000.0));
        println!("actual stderr processing={processing:?}");
    }
    #[test]
    fn processing_frames_are_separate_bounded_unknown_and_stale() {
        let feed = ProgressFeed::default();
        feed.observe(r#"DLPAD_PROGRESS:{"speed":1000,"eta":2}"#);
        for line in ["speed=9x", "out_time_us=100", "progress=continue"] {
            feed.observe_processing(line);
        }
        assert_eq!(feed.processing_snapshot(), None);
        feed.set_processing_duration(Some(4_000_000));
        for line in [
            "frame=1",
            "out_time_us=1500000",
            "speed=2.5x",
            "progress=continue",
        ] {
            feed.observe_processing(line);
        }
        assert_eq!(feed.processing_snapshot().unwrap().eta_seconds, Some(1.0));
        assert_eq!(feed.snapshot().unwrap().bytes_per_second, Some(1000.0));
        assert_eq!(
            feed.processing_snapshot_at(Instant::now() + std::time::Duration::from_secs(4)),
            None
        );
        for line in [
            "frame=2",
            "out_time_us=N/A",
            "speed=N/A",
            "progress=continue",
        ] {
            feed.observe_processing(line);
        }
        assert_eq!(feed.processing_snapshot().unwrap().speed, None);
        for line in ["frame=3", "human stats", "speed=3x", "progress=continue"] {
            feed.observe_processing(line);
        }
        assert_eq!(feed.processing_snapshot(), None);
        for line in ["frame=4", "out_time_us=4000000", "speed=2x", "progress=end"] {
            feed.observe_processing(line);
        }
        assert!(feed.processing_snapshot().unwrap().ended);
        assert_eq!(feed.processing_snapshot().unwrap().eta_seconds, None);
        for line in ["frame=1", "out_time_us=0", "speed=0x", "progress=continue"] {
            feed.observe_processing(line);
        }
        assert_eq!(feed.processing_snapshot().unwrap().output_time_us, Some(0));
        feed.observe_processing(&"x".repeat(MAX_METRIC_LINE_BYTES + 1));
        assert_eq!(feed.processing_snapshot(), None);
        feed.set_processing_duration(None);
        for line in [
            "frame=1",
            "out_time_us=1000000",
            "speed=2x",
            "progress=continue",
        ] {
            feed.observe_processing(line);
        }
        assert_eq!(feed.processing_snapshot().unwrap().eta_seconds, None);
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn actual_loopback_observations_are_selected_not_summed() {
        let first = sample_network_interface("lo").unwrap().unwrap();
        let next = sample_network_interface("lo").unwrap().unwrap();
        assert_eq!(first.interface, "lo");
        assert!(next.rate_since(&first).is_some());
        assert!(sample_network_interface("../dev").is_err());
    }
    #[test]
    fn latest_sample_unknown_invalid_and_stale_do_not_reuse_old_numbers() {
        let feed = ProgressFeed::default();
        assert_eq!(feed.snapshot(), None);
        feed.observe(r#"DLPAD_PROGRESS:{"speed":1000,"eta":2}"#);
        assert_eq!(feed.snapshot().unwrap().bytes_per_second, Some(1000.0));
        feed.observe(r#"DLPAD_PROGRESS:{"speed":null,"eta":"NA"}"#);
        assert_eq!(feed.snapshot().unwrap().eta_seconds, None);
        feed.observe("DLPAD_PROGRESS:broken");
        assert_eq!(feed.snapshot(), None);
        feed.observe(r#"DLPAD_PROGRESS:{"speed":0,"eta":0}"#);
        assert_eq!(feed.snapshot().unwrap().bytes_per_second, Some(0.0));
        assert_eq!(
            feed.snapshot_at(Instant::now() + std::time::Duration::from_secs(4)),
            None
        );
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricsParseError {
    TooLarge,
    Malformed,
    InvalidNumber,
}

/// JSON emitted by the fixed yt-dlp template, including bounded current-file byte counters.
/// Missing/null/NA fields remain unknown; zero is a measured value, never an unknown substitute.
pub fn parse_ytdlp_progress(line: &str) -> Result<DownloadMetrics, MetricsParseError> {
    if line.len() > MAX_METRIC_LINE_BYTES {
        return Err(MetricsParseError::TooLarge);
    }
    if line.contains(['\n', '\r']) {
        return Err(MetricsParseError::Malformed);
    }
    let value: serde_json::Value =
        serde_json::from_str(line).map_err(|_| MetricsParseError::Malformed)?;
    let object = value.as_object().ok_or(MetricsParseError::Malformed)?;
    fn number(value: Option<&serde_json::Value>) -> Result<Option<f64>, MetricsParseError> {
        match value {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(serde_json::Value::String(s))
                if matches!(s.as_str(), "NA" | "N/A" | "Unknown") =>
            {
                Ok(None)
            }
            Some(serde_json::Value::Number(n)) => {
                let n = n.as_f64().ok_or(MetricsParseError::InvalidNumber)?;
                if n.is_finite() && n >= 0.0 {
                    Ok(Some(n))
                } else {
                    Err(MetricsParseError::InvalidNumber)
                }
            }
            _ => Err(MetricsParseError::InvalidNumber),
        }
    }
    fn bytes(value: Option<&serde_json::Value>) -> Result<Option<u64>, MetricsParseError> {
        match value {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(serde_json::Value::String(value))
                if matches!(value.as_str(), "NA" | "N/A" | "Unknown") =>
            {
                Ok(None)
            }
            Some(serde_json::Value::Number(value)) => value
                .as_u64()
                .map(Some)
                .ok_or(MetricsParseError::InvalidNumber),
            _ => Err(MetricsParseError::InvalidNumber),
        }
    }
    let transfer_finished = match object.get("status") {
        None | Some(serde_json::Value::Null) => false,
        Some(serde_json::Value::String(value))
            if matches!(value.as_str(), "NA" | "N/A" | "Unknown" | "downloading") =>
        {
            false
        }
        Some(serde_json::Value::String(value)) if value == "finished" => true,
        _ => return Err(MetricsParseError::InvalidNumber),
    };
    let estimated_total_bytes = number(object.get("estimated_total"))?;
    if estimated_total_bytes.is_some_and(|value| value > u64::MAX as f64) {
        return Err(MetricsParseError::InvalidNumber);
    }
    Ok(DownloadMetrics {
        bytes_per_second: number(object.get("speed"))?,
        eta_seconds: number(object.get("eta"))?,
        downloaded_bytes: bytes(object.get("downloaded"))?,
        total_bytes: bytes(object.get("total"))?,
        estimated_total_bytes,
        transfer_finished,
    })
}

/// Parse only FFmpeg's machine progress `speed` key. It is media speed, not network B/s.
pub fn parse_ffmpeg_speed(line: &str) -> Option<f64> {
    if line.len() > MAX_METRIC_LINE_BYTES || line.contains(['\n', '\r']) {
        return None;
    }
    let value = line
        .trim()
        .strip_prefix("speed=")?
        .trim()
        .strip_suffix('x')?;
    let value: f64 = value.parse().ok()?;
    (value.is_finite() && value >= 0.0).then_some(value)
}

#[derive(Debug, Clone)]
pub struct NetworkSample {
    pub interface: String,
    pub received_bytes: u64,
    pub sent_bytes: u64,
    pub observed_at: Instant,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NetworkRate {
    pub received_bytes_per_second: f64,
    pub sent_bytes_per_second: f64,
}
impl NetworkSample {
    pub fn rate_since(&self, previous: &Self) -> Option<NetworkRate> {
        if self.interface != previous.interface {
            return None;
        }
        let elapsed = self
            .observed_at
            .checked_duration_since(previous.observed_at)?
            .as_secs_f64();
        if elapsed <= 0.0 {
            return None;
        }
        let received = self.received_bytes.checked_sub(previous.received_bytes)? as f64 / elapsed;
        let sent = self.sent_bytes.checked_sub(previous.sent_bytes)? as f64 / elapsed;
        if !received.is_finite() || !sent.is_finite() {
            return None;
        }
        Some(NetworkRate {
            received_bytes_per_second: received,
            sent_bytes_per_second: sent,
        })
    }
}
pub fn valid_interface(interface: &str) -> bool {
    !interface.is_empty()
        && interface.len() <= 64
        && interface
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

/// Parse a bounded Linux counter snapshot, selecting one interface rather than summing VPNs.
pub fn parse_network_snapshot(
    snapshot: &str,
    interface: &str,
    observed_at: Instant,
) -> io::Result<Option<NetworkSample>> {
    if !valid_interface(interface) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid network interface name",
        ));
    }
    if snapshot.len() > MAX_NETWORK_SNAPSHOT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Network snapshot exceeds limit",
        ));
    }
    let mut selected = None;
    for (index, line) in snapshot.lines().enumerate() {
        if index >= MAX_INTERFACE_ROWS + 2 || line.len() > MAX_METRIC_LINE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Network snapshot exceeds row limit",
            ));
        }
        let Some((name, fields)) = line.split_once(':') else {
            continue;
        };
        if name.trim() != interface {
            continue;
        }
        if selected.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Duplicate network interface",
            ));
        }
        let mut counters = [0_u64; 16];
        let mut count = 0;
        for word in fields.split_whitespace() {
            if count >= counters.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid network counters",
                ));
            }
            counters[count] = word.parse().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid network counter")
            })?;
            count += 1;
        }
        if count != counters.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Incomplete network counters",
            ));
        }
        selected = Some(NetworkSample {
            interface: interface.into(),
            received_bytes: counters[0],
            sent_bytes: counters[8],
            observed_at,
        });
    }
    Ok(selected)
}

pub fn sample_network_interface(interface: &str) -> io::Result<Option<NetworkSample>> {
    if !valid_interface(interface) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid network interface name",
        ));
    }
    #[cfg(target_os = "linux")]
    {
        use std::io::Read;
        let file = std::fs::File::open("/proc/net/dev")?;
        let mut bytes = Vec::new();
        file.take((MAX_NETWORK_SNAPSHOT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_NETWORK_SNAPSHOT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Network snapshot exceeds limit",
            ));
        }
        let text = std::str::from_utf8(&bytes).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid network snapshot encoding",
            )
        })?;
        parse_network_snapshot(text, interface, Instant::now())
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(None)
    }
}

/// Available bytes for the calling user on the target volume, not total free blocks.
pub fn volume_free_bytes(path: &Path) -> io::Result<Option<u64>> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let path = std::ffi::CString::new(path.as_os_str().as_bytes())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Path contains a NUL"))?;
        let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        // SAFETY: path is NUL-terminated; statvfs initializes the supplied struct on success.
        if unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: statvfs returned success.
        let stats = unsafe { stats.assume_init() };
        let bytes = (stats.f_bavail as u128)
            .checked_mul(stats.f_frsize as u128)
            .and_then(|value| u64::try_from(value).ok());
        Ok(bytes)
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
        if wide.contains(&0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Path contains a NUL",
            ));
        }
        wide.push(0);
        let mut available = 0_u64;
        // SAFETY: directory UTF-16 buffer is NUL-terminated; available points to writable u64.
        if unsafe {
            windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut available,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(Some(available))
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn progress_unknown_and_invalid_values_are_distinct() {
        assert_eq!(
            parse_ytdlp_progress(r#"{"speed":1024.5,"eta":0}"#).unwrap(),
            DownloadMetrics {
                downloaded_bytes: None,
                total_bytes: None,
                estimated_total_bytes: None,
                transfer_finished: false,
                bytes_per_second: Some(1024.5),
                eta_seconds: Some(0.0)
            }
        );
        assert_eq!(
            parse_ytdlp_progress(r#"{"speed":null,"eta":"NA"}"#).unwrap(),
            DownloadMetrics {
                downloaded_bytes: None,
                total_bytes: None,
                estimated_total_bytes: None,
                transfer_finished: false,
                bytes_per_second: None,
                eta_seconds: None
            }
        );
        for input in [
            r#"{"speed":-1}"#,
            r#"{"speed":"NaN"}"#,
            r#"{"eta":true}"#,
            "[]",
            "{broken",
        ] {
            assert!(parse_ytdlp_progress(input).is_err());
        }
        assert_eq!(
            parse_ytdlp_progress(&"x".repeat(MAX_METRIC_LINE_BYTES + 1)),
            Err(MetricsParseError::TooLarge)
        );
    }
    #[test]
    fn ffmpeg_speed_is_finite_and_separate() {
        assert_eq!(parse_ffmpeg_speed("speed= 2.5x"), Some(2.5));
        assert_eq!(parse_ffmpeg_speed("speed=0x"), Some(0.0));
        for input in [
            "speed=N/A",
            "speed=NaNx",
            "speed=infx",
            "speed=-2x",
            "fps=25",
            "speed=2x\nspeed=3x",
        ] {
            assert_eq!(parse_ffmpeg_speed(input), None);
        }
    }
    #[test]
    fn network_delta_reset_and_interface_scope() {
        let now = Instant::now();
        let previous =
            parse_network_snapshot(" eth0: 100 0 0 0 0 0 0 0 40 0 0 0 0 0 0 0", "eth0", now)
                .unwrap()
                .unwrap();
        let sample = parse_network_snapshot(
            " eth0: 300 0 0 0 0 0 0 0 140 0 0 0 0 0 0 0",
            "eth0",
            now + Duration::from_secs(2),
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            sample.rate_since(&previous),
            Some(NetworkRate {
                received_bytes_per_second: 100.0,
                sent_bytes_per_second: 50.0
            })
        );
        assert_eq!(previous.rate_since(&sample), None);
        let mut reset = sample.clone();
        reset.received_bytes = 1;
        assert_eq!(reset.rate_since(&previous), None);
        let mut changed = sample.clone();
        changed.interface = "vpn0".into();
        assert_eq!(changed.rate_since(&previous), None);
        assert_eq!(previous.rate_since(&previous), None);
        assert!(parse_network_snapshot("eth0: broken", "eth0", now).is_err());
        assert!(parse_network_snapshot("", "../eth0", now).is_err());
        assert!(
            parse_network_snapshot("", "missing", now)
                .unwrap()
                .is_none()
        );
    }
    #[test]
    fn bounded_snapshots_and_overflow_fail_without_partial_measurement() {
        let now = Instant::now();
        assert!(
            parse_network_snapshot(&"x".repeat(MAX_NETWORK_SNAPSHOT_BYTES + 1), "eth0", now)
                .is_err()
        );
        assert!(
            parse_network_snapshot(&"header\n".repeat(MAX_INTERFACE_ROWS + 3), "eth0", now)
                .is_err()
        );
        let line = "eth0: 1 0 0 0 0 0 0 0 2 0 0 0 0 0 0 0";
        assert!(parse_network_snapshot(&format!("{line}\n{line}"), "eth0", now).is_err());
        assert!(
            parse_network_snapshot(
                "eth0: 18446744073709551616 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0",
                "eth0",
                now
            )
            .is_err()
        );
        assert_eq!(
            parse_ffmpeg_speed(&"x".repeat(MAX_METRIC_LINE_BYTES + 1)),
            None
        );
    }
    #[test]
    fn target_volume_and_missing_path() {
        let temp = tempfile::tempdir().unwrap();
        #[cfg(any(unix, windows))]
        assert!(volume_free_bytes(temp.path()).unwrap().is_some());
        #[cfg(any(unix, windows))]
        assert!(volume_free_bytes(&temp.path().join("missing")).is_err());
    }
}

#[cfg(test)]
mod transfer_progress_tests {
    use super::*;
    #[test]
    fn actual_transfer_counter_scope_unknown_estimate_and_large_integer_edges() {
        let parse = |value| parse_ytdlp_progress(value).unwrap();
        assert_eq!(
            parse(r#"{"downloaded":0,"total":100}"#).transfer_fraction(),
            Some((0.0, false))
        );
        assert_eq!(
            parse(r#"{"downloaded":25,"total":100,"estimated_total":200}"#).transfer_fraction(),
            Some((0.25, false))
        );
        assert_eq!(
            parse(r#"{"downloaded":25,"estimated_total":100}"#).transfer_fraction(),
            Some((0.25, true))
        );
        for value in [
            r#"{}"#,
            r#"{"downloaded":25,"total":0}"#,
            r#"{"downloaded":101,"total":100}"#,
        ] {
            assert_eq!(parse(value).transfer_fraction(), None);
        }
        let near = parse(r#"{"downloaded":18446744073709551614,"total":18446744073709551615}"#);
        assert!(near.transfer_fraction().unwrap().0 < 1.0);
        let finished = parse(r#"{"downloaded":100,"total":100,"status":"finished"}"#);
        assert!(finished.transfer_finished);
        assert_eq!(finished.transfer_fraction(), Some((1.0, false)));
    }
    #[test]
    fn fractional_fragment_estimates_keep_rate_and_never_claim_early_completion() {
        let feed = ProgressFeed::default();
        for (downloaded, total) in [(25, 100.5), (1, 1.5), (25, 100.0)] {
            feed.observe(&format!(
                "DLPAD_PROGRESS:{{\"speed\":1500000,\"eta\":3,\"downloaded\":{downloaded},\"estimated_total\":{total}}}"
            ));
            let sample = feed.snapshot().unwrap();
            assert_eq!(sample.bytes_per_second, Some(1500000.0));
            assert_eq!(sample.eta_seconds, Some(3.0));
            let (fraction, estimated) = sample.transfer_fraction().unwrap();
            assert!(estimated);
            assert!((f64::from(fraction) - downloaded as f64 / total).abs() < 0.000001);
        }
        let parse = |text| parse_ytdlp_progress(text).unwrap();
        assert_eq!(
            parse(r#"{"downloaded":100,"estimated_total":100.0}"#).transfer_fraction(),
            Some((0.999, true))
        );
        assert_eq!(
            parse(r#"{"downloaded":100,"estimated_total":100.0,"status":"finished"}"#)
                .transfer_fraction(),
            Some((1.0, true))
        );
        assert_eq!(
            parse(r#"{"downloaded":25,"total":100,"estimated_total":100.5}"#).transfer_fraction(),
            Some((0.25, false))
        );
        assert_eq!(
            parse(r#"{"downloaded":1,"estimated_total":0.0}"#).transfer_fraction(),
            None
        );
        assert_eq!(
            parse(r#"{"downloaded":2,"estimated_total":1.5}"#).transfer_fraction(),
            None
        );
        for text in [
            r#"{"estimated_total":-0.5}"#,
            r#"{"estimated_total":1e30}"#,
            r#"{"estimated_total":1e400}"#,
            r#"{"downloaded":1.5}"#,
            r#"{"total":1.5}"#,
        ] {
            assert!(parse_ytdlp_progress(text).is_err());
            feed.observe(&format!("DLPAD_PROGRESS:{text}"));
            assert_eq!(feed.snapshot(), None);
        }
    }
    #[test]
    fn malformed_progress_clears_prior_percentage_and_legacy_remains_unknown() {
        let feed = ProgressFeed::default();
        feed.observe(r#"DLPAD_PROGRESS:{"downloaded":25,"total":100}"#);
        assert_eq!(
            feed.snapshot().unwrap().transfer_fraction(),
            Some((0.25, false))
        );
        for line in [
            r#"{"downloaded":-1}"#,
            r#"{"downloaded":1.5}"#,
            r#"{"total":18446744073709551616}"#,
            r#"{"status":true}"#,
        ] {
            assert!(parse_ytdlp_progress(line).is_err());
            feed.observe(&format!("DLPAD_PROGRESS:{line}"));
            assert_eq!(feed.snapshot(), None);
        }
        feed.observe(r#"DLPAD_PROGRESS:{"speed":1000,"eta":3}"#);
        assert_eq!(feed.snapshot().unwrap().transfer_fraction(), None);
    }
    #[test]
    fn stale_transfers_do_not_reappear_as_measured_zero_or_completion() {
        let feed = ProgressFeed::default();
        feed.observe(r#"DLPAD_PROGRESS:{"downloaded":100,"total":100,"status":"finished"}"#);
        assert_eq!(
            feed.snapshot_at(Instant::now() + std::time::Duration::from_secs(4)),
            None
        );
    }
}
