use super::*;
use serde_json::{Value, json};
use std::{io::Cursor, time::Duration};
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|v| format!("{v:02x}")).collect()
}
fn pin(bytes: &[u8]) -> Value {
    json!({"bytes":bytes.len(),"sha256":hex(&Sha256::digest(bytes))})
}
fn members() -> Vec<(String, Vec<u8>)> {
    let cpu = b"MZ synthetic CPU".to_vec();
    let installer = b"MZ synthetic installer".to_vec();
    let binding = json!({"schema":1,"version":"0.2.0","target":TARGET,"development_only":true,"release_ready":false,"signed_distribution":false,"complete_legal_coverage":false,
        "binaries":{"dlpad-cpu.exe":pin(&cpu),"dlpad-install.exe":pin(&installer)},"mapping":{"dlpad-cpu.exe":"payload/dlpad.exe","dlpad-install.exe":"dlpad-install.exe"},"identity":{"fixture":true}});
    let payload = vec![
        ("dlpad.exe", cpu),
        ("LICENSE", b"synthetic license".to_vec()),
        ("THIRD-PARTY-NOTICES.txt", b"synthetic notices".to_vec()),
        ("README.txt", b"synthetic README".to_vec()),
        ("binding-record.txt", serde_json::to_vec(&binding).unwrap()),
    ];
    let entries: Vec<_> = payload
        .iter()
        .map(|(name, body)| {
            let mut v = pin(body);
            v["name"] = json!(name);
            v
        })
        .collect();
    let manifest = json!({"schema":1,"version":"0.2.0","target":TARGET,"development_only":true,"entries":entries});
    let mut result = vec![
        ("dlpad-install.exe".to_owned(), installer),
        (
            "payload/manifest.json".to_owned(),
            serde_json::to_vec(&manifest).unwrap(),
        ),
    ];
    result.extend(
        payload
            .into_iter()
            .map(|(name, body)| (format!("payload/{name}"), body)),
    );
    result
}
fn zip(members: &[(String, Vec<u8>)], deflate: bool) -> Vec<u8> {
    use std::io::Write;
    let mut output = Vec::new();
    let mut archive = rawzip::ZipArchiveWriter::new(&mut output);
    for (name, body) in members {
        let (mut entry, config) = archive
            .new_file(name.as_str())
            .unix_permissions(0o100644)
            .compression_method(if deflate {
                CompressionMethod::DEFLATE
            } else {
                CompressionMethod::STORE
            })
            .start()
            .unwrap();
        if deflate {
            let encoder =
                flate2::write::DeflateEncoder::new(&mut entry, flate2::Compression::default());
            let mut writer = config.wrap(encoder);
            writer.write_all(body).unwrap();
            let (encoder, descriptor) = writer.finish().unwrap();
            encoder.finish().unwrap();
            entry.finish(descriptor).unwrap();
        } else {
            let mut writer = config.wrap(&mut entry);
            writer.write_all(body).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
    }
    archive.finish().unwrap();
    output
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(5)
}
fn check(bytes: &[u8]) -> Result<BundleObservation, BundleError> {
    inspect_reader(
        &mut Cursor::new(bytes),
        "0.2.0",
        2,
        bytes.len() as u64,
        &Sha256::digest(bytes).into(),
        &AtomicBool::new(false),
        deadline(),
    )
}
fn central_offsets(bytes: &[u8]) -> Vec<usize> {
    bytes
        .windows(4)
        .enumerate()
        .filter_map(|(n, b)| (b == b"PK\x01\x02").then_some(n))
        .collect()
}
fn replace_member(members: &mut [(String, Vec<u8>)], name: &str, body: Vec<u8>) {
    members.iter_mut().find(|(n, _)| n == name).unwrap().1 = body;
}
fn revise_manifest(members: &mut [(String, Vec<u8>)], f: impl FnOnce(&mut Value)) {
    let (_, body) = members
        .iter_mut()
        .find(|(n, _)| n == "payload/manifest.json")
        .unwrap();
    let mut value: Value = serde_json::from_slice(body).unwrap();
    f(&mut value);
    *body = serde_json::to_vec(&value).unwrap();
}
#[test]
fn stored_and_deflated_seven_member_previews_are_readonly_observations() {
    for deflate in [false, true] {
        let bytes = zip(&members(), deflate);
        let observed = check(&bytes).unwrap();
        assert_eq!(observed.version(), "0.2.0");
        assert_eq!(observed.sequence(), 2);
        assert_eq!(observed.package_bytes(), bytes.len() as u64);
        assert_eq!(
            observed.package_sha256().as_slice(),
            Sha256::digest(&bytes).as_slice()
        );
        assert_eq!(observed.members().len(), 7);
        assert!(
            observed
                .members()
                .iter()
                .all(|m| m.bytes() > 0 && m.sha256().len() == 32)
        );
    }
}
#[test]
fn outer_hash_and_exact_size_are_required_before_zip_interpretation() {
    let bytes = zip(&members(), false);
    let hash: [u8; 32] = Sha256::digest(&bytes).into();
    assert_eq!(
        inspect_reader(
            &mut Cursor::new(&bytes),
            "0.2.0",
            2,
            bytes.len() as u64,
            &[0; 32],
            &AtomicBool::new(false),
            deadline()
        )
        .unwrap_err(),
        BundleError::OuterHash
    );
    assert_eq!(
        inspect_reader(
            &mut Cursor::new(&bytes),
            "0.2.0",
            2,
            bytes.len() as u64 + 1,
            &hash,
            &AtomicBool::new(false),
            deadline()
        )
        .unwrap_err(),
        BundleError::OuterSize
    );
    assert_eq!(
        inspect_reader(
            &mut Cursor::new(&bytes),
            "0.2.0",
            2,
            MAX_PACKAGE_BYTES + 1,
            &hash,
            &AtomicBool::new(false),
            deadline()
        )
        .unwrap_err(),
        BundleError::OuterSize
    );
}
#[test]
fn duplicate_alias_traversal_unknown_directory_and_non_ascii_names_are_rejected() {
    for bad in [
        "payload/LICENSE",
        "payload/DLPAD.exe",
        "payload/../dlpad.exe",
        "payload\\dlpad.exe",
        "/payload/dlpad.exe",
        "payload/dlpad.exe:stream",
        "payload/dlpad.exe.",
        "payload/한글.exe",
        "payload/",
        "extra.exe",
    ] {
        let mut m = members();
        m[0].0 = bad.to_owned();
        assert!(check(&zip(&m, false)).is_err(), "{bad}");
    }
    let mut m = members();
    m.push(("unknown".to_owned(), b"data".to_vec()));
    assert!(check(&zip(&m, false)).is_err());
}
#[test]
fn manifest_version_target_entry_hash_and_duplicate_fields_are_rejected() {
    for (field, value) in [
        ("version", json!("0.3.0")),
        ("target", json!("x86_64-pc-windows-gnu")),
        ("schema", json!(2)),
        ("development_only", json!(false)),
    ] {
        let mut m = members();
        revise_manifest(&mut m, |v| v[field] = value);
        assert_eq!(check(&zip(&m, false)).unwrap_err(), BundleError::Manifest);
    }
    let mut m = members();
    replace_member(&mut m, "payload/dlpad.exe", b"MZ changed CPU!!".to_vec());
    assert_eq!(check(&zip(&m, false)).unwrap_err(), BundleError::MemberHash);
    let mut m = members();
    revise_manifest(&mut m, |v| v["entries"][0]["bytes"] = json!(1));
    assert_eq!(check(&zip(&m, false)).unwrap_err(), BundleError::MemberHash);
    let mut m = members();
    let raw = m
        .iter()
        .find(|(n, _)| n == "payload/manifest.json")
        .unwrap()
        .1
        .clone();
    let duplicate = String::from_utf8(raw)
        .unwrap()
        .replacen("{", "{\"schema\":1,", 1);
    replace_member(&mut m, "payload/manifest.json", duplicate.into_bytes());
    assert_eq!(check(&zip(&m, false)).unwrap_err(), BundleError::Manifest);
}
#[test]
fn binding_installer_cpu_and_readiness_must_match_manifest_and_all_bytes() {
    for mutate in [0, 1, 2] {
        let mut m = members();
        let (_, body) = m
            .iter_mut()
            .find(|(n, _)| n == "payload/binding-record.txt")
            .unwrap();
        let mut value: Value = serde_json::from_slice(body).unwrap();
        match mutate {
            0 => value["binaries"]["dlpad-install.exe"]["sha256"] = json!("0".repeat(64)),
            1 => value["mapping"]["dlpad-cpu.exe"] = json!("wrong.exe"),
            _ => value["release_ready"] = json!(true),
        };
        *body = serde_json::to_vec(&value).unwrap();
        let digest = hex(&Sha256::digest(&*body));
        let len = body.len();
        revise_manifest(&mut m, |v| {
            let entries = v["entries"].as_array_mut().unwrap();
            let entry = entries
                .iter_mut()
                .find(|v| v["name"] == "binding-record.txt")
                .unwrap();
            entry["sha256"] = json!(digest);
            entry["bytes"] = json!(len);
        });
        assert_eq!(check(&zip(&m, false)).unwrap_err(), BundleError::Binding);
    }
}
#[test]
fn crc_symlink_encryption_local_central_mismatch_and_unsupported_method_fail() {
    let original = zip(&members(), false);
    let first = central_offsets(&original)[0];
    for kind in 0..5 {
        let mut bytes = original.clone();
        match kind {
            0 => {
                let body = 30 + u16_at(&bytes, 26) as usize;
                bytes[body] ^= 1;
            }
            1 => bytes[first + 38..first + 42].copy_from_slice(&(0o120777u32 << 16).to_le_bytes()),
            2 => {
                bytes[first + 8] |= 1;
                bytes[6] |= 1;
            }
            3 => bytes[30] ^= b'x',
            _ => {
                bytes[first + 10..first + 12].copy_from_slice(&99u16.to_le_bytes());
                bytes[8..10].copy_from_slice(&99u16.to_le_bytes());
            }
        }
        assert!(check(&bytes).is_err(), "mutation {kind}");
    }
}
#[test]
fn size_bombs_offsets_trailing_bytes_and_archive_comments_are_rejected() {
    let original = zip(&members(), false);
    let first = central_offsets(&original)[0];
    let mut bytes = original.clone();
    bytes[first + 24..first + 28].copy_from_slice(&(MAX_EXE as u32 + 1).to_le_bytes());
    assert_eq!(check(&bytes).unwrap_err(), BundleError::Limit);
    let mut bytes = original.clone();
    bytes[first + 42..first + 46].copy_from_slice(&1u32.to_le_bytes());
    assert!(check(&bytes).is_err());
    let mut bytes = original.clone();
    bytes.push(0);
    assert!(check(&bytes).is_err());
    let mut bytes = original.clone();
    let eocd = bytes.len() - 22;
    bytes[eocd + 20..eocd + 22].copy_from_slice(&1u16.to_le_bytes());
    bytes.push(b'x');
    assert!(check(&bytes).is_err());
    for length in [0, 1, original.len() / 2, original.len() - 1] {
        assert!(check(&original[..length]).is_err());
    }
}
#[test]
fn cancellation_timeout_and_read_failure_preserve_original_input() {
    let bytes = zip(&members(), false);
    let original = bytes.clone();
    let hash: [u8; 32] = Sha256::digest(&bytes).into();
    assert_eq!(
        inspect_reader(
            &mut Cursor::new(&bytes),
            "0.2.0",
            2,
            bytes.len() as u64,
            &hash,
            &AtomicBool::new(true),
            deadline()
        )
        .unwrap_err(),
        BundleError::Cancelled
    );
    assert_eq!(
        inspect_reader(
            &mut Cursor::new(&bytes),
            "0.2.0",
            2,
            bytes.len() as u64,
            &hash,
            &AtomicBool::new(false),
            Instant::now()
        )
        .unwrap_err(),
        BundleError::TimedOut
    );
    struct Cancel<'a> {
        cursor: Cursor<&'a [u8]>,
        cancel: &'a AtomicBool,
    }
    impl Read for Cancel<'_> {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            let n = self.cursor.read(b)?;
            self.cancel.store(true, Ordering::Release);
            Ok(n)
        }
    }
    impl Seek for Cancel<'_> {
        fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
            self.cursor.seek(p)
        }
    }
    let cancel = AtomicBool::new(false);
    assert_eq!(
        inspect_reader(
            &mut Cancel {
                cursor: Cursor::new(&bytes),
                cancel: &cancel
            },
            "0.2.0",
            2,
            bytes.len() as u64,
            &hash,
            &cancel,
            deadline()
        )
        .unwrap_err(),
        BundleError::Cancelled
    );
    struct Broken(Cursor<Vec<u8>>);
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::ErrorKind::Other.into())
        }
    }
    impl Seek for Broken {
        fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
            self.0.seek(p)
        }
    }
    assert_eq!(
        inspect_reader(
            &mut Broken(Cursor::new(bytes.clone())),
            "0.2.0",
            2,
            bytes.len() as u64,
            &hash,
            &AtomicBool::new(false),
            deadline()
        )
        .unwrap_err(),
        BundleError::Io(io::ErrorKind::Other)
    );
    assert_eq!(bytes, original);
}
#[test]
fn change_after_zip_parse_is_rejected_by_final_outer_hash() {
    struct Change {
        cursor: Cursor<Vec<u8>>,
        ends: usize,
    }
    impl Read for Change {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            self.cursor.read(b)
        }
    }
    impl Seek for Change {
        fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
            if matches!(p, SeekFrom::End(0)) {
                self.ends += 1;
                if self.ends == 4 {
                    self.cursor.get_mut()[35] ^= 1;
                }
            }
            self.cursor.seek(p)
        }
    }
    let bytes = zip(&members(), false);
    let hash: [u8; 32] = Sha256::digest(&bytes).into();
    let length = bytes.len() as u64;
    assert_eq!(
        inspect_reader(
            &mut Change {
                cursor: Cursor::new(bytes),
                ends: 0
            },
            "0.2.0",
            2,
            length,
            &hash,
            &AtomicBool::new(false),
            deadline()
        )
        .unwrap_err(),
        BundleError::OuterHash
    );
}

