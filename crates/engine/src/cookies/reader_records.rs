//! Pure two-reader state. Anchored files, locks, sync and recovery authority remain caller-owned.
use crate::process::reader_protocol::{
    self as protocol, Capability, CleanupReceipt, ReaderIdentity,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

pub(crate) const MAX_RECORD_BYTES: usize = 4096;
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum RecordError {
    InvalidRecord,
    UnsupportedVersion,
    Oversized,
    InvalidTransition,
    IdentityMismatch,
    MissingReceipt,
}
impl fmt::Debug for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CookieReaderRecordError")
    }
}
impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid cookie reader record")
    }
}
impl std::error::Error for RecordError {}
#[derive(Clone, Eq, PartialEq)]
pub(crate) enum Slot {
    Empty,
    Intent(Capability),
    Bound(ReaderIdentity),
    Resolved(ReaderIdentity),
}
#[derive(Clone, Eq, PartialEq)]
pub(crate) struct ReaderRecords {
    slots: [Slot; 2],
}
impl fmt::Debug for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CookieReaderSlot(<redacted>)")
    }
}
impl fmt::Debug for ReaderRecords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CookieReaderRecords(<redacted>)")
    }
}
/// Canonical digest, shared by the native identity producer and record binder.
/// Intent token is separate; this digest authenticates the complete native identity bytes.
pub(crate) fn identity_digest(identity: &ReaderIdentity) -> Result<[u8; 32], RecordError> {
    let mut normalized = identity.clone();
    normalized.identity_digest = [1; 32];
    protocol::encode_identity(&normalized).map_err(|_| RecordError::InvalidRecord)?;
    let mut hash = Sha256::new();
    hash.update(b"DLPAD-READER-IDENTITY-V1\0");
    hash.update(identity.capability.boot);
    hash.update(identity.unit);
    hash.update(identity.invocation);
    for value in [
        identity.cgroup_mount,
        identity.cgroup_dev,
        identity.cgroup_ino,
    ] {
        hash.update(value.to_le_bytes());
    }
    hash.update((identity.path.len() as u16).to_le_bytes());
    hash.update(identity.path.as_bytes());
    Ok(hash.finalize().into())
}
fn checked_identity(identity: &ReaderIdentity) -> Result<(), RecordError> {
    if identity_digest(identity)? != identity.identity_digest {
        return Err(RecordError::IdentityMismatch);
    }
    Ok(())
}
impl Default for ReaderRecords {
    fn default() -> Self {
        Self {
            slots: [Slot::Empty, Slot::Empty],
        }
    }
}
impl ReaderRecords {
    pub fn slot(&self, index: usize) -> Option<&Slot> {
        self.slots.get(index)
    }
    pub fn begin(&mut self, capability: Capability) -> Result<(), RecordError> {
        protocol::encode_capability(&capability).map_err(|_| RecordError::InvalidRecord)?;
        let index = capability.index as usize;
        if !matches!(self.slots[index], Slot::Empty)
            || (index == 0 && !matches!(self.slots[1], Slot::Empty))
            || (index == 1 && !matches!(self.slots[0], Slot::Resolved(_)))
        {
            return Err(RecordError::InvalidTransition);
        }
        self.slots[index] = Slot::Intent(capability);
        Ok(())
    }
    pub fn bind(&mut self, identity: ReaderIdentity) -> Result<(), RecordError> {
        checked_identity(&identity)?;
        let index = identity.capability.index as usize;
        match &self.slots[index] {
            Slot::Intent(capability) if capability == &identity.capability => {}
            Slot::Intent(_) => return Err(RecordError::IdentityMismatch),
            _ => return Err(RecordError::InvalidTransition),
        }
        self.slots[index] = Slot::Bound(identity);
        Ok(())
    }
    pub fn resolve(&mut self, receipt: &CleanupReceipt) -> Result<(), RecordError> {
        protocol::encode_cleanup_receipt(receipt).map_err(|_| RecordError::InvalidRecord)?;
        let index = receipt.capability.index as usize;
        let identity = match &self.slots[index] {
            Slot::Bound(identity) => identity,
            _ => return Err(RecordError::InvalidTransition),
        };
        checked_identity(identity)?;
        receipt
            .matches(identity)
            .map_err(|_| RecordError::IdentityMismatch)?;
        self.slots[index] = Slot::Resolved(identity.clone());
        Ok(())
    }
    /// Never substitutes for the caller's live-owner lock or anchored file identity checks.
    pub fn cleanup_eligible(&self) -> bool {
        self.slots
            .iter()
            .all(|slot| matches!(slot, Slot::Empty | Slot::Resolved(_)))
    }
    pub fn encode(&self) -> Result<Vec<u8>, RecordError> {
        let wire = WireRecord {
            version: 2,
            slots: [
                WireSlot::from_slot(&self.slots[0]),
                WireSlot::from_slot(&self.slots[1]),
            ],
        };
        let bytes = serde_json::to_vec(&wire).map_err(|_| RecordError::InvalidRecord)?;
        if bytes.len() > MAX_RECORD_BYTES {
            return Err(RecordError::Oversized);
        }
        Ok(bytes)
    }
    /// Resolved phases require the independently validated durable receipt again on reload.
    pub fn decode(
        bytes: &[u8],
        receipts: [Option<&CleanupReceipt>; 2],
    ) -> Result<Self, RecordError> {
        if bytes.len() > MAX_RECORD_BYTES {
            return Err(RecordError::Oversized);
        }
        let version: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|_| RecordError::InvalidRecord)?;
        if version.get("version").and_then(serde_json::Value::as_u64) != Some(2) {
            return Err(RecordError::UnsupportedVersion);
        }
        let wire: WireRecord =
            serde_json::from_slice(bytes).map_err(|_| RecordError::InvalidRecord)?;
        let mut result = Self::default();
        for (index, slot) in wire.slots.into_iter().enumerate() {
            if slot.phase == 0 {
                if slot.capability.is_some() || slot.identity.is_some() {
                    return Err(RecordError::InvalidRecord);
                }
                continue;
            }
            let capability = slot
                .capability
                .ok_or(RecordError::InvalidRecord)?
                .into_capability()?;
            if capability.index as usize != index {
                return Err(RecordError::InvalidRecord);
            }
            result.begin(capability.clone())?;
            match slot.phase {
                1 if slot.identity.is_none() => {}
                2 | 3 => {
                    let identity = slot
                        .identity
                        .ok_or(RecordError::InvalidRecord)?
                        .into_identity(capability)?;
                    result.bind(identity)?;
                    if slot.phase == 3 {
                        result.resolve(receipts[index].ok_or(RecordError::MissingReceipt)?)?;
                    }
                }
                _ => return Err(RecordError::InvalidRecord),
            }
        }
        Ok(result)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRecord {
    version: u8,
    slots: [WireSlot; 2],
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSlot {
    phase: u8,
    capability: Option<WireCapability>,
    identity: Option<WireIdentity>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireCapability {
    index: u8,
    nonce: String,
    digest: String,
    boot: String,
    receipt_dev: u64,
    receipt_ino: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireIdentity {
    identity_digest: String,
    unit: String,
    invocation: String,
    cgroup_mount: u64,
    cgroup_dev: u64,
    cgroup_ino: u64,
    path: String,
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn unhex<const N: usize>(text: &str) -> Result<[u8; N], RecordError> {
    if text.len() != N * 2
        || !text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(RecordError::InvalidRecord);
    }
    let mut result = [0; N];
    for (index, chunk) in text.as_bytes().chunks_exact(2).enumerate() {
        let digit = |byte: u8| {
            if byte <= b'9' {
                byte - b'0'
            } else {
                byte - b'a' + 10
            }
        };
        result[index] = digit(chunk[0]) * 16 + digit(chunk[1]);
    }
    Ok(result)
}
impl WireCapability {
    fn from_capability(value: &Capability) -> Self {
        Self {
            index: value.index,
            nonce: hex(&value.nonce),
            digest: hex(&value.digest),
            boot: hex(&value.boot),
            receipt_dev: value.receipt_dev,
            receipt_ino: value.receipt_ino,
        }
    }
    fn into_capability(self) -> Result<Capability, RecordError> {
        Ok(Capability {
            index: self.index,
            nonce: unhex(&self.nonce)?,
            digest: unhex(&self.digest)?,
            boot: unhex(&self.boot)?,
            receipt_dev: self.receipt_dev,
            receipt_ino: self.receipt_ino,
        })
    }
}
impl WireIdentity {
    fn from_identity(value: &ReaderIdentity) -> Self {
        Self {
            identity_digest: hex(&value.identity_digest),
            unit: hex(&value.unit),
            invocation: hex(&value.invocation),
            cgroup_mount: value.cgroup_mount,
            cgroup_dev: value.cgroup_dev,
            cgroup_ino: value.cgroup_ino,
            path: value.path.clone(),
        }
    }
    fn into_identity(self, capability: Capability) -> Result<ReaderIdentity, RecordError> {
        Ok(ReaderIdentity {
            capability,
            identity_digest: unhex(&self.identity_digest)?,
            unit: unhex(&self.unit)?,
            invocation: unhex(&self.invocation)?,
            cgroup_mount: self.cgroup_mount,
            cgroup_dev: self.cgroup_dev,
            cgroup_ino: self.cgroup_ino,
            path: self.path,
        })
    }
}
impl WireSlot {
    fn from_slot(slot: &Slot) -> Self {
        let (phase, capability, identity) = match slot {
            Slot::Empty => (0, None, None),
            Slot::Intent(value) => (1, Some(WireCapability::from_capability(value)), None),
            Slot::Bound(value) => (
                2,
                Some(WireCapability::from_capability(&value.capability)),
                Some(WireIdentity::from_identity(value)),
            ),
            Slot::Resolved(value) => (
                3,
                Some(WireCapability::from_capability(&value.capability)),
                Some(WireIdentity::from_identity(value)),
            ),
        };
        Self {
            phase,
            capability,
            identity,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn identity(index: u8) -> ReaderIdentity {
        let mut value = ReaderIdentity {
            capability: Capability {
                index,
                nonce: [index + 1; 16],
                digest: [2; 32],
                boot: [3; 16],
                receipt_dev: 4,
                receipt_ino: index as u64 + 5,
            },
            identity_digest: [1; 32],
            unit: [6; 16],
            invocation: [7; 16],
            cgroup_mount: 8,
            cgroup_dev: 9,
            cgroup_ino: 10,
            path: "/user.slice/owned.service".into(),
        };
        value.identity_digest = identity_digest(&value).unwrap();
        value
    }
    #[test]
    fn ordered_two_readers_require_both_receipts_and_reload_proof() {
        let mut state = ReaderRecords::default();
        let first = identity(0);
        let second = identity(1);
        assert_eq!(
            state.begin(second.capability.clone()),
            Err(RecordError::InvalidTransition)
        );
        state.begin(first.capability.clone()).unwrap();
        assert!(!state.cleanup_eligible());
        state.bind(first.clone()).unwrap();
        let receipt0 = CleanupReceipt::from_identity(&first);
        state.resolve(&receipt0).unwrap();
        state.begin(second.capability.clone()).unwrap();
        state.bind(second.clone()).unwrap();
        assert!(!state.cleanup_eligible());
        let receipt1 = CleanupReceipt::from_identity(&second);
        state.resolve(&receipt1).unwrap();
        assert!(state.cleanup_eligible());
        let bytes = state.encode().unwrap();
        assert_eq!(
            ReaderRecords::decode(&bytes, [Some(&receipt0), Some(&receipt1)]).unwrap(),
            state
        );
        assert_eq!(
            ReaderRecords::decode(&bytes, [None, Some(&receipt1)]),
            Err(RecordError::MissingReceipt)
        );
        assert!(!format!("{state:?}").contains("owned"));
        assert!(state.slot(2).is_none());
    }
    #[test]
    fn identity_digest_path_capability_and_repeated_transitions_fail_closed() {
        let mut state = ReaderRecords::default();
        let value = identity(0);
        state.begin(value.capability.clone()).unwrap();
        assert!(state.begin(value.capability.clone()).is_err());
        let mut changed = value.clone();
        changed.path = "/changed.service".into();
        assert_eq!(state.bind(changed), Err(RecordError::IdentityMismatch));
        let mut changed = value.clone();
        changed.capability.receipt_ino += 1;
        assert_eq!(state.bind(changed), Err(RecordError::IdentityMismatch));
        state.bind(value.clone()).unwrap();
        assert!(state.bind(value.clone()).is_err());
        let mut bad = CleanupReceipt::from_identity(&value);
        bad.identity_digest = [9; 32];
        assert_eq!(state.resolve(&bad), Err(RecordError::IdentityMismatch));
        let mut bad = CleanupReceipt::from_identity(&value);
        bad.invocation = [9; 16];
        assert_eq!(state.resolve(&bad), Err(RecordError::IdentityMismatch));
        let receipt = CleanupReceipt::from_identity(&value);
        state.resolve(&receipt).unwrap();
        assert!(state.resolve(&receipt).is_err());
    }
    #[test]
    fn maximum_two_paths_encoded_record_stays_bounded() {
        let mut state = ReaderRecords::default();
        let mut receipts = Vec::new();
        for index in 0..2 {
            let mut value = identity(index);
            value.path = format!("/{}", "\\".repeat(511));
            value.identity_digest = identity_digest(&value).unwrap();
            state.begin(value.capability.clone()).unwrap();
            state.bind(value.clone()).unwrap();
            let receipt = CleanupReceipt::from_identity(&value);
            state.resolve(&receipt).unwrap();
            receipts.push(receipt);
        }
        let bytes = state.encode().unwrap();
        assert!(bytes.len() <= 4096);
        assert_eq!(
            ReaderRecords::decode(&bytes, [Some(&receipts[0]), Some(&receipts[1])]).unwrap(),
            state
        );
        assert_eq!(
            ReaderRecords::decode(&vec![b' '; 4097], [None, None]),
            Err(RecordError::Oversized)
        );
    }
    #[test]
    fn version_one_unknown_partial_and_zero_intent_preserve() {
        assert_eq!(
            ReaderRecords::decode(br#"{"version":1,"existing":"preserve"}"#, [None, None]),
            Err(RecordError::UnsupportedVersion)
        );
        assert!(ReaderRecords::decode(br#"{"version":2,"slots":[]}"#, [None, None]).is_err());
        let mut value = identity(0).capability;
        value.nonce = [0; 16];
        assert!(ReaderRecords::default().begin(value).is_err());
        let mut json: serde_json::Value =
            serde_json::from_slice(&ReaderRecords::default().encode().unwrap()).unwrap();
        json["extra"] = true.into();
        assert!(ReaderRecords::decode(&serde_json::to_vec(&json).unwrap(), [None, None]).is_err());
    }
}
