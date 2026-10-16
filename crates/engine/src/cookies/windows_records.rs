//! Pure Windows reader format. Native ACL/handle/receipt provenance is caller-owned.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
const MAX_RECORD: usize = 4096;
const RECEIPT_LEN: usize = 256;
const MAGIC: &[u8; 8] = b"DLPWRC01";
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecordError {
    Invalid,
    Bounds,
    Transition,
    Receipt,
}
impl std::fmt::Debug for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WindowsRecordError")
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WindowsAnchor {
    pub volume_serial: u64,
    pub file_id: [u8; 16],
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SessionAnchors {
    pub cache: WindowsAnchor,
    pub owner: WindowsAnchor,
    pub copy: WindowsAnchor,
    pub journal: WindowsAnchor,
    pub receipts: [WindowsAnchor; 2],
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BoundIdentity {
    pub index: u8,
    pub nonce: [u8; 16],
    pub guardian_created: u64,
    pub tool_created: u64,
    pub identity_digest: [u8; 32],
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", deny_unknown_fields)]
enum Slot {
    Empty,
    Intent { nonce: [u8; 16] },
    Bound { identity: BoundIdentity },
    Resolved { identity: BoundIdentity },
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WindowsReaderRecord {
    version: u8,
    session_nonce: [u8; 16],
    record_digest: [u8; 32],
    anchors: SessionAnchors,
    slots: [Slot; 2],
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct CompletionReceipt {
    index: u8,
    session_nonce: [u8; 16],
    record_digest: [u8; 32],
    identity_digest: [u8; 32],
}
macro_rules! redacted { ($($t:ty),*) => { $(impl std::fmt::Debug for $t {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("WindowsReaderRedacted")}})* }; }
redacted!(
    WindowsAnchor,
    SessionAnchors,
    BoundIdentity,
    WindowsReaderRecord,
    CompletionReceipt
);
fn nonzero(bytes: &[u8]) -> bool {
    bytes.iter().any(|b| *b != 0)
}
impl WindowsReaderRecord {
    /// Original immutable recovery-record digest, never the mutable slot JSON hash.
    pub(crate) fn record_digest(&self) -> [u8; 32] {
        self.record_digest
    }
    pub(crate) fn session_nonce(&self) -> [u8; 16] {
        self.session_nonce
    }
    pub(crate) fn anchors(&self) -> SessionAnchors {
        self.anchors
    }
    /// Only original live import ownership may use this as an unused-reader check.
    /// It is never startup recovery authority or a replacement for native receipts.
    pub(crate) fn is_pristine(&self) -> bool {
        self.slots.iter().all(|slot| matches!(slot, Slot::Empty))
    }
    /// Execution handoff requires pending Bound, never a previously resolved slot.
    pub(crate) fn pending_bound_identity(&self, index: u8) -> Option<BoundIdentity> {
        match self.slots.get(index as usize)? {
            Slot::Bound { identity } => Some(*identity),
            _ => None,
        }
    }
    pub(crate) fn bound_identity(&self, index: u8) -> Option<BoundIdentity> {
        match self.slots.get(index as usize)? {
            Slot::Bound { identity } | Slot::Resolved { identity } => Some(*identity),
            _ => None,
        }
    }
    pub(crate) fn new(
        session_nonce: [u8; 16],
        record_digest: [u8; 32],
        anchors: SessionAnchors,
    ) -> Result<Self, RecordError> {
        let value = Self {
            version: 1,
            session_nonce,
            record_digest,
            anchors,
            slots: [Slot::Empty, Slot::Empty],
        };
        value.validate()?;
        Ok(value)
    }
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, RecordError> {
        if bytes.len() > MAX_RECORD {
            return Err(RecordError::Bounds);
        }
        let value: Self = serde_json::from_slice(bytes).map_err(|_| RecordError::Invalid)?;
        value.validate()?;
        Ok(value)
    }
    pub(crate) fn encode(&self) -> Result<Vec<u8>, RecordError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| RecordError::Invalid)?;
        if bytes.len() > MAX_RECORD {
            return Err(RecordError::Bounds);
        }
        Ok(bytes)
    }
    fn identity_digest(&self, index: u8, nonce: [u8; 16], guardian: u64, tool: u64) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(b"DLPAD-WINDOWS-READER-V1\0");
        h.update(self.session_nonce);
        h.update(self.record_digest);
        for a in [
            self.anchors.cache,
            self.anchors.owner,
            self.anchors.copy,
            self.anchors.journal,
            self.anchors.receipts[0],
            self.anchors.receipts[1],
        ] {
            h.update(a.volume_serial.to_le_bytes());
            h.update(a.file_id);
        }
        h.update([index]);
        h.update(nonce);
        h.update(guardian.to_le_bytes());
        h.update(tool.to_le_bytes());
        h.finalize().into()
    }
    fn validate(&self) -> Result<(), RecordError> {
        if self.version != 1 || !nonzero(&self.session_nonce) || !nonzero(&self.record_digest) {
            return Err(RecordError::Invalid);
        }
        for a in [
            self.anchors.cache,
            self.anchors.owner,
            self.anchors.copy,
            self.anchors.journal,
            self.anchors.receipts[0],
            self.anchors.receipts[1],
        ] {
            if !nonzero(&a.file_id) {
                return Err(RecordError::Invalid);
            }
        }
        if !matches!(self.slots[1], Slot::Empty) && !matches!(self.slots[0], Slot::Resolved { .. })
        {
            return Err(RecordError::Transition);
        }
        let nonce_of = |s: &Slot| match s {
            Slot::Empty => None,
            Slot::Intent { nonce } => Some(*nonce),
            Slot::Bound { identity } | Slot::Resolved { identity } => Some(identity.nonce),
        };
        if let (Some(a), Some(b)) = (nonce_of(&self.slots[0]), nonce_of(&self.slots[1]))
            && a == b
        {
            return Err(RecordError::Invalid);
        }
        for (index, slot) in self.slots.iter().enumerate() {
            match slot {
                Slot::Empty => {}
                Slot::Intent { nonce } => {
                    if !nonzero(nonce) {
                        return Err(RecordError::Invalid);
                    }
                }
                Slot::Bound { identity: i } | Slot::Resolved { identity: i } => {
                    if i.index as usize != index
                        || !nonzero(&i.nonce)
                        || i.guardian_created == 0
                        || i.tool_created == 0
                        || i.identity_digest
                            != self.identity_digest(
                                i.index,
                                i.nonce,
                                i.guardian_created,
                                i.tool_created,
                            )
                    {
                        return Err(RecordError::Invalid);
                    }
                }
            }
        }
        Ok(())
    }
    pub(crate) fn begin(&mut self, index: u8, nonce: [u8; 16]) -> Result<(), RecordError> {
        self.validate()?;
        if index > 1
            || !nonzero(&nonce)
            || !matches!(self.slots[index as usize], Slot::Empty)
            || (index == 1 && !matches!(self.slots[0], Slot::Resolved { .. }))
        {
            return Err(RecordError::Transition);
        }
        if index == 1 {
            let Slot::Resolved { identity } = self.slots[0] else {
                return Err(RecordError::Transition);
            };
            if identity.nonce == nonce {
                return Err(RecordError::Transition);
            }
        }
        self.slots[index as usize] = Slot::Intent { nonce };
        Ok(())
    }
    pub(crate) fn bind(
        &mut self,
        index: u8,
        guardian_created: u64,
        tool_created: u64,
    ) -> Result<BoundIdentity, RecordError> {
        self.validate()?;
        if index > 1 || guardian_created == 0 || tool_created == 0 {
            return Err(RecordError::Invalid);
        }
        let Slot::Intent { nonce } = self.slots[index as usize] else {
            return Err(RecordError::Transition);
        };
        let identity = BoundIdentity {
            index,
            nonce,
            guardian_created,
            tool_created,
            identity_digest: self.identity_digest(index, nonce, guardian_created, tool_created),
        };
        self.slots[index as usize] = Slot::Bound { identity };
        Ok(identity)
    }
    pub(crate) fn receipt_for(&self, index: u8) -> Result<CompletionReceipt, RecordError> {
        self.validate()?;
        let Some(Slot::Bound { identity } | Slot::Resolved { identity }) =
            self.slots.get(index as usize)
        else {
            return Err(RecordError::Transition);
        };
        Ok(CompletionReceipt {
            index,
            session_nonce: self.session_nonce,
            record_digest: self.record_digest,
            identity_digest: identity.identity_digest,
        })
    }
    pub(crate) fn resolve(&mut self, receipt: &CompletionReceipt) -> Result<(), RecordError> {
        self.validate()?;
        if &self.receipt_for(receipt.index)? != receipt {
            return Err(RecordError::Receipt);
        }
        let Slot::Bound { identity } = self.slots[receipt.index as usize] else {
            return Err(RecordError::Transition);
        };
        self.slots[receipt.index as usize] = Slot::Resolved { identity };
        Ok(())
    }
    pub(crate) fn all_receipts_match(&self, receipts: &[CompletionReceipt]) -> bool {
        if self.validate().is_err() || receipts.is_empty() || receipts.len() > 2 {
            return false;
        }
        let mut count = 0;
        for (index, slot) in self.slots.iter().enumerate() {
            match slot {
                Slot::Empty => {}
                Slot::Intent { .. } => return false,
                Slot::Bound { .. } | Slot::Resolved { .. } => {
                    count += 1;
                    let Ok(expected) = self.receipt_for(index as u8) else {
                        return false;
                    };
                    if receipts.iter().filter(|r| **r == expected).count() != 1 {
                        return false;
                    }
                }
            }
        }
        count == receipts.len()
    }
}
impl CompletionReceipt {
    pub(crate) fn encode(&self) -> Result<[u8; RECEIPT_LEN], RecordError> {
        if self.index > 1
            || !nonzero(&self.session_nonce)
            || !nonzero(&self.record_digest)
            || !nonzero(&self.identity_digest)
        {
            return Err(RecordError::Receipt);
        }
        let mut out = [0; RECEIPT_LEN];
        out[..8].copy_from_slice(MAGIC);
        out[8] = 1;
        out[9] = self.index;
        out[16..32].copy_from_slice(&self.session_nonce);
        out[32..64].copy_from_slice(&self.record_digest);
        out[64..96].copy_from_slice(&self.identity_digest);
        out[248..256].copy_from_slice(b"COMPLETE");
        Ok(out)
    }
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, RecordError> {
        if bytes.len() != RECEIPT_LEN
            || &bytes[..8] != MAGIC
            || bytes[8] != 1
            || bytes[10..16]
                .iter()
                .chain(bytes[96..248].iter())
                .any(|b| *b != 0)
            || &bytes[248..] != b"COMPLETE"
        {
            return Err(RecordError::Receipt);
        }
        let value = Self {
            index: bytes[9],
            session_nonce: bytes[16..32].try_into().map_err(|_| RecordError::Receipt)?,
            record_digest: bytes[32..64].try_into().map_err(|_| RecordError::Receipt)?,
            identity_digest: bytes[64..96].try_into().map_err(|_| RecordError::Receipt)?,
        };
        value.encode()?;
        Ok(value)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> WindowsReaderRecord {
        let a = WindowsAnchor {
            volume_serial: 1,
            file_id: [3; 16],
        };
        WindowsReaderRecord::new(
            [1; 16],
            [2; 32],
            SessionAnchors {
                cache: a,
                owner: a,
                copy: a,
                journal: a,
                receipts: [a; 2],
            },
        )
        .unwrap()
    }
    #[test]
    fn two_readers_exact_receipts() {
        let mut r = fixture();
        assert!(!r.all_receipts_match(&[]));
        for index in 0..2 {
            r.begin(index, [index + 1; 16]).unwrap();
            r.bind(index, 10, 20).unwrap();
            let receipt =
                CompletionReceipt::decode(&r.receipt_for(index).unwrap().encode().unwrap())
                    .unwrap();
            r.resolve(&receipt).unwrap();
        }
        let receipts = [r.receipt_for(0).unwrap(), r.receipt_for(1).unwrap()];
        assert!(r.all_receipts_match(&receipts));
        assert!(!r.all_receipts_match(&[]));
        let encoded = r.encode().unwrap();
        assert!(encoded.len() <= 4096);
        assert!(
            WindowsReaderRecord::parse(&encoded)
                .unwrap()
                .all_receipts_match(&receipts)
        );
    }
    #[test]
    fn execution_handoff_rejects_empty_intent_and_resolved_slot() {
        let mut record = fixture();
        assert!(record.pending_bound_identity(0).is_none());
        assert!(record.pending_bound_identity(2).is_none());
        record.begin(0, [4; 16]).unwrap();
        assert!(record.pending_bound_identity(0).is_none());
        let identity = record.bind(0, 10, 20).unwrap();
        assert_eq!(record.pending_bound_identity(0), Some(identity));
        let receipt = record.receipt_for(0).unwrap();
        record.resolve(&receipt).unwrap();
        assert_eq!(record.bound_identity(0), Some(identity));
        assert!(record.pending_bound_identity(0).is_none());
        assert!(record.all_receipts_match(&[receipt]));
    }
    #[test]
    fn illegal_order_reuse_and_foreign_receipt_preserve() {
        let mut r = fixture();
        assert!(r.begin(1, [4; 16]).is_err());
        r.begin(0, [4; 16]).unwrap();
        assert!(r.begin(0, [5; 16]).is_err());
        r.bind(0, 10, 20).unwrap();
        let before = r.encode().unwrap();
        let mut receipt = r.receipt_for(0).unwrap();
        receipt.record_digest = [9; 32];
        assert!(r.resolve(&receipt).is_err());
        assert_eq!(r.encode().unwrap(), before);
        let good = r.receipt_for(0).unwrap();
        r.resolve(&good).unwrap();
        assert!(r.begin(1, [4; 16]).is_err());
        assert!(!r.all_receipts_match(&[]));
        assert!(r.all_receipts_match(&[good]));
        assert!(r.resolve(&good).is_err());
        assert!(r.begin(0, [8; 16]).is_err());
    }
    #[test]
    fn malformed_json_and_identity_rejected() {
        let r = fixture();
        let mut json: serde_json::Value = serde_json::from_slice(&r.encode().unwrap()).unwrap();
        json["extra"] = true.into();
        assert!(WindowsReaderRecord::parse(&serde_json::to_vec(&json).unwrap()).is_err());
        assert!(WindowsReaderRecord::parse(&[b' '; 4097]).is_err());
        let mut r = fixture();
        r.anchors.copy.file_id = [0; 16];
        assert!(r.encode().is_err());
        let mut r = fixture();
        r.begin(0, [4; 16]).unwrap();
        r.bind(0, 10, 20).unwrap();
        if let Slot::Bound { identity } = &mut r.slots[0] {
            identity.tool_created += 1;
        }
        assert!(r.encode().is_err());
    }
    #[test]
    fn receipt_partial_reserved_and_domain_rejected() {
        let mut r = fixture();
        r.begin(0, [4; 16]).unwrap();
        r.bind(0, 10, 20).unwrap();
        let bytes = r.receipt_for(0).unwrap().encode().unwrap();
        for n in [0, 8, 96, 255] {
            assert!(CompletionReceipt::decode(&bytes[..n]).is_err());
        }
        for offset in [0, 8, 9, 10, 96, 248] {
            let mut bad = bytes;
            bad[offset] = 255;
            assert!(CompletionReceipt::decode(&bad).is_err());
        }
    }
    #[test]
    fn parsed_state_order_nonce_and_external_receipt_proof() {
        let mut r = fixture();
        r.begin(0, [4; 16]).unwrap();
        r.bind(0, 10, 20).unwrap();
        let receipt = r.receipt_for(0).unwrap();
        let mut forged: serde_json::Value = serde_json::from_slice(&r.encode().unwrap()).unwrap();
        forged["slots"][0]["state"] = "Resolved".into();
        let parsed = WindowsReaderRecord::parse(&serde_json::to_vec(&forged).unwrap()).unwrap();
        assert!(!parsed.all_receipts_match(&[]));
        assert!(parsed.all_receipts_match(&[receipt]));
        assert!(!parsed.all_receipts_match(&[receipt, receipt]));
        let mut bad_order = forged.clone();
        bad_order["slots"][1] = serde_json::json!({"state":"Intent","nonce":vec![5;16]});
        bad_order["slots"][0]["state"] = "Bound".into();
        assert!(WindowsReaderRecord::parse(&serde_json::to_vec(&bad_order).unwrap()).is_err());
        let mut same_nonce = forged.clone();
        same_nonce["slots"][1] = serde_json::json!({"state":"Intent","nonce":vec![4;16]});
        assert!(WindowsReaderRecord::parse(&serde_json::to_vec(&same_nonce).unwrap()).is_err());
        forged["slots"][0]["extra"] = true.into();
        assert!(WindowsReaderRecord::parse(&serde_json::to_vec(&forged).unwrap()).is_err());
    }
    #[test]
    fn parsed_identity_rejects_changed_original_anchors_and_domains() {
        let mut r = fixture();
        r.begin(0, [4; 16]).unwrap();
        r.bind(0, 10, 20).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&r.encode().unwrap()).unwrap();
        for pointer in [
            "/anchors/cache/volume_serial",
            "/anchors/owner/volume_serial",
            "/anchors/copy/volume_serial",
            "/anchors/journal/volume_serial",
            "/anchors/receipts/0/volume_serial",
            "/anchors/receipts/1/volume_serial",
            "/session_nonce/0",
            "/record_digest/0",
        ] {
            let mut changed = json.clone();
            *changed.pointer_mut(pointer).unwrap() = 9.into();
            assert!(WindowsReaderRecord::parse(&serde_json::to_vec(&changed).unwrap()).is_err());
        }
        let mut unix = json;
        unix["anchors"]["copy"] = serde_json::json!({"Unix":{"device":1,"inode":2}});
        assert!(WindowsReaderRecord::parse(&serde_json::to_vec(&unix).unwrap()).is_err());
    }
    #[test]
    fn pristine_is_lost_permanently_when_a_reader_starts() {
        let mut record = fixture();
        assert!(record.is_pristine());
        assert!(!record.all_receipts_match(&[]));
        record.begin(0, [4; 16]).unwrap();
        assert!(!record.is_pristine());
        record.bind(0, 10, 20).unwrap();
        assert!(!record.is_pristine());
        let receipt = record.receipt_for(0).unwrap();
        record.resolve(&receipt).unwrap();
        assert!(!record.is_pristine());
        assert!(record.all_receipts_match(&[receipt]));
        record.begin(1, [5; 16]).unwrap();
        assert!(!record.is_pristine());
        assert!(!record.all_receipts_match(&[receipt]));
    }
}
