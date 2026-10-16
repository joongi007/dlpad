//! Public failure path uses only a synthetic nonexistent executable.
#![cfg(target_os = "linux")]
use dlpad_engine::process::{self, ProcessSpec};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

#[test]
fn nonexistent_tool_startup_returns_without_unbounded_launcher_wait() {
    let folder = tempfile::tempdir().unwrap();
    let started = Instant::now();
    let result = process::run(
        &ProcessSpec {
            #[cfg(windows)]
            runtime_environment: None,
            executable: folder.path().join("absent-synthetic-tool"),
            args: Vec::new(),
            timeout: Duration::from_secs(2),
            max_log_bytes: 1024,
        },
        &Arc::new(AtomicBool::new(false)),
    );
    let error = result.unwrap_err();
    assert!(process::cleanup_is_unconfirmed(&error));
    assert!(!error.to_string().contains("absent-synthetic-tool"));
    assert!(started.elapsed() < Duration::from_secs(18));
}
