//! Explicit current-cap authorization only. Journal/source/data are never disposed.
//! Original inspector roots and immutable expectations are transferred by private seams.
#![cfg(windows)]
use super::{
    InstallJournalInspection, UserBundleInspection, inspect_install_journal, inspect_user_bundle,
    windows_caps::{DeletionFile, DeletionRoot, ReadRoot},
};
use crate::file_identity::NativeFileIdentity;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
fn check(c: &AtomicBool, d: Instant) -> io::Result<()> {
    if c.load(Ordering::Acquire) {
        return Err(io::ErrorKind::Interrupted.into());
    }
    if Instant::now() >= d {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(())
}
fn bad() -> io::Error {
    io::ErrorKind::InvalidData.into()
}
/// Proposed private seam DTO: only original roots transferred from verified inspectors.
/// Fixed order outer/registered/receipt/active. Source getters/schema are root-owned.
pub(super) struct BundleRemovalMetadata {
    pub(super) root_identities: Vec<NativeFileIdentity>,
    pub(super) files: Vec<ExpectedFile>,
}
pub(super) struct ExpectedFile {
    pub(super) root: usize,
    pub(super) name: String,
    pub(super) identity: NativeFileIdentity,
    pub(super) bytes: u64,
    pub(super) sha256: String,
}
struct PinnedFile {
    root: usize,
    file: DeletionFile,
    expected: ExpectedFile,
}
struct State {
    pending_bundle: Option<Box<UserBundleInspection>>,
    pending_read_roots: Vec<Option<ReadRoot>>,
    roots: Vec<Option<DeletionRoot>>,
    files: Vec<PinnedFile>,
    journal: Option<InstallJournalInspection>,
    path: PathBuf,
    accepted_files: usize,
    accepted_roots: usize,
    authorized: bool,
    disposition_attempted: bool,
}
pub struct ExplicitBundleRemoval {
    state: Box<State>,
}
pub struct BundleRemovalFailure {
    kind: io::ErrorKind,
    state: Box<State>,
}
impl BundleRemovalFailure {
    pub fn kind(&self) -> io::ErrorKind {
        self.kind
    }
    pub fn accepted_file_dispositions(&self) -> usize {
        self.state.accepted_files
    }
    pub fn accepted_root_dispositions(&self) -> usize {
        self.state.accepted_roots
    }
    pub fn mutation_may_have_occurred(&self) -> bool {
        self.state.disposition_attempted
    }
    pub fn retained_file_cap_count(&self) -> usize {
        self.state.files.len()
    }
    pub fn retained_root_cap_count(&self) -> usize {
        self.state.roots.iter().filter(|v| v.is_some()).count()
    }
    /// Logical bundle ReadRoot capabilities not yet converted to DeletionRoot.
    /// Excludes roots held inside DeletionRoot and the external journal root.
    /// None means the original bundled read capabilities have not been unpacked;
    /// this value is never an OS HANDLE total.
    pub fn retained_unconverted_bundle_read_root_cap_count(&self) -> Option<usize> {
        if self.state.pending_bundle.is_some() {
            None
        } else {
            Some(
                self.state
                    .pending_read_roots
                    .iter()
                    .filter(|v| v.is_some())
                    .count(),
            )
        }
    }
    pub fn original_read_bundle_retained(&self) -> bool {
        self.state.pending_bundle.is_some()
    }
    pub fn retry(self, c: &AtomicBool, d: Instant) -> Result<BundleRemovalOutcome, Self> {
        ExplicitBundleRemoval { state: self.state }.remove(c, d)
    }
}
impl std::fmt::Debug for BundleRemovalFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BundleRemovalFailure")
            .field("kind", &self.kind)
            .field("accepted_files", &self.state.accepted_files)
            .field("accepted_roots", &self.state.accepted_roots)
            .finish()
    }
}
impl std::fmt::Display for BundleRemovalFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "explicit bundle removal incomplete ({:?})", self.kind)
    }
}
impl std::error::Error for BundleRemovalFailure {}
pub struct BundleRemovalOutcome {
    pub accepted_file_dispositions: usize,
    pub accepted_root_dispositions: usize,
    pub bundle_root_disposition_accepted: bool,
    pub outer_namespace_absence_confirmed: bool,
}
fn namespace(
    root: &DeletionRoot,
    expected: BTreeSet<String>,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<()> {
    check(c, d)?;
    root.verify()?;
    let mut actual = BTreeSet::new();
    for entry in std::fs::read_dir(root.path())? {
        check(c, d)?;
        let entry = entry?;
        if actual.len() >= 67 || entry.file_type()?.is_symlink() {
            return Err(bad());
        }
        let name = entry.file_name().into_string().map_err(|_| bad())?;
        if !actual.insert(name) {
            return Err(bad());
        }
    }
    if actual != expected {
        return Err(bad());
    }
    root.verify()?;
    check(c, d)
}
fn preflight(s: &State, c: &AtomicBool, d: Instant) -> io::Result<()> {
    check(c, d)?;
    if s.roots.len() != 4 {
        return Err(bad());
    }
    s.journal.as_ref().ok_or_else(bad)?.verify_current(c, d)?;
    for (i, root) in s.roots.iter().enumerate() {
        if let Some(root) = root {
            let expected = if i == 0 {
                [(1, "registered"), (2, "receipt"), (3, "active")]
                    .into_iter()
                    .filter(|(index, _)| s.roots[*index].is_some())
                    .map(|(_, name)| name.to_owned())
                    .collect()
            } else {
                s.files
                    .iter()
                    .filter(|v| v.root == i)
                    .map(|v| v.file.name().to_owned())
                    .collect()
            };
            namespace(root, expected, c, d)?;
        }
    }
    for pin in &s.files {
        check(c, d)?;
        let root = s.roots[pin.root].as_ref().ok_or_else(bad)?;
        pin.file.verify(root)?;
        let e = &pin.expected;
        if pin.file.identity() != &e.identity || pin.file.file().metadata()?.len() != e.bytes {
            return Err(bad());
        }
        let mut input = super::payload::reading_pin(pin.file.file())?;
        let mut hash = Sha256::new();
        let mut left = e.bytes;
        let mut buffer = [0u8; 65536];
        while left > 0 {
            check(c, d)?;
            let len = input.read(&mut buffer[..left.min(65536) as usize])?;
            check(c, d)?;
            if len == 0 {
                return Err(bad());
            }
            hash.update(&buffer[..len]);
            left -= len as u64;
        }
        check(c, d)?;
        if input.read(&mut [0])? != 0 || format!("{:x}", hash.finalize()) != e.sha256 {
            return Err(bad());
        }
        check(c, d)?;
        pin.file.verify(root)?;
    }
    check(c, d)
}
pub fn authorize_explicit_user_bundle_removal(
    bundle_root: &Path,
    journal_root: &Path,
    c: &AtomicBool,
    d: Instant,
) -> Result<ExplicitBundleRemoval, BundleRemovalFailure> {
    let mut state = Box::new(State {
        pending_bundle: None,
        pending_read_roots: Vec::new(),
        roots: (0..4).map(|_| None).collect(),
        files: Vec::new(),
        journal: None,
        path: bundle_root.to_owned(),
        accepted_files: 0,
        accepted_roots: 0,
        authorized: false,
        disposition_attempted: false,
    });
    let result = (|| -> io::Result<()> {
        check(c, d)?;
        state.pending_bundle = Some(Box::new(inspect_user_bundle(bundle_root, c, d)?));
        state.journal = Some(inspect_install_journal(journal_root, c, d)?);
        let bundle = state.pending_bundle.as_ref().ok_or_else(bad)?;
        state
            .journal
            .as_ref()
            .ok_or_else(bad)?
            .verify_active_bundle(bundle, c, d)?;
        bundle.verify_current(c, d)?;
        // REQUIRED root-owned BORROW seam; failure retains original inspector in State.
        let metadata = bundle.explicit_removal_metadata(c, d)?;
        if metadata.root_identities.len() != 4
            || metadata.files.is_empty()
            || metadata.files.len() > 69
        {
            return Err(bad());
        }
        check(c, d)?;
        // Exact fixed namespaces exclude nested roots; original native IDs also reject aliases.
        if metadata.root_identities.contains(
            &state
                .journal
                .as_ref()
                .ok_or_else(bad)?
                .intent()
                .journal_root_identity,
        ) {
            return Err(bad());
        }

        // REQUIRED INFALLIBLE move-only seam, no validation/allocation/file I/O here.
        let roots = (*state
            .pending_bundle
            .take()
            .expect("borrowed original bundle"))
        .into_explicit_removal_roots();
        state.pending_read_roots = roots.into_iter().map(Some).collect();
        if state.pending_read_roots.len() != 4 {
            return Err(bad());
        }
        for (index, expected) in metadata.root_identities.iter().enumerate() {
            check(c, d)?;
            let root = state.pending_read_roots[index].take().ok_or_else(bad)?;
            match DeletionRoot::from_read_root_retaining(root, expected) {
                Ok(value) => state.roots[index] = Some(value),
                Err(failure) => {
                    let (original, error) = *failure;
                    state.pending_read_roots[index] = Some(original);
                    return Err(error);
                }
            }
        }
        let mut names = BTreeSet::new();
        for expected in metadata.files {
            check(c, d)?;
            if !(1..=3).contains(&expected.root)
                || expected.bytes > 128 * 1024 * 1024
                || expected.sha256.len() != 64
                || !expected
                    .sha256
                    .bytes()
                    .all(|v| v.is_ascii_hexdigit() && !v.is_ascii_uppercase())
                || !names.insert((expected.root, expected.name.clone()))
            {
                return Err(bad());
            }
            let root = state.roots[expected.root].as_ref().ok_or_else(bad)?;
            let file = root.open_file(&expected.name, &expected.identity)?;
            state.files.push(PinnedFile {
                root: expected.root,
                file,
                expected,
            });
        }
        if state
            .files
            .iter()
            .filter(|p| p.root == 3 && p.file.name() == "active-selection.json")
            .count()
            != 1
            || state
                .files
                .iter()
                .filter(|p| p.root == 2 && p.file.name() == "registration-receipt.json")
                .count()
                != 1
        {
            return Err(bad());
        }
        preflight(&state, c, d)
    })();
    match result {
        Ok(()) => {
            state.authorized = true;
            Ok(ExplicitBundleRemoval { state })
        }
        Err(e) => Err(BundleRemovalFailure {
            kind: e.kind(),
            state,
        }),
    }
}
impl ExplicitBundleRemoval {
    pub fn path_hint(&self) -> &Path {
        &self.state.path
    }
    pub fn remove(
        self,
        c: &AtomicBool,
        d: Instant,
    ) -> Result<BundleRemovalOutcome, BundleRemovalFailure> {
        self.remove_with_hook(c, d, &mut |_, _| Ok(()))
    }
    fn remove_with_hook(
        mut self,
        c: &AtomicBool,
        d: Instant,
        hook: &mut impl FnMut(usize, usize) -> io::Result<()>,
    ) -> Result<BundleRemovalOutcome, BundleRemovalFailure> {
        let result = (|| -> io::Result<()> {
            if !self.state.authorized {
                return Err(bad());
            }
            preflight(&self.state, c, d)?;
            // Withdraw active first. Then registered exact files, receipt, and now-empty child roots.
            for root_index in [3usize, 1, 2] {
                while let Some(index) = self.state.files.iter().position(|p| p.root == root_index) {
                    check(c, d)?;
                    preflight(&self.state, c, d)?;
                    self.state.disposition_attempted = true;
                    let pin = &self.state.files[index];
                    pin.file.mark_delete_original(
                        self.state.roots[root_index].as_ref().ok_or_else(bad)?,
                    )?;
                    self.state.accepted_files += 1;
                    drop(self.state.files.remove(index));
                    hook(self.state.accepted_files, self.state.accepted_roots)?;
                }
            }
            for index in [1usize, 2, 3, 0] {
                if self.state.roots[index].is_none() {
                    continue;
                }
                check(c, d)?;
                preflight(&self.state, c, d)?;
                self.state.disposition_attempted = true;
                self.state.roots[index]
                    .as_ref()
                    .ok_or_else(bad)?
                    .mark_delete_original()?;
                self.state.accepted_roots += 1;
                drop(self.state.roots[index].take());
                hook(self.state.accepted_files, self.state.accepted_roots)?;
            }
            check(c, d)?;
            // Observation only: do not open/adopt a replacement root or dispose by pathname.
            match std::fs::symlink_metadata(&self.state.path) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    check(c, d)?;
                    Ok(())
                }
                Err(error) => Err(error),
                Ok(_) => Err(bad()),
            }
        })();
        match result {
            Ok(()) => Ok(BundleRemovalOutcome {
                accepted_file_dispositions: self.state.accepted_files,
                accepted_root_dispositions: self.state.accepted_roots,
                bundle_root_disposition_accepted: true,
                outer_namespace_absence_confirmed: true,
            }),
            Err(e) => Err(BundleRemovalFailure {
                kind: e.kind(),
                state: self.state,
            }),
        }
    }
}

