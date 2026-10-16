//! Explicitly selected Linux user-manager capability driver; ordinary tests stay host independent.
#![cfg(target_os = "linux")]
use dlpad_engine::process::{self, ProcessSpec};
use serde_json::json;
use std::{
    fs,
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

#[test]
#[ignore = "capability driver: use scripts/user_service_engine_probe.py after backend READY"]
fn user_service_driver() {
    let folder =
        PathBuf::from(std::env::var_os("DLPAD_SERVICE_TEST_ROOT").expect("explicit owned fixture"));
    let case = std::env::var("DLPAD_SERVICE_TEST_CASE").expect("explicit case");
    assert!(folder.is_absolute() && folder.starts_with("/tmp"));
    assert!(PathBuf::from(env!("CARGO_BIN_EXE_dlpad-process-guard")).is_file());
    let cancel = Arc::new(AtomicBool::new(false));
    let cancellation = if case == "cancel" {
        let cancel = cancel.clone();
        let ready = folder.join("worker.json");
        Some(thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !ready.exists() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(10));
            }
            thread::sleep(Duration::from_millis(500));
            cancel.store(true, Ordering::Release);
        }))
    } else {
        None
    };
    let args = if case == "fdleak" {
        vec!["-c".into(), "print('fd probe')".into()]
    } else {
        let mut args = vec![
            folder.join("worker.py").into_os_string(),
            folder.clone().into_os_string(),
            case.clone().into(),
        ];
        args.extend(
            [
                "$DLPAD_FIXTURE_VAR",
                "${DLPAD_FIXTURE_VAR}",
                "prefix-${DLPAD_FIXTURE_VAR}",
                "literal with spaces",
                "한글",
                "%n",
                "quote'\"literal",
            ]
            .into_iter()
            .map(Into::into),
        );
        args
    };
    let spec = ProcessSpec {
        #[cfg(windows)]
        runtime_environment: None,
        executable: "/usr/bin/python3".into(),
        args,
        timeout: if case == "timeout" {
            Duration::from_millis(600)
        } else {
            Duration::from_secs(10)
        },
        max_log_bytes: 4096,
    };
    let started = Instant::now();
    let before = descriptors();
    let mut runs = vec![];
    for _ in 0..if case == "fdleak" { 12 } else { 1 } {
        let value = if case == "metadata" {
            match process::run_metadata(&spec, &cancel) {
                Ok(result) => {
                    json!({"ok":true,"exit_code":result.execution.exit_code,"outcome":format!("{:?}",result.execution.outcome),"logs":result.execution.logs,"truncated":result.execution.truncated,"metadata":String::from_utf8(result.stdout).unwrap()})
                }
                Err(error) => {
                    json!({"ok":false,"error_kind":format!("{:?}",error.kind()),"error":error.to_string()})
                }
            }
        } else {
            match process::run(&spec, &cancel) {
                Ok(result) => {
                    json!({"ok":true,"exit_code":result.exit_code,"outcome":format!("{:?}",result.outcome),"logs":result.logs,"truncated":result.truncated})
                }
                Err(error) => {
                    json!({"ok":false,"error_kind":format!("{:?}",error.kind()),"error":error.to_string()})
                }
            }
        };
        runs.push(value);
    }
    if let Some(cancellation) = cancellation {
        cancellation.join().unwrap();
    }
    let value = json!({"case":case,"runs":runs,"fd_before":before,"fd_after":descriptors(),"elapsed_millis":started.elapsed().as_millis()});
    let data = serde_json::to_vec_pretty(&value).unwrap();
    assert!(data.len() <= 128 * 1024, "bounded test driver evidence");
    fs::write(folder.join("result.json"), data).unwrap();
}
