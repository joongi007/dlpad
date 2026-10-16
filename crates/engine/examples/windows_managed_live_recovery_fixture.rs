//! Synthetic live creator lease only; no real tool/helper reader is launched.
#[cfg(windows)]
mod native {
    use dlpad_engine::{file_identity, tools::ToolManager};
    use sha2::{Digest, Sha256};
    use std::{
        fs,
        io::{Read, Write},
        os::windows::{
            fs::{MetadataExt, OpenOptionsExt},
            io::AsRawHandle,
        },
        path::{Path, PathBuf},
        sync::mpsc,
        time::Duration,
    };
    const CAP: &[u8] = b"dlpad-managed-live-recovery-fixture-v1";
    const SOURCE: &[u8] = b"synthetic source; no media or credentials";
    const SENTINEL: &[u8] = b"preserve foreign live creator fixture";
    fn new(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if bytes.len() > 16384 {
            return Err("write bound".into());
        }
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        Ok(())
    }
    fn readable(root: &Path, path: &Path) -> Result<fs::File, Box<dyn std::error::Error>> {
        use windows_sys::Win32::Storage::FileSystem::*;
        if !path.is_absolute() || !path.starts_with(root) || path == root {
            return Err("fixture read outside owner".into());
        }
        for ancestor in path.ancestors() {
            if fs::symlink_metadata(ancestor)?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
            {
                return Err("fixture read reparse rejected".into());
            }
        }
        // Normal read access under the existing DACL; never change privileges or security.
        let file = fs::OpenOptions::new()
            .access_mode(FILE_GENERIC_READ)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?;
        let root_pin = fs::OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
            .open(root)?;
        file_identity::ensure_regular_under(&root_pin, &file)?;
        let mut standard = FILE_STANDARD_INFO::default();
        // SAFETY: original retained file handle and correctly sized initialized output.
        if unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileStandardInfo,
                (&mut standard as *mut FILE_STANDARD_INFO).cast(),
                std::mem::size_of_val(&standard) as u32,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
        if standard.NumberOfLinks != 1 || standard.DeletePending || standard.Directory {
            return Err("fixture read regular original file required".into());
        }
        Ok(file)
    }
    fn pins(
        root: &Path,
        paths: &[PathBuf],
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut records = Vec::new();
        for path in paths {
            let mut f = readable(root, path)?;
            let size = f.metadata()?.len();
            if size > 16384 {
                return Err("protected fixture bound".into());
            }
            let mut bytes = Vec::new();
            (&mut f).take(16385).read_to_end(&mut bytes)?;
            if bytes.len() as u64 != size {
                return Err("protected file changed".into());
            }
            records.push(serde_json::json!({"path":path,"bytes":size,"sha256":format!("{:x}",Sha256::digest(&bytes))}));
        }
        Ok(serde_json::Value::Array(records))
    }
    fn journal_pin(
        root: &Path,
        path: &Path,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut f = readable(root, path)?;
        let size = f.metadata()?.len();
        if size == 0 || size > 16384 {
            return Err("journal metadata bound".into());
        }
        let identity = file_identity::from_file(&f)?;
        let mut byte = [0u8; 1];
        let error = f
            .read(&mut byte)
            .err()
            .ok_or("journal unexpectedly readable")?;
        if error.raw_os_error() != Some(33) {
            return Err("unexpected journal read failure".into());
        }
        Ok(
            serde_json::json!({"path":path,"bytes":size,"native_identity":identity,"read_lock_denied":true,"read_error_code":33,"sha256_measured":false}),
        )
    }
    fn emit(value: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = serde_json::to_vec(value)?;
        if bytes.len() > 16384 {
            return Err("stdout evidence bound".into());
        }
        let mut out = std::io::stdout().lock();
        out.write_all(&bytes)?;
        out.write_all(b"\n")?;
        out.flush()?;
        Ok(())
    }
    fn creation() -> Result<u64, Box<dyn std::error::Error>> {
        use windows_sys::Win32::{
            Foundation::FILETIME,
            System::Threading::{GetCurrentProcess, GetProcessTimes},
        };
        let mut c = FILETIME::default();
        let mut e = FILETIME::default();
        let mut k = FILETIME::default();
        let mut u = FILETIME::default();
        if unsafe { GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut k, &mut u) } == 0 {
            return Err("parent creation unavailable".into());
        }
        Ok((u64::from(c.dwHighDateTime) << 32) | u64::from(c.dwLowDateTime))
    }
    pub(super) fn run(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if !root.is_absolute()
            || root.file_name().and_then(|v| v.to_str()) != Some("owned-fixture")
            || !root
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|v| v.to_str())
                .is_some_and(|v| v.starts_with("dlpad-managed-live-recovery-"))
        {
            return Err("owned namespace required".into());
        }
        for p in root.ancestors() {
            if p.exists() && fs::symlink_metadata(p)?.file_attributes() & 0x400 != 0 {
                return Err("reparse ancestor rejected".into());
            }
        }
        if root.exists() {
            if !root.is_dir() || fs::read_dir(root)?.next().is_some() {
                return Err("fresh empty root required".into());
            }
        } else {
            fs::create_dir(root)?;
        }
        let root = root.canonicalize()?;
        let data = root.join("data");
        fs::create_dir(&data)?;
        let cap = root.join("fixture.cap");
        let source = root.join("synthetic-source");
        let sentinel = root.join("foreign-sentinel");
        new(&cap, CAP)?;
        new(&source, SOURCE)?;
        new(&sentinel, SENTINEL)?;
        let managed = data.join("managed-tools");
        let manager = ToolManager::new(&managed)?;
        let execution = manager.begin_execution()?;
        let marker = managed.join(".execution-pending");
        let journal = managed.join(".execution-readers.json");
        let mut protected = vec![
            marker.clone(),
            cap.clone(),
            source.clone(),
            sentinel.clone(),
        ];
        let mut removed = vec![marker.clone(), journal.clone()];
        for i in 0..8 {
            let p = managed.join(format!(".execution-reader-{i}.receipt"));
            protected.push(p.clone());
            removed.push(p);
        }
        let before = pins(&root, &protected)?;
        let journal_before = journal_pin(&root, &journal)?;
        let live = manager.recover_execution()?;
        if live.recovered || !live.pending {
            return Err("live creator recovery was not pending".into());
        }
        if pins(&root, &protected)? != before || journal_pin(&root, &journal)? != journal_before {
            return Err("live recovery mutated original files".into());
        }
        let parent_creation = creation()?;
        let ready = serde_json::json!({"schema":1,"phase":"managed-live-ready","parent_creation":parent_creation,"scope":"live original creator lease; no tool helper","protected_files":protected,"protected_file_evidence":before,"removed_files":removed,"journal_path":journal,"journal_evidence":journal_before,"self_live_recovery_pending":true,"self_live_recovery_recovered":false});
        new(
            &root.join("live-ready-evidence.json"),
            &serde_json::to_vec_pretty(&ready)?,
        )?;
        emit(&ready)?;
        let (tx, rx) = mpsc::sync_channel(1);
        let reader = std::thread::spawn(move || {
            let mut byte = [0u8; 1];
            let result = std::io::stdin().read_exact(&mut byte).map(|_| byte[0]);
            let _ = tx.send(result);
        });
        match rx.recv_timeout(Duration::from_secs(90)) {
            Ok(Ok(b'R')) => {}
            _ => return Err("explicit release missing or invalid; pending preserved".into()),
        };
        reader.join().map_err(|_| "release reader panic")?;
        if pins(&root, &protected)? != before || journal_pin(&root, &journal)? != journal_before {
            return Err("original live files changed before release".into());
        }
        execution.finish()?;
        if removed.iter().any(|p| p.exists()) {
            return Err("pristine finish files remain".into());
        }
        if fs::read(&cap)? != CAP
            || fs::read(&source)? != SOURCE
            || fs::read(&sentinel)? != SENTINEL
        {
            return Err("fixture originals changed".into());
        }
        let finished = serde_json::json!({"schema":1,"phase":"managed-live-finished","parent_creation":parent_creation,"removed_files":removed,"source_foreign_preserved":true,"pristine_finish":true,"reader_joined":true,"scope":"creator lease only; no tool helper"});
        new(
            &root.join("live-finished-evidence.json"),
            &serde_json::to_vec_pretty(&finished)?,
        )?;
        emit(&finished)?;
        Ok(())
    }
}
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 || args[0] != "managed-live" {
        return Err("usage: managed-live owned-root".into());
    }
    native::run(std::path::Path::new(&args[1]))
}
#[cfg(not(windows))]
fn main() {
    eprintln!("Windows live creator lease fixture only; not executed");
}
