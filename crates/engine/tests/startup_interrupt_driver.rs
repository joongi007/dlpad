//! Explicit manager-independent public startup capability driver; synthetic fixtures only.
#![cfg(target_os = "linux")]
use dlpad_engine::process::{self, OwnershipCleanupError, ProcessSpec};
use serde_json::json;
use std::{
    fs, io,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
fn descriptors() -> usize {
    fs::read_dir("/proc/self/fd").unwrap().count()
}
fn fixed_error(error: io::Error) -> serde_json::Value {
    let marker = error
        .get_ref()
        .and_then(|value| value.downcast_ref::<OwnershipCleanupError>());
    json!({"ok":false,"error_kind":format!("{:?}",error.kind()),
        "cleanup_unconfirmed":process::cleanup_is_unconfirmed(&error),
        "ownership_marker":marker.map(|value|format!("{value:?}"))})
}
#[test]
#[ignore = "capability driver: use scripts/public_startup_interrupt_probe.py after READY"]
fn controlled_startup_driver() {
    let folder = PathBuf::from(
        std::env::var_os("DLPAD_STARTUP_INTERRUPT_ROOT").expect("explicit owned fixture"),
    );
    assert!(folder.is_absolute() && folder.starts_with("/tmp"));
    let case = std::env::var("DLPAD_STARTUP_INTERRUPT_CASE").expect("explicit case");
    assert!(matches!(
        case.as_str(),
        "noR_timeout" | "noR_cancel" | "partial_cancel" | "metadata_noR_timeout"
    ));
    let cancel = Arc::new(AtomicBool::new(false));
    let started = Instant::now();
    let before = descriptors();
    let cancellation = if matches!(case.as_str(), "noR_cancel" | "partial_cancel") {
        let cancel = cancel.clone();
        let marker = folder.join("launcher.json");
        Some(thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !marker.exists() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(5));
            }
            let observed = marker.exists();
            let seen = started.elapsed().as_millis();
            thread::sleep(Duration::from_millis(100));
            cancel.store(true, Ordering::Release);
            json!({"launcher_observed":observed,"launcher_seen_elapsed_millis":seen,"cancel_elapsed_millis":started.elapsed().as_millis()})
        }))
    } else {
        None
    };
    let spec = ProcessSpec {
        #[cfg(windows)]
        runtime_environment: None,
        executable: "/usr/bin/python3".into(),
        args: vec![
            folder.join("unexpected-tool.py").into_os_string(),
            folder.clone().into_os_string(),
        ],
        timeout: if case.ends_with("timeout") {
            Duration::from_millis(150)
        } else {
            Duration::from_secs(10)
        },
        max_log_bytes: 1024,
    };
    let execution = if case == "metadata_noR_timeout" {
        match process::run_metadata(&spec, &cancel) {
            Ok(result) => json!({"ok":true,"outcome":format!("{:?}",result.execution.outcome)}),
            Err(error) => fixed_error(error),
        }
    } else {
        match process::run(&spec, &cancel) {
            Ok(result) => json!({"ok":true,"outcome":format!("{:?}",result.outcome)}),
            Err(error) => fixed_error(error),
        }
    };
    let cancellation = cancellation.map(|task| task.join().expect("owned cancellation thread"));
    let value = json!({"case":case,"execution":execution,"cancellation":cancellation,
        "elapsed_millis":started.elapsed().as_millis(),"fd_before":before,"fd_after":descriptors()});
    let bytes = serde_json::to_vec_pretty(&value).unwrap();
    assert!(bytes.len() <= 128 * 1024);
    use std::io::Write;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(folder.join("result.json"))
        .unwrap();
    output.write_all(&bytes).unwrap();
}
