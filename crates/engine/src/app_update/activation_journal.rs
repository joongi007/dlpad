//! Local selection journal. Observations never confer executable/file mutation authority.
//! DPAPI protects local-user storage integrity, not against malicious code of the same SID.
#![cfg(any(windows, test))]
use crate::{
    app_update::{Channel, Policy, TrustedKey, VerifiedCandidate, verify_metadata},
    file_identity::NativeFileIdentity,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{io, path::Path};
const MAX_PLAINTEXT: usize = 192 * 1024;
#[cfg(windows)]
const MAX_CIPHERTEXT: usize = MAX_PLAINTEXT + 64 * 1024;
#[cfg(windows)]
const MAX_SLOTS: u32 = 512;
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ImageRecord {
    bootstrap_version: String,
    metadata: Vec<u8>,
    signature: String,
    key_id: String,
    version: String,
    channel: String,
    sequence: u64,
    validated_at_unix: u64,
    executable_path: String,
    executable_sha256: [u8; 32],
    executable_bytes: u64,
    executable_identity: NativeFileIdentity,
    active_record_sha256: [u8; 32],
    bundle_root: String,
    active_root: String,
    registered_root: String,
    receipt_root: String,
    profile_epoch: u32,
}
/// Authenticated local observation, not a retained original executable capability.
#[derive(Clone)]
pub struct InstalledImageEvidence(ImageRecord);
impl InstalledImageEvidence {
    pub fn bootstrap_version(&self) -> &str {
        &self.0.bootstrap_version
    }
    pub fn version(&self) -> &str {
        &self.0.version
    }
    pub fn signed_payload(&self) -> &[u8] {
        &self.0.metadata
    }
    pub fn signature_hex(&self) -> &str {
        &self.0.signature
    }
    pub fn key_id(&self) -> &str {
        &self.0.key_id
    }
    pub fn validated_at_unix(&self) -> u64 {
        self.0.validated_at_unix
    }
    pub fn executable_path(&self) -> &Path {
        Path::new(&self.0.executable_path)
    }
    pub fn executable_sha256(&self) -> &[u8; 32] {
        &self.0.executable_sha256
    }
    pub fn executable_bytes(&self) -> u64 {
        self.0.executable_bytes
    }
    pub fn executable_identity(&self) -> &NativeFileIdentity {
        &self.0.executable_identity
    }
    pub fn active_record_sha256(&self) -> &[u8; 32] {
        &self.0.active_record_sha256
    }
    pub fn bundle_root(&self) -> &Path {
        Path::new(&self.0.bundle_root)
    }
    pub fn active_root(&self) -> &Path {
        Path::new(&self.0.active_root)
    }
    pub fn registered_root(&self) -> &Path {
        Path::new(&self.0.registered_root)
    }
    pub fn receipt_root(&self) -> &Path {
        Path::new(&self.0.receipt_root)
    }
    pub fn profile_epoch(&self) -> u32 {
        self.0.profile_epoch
    }
    pub fn sequence(&self) -> u64 {
        self.0.sequence
    }
}
impl std::fmt::Debug for InstalledImageEvidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("InstalledImageEvidence(<redacted>)")
    }
}
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct SelectionState {
    active: Option<ImageRecord>,
    previous: Option<ImageRecord>,
    highest_sequence: u64,
    failed_sequence: Option<u64>,
}
#[derive(Clone)]
pub struct SelectionObservation {
    generation: u32,
    record_sha256: [u8; 32],
    state: SelectionState,
}
impl SelectionObservation {
    pub fn generation(&self) -> u32 {
        self.generation
    }
    pub fn record_sha256(&self) -> &[u8; 32] {
        &self.record_sha256
    }
    pub fn active(&self) -> Option<InstalledImageEvidence> {
        self.state.active.clone().map(InstalledImageEvidence)
    }
    pub fn previous(&self) -> Option<InstalledImageEvidence> {
        self.state.previous.clone().map(InstalledImageEvidence)
    }
    pub fn highest_sequence(&self) -> u64 {
        self.state.highest_sequence
    }
    pub fn failed_sequence(&self) -> Option<u64> {
        self.state.failed_sequence
    }
}
impl std::fmt::Debug for SelectionObservation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectionObservation")
            .field("generation", &self.generation)
            .field("highest_sequence", &self.state.highest_sequence)
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Event {
    Commit,
    Failed,
    Rollback,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema: u32,
    domain: String,
    generation: u32,
    previous_sha256: [u8; 32],
    root_identity: NativeFileIdentity,
    bootstrap_identity: NativeFileIdentity,
    bootstrap_version: String,
    profile_epoch: u32,
    event: Event,
    state: SelectionState,
}
fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "app update selection journal rejected",
    )
}
fn version(value: &str) -> Option<[u32; 3]> {
    let mut result = [0; 3];
    let mut parts = value.split('.');
    for v in &mut result {
        let p = parts.next()?;
        if p.is_empty()
            || p.len() > 9
            || (p.len() > 1 && p.starts_with('0'))
            || !p.bytes().all(|v| v.is_ascii_digit())
        {
            return None;
        }
        *v = p.parse().ok()?;
    }
    parts.next().is_none().then_some(result)
}
fn channel(value: &str) -> io::Result<Channel> {
    match value {
        "stable" => Ok(Channel::Stable),
        "preview" => Ok(Channel::Preview),
        _ => Err(invalid()),
    }
}
fn candidate(
    image: &ImageRecord,
    keys: &[TrustedKey<'_>],
    now: u64,
) -> io::Result<VerifiedCandidate> {
    let result = verify_metadata(
        &image.metadata,
        &image.key_id,
        &image.signature,
        keys,
        &Policy {
            installed_version: "0.0.0",
            installed_sequence: 0,
            channel: channel(&image.channel)?,
            now_unix: now,
        },
    )
    .map_err(|_| invalid())?;
    if result.version() != image.version || result.sequence() != image.sequence {
        return Err(invalid());
    }
    Ok(result)
}
fn windows_path(value: &str) -> bool {
    let s = value.strip_prefix("\\\\?\\").unwrap_or(value);
    let b = s.as_bytes();
    b.len() >= 3
        && b[0].is_ascii_alphabetic()
        && b[1] == b':'
        && b[2] == b'\\'
        && s.encode_utf16().count() <= 32767
        && !s.chars().any(|c| c.is_control() || c == '/' || c == '\0')
        && s[3..].split('\\').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && !p.ends_with(['.', ' '])
                && !p.contains([':', '"', '<', '>', '|', '?', '*'])
        })
}
fn validate_image(image: &ImageRecord, keys: &[TrustedKey<'_>], epoch: u32) -> io::Result<()> {
    candidate(image, keys, image.validated_at_unix)?;
    if image.profile_epoch != epoch
        || image.executable_bytes == 0
        || image.executable_bytes > 256 * 1024 * 1024
        || image.executable_sha256 == [0; 32]
        || !matches!(image.executable_identity,NativeFileIdentity::Windows{volume_serial,file_id} if volume_serial!=0&&file_id!=[0;16])
    {
        return Err(invalid());
    }
    for path in [
        &image.executable_path,
        &image.bundle_root,
        &image.active_root,
        &image.registered_root,
        &image.receipt_root,
    ] {
        if !windows_path(path) {
            return Err(invalid());
        }
    }
    if image.executable_path != format!("{}\\dlpad.exe", image.registered_root) {
        return Err(invalid());
    }
    Ok(())
}
fn canonical(record: &Record) -> io::Result<Vec<u8>> {
    let bytes = serde_json::to_vec(record).map_err(|_| invalid())?;
    if bytes.len() > MAX_PLAINTEXT {
        return Err(invalid());
    }
    Ok(bytes)
}
fn same_image(a: &Option<ImageRecord>, b: &Option<ImageRecord>) -> bool {
    serde_json::to_vec(a).ok() == serde_json::to_vec(b).ok()
}
fn valid_transition(
    previous: &SelectionState,
    next: &SelectionState,
    event: &Event,
    bootstrap: &str,
) -> io::Result<()> {
    if next.highest_sequence < previous.highest_sequence
        || next
            .failed_sequence
            .is_some_and(|v| v > next.highest_sequence)
    {
        return Err(invalid());
    }
    match event {
        Event::Commit => {
            let active = next.active.as_ref().ok_or_else(invalid)?;
            let old_version = previous
                .active
                .as_ref()
                .map_or(bootstrap, |v| v.version.as_str());
            if active.sequence <= previous.highest_sequence
                || next.highest_sequence != active.sequence
                || version(&active.version) <= version(old_version)
                || !same_image(&next.previous, &previous.active)
                || next.failed_sequence != previous.failed_sequence
            {
                return Err(invalid());
            }
        }
        Event::Failed => {
            let failed = next.failed_sequence.ok_or_else(invalid)?;
            if failed == 0
                || next.highest_sequence != previous.highest_sequence.max(failed)
                || (failed < previous.highest_sequence
                    && previous
                        .active
                        .as_ref()
                        .is_none_or(|v| v.sequence != failed))
                || !same_image(&next.active, &previous.active)
                || !same_image(&next.previous, &previous.previous)
            {
                return Err(invalid());
            }
        }
        Event::Rollback => {
            if previous.active.is_none()
                || !same_image(&next.active, &previous.previous)
                || next.previous.is_some()
                || next.highest_sequence != previous.highest_sequence
                || next.failed_sequence != previous.active.as_ref().map(|v| v.sequence)
            {
                return Err(invalid());
            }
        }
    }
    Ok(())
}
#[cfg(windows)]
mod windows {
    use super::*;
    use crate::{
        app_install::PreparedUpdateBundle,
        created_relative_file::{CreatedRelativeFile, PublishError, PublishedRelativeFile},
        file_identity, windows_private_security,
    };
    use std::{
        fs::{File, OpenOptions},
        io::Write,
        os::windows::{
            fs::OpenOptionsExt,
            io::{AsRawHandle, FromRawHandle},
        },
        path::PathBuf,
        sync::atomic::{AtomicBool, Ordering},
        time::Instant,
    };
    use windows_sys::Win32::{
        Foundation::{GENERIC_READ, INVALID_HANDLE_VALUE, LocalFree},
        Security::Cryptography::{
            CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
        },
        Storage::FileSystem::{
            FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
            ReOpenFile,
        },
    };
    /// Identity-only File handles cannot read bytes. Reopen the retained object,
    /// retaining exact identity/path/length while denying new writers and deleters.
    pub(super) fn reopen_content(original: &File) -> io::Result<File> {
        // SAFETY: retained original HANDLE; exact readonly access/share and unique result.
        let raw = unsafe {
            ReOpenFile(
                original.as_raw_handle(),
                GENERIC_READ,
                FILE_SHARE_READ,
                FILE_FLAG_OPEN_REPARSE_POINT,
            )
        };
        if raw == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: successful ReOpenFile returned a new uniquely owned HANDLE.
        let file = unsafe { File::from_raw_handle(raw) };
        if file_identity::from_file(original)? != file_identity::from_file(&file)?
            || file_identity::final_path(original)? != file_identity::final_path(&file)?
            || original.metadata()?.len() != file.metadata()?.len()
        {
            return Err(invalid());
        }
        Ok(file)
    }
    /// Existing private directory and pinned original bootstrap; no pathname adoption authority.
    pub struct JournalBinding<'a> {
        pub root: &'a File,
        pub root_path: &'a Path,
        pub bootstrap: &'a File,
        pub bootstrap_version: &'a str,
        pub profile_epoch: u32,
    }
    pub struct CapturedImage {
        image: ImageRecord,
        binding_sha256: [u8; 32],
    }
    impl std::fmt::Debug for CapturedImage {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("CapturedImage(<redacted>)")
        }
    }
    pub struct SignedImageInput<'a> {
        pub candidate: &'a VerifiedCandidate,
        pub prepared: &'a PreparedUpdateBundle,
        pub verified_at_unix: u64,
    }
    #[derive(Debug)]
    pub struct JournalFailure {
        kind: io::ErrorKind,
        os_code: Option<i32>,
        mutation_may_have_occurred: bool,
    }
    impl JournalFailure {
        pub fn kind(&self) -> io::ErrorKind {
            self.kind
        }
        pub fn raw_os_error(&self) -> Option<i32> {
            self.os_code
        }
        pub fn mutation_may_have_occurred(&self) -> bool {
            self.mutation_may_have_occurred
        }
    }
    impl std::fmt::Display for JournalFailure {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "app update journal unavailable ({:?})", self.kind)
        }
    }
    impl std::error::Error for JournalFailure {}
    fn failure(error: io::Error, mutation: bool) -> JournalFailure {
        JournalFailure {
            kind: error.kind(),
            os_code: error.raw_os_error(),
            mutation_may_have_occurred: mutation,
        }
    }
    fn checkpoint(cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
        if cancel.load(Ordering::Acquire) {
            return Err(io::ErrorKind::Interrupted.into());
        }
        if Instant::now() >= deadline {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(())
    }
    pub(super) fn protect(input: &[u8], entropy: &[u8], decrypt: bool) -> io::Result<Vec<u8>> {
        if input.is_empty() || input.len() > MAX_CIPHERTEXT || entropy.len() > 4096 {
            return Err(invalid());
        }
        let source = CRYPT_INTEGER_BLOB {
            cbData: input.len() as u32,
            pbData: input.as_ptr().cast_mut(),
        };
        let extra = CRYPT_INTEGER_BLOB {
            cbData: entropy.len() as u32,
            pbData: entropy.as_ptr().cast_mut(),
        };
        let mut output = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        };
        // SAFETY: live bounded input/entropy buffers, no prompt, current-user scope only.
        let result = unsafe {
            if decrypt {
                CryptUnprotectData(
                    &source,
                    std::ptr::null_mut(),
                    &extra,
                    std::ptr::null(),
                    std::ptr::null(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut output,
                )
            } else {
                CryptProtectData(
                    &source,
                    std::ptr::null(),
                    &extra,
                    std::ptr::null(),
                    std::ptr::null(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut output,
                )
            }
        };
        struct Allocation(*mut u8);
        impl Drop for Allocation {
            fn drop(&mut self) {
                if !self.0.is_null() {
                    // SAFETY: CryptProtect/Unprotect output is LocalAlloc-owned, freed exactly once.
                    unsafe {
                        LocalFree(self.0.cast());
                    }
                }
            }
        }
        let allocation = Allocation(output.pbData);
        if result == 0 {
            return Err(io::Error::last_os_error());
        }
        let maximum = if decrypt {
            MAX_PLAINTEXT
        } else {
            MAX_CIPHERTEXT
        };
        if allocation.0.is_null() || output.cbData == 0 || output.cbData as usize > maximum {
            return Err(invalid());
        }
        // SAFETY: successful DPAPI output of checked bounded length remains allocated.
        Ok(unsafe { std::slice::from_raw_parts(allocation.0, output.cbData as usize) }.to_vec())
    }
    struct Pin {
        file: File,
        identity: NativeFileIdentity,
        sha: [u8; 32],
        path: PathBuf,
    }
    pub struct Journal<'a> {
        root: File,
        root_path: PathBuf,
        root_id: NativeFileIdentity,
        bootstrap: File,
        bootstrap_id: NativeFileIdentity,
        bootstrap_version: String,
        epoch: u32,
        sid: String,
        _lease: File,
        keys: &'a [TrustedKey<'a>],
        entropy: Vec<u8>,
        head: SelectionObservation,
        pins: Vec<Pin>,
        published: Vec<PublishedRelativeFile>,
        pending: Option<CreatedRelativeFile>,
        poisoned: bool,
    }
    impl std::fmt::Debug for Journal<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("UpdateJournal")
                .field("head", &self.head)
                .field("poisoned", &self.poisoned)
                .finish_non_exhaustive()
        }
    }
    fn read_bytes(file: &File) -> io::Result<Vec<u8>> {
        use std::os::windows::fs::FileExt;
        let length = file.metadata()?.len();
        if length == 0 || length > MAX_CIPHERTEXT as u64 {
            return Err(invalid());
        }
        let mut bytes = vec![0; length as usize];
        let mut at = 0;
        while at < bytes.len() {
            let n = file.seek_read(&mut bytes[at..], at as u64)?;
            if n == 0 {
                return Err(invalid());
            }
            at += n;
        }
        if file.seek_read(&mut [0], length)? != 0 || file.metadata()?.len() != length {
            return Err(invalid());
        }
        Ok(bytes)
    }
    impl<'a> Journal<'a> {
        pub fn open(
            binding: JournalBinding<'_>,
            keys: &'a [TrustedKey<'a>],
            cancel: &AtomicBool,
            deadline: Instant,
        ) -> Result<Self, JournalFailure> {
            let mut creation_attempted = false;
            (|| -> io::Result<Self> {
                checkpoint(cancel, deadline)?;
                if version(binding.bootstrap_version).is_none()
                    || binding.profile_epoch == 0
                    || keys.is_empty()
                    || keys.len() > 8
                {
                    return Err(invalid());
                }
                let sid = windows_private_security::current_sid()?;
                windows_private_security::verify_acl(binding.root, &sid)?;
                if !binding.bootstrap.metadata()?.is_file()
                    || !binding.root.metadata()?.is_dir()
                    || file_identity::final_path(binding.root)? != binding.root_path
                {
                    return Err(invalid());
                }
                let root_id = file_identity::from_file(binding.root)?;
                let bootstrap_id = file_identity::from_file(binding.bootstrap)?;
                if !matches!(root_id, NativeFileIdentity::Windows { .. })
                    || !matches!(bootstrap_id, NativeFileIdentity::Windows { .. })
                {
                    return Err(invalid());
                }
                if !windows_path(binding.root_path.to_str().ok_or_else(invalid)?)
                    || !windows_path(
                        file_identity::final_path(binding.bootstrap)?
                            .to_str()
                            .ok_or_else(invalid)?,
                    )
                {
                    return Err(invalid());
                }
                let lock_path = binding.root_path.join("writer.lock");
                creation_attempted = true;
                let lease = match CreatedRelativeFile::create(binding.root, "writer.lock") {
                    Ok(mut created) => {
                        created.file_mut().sync_all()?;
                        let file = created.file_mut().try_clone()?;
                        created.verify_current(binding.root_path)?;
                        file
                    }
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                        OpenOptions::new()
                            .read(true)
                            .write(true)
                            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
                            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                            .open(&lock_path)?
                    }
                    Err(error) => return Err(error),
                };
                file_identity::ensure_regular_under(binding.root, &lease)?;
                windows_private_security::verify_acl(&lease, &sid)?;
                if file_identity::final_path(&lease)? != lock_path || lease.metadata()?.len() != 0 {
                    return Err(invalid());
                }
                fs2::FileExt::try_lock_exclusive(&lease)?;
                let entropy = serde_json::to_vec(&(
                    "dlpad.activation-journal.v1",
                    &root_id,
                    &bootstrap_id,
                    binding.bootstrap_version,
                    binding.profile_epoch,
                ))
                .map_err(|_| invalid())?;
                let mut journal = Self {
                    root: binding.root.try_clone()?,
                    root_path: binding.root_path.to_path_buf(),
                    root_id,
                    bootstrap: binding.bootstrap.try_clone()?,
                    bootstrap_id,
                    bootstrap_version: binding.bootstrap_version.into(),
                    epoch: binding.profile_epoch,
                    sid,
                    _lease: lease,
                    keys,
                    entropy,
                    head: SelectionObservation {
                        generation: 0,
                        record_sha256: [0; 32],
                        state: SelectionState::default(),
                    },
                    pins: Vec::new(),
                    published: Vec::new(),
                    pending: None,
                    poisoned: false,
                };
                journal.load(cancel, deadline)?;
                Ok(journal)
            })()
            .map_err(|e| failure(e, creation_attempted))
        }
        pub fn head(&self) -> &SelectionObservation {
            &self.head
        }
        fn verify_roots(&self) -> io::Result<()> {
            if self.poisoned
                || file_identity::from_file(&self.root)? != self.root_id
                || file_identity::from_file(&self.bootstrap)? != self.bootstrap_id
                || file_identity::final_path(&self.root)? != self.root_path
            {
                return Err(invalid());
            }
            windows_private_security::verify_acl(&self.root, &self.sid)
        }
        fn names(&self, cancel: &AtomicBool, deadline: Instant) -> io::Result<Vec<String>> {
            self.verify_roots()?;
            let mut names = Vec::new();
            let mut lock = false;
            for entry in std::fs::read_dir(&self.root_path)? {
                checkpoint(cancel, deadline)?;
                let entry = entry?;
                let name = entry.file_name().into_string().map_err(|_| invalid())?;
                if !entry.file_type()?.is_file() {
                    return Err(invalid());
                }
                if name == "writer.lock" {
                    if lock {
                        return Err(invalid());
                    }
                    lock = true;
                    continue;
                }
                if names.len() >= MAX_SLOTS as usize {
                    return Err(invalid());
                }
                names.push(name);
            }
            if !lock {
                return Err(invalid());
            }
            names.sort();
            for (i, name) in names.iter().enumerate() {
                if *name != format!("slot{:04}.dpapi", i + 1) {
                    return Err(invalid());
                }
            }
            self.verify_roots()?;
            Ok(names)
        }
        fn read_pin(&self, name: &str) -> io::Result<Pin> {
            let path = self.root_path.join(name);
            // PublishedRelativeFile retains a reduced duplicate of the original FILE_OBJECT.
            // Its original share_READ-only reservation still excludes foreign writers/deleters,
            // but a newly opened reader must acknowledge that object's WRITE/DELETE reservation.
            let sharing = if self.published.iter().any(|p| p.path() == path) {
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE
            } else {
                FILE_SHARE_READ
            };
            let file = OpenOptions::new()
                .read(true)
                .share_mode(sharing)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&path)?;
            file_identity::ensure_regular_under(&self.root, &file)?;
            windows_private_security::verify_acl(&file, &self.sid)?;
            if file_identity::final_path(&file)? != path {
                return Err(invalid());
            }
            let identity = file_identity::from_file(&file)?;
            let bytes = read_bytes(&file)?;
            let sha = Sha256::digest(bytes).into();
            Ok(Pin {
                file,
                identity,
                sha,
                path,
            })
        }
        fn decode(&self, pin: &Pin) -> io::Result<Record> {
            let raw = protect(&read_bytes(&pin.file)?, &self.entropy, true)?;
            let record: Record = serde_json::from_slice(&raw).map_err(|_| invalid())?;
            if canonical(&record)? != raw
                || record.schema != 1
                || record.domain != "dlpad.activation-journal.v1"
                || record.root_identity != self.root_id
                || record.bootstrap_identity != self.bootstrap_id
                || record.bootstrap_version != self.bootstrap_version
                || record.profile_epoch != self.epoch
            {
                return Err(invalid());
            }
            for image in [&record.state.active, &record.state.previous]
                .into_iter()
                .flatten()
            {
                if image.bootstrap_version != self.bootstrap_version {
                    return Err(invalid());
                }
                validate_image(image, self.keys, self.epoch)?;
            }
            Ok(record)
        }
        fn load(&mut self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
            for name in self.names(cancel, deadline)? {
                checkpoint(cancel, deadline)?;
                let pin = self.read_pin(&name)?;
                let record = self.decode(&pin)?;
                if record.generation != self.head.generation + 1
                    || record.previous_sha256 != self.head.record_sha256
                {
                    return Err(invalid());
                }
                valid_transition(
                    &self.head.state,
                    &record.state,
                    &record.event,
                    &self.bootstrap_version,
                )?;
                self.head = SelectionObservation {
                    generation: record.generation,
                    record_sha256: pin.sha,
                    state: record.state,
                };
                self.pins.push(pin);
            }
            self.verify_roots()?;
            checkpoint(cancel, deadline)
        }
        pub fn verify_current(
            &self,
            cancel: &AtomicBool,
            deadline: Instant,
        ) -> Result<(), JournalFailure> {
            (|| -> io::Result<()> {
                let names = self.names(cancel, deadline)?;
                if names.len() != self.head.generation as usize {
                    return Err(invalid());
                }
                for pin in &self.pins {
                    checkpoint(cancel, deadline)?;
                    if file_identity::from_file(&pin.file)? != pin.identity
                        || file_identity::final_path(&pin.file)? != pin.path
                        || <[u8; 32]>::from(Sha256::digest(read_bytes(&pin.file)?)) != pin.sha
                    {
                        return Err(invalid());
                    }
                }
                for published in &self.published {
                    published.verify_current(&self.root_path)?;
                }
                self.verify_roots()?;
                checkpoint(cancel, deadline)
            })()
            .map_err(|e| failure(e, false))
        }
        fn expected(
            &self,
            expected: &SelectionObservation,
            cancel: &AtomicBool,
            deadline: Instant,
        ) -> io::Result<()> {
            self.verify_current(cancel, deadline)
                .map_err(|e| io::Error::new(e.kind(), "journal verification failed"))?;
            if self.head.generation != expected.generation
                || self.head.record_sha256 != expected.record_sha256
            {
                return Err(invalid());
            }
            Ok(())
        }
        fn append(
            &mut self,
            expected: &SelectionObservation,
            state: SelectionState,
            event: Event,
            cancel: &AtomicBool,
            deadline: Instant,
        ) -> Result<SelectionObservation, JournalFailure> {
            self.expected(expected, cancel, deadline)
                .map_err(|e| failure(e, false))?;
            if self.head.generation >= MAX_SLOTS {
                return Err(failure(invalid(), false));
            }
            valid_transition(&self.head.state, &state, &event, &self.bootstrap_version)
                .map_err(|e| failure(e, false))?;
            let record = Record {
                schema: 1,
                domain: "dlpad.activation-journal.v1".into(),
                generation: self.head.generation + 1,
                previous_sha256: self.head.record_sha256,
                root_identity: self.root_id.clone(),
                bootstrap_identity: self.bootstrap_id.clone(),
                bootstrap_version: self.bootstrap_version.clone(),
                profile_epoch: self.epoch,
                event,
                state,
            };
            let raw = canonical(&record).map_err(|e| failure(e, false))?;
            let cipher = protect(&raw, &self.entropy, false).map_err(|e| failure(e, false))?;
            checkpoint(cancel, deadline).map_err(|e| failure(e, false))?;
            self.poisoned = true;
            let result = (|| -> io::Result<()> {
                let mut created = CreatedRelativeFile::create(&self.root, "pending.dpapi")?;
                let write = (|| -> io::Result<()> {
                    created.verify_current(&self.root_path)?;
                    created.file_mut().write_all(&cipher)?;
                    created.file_mut().sync_all()?;
                    if read_bytes(created.file_mut())? != cipher {
                        return Err(invalid());
                    }
                    created.verify_current(&self.root_path)?;
                    checkpoint(cancel, deadline)
                })();
                self.pending = Some(created);
                write?;
                let created = self.pending.take().expect("retained original pending");
                let leaf = format!("slot{:04}.dpapi", record.generation);
                let published = match created.publish_no_replace(&leaf) {
                    Ok(value) => value,
                    Err(PublishError::BeforePublish { created, error }) => {
                        self.pending = Some(created);
                        return Err(error);
                    }
                    Err(PublishError::Unconfirmed { error, .. }) => return Err(error),
                };
                self.published.push(published);
                self.published
                    .last()
                    .expect("retained original publication")
                    .verify_current(&self.root_path)?;
                let pin = self.read_pin(&leaf)?;
                let decoded = self.decode(&pin)?;
                if canonical(&decoded)? != raw {
                    return Err(invalid());
                }
                // Only verified durable bytes become the in-memory observed head.
                self.head = SelectionObservation {
                    generation: record.generation,
                    record_sha256: pin.sha,
                    state: record.state,
                };
                self.pins.push(pin);
                Ok(())
            })();
            match result {
                Ok(()) => {
                    self.poisoned = false;
                    if let Err(error) = self.verify_current(cancel, deadline) {
                        self.poisoned = true;
                        return Err(JournalFailure {
                            kind: error.kind,
                            os_code: error.os_code,
                            mutation_may_have_occurred: true,
                        });
                    }
                    Ok(self.head.clone())
                }
                Err(error) => Err(failure(error, true)),
            }
        }
        pub fn capture(
            &self,
            input: SignedImageInput<'_>,
            now_unix: u64,
            cancel: &AtomicBool,
            deadline: Instant,
        ) -> Result<CapturedImage, JournalFailure> {
            let image = (|| -> io::Result<ImageRecord> {
                self.verify_roots()?;
                checkpoint(cancel, deadline)?;
                input.prepared.verify_current(cancel, deadline)?;
                if input.verified_at_unix != now_unix
                    || input.candidate.version() != input.prepared.observation().version()
                    || input.candidate.asset_sha256()
                        != input.prepared.observation().package_sha256()
                {
                    return Err(invalid());
                }
                let active = input.prepared.installation().bundle().active().record();
                let exe = &active.selected_executable;
                let digest: Vec<u8> = exe
                    .sha256
                    .as_bytes()
                    .chunks_exact(2)
                    .map(|pair| {
                        std::str::from_utf8(pair)
                            .ok()
                            .and_then(|v| u8::from_str_radix(v, 16).ok())
                            .ok_or_else(invalid)
                    })
                    .collect::<io::Result<_>>()?;
                let image = ImageRecord {
                    bootstrap_version: self.bootstrap_version.clone(),
                    metadata: input.candidate.signed_payload().into(),
                    signature: input.candidate.signature_hex().into(),
                    key_id: input.candidate.key_id().into(),
                    version: input.candidate.version().into(),
                    channel: match input.candidate.channel() {
                        Channel::Stable => "stable",
                        Channel::Preview => "preview",
                    }
                    .into(),
                    sequence: input.candidate.sequence(),
                    validated_at_unix: now_unix,
                    executable_path: format!("{}\\dlpad.exe", active.registered_path),
                    executable_sha256: digest.try_into().map_err(|_| invalid())?,
                    executable_bytes: exe.bytes,
                    executable_identity: exe.identity.clone(),
                    active_record_sha256: Sha256::digest(active.canonical_bytes()?).into(),
                    bundle_root: input
                        .prepared
                        .installation()
                        .bundle()
                        .path()
                        .to_str()
                        .ok_or_else(invalid)?
                        .into(),
                    active_root: active.active_path.clone(),
                    registered_root: active.registered_path.clone(),
                    receipt_root: active.receipt_path.clone(),
                    profile_epoch: self.epoch,
                };
                validate_image(&image, self.keys, self.epoch)?;
                candidate(&image, self.keys, now_unix)?;
                input.prepared.verify_current(cancel, deadline)?;
                Ok(image)
            })()
            .map_err(|e| failure(e, false))?;
            Ok(CapturedImage {
                image,
                binding_sha256: Sha256::digest(&self.entropy).into(),
            })
        }
        fn verify_captured(
            &self,
            image: &ImageRecord,
            cancel: &AtomicBool,
            deadline: Instant,
        ) -> io::Result<()> {
            use crate::app_install::inspect_active_selection;
            use std::io::Read;
            checkpoint(cancel, deadline)?;
            let inspection = inspect_active_selection(
                Path::new(&image.active_root),
                Path::new(&image.registered_root),
                Path::new(&image.receipt_root),
                cancel,
                deadline,
            )?;
            if <[u8; 32]>::from(Sha256::digest(inspection.record().canonical_bytes()?))
                != image.active_record_sha256
            {
                return Err(invalid());
            }
            let original =
                file_identity::open_regular_no_reparse(Path::new(&image.executable_path))?;
            let mut file = reopen_content(&original)?;
            if file_identity::from_file(&file)? != image.executable_identity
                || file.metadata()?.len() != image.executable_bytes
                || file_identity::final_path(&file)? != Path::new(&image.executable_path)
            {
                return Err(invalid());
            }
            let mut digest = Sha256::new();
            let mut count = 0u64;
            let mut buffer = [0u8; 65536];
            loop {
                checkpoint(cancel, deadline)?;
                let limit = buffer.len().min(
                    usize::try_from(image.executable_bytes - count + 1).unwrap_or(buffer.len()),
                );
                let n = file.read(&mut buffer[..limit])?;
                if n == 0 {
                    break;
                }
                count += n as u64;
                if count > image.executable_bytes {
                    return Err(invalid());
                }
                digest.update(&buffer[..n]);
            }
            if count != image.executable_bytes
                || <[u8; 32]>::from(digest.finalize()) != image.executable_sha256
                || file_identity::from_file(&file)? != image.executable_identity
            {
                return Err(invalid());
            }
            inspection.verify_current(cancel, deadline)?;
            checkpoint(cancel, deadline)
        }
        pub fn commit(
            &mut self,
            expected: &SelectionObservation,
            captured: &CapturedImage,
            now_unix: u64,
            cancel: &AtomicBool,
            deadline: Instant,
        ) -> Result<SelectionObservation, JournalFailure> {
            self.expected(expected, cancel, deadline)
                .map_err(|e| failure(e, false))?;
            if captured.binding_sha256 != <[u8; 32]>::from(Sha256::digest(&self.entropy)) {
                return Err(failure(invalid(), false));
            }
            let mut image = captured.image.clone();
            candidate(&image, self.keys, now_unix).map_err(|e| failure(e, false))?;
            image.validated_at_unix = now_unix;
            validate_image(&image, self.keys, self.epoch).map_err(|e| failure(e, false))?;
            self.verify_captured(&image, cancel, deadline)
                .map_err(|e| failure(e, false))?;
            let mut state = self.head.state.clone();
            state.previous = state.active.take();
            state.highest_sequence = image.sequence;
            state.active = Some(image);
            self.append(expected, state, Event::Commit, cancel, deadline)
        }
        pub fn record_failed(
            &mut self,
            expected: &SelectionObservation,
            sequence: u64,
            cancel: &AtomicBool,
            deadline: Instant,
        ) -> Result<SelectionObservation, JournalFailure> {
            self.expected(expected, cancel, deadline)
                .map_err(|e| failure(e, false))?;
            if sequence == 0
                || (sequence < self.head.state.highest_sequence
                    && self
                        .head
                        .state
                        .active
                        .as_ref()
                        .is_none_or(|v| v.sequence != sequence))
            {
                return Err(failure(invalid(), false));
            }
            if self.head.state.failed_sequence == Some(sequence) {
                return Ok(self.head.clone());
            }
            let mut state = self.head.state.clone();
            state.highest_sequence = state.highest_sequence.max(sequence);
            state.failed_sequence = Some(sequence);
            self.append(expected, state, Event::Failed, cancel, deadline)
        }
        pub fn rollback(
            &mut self,
            expected: &SelectionObservation,
            cancel: &AtomicBool,
            deadline: Instant,
        ) -> Result<SelectionObservation, JournalFailure> {
            self.expected(expected, cancel, deadline)
                .map_err(|e| failure(e, false))?;
            let mut state = self.head.state.clone();
            if state.active.is_none() {
                return Err(failure(invalid(), false));
            }
            let failed = state
                .active
                .as_ref()
                .ok_or_else(|| failure(invalid(), false))?
                .sequence;
            state.active = state.previous.take();
            state.failed_sequence = Some(failed);
            self.append(expected, state, Event::Rollback, cancel, deadline)
        }
    }
}
#[cfg(windows)]
pub use windows::{CapturedImage, Journal, JournalBinding, JournalFailure, SignedImageInput};
#[cfg(test)]
mod tests;
