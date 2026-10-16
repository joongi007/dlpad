//! Credential-free bounded gate frames. Native ownership and durability are caller contracts.
use std::fmt;

pub(crate) const MAX_PACKET: usize = 1024;
pub(crate) const MAX_PATH: usize = 512;
pub(crate) const RECEIPT_SIZE: usize = 256;
const HEADER: usize = 8;
const MAGIC: &[u8; 4] = b"DLPG";
const VERSION: u8 = 1;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum ProtocolError {
    InvalidFrame,
    UnsupportedVersion,
    DomainMismatch,
    Oversized,
    Truncated,
    InvalidIdentity,
    InvalidPath,
    IdentityMismatch,
}
impl fmt::Debug for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ReaderProtocolError")
    }
}
impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid cookie reader protocol")
    }
}
impl std::error::Error for ProtocolError {}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Domain {
    Identity,
    Begin,
    Capability,
    Receipt,
}
impl Domain {
    fn code(self) -> u8 {
        match self {
            Self::Identity => b'I',
            Self::Begin => b'B',
            Self::Capability => b'C',
            Self::Receipt => b'P',
        }
    }
}
impl fmt::Debug for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ReaderProtocolDomain")
    }
}
#[derive(Clone, Eq, PartialEq)]
pub(crate) struct Capability {
    pub index: u8,
    pub nonce: [u8; 16],
    pub digest: [u8; 32],
    pub boot: [u8; 16],
    pub receipt_dev: u64,
    pub receipt_ino: u64,
}
#[derive(Clone, Eq, PartialEq)]
pub(crate) struct ReaderIdentity {
    pub identity_digest: [u8; 32],
    pub capability: Capability,
    pub unit: [u8; 16],
    pub invocation: [u8; 16],
    pub cgroup_mount: u64,
    pub cgroup_dev: u64,
    pub cgroup_ino: u64,
    pub path: String,
}
#[derive(Clone, Eq, PartialEq)]
pub(crate) struct Begin {
    pub index: u8,
    pub nonce: [u8; 16],
}
#[derive(Clone, Eq, PartialEq)]
pub(crate) struct CleanupReceipt {
    pub identity_digest: [u8; 32],
    pub capability: Capability,
    pub unit: [u8; 16],
    pub invocation: [u8; 16],
    pub cgroup_mount: u64,
    pub cgroup_dev: u64,
    pub cgroup_ino: u64,
}
macro_rules! redacted {
    ($($name:ty),+) => { $(impl fmt::Debug for $name {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("CookieReader(<redacted>)") }
    })+ };
}
redacted!(Capability, ReaderIdentity, Begin, CleanupReceipt);
fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}
fn validate_capability(value: &Capability) -> Result<(), ProtocolError> {
    if value.index > 1
        || !nonzero(&value.nonce)
        || !nonzero(&value.digest)
        || !nonzero(&value.boot)
        || value.receipt_ino == 0
    {
        return Err(ProtocolError::InvalidIdentity);
    }
    Ok(())
}
fn validate_path(path: &str) -> Result<(), ProtocolError> {
    if path.len() > MAX_PATH {
        return Err(ProtocolError::Oversized);
    }
    if !path.starts_with('/')
        || path == "/"
        || path.bytes().any(|b| b < 32 || b == 127)
        || path[1..]
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(ProtocolError::InvalidPath);
    }
    Ok(())
}
fn validate_group(
    unit: &[u8; 16],
    invocation: &[u8; 16],
    mount: u64,
    ino: u64,
) -> Result<(), ProtocolError> {
    if !nonzero(unit) || !nonzero(invocation) || mount == 0 || ino == 0 {
        return Err(ProtocolError::InvalidIdentity);
    }
    Ok(())
}
fn push_capability(output: &mut Vec<u8>, value: &Capability) {
    output.push(value.index);
    output.extend_from_slice(&value.nonce);
    output.extend_from_slice(&value.digest);
    output.extend_from_slice(&value.boot);
    output.extend_from_slice(&value.receipt_dev.to_le_bytes());
    output.extend_from_slice(&value.receipt_ino.to_le_bytes());
}
fn frame(domain: Domain, payload: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    if payload.len() > MAX_PACKET - HEADER {
        return Err(ProtocolError::Oversized);
    }
    let mut output = Vec::with_capacity(HEADER + payload.len());
    output.extend_from_slice(MAGIC);
    output.push(VERSION);
    output.push(domain.code());
    output.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    output.extend_from_slice(payload);
    Ok(output)
}
fn payload(bytes: &[u8], domain: Domain) -> Result<&[u8], ProtocolError> {
    if bytes.len() > MAX_PACKET {
        return Err(ProtocolError::Oversized);
    }
    if bytes.len() < HEADER {
        return Err(ProtocolError::Truncated);
    }
    if &bytes[..4] != MAGIC {
        return Err(ProtocolError::InvalidFrame);
    }
    if bytes[4] != VERSION {
        return Err(ProtocolError::UnsupportedVersion);
    }
    if bytes[5] != domain.code() {
        return Err(ProtocolError::DomainMismatch);
    }
    let size = u16::from_le_bytes([bytes[6], bytes[7]]) as usize;
    if size > MAX_PACKET - HEADER {
        return Err(ProtocolError::Oversized);
    }
    if bytes.len() < HEADER + size {
        return Err(ProtocolError::Truncated);
    }
    if bytes.len() != HEADER + size {
        return Err(ProtocolError::InvalidFrame);
    }
    Ok(&bytes[HEADER..])
}
struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Cursor<'a> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], ProtocolError> {
        let end = self.at.checked_add(N).ok_or(ProtocolError::Oversized)?;
        let bytes = self
            .bytes
            .get(self.at..end)
            .ok_or(ProtocolError::Truncated)?;
        self.at = end;
        bytes.try_into().map_err(|_| ProtocolError::InvalidFrame)
    }
    fn integer(&mut self) -> Result<u64, ProtocolError> {
        Ok(u64::from_le_bytes(self.take()?))
    }
    fn capability(&mut self) -> Result<Capability, ProtocolError> {
        let value = Capability {
            index: self.take::<1>()?[0],
            nonce: self.take()?,
            digest: self.take()?,
            boot: self.take()?,
            receipt_dev: self.integer()?,
            receipt_ino: self.integer()?,
        };
        validate_capability(&value)?;
        Ok(value)
    }
    fn done(&self) -> Result<(), ProtocolError> {
        if self.at == self.bytes.len() {
            Ok(())
        } else {
            Err(ProtocolError::InvalidFrame)
        }
    }
}
pub(crate) fn encode_capability(value: &Capability) -> Result<Vec<u8>, ProtocolError> {
    validate_capability(value)?;
    let mut output = Vec::new();
    push_capability(&mut output, value);
    frame(Domain::Capability, &output)
}
pub(crate) fn decode_capability(bytes: &[u8]) -> Result<Capability, ProtocolError> {
    let mut cursor = Cursor {
        bytes: payload(bytes, Domain::Capability)?,
        at: 0,
    };
    let value = cursor.capability()?;
    cursor.done()?;
    Ok(value)
}
pub(crate) fn encode_begin(value: &Begin) -> Result<Vec<u8>, ProtocolError> {
    if value.index > 1 || !nonzero(&value.nonce) {
        return Err(ProtocolError::InvalidIdentity);
    }
    let mut bytes = vec![value.index];
    bytes.extend_from_slice(&value.nonce);
    frame(Domain::Begin, &bytes)
}
pub(crate) fn decode_begin(bytes: &[u8]) -> Result<Begin, ProtocolError> {
    let mut cursor = Cursor {
        bytes: payload(bytes, Domain::Begin)?,
        at: 0,
    };
    let value = Begin {
        index: cursor.take::<1>()?[0],
        nonce: cursor.take()?,
    };
    cursor.done()?;
    if value.index > 1 || !nonzero(&value.nonce) {
        return Err(ProtocolError::InvalidIdentity);
    }
    Ok(value)
}
pub(crate) fn encode_identity(value: &ReaderIdentity) -> Result<Vec<u8>, ProtocolError> {
    validate_capability(&value.capability)?;
    if !nonzero(&value.identity_digest) {
        return Err(ProtocolError::InvalidIdentity);
    }
    validate_path(&value.path)?;
    validate_group(
        &value.unit,
        &value.invocation,
        value.cgroup_mount,
        value.cgroup_ino,
    )?;
    let mut bytes = Vec::new();
    push_capability(&mut bytes, &value.capability);
    bytes.extend_from_slice(&value.identity_digest);
    bytes.extend_from_slice(&value.unit);
    bytes.extend_from_slice(&value.invocation);
    for number in [value.cgroup_mount, value.cgroup_dev, value.cgroup_ino] {
        bytes.extend_from_slice(&number.to_le_bytes());
    }
    bytes.extend_from_slice(&(value.path.len() as u16).to_le_bytes());
    bytes.extend_from_slice(value.path.as_bytes());
    frame(Domain::Identity, &bytes)
}
pub(crate) fn decode_identity(bytes: &[u8]) -> Result<ReaderIdentity, ProtocolError> {
    let mut cursor = Cursor {
        bytes: payload(bytes, Domain::Identity)?,
        at: 0,
    };
    let capability = cursor.capability()?;
    let identity_digest = cursor.take()?;
    if !nonzero(&identity_digest) {
        return Err(ProtocolError::InvalidIdentity);
    }
    let unit = cursor.take()?;
    let invocation = cursor.take()?;
    let cgroup_mount = cursor.integer()?;
    let cgroup_dev = cursor.integer()?;
    let cgroup_ino = cursor.integer()?;
    let count = u16::from_le_bytes(cursor.take()?) as usize;
    if count > MAX_PATH {
        return Err(ProtocolError::Oversized);
    }
    let path = cursor
        .bytes
        .get(cursor.at..cursor.at + count)
        .ok_or(ProtocolError::Truncated)?;
    let path = std::str::from_utf8(path)
        .map_err(|_| ProtocolError::InvalidPath)?
        .to_owned();
    cursor.at += count;
    cursor.done()?;
    validate_path(&path)?;
    validate_group(&unit, &invocation, cgroup_mount, cgroup_ino)?;
    Ok(ReaderIdentity {
        identity_digest,
        capability,
        unit,
        invocation,
        cgroup_mount,
        cgroup_dev,
        cgroup_ino,
        path,
    })
}
impl CleanupReceipt {
    pub fn from_identity(identity: &ReaderIdentity) -> Self {
        Self {
            identity_digest: identity.identity_digest,
            capability: identity.capability.clone(),
            unit: identity.unit,
            invocation: identity.invocation,
            cgroup_mount: identity.cgroup_mount,
            cgroup_dev: identity.cgroup_dev,
            cgroup_ino: identity.cgroup_ino,
        }
    }
    pub fn matches(&self, identity: &ReaderIdentity) -> Result<(), ProtocolError> {
        if self == &Self::from_identity(identity) {
            Ok(())
        } else {
            Err(ProtocolError::IdentityMismatch)
        }
    }
}
pub(crate) fn encode_cleanup_receipt(value: &CleanupReceipt) -> Result<Vec<u8>, ProtocolError> {
    validate_capability(&value.capability)?;
    if !nonzero(&value.identity_digest) {
        return Err(ProtocolError::InvalidIdentity);
    }
    validate_group(
        &value.unit,
        &value.invocation,
        value.cgroup_mount,
        value.cgroup_ino,
    )?;
    let mut bytes = Vec::new();
    push_capability(&mut bytes, &value.capability);
    bytes.extend_from_slice(&value.identity_digest);
    bytes.extend_from_slice(&value.unit);
    bytes.extend_from_slice(&value.invocation);
    for number in [value.cgroup_mount, value.cgroup_dev, value.cgroup_ino] {
        bytes.extend_from_slice(&number.to_le_bytes());
    }
    // Proof kind 1 means exact owned native terminal-empty. Only the caller can establish it.
    bytes.push(1);
    if bytes.len() > RECEIPT_SIZE - HEADER {
        return Err(ProtocolError::Oversized);
    }
    bytes.resize(RECEIPT_SIZE - HEADER, 0);
    frame(Domain::Receipt, &bytes)
}
pub(crate) fn decode_cleanup_receipt(bytes: &[u8]) -> Result<CleanupReceipt, ProtocolError> {
    let data = payload(bytes, Domain::Receipt)?;
    if bytes.len() != RECEIPT_SIZE {
        return Err(ProtocolError::InvalidFrame);
    }
    let mut cursor = Cursor { bytes: data, at: 0 };
    let capability = cursor.capability()?;
    let identity_digest = cursor.take()?;
    if !nonzero(&identity_digest) {
        return Err(ProtocolError::InvalidIdentity);
    }
    let unit = cursor.take()?;
    let invocation = cursor.take()?;
    let cgroup_mount = cursor.integer()?;
    let cgroup_dev = cursor.integer()?;
    let cgroup_ino = cursor.integer()?;
    if cursor.take::<1>()?[0] != 1 || cursor.bytes[cursor.at..].iter().any(|b| *b != 0) {
        return Err(ProtocolError::InvalidFrame);
    }
    validate_group(&unit, &invocation, cgroup_mount, cgroup_ino)?;
    Ok(CleanupReceipt {
        identity_digest,
        capability,
        unit,
        invocation,
        cgroup_mount,
        cgroup_dev,
        cgroup_ino,
    })
}
pub(crate) struct Decoder {
    domain: Domain,
    bytes: Vec<u8>,
    complete: bool,
}
impl fmt::Debug for Decoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ReaderDecoder(<redacted>)")
    }
}
impl Decoder {
    pub fn new(domain: Domain) -> Self {
        Self {
            domain,
            bytes: Vec::with_capacity(MAX_PACKET),
            complete: false,
        }
    }
    pub fn push(&mut self, bytes: &[u8]) -> Result<Option<Vec<u8>>, ProtocolError> {
        if self.complete {
            return Err(ProtocolError::InvalidFrame);
        }
        if bytes.len() > MAX_PACKET - self.bytes.len() {
            return Err(ProtocolError::Oversized);
        }
        self.bytes.extend_from_slice(bytes);
        if self.bytes.len() < HEADER {
            return Ok(None);
        }
        match payload(&self.bytes, self.domain) {
            Ok(_) => {
                self.complete = true;
                Ok(Some(std::mem::take(&mut self.bytes)))
            }
            Err(ProtocolError::Truncated) => Ok(None),
            Err(error) => Err(error),
        }
    }
    pub fn finish(&self) -> Result<(), ProtocolError> {
        if self.complete {
            Ok(())
        } else {
            Err(ProtocolError::Truncated)
        }
    }
}

