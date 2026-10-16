//! Compile the pending reader contract in Cargo without claiming native recovery integration.
#[path = "../src/process/reader_protocol.rs"]
mod reader_protocol;
mod process {
    pub(crate) use crate::reader_protocol;
}
#[path = "../src/cookies/reader_records.rs"]
mod reader_records;

#[cfg(target_os = "linux")]
mod cookies {
    pub(crate) use crate::reader_records;
}
#[cfg(target_os = "linux")]
#[path = "../src/cookies/reader_journal.rs"]
mod reader_journal;

#[cfg(target_os = "linux")]
#[path = "../src/process/reader_gate.rs"]
mod reader_gate;

#[cfg(target_os = "linux")]
#[test]
fn begin_requires_actual_durable_journal_and_receipt_before_cleanup() {
    if reader_protocol::run_isolated_lock_test() {
        return;
    }
    use reader_protocol::{CleanupReceipt, ReaderIdentity};
    use std::{
        fs::File,
        io::Write,
        os::unix::net::UnixStream,
        thread,
        time::{Duration, Instant},
    };
    let folder = tempfile::tempdir().unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(folder.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut journal =
        reader_journal::ReaderJournal::create(File::open(folder.path()).unwrap()).unwrap();
    let mut launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
    let mut identity = ReaderIdentity {
        capability: launch.capability.clone(),
        identity_digest: [1; 32],
        unit: [4; 16],
        invocation: [5; 16],
        cgroup_mount: 6,
        cgroup_dev: 7,
        cgroup_ino: 8,
        path: "/user.slice/synthetic-owned.service".into(),
    };
    identity.identity_digest = reader_records::identity_digest(&identity).unwrap();
    let anchor = journal.anchor();
    let (mut app, mut peer) = UnixStream::pair().unwrap();
    let sent = identity.clone();
    let worker = thread::spawn(move || {
        reader_gate::send_identity_then_wait_begin(
            &sent,
            &mut peer,
            Instant::now() + Duration::from_secs(2),
            None,
        )
        .unwrap();
        launch
            .writer
            .write_all(
                &reader_protocol::encode_cleanup_receipt(&CleanupReceipt::from_identity(&sent))
                    .unwrap(),
            )
            .unwrap();
        launch.writer.sync_all().unwrap();
    });
    let received = reader_gate::receive_identity_then_begin(
        &mut app,
        Instant::now() + Duration::from_secs(2),
        None,
        |value| {
            journal.bind(value.clone())?;
            assert!(!journal.cleanup_eligible().unwrap());
            Ok::<(), reader_journal::JournalError>(())
        },
    )
    .unwrap();
    assert_eq!(received, identity);
    worker.join().unwrap();
    // Even a valid receipt must be explicitly validated and durably resolved.
    assert!(!journal.cleanup_eligible().unwrap());
    journal.resolve(0).unwrap();
    assert!(journal.cleanup_eligible().unwrap());
    drop(journal);
    drop(launch.lock);
    let reopened =
        reader_journal::ReaderJournal::reopen(File::open(folder.path()).unwrap(), anchor).unwrap();
    assert!(reopened.cleanup_eligible().unwrap());
}

#[cfg(target_os = "linux")]
#[path = "../src/process/reader_cgroup.rs"]
mod reader_cgroup;

#[cfg(target_os = "linux")]
#[test]
#[ignore = "explicit native cgroup capability: reads only this test process group"]
fn actual_live_cgroup_cannot_confirm_empty() {
    let text = std::fs::read_to_string("/proc/self/cgroup").unwrap();
    let relative = text
        .lines()
        .find_map(|line| line.strip_prefix("0::"))
        .unwrap();
    assert_ne!(relative, "/", "root group is outside reader proof contract");
    let path = std::path::Path::new("/sys/fs/cgroup").join(relative.trim_start_matches('/'));
    let pinned = reader_cgroup::PinnedCgroup::open(&path).unwrap();
    let identity = pinned.identity();
    assert_eq!(identity.relative_path, relative);
    assert_ne!(identity.mount_id, 0);
    assert_ne!(identity.inode, 0);
    assert!(
        pinned.confirm_empty().is_err(),
        "this live test must never prove its own group empty"
    );
}

#[cfg(target_os = "linux")]
#[path = "../src/process/reader_receipt.rs"]
mod reader_receipt;
