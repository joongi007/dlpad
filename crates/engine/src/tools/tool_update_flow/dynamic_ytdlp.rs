//! Explicit Windows x64 release materialization. Source notices/evidence are saved
//! BEFORE install-only execution; no active version write or self-updater.
use super::release_discovery::{self, CapturedPin, DiscoveryError, Tool};
use crate::tools::direct_download::OwnedDirectory;
use crate::tools::{InstallManifest, InstalledTool, Ownership, ToolError, ToolManager};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
const JSON_LIMIT: usize = 64 * 1024;
const NOTICE_LIMIT: usize = 1024 * 1024;
const SOURCE_BUDGET: Duration = Duration::from_secs(20);
const INSTALL_BUDGET: Duration = Duration::from_secs(120);
#[derive(Debug)]
pub enum DynamicError {
    Discovery(DiscoveryError),
    Tool(ToolError),
    InvalidRelease,
    InvalidSource,
    InvalidNotice,
    /// Artifact did not finish successfully. Prepared evidence is deliberately retained.
    InstallFailed {
        receipt_directory: PathBuf,
        error: ToolError,
    },
    /// Artifact is installed INACTIVE, but completion evidence failed. Never report full success.
    InstalledEvidenceFailed {
        installed: Box<InstalledTool>,
        receipt_directory: PathBuf,
        error: ToolError,
    },
}
impl From<DiscoveryError> for DynamicError {
    fn from(e: DiscoveryError) -> Self {
        Self::Discovery(e)
    }
}
impl From<ToolError> for DynamicError {
    fn from(e: ToolError) -> Self {
        Self::Tool(e)
    }
}
impl From<std::io::Error> for DynamicError {
    fn from(e: std::io::Error) -> Self {
        Self::Tool(ToolError::Io(e))
    }
}
#[derive(Debug, Serialize)]
struct SourcePin {
    url: String,
    bytes: u64,
    sha256: String,
}
#[derive(Deserialize)]
struct GitObject {
    #[serde(rename = "type")]
    kind: String,
    sha: String,
    url: String,
}
#[derive(Deserialize)]
struct Reference {
    #[serde(rename = "ref")]
    reference: String,
    object: GitObject,
}
#[derive(Deserialize)]
struct AnnotatedTag {
    sha: String,
    object: GitObject,
}
struct Notices {
    commit: String,
    chain: Vec<SourcePin>,
    third_party: Vec<u8>,
    license: Vec<u8>,
}
trait SourceTransport {
    fn read(
        &mut self,
        url: &str,
        limit: usize,
        cancel: &Arc<AtomicBool>,
        deadline: Instant,
    ) -> Result<Vec<u8>, DynamicError>;
}
struct OfficialSource;
impl SourceTransport for OfficialSource {
    fn read(
        &mut self,
        url: &str,
        limit: usize,
        cancel: &Arc<AtomicBool>,
        deadline: Instant,
    ) -> Result<Vec<u8>, DynamicError> {
        Ok(release_discovery::read_ytdlp_source(
            url, limit, cancel, deadline,
        )?)
    }
}
fn digest(raw: &[u8]) -> String {
    format!("{:x}", Sha256::digest(raw))
}
fn checkpoint(cancel: &Arc<AtomicBool>, deadline: Instant) -> Result<(), DynamicError> {
    if cancel.load(Ordering::Acquire) {
        return Err(ToolError::Cancelled.into());
    }
    if Instant::now() >= deadline {
        return Err(ToolError::TimedOut.into());
    }
    Ok(())
}
fn revision(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|c| c.is_ascii_hexdigit())
}
fn token(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
        && s != "."
        && s != ".."
}
fn source_pin(url: String, raw: &[u8]) -> SourcePin {
    SourcePin {
        url,
        bytes: raw.len() as u64,
        sha256: digest(raw),
    }
}
fn json<T: serde::de::DeserializeOwned>(raw: &[u8]) -> Result<T, DynamicError> {
    if raw.len() > JSON_LIMIT {
        return Err(DynamicError::InvalidSource);
    }
    serde_json::from_slice(raw).map_err(|_| DynamicError::InvalidSource)
}
fn validate_object(object: &GitObject) -> Result<(), DynamicError> {
    if !revision(&object.sha)
        || !matches!(object.kind.as_str(), "commit" | "tag")
        || object.url
            != format!(
                "https://api.github.com/repos/yt-dlp/yt-dlp/git/{}s/{}",
                object.kind, object.sha
            )
    {
        return Err(DynamicError::InvalidSource);
    }
    Ok(())
}
fn text_notice(raw: &[u8], third_party: bool) -> Result<(), DynamicError> {
    if raw.is_empty() || raw.len() > NOTICE_LIMIT {
        return Err(DynamicError::InvalidNotice);
    }
    let text = std::str::from_utf8(raw).map_err(|_| DynamicError::InvalidNotice)?;
    if text.contains('\0')
        || text.trim().is_empty()
        || (third_party && !text.starts_with("THIRD-PARTY LICENSES"))
        || (!third_party
            && !text.starts_with(
                "This is free and unencumbered software released into the public domain.",
            ))
    {
        return Err(DynamicError::InvalidNotice);
    }
    Ok(())
}
fn notices(
    tag: &str,
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
    transport: &mut dyn SourceTransport,
) -> Result<Notices, DynamicError> {
    if !token(tag) {
        return Err(DynamicError::InvalidRelease);
    }
    checkpoint(cancel, deadline)?;
    let ref_url = format!("https://api.github.com/repos/yt-dlp/yt-dlp/git/ref/tags/{tag}");
    let raw = transport.read(&ref_url, JSON_LIMIT, cancel, deadline)?;
    let reference: Reference = json(&raw)?;
    if reference.reference != format!("refs/tags/{tag}") {
        return Err(DynamicError::InvalidSource);
    }
    let mut chain = vec![source_pin(ref_url, &raw)];
    let mut object = reference.object;
    for depth in 0..=2 {
        checkpoint(cancel, deadline)?;
        validate_object(&object)?;
        if object.kind == "commit" {
            break;
        }
        if depth == 2 {
            return Err(DynamicError::InvalidSource);
        }
        let raw = transport.read(&object.url, JSON_LIMIT, cancel, deadline)?;
        let tag: AnnotatedTag = json(&raw)?;
        if tag.sha != object.sha {
            return Err(DynamicError::InvalidSource);
        }
        chain.push(source_pin(object.url, &raw));
        object = tag.object;
    }
    if object.kind != "commit" {
        return Err(DynamicError::InvalidSource);
    }
    let commit = object.sha;
    let third_url = format!(
        "https://raw.githubusercontent.com/yt-dlp/yt-dlp/{commit}/THIRD_PARTY_LICENSES.txt"
    );
    let third_party = transport.read(&third_url, NOTICE_LIMIT, cancel, deadline)?;
    text_notice(&third_party, true)?;
    chain.push(source_pin(third_url, &third_party));
    let license_url = format!("https://raw.githubusercontent.com/yt-dlp/yt-dlp/{commit}/LICENSE");
    let license = transport.read(&license_url, NOTICE_LIMIT, cancel, deadline)?;
    text_notice(&license, false)?;
    chain.push(source_pin(license_url, &license));
    checkpoint(cancel, deadline)?;
    Ok(Notices {
        commit,
        chain,
        third_party,
        license,
    })
}
struct EvidenceFile {
    name: &'static str,
    pin: File,
    bytes: u64,
    sha256: String,
}
/// Opaque, non-deserializable prepared capability. No public arbitrary manifest/URL constructor.
pub struct PreparedYtdlp {
    manifest: InstallManifest,
    bytes: u64,
    evidence: OwnedDirectory,
    files: Vec<EvidenceFile>,
    manager_root: PathBuf,
    manager_root_pin: File,
}
impl PreparedYtdlp {
    pub fn manifest(&self) -> &InstallManifest {
        &self.manifest
    }
    pub fn receipt_directory(&self) -> &Path {
        &self.evidence.path
    }
    fn verify_evidence(
        &self,
        cancel: &Arc<AtomicBool>,
        deadline: Instant,
    ) -> Result<(), DynamicError> {
        self.evidence.check()?;
        for expected in &self.files {
            checkpoint(cancel, deadline)?;
            let path = self.evidence.path.join(expected.name);
            let mut file = crate::tools::native_files::open_regular(&path)?;
            if !crate::tools::native_files::same_file(&path, &expected.pin)
                || !file.metadata()?.is_file()
            {
                return Err(ToolError::UnsafePath.into());
            }
            let mut chunk = [0u8; 4096];
            let mut hash = Sha256::new();
            let mut bytes = 0u64;
            loop {
                checkpoint(cancel, deadline)?;
                let n = file.read(&mut chunk)?;
                checkpoint(cancel, deadline)?;
                if n == 0 {
                    break;
                }
                bytes += n as u64;
                if bytes > expected.bytes {
                    return Err(ToolError::HashMismatch.into());
                }
                hash.update(&chunk[..n]);
            }
            if bytes != expected.bytes
                || format!("{:x}", hash.finalize()) != expected.sha256
                || !crate::tools::native_files::same_file(&path, &expected.pin)
            {
                return Err(ToolError::HashMismatch.into());
            }
        }
        let mut entries = 0;
        for entry in fs::read_dir(&self.evidence.path)? {
            let entry = entry?;
            checkpoint(cancel, deadline)?;
            entries += 1;
            if entries > self.files.len() {
                return Err(ToolError::UnexpectedFiles.into());
            }
            if !self.files.iter().any(|f| entry.file_name() == f.name) {
                return Err(ToolError::UnexpectedFiles.into());
            }
        }
        if entries != self.files.len() {
            return Err(ToolError::UnexpectedFiles.into());
        }
        self.evidence.check()?;
        Ok(())
    }
}
fn write_evidence(
    dir: &mut OwnedDirectory,
    name: &'static str,
    raw: &[u8],
    files: &mut Vec<EvidenceFile>,
) -> Result<(), DynamicError> {
    let mut file = dir.file(name)?;
    file.write_all(raw)?;
    file.sync_all()?;
    dir.check()?;
    if !crate::tools::native_files::same_file(&dir.path.join(name), &file) {
        return Err(ToolError::UnsafePath.into());
    }
    files.push(EvidenceFile {
        name,
        pin: file,
        bytes: raw.len() as u64,
        sha256: digest(raw),
    });
    Ok(())
}
fn persist(
    pin: CapturedPin,
    source: Notices,
    leaf: &Path,
    manager: &ToolManager,
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
) -> Result<PreparedYtdlp, DynamicError> {
    checkpoint(cancel, deadline)?;
    manager.verify_root()?;
    if !leaf.is_absolute() || leaf.starts_with(&manager.root) || manager.root.starts_with(leaf) {
        return Err(ToolError::UnsafePath.into());
    }
    crate::tools::selected_path(leaf)?;
    crate::tools::safe_components(leaf)?;
    let parent = fs::canonicalize(leaf.parent().ok_or(ToolError::UnsafePath)?)?;
    let leaf = parent.join(leaf.file_name().ok_or(ToolError::UnsafePath)?);
    if leaf.starts_with(&manager.root) || manager.root.starts_with(&leaf) {
        return Err(ToolError::UnsafePath.into());
    }
    let parent_pin = crate::tools::native_files::open_directory(&parent)?;
    let manager_root_pin = crate::tools::native_files::open_directory(&manager.root)?;
    let observed = pin.observation();
    if observed.tool != Tool::YtDlp
        || observed.asset_name != "yt-dlp.exe"
        || !token(&observed.tag)
        || observed.asset_url
            != format!(
                "https://github.com/yt-dlp/yt-dlp/releases/download/{}/yt-dlp.exe",
                observed.tag
            )
        || observed.asset_bytes == 0
        || observed.asset_bytes > crate::tools::MAX_DOWNLOAD_BYTES
    {
        return Err(DynamicError::InvalidRelease);
    }
    let bytes = observed.asset_bytes;
    let third_sha = digest(&source.third_party);
    let manifest = InstallManifest {
        tool: "yt-dlp".into(),
        version: observed.tag.clone(),
        target: "x86_64-pc-windows-msvc".into(),
        source_url: observed.asset_url.clone(),
        sha256: pin.artifact_sha256().into(),
        license: format!(
            "GPL-3.0-or-later standalone; Unlicense source; THIRD_PARTY_LICENSES.txt at commit {} SHA256 {third_sha}",
            source.commit
        ),
        ownership: Ownership::Managed,
        executable: "yt-dlp.exe".into(),
    };
    crate::tools::validate(&manifest)?;
    let receipt=serde_json::to_vec_pretty(&serde_json::json!({"schema":1,"stage":"prepared-before-install","source_pin":pin,"source_commit":source.commit,"source_evidence":source.chain,"install_manifest":manifest,"third_party_notice_sha256":third_sha,"publisher_signature_verified":false,"publisher_trust_verified":false,"active_version_changed":false})) .map_err(ToolError::Json)?;
    if receipt.len() > JSON_LIMIT {
        return Err(DynamicError::InvalidSource);
    }
    let mut evidence = OwnedDirectory::create(leaf.to_owned())?;
    let mut files = vec![];
    write_evidence(
        &mut evidence,
        "THIRD_PARTY_LICENSES.txt",
        &source.third_party,
        &mut files,
    )?;
    write_evidence(&mut evidence, "LICENSE", &source.license, &mut files)?;
    write_evidence(&mut evidence, "supply-receipt.json", &receipt, &mut files)?;
    if !crate::tools::native_files::same_directory(&parent, &parent_pin) {
        return Err(ToolError::UnsafePath.into());
    }
    let mut prepared = PreparedYtdlp {
        manifest,
        bytes,
        evidence,
        files,
        manager_root: manager.root.clone(),
        manager_root_pin,
    };
    prepared.verify_evidence(cancel, deadline)?;
    // From here retain exact source/notice evidence even if cancelled or runtime install fails.
    prepared.evidence.keep = true;
    Ok(prepared)
}
/// Worker creates a fresh absent owned leaf OUTSIDE managed installation namespace.
/// Keep its lifetime/retention cap in the caller's owned update-receipt repository.
pub fn prepare_ytdlp(
    pin: CapturedPin,
    receipt_leaf: &Path,
    manager: &ToolManager,
    cancel: &Arc<AtomicBool>,
) -> Result<PreparedYtdlp, DynamicError> {
    if pin.observation().tool != Tool::YtDlp {
        return Err(DynamicError::InvalidRelease);
    }
    let deadline = Instant::now()
        .checked_add(SOURCE_BUDGET)
        .ok_or(ToolError::TimedOut)?;
    let source = notices(
        &pin.observation().tag,
        cancel,
        deadline,
        &mut OfficialSource,
    )?;
    persist(pin, source, receipt_leaf, manager, cancel, deadline)
}
pub fn prepare_latest_ytdlp(
    receipt_leaf: &Path,
    manager: &ToolManager,
    cancel: &Arc<AtomicBool>,
) -> Result<PreparedYtdlp, DynamicError> {
    prepare_ytdlp(
        release_discovery::capture_latest(Tool::YtDlp, cancel)?,
        receipt_leaf,
        manager,
        cancel,
    )
}
pub fn prepare_version_ytdlp(
    version: &str,
    receipt_leaf: &Path,
    manager: &ToolManager,
    cancel: &Arc<AtomicBool>,
) -> Result<PreparedYtdlp, DynamicError> {
    prepare_ytdlp(
        release_discovery::capture_version(Tool::YtDlp, version, cancel)?,
        receipt_leaf,
        manager,
        cancel,
    )
}
#[derive(Debug)]
pub struct InstalledWithEvidence {
    pub installed: InstalledTool,
    pub receipt_directory: PathBuf,
}
pub fn install_prepared_ytdlp(
    mut prepared: PreparedYtdlp,
    manager: &mut ToolManager,
    cancel: &Arc<AtomicBool>,
) -> Result<InstalledWithEvidence, DynamicError> {
    if !cfg!(all(windows, target_arch = "x86_64")) {
        return Err(DynamicError::InvalidRelease);
    }
    let deadline = Instant::now()
        .checked_add(INSTALL_BUDGET)
        .ok_or(ToolError::TimedOut)?;
    if manager.root != prepared.manager_root
        || !crate::tools::native_files::same_directory(&manager.root, &prepared.manager_root_pin)
    {
        return Err(ToolError::UnsafePath.into());
    }
    prepared.verify_evidence(cancel, deadline)?;
    let installed = manager
        .download_and_install_inactive_until(
            prepared.manifest.clone(),
            prepared.bytes,
            cancel,
            deadline,
        )
        .map_err(|error| DynamicError::InstallFailed {
            receipt_directory: prepared.evidence.path.clone(),
            error,
        })?;
    let complete=serde_json::to_vec_pretty(&serde_json::json!({"schema":1,"stage":"installed-inactive-runtime-verified","manifest":installed.manifest,"bytes":installed.bytes,"active_version_changed":false,"publisher_signature_verified":false})).map_err(|e|DynamicError::InstalledEvidenceFailed{installed:Box::new(installed.clone()),receipt_directory:prepared.evidence.path.clone(),error:ToolError::Json(e)})?;
    let result = prepared.verify_evidence(cancel, deadline).and_then(|()| {
        write_evidence(
            &mut prepared.evidence,
            "installation-result.json",
            &complete,
            &mut prepared.files,
        )
    });
    if let Err(e) = result {
        return Err(DynamicError::InstalledEvidenceFailed {
            installed: Box::new(installed),
            receipt_directory: prepared.evidence.path.clone(),
            error: match e {
                DynamicError::Tool(e) => e,
                _ => ToolError::InvalidManifest,
            },
        });
    }
    Ok(InstalledWithEvidence {
        installed,
        receipt_directory: prepared.evidence.path.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    struct FakeSource {
        replies: std::collections::VecDeque<Vec<u8>>,
        requests: Vec<String>,
    }
    impl SourceTransport for FakeSource {
        fn read(
            &mut self,
            url: &str,
            _: usize,
            _: &Arc<AtomicBool>,
            _: Instant,
        ) -> Result<Vec<u8>, DynamicError> {
            self.requests.push(url.into());
            Ok(self.replies.pop_front().expect("unexpected source request"))
        }
    }
    fn fake(replies: Vec<Vec<u8>>) -> FakeSource {
        FakeSource {
            replies: replies.into(),
            requests: vec![],
        }
    }
    fn object(kind: &str, sha: &str) -> serde_json::Value {
        serde_json::json!({"type":kind,"sha":sha,"url":format!("https://api.github.com/repos/yt-dlp/yt-dlp/git/{kind}s/{sha}")})
    }
    fn reference(kind: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({"ref":"refs/tags/2026.08.19","object":object(kind,&"a".repeat(40))})).unwrap()
    }
    fn cancel() -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(false))
    }
    fn deadline() -> Instant {
        Instant::now() + SOURCE_BUDGET
    }
    #[test]
    fn lightweight_tag_notice_urls_use_exact_commit_not_mutable_tag() {
        let mut t = fake(vec![
            reference("commit"),
            b"THIRD-PARTY LICENSES\nsynthetic fixture".to_vec(),
            b"This is free and unencumbered software released into the public domain. synthetic"
                .to_vec(),
        ]);
        let n = notices("2026.08.19", &cancel(), deadline(), &mut t).unwrap();
        assert_eq!(n.commit, "a".repeat(40));
        assert_eq!(t.requests.len(), 3);
        assert!(t.requests[1].contains(&format!("/{}/THIRD_PARTY_LICENSES.txt", "a".repeat(40))));
        assert!(t.requests[2].ends_with("/LICENSE"));
        assert_eq!(n.chain[1].sha256, digest(&n.third_party));
    }
    #[test]
    fn annotated_tag_reference_and_object_binding_fail_closed() {
        let raw = serde_json::to_vec(
            &serde_json::json!({"sha":"a".repeat(40),"object":object("commit",&"b".repeat(40))}),
        )
        .unwrap();
        let mut t = fake(vec![
            reference("tag"),
            raw,
            b"THIRD-PARTY LICENSES\nfixture".to_vec(),
            b"This is free and unencumbered software released into the public domain. fixture"
                .to_vec(),
        ]);
        assert_eq!(
            notices("2026.08.19", &cancel(), deadline(), &mut t)
                .unwrap()
                .commit,
            "b".repeat(40)
        );
        for mode in ["ref", "host", "sha", "branch"] {
            let mut v: serde_json::Value = serde_json::from_slice(&reference("commit")).unwrap();
            match mode {
                "ref" => v["ref"] = "refs/tags/other".into(),
                "host" => v["object"]["url"] = "https://evil.invalid/commit".into(),
                "sha" => v["object"]["sha"] = "bad".into(),
                _ => v["object"]["type"] = "tree".into(),
            }
            let mut t = fake(vec![serde_json::to_vec(&v).unwrap()]);
            assert!(matches!(
                notices("2026.08.19", &cancel(), deadline(), &mut t),
                Err(DynamicError::InvalidSource)
            ));
            assert_eq!(t.requests.len(), 1);
        }
    }
    #[test]
    fn missing_invalid_or_oversize_notices_and_cancel_cannot_prepare_supply() {
        for raw in [
            vec![],
            b"<html>error</html>".to_vec(),
            b"THIRD-PARTY LICENSES\0".to_vec(),
            vec![b'x'; NOTICE_LIMIT + 1],
        ] {
            let mut t = fake(vec![reference("commit"), raw]);
            assert!(notices("2026.08.19", &cancel(), deadline(), &mut t).is_err());
            assert_eq!(t.requests.len(), 2);
        }
        let mut t = fake(vec![]);
        assert!(notices("../bad", &cancel(), deadline(), &mut t).is_err());
        assert!(t.requests.is_empty());
        assert!(matches!(
            notices(
                "2026.08.19",
                &Arc::new(AtomicBool::new(true)),
                deadline(),
                &mut t
            ),
            Err(DynamicError::Tool(ToolError::Cancelled))
        ));
        assert!(t.requests.is_empty());
    }
    #[test]
    fn annotated_cycles_and_incorrect_license_cannot_materialize() {
        let raw = serde_json::to_vec(
            &serde_json::json!({"sha":"a".repeat(40),"object":object("tag",&"a".repeat(40))}),
        )
        .unwrap();
        let mut t = fake(vec![reference("tag"), raw.clone(), raw]);
        assert!(matches!(
            notices("2026.08.19", &cancel(), deadline(), &mut t),
            Err(DynamicError::InvalidSource)
        ));
        assert_eq!(t.requests.len(), 3);
        let mut t = fake(vec![
            reference("commit"),
            b"THIRD-PARTY LICENSES\nfixture".to_vec(),
            b"<html>not a license</html>".to_vec(),
        ]);
        assert!(matches!(
            notices("2026.08.19", &cancel(), deadline(), &mut t),
            Err(DynamicError::InvalidNotice)
        ));
        assert_eq!(t.requests.len(), 3);
    }
    #[test]
    fn source_failure_timeout_and_cancel_after_ref_never_create_a_supply() {
        struct Failed;
        impl SourceTransport for Failed {
            fn read(
                &mut self,
                _: &str,
                _: usize,
                _: &Arc<AtomicBool>,
                _: Instant,
            ) -> Result<Vec<u8>, DynamicError> {
                Err(DiscoveryError::Http(404).into())
            }
        }
        assert!(matches!(
            notices("2026.08.19", &cancel(), deadline(), &mut Failed),
            Err(DynamicError::Discovery(DiscoveryError::Http(404)))
        ));
        let mut t = fake(vec![]);
        assert!(matches!(
            notices("2026.08.19", &cancel(), Instant::now(), &mut t),
            Err(DynamicError::Tool(ToolError::TimedOut))
        ));
        assert!(t.requests.is_empty());
        struct CancelRef {
            requests: usize,
        }
        impl SourceTransport for CancelRef {
            fn read(
                &mut self,
                _: &str,
                _: usize,
                c: &Arc<AtomicBool>,
                _: Instant,
            ) -> Result<Vec<u8>, DynamicError> {
                self.requests += 1;
                c.store(true, Ordering::Release);
                Ok(reference("commit"))
            }
        }
        let mut t = CancelRef { requests: 0 };
        assert!(matches!(
            notices("2026.08.19", &cancel(), deadline(), &mut t),
            Err(DynamicError::Tool(ToolError::Cancelled))
        ));
        assert_eq!(t.requests, 1);
    }
    fn pin() -> CapturedPin {
        struct Capture(std::collections::VecDeque<Vec<u8>>);
        impl release_discovery::Transport for Capture {
            fn open(
                &mut self,
                _: &str,
                _: Duration,
                _: bool,
                _: &Arc<AtomicBool>,
            ) -> Result<Box<dyn Read>, DiscoveryError> {
                Ok(Box::new(Cursor::new(self.0.pop_front().unwrap())))
            }
        }
        let sums = format!("{}  yt-dlp.exe\n", "c".repeat(64));
        let meta=serde_json::to_vec(&serde_json::json!({"id":1,"tag_name":"2026.08.19","draft":false,"prerelease":false,"published_at":"fixture","assets":[{"id":1,"name":"yt-dlp.exe","size":100,"browser_download_url":"https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/yt-dlp.exe"},{"id":2,"name":"SHA2-256SUMS","size":sums.len(),"browser_download_url":"https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/SHA2-256SUMS"}]})).unwrap();
        release_discovery::capture_with(
            Tool::YtDlp,
            &cancel(),
            &mut Capture(vec![meta, sums.into_bytes()].into()),
        )
        .unwrap()
    }
    fn source() -> Notices {
        notices(
            "2026.08.19",
            &cancel(),
            deadline(),
            &mut fake(vec![
                reference("commit"),
                b"THIRD-PARTY LICENSES\nfixture".to_vec(),
                b"This is free and unencumbered software released into the public domain. fixture"
                    .to_vec(),
            ]),
        )
        .unwrap()
    }
    #[test]
    fn notice_and_receipt_persist_before_execution_without_active_mutation() {
        let root = tempfile::tempdir().unwrap();
        let manager = ToolManager::new(&root.path().join("managed")).unwrap();
        manager
            .db
            .execute(
                "INSERT INTO active(tool,version,target) VALUES('yt-dlp','sentinel','synthetic')",
                [],
            )
            .unwrap();
        let prepared = persist(
            pin(),
            source(),
            &root.path().join("receipt"),
            &manager,
            &cancel(),
            deadline(),
        )
        .unwrap();
        prepared.verify_evidence(&cancel(), deadline()).unwrap();
        assert_eq!(prepared.files.len(), 3);
        let raw = fs::read(prepared.receipt_directory().join("supply-receipt.json")).unwrap();
        let record: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(record["stage"], "prepared-before-install");
        assert_eq!(record["publisher_trust_verified"], false);
        assert_eq!(record["source_commit"], "a".repeat(40));
        assert_eq!(record["install_manifest"]["sha256"], "c".repeat(64));
        assert_eq!(
            manager.active("yt-dlp").unwrap(),
            Some(("sentinel".into(), "synthetic".into()))
        );
        assert!(manager.installed("yt-dlp").unwrap().is_empty());
        let path = prepared.receipt_directory().to_owned();
        drop(prepared);
        assert!(path.join("THIRD_PARTY_LICENSES.txt").is_file());
    }
    #[test]
    fn existing_leaf_tamper_unknown_file_and_managed_overlap_are_refused() {
        let root = tempfile::tempdir().unwrap();
        let manager = ToolManager::new(&root.path().join("managed")).unwrap();
        let leaf = root.path().join("receipt");
        let prepared = persist(pin(), source(), &leaf, &manager, &cancel(), deadline()).unwrap();
        assert!(persist(pin(), source(), &leaf, &manager, &cancel(), deadline()).is_err());
        assert!(
            persist(
                pin(),
                source(),
                &manager.root.join("notice"),
                &manager,
                &cancel(),
                deadline()
            )
            .is_err()
        );
        fs::write(prepared.receipt_directory().join("LICENSE"), b"tamper").unwrap();
        assert!(prepared.verify_evidence(&cancel(), deadline()).is_err());
        let other = persist(
            pin(),
            source(),
            &root.path().join("receipt2"),
            &manager,
            &cancel(),
            deadline(),
        )
        .unwrap();
        fs::write(other.receipt_directory().join("foreign"), b"preserve").unwrap();
        assert!(other.verify_evidence(&cancel(), deadline()).is_err());
        drop(other);
        assert_eq!(
            fs::read(root.path().join("receipt2/foreign")).unwrap(),
            b"preserve"
        );
    }
    #[cfg(all(windows, target_arch = "x86_64"))]
    #[test]
    #[ignore = "official public network/tool runtime; Root-owned windows-cli-host private desktop/Job required"]
    fn official_ytdlp_dynamic_latest_inactive_with_notices() {
        #[link(name = "user32")]
        unsafe extern "system" {
            fn GetThreadDesktop(thread: u32) -> isize;
            fn GetUserObjectInformationW(
                handle: isize,
                index: i32,
                info: *mut std::ffi::c_void,
                len: u32,
                needed: *mut u32,
            ) -> i32;
        }
        let desktop = unsafe {
            GetThreadDesktop(windows_sys::Win32::System::Threading::GetCurrentThreadId())
        };
        assert_ne!(desktop, 0);
        let mut name = [0u16; 128];
        let mut needed = 0;
        assert_ne!(
            unsafe {
                GetUserObjectInformationW(desktop, 2, name.as_mut_ptr().cast(), 256, &mut needed)
            },
            0
        );
        assert!((2..=256).contains(&needed) && needed.is_multiple_of(2));
        let end = name
            .iter()
            .position(|c| *c == 0)
            .expect("bounded terminated original private desktop name");
        let actual = String::from_utf16(&name[..end]).unwrap();
        let nonce = actual
            .strip_prefix("dlpad-cli-")
            .expect("never Default/visible desktop fallback");
        assert_eq!(nonce.len(), 32);
        assert!(nonce.bytes().all(|c| c.is_ascii_hexdigit()));
        // This name check is supplementary. Root runner must independently bind the owned desktop/Job/PID.
        let root = tempfile::tempdir().unwrap();
        let mut manager = ToolManager::new(&root.path().join("managed")).unwrap();
        manager.db.execute("INSERT INTO active(tool,version,target) VALUES('yt-dlp','old-sentinel','synthetic')",[]).unwrap();
        let prepared =
            prepare_latest_ytdlp(&root.path().join("receipt"), &manager, &cancel()).unwrap();
        let expected = prepared.manifest().clone();
        let receipt = prepared.receipt_directory().to_owned();
        assert!(receipt.join("THIRD_PARTY_LICENSES.txt").is_file());
        assert!(receipt.join("LICENSE").is_file());
        assert!(!receipt.join("installation-result.json").exists());
        let complete = install_prepared_ytdlp(prepared, &mut manager, &cancel()).unwrap();
        assert_eq!(complete.installed.manifest.sha256, expected.sha256);
        assert_eq!(complete.installed.manifest.version, expected.version);
        assert!(complete.installed.executable.is_file());
        assert_eq!(
            manager.active("yt-dlp").unwrap(),
            Some(("old-sentinel".into(), "synthetic".into()))
        );
        assert!(receipt.join("installation-result.json").is_file());
        assert_eq!(complete.receipt_directory, receipt);
    }
}
