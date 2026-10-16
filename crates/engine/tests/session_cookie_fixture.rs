//! Explicit synthetic GUI fixture generator; never reads user authentication.
#![cfg(target_os = "linux")]
use dlpad_engine::cookies::import_session;
use std::{fs, os::unix::fs::MetadataExt, path::Path};

fn owned_path(path: &Path) {
    assert!(path.is_absolute() && path.starts_with("/tmp"));
    for component in path
        .ancestors()
        .take_while(|part| *part != Path::new("/tmp"))
    {
        let metadata = fs::symlink_metadata(component).unwrap();
        assert!(!metadata.file_type().is_symlink());
        // SAFETY: getuid reads the current effective task ownership context.
        assert_eq!(metadata.uid(), unsafe { libc::getuid() });
    }
}

#[test]
#[ignore = "explicit owned /tmp fixture for private native GUI only"]
fn create_deferred_session_cookie_fixture() {
    let root = std::env::var_os("DLPAD_OWNED_COOKIE_FIXTURE_ROOT")
        .map(std::path::PathBuf::from)
        .expect("explicit owned fixture root required");
    assert!(
        root.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("dlpad-cookie-gui-owned-")
    );
    owned_path(&root);
    assert!(root.is_dir() && root.metadata().unwrap().mode() & 0o777 == 0o700);
    let source = root.join("source.cookies");
    let cache = root.join("data/session-cookies");
    owned_path(&source);
    owned_path(&cache);
    assert!(source.is_file() && source.metadata().unwrap().len() <= 4096);
    assert_eq!(fs::read(&source).unwrap(), b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\tsynthetic-native-session-only\n");
    assert!(cache.is_dir() && cache.metadata().unwrap().mode() & 0o777 == 0o700);
    let before = fs::read(&source).unwrap();
    let mut lease = import_session(&source, &cache).unwrap();
    let path = lease.path_for_tool().to_owned();
    lease.defer_cleanup();
    assert!(lease.validate_for_tool().is_err());
    assert_eq!(fs::read(&source).unwrap(), before);
    assert_eq!(fs::read(&path).unwrap(), before);
    assert!(path.parent().unwrap().join("recovery.json").is_file());
    assert!(path.parent().unwrap().join("owner.marker").is_file());
    // The harness inspects only owned directory metadata/hash after this driver
    // exits. No path or raw synthetic credential is printed by the generator.
}