#[test]
fn deflate_output_exceeding_its_declared_small_size_is_stopped() {
    let mut m = members();
    let expected = m[0].1.len() as u32;
    m[0].1 = vec![b'x'; 1024 * 1024];
    let mut bytes = zip(&m, true);
    let central = central_offsets(&bytes)[0];
    let entry = rawzip::ZipArchive::from_slice(&bytes).unwrap();
    let record = entry.entries().next().unwrap().unwrap();
    let data_end = entry
        .get_entry(record.wayfinder())
        .unwrap()
        .compressed_data_range()
        .1 as usize;
    bytes[central + 24..central + 28].copy_from_slice(&expected.to_le_bytes());
    let prefix = if u32_at(&bytes, data_end) == 0x08074b50 {
        4
    } else {
        0
    };
    bytes[data_end + prefix + 8..data_end + prefix + 12].copy_from_slice(&expected.to_le_bytes());
    assert!(matches!(
        check(&bytes),
        Err(BundleError::Limit | BundleError::Zip)
    ));
}

#[cfg(windows)]
mod windows {
    use super::*;
    use crate::app_update::{
        Channel, Policy, StageRequest, TrustedKey, stage_verified_package, verify_metadata,
    };
    use ring::signature::{Ed25519KeyPair, KeyPair};
    use std::os::windows::fs::OpenOptionsExt;
    fn candidate(bytes: &[u8]) -> crate::app_update::VerifiedCandidate {
        let key = Ed25519KeyPair::from_seed_unchecked(&[7; 32]).unwrap();
        let public: [u8; 32] = key.public_key().as_ref().try_into().unwrap();
        let data=serde_json::to_vec(&json!({"schema":1,"domain":"dlpad.app-update.v1","key_id":"fixture","sequence":2,"version":"0.2.0","channel":"stable","target":TARGET,"repository":"joongi007/dlpad","asset_name":"dlpad-0.2.0-windows-x86_64-installer.zip","asset_url":"https://github.com/joongi007/dlpad/releases/download/v0.2.0/dlpad-0.2.0-windows-x86_64-installer.zip","asset_bytes":bytes.len(),"asset_sha256":hex(&Sha256::digest(bytes)),"issued_at_unix":100,"expires_at_unix":200})).unwrap();
        verify_metadata(
            &data,
            "fixture",
            &hex(key.sign(&data).as_ref()),
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
    #[test]
    fn real_original_stage_cap_is_checked_without_extraction_and_failure_preserves_zip() {
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
        };
        for valid in [true, false] {
            let temp = tempfile::tempdir().unwrap();
            let path = std::fs::canonicalize(temp.path()).unwrap();
            let parent = std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&path)
                .unwrap();
            let mut m = members();
            if !valid {
                replace_member(&mut m, "payload/dlpad.exe", b"bad changed CPU".to_vec());
            }
            let bytes = zip(&m, false);
            let c = candidate(&bytes);
            let mut package = stage_verified_package(
                &c,
                &mut Cursor::new(&bytes),
                StageRequest {
                    parent: &parent,
                    parent_path: &path,
                    nonce: "bundle",
                },
                &AtomicBool::new(false),
                deadline(),
            )
            .unwrap();
            let identity = package.identity().clone();
            let result = inspect_staged_bundle(&mut package, &AtomicBool::new(false), deadline());
            assert_eq!(result.is_ok(), valid);
            package.verify_current().unwrap();
            assert_eq!(*package.identity(), identity);
            assert_eq!(std::fs::read_dir(&path).unwrap().count(), 1);
            assert!(!path.join("payload").exists());
            assert_eq!(
                inspect_staged_bundle(&mut package, &AtomicBool::new(true), deadline())
                    .unwrap_err(),
                BundleError::Cancelled
            );
            drop(package);
            assert_eq!(std::fs::read(path.join(c.asset_name())).unwrap(), bytes);
        }
    }
}
