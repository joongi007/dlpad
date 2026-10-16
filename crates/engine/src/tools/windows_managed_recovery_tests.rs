use super::recover;
use crate::tools::ToolManager;
use std::{collections::BTreeMap, fs, path::PathBuf};
type FileObservation = (u64, Result<Vec<u8>, Option<i32>>);
struct Fixture {
    manager: ToolManager,
    root: PathBuf,
    source: PathBuf,
    sentinel: PathBuf,
    _temp: tempfile::TempDir,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("managed");
        let manager = ToolManager::new(&root).unwrap();
        let source = temp.path().join("source.synthetic");
        let sentinel = temp.path().join("sentinel.synthetic");
        fs::write(&source, b"source preserved").unwrap();
        fs::write(&sentinel, b"foreign sentinel preserved").unwrap();
        Self {
            manager,
            root,
            source,
            sentinel,
            _temp: temp,
        }
    }
    fn pending(&self) {
        drop(self.manager.begin_execution().unwrap());
    }
    fn snapshot(&self) -> BTreeMap<String, FileObservation> {
        let mut files = BTreeMap::new();
        let names = [
            ".execution-pending",
            ".execution-readers.json",
            ".install-transfer-lock",
        ]
        .into_iter()
        .map(str::to_owned)
        .chain((0..8).map(|i| format!(".execution-reader-{i}.receipt")));
        for name in names {
            let path = self.root.join(&name);
            let metadata = fs::symlink_metadata(&path).unwrap();
            let observed = fs::read(&path).map_err(|error| error.raw_os_error());
            files.insert(name, (metadata.len(), observed));
        }
        files
    }
    fn reject_preserving(&self) {
        let before = self.snapshot();
        assert!(!matches!(recover(&self.manager), Ok(true)));
        assert_eq!(self.snapshot(), before);
        assert_eq!(fs::read(&self.source).unwrap(), b"source preserved");
        assert_eq!(
            fs::read(&self.sentinel).unwrap(),
            b"foreign sentinel preserved"
        );
        assert!(self.root.join(".execution-pending").exists());
    }
    fn marker_mutate(&self, change: impl FnOnce(&mut serde_json::Value)) {
        let path = self.root.join(".execution-pending");
        let mut wire = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        change(&mut wire);
        fs::write(path, serde_json::to_vec(&wire).unwrap()).unwrap();
    }
}
#[test]
fn absent_recovery_does_not_create_pending_files() {
    let f = Fixture::new();
    assert!(!recover(&f.manager).unwrap());
    assert!(!f.root.join(".execution-pending").exists());
    assert!(!f.root.join(".execution-readers.json").exists());
    assert_eq!(fs::read(&f.source).unwrap(), b"source preserved");
    assert_eq!(
        fs::read(&f.sentinel).unwrap(),
        b"foreign sentinel preserved"
    );
}
#[test]
fn live_original_creator_is_busy_and_preserves_every_file() {
    let f = Fixture::new();
    let scope = f.manager.begin_execution().unwrap();
    f.reject_preserving();
    scope.finish().unwrap();
}
#[test]
fn dropped_pristine_is_not_startup_cleanup_authority() {
    let f = Fixture::new();
    f.pending();
    f.reject_preserving();
}
#[test]
fn legacy_unknown_and_wrong_base_digest_are_preserved() {
    for variant in 0..3 {
        let f = Fixture::new();
        f.pending();
        f.marker_mutate(|wire| match variant {
            0 => {
                wire["version"] = 2.into();
                wire.as_object_mut().unwrap().remove("managed");
            }
            1 => {
                wire["unknown"] = true.into();
            }
            _ => {
                let old = wire["managed"]["digest"][0].as_u64().unwrap();
                wire["managed"]["digest"][0] = ((old + 1) % 256).into();
            }
        });
        f.reject_preserving();
    }
}
#[test]
fn oversized_and_partial_marker_are_preserved() {
    for payload in [vec![b' '; 4097], b"{\"version\":3".to_vec()] {
        let f = Fixture::new();
        f.pending();
        fs::write(f.root.join(".execution-pending"), payload).unwrap();
        f.reject_preserving();
    }
}
#[test]
fn foreign_receipt_replacement_preserves_original_and_foreign_bytes() {
    let f = Fixture::new();
    f.pending();
    let receipt = f.root.join(".execution-reader-0.receipt");
    let backup = f._temp.path().join("original-receipt");
    fs::rename(&receipt, &backup).unwrap();
    fs::write(&receipt, b"foreign receipt sentinel").unwrap();
    f.reject_preserving();
    assert_eq!(fs::read(backup).unwrap(), b"");
    assert_eq!(fs::read(receipt).unwrap(), b"foreign receipt sentinel");
}
#[test]
fn replaced_execution_lock_never_adopts_current_identity() {
    let f = Fixture::new();
    f.pending();
    let lock = f.root.join(".install-transfer-lock");
    let backup = f._temp.path().join("original-lock");
    fs::rename(&lock, &backup).unwrap();
    fs::write(&lock, b"foreign lock sentinel").unwrap();
    f.reject_preserving();
    assert_eq!(fs::read(backup).unwrap(), b"");
    assert_eq!(fs::read(lock).unwrap(), b"foreign lock sentinel");
}
#[test]
fn widened_owned_marker_acl_cannot_authorize_recovery() {
    use std::{
        fs::OpenOptions,
        os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    };
    use windows_sys::Win32::{
        Security::{
            Authorization::{SE_FILE_OBJECT, SetSecurityInfo},
            DACL_SECURITY_INFORMATION, UNPROTECTED_DACL_SECURITY_INFORMATION,
        },
        Storage::FileSystem::*,
    };
    let f = Fixture::new();
    f.pending();
    let handle = OpenOptions::new()
        .access_mode(WRITE_DAC | READ_CONTROL)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .open(f.root.join(".execution-pending"))
        .unwrap();
    // SAFETY: only this test's owned synthetic marker is made deliberately unsafe.
    assert_eq!(
        unsafe {
            SetSecurityInfo(
                handle.as_raw_handle(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | UNPROTECTED_DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        },
        0
    );
    drop(handle);
    f.reject_preserving();
}
