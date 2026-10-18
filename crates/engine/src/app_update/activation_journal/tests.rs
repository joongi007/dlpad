use super::*;
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde_json::json;
fn fixture() -> (ImageRecord, [u8; 32]) {
    let key = Ed25519KeyPair::from_seed_unchecked(&[17; 32]).unwrap();
    let public = key.public_key().as_ref().try_into().unwrap();
    let raw=serde_json::to_vec(&json!({"schema":1,"domain":"dlpad.app-update.v1","key_id":"fixture","sequence":2,"version":"0.2.0","channel":"stable","target":"x86_64-pc-windows-msvc","repository":"joongi007/dlpad","asset_name":"dlpad-0.2.0-windows-x86_64-installer.zip","asset_url":"https://github.com/joongi007/dlpad/releases/download/v0.2.0/dlpad-0.2.0-windows-x86_64-installer.zip","asset_bytes":3,"asset_sha256":format!("{:x}",Sha256::digest(b"zip")),"issued_at_unix":100,"expires_at_unix":200})).unwrap();
    let signature = key
        .sign(&raw)
        .as_ref()
        .iter()
        .map(|v| format!("{v:02x}"))
        .collect();
    (
        ImageRecord {
            bootstrap_version: "0.1.0".into(),
            metadata: raw,
            signature,
            key_id: "fixture".into(),
            version: "0.2.0".into(),
            channel: "stable".into(),
            sequence: 2,
            validated_at_unix: 150,
            executable_path: r"C:\bundle\registered\dlpad.exe".into(),
            executable_sha256: [7; 32],
            executable_bytes: 70000,
            executable_identity: NativeFileIdentity::Windows {
                volume_serial: 1,
                file_id: [1; 16],
            },
            active_record_sha256: [8; 32],
            bundle_root: r"C:\bundle".into(),
            active_root: r"C:\bundle\active".into(),
            registered_root: r"C:\bundle\registered".into(),
            receipt_root: r"C:\bundle\receipt".into(),
            profile_epoch: 1,
        },
        public,
    )
}
#[test]
fn expired_metadata_allows_previously_validated_code_but_not_new_activation() {
    let (image, key) = fixture();
    let keys = [TrustedKey {
        id: "fixture",
        public_key: &key,
    }];
    validate_image(&image, &keys, 1).unwrap();
    assert!(candidate(&image, &keys, 201).is_err());
}
#[test]
fn modified_metadata_and_unpinned_keys_are_rejected() {
    let (mut image, key) = fixture();
    let keys = [TrustedKey {
        id: "fixture",
        public_key: &key,
    }];
    image.metadata.push(b' ');
    assert!(validate_image(&image, &keys, 1).is_err());
    assert!(validate_image(&image, &[], 1).is_err());
}
#[test]
fn same_or_low_sequence_and_incompatible_epoch_rejected() {
    let (image, key) = fixture();
    let keys = [TrustedKey {
        id: "fixture",
        public_key: &key,
    }];
    assert!(validate_image(&image, &keys, 2).is_err());
    let old = SelectionState {
        highest_sequence: 2,
        ..Default::default()
    };
    let next = SelectionState {
        active: Some(image),
        highest_sequence: 2,
        ..Default::default()
    };
    assert!(valid_transition(&old, &next, &Event::Commit, "0.1.0").is_err());
}
#[test]
fn first_commit_then_bootstrap_rollback_preserves_sequence_barrier() {
    let (image, _) = fixture();
    let initial = SelectionState::default();
    let next = SelectionState {
        active: Some(image),
        previous: None,
        highest_sequence: 2,
        failed_sequence: None,
    };
    valid_transition(&initial, &next, &Event::Commit, "0.1.0").unwrap();
    let rollback = SelectionState {
        active: None,
        previous: None,
        highest_sequence: 2,
        failed_sequence: Some(2),
    };
    valid_transition(&next, &rollback, &Event::Rollback, "0.1.0").unwrap();
    let replay = SelectionState {
        active: next.active.clone(),
        previous: None,
        highest_sequence: 2,
        failed_sequence: Some(2),
    };
    assert!(valid_transition(&rollback, &replay, &Event::Commit, "0.1.0").is_err());
}
#[test]
fn failed_precommit_and_arbitrary_rollback_are_distinct() {
    let (image, _) = fixture();
    let initial = SelectionState::default();
    let failed = SelectionState {
        highest_sequence: 2,
        failed_sequence: Some(2),
        ..Default::default()
    };
    valid_transition(&initial, &failed, &Event::Failed, "0.1.0").unwrap();
    assert!(valid_transition(&failed, &initial, &Event::Rollback, "0.1.0").is_err());
    let invalid = SelectionState {
        active: Some(image),
        highest_sequence: 2,
        ..Default::default()
    };
    assert!(valid_transition(&initial, &invalid, &Event::Rollback, "0.1.0").is_err());
}
#[test]
fn paths_and_record_payload_have_hard_bounds() {
    assert!(!windows_path(r"C:\bundle\..\other"));
    assert!(!windows_path(r"\\server\share\file"));
    assert!(!windows_path("C:\\bundle\\bad\0"));
    let (image, _) = fixture();
    let mut record = Record {
        schema: 1,
        domain: "dlpad.activation-journal.v1".into(),
        generation: 1,
        previous_sha256: [0; 32],
        root_identity: NativeFileIdentity::Windows {
            volume_serial: 1,
            file_id: [1; 16],
        },
        bootstrap_identity: NativeFileIdentity::Windows {
            volume_serial: 1,
            file_id: [2; 16],
        },
        bootstrap_version: "0.1.0".into(),
        profile_epoch: 1,
        event: Event::Commit,
        state: SelectionState {
            active: Some(image),
            highest_sequence: 2,
            ..Default::default()
        },
    };
    assert!(canonical(&record).is_ok());
    record.domain = "a".repeat(MAX_PLAINTEXT);
    assert!(canonical(&record).is_err());
}
#[test]
fn cold_failure_of_older_active_preserves_higher_failed_barrier() {
    let (image, _) = fixture();
    let previous = SelectionState {
        active: Some(image),
        previous: None,
        highest_sequence: 7,
        failed_sequence: Some(7),
    };
    let mut failed = previous.clone();
    failed.failed_sequence = Some(2);
    valid_transition(&previous, &failed, &Event::Failed, "0.1.0").unwrap();
    let rollback = SelectionState {
        active: None,
        previous: None,
        highest_sequence: 7,
        failed_sequence: Some(2),
    };
    valid_transition(&failed, &rollback, &Event::Rollback, "0.1.0").unwrap();
}
#[cfg(windows)]
mod native {
    use super::super::windows::*;
    use super::*;
    use crate::app_install::FreshRoot;
    use std::{
        fs::{File, OpenOptions},
        os::windows::fs::OpenOptionsExt,
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    fn deadline() -> Instant {
        Instant::now() + Duration::from_secs(30)
    }
    fn root_handle(path: &Path) -> File {
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
        };
        OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .unwrap()
    }
    #[test]
    fn identity_handle_is_reopened_for_content_and_denies_foreign_mutation() {
        use std::io::Read;
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("synthetic-image.exe");
        std::fs::write(&path, b"synthetic content").unwrap();
        let original = crate::file_identity::open_regular_no_reparse(&path).unwrap();
        let mut identity_only = original.try_clone().unwrap();
        assert_eq!(
            identity_only.read(&mut [0u8; 1]).unwrap_err().kind(),
            std::io::ErrorKind::PermissionDenied
        );
        let mut content = reopen_content(&original).unwrap();
        let mut bytes = Vec::new();
        content.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"synthetic content");
        assert_eq!(
            crate::file_identity::from_file(&content).unwrap(),
            crate::file_identity::from_file(&original).unwrap()
        );
        assert!(OpenOptions::new().write(true).open(&path).is_err());
        assert!(std::fs::rename(&path, temp.path().join("changed.exe")).is_err());
        drop(content);
        assert_eq!(std::fs::read(&path).unwrap(), b"synthetic content");
    }
    #[test]
    fn dpapi_authenticates_bytes_and_native_binding() {
        let entropy = b"fixture root/bootstrap binding";
        let cipher = protect(b"bounded journal", entropy, false).unwrap();
        assert_eq!(protect(&cipher, entropy, true).unwrap(), b"bounded journal");
        assert!(protect(&cipher, b"other bootstrap", true).is_err());
        let mut changed = cipher;
        let at = changed.len() / 2;
        changed[at] ^= 1;
        assert!(protect(&changed, entropy, true).is_err());
    }
    #[test]
    fn failure_head_is_durable_idempotent_and_rejects_stale_cas() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("journal");
        let _fresh = FreshRoot::create_new(&path).unwrap();
        let path = std::fs::canonicalize(&path).unwrap();
        let root = root_handle(&path);
        // Match the host: retain original readonly identity, then release creation DELETE caps.
        _fresh.verify().unwrap();
        drop(_fresh);
        let bootstrap_path = temp.path().join("bootstrap.exe");
        std::fs::write(&bootstrap_path, b"synthetic baseline").unwrap();
        let bootstrap = File::open(&bootstrap_path).unwrap();
        let (_, key) = fixture();
        let keys = [TrustedKey {
            id: "fixture",
            public_key: &key,
        }];
        let cancel = AtomicBool::new(false);
        let mut journal = Journal::open(
            JournalBinding {
                root: &root,
                root_path: &path,
                bootstrap: &bootstrap,
                bootstrap_version: "0.1.0",
                profile_epoch: 1,
            },
            &keys,
            &cancel,
            deadline(),
        )
        .unwrap();
        let baseline = journal.head().clone();
        let failed = journal
            .record_failed(&baseline, 2, &cancel, deadline())
            .unwrap();
        assert_eq!(failed.highest_sequence(), 2);
        assert_eq!(failed.generation(), 1);
        assert_eq!(
            journal
                .record_failed(&failed, 2, &cancel, deadline())
                .unwrap()
                .generation(),
            1
        );
        assert!(
            journal
                .record_failed(&baseline, 3, &cancel, deadline())
                .is_err()
        );
        drop(journal);
        let reopened = Journal::open(
            JournalBinding {
                root: &root,
                root_path: &path,
                bootstrap: &bootstrap,
                bootstrap_version: "0.1.0",
                profile_epoch: 1,
            },
            &keys,
            &cancel,
            deadline(),
        )
        .unwrap();
        assert_eq!(reopened.head().highest_sequence(), 2);
        assert_eq!(reopened.head().failed_sequence(), Some(2));
    }
    #[test]
    fn unknown_namespace_and_partial_records_fail_closed() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("journal");
        let _fresh = FreshRoot::create_new(&path).unwrap();
        let path = std::fs::canonicalize(&path).unwrap();
        let root = root_handle(&path);
        // Match the host: retain original readonly identity, then release creation DELETE caps.
        _fresh.verify().unwrap();
        drop(_fresh);
        let bootstrap_path = temp.path().join("bootstrap.exe");
        std::fs::write(&bootstrap_path, b"baseline").unwrap();
        let bootstrap = File::open(&bootstrap_path).unwrap();
        let (_, key) = fixture();
        let keys = [TrustedKey {
            id: "fixture",
            public_key: &key,
        }];
        std::fs::write(path.join("unknown"), b"preserve").unwrap();
        assert!(
            Journal::open(
                JournalBinding {
                    root: &root,
                    root_path: &path,
                    bootstrap: &bootstrap,
                    bootstrap_version: "0.1.0",
                    profile_epoch: 1
                },
                &keys,
                &AtomicBool::new(false),
                deadline()
            )
            .is_err()
        );
        assert_eq!(std::fs::read(path.join("unknown")).unwrap(), b"preserve");
        let partial_path = temp.path().join("partial-journal");
        let _partial_fresh = FreshRoot::create_new(&partial_path).unwrap();
        let partial_path = std::fs::canonicalize(&partial_path).unwrap();
        let partial_root = root_handle(&partial_path);
        let mut partial = crate::created_relative_file::CreatedRelativeFile::create(
            &partial_root,
            "slot0001.dpapi",
        )
        .unwrap();
        use std::io::Write;
        partial.file_mut().write_all(b"partial").unwrap();
        partial.file_mut().sync_all().unwrap();
        drop(partial);
        assert!(
            Journal::open(
                JournalBinding {
                    root: &partial_root,
                    root_path: &partial_path,
                    bootstrap: &bootstrap,
                    bootstrap_version: "0.1.0",
                    profile_epoch: 1
                },
                &keys,
                &AtomicBool::new(false),
                deadline()
            )
            .is_err()
        );
        assert_eq!(
            std::fs::read(partial_path.join("slot0001.dpapi")).unwrap(),
            b"partial"
        );
    }
}
