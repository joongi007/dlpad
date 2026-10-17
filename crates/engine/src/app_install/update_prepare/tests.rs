use super::*;
use crate::app_update::{
    Channel, Policy, StageRequest, TrustedKey, VerifiedCandidate, stage_verified_package,
    verify_metadata,
};
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    io::{Cursor, Write},
    os::windows::fs::OpenOptionsExt,
    path::PathBuf,
    time::Duration,
};
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}
fn pin(bytes: &[u8]) -> Value {
    json!({"bytes":bytes.len(),"sha256":format!("{:x}", Sha256::digest(bytes))})
}
fn pe() -> Vec<u8> {
    let mut bytes = vec![0; 70000];
    bytes[..2].copy_from_slice(b"MZ");
    bytes[60..64].copy_from_slice(&64u32.to_le_bytes());
    bytes[64..68].copy_from_slice(b"PE\0\0");
    bytes[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
    bytes[84..86].copy_from_slice(&112u16.to_le_bytes());
    bytes[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
    bytes
}
fn archive(tampered: bool, invalid_pe: bool) -> Vec<u8> {
    let cpu = if invalid_pe {
        b"MZ invalid PE".to_vec()
    } else {
        pe()
    };
    let installer = b"MZ unused installer".to_vec();
    let binding = json!({"schema":1,"version":"0.2.0","target":"x86_64-pc-windows-msvc","development_only":true,"release_ready":false,"signed_distribution":false,"complete_legal_coverage":false,
        "binaries":{"dlpad-cpu.exe":pin(&cpu),"dlpad-install.exe":pin(&installer)},"mapping":{"dlpad-cpu.exe":"payload/dlpad.exe","dlpad-install.exe":"dlpad-install.exe"},"identity":{"fixture":true}});
    let mut payload = vec![
        ("dlpad.exe", cpu),
        ("LICENSE", b"fixture license".to_vec()),
        ("THIRD-PARTY-NOTICES.txt", b"fixture notices".to_vec()),
        ("README.txt", b"fixture readme".to_vec()),
        ("binding-record.txt", serde_json::to_vec(&binding).unwrap()),
    ];
    let entries: Vec<_> = payload
        .iter()
        .map(|(name, body)| {
            let mut value = pin(body);
            value["name"] = json!(name);
            value
        })
        .collect();
    let manifest = json!({"schema":1,"version":"0.2.0","target":"x86_64-pc-windows-msvc","development_only":true,"entries":entries});
    if tampered {
        payload[0].1 = b"tampered".to_vec();
    }
    let mut members = vec![
        ("dlpad-install.exe".to_owned(), installer),
        (
            "payload/manifest.json".to_owned(),
            serde_json::to_vec(&manifest).unwrap(),
        ),
    ];
    members.extend(
        payload
            .into_iter()
            .map(|(name, body)| (format!("payload/{name}"), body)),
    );
    let mut out = Vec::new();
    let mut z = rawzip::ZipArchiveWriter::new(&mut out);
    for (name, body) in members {
        let (mut entry, config) = z
            .new_file(name.as_str())
            .unix_permissions(0o100644)
            .start()
            .unwrap();
        let mut writer = config.wrap(&mut entry);
        writer.write_all(&body).unwrap();
        let (_, descriptor) = writer.finish().unwrap();
        entry.finish(descriptor).unwrap();
    }
    z.finish().unwrap();
    out
}
fn candidate(bytes: &[u8], version: &str) -> VerifiedCandidate {
    let key = Ed25519KeyPair::from_seed_unchecked(&[11; 32]).unwrap();
    let public: [u8; 32] = key.public_key().as_ref().try_into().unwrap();
    let data=serde_json::to_vec(&json!({"schema":1,"domain":"dlpad.app-update.v1","key_id":"fixture","sequence":2,"version":version,"channel":"stable","target":"x86_64-pc-windows-msvc","repository":"joongi007/dlpad","asset_name":format!("dlpad-{version}-windows-x86_64-installer.zip"),"asset_url":format!("https://github.com/joongi007/dlpad/releases/download/v{version}/dlpad-{version}-windows-x86_64-installer.zip"),"asset_bytes":bytes.len(),"asset_sha256":format!("{:x}",Sha256::digest(bytes)),"issued_at_unix":100,"expires_at_unix":200})).unwrap();
    let sig = key
        .sign(&data)
        .as_ref()
        .iter()
        .map(|v| format!("{v:02x}"))
        .collect::<String>();
    verify_metadata(
        &data,
        "fixture",
        &sig,
        &[TrustedKey {
            id: "fixture",
            public_key: &public,
        }],
        &Policy {
            installed_version: "0.0.0",
            installed_sequence: 1,
            channel: Channel::Stable,
            now_unix: 150,
        },
    )
    .unwrap()
}
struct Fixture {
    temp: tempfile::TempDir,
    parent: File,
    package: StagedPackage,
    bytes: Vec<u8>,
    paths: [PathBuf; 3],
}
impl Fixture {
    fn new(tampered: bool, invalid_pe: bool) -> Self {
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
        };
        let temp = tempfile::tempdir().unwrap();
        let path = std::fs::canonicalize(temp.path()).unwrap();
        let parent = OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&path)
            .unwrap();
        let bytes = archive(tampered, invalid_pe);
        let c = candidate(&bytes, "0.2.0");
        let package = stage_verified_package(
            &c,
            &mut Cursor::new(&bytes),
            StageRequest {
                parent: &parent,
                parent_path: &path,
                nonce: "prepare",
            },
            &AtomicBool::new(false),
            deadline(),
        )
        .unwrap();
        for name in ["old-app", "media", "external-tool", "profile"] {
            std::fs::write(path.join(name), format!("preserve {name}")).unwrap();
        }
        Self {
            temp,
            parent,
            package,
            bytes,
            paths: [
                path.join("extract"),
                path.join("new-bundle"),
                path.join("new-journal"),
            ],
        }
    }
    fn assert_preserved(&self) {
        let root = std::fs::canonicalize(self.temp.path()).unwrap();
        for name in ["old-app", "media", "external-tool", "profile"] {
            assert_eq!(
                std::fs::read(root.join(name)).unwrap(),
                format!("preserve {name}").as_bytes()
            );
        }
        self.package.verify_current().unwrap();
        assert_eq!(std::fs::read(self.package.path()).unwrap(), self.bytes);
        assert!(self.parent.metadata().unwrap().is_dir());
    }
    fn prepare(
        &mut self,
        c: &AtomicBool,
        d: Instant,
    ) -> Result<PreparedUpdateBundle, UpdatePreparationFailure> {
        prepare_staged_update(
            &mut self.package,
            &UpdatePreparationRequest {
                extraction_root: &self.paths[0],
                bundle_root: &self.paths[1],
                journal_root: &self.paths[2],
            },
            c,
            d,
        )
    }
}
#[test]
fn original_signed_zip_prepares_separate_bundle_and_preserves_old_app_profile_media() {
    let mut f = Fixture::new(false, false);
    let c = AtomicBool::new(false);
    let prepared = f.prepare(&c, deadline()).unwrap();
    prepared.verify_current(&c, deadline()).unwrap();
    assert_eq!(prepared.observation().version(), "0.2.0");
    assert_eq!(prepared.observation().sequence(), 2);
    assert_eq!(std::fs::read_dir(&f.paths[0]).unwrap().count(), 7);
    assert_eq!(
        prepared
            .installation()
            .bundle()
            .registration()
            .record()
            .prepared
            .entries
            .len(),
        5
    );
    f.assert_preserved();
    drop(prepared);
    assert!(f.paths.iter().all(|p| p.exists()));
    f.assert_preserved();
}
#[test]
fn cancel_and_expired_deadline_create_no_output() {
    for expired in [false, true] {
        let mut f = Fixture::new(false, false);
        let e = f
            .prepare(
                &AtomicBool::new(!expired),
                if expired { Instant::now() } else { deadline() },
            )
            .err()
            .unwrap();
        assert_eq!(
            e.kind(),
            UpdatePreparationError::Io(if expired {
                io::ErrorKind::TimedOut
            } else {
                io::ErrorKind::Interrupted
            })
        );
        assert!(!e.mutation_may_have_occurred());
        assert!(f.paths.iter().all(|p| !p.exists()));
        f.assert_preserved();
    }
}
#[test]
fn valid_outer_signature_with_tampered_inner_manifest_creates_no_output() {
    let mut f = Fixture::new(true, false);
    let e = f
        .prepare(&AtomicBool::new(false), deadline())
        .err()
        .unwrap();
    assert!(matches!(e.kind(), UpdatePreparationError::Bundle(_)));
    assert!(!e.mutation_may_have_occurred());
    assert!(f.paths.iter().all(|p| !p.exists()));
    f.assert_preserved();
}
#[test]
fn every_existing_destination_preserves_foreign_sentinel_and_previous_installation() {
    for index in 0..3 {
        let mut f = Fixture::new(false, false);
        std::fs::create_dir(&f.paths[index]).unwrap();
        std::fs::write(f.paths[index].join("foreign"), b"preserve foreign").unwrap();
        let e = f
            .prepare(&AtomicBool::new(false), deadline())
            .err()
            .unwrap();
        assert!(e.mutation_may_have_occurred());
        assert_eq!(
            std::fs::read(f.paths[index].join("foreign")).unwrap(),
            b"preserve foreign"
        );
        assert_eq!(std::fs::read_dir(&f.paths[index]).unwrap().count(), 1);
        f.assert_preserved();
    }
}
#[test]
fn equal_or_nested_targets_are_rejected_before_extraction() {
    for nested in [false, true] {
        let mut f = Fixture::new(false, false);
        f.paths[1] = if nested {
            f.paths[0].join("nested")
        } else {
            f.paths[0].clone()
        };
        let e = f
            .prepare(&AtomicBool::new(false), deadline())
            .err()
            .unwrap();
        assert!(!e.mutation_may_have_occurred());
        assert!(f.paths.iter().all(|p| !p.exists()));
        f.assert_preserved();
    }
}
#[test]
fn valid_bundle_bytes_with_invalid_pe_preserve_extracted_capabilities_on_failure() {
    let mut f = Fixture::new(false, true);
    let e = f
        .prepare(&AtomicBool::new(false), deadline())
        .err()
        .unwrap();
    assert!(e.mutation_may_have_occurred() && e.extraction_capability_present());
    assert_eq!(e.retained_source_files(), 7);
    assert!(!e.installation_capabilities_present());
    assert!(!f.paths[1].exists() && !f.paths[2].exists());
    drop(e);
    assert_eq!(std::fs::read_dir(&f.paths[0]).unwrap().count(), 7);
    f.assert_preserved();
}
#[test]
fn prepared_sources_are_readonly_and_foreign_namespace_is_detected_without_touching_old_app() {
    let mut f = Fixture::new(false, false);
    let c = AtomicBool::new(false);
    let mut prepared = f.prepare(&c, deadline()).unwrap();
    assert!(
        prepared
            .files
            .iter_mut()
            .find(|f| f.name() == "README.txt")
            .unwrap()
            .file_mut()
            .write_all(b"changed")
            .is_err()
    );
    prepared.verify_current(&c, deadline()).unwrap();
    std::fs::write(f.paths[0].join("foreign"), b"preserve unexpected").unwrap();
    assert!(prepared.verify_current(&c, deadline()).is_err());
    prepared
        .installation()
        .verify_current(&c, deadline())
        .unwrap();
    f.assert_preserved();
}

