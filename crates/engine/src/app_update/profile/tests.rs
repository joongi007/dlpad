use super::*;
use sha2::{Digest, Sha256};
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(5)
}
fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = fs::read_dir(root)
        .unwrap()
        .map(|p| {
            let p = p.unwrap();
            (
                p.file_name().to_string_lossy().into_owned(),
                Sha256::digest(fs::read(p.path()).unwrap()).to_vec(),
            )
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}
#[test]
fn existing_profile_is_read_without_writes_or_new_sidecars() {
    let d = tempfile::tempdir().unwrap();
    for name in ["theme.sqlite", "library.sqlite"] {
        let s = crate::store::Store::open(&d.path().join(name)).unwrap();
        s.set_setting("fixture", "한글 #?%").unwrap();
    }
    drop(crate::download::DownloadRepository::open(&d.path().join("jobs.sqlite")).unwrap());
    let before = snapshot(d.path());
    inspect(d.path(), PROFILE_EPOCH, &AtomicBool::new(false), deadline()).unwrap();
    assert_eq!(snapshot(d.path()), before);
}
#[test]
fn fresh_profile_and_reserved_uri_characters_are_preserved() {
    let d = tempfile::tempdir().unwrap();
    let root = d.path().join(if cfg!(windows) {
        "한글 #% 프로필"
    } else {
        "한글 #?% 프로필"
    });
    fs::create_dir(&root).unwrap();
    inspect(&root, PROFILE_EPOCH, &AtomicBool::new(false), deadline()).unwrap();
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    drop(crate::store::Store::open(&root.join("theme.sqlite")).unwrap());
    let before = snapshot(&root);
    inspect(
        &root.canonicalize().unwrap(),
        PROFILE_EPOCH,
        &AtomicBool::new(false),
        deadline(),
    )
    .unwrap();
    assert_eq!(snapshot(&root), before);
}
#[test]
fn newer_schema_and_wrong_epoch_fail_without_mutation() {
    let d = tempfile::tempdir().unwrap();
    let db = d.path().join("library.sqlite");
    drop(crate::store::Store::open(&db).unwrap());
    {
        let c = Connection::open(&db).unwrap();
        c.execute("INSERT INTO schema_migrations VALUES(3)", [])
            .unwrap();
    }
    let before = snapshot(d.path());
    assert_eq!(
        inspect(d.path(), 1, &AtomicBool::new(false), deadline()),
        Err(Error::Incompatible)
    );
    assert_eq!(snapshot(d.path()), before);
    assert_eq!(
        inspect(d.path(), 2, &AtomicBool::new(false), deadline()),
        Err(Error::Incompatible)
    );
}
#[test]
fn corrupt_database_and_pending_wal_remain_unchanged() {
    let d = tempfile::tempdir().unwrap();
    let db = d.path().join("jobs.sqlite");
    fs::write(&db, b"synthetic damaged sqlite").unwrap();
    let before = snapshot(d.path());
    assert!(inspect(d.path(), 1, &AtomicBool::new(false), deadline()).is_err());
    assert_eq!(snapshot(d.path()), before);
    fs::write(
        d.path().join("jobs.sqlite-wal"),
        b"synthetic uncheckpointed WAL",
    )
    .unwrap();
    let before = snapshot(d.path());
    assert_eq!(
        inspect(d.path(), 1, &AtomicBool::new(false), deadline()),
        Err(Error::PendingRecovery)
    );
    assert_eq!(snapshot(d.path()), before);
}
#[test]
fn supported_migration_labels_do_not_adopt_a_missing_or_incompatible_table() {
    let d = tempfile::tempdir().unwrap();
    let db = d.path().join("theme.sqlite");
    drop(crate::store::Store::open(&db).unwrap());
    {
        let connection = Connection::open(&db).unwrap();
        connection.execute_batch("DROP TABLE settings;").unwrap();
    }
    let before = snapshot(d.path());
    assert_eq!(
        inspect(d.path(), 1, &AtomicBool::new(false), deadline()),
        Err(Error::Incompatible)
    );
    assert_eq!(snapshot(d.path()), before);
    {
        let connection = Connection::open(&db).unwrap();
        connection
            .execute_batch("CREATE TABLE settings(unrelated TEXT);")
            .unwrap();
    }
    let before = snapshot(d.path());
    assert_eq!(
        inspect(d.path(), 1, &AtomicBool::new(false), deadline()),
        Err(Error::Incompatible)
    );
    assert_eq!(snapshot(d.path()), before);
}
#[test]
fn cancellation_and_timeout_never_open_or_create_databases() {
    let d = tempfile::tempdir().unwrap();
    assert_eq!(
        inspect(d.path(), 1, &AtomicBool::new(true), deadline()),
        Err(Error::Cancelled)
    );
    assert_eq!(
        inspect(d.path(), 1, &AtomicBool::new(false), Instant::now()),
        Err(Error::TimedOut)
    );
    assert_eq!(fs::read_dir(d.path()).unwrap().count(), 0);
}
#[cfg(unix)]
#[test]
fn symlinked_profile_or_database_is_rejected() {
    use std::os::unix::fs::symlink;
    let d = tempfile::tempdir().unwrap();
    let root = d.path().join("root");
    fs::create_dir(&root).unwrap();
    let outside = d.path().join("external");
    fs::write(&outside, b"preserve").unwrap();
    symlink(&outside, root.join("theme.sqlite")).unwrap();
    assert_eq!(
        inspect(&root, 1, &AtomicBool::new(false), deadline()),
        Err(Error::Incompatible)
    );
    assert_eq!(fs::read(outside).unwrap(), b"preserve");
}
