use super::*;
use std::{
    io::Cursor,
    sync::{Arc, atomic::AtomicUsize},
    time::Duration,
};
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}
#[test]
fn bounded_reads_publish_actual_latest_progress() {
    let cancel = AtomicBool::new(false);
    let mut events = Vec::new();
    let mut callback = |event| events.push(event);
    let mut reader = CountingReader {
        source: Cursor::new(vec![7; 70000]),
        cancel: &cancel,
        deadline: deadline(),
        received: 0,
        total: 70000,
        progress: &mut callback,
    };
    let mut large = vec![0; 100000];
    assert_eq!(reader.read(&mut large).unwrap(), 65536);
    assert_eq!(reader.read(&mut large).unwrap(), 4464);
    assert_eq!(reader.read(&mut large).unwrap(), 0);
    assert_eq!(
        events.iter().map(|v| v.received).collect::<Vec<_>>(),
        [65536, 70000]
    );
    assert!(
        events
            .iter()
            .all(|v| v.total == 70000 && v.phase == Phase::Downloading)
    );
}
#[test]
fn oversize_stream_never_publishes_over_limit_progress() {
    let cancel = AtomicBool::new(false);
    let mut events = Vec::new();
    let mut callback = |event| events.push(event);
    let mut reader = CountingReader {
        source: Cursor::new(vec![0; 20]),
        cancel: &cancel,
        deadline: deadline(),
        received: 0,
        total: 3,
        progress: &mut callback,
    };
    assert_eq!(
        reader.read(&mut [0; 20]).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(reader.source.position(), 4);
    assert_eq!(
        reader.read(&mut [0; 20]).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(reader.source.position(), 4);
    assert!(events.is_empty());
}
#[test]
fn cancellation_and_deadline_prevent_reads() {
    struct Tracked(Arc<AtomicUsize>);
    impl Read for Tracked {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            self.0.fetch_add(1, Ordering::Relaxed);
            Ok(0)
        }
    }
    let reads = Arc::new(AtomicUsize::new(0));
    let cancel = AtomicBool::new(true);
    let mut callback = |_| {};
    let mut reader = CountingReader {
        source: Tracked(reads.clone()),
        cancel: &cancel,
        deadline: deadline(),
        received: 0,
        total: 2,
        progress: &mut callback,
    };
    assert_eq!(
        reader.read(&mut [0; 1]).unwrap_err().kind(),
        io::ErrorKind::Interrupted
    );
    cancel.store(false, Ordering::Release);
    reader.deadline = Instant::now();
    assert_eq!(
        reader.read(&mut [0; 1]).unwrap_err().kind(),
        io::ErrorKind::TimedOut
    );
    assert_eq!(reads.load(Ordering::Relaxed), 0);
}
#[test]
fn cancel_during_read_returns_no_progress() {
    struct Cancelling(Arc<AtomicBool>);
    impl Read for Cancelling {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            self.0.store(true, Ordering::Release);
            b[0] = 1;
            Ok(1)
        }
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let mut events = Vec::new();
    let mut callback = |event| events.push(event);
    let mut reader = CountingReader {
        source: Cancelling(cancel.clone()),
        cancel: &cancel,
        deadline: deadline(),
        received: 0,
        total: 2,
        progress: &mut callback,
    };
    assert_eq!(
        reader.read(&mut [0; 1]).unwrap_err().kind(),
        io::ErrorKind::Interrupted
    );
    assert!(events.is_empty());
}
#[test]
fn progress_callback_cancellation_is_observed_before_return() {
    let cancel = AtomicBool::new(false);
    let mut callback = |_| cancel.store(true, Ordering::Release);
    let mut reader = CountingReader {
        source: Cursor::new(b"a"),
        cancel: &cancel,
        deadline: deadline(),
        received: 0,
        total: 1,
        progress: &mut callback,
    };
    assert_eq!(
        reader.read(&mut [0; 1]).unwrap_err().kind(),
        io::ErrorKind::Interrupted
    );
}
#[test]
fn io_failure_keeps_original_kind() {
    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::ErrorKind::ConnectionReset.into())
        }
    }
    let cancel = AtomicBool::new(false);
    let mut callback = |_| {};
    let mut reader = CountingReader {
        source: Broken,
        cancel: &cancel,
        deadline: deadline(),
        received: 0,
        total: 1,
        progress: &mut callback,
    };
    assert_eq!(
        reader.read(&mut [0; 1]).unwrap_err().kind(),
        io::ErrorKind::ConnectionReset
    );
}
#[cfg(windows)]
mod windows {
    use super::*;
    use crate::{
        app_install::UpdatePreparationRequest,
        app_update::{
            Channel, Policy, StageRequest, TrustedKey,
            transport::{Response, Transport},
        },
    };
    use ring::signature::{Ed25519KeyPair, KeyPair};
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use std::{
        collections::VecDeque,
        fs::{File, OpenOptions},
        io::Write,
        os::windows::fs::OpenOptionsExt,
        path::PathBuf,
    };
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
    struct Fake {
        responses: VecDeque<Vec<u8>>,
        requests: usize,
    }
    impl Transport for Fake {
        fn request(
            &mut self,
            _: &str,
            _: Duration,
        ) -> Result<Response, crate::app_update::transport::Error> {
            self.requests += 1;
            Ok(Response {
                status: 200,
                location: None,
                body: Box::new(Cursor::new(
                    self.responses.pop_front().expect("unexpected request"),
                )),
            })
        }
    }
    struct Fixture {
        temp: tempfile::TempDir,
        parent: File,
        root: PathBuf,
        paths: [PathBuf; 3],
        key: [u8; 32],
        transport: Fake,
    }
    impl Fixture {
        fn new(tampered: bool, hash_mismatch: bool) -> Self {
            use windows_sys::Win32::Storage::FileSystem::{
                FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
            };
            let temp = tempfile::tempdir().unwrap();
            let root = std::fs::canonicalize(temp.path()).unwrap();
            let parent = OpenOptions::new()
                .read(true)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&root)
                .unwrap();
            for name in ["old-app", "profile", "media", "external-tool"] {
                std::fs::write(root.join(name), format!("preserve {name}")).unwrap();
            }
            let mut bytes = archive(tampered, false);
            let pair = Ed25519KeyPair::from_seed_unchecked(&[13; 32]).unwrap();
            let key = pair.public_key().as_ref().try_into().unwrap();
            let payload=serde_json::to_vec(&json!({"schema":1,"domain":"dlpad.app-update.v1","key_id":"fixture","sequence":2,"version":"0.2.0","channel":"stable","target":"x86_64-pc-windows-msvc","repository":"joongi007/dlpad","asset_name":"dlpad-0.2.0-windows-x86_64-installer.zip","asset_url":"https://github.com/joongi007/dlpad/releases/download/v0.2.0/dlpad-0.2.0-windows-x86_64-installer.zip","asset_bytes":bytes.len(),"asset_sha256":format!("{:x}",Sha256::digest(&bytes)),"issued_at_unix":100,"expires_at_unix":200})).unwrap();
            let signature = pair
                .sign(&payload)
                .as_ref()
                .iter()
                .map(|v| format!("{v:02x}"))
                .collect::<String>()
                .into_bytes();
            if hash_mismatch {
                bytes[0] ^= 1;
            }
            let paths = [
                root.join("extraction"),
                root.join("new-bundle"),
                root.join("new-journal"),
            ];
            Self {
                temp,
                parent,
                root,
                paths,
                key,
                transport: Fake {
                    responses: vec![payload, signature, bytes].into(),
                    requests: 0,
                },
            }
        }
        fn run(
            &mut self,
            cancel: Arc<AtomicBool>,
            progress: &mut impl FnMut(Progress),
        ) -> Result<PreparedDownload, DownloadFailure> {
            let keys = [TrustedKey {
                id: "fixture",
                public_key: &self.key,
            }];
            let request = DownloadRequest {
                version: "0.2.0",
                key_id_hint: "fixture",
                keys: &keys,
                policy: Policy {
                    installed_version: "0.1.0",
                    installed_sequence: 1,
                    channel: Channel::Stable,
                    now_unix: 150,
                },
                stage: StageRequest {
                    parent: &self.parent,
                    parent_path: &self.root,
                    nonce: "download",
                },
                prepare: UpdatePreparationRequest {
                    extraction_root: &self.paths[0],
                    bundle_root: &self.paths[1],
                    journal_root: &self.paths[2],
                },
            };
            prepare_download(&mut self.transport, &request, cancel, deadline(), progress)
        }
        fn preserved(&self) {
            for name in ["old-app", "profile", "media", "external-tool"] {
                assert_eq!(
                    std::fs::read(self.temp.path().join(name)).unwrap(),
                    format!("preserve {name}").as_bytes()
                );
            }
        }
    }
    #[test]
    fn signed_download_prepares_new_bundle_preserving_a_and_profile() {
        let mut f = Fixture::new(false, false);
        let mut progress = Vec::new();
        let mut prepared = f
            .run(Arc::new(AtomicBool::new(false)), &mut |v| progress.push(v))
            .unwrap();
        prepared
            .verify_current(&AtomicBool::new(false), deadline())
            .unwrap();
        assert_eq!(prepared.candidate().version(), "0.2.0");
        assert_eq!(f.transport.requests, 3);
        assert_eq!(progress.last().unwrap().phase, Phase::Prepared);
        assert!(prepared.package().path().is_file());
        assert!(f.paths.iter().all(|v| v.is_dir()));
        f.preserved();
    }
    #[test]
    fn wrong_signed_hash_never_prepares_bundle() {
        let mut f = Fixture::new(false, true);
        let error = f
            .run(Arc::new(AtomicBool::new(false)), &mut |_| {})
            .err()
            .unwrap();
        assert!(matches!(error.kind(), FailureKind::Stage(_)));
        assert!(error.stage_failure().is_some());
        assert!(error.package().is_none());
        assert!(error.candidate().is_some());
        assert!(f.paths.iter().all(|v| !v.exists()));
        f.preserved();
    }
    #[test]
    fn invalid_inner_bundle_retains_original_staged_package() {
        let mut f = Fixture::new(true, false);
        let error = f
            .run(Arc::new(AtomicBool::new(false)), &mut |_| {})
            .err()
            .unwrap();
        assert!(matches!(error.kind(), FailureKind::Preparation(_)));
        assert!(error.preparation_failure().is_some());
        assert!(error.package().unwrap().path().is_file());
        assert!(f.paths.iter().all(|v| !v.exists()));
        f.preserved();
    }
    #[test]
    fn cancel_before_request_has_no_network_or_destination() {
        let mut f = Fixture::new(false, false);
        assert!(f.run(Arc::new(AtomicBool::new(true)), &mut |_| {}).is_err());
        assert_eq!(f.transport.requests, 0);
        assert!(f.paths.iter().all(|v| !v.exists()));
        f.preserved();
    }
    #[test]
    fn cancel_after_download_retains_package_without_preparation() {
        let mut f = Fixture::new(false, false);
        let cancel = Arc::new(AtomicBool::new(false));
        let signal = cancel.clone();
        let error = f
            .run(cancel, &mut |v| {
                if v.phase == Phase::Preparing {
                    signal.store(true, Ordering::Release)
                }
            })
            .err()
            .unwrap();
        assert!(error.package().unwrap().path().is_file());
        assert!(error.preparation_failure().is_some());
        assert!(f.paths.iter().all(|v| !v.exists()));
        f.preserved();
    }
    #[test]
    fn cancel_after_preparation_retains_bundle_capabilities() {
        let mut f = Fixture::new(false, false);
        let cancel = Arc::new(AtomicBool::new(false));
        let signal = cancel.clone();
        let error = f
            .run(cancel, &mut |v| {
                if v.phase == Phase::Prepared {
                    signal.store(true, Ordering::Release)
                }
            })
            .err()
            .unwrap();
        assert!(error.package().is_some());
        assert!(error.bundle().is_some());
        assert!(f.paths.iter().all(|v| v.is_dir()));
        f.preserved();
    }
}
