//! Development install evidence only. Never a deletion/replacement/resume capability.
use super::{
    manifest::BundleManifest,
    windows_caps::{CreatedFile, FreshDestination, FreshRoot, ReadFile, ReadRoot},
};
use crate::file_identity::NativeFileIdentity;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CompareStringOrdinal(a: *const u16, al: i32, b: *const u16, bl: i32, ignore: i32) -> i32;
}
fn contains(a: &Path, b: &Path) -> io::Result<bool> {
    use std::os::windows::ffi::OsStrExt;
    let a: Vec<_> = a.as_os_str().encode_wide().collect();
    let b: Vec<_> = b.as_os_str().encode_wide().collect();
    if a.len() > b.len() {
        return Ok(false);
    }
    // SAFETY: bounded valid UTF16 slices retained for the synchronous native comparison.
    let result =
        unsafe { CompareStringOrdinal(a.as_ptr(), a.len() as i32, b.as_ptr(), a.len() as i32, 1) };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(result == 2 && (a.len() == b.len() || a.last() == Some(&92) || b.get(a.len()) == Some(&92)))
}
fn same(a: &Path, b: &Path) -> io::Result<bool> {
    Ok(contains(a, b)? && contains(b, a)?)
}
enum TargetObservation {
    Existing(ReadRoot, PathBuf),
    Fresh(FreshDestination),
}
impl TargetObservation {
    fn observe(p: &Path) -> io::Result<Self> {
        match ReadRoot::open_existing(p) {
            Ok(r) => {
                let path = r.original_final_path()?;
                Ok(Self::Existing(r, path))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                Ok(Self::Fresh(FreshDestination::observe(p)?))
            }
            Err(e) => Err(e),
        }
    }
    fn final_path(&self) -> &Path {
        match self {
            Self::Existing(_, p) => p,
            Self::Fresh(d) => d.final_path(),
        }
    }
    fn verify(&self) -> io::Result<()> {
        match self {
            Self::Existing(r, _) => r.verify(),
            Self::Fresh(d) => d.verify(),
        }
    }
}