#[cfg(test)]
// Some integration harnesses import the protocol without lock-sensitive fixtures.
#[allow(dead_code)]
pub(crate) fn run_isolated_lock_test() -> bool {
    const MARKER: &str = "DLPAD_TEST_OWNED_LOCK_ISOLATED";
    const ENTRY: &str = "DLPAD_TEST_OWNED_LOCK_ENTRY";
    let thread = std::thread::current();
    let name = thread.name().expect("exact harness test name unavailable");
    if std::env::var_os(MARKER).as_deref() == Some(std::ffi::OsStr::new(name)) {
        let entry = std::env::var_os(ENTRY).expect("owned exact lock entry unavailable");
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(entry)
            .expect("owned exact lock entry create failed");
        std::io::Write::write_all(&mut file, b"1").expect("owned exact lock entry write failed");
        return false;
    }
    let marker = tempfile::tempdir().expect("owned exact lock marker directory failed");
    let entry = marker.path().join("entry");
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture", "--test-threads=1"])
        .env(MARKER, name)
        .env(ENTRY, &entry)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .expect("owned exact lock test spawn failed");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                assert!(status.success(), "isolated strict lock test failed");
                let file =
                    std::fs::File::open(&entry).expect("exact child did not enter strict test");
                let mut bytes = Vec::with_capacity(2);
                std::io::Read::read_to_end(&mut std::io::Read::take(file, 2), &mut bytes)
                    .expect("owned exact lock marker read failed");
                assert_eq!(bytes, b"1", "exact child entry marker invalid");
                return true;
            }
            Ok(None) if std::time::Instant::now() < deadline => {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            result => {
                let _ = child.kill();
                let reap_deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
                let mut reaped = false;
                while std::time::Instant::now() < reap_deadline {
                    if matches!(child.try_wait(), Ok(Some(_))) {
                        reaped = true;
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                panic!(
                    "owned lock test timeout/wait failure; wait_error={} reaped={reaped}",
                    result.is_err()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn identity() -> ReaderIdentity {
        ReaderIdentity {
            identity_digest: [9; 32],
            capability: Capability {
                index: 1,
                nonce: [1; 16],
                digest: [2; 32],
                boot: [3; 16],
                receipt_dev: 5,
                receipt_ino: 6,
            },
            unit: [4; 16],
            invocation: [5; 16],
            cgroup_mount: 7,
            cgroup_dev: 8,
            cgroup_ino: 9,
            path: "/user.slice/dlpad-job-owned.service".into(),
        }
    }
    #[test]
    fn roundtrip_literal_identity_begin_capability_receipt() {
        let value = identity();
        assert_eq!(
            decode_identity(&encode_identity(&value).unwrap()).unwrap(),
            value
        );
        assert_eq!(
            decode_capability(&encode_capability(&value.capability).unwrap()).unwrap(),
            value.capability
        );
        let begin = Begin {
            index: 1,
            nonce: [1; 16],
        };
        assert_eq!(decode_begin(&encode_begin(&begin).unwrap()).unwrap(), begin);
        let receipt = CleanupReceipt::from_identity(&value);
        let bytes = encode_cleanup_receipt(&receipt).unwrap();
        assert_eq!(bytes.len(), 256);
        assert_eq!(decode_cleanup_receipt(&bytes).unwrap(), receipt);
        receipt.matches(&value).unwrap();
        assert!(!format!("{value:?}").contains("owned"));
    }
    #[test]
    fn bounded_partial_decoder_and_truncation() {
        let bytes = encode_identity(&identity()).unwrap();
        let mut decoder = Decoder::new(Domain::Identity);
        for byte in &bytes[..bytes.len() - 1] {
            assert!(decoder.push(&[*byte]).unwrap().is_none());
        }
        assert_eq!(decoder.finish(), Err(ProtocolError::Truncated));
        assert_eq!(
            decoder.push(&bytes[bytes.len() - 1..]).unwrap(),
            Some(bytes.clone())
        );
        decoder.finish().unwrap();
        assert_eq!(decoder.push(&[]), Err(ProtocolError::InvalidFrame));
        for end in 0..bytes.len() {
            assert_eq!(
                decode_identity(&bytes[..end]),
                Err(ProtocolError::Truncated)
            );
        }
        assert_eq!(
            Decoder::new(Domain::Identity).push(&vec![0; MAX_PACKET + 1]),
            Err(ProtocolError::Oversized)
        );
    }
    #[test]
    fn invalid_paths_zero_fields_domains_and_padding_fail_closed() {
        for path in [
            "/", "relative", "/a//b", "/a/../b", "/a/./b", "/a/", "/a\0b", "/a\nb",
        ] {
            let mut value = identity();
            value.path = path.into();
            assert!(encode_identity(&value).is_err());
        }
        let mut value = identity();
        value.path = format!("/{}", "x".repeat(512));
        assert_eq!(encode_identity(&value), Err(ProtocolError::Oversized));
        value = identity();
        value.capability.index = 2;
        assert!(encode_identity(&value).is_err());
        value = identity();
        value.capability.nonce = [0; 16];
        assert!(encode_identity(&value).is_err());
        value = identity();
        value.invocation = [0; 16];
        assert!(encode_identity(&value).is_err());
        let bytes = encode_identity(&identity()).unwrap();
        assert_eq!(
            decode_capability(&bytes),
            Err(ProtocolError::DomainMismatch)
        );
        let mut bad = bytes.clone();
        bad[4] = 2;
        assert_eq!(
            decode_identity(&bad),
            Err(ProtocolError::UnsupportedVersion)
        );
        let mut bad = bytes;
        bad[6..8].copy_from_slice(&1024u16.to_le_bytes());
        assert_eq!(decode_identity(&bad), Err(ProtocolError::Oversized));
        let mut receipt =
            encode_cleanup_receipt(&CleanupReceipt::from_identity(&identity())).unwrap();
        receipt[255] = 1;
        assert_eq!(
            decode_cleanup_receipt(&receipt),
            Err(ProtocolError::InvalidFrame)
        );
    }
    #[test]
    fn cleanup_receipt_cannot_match_another_reader_or_invocation() {
        let value = identity();
        let receipt = CleanupReceipt::from_identity(&value);
        let mut different = value.clone();
        different.capability.nonce = [6; 16];
        assert_eq!(
            receipt.matches(&different),
            Err(ProtocolError::IdentityMismatch)
        );
        different = value.clone();
        different.invocation = [7; 16];
        assert_eq!(
            receipt.matches(&different),
            Err(ProtocolError::IdentityMismatch)
        );
        different = value.clone();
        different.identity_digest = [8; 32];
        assert_eq!(
            receipt.matches(&different),
            Err(ProtocolError::IdentityMismatch)
        );
        // Codec checks the provided digest, not its cryptographic derivation from path.
        different = value.clone();
        different.path = "/different-owned-path".into();
        receipt.matches(&different).unwrap();
        different = value;
        different.capability.receipt_ino += 1;
        assert_eq!(
            receipt.matches(&different),
            Err(ProtocolError::IdentityMismatch)
        );
    }
}
