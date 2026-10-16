//! Synthetic native managed-reader acceptance. No user files or accounts.
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use dlpad_engine::{
        process,
        tools::{ManagedReaderStage, ToolManager},
    };
    use std::{
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        time::Duration,
    };
    process::guardian_entry()?;
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--synthetic-tool") {
        match args.get(2).map(String::as_str) {
            Some("metadata") => println!("{{\"duration\":42}}"),
            Some("failure") => std::process::exit(7),
            Some("cancel") => std::thread::sleep(Duration::from_secs(30)),
            _ => println!("synthetic success"),
        }
        return Ok(());
    }
    process::register_self_guardian()?;
    let temp = tempfile::tempdir()?;
    let root = temp.path().join("managed");
    let foreign = temp.path().join("foreign");
    std::fs::write(&foreign, b"preserve original")?;
    let manager = ToolManager::new(&root)?;
    for role in ["metadata", "success", "failure", "cancel"] {
        let execution = manager.begin_execution()?;
        let cancel = Arc::new(AtomicBool::new(false));
        let stopper = if role == "cancel" {
            let flag = cancel.clone();
            Some(std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(600));
                flag.store(true, Ordering::Release);
            }))
        } else {
            None
        };
        let spec = process::ProcessSpec {
            #[cfg(windows)]
            runtime_environment: None,
            executable: std::env::current_exe()?,
            args: vec!["--synthetic-tool".into(), role.into()],
            timeout: Duration::from_secs(10),
            max_log_bytes: 4096,
        };
        let result = execution.run_tool(
            &spec,
            &cancel,
            role == "metadata",
            None,
            if role == "metadata" {
                ManagedReaderStage::Metadata
            } else {
                ManagedReaderStage::Download
            },
        );
        if let Some(thread) = stopper {
            thread.join().unwrap();
        }
        let result = result?;
        match role {
            "cancel" => assert_eq!(result.execution.outcome, process::ProcessOutcome::Cancelled),
            "failure" => assert_eq!(result.execution.exit_code, Some(7)),
            "metadata" => {
                assert_eq!(result.execution.exit_code, Some(0));
                let data: serde_json::Value = serde_json::from_slice(&result.stdout)?;
                assert_eq!(data["duration"], 42);
            }
            _ => assert_eq!(result.execution.exit_code, Some(0)),
        }
        execution.finish()?;
        assert!(!root.join(".execution-pending").exists());
        assert!(!root.join(".execution-readers.json").exists());
        assert_eq!(std::fs::read(&foreign)?, b"preserve original");
        println!("managed-native-case={role} PASS");
    }
    manager.begin_execution()?.finish()?;
    println!("managed-native-reuse PASS");
    Ok(())
}
#[cfg(not(windows))]
fn main() {
    eprintln!("Windows native acceptance only");
}
