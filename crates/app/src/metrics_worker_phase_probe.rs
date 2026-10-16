//! Default-OFF diagnostic on the actual metrics caller worker; no sampler policy changes.
use dlpad_engine::thread_cpu::ThreadCpuClock;
use serde::Serialize;
use std::{
    cell::RefCell,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
    time::{Duration, Instant},
};
const MAX_TICKS: usize = 64;
const MAX_REPORT: usize = 65536;
const MARKER: &[u8] = b"dlpad-metrics-worker-phase-owned-v1\n";
thread_local! { static ACTIVE: RefCell<Option<Probe>> = const { RefCell::new(None) }; }
#[derive(Clone, Copy)]
pub(super) enum Phase {
    Discovery = 0,
    Volume = 1,
    Automatic = 2,
    Sample = 3,
    Mailbox = 4,
    Repaint = 5,
}
#[derive(Serialize)]
struct Measurement {
    p: u8,
    w: u64,
    c0: Option<u64>,
    c1: Option<u64>,
    d: Option<u64>,
    s: u8,
}
#[derive(Serialize)]
struct Tick {
    t: u16,
    due: u64,
    begin: u64,
    end: u64,
    c0: Option<u64>,
    c1: Option<u64>,
    d: Option<u64>,
    ph: [Option<Measurement>; 6],
}
#[derive(Serialize)]
struct Report {
    schema: u8,
    start_unix_ms: Option<u64>,
    diagnostic_only: bool,
    probe_overhead_not_subtracted: bool,
    cpu_available: bool,
    cpu_failed: bool,
    dropped_ticks: u64,
    ticks: Vec<Tick>,
}
struct Started {
    at: Instant,
    cpu: Option<u64>,
    index: usize,
}
struct Probe {
    origin: Instant,
    clock: Option<ThreadCpuClock>,
    output: File,
    report: Report,
    recording: bool,
}
fn rejected() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "owned metrics phase probe rejected",
    )
}
fn ns(d: Duration) -> u64 {
    u64::try_from(d.as_nanos()).unwrap_or(u64::MAX)
}
fn plain(path: &Path) -> io::Result<()> {
    if !path.is_absolute()
        || path.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err(rejected());
    }
    let mut count = 0;
    for ancestor in path.ancestors() {
        count += 1;
        if count > 64 {
            return Err(rejected());
        }
        match fs::symlink_metadata(ancestor) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    return Err(rejected());
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err(rejected());
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
impl Probe {
    fn admit(
        enabled: bool,
        fixture: bool,
        private_desktop: bool,
        root: &Path,
        data: &Path,
        media: &Path,
        temp: &Path,
    ) -> io::Result<Option<Self>> {
        if !enabled {
            return Ok(None);
        }
        if !fixture
            || !private_desktop
            || root.parent() != Some(temp)
            || !root
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.starts_with("dlpad-metrics-phase-owned-") && s.len() <= 96)
            || data != root.join("data")
            || media != root.join("media")
        {
            return Err(rejected());
        }
        for path in [root, data, media] {
            plain(path)?;
            if !fs::symlink_metadata(path)?.is_dir() {
                return Err(rejected());
            }
        }
        let marker = root.join("metrics-worker-phase.cap");
        plain(&marker)?;
        let meta = fs::symlink_metadata(&marker)?;
        if !meta.is_file() || meta.len() != MARKER.len() as u64 {
            return Err(rejected());
        }
        let mut bytes = Vec::with_capacity(MARKER.len());
        File::open(&marker)?
            .take((MARKER.len() + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes != MARKER {
            return Err(rejected());
        }
        let target = root.join("metrics-worker-phase.json");
        plain(&target)?;
        let output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target)?;
        let (clock, failed) = match ThreadCpuClock::new() {
            Ok(clock) => (clock, false),
            Err(_) => (None, true),
        };
        let available = clock.is_some();
        Ok(Some(Self {
            origin: Instant::now(),
            clock,
            output,
            recording: false,
            report: Report {
                schema: 1,
                start_unix_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .and_then(|d| u64::try_from(d.as_millis()).ok()),
                diagnostic_only: true,
                probe_overhead_not_subtracted: true,
                cpu_available: available,
                cpu_failed: failed,
                dropped_ticks: 0,
                ticks: Vec::with_capacity(MAX_TICKS),
            },
        }))
    }
    fn cpu(&mut self) -> Option<u64> {
        let clock = self.clock.as_mut()?;
        match clock.sample() {
            Ok(v) => Some(v),
            Err(_) => {
                self.report.cpu_failed = true;
                self.clock = None;
                None
            }
        }
    }
    fn begin_tick(&mut self, due: Instant) {
        self.recording = self.report.ticks.len() < MAX_TICKS;
        if !self.recording {
            self.report.dropped_ticks = self.report.dropped_ticks.saturating_add(1);
            return;
        }
        let at = Instant::now();
        let cpu = self.cpu();
        self.report.ticks.push(Tick {
            t: self.report.ticks.len() as u16 + 1,
            due: ns(at.saturating_duration_since(due)),
            begin: ns(at.duration_since(self.origin)),
            end: 0,
            c0: cpu,
            c1: None,
            d: None,
            ph: std::array::from_fn(|_| None),
        });
    }
    fn start(&mut self) -> Option<Started> {
        if !self.recording {
            return None;
        }
        Some(Started {
            at: Instant::now(),
            cpu: self.cpu(),
            index: self.report.ticks.len() - 1,
        })
    }
    fn end(&mut self, phase: Phase, started: Started, status: u8) {
        let wall = ns(started.at.elapsed());
        let cpu = self.cpu();
        self.report.ticks[started.index].ph[phase as usize] = Some(Measurement {
            p: phase as u8,
            w: wall,
            c0: started.cpu,
            c1: cpu,
            d: started.cpu.zip(cpu).and_then(|(a, b)| b.checked_sub(a)),
            s: status,
        });
    }
    fn end_tick(&mut self) {
        if !self.recording {
            return;
        }
        let end = ns(self.origin.elapsed());
        let cpu = self.cpu();
        if let Some(tick) = self.report.ticks.last_mut() {
            tick.end = end;
            tick.c1 = cpu;
            tick.d = tick.c0.zip(cpu).and_then(|(a, b)| b.checked_sub(a));
        }
        self.recording = false;
    }
    fn drain(mut self) -> io::Result<()> {
        self.end_tick();
        struct Limited(Vec<u8>);
        impl Write for Limited {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                if self
                    .0
                    .len()
                    .checked_add(bytes.len())
                    .is_none_or(|size| size > MAX_REPORT)
                {
                    return Err(io::Error::new(
                        io::ErrorKind::FileTooLarge,
                        "probe report bound",
                    ));
                }
                self.0.extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut serialized = Limited(Vec::with_capacity(MAX_REPORT));
        serde_json::to_writer(&mut serialized, &self.report).map_err(io::Error::other)?;
        self.output.write_all(&serialized.0)?;
        self.output.flush()
    }
}
pub(super) struct Owner(std::marker::PhantomData<std::rc::Rc<()>>);
impl Owner {
    #[cfg(all(windows, feature = "first-paint-fixture"))]
    pub(super) fn from_environment() -> Self {
        // All admission I/O happens on this caller worker. Default absent env does no I/O/clock allocation.
        let admitted: io::Result<Option<Probe>> = (|| {
            let Some(root) = std::env::var_os("DLPAD_METRICS_WORKER_PHASE_ROOT") else {
                return Ok(None);
            };
            let root = std::path::PathBuf::from(root);
            let data =
                std::path::PathBuf::from(std::env::var_os("DLPAD_TEST_DATA").ok_or_else(rejected)?);
            let media = std::path::PathBuf::from(
                std::env::var_os("DLPAD_LIBRARY_ROOT").ok_or_else(rejected)?,
            );
            Probe::admit(
                true,
                std::env::args_os().any(|a| a == "--fixture"),
                private_worker_desktop()?,
                &root,
                &data,
                &media,
                &std::env::temp_dir(),
            )
        })();
        ACTIVE.with(|active| *active.borrow_mut() = admitted.ok().flatten());
        Self(std::marker::PhantomData)
    }
    #[cfg(test)]
    fn explicit(probe: Option<Probe>) -> Self {
        ACTIVE.with(|active| *active.borrow_mut() = probe);
        Self(std::marker::PhantomData)
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        ACTIVE.with(|active| {
            if let Some(probe) = active.borrow_mut().take() {
                let _ = probe.drain();
            }
        });
    }
}
pub(super) fn begin_tick(due: Instant) {
    ACTIVE.with(|active| {
        if let Some(probe) = active.borrow_mut().as_mut() {
            probe.begin_tick(due);
        }
    });
}
pub(super) fn end_tick() {
    ACTIVE.with(|active| {
        if let Some(probe) = active.borrow_mut().as_mut() {
            probe.end_tick();
        }
    });
}
pub(super) fn measured<T>(
    phase: Phase,
    operation: impl FnOnce() -> T,
    status: impl FnOnce(&T) -> u8,
) -> T {
    let start = ACTIVE.with(|active| active.borrow_mut().as_mut().and_then(Probe::start));
    let result = operation();
    if let Some(started) = start {
        ACTIVE.with(|active| {
            if let Some(probe) = active.borrow_mut().as_mut() {
                probe.end(phase, started, status(&result));
            }
        });
    }
    result
}
#[cfg(all(windows, feature = "first-paint-fixture"))]
fn private_worker_desktop() -> io::Result<bool> {
    use windows_sys::Win32::{
        Foundation::HANDLE,
        System::{
            StationsAndDesktops::{
                CloseDesktop, GetThreadDesktop, GetUserObjectInformationW, OpenInputDesktop,
            },
            Threading::GetCurrentThreadId,
        },
    };
    fn name(handle: HANDLE) -> io::Result<Vec<u16>> {
        let mut out = [0u16; 256];
        let mut required = 0;
        if unsafe {
            GetUserObjectInformationW(
                handle,
                2,
                out.as_mut_ptr().cast(),
                std::mem::size_of_val(&out) as u32,
                &mut required,
            )
        } == 0
            || required == 0
            || required as usize > std::mem::size_of_val(&out)
        {
            return Err(rejected());
        }
        let n = out.iter().position(|c| *c == 0).ok_or_else(rejected)?;
        Ok(out[..n].to_vec())
    }
    let current = name(unsafe { GetThreadDesktop(GetCurrentThreadId()) })?;
    let input = unsafe { OpenInputDesktop(0, 0, 1) };
    if input.is_null() {
        return Err(io::Error::last_os_error());
    }
    let original = name(input);
    let closed = unsafe { CloseDesktop(input) };
    if closed == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(current != original?
        && String::from_utf16(&current)
            .map_err(|_| rejected())?
            .starts_with("dlpad-"))
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            // Only exact owned fixture files and empty directories; no recursive/foreign cleanup.
            for leaf in ["metrics-worker-phase.json", "metrics-worker-phase.cap"] {
                let path = self.0.join(leaf);
                if path.exists() {
                    let _ = fs::remove_file(path);
                }
            }
            for leaf in ["data", "media"] {
                let _ = fs::remove_dir(self.0.join(leaf));
            }
            let _ = fs::remove_dir(&self.0);
        }
    }
    fn fixture() -> Fixture {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let nonce = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "dlpad-metrics-phase-owned-{}-{nonce}-{time}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("fresh owned fixture only");
        let root = Fixture(path);
        fs::create_dir(root.path().join("data")).unwrap();
        fs::create_dir(root.path().join("media")).unwrap();
        root
    }
    fn admit(root: &Path) -> io::Result<Option<Probe>> {
        Probe::admit(
            true,
            true,
            true,
            root,
            &root.join("data"),
            &root.join("media"),
            root.parent().unwrap(),
        )
    }
    #[test]
    fn metrics_phase_disabled_missing_marker_and_existing_report_do_not_write() {
        let root = fixture();
        let report = root.path().join("metrics-worker-phase.json");
        assert!(
            Probe::admit(
                false,
                false,
                false,
                Path::new("nonexistent"),
                Path::new("no-data"),
                Path::new("no-media"),
                Path::new("no-temp")
            )
            .unwrap()
            .is_none()
        );
        assert!(admit(root.path()).is_err());
        assert!(!report.exists());
        fs::write(root.path().join("metrics-worker-phase.cap"), MARKER).unwrap();
        fs::write(&report, b"retained-original").unwrap();
        assert_eq!(
            admit(root.path()).err().unwrap().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(fs::read(report).unwrap(), b"retained-original");
    }
    #[test]
    fn metrics_phase_bound_drains_once_with_numeric_only_report_and_exact_operations() {
        let root = fixture();
        fs::write(root.path().join("metrics-worker-phase.cap"), MARKER).unwrap();
        let owner = Owner::explicit(admit(root.path()).unwrap());
        let mut calls = 0;
        for _ in 0..70 {
            begin_tick(Instant::now());
            for phase in [
                Phase::Discovery,
                Phase::Volume,
                Phase::Automatic,
                Phase::Sample,
                Phase::Mailbox,
                Phase::Repaint,
            ] {
                measured(
                    phase,
                    || {
                        calls += 1;
                    },
                    |_| 0,
                );
            }
            end_tick();
        }
        drop(owner);
        assert_eq!(calls, 420);
        let raw = fs::read(root.path().join("metrics-worker-phase.json")).unwrap();
        assert!(raw.len() <= MAX_REPORT);
        let value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(value["ticks"].as_array().unwrap().len(), 64);
        assert_eq!(value["dropped_ticks"], 6);
        assert!(
            !String::from_utf8(raw)
                .unwrap()
                .contains(root.path().to_str().unwrap())
        );
        #[cfg(not(windows))]
        {
            assert_eq!(value["cpu_available"], false);
            assert!(value["ticks"][0]["c0"].is_null());
        }
    }
    #[test]
    fn metrics_phase_full_64_tick_report_with_maximum_numeric_values_fits_64k() {
        let root = fixture();
        fs::write(root.path().join("metrics-worker-phase.cap"), MARKER).unwrap();
        let mut probe = admit(root.path()).unwrap().unwrap();
        for tick in 1..=64 {
            probe.report.ticks.push(Tick {
                t: tick,
                due: u64::MAX,
                begin: u64::MAX,
                end: u64::MAX,
                c0: Some(u64::MAX),
                c1: Some(u64::MAX),
                d: Some(u64::MAX),
                ph: std::array::from_fn(|p| {
                    Some(Measurement {
                        p: p as u8,
                        w: u64::MAX,
                        c0: Some(u64::MAX),
                        c1: Some(u64::MAX),
                        d: Some(u64::MAX),
                        s: u8::MAX,
                    })
                }),
            });
        }
        probe.report.dropped_ticks = u64::MAX;
        probe.report.start_unix_ms = Some(u64::MAX);
        probe.drain().unwrap();
        let bytes = fs::read(root.path().join("metrics-worker-phase.json")).unwrap();
        assert!(
            bytes.len() <= MAX_REPORT,
            "actual bounded writer emitted {} bytes",
            bytes.len()
        );
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed["ticks"].as_array().unwrap().len(), MAX_TICKS);
    }
    #[test]
    fn metrics_phase_incorrect_fixture_and_bad_marker_are_refused_without_report() {
        let root = fixture();
        fs::write(
            root.path().join("metrics-worker-phase.cap"),
            b"wrong marker",
        )
        .unwrap();
        assert!(admit(root.path()).is_err());
        assert!(!root.path().join("metrics-worker-phase.json").exists());
        fs::write(root.path().join("metrics-worker-phase.cap"), MARKER).unwrap();
        assert!(
            Probe::admit(
                true,
                true,
                false,
                root.path(),
                &root.path().join("data"),
                &root.path().join("media"),
                root.path().parent().unwrap()
            )
            .is_err()
        );
        assert!(
            Probe::admit(
                true,
                true,
                true,
                root.path(),
                Path::new("real-user-data"),
                &root.path().join("media"),
                root.path().parent().unwrap()
            )
            .is_err()
        );
    }
}
