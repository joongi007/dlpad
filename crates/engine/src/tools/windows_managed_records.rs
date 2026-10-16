//! Pure managed Windows reader format; no native HANDLE/ACL/Job or deletion authority.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
const MAX_RECORD: usize = 16 * 1024;
const RECEIPT_LEN: usize = 256;
const MAGIC: &[u8; 8] = b"DLPMRC01";
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecordError {
    Invalid,
    Bounds,
    Transition,
    Receipt,
}
impl std::fmt::Debug for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ManagedRecordError")
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReaderStage {
    VersionProbe,
    Metadata,
    Download,
    DirectProcessing,
}
impl ReaderStage {
    fn code(self) -> u8 {
        match self {
            Self::VersionProbe => 1,
            Self::Metadata => 2,
            Self::Download => 3,
            Self::DirectProcessing => 4,
        }
    }
}
impl std::fmt::Debug for ReaderStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ManagedStage")
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
    pub root: WindowsAnchor,
    pub marker: WindowsAnchor,
    pub lock: WindowsAnchor,
    pub journal: WindowsAnchor,
    pub receipts: [WindowsAnchor; 8],
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BoundIdentity {
    pub index: u8,
    pub stage: ReaderStage,
    pub nonce: [u8; 16],
    pub guardian_created: u64,
    pub tool_created: u64,
    pub identity_digest: [u8; 32],
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", deny_unknown_fields)]
enum Slot {
    Empty,
    Intent { nonce: [u8; 16], stage: ReaderStage },
    Bound { identity: BoundIdentity },
    Resolved { identity: BoundIdentity },
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManagedReaderRecord {
    version: u8,
    session_nonce: [u8; 16],
    record_digest: [u8; 32],
    anchors: SessionAnchors,
    slots: [Slot; 8],
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct CompletionReceipt {
    index: u8,
    stage: ReaderStage,
    reader_nonce: [u8; 16],
    guardian_created: u64,
    tool_created: u64,
    session_nonce: [u8; 16],
    record_digest: [u8; 32],
    identity_digest: [u8; 32],
}
macro_rules! redacted { ($($t:ty),*) => { $(impl std::fmt::Debug for $t {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("ManagedReaderRedacted")}})* }; }
redacted!(
    WindowsAnchor,
    SessionAnchors,
    BoundIdentity,
    ManagedReaderRecord,
    CompletionReceipt
);
fn nonzero(bytes: &[u8]) -> bool {
    bytes.iter().any(|b| *b != 0)
}
impl ManagedReaderRecord {
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
            slots: std::array::from_fn(|_| Slot::Empty),
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
    fn identity_digest(
        &self,
        index: u8,
        stage: ReaderStage,
        nonce: [u8; 16],
        guardian: u64,
        tool: u64,
    ) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(b"DLPAD-WINDOWS-MANAGED-READER-V1\0");
        h.update(self.session_nonce);
        h.update(self.record_digest);
        for a in [
            self.anchors.root,
            self.anchors.marker,
            self.anchors.lock,
            self.anchors.journal,
            self.anchors.receipts[0],
            self.anchors.receipts[1],
            self.anchors.receipts[2],
            self.anchors.receipts[3],
            self.anchors.receipts[4],
            self.anchors.receipts[5],
            self.anchors.receipts[6],
            self.anchors.receipts[7],
        ] {
            h.update(a.volume_serial.to_le_bytes());
            h.update(a.file_id);
        }
        h.update([index, stage.code()]);
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
            self.anchors.root,
            self.anchors.marker,
            self.anchors.lock,
            self.anchors.journal,
            self.anchors.receipts[0],
            self.anchors.receipts[1],
            self.anchors.receipts[2],
            self.anchors.receipts[3],
            self.anchors.receipts[4],
            self.anchors.receipts[5],
            self.anchors.receipts[6],
            self.anchors.receipts[7],
        ] {
            if !nonzero(&a.file_id) {
                return Err(RecordError::Invalid);
            }
        }
        let mut ids = std::collections::BTreeSet::new();
        for a in [
            self.anchors.root,
            self.anchors.marker,
            self.anchors.lock,
            self.anchors.journal,
        ]
        .into_iter()
        .chain(self.anchors.receipts)
        {
            if !ids.insert((a.volume_serial, a.file_id)) {
                return Err(RecordError::Invalid);
            }
        }
        let mut seen_empty = false;
        let mut nonces = std::collections::BTreeSet::new();
        for (index, slot) in self.slots.iter().enumerate() {
            if matches!(slot, Slot::Empty) {
                seen_empty = true;
                continue;
            }
            if seen_empty || (index > 0 && !matches!(self.slots[index - 1], Slot::Resolved { .. }))
            {
                return Err(RecordError::Transition);
            }
            let nonce = match slot {
                Slot::Intent { nonce, .. } => *nonce,
                Slot::Bound { identity } | Slot::Resolved { identity } => identity.nonce,
                Slot::Empty => unreachable!(),
            };
            if !nonces.insert(nonce) {
                return Err(RecordError::Invalid);
            }
        }
        for (index, slot) in self.slots.iter().enumerate() {
            match slot {
                Slot::Empty => {}
                Slot::Intent { nonce, .. } => {
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
                                i.stage,
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
    pub(crate) fn begin(
        &mut self,
        index: u8,
        stage: ReaderStage,
        nonce: [u8; 16],
    ) -> Result<(), RecordError> {
        self.validate()?;
        if index >= 8
            || !nonzero(&nonce)
            || !matches!(self.slots[index as usize], Slot::Empty)
            || (index > 0 && !matches!(self.slots[index as usize - 1], Slot::Resolved { .. }))
        {
            return Err(RecordError::Transition);
        }
        if self.slots.iter().any(|slot| match slot {
            Slot::Intent { nonce: n, .. } => *n == nonce,
            Slot::Bound { identity } | Slot::Resolved { identity } => identity.nonce == nonce,
            Slot::Empty => false,
        }) {
            return Err(RecordError::Transition);
        }
        self.slots[index as usize] = Slot::Intent { nonce, stage };
        Ok(())
    }
    pub(crate) fn bind(
        &mut self,
        index: u8,
        guardian_created: u64,
        tool_created: u64,
    ) -> Result<BoundIdentity, RecordError> {
        self.validate()?;
        if index >= 8 || guardian_created == 0 || tool_created == 0 {
            return Err(RecordError::Invalid);
        }
        let Slot::Intent { nonce, stage } = self.slots[index as usize] else {
            return Err(RecordError::Transition);
        };
        let identity = BoundIdentity {
            index,
            stage,
            nonce,
            guardian_created,
            tool_created,
            identity_digest: self.identity_digest(
                index,
                stage,
                nonce,
                guardian_created,
                tool_created,
            ),
        };
        self.slots[index as usize] = Slot::Bound { identity };
        Ok(identity)
    }
    /// Pure expected encoding only; never proof of process termination.
    /// Actual writer provenance and original native receipt handles are caller-owned.
    pub(crate) fn receipt_for(&self, index: u8) -> Result<CompletionReceipt, RecordError> {
        self.validate()?;
        let Some(Slot::Bound { identity } | Slot::Resolved { identity }) =
            self.slots.get(index as usize)
        else {
            return Err(RecordError::Transition);
        };
        Ok(CompletionReceipt {
            index,
            stage: identity.stage,
            reader_nonce: identity.nonce,
            guardian_created: identity.guardian_created,
            tool_created: identity.tool_created,
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
    /// Caller must first pin original receipt IDs, ACLs and native terminal writer.
    /// Parsing matching bytes alone never grants deletion or Job authority.
    pub(crate) fn all_receipt_bytes_match(&self, bytes: &[&[u8]]) -> bool {
        if bytes.is_empty() || bytes.len() > 8 {
            return false;
        }
        let parsed: Result<Vec<_>, _> =
            bytes.iter().map(|b| CompletionReceipt::decode(b)).collect();
        parsed.is_ok_and(|receipts| self.all_receipts_match(&receipts))
    }
    pub(crate) fn all_receipts_match(&self, receipts: &[CompletionReceipt]) -> bool {
        if self.validate().is_err() || receipts.is_empty() || receipts.len() > 8 {
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
        if !nonzero(&self.reader_nonce)
            || self.guardian_created == 0
            || self.tool_created == 0
            || self.index >= 8
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
        out[10] = self.stage.code();
        out[96..112].copy_from_slice(&self.reader_nonce);
        out[112..120].copy_from_slice(&self.guardian_created.to_le_bytes());
        out[120..128].copy_from_slice(&self.tool_created.to_le_bytes());
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
            || bytes[11..16]
                .iter()
                .chain(bytes[128..248].iter())
                .any(|b| *b != 0)
            || &bytes[248..] != b"COMPLETE"
        {
            return Err(RecordError::Receipt);
        }
        let value = Self {
            index: bytes[9],
            stage: match bytes[10] {
                1 => ReaderStage::VersionProbe,
                2 => ReaderStage::Metadata,
                3 => ReaderStage::Download,
                4 => ReaderStage::DirectProcessing,
                _ => return Err(RecordError::Receipt),
            },
            reader_nonce: bytes[96..112]
                .try_into()
                .map_err(|_| RecordError::Receipt)?,
            guardian_created: u64::from_le_bytes(
                bytes[112..120]
                    .try_into()
                    .map_err(|_| RecordError::Receipt)?,
            ),
            tool_created: u64::from_le_bytes(
                bytes[120..128]
                    .try_into()
                    .map_err(|_| RecordError::Receipt)?,
            ),
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
    fn record() -> ManagedReaderRecord {
        let anchor = |id: u8| WindowsAnchor {
            volume_serial: 1,
            file_id: [id; 16],
        };
        ManagedReaderRecord::new(
            [1; 16],
            [2; 32],
            SessionAnchors {
                root: anchor(1),
                marker: anchor(2),
                lock: anchor(3),
                journal: anchor(4),
                receipts: std::array::from_fn(|i| anchor(i as u8 + 5)),
            },
        )
        .unwrap()
    }
    #[test]
    fn eight_single_use_readers_require_actual_receipts() {
        let mut r = record();
        assert!(!r.all_receipts_match(&[]));
        let mut receipts = Vec::new();
        for index in 0..8 {
            r.begin(index, ReaderStage::VersionProbe, [index + 10; 16])
                .unwrap();
            assert!(r.begin(index, ReaderStage::Metadata, [99; 16]).is_err());
            r.bind(index, 100 + u64::from(index), 200 + u64::from(index))
                .unwrap();
            let receipt = r.receipt_for(index).unwrap();
            let original = CompletionReceipt::decode(&receipt.encode().unwrap()).unwrap();
            r.resolve(&original).unwrap();
            receipts.push(original);
            assert!(r.all_receipts_match(&receipts));
        }
        assert!(r.begin(8, ReaderStage::Download, [90; 16]).is_err());
        assert!(r.begin(0, ReaderStage::Download, [90; 16]).is_err());
        let parsed = ManagedReaderRecord::parse(&r.encode().unwrap()).unwrap();
        assert!(!parsed.all_receipts_match(&[]));
        assert!(parsed.all_receipts_match(&receipts));
        assert!(!parsed.all_receipts_match(&receipts[..7]));
    }
    #[test]
    fn illegal_order_stage_nonce_and_anchor_changes_rejected() {
        let mut r = record();
        assert!(r.begin(1, ReaderStage::Download, [10; 16]).is_err());
        assert!(r.bind(0, 1, 1).is_err());
        r.begin(0, ReaderStage::Metadata, [10; 16]).unwrap();
        assert!(!r.all_receipts_match(&[]));
        assert!(r.bind(0, 0, 1).is_err());
        r.bind(0, 1, 2).unwrap();
        let receipt = r.receipt_for(0).unwrap();
        let mut wrong = receipt;
        wrong.stage = ReaderStage::Download;
        assert!(r.resolve(&wrong).is_err());
        wrong = receipt;
        wrong.reader_nonce = [3; 16];
        assert!(r.resolve(&wrong).is_err());
        r.resolve(&receipt).unwrap();
        assert!(r.begin(1, ReaderStage::Download, [10; 16]).is_err());
        let mut json = serde_json::to_value(&r).unwrap();
        json["anchors"]["lock"]["file_id"] = json["anchors"]["root"]["file_id"].clone();
        assert!(ManagedReaderRecord::parse(&serde_json::to_vec(&json).unwrap()).is_err());
        let mut json = serde_json::to_value(&r).unwrap();
        json["slots"][0]["identity"]["stage"] = serde_json::json!("Download");
        assert!(ManagedReaderRecord::parse(&serde_json::to_vec(&json).unwrap()).is_err());
    }
    #[test]
    fn receipt_domain_reserved_partial_and_mismatch() {
        let mut r = record();
        r.begin(0, ReaderStage::DirectProcessing, [10; 16]).unwrap();
        r.bind(0, 1, 2).unwrap();
        let receipt = r.receipt_for(0).unwrap();
        let bytes = receipt.encode().unwrap();
        for offset in [0, 8, 9, 10, 11, 128, 248] {
            let mut bad = bytes;
            bad[offset] = 255;
            assert!(CompletionReceipt::decode(&bad).is_err());
        }
        assert!(CompletionReceipt::decode(&bytes[..255]).is_err());
        let mut wrong = receipt;
        wrong.record_digest = [5; 32];
        assert!(!r.all_receipts_match(&[wrong]));
        assert!(!r.all_receipts_match(&[receipt, receipt]));
        r.resolve(&receipt).unwrap();
        assert!(!r.all_receipts_match(&[]));
    }
    #[test]
    fn journal_unknown_oversize_and_state_forgery() {
        let r = record();
        let mut json = serde_json::to_value(&r).unwrap();
        json["secret"] = serde_json::json!("no");
        assert!(ManagedReaderRecord::parse(&serde_json::to_vec(&json).unwrap()).is_err());
        assert!(ManagedReaderRecord::parse(&vec![b' '; MAX_RECORD + 1]).is_err());
        let mut json = serde_json::to_value(&r).unwrap();
        json["slots"][2] = serde_json::json!({"state":"Intent","nonce":[9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9],"stage":"Download"});
        assert!(ManagedReaderRecord::parse(&serde_json::to_vec(&json).unwrap()).is_err());
    }
}