const LIMIT: usize = 65536;
const NAMES: [&str; 5] = [
    "intent.json",
    "prepared.json",
    "registered.json",
    "receipt.json",
    "active.json",
];
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallIntent {
    pub schema: u32,
    pub domain: String,
    pub development_only: bool,
    pub journal_root_identity: NativeFileIdentity,
    pub intent_identity: NativeFileIdentity,
    pub bundle_target: PathBuf,
    pub manifest_sha256: String,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Phase {
    Prepared,
    Registered,
    Receipt,
    Active,
}
impl Phase {
    fn index(&self) -> usize {
        match self {
            Self::Prepared => 1,
            Self::Registered => 2,
            Self::Receipt => 3,
            Self::Active => 4,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseEvidence {
    pub schema: u32,
    pub domain: String,
    pub phase: Phase,
    pub journal_root_identity: NativeFileIdentity,
    pub file_identity: NativeFileIdentity,
    pub previous_sha256: String,
    /// Caller-supplied fingerprint: syntax/chain checked, NOT proof of prior verified stage.
    pub stage_record_sha256: String,
}
fn bad() -> io::Error {
    io::ErrorKind::InvalidData.into()
}
fn check(c: &AtomicBool, d: Instant) -> io::Result<()> {
    if c.load(Ordering::Acquire) {
        return Err(io::ErrorKind::Interrupted.into());
    }
    if Instant::now() >= d {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(())
}
fn digest(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn hash_valid(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn native(id: &NativeFileIdentity) -> bool {
    matches!(id,NativeFileIdentity::Windows{volume_serial,file_id} if *volume_serial!=0&&*file_id!=[0;16])
}
fn encode<T: Serialize>(v: &T) -> io::Result<Vec<u8>> {
    let b = serde_json::to_vec(v)?;
    if b.len() > LIMIT {
        return Err(bad());
    }
    Ok(b)
}
impl InstallIntent {
    fn validate(&self) -> io::Result<()> {
        if self.schema != 1
            || self.domain != "dlpad-install-intent-v1"
            || !self.development_only
            || !native(&self.journal_root_identity)
            || !native(&self.intent_identity)
            || self.intent_identity == self.journal_root_identity
            || !self.bundle_target.is_absolute()
            || !hash_valid(&self.manifest_sha256)
        {
            return Err(bad());
        }
        Ok(())
    }
}
fn namespace(path: &Path, count: usize, c: &AtomicBool, d: Instant) -> io::Result<()> {
    check(c, d)?;
    let mut seen = [false; 5];
    let mut total = 0;
    for entry in std::fs::read_dir(path)? {
        check(c, d)?;
        let e = entry?;
        let name = e.file_name();
        let Some(i) = NAMES[..count]
            .iter()
            .position(|n| name == std::ffi::OsStr::new(n))
        else {
            return Err(bad());
        };
        if seen[i] || !e.file_type()?.is_file() || e.file_type()?.is_symlink() {
            return Err(bad());
        }
        seen[i] = true;
        total += 1;
    }
    if total != count {
        return Err(bad());
    }
    check(c, d)
}
/// Opaque retained original caps even if create/write/sync fails. Drop deletes nothing.
pub struct JournalFailure {
    error: io::Error,
    state: Box<WriterState>,
}
impl std::fmt::Debug for JournalFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("JournalFailure(<redacted>)")
    }
}
impl JournalFailure {
    pub fn kind(&self) -> io::ErrorKind {
        self.error.kind()
    }
    pub fn mutation_may_have_occurred(&self) -> bool {
        self.state.mutation
    }
    pub fn root_capability_acquired(&self) -> bool {
        self.state.root.is_some()
    }
    pub fn retained_file_cap_count(&self) -> usize {
        self.state.files.len()
    }
}
struct WriterState {
    root: Option<FreshRoot>,
    files: Vec<CreatedFile>,
    canonical: Vec<Vec<u8>>,
    mutation: bool,
}
pub struct FreshInstallJournal {
    state: WriterState,
}
fn fail(error: io::Error, state: WriterState) -> JournalFailure {
    JournalFailure {
        error,
        state: Box::new(state),
    }
}
fn verify_writer(state: &WriterState, c: &AtomicBool, d: Instant) -> io::Result<()> {
    use std::os::windows::fs::FileExt;
    check(c, d)?;
    let root = state.root.as_ref().ok_or_else(bad)?;
    if state.files.is_empty() || state.files.len() != state.canonical.len() || state.files.len() > 5
    {
        return Err(bad());
    }
    root.verify()?;
    namespace(root.path(), state.files.len(), c, d)?;
    for (file, expected) in state.files.iter().zip(&state.canonical) {
        check(c, d)?;
        file.verify(root)?;
        if expected.len() > LIMIT || file.file().metadata()?.len() != expected.len() as u64 {
            return Err(bad());
        }
        let mut bytes = vec![0; expected.len()];
        let mut offset = 0;
        while offset < bytes.len() {
            check(c, d)?;
            let end = (offset + 4096).min(bytes.len());
            let count = file
                .file()
                .seek_read(&mut bytes[offset..end], offset as u64)?;
            check(c, d)?;
            if count == 0 {
                return Err(bad());
            }
            offset += count;
        }
        if bytes != *expected || file.file().metadata()?.len() != expected.len() as u64 {
            return Err(bad());
        }
        file.verify(root)?;
    }
    root.verify()?;
    namespace(root.path(), state.files.len(), c, d)
}
impl FreshInstallJournal {
    pub(super) fn verify_current(&self, c: &AtomicBool, d: Instant) -> io::Result<()> {
        verify_writer(&self.state, c, d)
    }
    pub fn create(
        manifest: &BundleManifest,
        bundle_target: &Path,
        journal_target: &Path,
        c: &AtomicBool,
        d: Instant,
    ) -> Result<Self, JournalFailure> {
        let mut state = WriterState {
            root: None,
            files: Vec::with_capacity(5),
            canonical: Vec::with_capacity(5),
            mutation: false,
        };
        let result = (|| {
            check(c, d)?;
            manifest.validate()?;
            let target = TargetObservation::observe(bundle_target)?;
            let destination = FreshDestination::observe(journal_target)?;
            if contains(target.final_path(), destination.final_path())?
                || contains(destination.final_path(), target.final_path())?
            {
                return Err(bad());
            }
            target.verify()?;
            destination.verify()?;
            check(c, d)?;
            let manifest_sha256 = digest(&encode(manifest)?);
            state.mutation = true; // Creation attempt can mutate before a capability is returned.
            state.root = Some(FreshRoot::create_new(journal_target)?);
            check(c, d)?;
            let root = state.root.as_ref().ok_or_else(bad)?;
            state.files.push(root.create_file(NAMES[0])?);
            let intent = InstallIntent {
                schema: 1,
                domain: "dlpad-install-intent-v1".into(),
                development_only: true,
                journal_root_identity: root.identity().clone(),
                intent_identity: state.files[0].identity().clone(),
                bundle_target: target.final_path().into(),
                manifest_sha256,
            };
            intent.validate()?;
            let bytes = encode(&intent)?;
            check(c, d)?;
            state.files[0].file_mut().write_all(&bytes)?;
            state.files[0].file().sync_all()?;
            target.verify()?;
            destination.verify()?;
            state.files[0].verify(root)?;
            namespace(root.path(), 1, c, d)?;
            state.canonical.push(bytes);
            verify_writer(&state, c, d)
        })();
        match result {
            Ok(()) => Ok(Self { state }),
            Err(e) => Err(fail(e, state)),
        }
    }
    /// Caller supplies evidence from its own stage verification. This method does not claim it verified a bundle.
    pub fn record_phase(
        mut self,
        phase: Phase,
        stage_record_sha256: &str,
        c: &AtomicBool,
        d: Instant,
    ) -> Result<Self, JournalFailure> {
        let result = (|| {
            check(c, d)?;
            if phase.index() != self.state.canonical.len() || !hash_valid(stage_record_sha256) {
                return Err(bad());
            }
            self.verify_current(c, d)?;
            let root = self.state.root.as_ref().ok_or_else(bad)?;
            self.state
                .files
                .push(root.create_file(NAMES[phase.index()])?);
            let i = self.state.files.len() - 1;
            let value = PhaseEvidence {
                schema: 1,
                domain: "dlpad-install-phase-v1".into(),
                phase,
                journal_root_identity: root.identity().clone(),
                file_identity: self.state.files[i].identity().clone(),
                previous_sha256: digest(self.state.canonical.last().ok_or_else(bad)?),
                stage_record_sha256: stage_record_sha256.into(),
            };
            let bytes = encode(&value)?;
            check(c, d)?;
            self.state.files[i].file_mut().write_all(&bytes)?;
            self.state.files[i].file().sync_all()?;
            self.state.files[i].verify(root)?;
            namespace(root.path(), i + 1, c, d)?;
            self.state.canonical.push(bytes);
            self.verify_current(c, d)
        })();
        match result {
            Ok(()) => Ok(self),
            Err(e) => Err(fail(e, self.state)),
        }
    }
}
/// Readonly evidence chain; even Active evidence is not proof of commit or current installation.
pub struct InstallJournalInspection {
    root: ReadRoot,
    files: Vec<ReadFile>,
    intent: InstallIntent,
    evidence: Vec<PhaseEvidence>,
    canonical: Vec<Vec<u8>>,
}
impl InstallJournalInspection {
    pub fn intent(&self) -> &InstallIntent {
        &self.intent
    }
    pub fn evidence(&self) -> &[PhaseEvidence] {
        &self.evidence
    }
    pub fn verify_current(&self, c: &AtomicBool, d: Instant) -> io::Result<()> {
        check(c, d)?;
        self.root.verify()?;
        namespace(self.root.path(), self.files.len(), c, d)?;
        for (file, expected) in self.files.iter().zip(&self.canonical) {
            check(c, d)?;
            file.verify(&self.root)?;
            if read(file.file(), c, d)? != *expected {
                return Err(bad());
            }
            check(c, d)?;
        }
        self.root.verify()?;
        namespace(self.root.path(), self.files.len(), c, d)
    }
}
fn read(file: &std::fs::File, c: &AtomicBool, d: Instant) -> io::Result<Vec<u8>> {
    check(c, d)?;
    // Independent read cursor is needed because inspection must retain the original read cap.
    use std::os::windows::io::{AsRawHandle, FromRawHandle};
    use windows_sys::Win32::Storage::FileSystem::{FILE_GENERIC_READ, FILE_SHARE_READ, ReOpenFile};
    // SAFETY: original live HANDLE, valid fixed access/share, output adopted only on success.
    let raw = unsafe { ReOpenFile(file.as_raw_handle(), FILE_GENERIC_READ, FILE_SHARE_READ, 0) };
    if raw == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful ReOpenFile returns a newly owned HANDLE.
    let mut fresh = unsafe { std::fs::File::from_raw_handle(raw) };
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 4096];
    loop {
        check(c, d)?;
        let remaining = LIMIT + 1 - bytes.len();
        let n = fresh.read(&mut buffer[..remaining.min(4096)])?;
        check(c, d)?;
        if n == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..n]);
        if bytes.len() > LIMIT {
            return Err(bad());
        }
    }
    check(c, d)?;
    Ok(bytes)
}
pub fn inspect_install_journal(
    path: &Path,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<InstallJournalInspection> {
    check(c, d)?;
    let root = ReadRoot::open_existing(path)?;
    root.verify()?;
    let mut names = Vec::new();
    for e in std::fs::read_dir(root.path())? {
        check(c, d)?;
        let name = e?.file_name();
        let Some(i) = NAMES.iter().position(|n| name == std::ffi::OsStr::new(n)) else {
            return Err(bad());
        };
        names.push(i);
        if names.len() > 5 {
            return Err(bad());
        }
    }
    names.sort_unstable();
    if names.is_empty() || names.iter().copied().ne(0..names.len()) {
        return Err(bad());
    }
    namespace(root.path(), names.len(), c, d)?;
    let mut files = Vec::new();
    let mut canonical = Vec::new();
    let mut evidence = Vec::new();
    for i in names {
        check(c, d)?;
        let file = root.open_file(NAMES[i])?;
        let bytes = read(file.file(), c, d)?;
        files.push(file);
        canonical.push(bytes);
        check(c, d)?;
    }
    check(c, d)?;
    let intent: InstallIntent = serde_json::from_slice(&canonical[0])?;
    intent.validate()?;
    check(c, d)?;
    if encode(&intent)? != canonical[0]
        || intent.journal_root_identity != *root.identity()
        || intent.intent_identity != *files[0].identity()
    {
        return Err(bad());
    }
    for i in 1..files.len() {
        check(c, d)?;
        let e: PhaseEvidence = serde_json::from_slice(&canonical[i])?;
        if e.schema != 1
            || e.domain != "dlpad-install-phase-v1"
            || e.phase.index() != i
            || e.journal_root_identity != *root.identity()
            || e.file_identity != *files[i].identity()
            || !hash_valid(&e.stage_record_sha256)
            || e.previous_sha256 != digest(&canonical[i - 1])
            || encode(&e)? != canonical[i]
        {
            return Err(bad());
        }
        evidence.push(e);
        check(c, d)?;
    }
    let value = InstallJournalInspection {
        root,
        files,
        intent,
        evidence,
        canonical,
    };
    value.verify_current(c, d)?;
    Ok(value)
}
impl InstallJournalInspection {
    /// Explicit caller-provided current bundle proof. No marker-selected path discovery.
    /// Success reports current consistency only; never crash commit or historical ownership.
    pub fn verify_active_bundle(
        &self,
        bundle: &super::UserBundleInspection,
        c: &AtomicBool,
        d: Instant,
    ) -> io::Result<()> {
        self.verify_current(c, d)?;
        bundle.verify_current(c, d)?;
        let actual_target = ReadRoot::open_existing(bundle.path())?;
        if !same(
            actual_target.original_final_path()?.as_path(),
            &self.intent.bundle_target,
        )? || self.evidence.len() != 4
        {
            return Err(bad());
        }
        let prepared = bundle.selection().original_prepared_record();
        let actual = BundleManifest {
            schema: 1,
            version: prepared.version.clone(),
            target: prepared.target.clone(),
            development_only: prepared.development_only,
            entries: prepared
                .entries
                .iter()
                .map(|e| super::manifest::BundleEntry {
                    name: e.name.clone(),
                    bytes: e.bytes,
                    sha256: e.sha256.clone(),
                })
                .collect(),
        };
        actual.validate()?;
        if digest(&encode(&actual)?) != self.intent.manifest_sha256 {
            return Err(bad());
        }
        let current = digest(&bundle.selection().record().canonical_bytes()?);
        if self.evidence[3].stage_record_sha256 != current {
            return Err(bad());
        }
        bundle.verify_current(c, d)?;
        self.verify_current(c, d)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn later() -> Instant {
        Instant::now() + std::time::Duration::from_secs(10)
    }
    fn manifest() -> BundleManifest {
        BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-msvc".into(),
            development_only: true,
            entries: vec![super::super::manifest::BundleEntry {
                name: "dlpad.exe".into(),
                bytes: 256,
                sha256: "a".repeat(64),
            }],
        }
    }
    fn bytes(p: &Path) -> Vec<u8> {
        use std::os::windows::fs::OpenOptionsExt;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(7)
            .open(p)
            .unwrap();
        let mut bytes = Vec::new();
        file.take(LIMIT as u64 + 1).read_to_end(&mut bytes).unwrap();
        assert!(bytes.len() <= LIMIT);
        bytes
    }
    #[test]
    fn cancellation_deadline_and_existing_root_preserve_foreign() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("journal");
        let bundle = dir.path().join("bundle");
        assert_eq!(
            FreshInstallJournal::create(
                &manifest(),
                &bundle,
                &target,
                &AtomicBool::new(true),
                later()
            )
            .err()
            .unwrap()
            .kind(),
            io::ErrorKind::Interrupted
        );
        assert!(!target.exists());
        assert_eq!(
            FreshInstallJournal::create(
                &manifest(),
                &bundle,
                &target,
                &AtomicBool::new(false),
                Instant::now()
            )
            .err()
            .unwrap()
            .kind(),
            io::ErrorKind::TimedOut
        );
        assert!(!target.exists());
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("foreign"), b"preserve").unwrap();
        let failure = FreshInstallJournal::create(
            &manifest(),
            &bundle,
            &target,
            &AtomicBool::new(false),
            later(),
        )
        .err()
        .unwrap();
        assert_eq!(failure.kind(), io::ErrorKind::AlreadyExists);
        assert!(!failure.root_capability_acquired());
        assert_eq!(failure.retained_file_cap_count(), 0);
        assert_eq!(std::fs::read(target.join("foreign")).unwrap(), b"preserve");
    }
    #[test]
    fn original_writer_rejects_changed_prior_bytes_without_new_phase_or_cleanup() {
        use std::io::Seek;
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("bundle");
        let path = dir.path().join("journal");
        let c = AtomicBool::new(false);
        let deadline = later();
        let mut live = FreshInstallJournal::create(&manifest(), &target, &path, &c, deadline)
            .ok()
            .unwrap();
        let mut changed = live.state.canonical[0].clone();
        changed[0] = b'[';
        live.state.files[0]
            .file_mut()
            .seek(std::io::SeekFrom::Start(0))
            .unwrap();
        live.state.files[0].file_mut().write_all(&changed).unwrap();
        live.state.files[0].file().sync_all().unwrap();
        let failure = live
            .record_phase(Phase::Prepared, &"b".repeat(64), &c, deadline)
            .err()
            .unwrap();
        assert_eq!(failure.kind(), io::ErrorKind::InvalidData);
        assert!(failure.root_capability_acquired());
        assert_eq!(failure.retained_file_cap_count(), 1);
        assert!(!path.join("prepared.json").exists());
        drop(failure);
        assert_eq!(bytes(&path.join("intent.json")), changed);
    }
    #[test]
    fn phase_skip_cancel_and_unknown_preserve_original_caps_and_intent() {
        for mode in ["skip", "cancel", "foreign"] {
            let dir = tempfile::tempdir().unwrap();
            let target = dir.path().join("journal");
            let c = AtomicBool::new(false);
            let writer = FreshInstallJournal::create(
                &manifest(),
                &dir.path().join("bundle"),
                &target,
                &c,
                later(),
            )
            .ok()
            .unwrap();
            let original = bytes(&target.join("intent.json"));
            if mode == "foreign" {
                std::fs::write(target.join("foreign"), b"preserve").unwrap();
            }
            let phase = if mode == "skip" {
                Phase::Registered
            } else {
                Phase::Prepared
            };
            let failure = writer
                .record_phase(
                    phase,
                    &"b".repeat(64),
                    &AtomicBool::new(mode == "cancel"),
                    later(),
                )
                .err()
                .unwrap();
            assert!(failure.root_capability_acquired());
            assert_eq!(failure.retained_file_cap_count(), 1);
            assert_eq!(bytes(&target.join("intent.json")), original);
            assert!(!target.join("prepared.json").exists());
            if mode == "foreign" {
                assert_eq!(std::fs::read(target.join("foreign")).unwrap(), b"preserve");
            }
        }
    }
    #[test]
    fn reordered_phase_and_cloned_native_id_fail_without_repair() {
        for mode in ["reordered", "cloned-id"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("journal");
            let c = AtomicBool::new(false);
            let writer = FreshInstallJournal::create(
                &manifest(),
                &dir.path().join("bundle"),
                &path,
                &c,
                later(),
            )
            .ok()
            .unwrap();
            let writer = writer
                .record_phase(Phase::Prepared, &"b".repeat(64), &c, later())
                .ok()
                .unwrap();
            drop(writer);
            let intent: InstallIntent =
                serde_json::from_slice(&std::fs::read(path.join("intent.json")).unwrap()).unwrap();
            let file = path.join("prepared.json");
            let mut value: PhaseEvidence =
                serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
            if mode == "reordered" {
                value.phase = Phase::Registered;
            } else {
                value.file_identity = intent.intent_identity;
            }
            let malformed = encode(&value).unwrap();
            std::fs::write(&file, &malformed).unwrap();
            assert!(inspect_install_journal(&path, &c, later()).is_err());
            assert_eq!(std::fs::read(file).unwrap(), malformed);
        }
    }
    #[test]
    fn readonly_prefix_is_incomplete_not_commit_and_torn_unknown_are_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("journal");
        let c = AtomicBool::new(false);
        let writer = FreshInstallJournal::create(
            &manifest(),
            &dir.path().join("bundle"),
            &path,
            &c,
            later(),
        )
        .ok()
        .unwrap();
        let writer = writer
            .record_phase(Phase::Prepared, &"b".repeat(64), &c, later())
            .ok()
            .unwrap();
        drop(writer);
        let reader = inspect_install_journal(&path, &c, later()).unwrap();
        assert_eq!(reader.evidence().len(), 1);
        reader.verify_current(&c, later()).unwrap();
        drop(reader);
        std::fs::write(path.join("registered.json"), b"{torn").unwrap();
        assert!(inspect_install_journal(&path, &c, later()).is_err());
        assert_eq!(
            std::fs::read(path.join("registered.json")).unwrap(),
            b"{torn"
        );
        std::fs::write(path.join("foreign"), b"preserve").unwrap();
        assert!(inspect_install_journal(&path, &c, later()).is_err());
        assert_eq!(std::fs::read(path.join("foreign")).unwrap(), b"preserve");
    }
}
#[cfg(test)]
mod actual_binding_tests {
    use super::*;
    #[test]
    fn valid_manifest_a_cannot_accept_valid_bundle_b_same_explicit_path() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("bundle");
        let mut bytes = vec![0; 70000];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[60..64].copy_from_slice(&64u32.to_le_bytes());
        bytes[64..68].copy_from_slice(b"PE\0\0");
        bytes[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        bytes[84..86].copy_from_slice(&112u16.to_le_bytes());
        bytes[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        let source = dir.path().join("source");
        std::fs::write(&source, &bytes).unwrap();
        let file = std::fs::File::open(source).unwrap();
        let b = BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-msvc".into(),
            development_only: true,
            entries: vec![super::super::manifest::BundleEntry {
                name: "dlpad.exe".into(),
                bytes: bytes.len() as u64,
                sha256: digest(&bytes),
            }],
        };
        let c = AtomicBool::new(false);
        let deadline = Instant::now() + std::time::Duration::from_secs(30);
        let fresh = super::super::create_fresh_user_bundle(
            &b,
            &[("dlpad.exe", &file)],
            &target,
            &c,
            deadline,
        )
        .ok()
        .unwrap();
        fresh.verify_current(&c, deadline).unwrap();
        drop(fresh);
        let current = super::super::inspect_user_bundle(&target, &c, deadline).unwrap();
        let active_sha = digest(&current.selection().record().canonical_bytes().unwrap());
        let mut a = b.clone();
        a.version = "0.2.0".into();
        a.validate().unwrap();
        let path = dir.path().join("journal");
        let mut journal = FreshInstallJournal::create(&a, &target, &path, &c, deadline)
            .ok()
            .unwrap();
        for phase in [Phase::Prepared, Phase::Registered, Phase::Receipt] {
            journal = journal
                .record_phase(phase, &"b".repeat(64), &c, deadline)
                .ok()
                .unwrap();
        }
        journal = journal
            .record_phase(Phase::Active, &active_sha, &c, deadline)
            .ok()
            .unwrap();
        drop(journal);
        let evidence = inspect_install_journal(&path, &c, deadline).unwrap();
        assert_eq!(
            evidence
                .verify_active_bundle(&current, &c, deadline)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
        current.verify_current(&c, deadline).unwrap();
        evidence.verify_current(&c, deadline).unwrap();
        let good_path = dir.path().join("good-journal");
        let mut good = FreshInstallJournal::create(&b, &target, &good_path, &c, deadline)
            .ok()
            .unwrap();
        for phase in [Phase::Prepared, Phase::Registered, Phase::Receipt] {
            good = good
                .record_phase(phase, &"b".repeat(64), &c, deadline)
                .ok()
                .unwrap();
        }
        good = good
            .record_phase(Phase::Active, &active_sha, &c, deadline)
            .ok()
            .unwrap();
        drop(good);
        inspect_install_journal(&good_path, &c, deadline)
            .unwrap()
            .verify_active_bundle(&current, &c, deadline)
            .unwrap();
    }
}
