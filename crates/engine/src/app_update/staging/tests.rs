use super::*;
use crate::app_update::{Channel, Policy, TrustedKey, verify_metadata};
use ring::signature::{Ed25519KeyPair, KeyPair};
use sha2::{Digest, Sha256};
#[cfg(windows)]
use std::sync::atomic::Ordering;
use std::{io::Cursor, time::Duration};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn candidate() -> VerifiedCandidate {
    let key = Ed25519KeyPair::from_seed_unchecked(&[7; 32]).unwrap();
    let public: [u8; 32] = key.public_key().as_ref().try_into().unwrap();
    let payload=serde_json::to_vec(&serde_json::json!({
        "schema":1,"domain":"dlpad.app-update.v1","key_id":"fixture","sequence":2,
        "version":"0.2.0","channel":"stable","target":"x86_64-pc-windows-msvc",
        "repository":"joongi007/dlpad","asset_name":"dlpad-0.2.0-windows-x86_64-installer.zip",
        "asset_url":"https://github.com/joongi007/dlpad/releases/download/v0.2.0/dlpad-0.2.0-windows-x86_64-installer.zip",
        "asset_bytes":3,"asset_sha256":hex(&Sha256::digest(b"abc")),"issued_at_unix":100,"expires_at_unix":200,
    })).unwrap();
    verify_metadata(
        &payload,
        "fixture",
        &hex(key.sign(&payload).as_ref()),
        &[TrustedKey {
            id: "fixture",
            public_key: &public,
        }],
        &Policy {
            installed_version: "0.1.0",
            installed_sequence: 1,
            channel: Channel::Stable,
            now_unix: 150,
        },
    )
    .unwrap()
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
#[cfg(windows)]
fn parent(path: &Path) -> File {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    };
    std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .unwrap()
}