/// Explicit local real-package compatibility test. The seed above is only a test key;
/// passing this fixture never certifies a production signature, network transfer or launch.
#[test]
#[ignore = "requires explicit local unsigned preview ZIP, its SHA and version; test trust only"]
fn explicit_hosted_zip_prepares_new_bundle_with_test_key_only() {
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    };
    let path = PathBuf::from(
        std::env::var_os("DLPAD_TEST_UPDATE_ZIP").expect("explicit owned fixture ZIP"),
    );
    let expected = std::env::var("DLPAD_TEST_UPDATE_SHA256").expect("explicit original SHA256");
    let version = std::env::var("DLPAD_TEST_UPDATE_VERSION").expect("explicit fixture version");
    let file = File::open(path).unwrap();
    assert!(file.metadata().unwrap().len() <= 32 * 1024 * 1024);
    let mut bytes = Vec::new();
    file.take(32 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .unwrap();
    assert!(bytes.len() <= 32 * 1024 * 1024);
    assert_eq!(format!("{:x}", Sha256::digest(&bytes)), expected);
    let temp = tempfile::tempdir().unwrap();
    let root = std::fs::canonicalize(temp.path()).unwrap();
    let parent = OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(&root)
        .unwrap();
    let candidate = candidate(&bytes, &version);
    let cancel = AtomicBool::new(false);
    let mut package = stage_verified_package(
        &candidate,
        &mut Cursor::new(&bytes),
        StageRequest {
            parent: &parent,
            parent_path: &root,
            nonce: "hosted-fixture",
        },
        &cancel,
        deadline(),
    )
    .unwrap();
    let zip_identity = package.identity().clone();
    let prepared = prepare_staged_update(
        &mut package,
        &UpdatePreparationRequest {
            extraction_root: &root.join("extracted"),
            bundle_root: &root.join("bundle"),
            journal_root: &root.join("journal"),
        },
        &cancel,
        deadline(),
    )
    .unwrap();
    prepared.verify_current(&cancel, deadline()).unwrap();
    assert_eq!(prepared.observation().version(), version);
    assert_eq!(
        prepared
            .installation()
            .bundle()
            .registration()
            .record()
            .prepared
            .entries
            .len(),
        5
    );
    package.verify_current().unwrap();
    assert_eq!(*package.identity(), zip_identity);
    assert_eq!(std::fs::read(package.path()).unwrap(), bytes);
}
