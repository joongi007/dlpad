//! Linux anchored journal foundation. No cleanup or recovery is inferred from Drop.
use super::reader_records::{MAX_RECORD_BYTES, ReaderRecords, Slot};
use crate::process::reader_protocol::{
    Capability, RECEIPT_SIZE, ReaderIdentity, decode_cleanup_receipt,
};
use std::{
    ffi::CStr,
    fmt,
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::{FileExt, MetadataExt},
    },
};

const RECORD: &CStr = c"recordstate.json";
const RECEIPTS: [&CStr; 2] = [c"reader0.receipt", c"reader1.receipt"];
#[derive(Clone, Copy, Eq, PartialEq)]
struct Identity {
    dev: u64,
    ino: u64,
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum JournalError {
    OwnershipChanged,
    Busy,
    InvalidState,
    Io,
    Poisoned,
}
impl fmt::Debug for JournalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CookieReaderJournalError")
    }
}
impl fmt::Display for JournalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("cookie reader journal unavailable")
    }
}
impl std::error::Error for JournalError {}
/// Explicit authority from the original recovery marker; never inferred from PID absence.
pub(crate) struct PreviousBootAuthority {
    original_boot: [u8; 16],
    current_boot: [u8; 16],
    record_digest: [u8; 32],
    #[cfg(test)]
    synthetic_current: bool,
}
impl fmt::Debug for PreviousBootAuthority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PreviousBootAuthority(<redacted>)")
    }
}
fn parse_kernel_boot(bytes: &[u8]) -> Result<[u8; 16], JournalError> {
    if bytes.len() != 37 || bytes[36] != b'\n' {
        return Err(JournalError::InvalidState);
    }
    let mut value = [0; 16];
    let mut nibble = 0;
    for (index, byte) in bytes[..36].iter().copied().enumerate() {
        if [8, 13, 18, 23].contains(&index) {
            if byte != b'-' {
                return Err(JournalError::InvalidState);
            }
            continue;
        }
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return Err(JournalError::InvalidState),
        };
        if nibble % 2 == 0 {
            value[nibble / 2] = digit << 4;
        } else {
            value[nibble / 2] |= digit;
        }
        nibble += 1;
    }
    if value == [0; 16] {
        return Err(JournalError::InvalidState);
    }
    Ok(value)
}
fn current_kernel_boot() -> Result<[u8; 16], JournalError> {
    use std::os::unix::fs::OpenOptionsExt;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open("/proc/sys/kernel/random/boot_id")
        .map_err(|_| JournalError::Io)?;
    let mut bytes = Vec::with_capacity(38);
    file.take(38)
        .read_to_end(&mut bytes)
        .map_err(|_| JournalError::Io)?;
    parse_kernel_boot(&bytes)
}
impl PreviousBootAuthority {
    pub(crate) fn new(
        original_boot: [u8; 16],
        record_digest: [u8; 32],
    ) -> Result<Self, JournalError> {
        let current_boot = current_kernel_boot()?;
        if original_boot == [0; 16] || record_digest == [0; 32] || original_boot == current_boot {
            return Err(JournalError::InvalidState);
        }
        Ok(Self {
            original_boot,
            current_boot,
            record_digest,
            #[cfg(test)]
            synthetic_current: false,
        })
    }
    #[cfg(test)]
    pub(crate) fn for_test_boot(
        original_boot: [u8; 16],
        current_boot: [u8; 16],
        record_digest: [u8; 32],
    ) -> Result<Self, JournalError> {
        if original_boot == [0; 16]
            || current_boot == [0; 16]
            || record_digest == [0; 32]
            || original_boot == current_boot
        {
            return Err(JournalError::InvalidState);
        }
        Ok(Self {
            original_boot,
            current_boot,
            record_digest,
            synthetic_current: true,
        })
    }
    pub(crate) fn revalidate(&self) -> Result<(), JournalError> {
        #[cfg(test)]
        if self.synthetic_current {
            return Ok(());
        }
        if current_kernel_boot()? != self.current_boot || self.original_boot == self.current_boot {
            return Err(JournalError::InvalidState);
        }
        Ok(())
    }
}

