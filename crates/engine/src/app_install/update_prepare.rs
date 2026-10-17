//! Worker-only preparation of a separate new installation from an original staged ZIP.
//! Never switches the running installation, touches its profile, executes, or deletes files.
use super::{
    FreshJournalAwareBundle, FreshRoot, JournalBundleFailure, create_journal_aware_bundle,
    manifest::BundleManifest,
    payload::reading_pin,
    windows_caps::{CreatedFile, pin_source},
};
use crate::app_update::{
    StagedPackage,
    bundle::{BundleError, BundleObservation, copy_staged_bundle, inspect_staged_bundle},
};
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read},
    os::windows::ffi::OsStrExt,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
const MAPPING: [(&str, &str); 7] = [
    ("dlpad-install.exe", "dlpad-install.exe"),
    ("payload/manifest.json", "manifest.json"),
    ("payload/dlpad.exe", "dlpad.exe"),
    ("payload/LICENSE", "LICENSE"),
    ("payload/THIRD-PARTY-NOTICES.txt", "THIRD-PARTY-NOTICES.txt"),
    ("payload/README.txt", "README.txt"),
    ("payload/binding-record.txt", "binding-record.txt"),
];
/// All three paths must be absent, disjoint children of existing local Windows directories.
/// No previous installation path or authority is accepted by this operation.
pub struct UpdatePreparationRequest<'a> {
    pub extraction_root: &'a Path,
    pub bundle_root: &'a Path,
    pub journal_root: &'a Path,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdatePreparationError {
    Io(io::ErrorKind),
    Bundle(BundleError),
}
/// Retains original newly-created capabilities. Drop preserves every object.
pub struct PreparedUpdateBundle {
    extraction: FreshRoot,
    files: Vec<CreatedFile>,
    installation: FreshJournalAwareBundle,
    observation: BundleObservation,
}
impl PreparedUpdateBundle {
    pub fn observation(&self) -> &BundleObservation {
        &self.observation
    }
    pub fn installation(&self) -> &FreshJournalAwareBundle {
        &self.installation
    }
    /// Current prepared bytes only, never permission to activate, launch or remove anything.
    pub fn verify_current(&self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
        verify_extracted(
            &self.extraction,
            &self.files,
            &self.observation,
            cancel,
            deadline,
        )?;
        self.installation.verify_current(cancel, deadline)?;
        verify_extracted(
            &self.extraction,
            &self.files,
            &self.observation,
            cancel,
            deadline,
        )
    }
}
struct FailureState {
    error: UpdatePreparationError,
    mutation_may_have_occurred: bool,
    extraction: Option<FreshRoot>,
    files: Vec<CreatedFile>,
    installation: Option<FreshJournalAwareBundle>,
    install_failure: Option<JournalBundleFailure>,
}
/// Partial creation is preserved. Path hints and these facts grant no cleanup authority.
pub struct UpdatePreparationFailure(Box<FailureState>);
impl UpdatePreparationFailure {
    pub fn kind(&self) -> UpdatePreparationError {
        self.0.error
    }
    pub fn mutation_may_have_occurred(&self) -> bool {
        self.0.mutation_may_have_occurred
    }
    pub fn extraction_capability_present(&self) -> bool {
        self.0.extraction.is_some()
    }
    pub fn retained_source_files(&self) -> usize {
        self.0.files.len()
    }
    pub fn installation_capabilities_present(&self) -> bool {
        self.0.installation.is_some()
            || self.0.install_failure.as_ref().is_some_and(|e| {
                e.bundle_capabilities_present() || e.journal_capabilities_present()
            })
    }
}
impl std::fmt::Debug for UpdatePreparationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdatePreparationFailure")
            .field("error", &self.kind())
            .field(
                "mutation_may_have_occurred",
                &self.mutation_may_have_occurred(),
            )
            .finish_non_exhaustive()
    }
}
impl std::fmt::Display for UpdatePreparationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "separate app preparation failed ({:?})", self.kind())
    }
}
impl std::error::Error for UpdatePreparationFailure {}
fn invalid() -> io::Error {
    io::ErrorKind::InvalidInput.into()
}
fn checkpoint(c: &AtomicBool, d: Instant) -> io::Result<()> {
    if c.load(Ordering::Acquire) {
        return Err(io::ErrorKind::Interrupted.into());
    }
    if Instant::now() >= d {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(())
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CompareStringOrdinal(a: *const u16, al: i32, b: *const u16, bl: i32, ignore: i32) -> i32;
}
fn contains(a: &Path, b: &Path) -> io::Result<bool> {
    let a: Vec<_> = a.as_os_str().encode_wide().collect();
    let b: Vec<_> = b.as_os_str().encode_wide().collect();
    if a.is_empty() || a.len() > 32767 || b.len() > 32767 {
        return Err(invalid());
    }
    if a.len() > b.len() {
        return Ok(false);
    }
    // SAFETY: bounded live UTF16 buffers; comparing only the available prefix of b.
    let result =
        unsafe { CompareStringOrdinal(a.as_ptr(), a.len() as i32, b.as_ptr(), a.len() as i32, 1) };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(result == 2 && (a.len() == b.len() || a.last() == Some(&92) || b.get(a.len()) == Some(&92)))
}
fn validate_paths(r: &UpdatePreparationRequest<'_>, c: &AtomicBool, d: Instant) -> io::Result<()> {
    let mut canonical = Vec::with_capacity(3);
    for path in [r.extraction_root, r.bundle_root, r.journal_root] {
        checkpoint(c, d)?;
        super::local_prepare::validate(path)?;
        let parent = std::fs::canonicalize(path.parent().ok_or_else(invalid)?)?;
        let leaf = path.file_name().ok_or_else(invalid)?;
        canonical.push(parent.join(leaf));
    }
    for (i, a) in canonical.iter().enumerate() {
        for b in &canonical[i + 1..] {
            if contains(a, b)? || contains(b, a)? {
                return Err(invalid());
            }
        }
    }
    checkpoint(c, d)
}
fn verify_extracted(
    root: &FreshRoot,
    files: &[CreatedFile],
    observation: &BundleObservation,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<()> {
    checkpoint(c, d)?;
    root.verify()?;
    if files.len() != MAPPING.len() || observation.members().len() != MAPPING.len() {
        return Err(io::ErrorKind::InvalidData.into());
    }
    let mut actual = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(root.path())? {
        checkpoint(c, d)?;
        let entry = entry?;
        if actual.len() >= MAPPING.len() || !entry.file_type()?.is_file() {
            return Err(io::ErrorKind::InvalidData.into());
        }
        actual.insert(entry.file_name().into_string().map_err(|_| invalid())?);
    }
    let expected = MAPPING.iter().map(|(_, name)| (*name).to_owned()).collect();
    if actual != expected {
        return Err(io::ErrorKind::InvalidData.into());
    }
    for (member, name) in MAPPING {
        checkpoint(c, d)?;
        let file = files
            .iter()
            .find(|f| f.name() == name)
            .ok_or_else(invalid)?;
        file.verify(root)?;
        let expected = observation
            .members()
            .iter()
            .find(|m| m.name() == member)
            .ok_or_else(invalid)?;
        let mut reader = reading_pin(file.file())?;
        if reader.metadata()?.len() != expected.bytes() {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let mut digest = Sha256::new();
        let mut count = 0u64;
        let mut buffer = [0u8; 65536];
        loop {
            checkpoint(c, d)?;
            let limit = usize::try_from(expected.bytes() - count + 1)
                .unwrap_or(buffer.len())
                .min(buffer.len());
            let n = reader.read(&mut buffer[..limit])?;
            if n == 0 {
                break;
            }
            count += n as u64;
            if count > expected.bytes() {
                return Err(io::ErrorKind::InvalidData.into());
            }
            digest.update(&buffer[..n]);
        }
        let sha: [u8; 32] = digest.finalize().into();
        if count != expected.bytes() || &sha != expected.sha256() {
            return Err(io::ErrorKind::InvalidData.into());
        }
        file.verify(root)?;
    }
    root.verify()?;
    checkpoint(c, d)
}
/// Prepares a new fixed bundle and its own journal; never updates an existing selection.
/// Caller must later obtain fresh signed policy, process and profile compatibility authority.
pub fn prepare_staged_update(
    package: &mut StagedPackage,
    request: &UpdatePreparationRequest<'_>,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<PreparedUpdateBundle, UpdatePreparationFailure> {
    let mut state = Box::new(FailureState {
        error: UpdatePreparationError::Io(io::ErrorKind::InvalidData),
        mutation_may_have_occurred: false,
        extraction: None,
        files: Vec::with_capacity(7),
        installation: None,
        install_failure: None,
    });
    let result = (|| -> Result<BundleObservation, UpdatePreparationError> {
        let io = UpdatePreparationError::Io;
        checkpoint(cancel, deadline).map_err(|e| io(e.kind()))?;
        validate_paths(request, cancel, deadline).map_err(|e| io(e.kind()))?;
        // Verify the entire source before creating any destination.
        inspect_staged_bundle(package, cancel, deadline).map_err(UpdatePreparationError::Bundle)?;
        state.mutation_may_have_occurred = true;
        state.extraction =
            Some(FreshRoot::create_new(request.extraction_root).map_err(|e| io(e.kind()))?);
        let root = state
            .extraction
            .as_ref()
            .ok_or(io(io::ErrorKind::InvalidData))?;
        for (_, name) in MAPPING {
            checkpoint(cancel, deadline).map_err(|e| io(e.kind()))?;
            state
                .files
                .push(root.create_file(name).map_err(|e| io(e.kind()))?);
        }
        let mut outputs: Vec<_> = state
            .files
            .iter_mut()
            .zip(MAPPING)
            .map(|(f, (member, _))| (member, f.file_mut()))
            .collect();
        let observation = copy_staged_bundle(package, &mut outputs, cancel, deadline)
            .map_err(UpdatePreparationError::Bundle)?;
        // First retain a read-only original HANDLE while closing the writer, then
        // acquire the existing share-read source pin. No path reopen or handle gap.
        for file in &mut state.files {
            checkpoint(cancel, deadline).map_err(|e| io(e.kind()))?;
            *file.file_mut() = reading_pin(file.file()).map_err(|e| io(e.kind()))?;
            *file.file_mut() = pin_source(file.file()).map_err(|e| io(e.kind()))?;
            file.verify(root).map_err(|e| io(e.kind()))?;
        }
        verify_extracted(root, &state.files, &observation, cancel, deadline)
            .map_err(|e| io(e.kind()))?;
        let manifest_file = state
            .files
            .iter()
            .find(|f| f.name() == "manifest.json")
            .ok_or(io(io::ErrorKind::InvalidData))?;
        let manifest =
            BundleManifest::read(reading_pin(manifest_file.file()).map_err(|e| io(e.kind()))?)
                .map_err(|e| io(e.kind()))?;
        let mut inputs = Vec::with_capacity(manifest.entries.len());
        for e in &manifest.entries {
            let file = state
                .files
                .iter()
                .find(|f| f.name() == e.name)
                .ok_or(io(io::ErrorKind::InvalidData))?;
            inputs.push((e.name.as_str(), file.file()));
        }
        match create_journal_aware_bundle(
            &manifest,
            &inputs,
            request.bundle_root,
            request.journal_root,
            cancel,
            deadline,
        ) {
            Ok(bundle) => state.installation = Some(bundle),
            Err(error) => {
                let kind = error.kind();
                state.install_failure = Some(error);
                return Err(io(kind));
            }
        }
        inspect_staged_bundle(package, cancel, deadline).map_err(UpdatePreparationError::Bundle)?;
        state
            .installation
            .as_ref()
            .ok_or(io(io::ErrorKind::InvalidData))?
            .verify_current(cancel, deadline)
            .map_err(|e| io(e.kind()))?;
        verify_extracted(root, &state.files, &observation, cancel, deadline)
            .map_err(|e| io(e.kind()))?;
        Ok(observation)
    })();
    match result {
        Ok(observation) => Ok(PreparedUpdateBundle {
            extraction: state
                .extraction
                .take()
                .expect("successful preparation retains extraction"),
            files: std::mem::take(&mut state.files),
            installation: state
                .installation
                .take()
                .expect("successful preparation retains installation"),
            observation,
        }),
        Err(error) => {
            state.error = error;
            Err(UpdatePreparationFailure(state))
        }
    }
}

#[cfg(test)]
mod tests;