// Include inside bundle_removal module. Windows-only native tests, not executed here.
// Requires production-loop cfg(test) remove_with_hook(c,d,hook) described below.
#[cfg(test)]
mod bundle_removal_failure_tests {
    use super::*;
    #[test]
    fn foreign_after_active_withdrawal_stops_before_payload_and_preserves_retry() {
        let (temp, bundle, journal, source, before) = fixture();
        let removal = authorize_explicit_user_bundle_removal(
            &bundle,
            &journal,
            &AtomicBool::new(false),
            end(),
        )
        .unwrap();
        let foreign = bundle.join("foreign-after-active.txt");
        let failure = failed(removal.remove_with_hook(
            &AtomicBool::new(false),
            end(),
            &mut |files, roots| {
                if files == 1 && roots == 0 {
                    fs::write(&foreign, b"preserve-foreign")?;
                }
                Ok(())
            },
        ));
        assert_eq!(failure.kind(), io::ErrorKind::InvalidData);
        assert_eq!(failure.accepted_file_dispositions(), 1);
        assert_eq!(failure.retained_file_cap_count(), 5);
        assert_eq!(fs::read(&foreign).unwrap(), b"preserve-foreign");
        assert_eq!(
            fs::read(bundle.join("registered/dlpad.exe")).unwrap(),
            source
        );
        outside(&temp, &journal, &source, &before);
        // Remove only this test's own injected fixture, then retry the retained caps.
        fs::remove_file(&foreign).unwrap();
        let result = failure.retry(&AtomicBool::new(false), end()).unwrap();
        assert!(result.outer_namespace_absence_confirmed);
        outside(&temp, &journal, &source, &before);
    }
    #[test]
    fn cancelled_after_all_root_dispositions_retries_absence_observation_only() {
        let (temp, bundle, journal, source, before) = fixture();
        let removal = authorize_explicit_user_bundle_removal(
            &bundle,
            &journal,
            &AtomicBool::new(false),
            end(),
        )
        .unwrap();
        let failure =
            failed(
                removal.remove_with_hook(&AtomicBool::new(false), end(), &mut |_, roots| {
                    if roots == 4 {
                        Err(io::ErrorKind::Interrupted.into())
                    } else {
                        Ok(())
                    }
                }),
            );
        assert_eq!(failure.accepted_root_dispositions(), 4);
        assert_eq!(failure.retained_root_cap_count(), 0);
        assert_eq!(failure.retained_file_cap_count(), 0);
        assert!(!bundle.exists());
        let result = failure.retry(&AtomicBool::new(false), end()).unwrap();
        assert_eq!(result.accepted_file_dispositions, 6);
        assert_eq!(result.accepted_root_dispositions, 4);
        assert!(result.outer_namespace_absence_confirmed);
        outside(&temp, &journal, &source, &before);
    }
    use crate::app_install::{
        create_journal_aware_bundle,
        manifest::{BundleEntry, BundleManifest},
    };
    use std::{
        fs,
        fs::File,
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    fn end() -> Instant {
        Instant::now() + Duration::from_secs(30)
    }
    type Fixture = (
        tempfile::TempDir,
        std::path::PathBuf,
        std::path::PathBuf,
        Vec<u8>,
        Vec<(String, Vec<u8>)>,
    );
    fn fixture() -> Fixture {
        let t = tempfile::tempdir().unwrap();
        let mut b = vec![0u8; 70000];
        b[..2].copy_from_slice(b"MZ");
        b[60..64].copy_from_slice(&64u32.to_le_bytes());
        b[64..68].copy_from_slice(b"PE\0\0");
        b[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        b[84..86].copy_from_slice(&112u16.to_le_bytes());
        b[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        fs::write(t.path().join("source"), &b).unwrap();
        fs::write(t.path().join("outside-sentinel"), b"preserve").unwrap();
        let f = File::open(t.path().join("source")).unwrap();
        let m = BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-msvc".into(),
            development_only: true,
            entries: vec![BundleEntry {
                name: "dlpad.exe".into(),
                bytes: b.len() as u64,
                sha256: format!("{:x}", Sha256::digest(&b)),
            }],
        };
        let bp = t.path().join("bundle");
        let jp = t.path().join("journal");
        let created = create_journal_aware_bundle(
            &m,
            &[("dlpad.exe", &f)],
            &bp,
            &jp,
            &AtomicBool::new(false),
            end(),
        )
        .unwrap();
        created
            .verify_current(&AtomicBool::new(false), end())
            .unwrap();
        drop(created);
        let journal = snapshot(&jp);
        (t, bp, jp, b, journal)
    }
    fn snapshot(p: &Path) -> Vec<(String, Vec<u8>)> {
        let mut rows: Vec<_> = fs::read_dir(p)
            .unwrap()
            .map(|e| {
                let e = e.unwrap();
                assert!(e.file_type().unwrap().is_file());
                (
                    e.file_name().into_string().unwrap(),
                    fs::read(e.path()).unwrap(),
                )
            })
            .collect();
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        rows
    }
    fn outside(t: &tempfile::TempDir, j: &Path, source: &[u8], journal: &[(String, Vec<u8>)]) {
        assert!(
            fs::read(t.path().join("source")).unwrap() == source,
            "outside source changed"
        );
        assert_eq!(
            fs::read(t.path().join("outside-sentinel")).unwrap(),
            b"preserve"
        );
        assert_eq!(snapshot(j), journal);
    }
    fn failed<T>(r: Result<T, BundleRemovalFailure>) -> BundleRemovalFailure {
        match r {
            Err(e) => e,
            Ok(_) => panic!("expected explicit removal failure"),
        }
    }
    #[test]
    fn exact_one_payload_plus_five_markers_and_four_roots_removed_external_journal_preserved() {
        let (t, b, j, s, js) = fixture();
        let a =
            authorize_explicit_user_bundle_removal(&b, &j, &AtomicBool::new(false), end()).unwrap();
        let o = a.remove(&AtomicBool::new(false), end()).unwrap();
        assert_eq!(o.accepted_file_dispositions, 6);
        assert_eq!(o.accepted_root_dispositions, 4);
        assert!(o.bundle_root_disposition_accepted);
        assert!(!b.exists());
        outside(&t, &j, &s, &js);
    }
    #[test]
    fn precancelled_and_expired_factory_accept_zero_and_preserve_complete_bundle() {
        for cancel in [true, false] {
            let (t, b, j, s, js) = fixture();
            let before = snapshot(&b.join("registered"));
            let f = failed(authorize_explicit_user_bundle_removal(
                &b,
                &j,
                &AtomicBool::new(cancel),
                if cancel { end() } else { Instant::now() },
            ));
            assert_eq!(
                f.kind(),
                if cancel {
                    io::ErrorKind::Interrupted
                } else {
                    io::ErrorKind::TimedOut
                }
            );
            assert_eq!(f.accepted_file_dispositions(), 0);
            assert_eq!(f.accepted_root_dispositions(), 0);
            assert!(!f.mutation_may_have_occurred());
            assert_eq!(snapshot(&b.join("registered")), before);
            outside(&t, &j, &s, &js);
        }
    }
    #[test]
    fn foreign_outer_before_or_after_authorization_refuses_without_disposition() {
        for after in [false, true] {
            let (t, b, j, s, js) = fixture();
            let a = if after {
                Some(
                    authorize_explicit_user_bundle_removal(&b, &j, &AtomicBool::new(false), end())
                        .unwrap(),
                )
            } else {
                None
            };
            fs::write(b.join("foreign.txt"), b"foreign-preserve").unwrap();
            let before = snapshot(&b.join("registered"));
            let f = if let Some(a) = a {
                failed(a.remove(&AtomicBool::new(false), end()))
            } else {
                failed(authorize_explicit_user_bundle_removal(
                    &b,
                    &j,
                    &AtomicBool::new(false),
                    end(),
                ))
            };
            assert_eq!(f.kind(), io::ErrorKind::InvalidData);
            assert_eq!(f.accepted_file_dispositions(), 0);
            assert_eq!(f.accepted_root_dispositions(), 0);
            assert!(!f.mutation_may_have_occurred());
            assert_eq!(
                fs::read(b.join("foreign.txt")).unwrap(),
                b"foreign-preserve"
            );
            assert_eq!(snapshot(&b.join("registered")), before);
            outside(&t, &j, &s, &js);
        }
    }
    #[test]
    fn interrupted_after_actual_first_disposition_retries_original_remaining_caps() {
        let (t, b, j, s, js) = fixture();
        let a =
            authorize_explicit_user_bundle_removal(&b, &j, &AtomicBool::new(false), end()).unwrap();
        let f = failed(
            a.remove_with_hook(&AtomicBool::new(false), end(), &mut |files, roots| {
                if files == 1 && roots == 0 {
                    Err(io::ErrorKind::Interrupted.into())
                } else {
                    Ok(())
                }
            }),
        );
        assert_eq!(f.kind(), io::ErrorKind::Interrupted);
        assert_eq!(f.accepted_file_dispositions(), 1);
        assert_eq!(f.accepted_root_dispositions(), 0);
        assert!(f.mutation_may_have_occurred());
        assert_eq!(f.retained_file_cap_count(), 5);
        assert_eq!(f.retained_root_cap_count(), 4);
        assert!(!b.join("active/active-selection.json").exists());
        outside(&t, &j, &s, &js);
        let o = f.retry(&AtomicBool::new(false), end()).unwrap();
        assert_eq!(o.accepted_file_dispositions, 6);
        assert_eq!(o.accepted_root_dispositions, 4);
        assert!(!b.exists());
        outside(&t, &j, &s, &js);
    }
    #[test]
    fn error_after_first_root_gone_retry_does_not_reopen_retired_namespace() {
        let (t, b, j, s, js) = fixture();
        let a =
            authorize_explicit_user_bundle_removal(&b, &j, &AtomicBool::new(false), end()).unwrap();
        let f = failed(
            a.remove_with_hook(&AtomicBool::new(false), end(), &mut |_files, roots| {
                if roots == 1 {
                    Err(io::ErrorKind::WriteZero.into())
                } else {
                    Ok(())
                }
            }),
        );
        assert_eq!(f.kind(), io::ErrorKind::WriteZero);
        assert_eq!(f.accepted_file_dispositions(), 6);
        assert_eq!(f.accepted_root_dispositions(), 1);
        assert_eq!(f.retained_file_cap_count(), 0);
        assert_eq!(f.retained_root_cap_count(), 3);
        assert!(!b.join("registered").exists());
        outside(&t, &j, &s, &js);
        let o = f.retry(&AtomicBool::new(false), end()).unwrap();
        assert_eq!(o.accepted_file_dispositions, 6);
        assert_eq!(o.accepted_root_dispositions, 4);
        assert!(!b.exists());
        outside(&t, &j, &s, &js);
    }
}