/// Must be durably bound to the original owned recovery marker before it is reused.
/// Constructing this from current observations is not original ownership evidence.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct JournalAnchor {
    pub directory_dev: u64,
    pub directory_ino: u64,
    pub record_dev: u64,
    pub record_ino: u64,
}
impl fmt::Debug for JournalAnchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CookieReaderAnchor(<redacted>)")
    }
}
pub(crate) struct ReaderLaunch {
    pub capability: Capability,
    pub writer: File,
    pub lock: File,
}
pub(crate) struct ReaderJournal {
    directory: File,
    directory_id: Identity,
    record: File,
    record_id: Identity,
    receipts: [Option<(File, Identity)>; 2],
    state: ReaderRecords,
    poisoned: bool,
}
impl fmt::Debug for ReaderLaunch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CookieReaderLaunch(<redacted>)")
    }
}
impl fmt::Debug for ReaderJournal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CookieReaderJournal(<redacted>)")
    }
}
fn identity(file: &File, directory: bool) -> Result<Identity, JournalError> {
    let meta = file
        .metadata()
        .map_err(|_| JournalError::OwnershipChanged)?;
    // SAFETY: geteuid has no input or mutable output.
    let uid = unsafe { libc::geteuid() };
    if meta.uid() != uid
        || (meta.mode() & 0o7777) != if directory { 0o700 } else { 0o600 }
        || if directory {
            !meta.is_dir()
        } else {
            !meta.is_file() || meta.nlink() != 1
        }
    {
        return Err(JournalError::OwnershipChanged);
    }
    Ok(Identity {
        dev: meta.dev(),
        ino: meta.ino(),
    })
}
fn open(directory: &File, name: &CStr, flags: i32) -> Result<File, JournalError> {
    // SAFETY: anchored owned directory descriptor and fixed NUL-terminated basename.
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            flags | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600,
        )
    };
    if fd < 0 {
        return Err(JournalError::Io);
    }
    // SAFETY: fresh descriptor is adopted exactly once.
    Ok(unsafe { File::from_raw_fd(fd) })
}
fn bound(
    directory: &File,
    name: &CStr,
    file: &File,
    expected: Identity,
) -> Result<(), JournalError> {
    if identity(file, false)? != expected {
        return Err(JournalError::OwnershipChanged);
    }
    // SAFETY: fstatat writes one initialized stat; no symlink traversal.
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe {
        libc::fstatat(
            directory.as_raw_fd(),
            name.as_ptr(),
            &mut stat,
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
        || stat.st_dev != expected.dev
        || stat.st_ino != expected.ino
        || stat.st_mode & libc::S_IFMT != libc::S_IFREG
        || stat.st_nlink != 1
    {
        return Err(JournalError::OwnershipChanged);
    }
    Ok(())
}
impl ReaderJournal {
    pub fn create(directory: File) -> Result<Self, JournalError> {
        let directory_id = identity(&directory, true)?;
        let mut record = open(
            &directory,
            RECORD,
            libc::O_CREAT | libc::O_EXCL | libc::O_RDWR,
        )?;
        let record_id = identity(&record, false)?;
        // SAFETY: valid exclusively owned record descriptor. No blocking admission.
        if unsafe { libc::flock(record.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(JournalError::Busy);
        }
        let state = ReaderRecords::default();
        let bytes = state.encode().map_err(|_| JournalError::InvalidState)?;
        record
            .write_all(&bytes)
            .and_then(|_| record.sync_all())
            .and_then(|_| directory.sync_all())
            .map_err(|_| JournalError::Io)?;
        let journal = Self {
            directory,
            directory_id,
            record,
            record_id,
            receipts: [None, None],
            state,
            poisoned: false,
        };
        journal.validate()?;
        Ok(journal)
    }

    pub fn anchor(&self) -> JournalAnchor {
        JournalAnchor {
            directory_dev: self.directory_id.dev,
            directory_ino: self.directory_id.ino,
            record_dev: self.record_id.dev,
            record_ino: self.record_id.ino,
        }
    }
    /// No creation, truncation or cleanup occurs here. The expected anchor is mandatory.
    pub fn reopen(directory: File, expected: JournalAnchor) -> Result<Self, JournalError> {
        let directory_id = identity(&directory, true)?;
        if directory_id
            != (Identity {
                dev: expected.directory_dev,
                ino: expected.directory_ino,
            })
            || expected.record_ino == 0
        {
            return Err(JournalError::OwnershipChanged);
        }
        let record = open(&directory, RECORD, libc::O_RDWR)?;
        let record_id = Identity {
            dev: expected.record_dev,
            ino: expected.record_ino,
        };
        bound(&directory, RECORD, &record, record_id)?;
        // SAFETY: separately opened, validated record; never blocks a live owner.
        if unsafe { libc::flock(record.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(JournalError::Busy);
        }
        let length = record
            .metadata()
            .map_err(|_| JournalError::OwnershipChanged)?
            .len();
        if length == 0 || length > MAX_RECORD_BYTES as u64 {
            return Err(JournalError::InvalidState);
        }
        let mut bytes = vec![0; length as usize];
        record
            .read_exact_at(&mut bytes, 0)
            .map_err(|_| JournalError::Io)?;
        bound(&directory, RECORD, &record, record_id)?;
        let fields: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| JournalError::InvalidState)?;
        let slots = fields
            .get("slots")
            .and_then(serde_json::Value::as_array)
            .filter(|slots| slots.len() == 2)
            .ok_or(JournalError::InvalidState)?;
        let mut receipts: [Option<(File, Identity)>; 2] = [None, None];
        let mut proofs = [None, None];
        for index in 0..2 {
            let phase = slots[index]
                .get("phase")
                .and_then(serde_json::Value::as_u64)
                .ok_or(JournalError::InvalidState)?;
            if phase == 0 {
                continue;
            }
            let capability = slots[index]
                .get("capability")
                .ok_or(JournalError::InvalidState)?;
            let id = Identity {
                dev: capability
                    .get("receipt_dev")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(JournalError::InvalidState)?,
                ino: capability
                    .get("receipt_ino")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or(JournalError::InvalidState)?,
            };
            let file = open(&directory, RECEIPTS[index], libc::O_RDONLY)?;
            bound(&directory, RECEIPTS[index], &file, id)?;
            let length = file
                .metadata()
                .map_err(|_| JournalError::OwnershipChanged)?
                .len();
            if length > RECEIPT_SIZE as u64 {
                return Err(JournalError::InvalidState);
            }
            if length == RECEIPT_SIZE as u64 {
                let mut receipt = [0; RECEIPT_SIZE];
                file.read_exact_at(&mut receipt, 0)
                    .map_err(|_| JournalError::Io)?;
                bound(&directory, RECEIPTS[index], &file, id)?;
                proofs[index] = decode_cleanup_receipt(&receipt).ok();
            }
            receipts[index] = Some((file, id));
        }
        let state = ReaderRecords::decode(&bytes, [proofs[0].as_ref(), proofs[1].as_ref()])
            .map_err(|_| JournalError::InvalidState)?;
        let journal = Self {
            directory,
            directory_id,
            record,
            record_id,
            receipts,
            state,
            poisoned: false,
        };
        // Reject concurrent mutation/noncanonical record bytes without changing them.
        journal.validate()?;
        Ok(journal)
    }
    fn validate_identity(&self) -> Result<(), JournalError> {
        if self.poisoned {
            return Err(JournalError::Poisoned);
        }
        if identity(&self.directory, true)? != self.directory_id {
            return Err(JournalError::OwnershipChanged);
        }
        bound(&self.directory, RECORD, &self.record, self.record_id)?;
        for (index, receipt) in self.receipts.iter().enumerate() {
            if let Some((file, id)) = receipt {
                bound(&self.directory, RECEIPTS[index], file, *id)?;
            }
        }
        Ok(())
    }
    fn record_matches(&self, state: &ReaderRecords) -> Result<(), JournalError> {
        let expected = state.encode().map_err(|_| JournalError::InvalidState)?;
        let length = self
            .record
            .metadata()
            .map_err(|_| JournalError::OwnershipChanged)?
            .len();
        if length > MAX_RECORD_BYTES as u64 || length != expected.len() as u64 {
            return Err(JournalError::OwnershipChanged);
        }
        let mut actual = vec![0; expected.len()];
        self.record
            .read_exact_at(&mut actual, 0)
            .map_err(|_| JournalError::OwnershipChanged)?;
        if actual != expected {
            return Err(JournalError::OwnershipChanged);
        }
        Ok(())
    }
    fn validate_resolved_proofs(&self, state: &ReaderRecords) -> Result<(), JournalError> {
        for (index, name) in RECEIPTS.iter().enumerate() {
            if let Some(Slot::Resolved(identity)) = state.slot(index) {
                let (file, id) = self.receipts[index]
                    .as_ref()
                    .ok_or(JournalError::InvalidState)?;
                bound(&self.directory, name, file, *id)?;
                if file.metadata().map_err(|_| JournalError::Io)?.len() != RECEIPT_SIZE as u64 {
                    return Err(JournalError::InvalidState);
                }
                let mut bytes = [0; RECEIPT_SIZE];
                file.read_exact_at(&mut bytes, 0)
                    .map_err(|_| JournalError::Io)?;
                let proof =
                    decode_cleanup_receipt(&bytes).map_err(|_| JournalError::InvalidState)?;
                proof
                    .matches(identity)
                    .map_err(|_| JournalError::InvalidState)?;
                bound(&self.directory, name, file, *id)?;
                if file.metadata().map_err(|_| JournalError::Io)?.len() != RECEIPT_SIZE as u64 {
                    return Err(JournalError::InvalidState);
                }
            }
        }
        Ok(())
    }
    fn validate(&self) -> Result<(), JournalError> {
        self.validate_identity()?;
        self.record_matches(&self.state)?;
        self.validate_resolved_proofs(&self.state)
    }
    fn persist(&mut self, state: ReaderRecords) -> Result<(), JournalError> {
        self.persist_with(state, |file, bytes| {
            file.set_len(0)?;
            file.seek(SeekFrom::Start(0))?;
            file.write_all(bytes)
        })
    }
    fn persist_with(
        &mut self,
        state: ReaderRecords,
        write: impl FnOnce(&mut File, &[u8]) -> io::Result<()>,
    ) -> Result<(), JournalError> {
        self.validate()?;
        let bytes = state.encode().map_err(|_| JournalError::InvalidState)?;
        if bytes.len() > MAX_RECORD_BYTES {
            return Err(JournalError::InvalidState);
        }
        let result = write(&mut self.record, &bytes)
            .and_then(|_| self.record.sync_all())
            .and_then(|_| self.directory.sync_all());
        if result.is_err() {
            self.poisoned = true;
            return Err(JournalError::Io);
        }
        if self
            .validate_identity()
            .and_then(|_| self.record_matches(&state))
            .and_then(|_| self.validate_resolved_proofs(&state))
            .is_err()
        {
            self.poisoned = true;
            return Err(JournalError::OwnershipChanged);
        }
        self.state = state;
        Ok(())
    }
    pub fn begin(
        &mut self,
        index: u8,
        nonce: [u8; 16],
        digest: [u8; 32],
        boot: [u8; 16],
    ) -> Result<ReaderLaunch, JournalError> {
        self.validate()?;
        if index > 1 {
            return Err(JournalError::InvalidState);
        }
        // Validate the transition before creating files. Placeholder inode is replaced below.
        let mut candidate = self.state.clone();
        let mut capability = Capability {
            index,
            nonce,
            digest,
            boot,
            receipt_dev: 0,
            receipt_ino: 1,
        };
        candidate
            .begin(capability.clone())
            .map_err(|_| JournalError::InvalidState)?;
        let writer = open(
            &self.directory,
            RECEIPTS[index as usize],
            libc::O_CREAT | libc::O_EXCL | libc::O_WRONLY,
        )?;
        let id = identity(&writer, false)?;
        bound(&self.directory, RECEIPTS[index as usize], &writer, id)?;
        writer.sync_all().map_err(|_| JournalError::Io)?;
        let reader = open(&self.directory, RECEIPTS[index as usize], libc::O_RDONLY)?;
        bound(&self.directory, RECEIPTS[index as usize], &reader, id)?;
        self.receipts[index as usize] = Some((reader, id));
        capability.receipt_dev = id.dev;
        capability.receipt_ino = id.ino;
        candidate = self.state.clone();
        candidate
            .begin(capability.clone())
            .map_err(|_| JournalError::InvalidState)?;
        self.persist(candidate)?;
        self.validate()?;
        let lock = self.record.try_clone().map_err(|_| JournalError::Io)?;
        Ok(ReaderLaunch {
            capability,
            writer,
            lock,
        })
    }
    pub fn bind(&mut self, identity: ReaderIdentity) -> Result<(), JournalError> {
        self.validate()?;
        let mut candidate = self.state.clone();
        candidate
            .bind(identity)
            .map_err(|_| JournalError::InvalidState)?;
        self.persist(candidate)
    }
    pub fn resolve(&mut self, index: u8) -> Result<(), JournalError> {
        self.validate()?;
        if index > 1 {
            return Err(JournalError::InvalidState);
        }
        let (file, id) = self.receipts[index as usize]
            .as_mut()
            .ok_or(JournalError::InvalidState)?;
        bound(&self.directory, RECEIPTS[index as usize], file, *id)?;
        if file.metadata().map_err(|_| JournalError::Io)?.len() != RECEIPT_SIZE as u64 {
            return Err(JournalError::InvalidState);
        }
        let mut bytes = [0u8; RECEIPT_SIZE];
        file.seek(SeekFrom::Start(0))
            .and_then(|_| file.read_exact(&mut bytes))
            .map_err(|_| JournalError::Io)?;
        bound(&self.directory, RECEIPTS[index as usize], file, *id)?;
        if file.metadata().map_err(|_| JournalError::Io)?.len() != RECEIPT_SIZE as u64 {
            return Err(JournalError::InvalidState);
        }
        let receipt = decode_cleanup_receipt(&bytes).map_err(|_| JournalError::InvalidState)?;
        let mut candidate = self.state.clone();
        candidate
            .resolve(&receipt)
            .map_err(|_| JournalError::InvalidState)?;
        self.persist(candidate)
    }
    /// Resolve only original bound readers with actual durable terminal receipts.
    /// Empty journals and intents are never same-boot deletion evidence.
    pub(crate) fn recover_completed_readers(&mut self) -> Result<bool, JournalError> {
        self.validate_identity()?;
        self.record_matches(&self.state)?;
        let mut original_reader = false;
        for index in 0..2 {
            match self.state.slot(index) {
                Some(Slot::Intent(_)) => return Ok(false),
                Some(Slot::Bound(_) | Slot::Resolved(_)) => original_reader = true,
                Some(Slot::Empty) => {}
                None => return Err(JournalError::InvalidState),
            }
        }
        if !original_reader {
            return Ok(false);
        }
        match self.validate_resolved_proofs(&self.state) {
            Ok(()) => {}
            Err(JournalError::InvalidState) => return Ok(false),
            Err(error) => return Err(error),
        }
        // Preflight all proofs before changing any state. Native ownership errors
        // stay errors; incomplete or mismatching original proof bytes stay pending.
        for (index, name) in RECEIPTS.iter().enumerate() {
            if let Some(Slot::Bound(identity)) = self.state.slot(index) {
                let (file, id) = self.receipts[index]
                    .as_ref()
                    .ok_or(JournalError::InvalidState)?;
                bound(&self.directory, name, file, *id)?;
                if file.metadata().map_err(|_| JournalError::Io)?.len() != RECEIPT_SIZE as u64 {
                    return Ok(false);
                }
                let mut bytes = [0; RECEIPT_SIZE];
                file.read_exact_at(&mut bytes, 0)
                    .map_err(|_| JournalError::Io)?;
                bound(&self.directory, name, file, *id)?;
                if file.metadata().map_err(|_| JournalError::Io)?.len() != RECEIPT_SIZE as u64 {
                    return Ok(false);
                }
                let Ok(receipt) = decode_cleanup_receipt(&bytes) else {
                    return Ok(false);
                };
                if receipt.matches(identity).is_err() {
                    return Ok(false);
                }
            }
        }
        for index in 0..2 {
            if matches!(self.state.slot(index), Some(Slot::Bound(_))) {
                match self.resolve(index as u8) {
                    Ok(()) => {}
                    Err(JournalError::InvalidState) => return Ok(false),
                    Err(error) => return Err(error),
                }
            }
        }
        self.validate()?;
        Ok((0..2).all(|index| {
            matches!(
                self.state.slot(index),
                Some(Slot::Empty | Slot::Resolved(_))
            )
        }))
    }
    /// Complete owner-entry allowlisting and cookie/recovery marker validation are caller-owned.
    pub fn validate_owned_entries(&self) -> Result<(), JournalError> {
        self.validate()?;
        for index in 0..2 {
            let active = !matches!(self.state.slot(index), Some(Slot::Empty));
            if active != self.receipts[index].is_some() {
                return Err(JournalError::InvalidState);
            }
        }
        Ok(())
    }
    pub fn owned_names(&self) -> Result<Vec<&'static CStr>, JournalError> {
        self.validate_owned_entries()?;
        let mut names = vec![RECORD];
        for (index, receipt) in self.receipts.iter().enumerate() {
            if receipt.is_some() {
                names.push(RECEIPTS[index]);
            }
        }
        Ok(names)
    }
    fn validate_previous_capabilities(
        &self,
        authority: &PreviousBootAuthority,
    ) -> Result<(), JournalError> {
        authority.revalidate()?;
        for index in 0..2 {
            let capability = match self.state.slot(index) {
                Some(Slot::Empty) => continue,
                Some(Slot::Intent(capability)) => capability,
                Some(Slot::Bound(identity) | Slot::Resolved(identity)) => &identity.capability,
                None => return Err(JournalError::InvalidState),
            };
            crate::process::reader_protocol::encode_capability(capability)
                .map_err(|_| JournalError::InvalidState)?;
            if usize::from(capability.index) != index
                || capability.boot != authority.original_boot
                || capability.digest != authority.record_digest
            {
                return Err(JournalError::InvalidState);
            }
        }
        Ok(())
    }
    pub(crate) fn validate_previous_boot(
        &self,
        authority: &PreviousBootAuthority,
    ) -> Result<(), JournalError> {
        self.validate_owned_entries()?;
        self.validate_previous_capabilities(authority)?;
        for receipt in self.receipts.iter().flatten() {
            if receipt.0.metadata().map_err(|_| JournalError::Io)?.len() > RECEIPT_SIZE as u64 {
                return Err(JournalError::InvalidState);
            }
        }
        Ok(())
    }
    fn validate_remaining_for_removal(
        &self,
        removed: [bool; 2],
        authority: Option<&PreviousBootAuthority>,
    ) -> Result<(), JournalError> {
        if let Some(authority) = authority {
            self.validate_previous_capabilities(authority)?;
        }

        if identity(&self.directory, true)? != self.directory_id {
            return Err(JournalError::OwnershipChanged);
        }
        bound(&self.directory, RECORD, &self.record, self.record_id)?;
        self.record_matches(&self.state)?;
        for (index, receipt) in self.receipts.iter().enumerate() {
            if removed[index] {
                continue;
            }
            if let Some((file, id)) = receipt {
                bound(&self.directory, RECEIPTS[index], file, *id)?;
                let Some(Slot::Resolved(identity)) = self.state.slot(index) else {
                    if authority.is_some()
                        && matches!(
                            self.state.slot(index),
                            Some(Slot::Intent(_) | Slot::Bound(_))
                        )
                    {
                        if file.metadata().map_err(|_| JournalError::Io)?.len()
                            > RECEIPT_SIZE as u64
                        {
                            return Err(JournalError::InvalidState);
                        }
                        continue;
                    }
                    return Err(JournalError::InvalidState);
                };
                if file.metadata().map_err(|_| JournalError::Io)?.len() != RECEIPT_SIZE as u64 {
                    return Err(JournalError::InvalidState);
                }
                let mut bytes = [0; RECEIPT_SIZE];
                file.read_exact_at(&mut bytes, 0)
                    .map_err(|_| JournalError::Io)?;
                decode_cleanup_receipt(&bytes)
                    .map_err(|_| JournalError::InvalidState)?
                    .matches(identity)
                    .map_err(|_| JournalError::InvalidState)?;
                bound(&self.directory, RECEIPTS[index], file, *id)?;
                if file.metadata().map_err(|_| JournalError::Io)?.len() != RECEIPT_SIZE as u64 {
                    return Err(JournalError::InvalidState);
                }
            }
        }
        Ok(())
    }
    /// Deletes only this journal's fixed, bound files. Partial failure is not transactional.
    /// Caller must first validate the entire owner allowlist and original cookie/marker evidence.
    pub fn remove_owned_files(self) -> Result<(), JournalError> {
        self.validate_owned_entries()?;
        if !self.state.cleanup_eligible() {
            return Err(JournalError::InvalidState);
        }
        self.remove_files(None)
    }
    pub(crate) fn remove_owned_files_previous_boot(
        self,
        authority: &PreviousBootAuthority,
    ) -> Result<(), JournalError> {
        self.validate_previous_boot(authority)?;
        self.remove_files(Some(authority))
    }
    fn remove_files(self, authority: Option<&PreviousBootAuthority>) -> Result<(), JournalError> {
        let mut removed = [false; 2];
        for (index, receipt) in self.receipts.iter().enumerate() {
            if receipt.is_none() {
                continue;
            }
            self.validate_remaining_for_removal(removed, authority)?;
            // SAFETY: fixed basename has just been bound to this owned receipt handle.
            if unsafe { libc::unlinkat(self.directory.as_raw_fd(), RECEIPTS[index].as_ptr(), 0) }
                != 0
            {
                return Err(JournalError::Io);
            }
            removed[index] = true;
        }
        self.validate_remaining_for_removal(removed, authority)?;
        // SAFETY: exact original record ID and fixed basename; never a directory deletion.
        if unsafe { libc::unlinkat(self.directory.as_raw_fd(), RECORD.as_ptr(), 0) } != 0 {
            return Err(JournalError::Io);
        }
        if identity(&self.directory, true)? != self.directory_id {
            return Err(JournalError::OwnershipChanged);
        }
        self.directory.sync_all().map_err(|_| JournalError::Io)
    }
    /// Caller still must hold the live-owner lock and validate the complete OwnedCopy.
    pub fn cleanup_eligible(&self) -> Result<bool, JournalError> {
        self.validate()?;
        Ok(self.state.cleanup_eligible())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cookies::reader_records::identity_digest,
        process::reader_protocol::{CleanupReceipt, encode_cleanup_receipt},
    };
    use std::os::unix::fs::PermissionsExt;
    fn reopen_with_diagnostic(
        directory: File,
        anchor: JournalAnchor,
    ) -> Result<ReaderJournal, JournalError> {
        ReaderJournal::reopen(directory, anchor).inspect_err(|error| {
            let kind = match error {
                JournalError::OwnershipChanged => "OwnershipChanged",
                JournalError::Busy => "Busy",
                JournalError::InvalidState => "InvalidState",
                JournalError::Io => "Io",
                JournalError::Poisoned => "Poisoned",
            };
            eprintln!("owned journal reopen test error kind={kind}");
        })
    }
    fn fixture() -> (tempfile::TempDir, ReaderJournal) {
        let folder = tempfile::tempdir().unwrap();
        std::fs::set_permissions(folder.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let journal = ReaderJournal::create(File::open(folder.path()).unwrap()).unwrap();
        (folder, journal)
    }
    fn reader(capability: Capability) -> ReaderIdentity {
        let mut value = ReaderIdentity {
            capability,
            identity_digest: [0; 32],
            unit: [4; 16],
            invocation: [5; 16],
            cgroup_mount: 6,
            cgroup_dev: 7,
            cgroup_ino: 8,
            path: "/user.slice/owned.service".into(),
        };
        value.identity_digest = identity_digest(&value).unwrap();
        value
    }
    #[test]
    fn recovery_actual_bound_receipt_resolves_and_survives_reopen() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        let (folder, mut journal) = fixture();
        let mut launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
        let value = reader(launch.capability.clone());
        journal.bind(value.clone()).unwrap();
        let anchor = journal.anchor();
        launch
            .writer
            .write_all(&encode_cleanup_receipt(&CleanupReceipt::from_identity(&value)).unwrap())
            .unwrap();
        launch.writer.sync_all().unwrap();
        assert!(journal.recover_completed_readers().unwrap());
        assert!(matches!(journal.state.slot(0), Some(Slot::Resolved(_))));
        drop(launch);
        drop(journal);
        let mut reopened =
            reopen_with_diagnostic(File::open(folder.path()).unwrap(), anchor).unwrap();
        assert!(reopened.recover_completed_readers().unwrap());
    }
    #[test]
    fn recovery_empty_and_intent_never_become_sameboot_proof() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        let (folder, mut journal) = fixture();
        assert!(!journal.recover_completed_readers().unwrap());
        let mut launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
        let value = reader(launch.capability.clone());
        launch
            .writer
            .write_all(&encode_cleanup_receipt(&CleanupReceipt::from_identity(&value)).unwrap())
            .unwrap();
        launch.writer.sync_all().unwrap();
        let before = std::fs::read(folder.path().join("recordstate.json")).unwrap();
        assert!(!journal.recover_completed_readers().unwrap());
        assert_eq!(
            std::fs::read(folder.path().join("recordstate.json")).unwrap(),
            before
        );
    }
    #[test]
    fn recovery_partial_mismatch_and_replaced_receipts_preserve_bytes() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        for case in ["zero", "partial", "mismatch", "replace"] {
            let (folder, mut journal) = fixture();
            let mut launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
            let value = reader(launch.capability.clone());
            journal.bind(value.clone()).unwrap();
            match case {
                "partial" => launch.writer.write_all(b"partial").unwrap(),
                "mismatch" => {
                    let mut receipt = CleanupReceipt::from_identity(&value);
                    receipt.invocation = [9; 16];
                    launch
                        .writer
                        .write_all(&encode_cleanup_receipt(&receipt).unwrap())
                        .unwrap();
                }
                "replace" => {
                    std::fs::rename(
                        folder.path().join("reader0.receipt"),
                        folder.path().join("original"),
                    )
                    .unwrap();
                    std::fs::write(folder.path().join("reader0.receipt"), b"foreign sentinel")
                        .unwrap();
                }
                _ => {}
            }
            let record = std::fs::read(folder.path().join("recordstate.json")).unwrap();
            let receipt = std::fs::read(folder.path().join("reader0.receipt")).unwrap();
            if case == "replace" {
                assert!(journal.recover_completed_readers().is_err());
            } else {
                assert!(!journal.recover_completed_readers().unwrap());
            }
            assert_eq!(
                std::fs::read(folder.path().join("recordstate.json")).unwrap(),
                record
            );
            assert_eq!(
                std::fs::read(folder.path().join("reader0.receipt")).unwrap(),
                receipt
            );
        }
    }
    #[test]
    fn previous_boot_authority_validates_kernel_and_bounded_uuid() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        let current = current_kernel_boot().unwrap();
        assert!(PreviousBootAuthority::new(current, [2; 32]).is_err());
        assert!(PreviousBootAuthority::new([0; 16], [2; 32]).is_err());
        assert!(PreviousBootAuthority::new([1; 16], [0; 32]).is_err());
        let mut original = current;
        original[0] ^= 1;
        let mut authority = PreviousBootAuthority::new(original, [2; 32]).unwrap();
        authority.revalidate().unwrap();
        authority.current_boot[0] ^= 1;
        assert!(authority.revalidate().is_err());
        assert!(PreviousBootAuthority::for_test_boot([1; 16], [1; 16], [2; 32]).is_err());
        for bytes in [
            b"".as_slice(),
            b"00000000-0000-0000-0000-000000000000\n",
            b"11111111_1111-1111-1111-111111111111\n",
            b"11111111-1111-1111-1111-111111111111\nx",
        ] {
            assert!(parse_kernel_boot(bytes).is_err());
        }
        assert_eq!(
            parse_kernel_boot(b"11111111-1111-1111-1111-111111111111\n").unwrap(),
            [0x11; 16]
        );
    }
    #[test]
    fn previous_boot_removes_original_intent_bound_and_mixed_resolved_slots() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        let authority = PreviousBootAuthority::for_test_boot([3; 16], [9; 16], [2; 32]).unwrap();
        for phase in ["empty", "intent", "bound", "mixed"] {
            let (folder, mut journal) = fixture();
            if phase != "empty" {
                let mut launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
                let value = reader(launch.capability.clone());
                if phase != "intent" {
                    journal.bind(value.clone()).unwrap();
                }
                if phase == "mixed" {
                    launch
                        .writer
                        .write_all(
                            &encode_cleanup_receipt(&CleanupReceipt::from_identity(&value))
                                .unwrap(),
                        )
                        .unwrap();
                    launch.writer.sync_all().unwrap();
                    journal.resolve(0).unwrap();
                    let second = journal.begin(1, [4; 16], [2; 32], [3; 16]).unwrap();
                    journal.bind(reader(second.capability.clone())).unwrap();
                    drop(second);
                }
                drop(launch);
            }
            journal.validate_previous_boot(&authority).unwrap();
            journal
                .remove_owned_files_previous_boot(&authority)
                .unwrap();
            assert!(!folder.path().join("recordstate.json").exists());
            assert!(!folder.path().join("reader0.receipt").exists());
            assert!(!folder.path().join("reader1.receipt").exists());
        }
    }
    #[test]
    fn previous_boot_mismatch_changed_entry_and_resolved_tamper_preserve_files() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        for case in ["boot", "digest", "replace", "resolved"] {
            let (folder, mut journal) = fixture();
            let mut launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
            if case == "resolved" {
                let value = reader(launch.capability.clone());
                journal.bind(value.clone()).unwrap();
                launch
                    .writer
                    .write_all(
                        &encode_cleanup_receipt(&CleanupReceipt::from_identity(&value)).unwrap(),
                    )
                    .unwrap();
                launch.writer.sync_all().unwrap();
                journal.resolve(0).unwrap();
                launch.writer.set_len(0).unwrap();
            }
            if case == "replace" {
                std::fs::rename(
                    folder.path().join("reader0.receipt"),
                    folder.path().join("original"),
                )
                .unwrap();
                std::fs::write(folder.path().join("reader0.receipt"), b"foreign sentinel").unwrap();
            }
            let authority = PreviousBootAuthority::for_test_boot(
                if case == "boot" { [4; 16] } else { [3; 16] },
                [9; 16],
                if case == "digest" { [5; 32] } else { [2; 32] },
            )
            .unwrap();
            let record = std::fs::read(folder.path().join("recordstate.json")).unwrap();
            let proof = std::fs::read(folder.path().join("reader0.receipt")).unwrap();
            assert!(
                journal
                    .remove_owned_files_previous_boot(&authority)
                    .is_err()
            );
            assert_eq!(
                std::fs::read(folder.path().join("recordstate.json")).unwrap(),
                record
            );
            assert_eq!(
                std::fs::read(folder.path().join("reader0.receipt")).unwrap(),
                proof
            );
        }
    }
    #[test]
    fn cleanup_empty_and_resolved_only_removes_exact_journal_names() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        let (folder, journal) = fixture();
        std::fs::write(folder.path().join("foreign"), b"sentinel").unwrap();
        assert_eq!(journal.owned_names().unwrap(), vec![RECORD]);
        journal.remove_owned_files().unwrap();
        assert!(!folder.path().join("recordstate.json").exists());
        assert_eq!(
            std::fs::read(folder.path().join("foreign")).unwrap(),
            b"sentinel"
        );
        assert!(folder.path().is_dir());
        let (folder, mut journal) = fixture();
        let mut launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
        let value = reader(launch.capability.clone());
        journal.bind(value.clone()).unwrap();
        launch
            .writer
            .write_all(&encode_cleanup_receipt(&CleanupReceipt::from_identity(&value)).unwrap())
            .unwrap();
        launch.writer.sync_all().unwrap();
        journal.resolve(0).unwrap();
        drop(launch);
        assert_eq!(journal.owned_names().unwrap(), vec![RECORD, RECEIPTS[0]]);
        journal.remove_owned_files().unwrap();
        assert!(!folder.path().join("recordstate.json").exists());
        assert!(!folder.path().join("reader0.receipt").exists());
    }
    #[test]
    fn cleanup_pending_replaced_or_tampered_preserves_original_bytes() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        let (folder, mut journal) = fixture();
        let launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
        drop(launch);
        let before = std::fs::read(folder.path().join("recordstate.json")).unwrap();
        assert_eq!(
            journal.remove_owned_files(),
            Err(JournalError::InvalidState)
        );
        assert_eq!(
            std::fs::read(folder.path().join("recordstate.json")).unwrap(),
            before
        );
        assert!(folder.path().join("reader0.receipt").exists());
        let (folder, journal) = fixture();
        let path = folder.path().join("recordstate.json");
        std::fs::rename(&path, folder.path().join("oldrecord")).unwrap();
        std::fs::write(&path, b"foreign").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            journal.remove_owned_files(),
            Err(JournalError::OwnershipChanged)
        );
        assert_eq!(std::fs::read(path).unwrap(), b"foreign");
        let (folder, mut journal) = fixture();
        let mut launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
        let value = reader(launch.capability.clone());
        journal.bind(value.clone()).unwrap();
        launch
            .writer
            .write_all(&encode_cleanup_receipt(&CleanupReceipt::from_identity(&value)).unwrap())
            .unwrap();
        journal.resolve(0).unwrap();
        let before = std::fs::read(folder.path().join("recordstate.json")).unwrap();
        launch.writer.set_len(0).unwrap();
        assert_eq!(
            journal.remove_owned_files(),
            Err(JournalError::InvalidState)
        );
        assert_eq!(
            std::fs::read(folder.path().join("recordstate.json")).unwrap(),
            before
        );
        assert_eq!(
            std::fs::metadata(folder.path().join("reader0.receipt"))
                .unwrap()
                .len(),
            0
        );
    }
    #[test]
    fn reopen_preserves_pending_and_requires_independent_resolved_proof() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        let (folder, mut journal) = fixture();
        let mut launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
        let value = reader(launch.capability.clone());
        journal.bind(value.clone()).unwrap();
        let anchor = journal.anchor();
        drop(journal);
        drop(launch.lock);
        let mut reopened =
            reopen_with_diagnostic(File::open(folder.path()).unwrap(), anchor).unwrap();
        assert!(!reopened.cleanup_eligible().unwrap());
        let receipt = encode_cleanup_receipt(&CleanupReceipt::from_identity(&value)).unwrap();
        launch.writer.write_all(&receipt).unwrap();
        launch.writer.sync_all().unwrap();
        reopened.resolve(0).unwrap();
        drop(reopened);
        let reopened = reopen_with_diagnostic(File::open(folder.path()).unwrap(), anchor).unwrap();
        assert!(reopened.cleanup_eligible().unwrap());
        launch.writer.set_len(0).unwrap();
        assert_eq!(reopened.cleanup_eligible(), Err(JournalError::InvalidState));
        drop(reopened);
        assert!(reopen_with_diagnostic(File::open(folder.path()).unwrap(), anchor).is_err());
        assert_eq!(
            std::fs::metadata(folder.path().join("reader0.receipt"))
                .unwrap()
                .len(),
            0
        );
    }
    #[test]
    fn reopen_intent_and_busy_lock_never_mutate_existing_files() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        let (folder, mut journal) = fixture();
        let launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
        let anchor = journal.anchor();
        let before = std::fs::read(folder.path().join("recordstate.json")).unwrap();
        assert!(matches!(
            reopen_with_diagnostic(File::open(folder.path()).unwrap(), anchor),
            Err(JournalError::Busy)
        ));
        drop(journal);
        assert!(matches!(
            reopen_with_diagnostic(File::open(folder.path()).unwrap(), anchor),
            Err(JournalError::Busy)
        ));
        drop(launch);
        let reopened = reopen_with_diagnostic(File::open(folder.path()).unwrap(), anchor).unwrap();
        assert!(!reopened.cleanup_eligible().unwrap());
        drop(reopened);
        assert_eq!(
            std::fs::read(folder.path().join("recordstate.json")).unwrap(),
            before
        );
    }
    #[test]
    fn reopen_rejects_replaced_record_receipt_and_wrong_anchor() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        let (folder, mut journal) = fixture();
        let launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
        let anchor = journal.anchor();
        drop(launch);
        drop(journal);
        let mut wrong = anchor;
        wrong.directory_ino += 1;
        assert!(reopen_with_diagnostic(File::open(folder.path()).unwrap(), wrong).is_err());
        let path = folder.path().join("reader0.receipt");
        std::fs::rename(&path, folder.path().join("old-receipt")).unwrap();
        std::fs::write(&path, b"foreign").unwrap();
        assert!(reopen_with_diagnostic(File::open(folder.path()).unwrap(), anchor).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"foreign");
        let path = folder.path().join("recordstate.json");
        let before = std::fs::read(&path).unwrap();
        std::fs::rename(&path, folder.path().join("old-record")).unwrap();
        std::fs::write(&path, &before).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(matches!(
            reopen_with_diagnostic(File::open(folder.path()).unwrap(), anchor),
            Err(JournalError::OwnershipChanged)
        ));
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
    #[test]
    fn reopen_invalid_or_oversized_record_is_preserved() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        for bytes in [b"{".to_vec(), vec![b'x'; MAX_RECORD_BYTES + 1]] {
            let (folder, journal) = fixture();
            let anchor = journal.anchor();
            drop(journal);
            let path = folder.path().join("recordstate.json");
            std::fs::write(&path, &bytes).unwrap();
            assert!(reopen_with_diagnostic(File::open(folder.path()).unwrap(), anchor).is_err());
            assert_eq!(std::fs::read(path).unwrap(), bytes);
        }
    }
    #[test]
    fn actual_intent_bind_receipt_sync_and_record_lock() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        let (folder, mut journal) = fixture();
        let mut launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
        assert!(!journal.cleanup_eligible().unwrap());
        let value = reader(launch.capability.clone());
        journal.bind(value.clone()).unwrap();
        let bytes = encode_cleanup_receipt(&CleanupReceipt::from_identity(&value)).unwrap();
        launch.writer.write_all(&bytes).unwrap();
        launch.writer.sync_all().unwrap();
        journal.resolve(0).unwrap();
        assert!(journal.cleanup_eligible().unwrap());
        let other = File::open(folder.path().join("recordstate.json")).unwrap();
        assert_ne!(
            unsafe { libc::flock(other.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
            0
        );
        drop(journal);
        assert_ne!(
            unsafe { libc::flock(other.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
            0
        );
        drop(launch.lock);
        assert_eq!(
            unsafe { libc::flock(other.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
            0
        );
        assert!(folder.path().join("reader0.receipt").is_file());
    }
    #[test]
    fn partial_commit_error_poison_preserves_files_and_memory() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        let (folder, mut journal) = fixture();
        let _launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
        let before = journal.state.clone();
        let result = journal.persist_with(before.clone(), |file, _| {
            file.set_len(0)?;
            file.seek(SeekFrom::Start(0))?;
            file.write_all(b"{")?;
            Err(io::Error::other("synthetic writer failure"))
        });
        assert_eq!(result, Err(JournalError::Io));
        assert_eq!(journal.state, before);
        assert_eq!(journal.cleanup_eligible(), Err(JournalError::Poisoned));
        drop(journal);
        assert_eq!(
            std::fs::read(folder.path().join("recordstate.json")).unwrap(),
            b"{"
        );
    }
    #[test]
    fn replaced_receipt_and_foreign_unknown_entries_are_preserved() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        let (folder, mut journal) = fixture();
        let _launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
        let path = folder.path().join("reader0.receipt");
        std::fs::rename(&path, folder.path().join("old-receipt")).unwrap();
        std::fs::write(&path, b"foreign").unwrap();
        std::fs::write(folder.path().join("unknown"), b"sentinel").unwrap();
        assert_eq!(journal.resolve(0), Err(JournalError::OwnershipChanged));
        drop(journal);
        assert_eq!(std::fs::read(path).unwrap(), b"foreign");
        assert_eq!(
            std::fs::read(folder.path().join("unknown")).unwrap(),
            b"sentinel"
        );
    }
    #[test]
    fn oversized_or_modified_record_preserves_anchored_files() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        let (folder, journal) = fixture();
        let path = folder.path().join("recordstate.json");
        std::fs::write(&path, vec![b'x'; MAX_RECORD_BYTES + 1]).unwrap();
        assert_eq!(
            journal.cleanup_eligible(),
            Err(JournalError::OwnershipChanged)
        );
        drop(journal);
        assert_eq!(
            std::fs::metadata(path).unwrap().len(),
            (MAX_RECORD_BYTES + 1) as u64
        );
    }
    #[test]
    fn partial_zero_oversize_and_wrong_receipt_never_resolve() {
        if crate::process::reader_protocol::run_isolated_lock_test() {
            return;
        }
        let (_folder, mut journal) = fixture();
        let mut launch = journal.begin(0, [1; 16], [2; 32], [3; 16]).unwrap();
        let value = reader(launch.capability.clone());
        journal.bind(value.clone()).unwrap();
        for bytes in [vec![0; 1], vec![0; RECEIPT_SIZE], vec![0; RECEIPT_SIZE + 1]] {
            launch.writer.set_len(0).unwrap();
            launch.writer.seek(SeekFrom::Start(0)).unwrap();
            launch.writer.write_all(&bytes).unwrap();
            assert_eq!(journal.resolve(0), Err(JournalError::InvalidState));
            assert!(!journal.cleanup_eligible().unwrap());
        }
        let mut receipt = CleanupReceipt::from_identity(&value);
        receipt.invocation = [9; 16];
        launch.writer.set_len(0).unwrap();
        launch.writer.seek(SeekFrom::Start(0)).unwrap();
        launch
            .writer
            .write_all(&encode_cleanup_receipt(&receipt).unwrap())
            .unwrap();
        assert_eq!(journal.resolve(0), Err(JournalError::InvalidState));
    }
}