#[cfg(not(windows))]
#[test]
fn non_windows_is_unsupported_without_consuming_source_or_creating_files() {
    let temporary = tempfile::tempdir().unwrap();
    let parent = File::open(temporary.path()).unwrap();
    let mut source = Cursor::new(b"abc");
    let failure = stage_verified_package(
        &candidate(),
        &mut source,
        StageRequest {
            parent: &parent,
            parent_path: temporary.path(),
            nonce: "fixture",
        },
        &AtomicBool::new(false),
        deadline(),
    )
    .unwrap_err();
    assert_eq!(
        failure.kind,
        StageFailureKind::Io(io::ErrorKind::Unsupported)
    );
    assert_eq!(failure.creation, Creation::NotAttempted);
    assert_eq!(source.position(), 0);
    assert_eq!(std::fs::read_dir(temporary.path()).unwrap().count(), 0);
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::io::{Read, Write};
    #[test]
    fn successful_export_retains_original_readonly_pin_and_drop_preserves_bytes() {
        let temp = tempfile::tempdir().unwrap();
        let path = std::fs::canonicalize(temp.path()).unwrap();
        let parent = parent(&path);
        let c = candidate();
        let mut p = stage_verified_package(
            &c,
            &mut Cursor::new(b"abc"),
            StageRequest {
                parent: &parent,
                parent_path: &path,
                nonce: "success",
            },
            &AtomicBool::new(false),
            deadline(),
        )
        .unwrap();
        assert_eq!(p.version(), "0.2.0");
        assert_eq!(p.sequence(), 2);
        assert_eq!(p.bytes(), 3);
        assert_eq!(p.sha256().as_slice(), Sha256::digest(b"abc").as_slice());
        p.verify_current().unwrap();
        assert_eq!(
            crate::file_identity::from_file(p.reader_mut()).unwrap(),
            *p.identity()
        );
        let mut observed = Vec::new();
        p.reader_mut().read_to_end(&mut observed).unwrap();
        assert_eq!(observed, b"abc");
        assert!(p.reader_mut().write_all(b"bad").is_err());
        let output = p.path().to_owned();
        drop(p);
        assert_eq!(std::fs::read(output).unwrap(), b"abc");
        assert!(!path.join(".dlpad-update-success.partial").exists());
    }
    #[test]
    fn final_and_stage_collisions_preserve_existing_identity_and_bytes() {
        for stage_collision in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let path = std::fs::canonicalize(temp.path()).unwrap();
            let parent = parent(&path);
            let c = candidate();
            let output = path.join(if stage_collision {
                ".dlpad-update-collision.partial"
            } else {
                c.asset_name()
            });
            std::fs::write(&output, b"user original").unwrap();
            let original = crate::file_identity::from_file(&File::open(&output).unwrap()).unwrap();
            let failure = stage_verified_package(
                &c,
                &mut Cursor::new(b"abc"),
                StageRequest {
                    parent: &parent,
                    parent_path: &path,
                    nonce: "collision",
                },
                &AtomicBool::new(false),
                deadline(),
            )
            .unwrap_err();
            assert_eq!(std::fs::read(&output).unwrap(), b"user original");
            assert_eq!(
                crate::file_identity::from_file(&File::open(&output).unwrap()).unwrap(),
                original
            );
            assert_eq!(failure.publication, Publication::NotAttempted);
            assert_eq!(
                failure.creation,
                if stage_collision {
                    Creation::AttemptedUnconfirmed
                } else {
                    Creation::OwnedCreated
                }
            );
            assert_eq!(
                failure.cleanup,
                if stage_collision {
                    Cleanup::Unconfirmed
                } else {
                    Cleanup::OriginalRemoved
                }
            );
        }
    }
    #[test]
    fn bad_hash_short_long_and_read_error_cleanup_only_owned_stage() {
        struct Broken;
        impl Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::ErrorKind::UnexpectedEof.into())
            }
        }
        for case in [0, 1, 2, 3] {
            let temp = tempfile::tempdir().unwrap();
            let path = std::fs::canonicalize(temp.path()).unwrap();
            let parent = parent(&path);
            let c = candidate();
            std::fs::write(path.join("sentinel"), b"preserve").unwrap();
            let mut source: Box<dyn Read> = match case {
                0 => Box::new(Cursor::new(b"abd")),
                1 => Box::new(Cursor::new(b"ab")),
                2 => Box::new(Cursor::new(b"abcd")),
                _ => Box::new(Broken),
            };
            let failure = stage_verified_package(
                &c,
                &mut source,
                StageRequest {
                    parent: &parent,
                    parent_path: &path,
                    nonce: "invalid",
                },
                &AtomicBool::new(false),
                deadline(),
            )
            .unwrap_err();
            assert_eq!(failure.cleanup, Cleanup::OriginalRemoved);
            assert_eq!(failure.publication, Publication::NotAttempted);
            assert!(!path.join(c.asset_name()).exists());
            assert!(!path.join(".dlpad-update-invalid.partial").exists());
            assert_eq!(std::fs::read(path.join("sentinel")).unwrap(), b"preserve");
        }
    }
    #[test]
    fn cancellation_before_and_during_body_and_deadline_never_publish() {
        struct Cancel<'a>(&'a AtomicBool);
        impl Read for Cancel<'_> {
            fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
                b[0] = b'a';
                self.0.store(true, Ordering::Release);
                Ok(1)
            }
        }
        let temp = tempfile::tempdir().unwrap();
        let path = std::fs::canonicalize(temp.path()).unwrap();
        let parent = parent(&path);
        let c = candidate();
        let stopped = AtomicBool::new(true);
        let failure = stage_verified_package(
            &c,
            &mut Cursor::new(b"abc"),
            StageRequest {
                parent: &parent,
                parent_path: &path,
                nonce: "pre",
            },
            &stopped,
            deadline(),
        )
        .unwrap_err();
        assert_eq!(failure.kind, StageFailureKind::Validation(Error::Cancelled));
        assert_eq!(failure.creation, Creation::NotAttempted);
        let cancel = AtomicBool::new(false);
        let failure = stage_verified_package(
            &c,
            &mut Cancel(&cancel),
            StageRequest {
                parent: &parent,
                parent_path: &path,
                nonce: "during",
            },
            &cancel,
            deadline(),
        )
        .unwrap_err();
        assert_eq!(failure.kind, StageFailureKind::Validation(Error::Cancelled));
        assert_eq!(failure.cleanup, Cleanup::OriginalRemoved);
        let failure = stage_verified_package(
            &c,
            &mut Cursor::new(b"abc"),
            StageRequest {
                parent: &parent,
                parent_path: &path,
                nonce: "deadline",
            },
            &AtomicBool::new(false),
            Instant::now(),
        )
        .unwrap_err();
        assert_eq!(failure.kind, StageFailureKind::Validation(Error::TimedOut));
        assert_eq!(failure.creation, Creation::NotAttempted);
        assert_eq!(std::fs::read_dir(&path).unwrap().count(), 0);
    }
    #[test]
    fn invalid_nonce_and_mismatched_parent_path_create_nothing() {
        let temp = tempfile::tempdir().unwrap();
        let path = std::fs::canonicalize(temp.path()).unwrap();
        let parent = parent(&path);
        let c = candidate();
        for nonce in ["", "../escape", "contains space"] {
            let failure = stage_verified_package(
                &c,
                &mut Cursor::new(b"abc"),
                StageRequest {
                    parent: &parent,
                    parent_path: &path,
                    nonce,
                },
                &AtomicBool::new(false),
                deadline(),
            )
            .unwrap_err();
            assert_eq!(failure.creation, Creation::NotAttempted);
        }
        let failure = stage_verified_package(
            &c,
            &mut Cursor::new(b"abc"),
            StageRequest {
                parent: &parent,
                parent_path: &path.join("wrong"),
                nonce: "mismatch",
            },
            &AtomicBool::new(false),
            deadline(),
        )
        .unwrap_err();
        assert_eq!(failure.creation, Creation::NotAttempted);
        assert_eq!(std::fs::read_dir(&path).unwrap().count(), 0);
    }
    #[test]
    fn replaced_selected_parent_is_rejected_before_creation_and_preserves_both_folders() {
        let temp = tempfile::tempdir().unwrap();
        let original = temp.path().join("selected");
        let moved = temp.path().join("moved");
        std::fs::create_dir(&original).unwrap();
        std::fs::write(original.join("sentinel"), b"old parent").unwrap();
        let original = std::fs::canonicalize(original).unwrap();
        let parent = parent(&original);
        std::fs::rename(&original, &moved).unwrap();
        std::fs::create_dir(&original).unwrap();
        std::fs::write(original.join("sentinel"), b"new parent").unwrap();
        let c = candidate();
        let failure = stage_verified_package(
            &c,
            &mut Cursor::new(b"abc"),
            StageRequest {
                parent: &parent,
                parent_path: &original,
                nonce: "replaced",
            },
            &AtomicBool::new(false),
            deadline(),
        )
        .unwrap_err();
        assert_eq!(failure.creation, Creation::NotAttempted);
        assert_eq!(
            std::fs::read(original.join("sentinel")).unwrap(),
            b"new parent"
        );
        assert_eq!(
            std::fs::read(moved.join("sentinel")).unwrap(),
            b"old parent"
        );
        assert!(!original.join(c.asset_name()).exists());
        assert!(!moved.join(c.asset_name()).exists());
    }
}
