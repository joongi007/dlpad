//! Default-off Windows updater integration fixture; no production trust key or network.
//! Run under the CI owned Job/private desktop wrapper, with an existing temporary parent.
#[cfg(windows)]
mod windows {

    use dlpad_engine::{
        app_update::{
            self, Channel, TrustedKey,
            activation_journal::{Journal, JournalBinding},
            bootstrap::{self, PROFILE_EPOCH, ParentEvent, UiChild, UiRequest},
            host,
            transport::{Error as TransportError, Response, Transport},
        },
        file_identity,
        instance::InstanceGuard,
        store::Store,
    };
    use ring::signature::{Ed25519KeyPair, KeyPair};
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use std::{
        error::Error,
        fs::{self, File, OpenOptions},
        io::{self, Cursor, Read, Write},
        os::windows::fs::OpenOptionsExt,
        path::{Path, PathBuf},
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };
    type Result<T> = std::result::Result<T, Box<dyn Error>>;
    const VERSION: &str = "0.1.2";
    fn limit() -> Instant {
        Instant::now() + Duration::from_secs(25)
    }
    fn directory(path: &Path) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .share_mode(7)
            .custom_flags(0x02000000 | 0x00200000)
            .open(path)
    }
    fn stamp() -> Result<u64> {
        Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
    }
    fn marker(root: &Path, name: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join(name))?;
        writeln!(file, "pid={}", std::process::id())?;
        file.sync_all()?;
        Ok(())
    }
    const CHILD_STAGES: &[&str] = &[
        "entry",
        "preflight",
        "committed",
        "ready",
        "prepare",
        "prepared",
        "restart-ack",
        "failed",
        "exit-error",
        "bootstrap-error",
    ];
    fn error_facts(error: &(dyn Error + 'static)) -> Value {
        if let Some(value) = error.downcast_ref::<io::Error>() {
            json!({"class":"io","kind":format!("{:?}", value.kind()),"os_code":value.raw_os_error()})
        } else if let Some(value) = error.downcast_ref::<app_update::profile::Error>() {
            json!({"class":"profile","kind":format!("{value:?}")})
        } else {
            // Arbitrary error strings may include paths or input. Only fixed classifications leave this fixture.
            json!({"class":"fixture"})
        }
    }
    fn child_diag(root: &Path, stage: &'static str, detail: Value) {
        if !CHILD_STAGES.contains(&stage) {
            return;
        }
        let path = root.join(format!("child-diag-{}-{stage}.json", std::process::id()));
        if let Ok(mut file) = OpenOptions::new().write(true).create_new(true).open(path) {
            let bytes =
                serde_json::to_vec(&json!({"stage":stage,"detail":detail})).unwrap_or_default();
            if bytes.len() <= 2048 {
                let _ = file.write_all(&bytes);
                let _ = file.sync_all();
            }
        }
    }
    struct OwnerDiagnostic {
        root: PathBuf,
        stage: &'static str,
        passed: bool,
        source: Option<Arc<Supply>>,
    }
    fn fixture_root_from_env() -> Option<PathBuf> {
        let root = PathBuf::from(std::env::var_os("DLPAD_HOST_FIXTURE_ROOT")?);
        let metadata = fs::symlink_metadata(&root).ok()?;
        use std::os::windows::fs::MetadataExt;
        if !root.is_absolute() || !metadata.is_dir() || metadata.file_attributes() & 0x400 != 0 {
            return None;
        }
        let sentinel = root.join("user-sentinel");
        let metadata = fs::symlink_metadata(&sentinel).ok()?;
        if !metadata.is_file() || metadata.file_attributes() & 0x400 != 0 {
            return None;
        }
        let file = File::open(sentinel).ok()?;
        let mut bytes = Vec::new();
        file.take(64).read_to_end(&mut bytes).ok()?;
        (bytes == b"synthetic profile/media/tool data untouched").then_some(root)
    }
    impl Drop for OwnerDiagnostic {
        fn drop(&mut self) {
            if self.passed {
                return;
            }
            let markers: Vec<_> = [
                "a-first",
                "a-restart-ack",
                "b-ready",
                "b-cold",
                "a-recovered",
                "expected-failure",
            ]
            .into_iter()
            .filter(|name| {
                fs::symlink_metadata(self.root.join(name))
                    .is_ok_and(|m| m.is_file() && !m.file_type().is_symlink())
            })
            .collect();
            let mut children = Vec::new();
            let mut truncated = false;
            if let Ok(entries) = fs::read_dir(&self.root) {
                for (index, entry) in entries.enumerate() {
                    if index >= 128 {
                        truncated = true;
                        break;
                    }
                    let Ok(entry) = entry else {
                        continue;
                    };
                    let name = entry.file_name();
                    let Some(name) = name.to_str() else {
                        continue;
                    };
                    if !name.starts_with("child-diag-") || !name.ends_with(".json") {
                        continue;
                    }
                    if children.len() >= 32 {
                        truncated = true;
                        break;
                    }
                    if !entry
                        .file_type()
                        .is_ok_and(|t| t.is_file() && !t.is_symlink())
                    {
                        continue;
                    }
                    let Ok(file) = File::open(entry.path()) else {
                        continue;
                    };
                    let mut bytes = Vec::new();
                    if file.take(2049).read_to_end(&mut bytes).is_err() || bytes.len() > 2048 {
                        continue;
                    }
                    let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
                        continue;
                    };
                    if value
                        .get("stage")
                        .and_then(Value::as_str)
                        .is_some_and(|v| CHILD_STAGES.contains(&v))
                    {
                        children.push(value);
                    }
                }
            }
            eprintln!(
                "FIXTURE_DIAGNOSTIC {}",
                json!({"stage":self.stage,"markers":markers,"children":children,"truncated":truncated,"requests":self.source.as_ref().map(|v|v.calls.load(Ordering::SeqCst)),"workers":self.source.as_ref().map(|v|v.factories.load(Ordering::SeqCst))})
            );
        }
    }
    fn next_event(child: &mut UiChild) -> Result<ParentEvent> {
        let deadline = limit();
        loop {
            if let Some(event) = child.try_event()? {
                return Ok(event);
            }
            if Instant::now() >= deadline {
                return Err("event deadline".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    fn child(mut endpoint: UiChild) -> Result<()> {
        let root = fs::canonicalize(PathBuf::from(
            std::env::var_os("DLPAD_HOST_FIXTURE_ROOT").ok_or("fixture root missing")?,
        ))?;
        child_diag(&root, "entry", json!(null));
        let case = std::env::var("DLPAD_HOST_FIXTURE_CASE")?;
        let is_b = std::env::current_exe()?
            .file_name()
            .is_some_and(|v| v == "dlpad.exe");
        if is_b && case == "start-fail" {
            return Err("synthetic B preflight failure".into());
        }
        let cancel = AtomicBool::new(false);
        let guard = InstanceGuard::acquire(&root)?;
        if endpoint.requires_profile_preflight() {
            app_update::profile::inspect(&root, PROFILE_EPOCH, &cancel, limit())?;
        }
        child_diag(&root, "preflight", json!(null));
        let id = file_identity::from_file(&directory(&root)?)?;
        endpoint.await_commit(&id, PROFILE_EPOCH, &cancel, limit())?;
        child_diag(&root, "committed", json!(null));
        endpoint.runtime_ready(&cancel, limit())?;
        child_diag(&root, "ready", json!(null));
        if is_b {
            marker(&root, if case == "cold" { "b-cold" } else { "b-ready" })?;
            let deadline = limit();
            loop {
                let mut committed = false;
                for entry in fs::read_dir(&root)? {
                    let entry = entry?;
                    if entry
                        .file_name()
                        .to_str()
                        .is_some_and(|v| v.starts_with("app-update-selection-"))
                        && entry.path().join("slot0001.dpapi").is_file()
                    {
                        committed = true;
                        break;
                    }
                }
                if committed {
                    break;
                }
                if Instant::now() >= deadline {
                    return Err("selection commit deadline".into());
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            // RuntimeReady is not a parent ACK: leave time for journal verification before exit.
            std::thread::sleep(Duration::from_secs(1));
            if case == "runtime-crash" {
                std::process::exit(37);
            }
            drop(guard);
            return Ok(());
        }
        if case != "replay" && root.join("a-first").exists() {
            match next_event(&mut endpoint)? {
                ParentEvent::Failed {
                    code: 13,
                    generation: 0,
                } => marker(&root, "a-recovered")?,
                event => return Err(format!("missing recovery notice: {event:?}").into()),
            };
            std::thread::sleep(Duration::from_millis(150));
            return Ok(());
        }
        if case != "replay" {
            marker(&root, "a-first")?;
        }
        endpoint.send_request(
            &UiRequest::Prepare {
                version: VERSION.into(),
                channel: Channel::Stable,
            },
            &cancel,
            limit(),
        )?;
        child_diag(&root, "prepare", json!(null));
        let mut requested_cancel = false;
        let mut requested_busy = false;
        let mut stale = false;
        loop {
            match next_event(&mut endpoint)? {
                ParentEvent::Progress { phase: 1, .. } if case == "cancel" && !requested_cancel => {
                    endpoint.send_request(&UiRequest::CancelPrepare, &cancel, limit())?;
                    requested_cancel = true;
                }
                ParentEvent::Progress { phase: 1, .. } if case == "busy" && !requested_busy => {
                    endpoint.send_request(
                        &UiRequest::Prepare {
                            version: VERSION.into(),
                            channel: Channel::Stable,
                        },
                        &cancel,
                        limit(),
                    )?;
                    requested_busy = true;
                }
                ParentEvent::Progress { .. } => (),
                ParentEvent::Prepared {
                    version,
                    generation,
                } => {
                    child_diag(&root, "prepared", json!({"generation":generation}));
                    if version != VERSION {
                        return Err("candidate version mismatch".into());
                    }
                    let generation = if case == "stale" && !stale {
                        stale = true;
                        generation + 1
                    } else {
                        generation
                    };
                    endpoint.send_request(
                        &UiRequest::RestartPrepared { generation },
                        &cancel,
                        limit(),
                    )?;
                }
                ParentEvent::RestartAccepted { .. } => {
                    child_diag(&root, "restart-ack", json!(null));
                    marker(&root, "a-restart-ack")?;
                    return Ok(());
                }
                ParentEvent::Failed {
                    code: 9,
                    generation,
                } if case == "stale" && stale => {
                    endpoint.send_request(
                        &UiRequest::RestartPrepared {
                            generation: generation - 1,
                        },
                        &cancel,
                        limit(),
                    )?;
                }
                ParentEvent::Failed { code, generation } => {
                    child_diag(
                        &root,
                        "failed",
                        json!({"code":code,"generation":generation}),
                    );
                    let expected = match case.as_str() {
                        "cancel" => 1,
                        "busy" => 7,
                        "bad-hash" | "replay" => 4,
                        _ => return Err(format!("unexpected failure {code}/{generation}").into()),
                    };
                    if code != expected || generation == 0 || (case == "busy" && generation < 2) {
                        return Err(format!("failure mismatch {code}/{generation}").into());
                    }
                    marker(&root, "expected-failure")?;
                    return Ok(());
                }
            }
        }
    }
    fn pin(bytes: &[u8]) -> Value {
        json!({"bytes":bytes.len(),"sha256":format!("{:x}",Sha256::digest(bytes))})
    }
    fn archive(cpu: &[u8]) -> Result<Vec<u8>> {
        let installer = b"MZ synthetic unused installer";
        let binding = json!({"schema":1,"version":VERSION,"target":"x86_64-pc-windows-msvc","development_only":true,"release_ready":false,"signed_distribution":false,"complete_legal_coverage":false,"binaries":{"dlpad-cpu.exe":pin(cpu),"dlpad-install.exe":pin(installer)},"mapping":{"dlpad-cpu.exe":"payload/dlpad.exe","dlpad-install.exe":"dlpad-install.exe"},"identity":{"fixture":true}});
        let payload = vec![
            ("dlpad.exe", cpu.to_vec()),
            ("LICENSE", b"fixture license".to_vec()),
            ("THIRD-PARTY-NOTICES.txt", b"fixture notices".to_vec()),
            ("README.txt", b"fixture readme".to_vec()),
            ("binding-record.txt", serde_json::to_vec(&binding)?),
        ];
        let entries: Vec<_> = payload
            .iter()
            .map(|(name, body)| {
                let mut value = pin(body);
                value["name"] = json!(name);
                value
            })
            .collect();
        let manifest = serde_json::to_vec(
            &json!({"schema":1,"version":VERSION,"target":"x86_64-pc-windows-msvc","development_only":true,"entries":entries}),
        )?;
        let mut members = vec![
            ("dlpad-install.exe".to_owned(), installer.to_vec()),
            ("payload/manifest.json".into(), manifest),
        ];
        members.extend(
            payload
                .into_iter()
                .map(|(name, body)| (format!("payload/{name}"), body)),
        );
        let mut bytes = Vec::new();
        let mut zip = rawzip::ZipArchiveWriter::new(&mut bytes);
        for (name, body) in members {
            let (mut entry, config) = zip
                .new_file(name.as_str())
                .unix_permissions(0o100644)
                .start()?;
            let mut writer = config.wrap(&mut entry);
            writer.write_all(&body)?;
            let (_, descriptor) = writer.finish()?;
            entry.finish(descriptor)?;
        }
        zip.finish()?;
        Ok(bytes)
    }
    struct Supply {
        metadata: Vec<u8>,
        signature: Vec<u8>,
        package: Vec<u8>,
        calls: AtomicUsize,
        factories: AtomicUsize,
        slow: bool,
    }
    struct Fake {
        source: Arc<Supply>,
    }
    struct Slow {
        bytes: Cursor<Vec<u8>>,
    }
    impl Read for Slow {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            std::thread::sleep(Duration::from_millis(20));
            self.bytes.read(b)
        }
    }
    impl Transport for Fake {
        fn request(
            &mut self,
            url: &str,
            _: Duration,
        ) -> std::result::Result<Response, TransportError> {
            self.source.calls.fetch_add(1, Ordering::SeqCst);
            let prefix =
                format!("https://github.com/joongi007/dlpad/releases/download/v{VERSION}/");
            if !url.starts_with(&prefix) {
                return Err(TransportError::InvalidUrl);
            }
            let leaf = &url[prefix.len()..];
            let body: Box<dyn Read> = if leaf == "update-windows-x86_64.json" {
                Box::new(Cursor::new(self.source.metadata.clone()))
            } else if leaf == "update-windows-x86_64.json.sig" {
                Box::new(Cursor::new(self.source.signature.clone()))
            } else if leaf == format!("dlpad-{VERSION}-windows-x86_64-installer.zip") {
                let data = Cursor::new(self.source.package.clone());
                if self.source.slow {
                    Box::new(Slow { bytes: data })
                } else {
                    Box::new(data)
                }
            } else {
                return Err(TransportError::InvalidUrl);
            };
            Ok(Response {
                status: 200,
                location: None,
                body,
            })
        }
    }
    fn supply(case: &str) -> Result<(Arc<Supply>, [u8; 32])> {
        let exe = fs::read(std::env::current_exe()?)?;
        if exe.len() > 64 * 1024 * 1024 {
            return Err("fixture executable budget".into());
        }
        let mut package = archive(&exe)?;
        let key = Ed25519KeyPair::from_seed_unchecked(&[117; 32]).map_err(|_| "fixture key")?;
        let public = key.public_key().as_ref().try_into()?;
        let now = stamp()?;
        let metadata = serde_json::to_vec(
            &json!({"schema":1,"domain":"dlpad.app-update.v1","key_id":"fixture","sequence":2,"version":VERSION,"channel":"stable","target":"x86_64-pc-windows-msvc","repository":"joongi007/dlpad","asset_name":format!("dlpad-{VERSION}-windows-x86_64-installer.zip"),"asset_url":format!("https://github.com/joongi007/dlpad/releases/download/v{VERSION}/dlpad-{VERSION}-windows-x86_64-installer.zip"),"asset_bytes":package.len(),"asset_sha256":format!("{:x}",Sha256::digest(&package)),"issued_at_unix":now-1,"expires_at_unix":now+3600}),
        )?;
        let signature = key
            .sign(&metadata)
            .as_ref()
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>()
            .into_bytes();
        if case == "bad-hash" {
            package[0] ^= 1;
        }
        Ok((
            Arc::new(Supply {
                metadata,
                signature,
                package,
                calls: AtomicUsize::new(0),
                factories: AtomicUsize::new(0),
                slow: case == "cancel" || case == "busy",
            }),
            public,
        ))
    }
    fn observe(root: &Path, key: &[u8; 32]) -> Result<(u32, u64, Option<u64>, Option<String>)> {
        let bootstrap = file_identity::open_regular_no_reparse(&std::env::current_exe()?)?;
        let id = file_identity::from_file(&bootstrap)?;
        let suffix = format!("{:x}", Sha256::digest(serde_json::to_vec(&(&id, "0.1.1"))?));
        let path = root.join(format!("app-update-selection-{suffix}"));
        let parent = directory(&path)?;
        let keys = [TrustedKey {
            id: "fixture",
            public_key: key,
        }];
        let journal = Journal::open(
            JournalBinding {
                root: &parent,
                root_path: &path,
                bootstrap: &bootstrap,
                bootstrap_version: "0.1.1",
                profile_epoch: 1,
            },
            &keys,
            &AtomicBool::new(false),
            limit(),
        )?;
        journal.verify_current(&AtomicBool::new(false), limit())?;
        let head = journal.head();
        Ok((
            head.generation(),
            head.highest_sequence(),
            head.failed_sequence(),
            head.active().map(|v| v.version().to_owned()),
        ))
    }
    fn run(root: &Path, source: Arc<Supply>, key: &[u8; 32]) -> Result<()> {
        let keys = [TrustedKey {
            id: "fixture",
            public_key: key,
        }];
        host::run_with_transport(root, &keys, "fixture", "0.1.1", move || {
            source.factories.fetch_add(1, Ordering::SeqCst);
            Fake {
                source: source.clone(),
            }
        })?;
        Ok(())
    }
    fn owner(case: &str, owned: &Path) -> Result<()> {
        if ![
            "success",
            "start-fail",
            "runtime-crash",
            "bad-hash",
            "cancel",
            "busy",
            "stale",
        ]
        .contains(&case)
        {
            return Err("unknown case".into());
        }
        if !owned.is_absolute() {
            return Err("owned parent must be an existing absolute directory".into());
        }
        let metadata = fs::symlink_metadata(owned)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err("owned parent must be a regular directory".into());
        }
        let owned = fs::canonicalize(owned)?;
        let root = owned.join(format!(
            "activation-{}-{}-{}",
            case,
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        fs::create_dir(&root)?;
        let root = fs::canonicalize(root)?;
        let mut diagnostic = OwnerDiagnostic {
            root: root.clone(),
            stage: "profile",
            passed: false,
            source: None,
        };
        drop(InstanceGuard::acquire(&root)?);
        fs::write(
            root.join("user-sentinel"),
            b"synthetic profile/media/tool data untouched",
        )?;
        let db = root.join("theme.sqlite");
        {
            Store::open(&db)?.set_setting("theme", "Light")?;
        }
        let before = fs::read(&db)?;
        // SAFETY: single-threaded fixture owner, before any host/child/worker is spawned.
        unsafe {
            std::env::set_var("DLPAD_HOST_FIXTURE_ROOT", &root);
            std::env::set_var("DLPAD_HOST_FIXTURE_CASE", case);
        }
        diagnostic.stage = "supply";
        let (source, key) = supply(case)?;
        diagnostic.source = Some(source.clone());
        diagnostic.stage = "host-first";
        if let Err(error) = run(&root, source.clone(), &key) {
            eprintln!(
                "OWNER_FAILURE {}",
                json!({"error":error_facts(error.as_ref()),"observation":observe(&root,&key).ok()})
            );
            return Err(error);
        }
        diagnostic.stage = "observe-first";
        let first = observe(&root, &key)?;
        match case {
            "success" | "stale" => {
                if first.1 != 2
                    || first.3.as_deref() != Some(VERSION)
                    || !root.join("b-ready").is_file()
                {
                    return Err(format!("activation observation {first:?}").into());
                }
                unsafe {
                    std::env::set_var("DLPAD_HOST_FIXTURE_CASE", "cold");
                }
                diagnostic.stage = "host-cold";
                let requests = source.calls.load(Ordering::SeqCst);
                run(&root, source.clone(), &key)?;
                if observe(&root, &key)? != first
                    || source.calls.load(Ordering::SeqCst) != requests
                    || !root.join("b-cold").is_file()
                {
                    return Err("cold selection changed/network used".into());
                }
            }
            "start-fail" | "runtime-crash" => {
                if first.1 != 2
                    || first.2 != Some(2)
                    || first.3.is_some()
                    || !root.join("a-recovered").is_file()
                {
                    return Err(format!("rollback observation {first:?}").into());
                }
                unsafe {
                    std::env::set_var("DLPAD_HOST_FIXTURE_CASE", "replay");
                }
                diagnostic.stage = "host-replay";
                run(&root, source.clone(), &key)?;
                if observe(&root, &key)? != first || !root.join("expected-failure").is_file() {
                    return Err("replay was accepted".into());
                }
            }
            _ => {
                if first.0 != 0
                    || first.1 != 0
                    || first.3.is_some()
                    || !root.join("expected-failure").is_file()
                {
                    return Err(format!("failure advanced selection {first:?}").into());
                }
            }
        }
        diagnostic.stage = "preservation";
        let expected_workers = if matches!(case, "start-fail" | "runtime-crash") {
            2
        } else {
            1
        };
        if source.factories.load(Ordering::SeqCst) != expected_workers
            || source.calls.load(Ordering::SeqCst)
                != expected_workers * 3 - usize::from(expected_workers == 2)
        {
            return Err(format!(
                "unexpected transport work: workers={} requests={}",
                source.factories.load(Ordering::SeqCst),
                source.calls.load(Ordering::SeqCst)
            )
            .into());
        }
        if fs::read(&db)? != before
            || fs::read(root.join("user-sentinel"))?
                != b"synthetic profile/media/tool data untouched"
        {
            return Err("synthetic user data changed".into());
        }
        diagnostic.stage = "cleanup";
        fs::remove_dir_all(&root)?;
        diagnostic.passed = true;
        println!(
            "CASE_PASS {}",
            json!({"case":case,"generation":first.0,"highest":first.1,"failed":first.2,"active":first.3,"requests":source.calls.load(Ordering::SeqCst),"workers":source.factories.load(Ordering::SeqCst),"preserved":true})
        );
        Ok(())
    }
    #[cfg(test)]
    mod diagnostics_tests {
        use super::*;
        #[test]
        fn error_details_do_not_include_paths_or_arbitrary_messages() {
            let error = io::Error::new(
                io::ErrorKind::PermissionDenied,
                "private-path-cookie-secret",
            );
            let facts = error_facts(&error);
            assert_eq!(facts["kind"], "PermissionDenied");
            assert!(!facts.to_string().contains("private-path"));
            let other: Box<dyn Error> = "private-path-cookie-secret".into();
            assert_eq!(error_facts(other.as_ref()), json!({"class":"fixture"}));
        }
        #[test]
        fn child_records_are_fixed_stage_bounded_and_create_only() {
            let temp = tempfile::tempdir().unwrap();
            child_diag(temp.path(), "unknown", json!(null));
            assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
            child_diag(temp.path(), "failed", json!({"code":4,"generation":1}));
            child_diag(temp.path(), "failed", json!({"code":99}));
            let file = fs::read_dir(temp.path())
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path();
            let bytes = fs::read(file).unwrap();
            assert!(bytes.len() <= 2048);
            let value: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(value["detail"]["code"], 4);
        }
    }
    pub fn entry() -> Result<()> {
        let endpoint = match bootstrap::child_entry() {
            Ok(value) => value,
            Err(error) => {
                if let Some(root) = fixture_root_from_env() {
                    child_diag(&root, "bootstrap-error", error_facts(&error));
                }
                return Err(error.into());
            }
        };
        if let Some(endpoint) = endpoint {
            let result = child(endpoint);
            if let Err(error) = &result
                && let Some(root) = fixture_root_from_env()
            {
                child_diag(&root, "exit-error", error_facts(error.as_ref()));
            }
            return result;
        }
        let mut args = std::env::args_os().skip(1);
        let mut case = None;
        let mut owned = None;
        while let Some(option) = args.next() {
            let value = args.next().ok_or("option value missing")?;
            match option.to_str() {
                Some("--case") if case.is_none() => {
                    case = Some(value.into_string().map_err(|_| "invalid case")?)
                }
                Some("--owned-parent") if owned.is_none() => owned = Some(PathBuf::from(value)),
                _ => return Err("unknown or duplicate option".into()),
            }
        }
        owner(
            &case.ok_or("--case is required")?,
            &owned.ok_or("--owned-parent is required")?,
        )
    }
}
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    windows::entry()
}
#[cfg(not(windows))]
fn main() -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Windows activation fixture is unavailable on this target",
    ))
}
