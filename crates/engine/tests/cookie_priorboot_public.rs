//! Synthetic previous-boot fixture, exercising the normally compiled public library.
#![cfg(target_os = "linux")]
#[path = "../src/process/reader_protocol.rs"]
mod reader_protocol;
mod process {
    pub(crate) use crate::reader_protocol;
}
#[path = "../src/cookies/reader_records.rs"]
mod reader_records;
mod cookies {
    pub(crate) use crate::reader_records;
}
#[path = "../src/cookies/reader_journal.rs"]
mod reader_journal;

use dlpad_engine::cookies::{import_session, recover_sessions};
use reader_journal::{JournalAnchor, ReaderJournal};
use reader_protocol::{CleanupReceipt, ReaderIdentity, encode_cleanup_receipt};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

struct Fixture {
    _temp: tempfile::TempDir,
    source: PathBuf,
    source_bytes: Vec<u8>,
    sentinel: PathBuf,
    cache: PathBuf,
    owner: PathBuf,
    anchor: JournalAnchor,
    actual_boot: String,
    old_boot: String,
    old_bytes: [u8; 16],
    digest: [u8; 32],
}
fn decode<const N: usize>(text: &str) -> [u8; N] {
    assert_eq!(text.len(), N * 2);
    let mut bytes = [0; N];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).unwrap();
    }
    bytes
}
fn replace_one(text: &str, old: &str, new: &str) -> String {
    assert_eq!(text.matches(old).count(), 1);
    text.replacen(old, new, 1)
}
impl Fixture {
    fn new(previous: bool) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let source_bytes = b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\tsynthetic-secret\n".to_vec();
        fs::write(&source, &source_bytes).unwrap();
        let sentinel = temp.path().join("foreign-sentinel");
        fs::write(&sentinel, b"foreign sentinel").unwrap();
        let cache = temp.path().join("cache");
        fs::create_dir(&cache).unwrap();
        fs::set_permissions(&cache, fs::Permissions::from_mode(0o700)).unwrap();
        let mut lease = import_session(&source, &cache).unwrap();
        let owner = lease.path_for_tool().parent().unwrap().to_owned();
        let record = fs::read_to_string(owner.join("recovery.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&record).unwrap();
        let binding = &parsed["journal"];
        let anchor = JournalAnchor {
            directory_dev: binding["directory_dev"].as_u64().unwrap(),
            directory_ino: binding["directory_ino"].as_u64().unwrap(),
            record_dev: binding["record_dev"].as_u64().unwrap(),
            record_ino: binding["record_ino"].as_u64().unwrap(),
        };
        let actual_boot = parsed["boot"].as_str().unwrap().to_owned();
        let mut old_boot = actual_boot.clone().into_bytes();
        if previous {
            old_boot[0] = if old_boot[0] == b'0' { b'1' } else { b'0' };
        }
        let old_boot = String::from_utf8(old_boot).unwrap();
        let old_bytes = decode(&old_boot.replace('-', ""));
        lease.defer_cleanup();
        drop(lease);
        let old_record = replace_one(
            &record,
            &format!("\"boot\":\"{actual_boot}\""),
            &format!("\"boot\":\"{old_boot}\""),
        );
        assert_eq!(old_record.len(), record.len());
        let digest: [u8; 32] = Sha256::digest(old_record.as_bytes()).into();
        let marker = fs::read_to_string(owner.join("owner.marker")).unwrap();
        let marker_value: serde_json::Value = serde_json::from_str(&marker).unwrap();
        let old_digest = marker_value["digest"].as_str().unwrap();
        let new_digest = format!("{:x}", Sha256::digest(old_record.as_bytes()));
        let new_marker = replace_one(
            &marker,
            &format!("\"digest\":\"{old_digest}\""),
            &format!("\"digest\":\"{new_digest}\""),
        );
        fs::write(owner.join("recovery.json"), old_record.as_bytes()).unwrap();
        fs::write(owner.join("owner.marker"), new_marker.as_bytes()).unwrap();
        File::open(owner.join("recovery.json"))
            .unwrap()
            .sync_all()
            .unwrap();
        File::open(owner.join("owner.marker"))
            .unwrap()
            .sync_all()
            .unwrap();
        File::open(&owner).unwrap().sync_all().unwrap();
        Self {
            _temp: temp,
            source,
            source_bytes,
            sentinel,
            cache,
            owner,
            anchor,
            actual_boot,
            old_boot,
            old_bytes,
            digest,
        }
    }
    fn identity(&self, capability: reader_protocol::Capability) -> ReaderIdentity {
        let mut identity = ReaderIdentity {
            capability,
            identity_digest: [1; 32],
            unit: [2; 16],
            invocation: [3; 16],
            cgroup_mount: 4,
            cgroup_dev: 5,
            cgroup_ino: 6,
            path: "/synthetic.slice".into(),
        };
        identity.identity_digest = reader_records::identity_digest(&identity).unwrap();
        identity
    }
    fn journal(&self, slot: u8, bound: bool, mismatch: Option<&str>) -> ReaderJournal {
        let mut journal =
            ReaderJournal::reopen(File::open(&self.owner).unwrap(), self.anchor).unwrap();
        if slot == 1 {
            let mut launch = journal
                .begin(0, [1; 16], self.digest, self.old_bytes)
                .unwrap();
            let identity = self.identity(launch.capability.clone());
            journal.bind(identity.clone()).unwrap();
            launch
                .writer
                .write_all(
                    &encode_cleanup_receipt(&CleanupReceipt::from_identity(&identity)).unwrap(),
                )
                .unwrap();
            launch.writer.sync_all().unwrap();
            journal.resolve(0).unwrap();
            drop(launch);
        }
        let boot = if mismatch == Some("boot") {
            decode(&self.actual_boot.replace('-', ""))
        } else {
            self.old_bytes
        };
        let digest = if mismatch == Some("digest") {
            [9; 32]
        } else {
            self.digest
        };
        let launch = journal.begin(slot, [7; 16], digest, boot).unwrap();
        if bound {
            journal
                .bind(self.identity(launch.capability.clone()))
                .unwrap();
        }
        drop(launch);
        journal
    }
    fn unchanged_inputs(&self) {
        assert_eq!(fs::read(&self.source).unwrap(), self.source_bytes);
        assert_eq!(fs::read(&self.sentinel).unwrap(), b"foreign sentinel");
    }
}
#[test]
fn public_production_previous_boot_authority_recovers_four_pending_combinations() {
    if reader_protocol::run_isolated_lock_test() {
        return;
    }
    for (slot, bound) in [(0, false), (0, true), (1, false), (1, true)] {
        let fixture = Fixture::new(true);
        assert_ne!(fixture.actual_boot, fixture.old_boot);
        drop(fixture.journal(slot, bound, None));
        let summary = recover_sessions(&fixture.cache).unwrap();
        assert_eq!(
            (
                summary.scanned,
                summary.recovered,
                summary.deferred,
                summary.rejected
            ),
            (1, 1, 0, 0)
        );
        assert!(!fixture.owner.exists());
        fixture.unchanged_inputs();
    }
}
#[test]
fn public_production_recovery_preserves_sameboot_and_untrusted_pending_records() {
    if reader_protocol::run_isolated_lock_test() {
        return;
    }
    for case in [
        "sameboot",
        "boot",
        "digest",
        "hardlink",
        "busy",
        "extra",
        "partialrecord",
    ] {
        let fixture = Fixture::new(case != "sameboot");
        let journal = fixture.journal(0, true, Some(case));
        let retained = if case == "busy" {
            Some(journal)
        } else {
            drop(journal);
            None
        };
        match case {
            "hardlink" => fs::hard_link(
                fixture.owner.join("cookies.txt"),
                fixture._temp.path().join("cookie-link"),
            )
            .unwrap(),
            "extra" => fs::write(fixture.owner.join("unknown"), b"foreign entry").unwrap(),
            "partialrecord" => {
                fs::write(fixture.owner.join("recovery.json"), b"{\"version\":").unwrap()
            }
            _ => {}
        }
        let record_before = fs::read(fixture.owner.join("recovery.json")).unwrap();
        let state_before = fs::read(fixture.owner.join("recordstate.json")).unwrap();
        let cookie_before = fs::read(fixture.owner.join("cookies.txt")).unwrap();
        let summary = recover_sessions(&fixture.cache).unwrap();
        let expected = if matches!(case, "sameboot" | "busy") {
            (1, 0, 1, 0)
        } else {
            (1, 0, 0, 1)
        };
        assert_eq!(
            (
                summary.scanned,
                summary.recovered,
                summary.deferred,
                summary.rejected
            ),
            expected,
            "failed preservation case {case}"
        );
        assert!(!summary.truncated);
        assert!(fixture.owner.join("cookies.txt").is_file());
        assert_eq!(
            fs::read(fixture.owner.join("cookies.txt")).unwrap(),
            cookie_before
        );
        if case == "hardlink" {
            assert_eq!(
                fs::read(fixture._temp.path().join("cookie-link")).unwrap(),
                cookie_before
            );
        }
        if case == "extra" {
            assert_eq!(
                fs::read(fixture.owner.join("unknown")).unwrap(),
                b"foreign entry"
            );
        }
        assert_eq!(
            fs::read(fixture.owner.join("recovery.json")).unwrap(),
            record_before
        );
        assert_eq!(
            fs::read(fixture.owner.join("recordstate.json")).unwrap(),
            state_before
        );
        fixture.unchanged_inputs();
        drop(retained);
    }
}
