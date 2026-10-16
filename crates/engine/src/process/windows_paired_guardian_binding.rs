//! Pure paired binding only. No handle, Job, durable receipt, or cleanup authority.
use crate::{
    cookies::windows_records::{BoundIdentity as CookieBoundIdentity, WindowsReaderRecord},
    tools::windows_managed_records::{
        BoundIdentity as ManagedBoundIdentity, ManagedReaderRecord, ReaderStage,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
const MAX_BYTES: usize = 32 * 1024;
const DOMAIN: &str = "dlpad-windows-managed-cookie-job-binding-v1";
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindingError {
    Bounds,
    Invalid,
}
impl std::fmt::Debug for BindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PairedGuardianBindingError")
    }
}
impl std::fmt::Display for BindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PairedGuardianBindingError")
    }
}
impl std::error::Error for BindingError {}
#[derive(Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PairedGuardianBinding {
    version: u8,
    domain: String,
    managed: ManagedReaderRecord,
    managed_slot: u8,
    cookie: WindowsReaderRecord,
    cookie_slot: u8,
    pair_digest: [u8; 32],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingWire {
    version: u8,
    domain: String,
    managed: ManagedReaderRecord,
    managed_slot: u8,
    cookie: WindowsReaderRecord,
    cookie_slot: u8,
    pair_digest: [u8; 32],
}
impl<'de> Deserialize<'de> for PairedGuardianBinding {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = BindingWire::deserialize(deserializer)?;
        let value = Self {
            version: wire.version,
            domain: wire.domain,
            managed: wire.managed,
            managed_slot: wire.managed_slot,
            cookie: wire.cookie,
            cookie_slot: wire.cookie_slot,
            pair_digest: wire.pair_digest,
        };
        value
            .validate()
            .map_err(|_| serde::de::Error::custom("PairedGuardianBindingError"))?;
        Ok(value)
    }
}
impl std::fmt::Debug for PairedGuardianBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PairedGuardianBindingRedacted")
    }
}
impl PairedGuardianBinding {
    pub(crate) fn new(
        managed: &ManagedReaderRecord,
        managed_slot: u8,
        cookie: &WindowsReaderRecord,
        cookie_slot: u8,
    ) -> Result<Self, BindingError> {
        let mut result = Self {
            version: 1,
            domain: DOMAIN.into(),
            managed: managed.clone(),
            managed_slot,
            cookie: cookie.clone(),
            cookie_slot,
            pair_digest: [0; 32],
        };
        let (m, c) = result.identities()?;
        result.pair_digest = Self::hash(m, c);
        result.validate()?;
        Ok(result)
    }
    pub(crate) fn from_intents(
        managed: &ManagedReaderRecord,
        managed_slot: u8,
        cookie: &WindowsReaderRecord,
        cookie_slot: u8,
        creation: (u64, u64),
    ) -> Result<Self, BindingError> {
        let mut m = managed.clone();
        let mut c = cookie.clone();
        m.bind(managed_slot, creation.0, creation.1)
            .map_err(|_| BindingError::Invalid)?;
        c.bind(cookie_slot, creation.0, creation.1)
            .map_err(|_| BindingError::Invalid)?;
        Self::new(&m, managed_slot, &c, cookie_slot)
    }
    fn identities(&self) -> Result<(ManagedBoundIdentity, CookieBoundIdentity), BindingError> {
        // Reuse each original format's canonical identity-hash and anchor validation.
        let mbytes = self.managed.encode().map_err(|_| BindingError::Invalid)?;
        let cbytes = self.cookie.encode().map_err(|_| BindingError::Invalid)?;
        ManagedReaderRecord::parse(&mbytes).map_err(|_| BindingError::Invalid)?;
        WindowsReaderRecord::parse(&cbytes).map_err(|_| BindingError::Invalid)?;
        let ma = self.managed.anchors();
        let ca = self.cookie.anchors();
        let managed_files = [
            ma.marker,
            ma.lock,
            ma.journal,
            ma.receipts[0],
            ma.receipts[1],
            ma.receipts[2],
            ma.receipts[3],
            ma.receipts[4],
            ma.receipts[5],
            ma.receipts[6],
            ma.receipts[7],
        ];
        let cookie_files = [ca.copy, ca.journal, ca.receipts[0], ca.receipts[1]];
        if cookie_files.iter().any(|c| {
            managed_files
                .iter()
                .any(|m| c.volume_serial == m.volume_serial && c.file_id == m.file_id)
        }) {
            return Err(BindingError::Invalid);
        }
        let m = self
            .managed
            .pending_bound_identity(self.managed_slot)
            .ok_or(BindingError::Invalid)?;
        let c = self
            .cookie
            .pending_bound_identity(self.cookie_slot)
            .ok_or(BindingError::Invalid)?;
        if m.guardian_created != c.guardian_created
            || m.tool_created != c.tool_created
            || m.guardian_created == 0
            || m.tool_created == 0
            || m.nonce == [0; 16]
            || c.nonce == [0; 16]
            || m.identity_digest == [0; 32]
            || c.identity_digest == [0; 32]
            || !matches!(
                (m.stage, c.index),
                (ReaderStage::Metadata, 0) | (ReaderStage::Download, 1)
            )
        {
            return Err(BindingError::Invalid);
        }
        Ok((m, c))
    }
    fn hash(m: ManagedBoundIdentity, c: CookieBoundIdentity) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(DOMAIN.as_bytes());
        h.update([0, 1]);
        h.update(b"managed\0");
        h.update([
            m.index,
            match m.stage {
                ReaderStage::Metadata => 2,
                ReaderStage::Download => 3,
                _ => 0,
            },
        ]);
        h.update(m.nonce);
        h.update(m.identity_digest);
        h.update(b"cookie\0");
        h.update([c.index]);
        h.update(c.nonce);
        h.update(c.identity_digest);
        h.update(m.guardian_created.to_le_bytes());
        h.update(m.tool_created.to_le_bytes());
        h.finalize().into()
    }
    fn validate(&self) -> Result<(), BindingError> {
        if self.version != 1 || self.domain != DOMAIN {
            return Err(BindingError::Invalid);
        }
        let (m, c) = self.identities()?;
        if self.pair_digest == [0; 32] || self.pair_digest != Self::hash(m, c) {
            return Err(BindingError::Invalid);
        }
        Ok(())
    }
    pub(crate) fn encode(&self) -> Result<Vec<u8>, BindingError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| BindingError::Invalid)?;
        if bytes.len() > MAX_BYTES {
            return Err(BindingError::Bounds);
        }
        Ok(bytes)
    }
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, BindingError> {
        if bytes.is_empty() || bytes.len() > MAX_BYTES {
            return Err(BindingError::Bounds);
        }
        let value: Self = serde_json::from_slice(bytes).map_err(|_| BindingError::Invalid)?;
        value.validate()?;
        Ok(value)
    }
    pub(crate) fn managed_identity(&self) -> ManagedBoundIdentity {
        self.managed
            .pending_bound_identity(self.managed_slot)
            .expect("validated paired managed identity")
    }
    pub(crate) fn cookie_identity(&self) -> CookieBoundIdentity {
        self.cookie
            .pending_bound_identity(self.cookie_slot)
            .expect("validated paired cookie identity")
    }
    pub(crate) fn digest(&self) -> [u8; 32] {
        self.pair_digest
    }
    pub(crate) fn creation(&self) -> (u64, u64) {
        let m = self.managed_identity();
        (m.guardian_created, m.tool_created)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cookies::windows_records::{
            SessionAnchors as CookieAnchors, WindowsAnchor as CookieAnchor,
        },
        tools::windows_managed_records::{
            SessionAnchors as ManagedAnchors, WindowsAnchor as ManagedAnchor,
        },
    };
    fn intents(stage: ReaderStage, cookie_slot: u8) -> (ManagedReaderRecord, WindowsReaderRecord) {
        let ma = |n| ManagedAnchor {
            volume_serial: 1,
            file_id: [n; 16],
        };
        let ca = |n| CookieAnchor {
            volume_serial: 2,
            file_id: [n; 16],
        };
        let mut m = ManagedReaderRecord::new(
            [1; 16],
            [2; 32],
            ManagedAnchors {
                root: ma(1),
                marker: ma(2),
                lock: ma(3),
                journal: ma(4),
                receipts: std::array::from_fn(|i| ma(i as u8 + 5)),
            },
        )
        .unwrap();
        m.begin(0, stage, [3; 16]).unwrap();
        let mut c = WindowsReaderRecord::new(
            [4; 16],
            [5; 32],
            CookieAnchors {
                cache: ca(1),
                owner: ca(2),
                copy: ca(3),
                journal: ca(4),
                receipts: [ca(5), ca(6)],
            },
        )
        .unwrap();
        c.begin(0, [6; 16]).unwrap();
        if cookie_slot == 1 {
            c.bind(0, 10, 20).unwrap();
            let r = c.receipt_for(0).unwrap();
            c.resolve(&r).unwrap();
            c.begin(1, [7; 16]).unwrap();
        }
        (m, c)
    }
    #[test]
    fn exact_domains_stage_mapping_and_roundtrip() {
        for (stage, slot) in [(ReaderStage::Metadata, 0), (ReaderStage::Download, 1)] {
            let (m, c) = intents(stage, slot);
            let pair = PairedGuardianBinding::from_intents(&m, 0, &c, slot, (30, 40)).unwrap();
            assert_eq!(pair.creation(), (30, 40));
            assert_eq!(pair.cookie_identity().index, slot);
            assert_ne!(pair.digest(), pair.managed_identity().identity_digest);
            assert_ne!(pair.digest(), pair.cookie_identity().identity_digest);
            let bytes = pair.encode().unwrap();
            assert!(bytes.len() <= MAX_BYTES);
            let decoded = PairedGuardianBinding::decode(&bytes).unwrap();
            assert_eq!(decoded.digest(), pair.digest());
            assert!(ManagedReaderRecord::parse(&bytes).is_err());
            assert!(WindowsReaderRecord::parse(&bytes).is_err());
            assert_eq!(format!("{pair:?}"), "PairedGuardianBindingRedacted");
        }
    }
    #[test]
    fn mismatched_creation_stage_slot_and_zero_rejected() {
        let (m, c) = intents(ReaderStage::Metadata, 0);
        for creation in [(0, 1), (1, 0)] {
            assert!(PairedGuardianBinding::from_intents(&m, 0, &c, 0, creation).is_err());
        }
        let mut mb = m.clone();
        let mut cb = c.clone();
        mb.bind(0, 10, 20).unwrap();
        cb.bind(0, 10, 21).unwrap();
        assert!(PairedGuardianBinding::new(&mb, 0, &cb, 0).is_err());
        assert!(PairedGuardianBinding::from_intents(&m, 8, &c, 0, (10, 20)).is_err());
        assert!(PairedGuardianBinding::from_intents(&m, 0, &c, 2, (10, 20)).is_err());
        for (stage, slot) in [
            (ReaderStage::VersionProbe, 0),
            (ReaderStage::DirectProcessing, 0),
            (ReaderStage::Download, 0),
            (ReaderStage::Metadata, 1),
        ] {
            let (m, c) = intents(stage, slot);
            assert!(PairedGuardianBinding::from_intents(&m, 0, &c, slot, (10, 20)).is_err());
        }
    }
    #[test]
    fn unknown_fields_tampered_anchors_digest_domains_and_bounds_rejected() {
        let (m, c) = intents(ReaderStage::Metadata, 0);
        let pair = PairedGuardianBinding::from_intents(&m, 0, &c, 0, (10, 20)).unwrap();
        let bytes = pair.encode().unwrap();
        let original: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        for path in [
            "/version",
            "/managed_slot",
            "/cookie_slot",
            "/managed/anchors/root/volume_serial",
            "/cookie/anchors/copy/volume_serial",
            "/managed/slots/0/identity/identity_digest/0",
            "/cookie/slots/0/identity/nonce/0",
            "/pair_digest/0",
        ] {
            let mut value = original.clone();
            let target = value.pointer_mut(path).unwrap();
            let n = target.as_u64().unwrap();
            *target = (n + 1).into();
            let changed = serde_json::to_vec(&value).unwrap();
            assert!(PairedGuardianBinding::decode(&changed).is_err(), "{path}");
            assert!(
                serde_json::from_slice::<PairedGuardianBinding>(&changed).is_err(),
                "nested {path}"
            );
        }
        let mut value = original.clone();
        value["domain"] = "dlpad-windows-cookie-reader-v1".into();
        assert!(PairedGuardianBinding::decode(&serde_json::to_vec(&value).unwrap()).is_err());
        value = original;
        value["foreign"] = true.into();
        assert!(PairedGuardianBinding::decode(&serde_json::to_vec(&value).unwrap()).is_err());
        assert!(matches!(
            PairedGuardianBinding::decode(&vec![b' '; MAX_BYTES + 1]),
            Err(BindingError::Bounds)
        ));
        for input in [&bytes[..bytes.len() - 1], b"{}", b"\xff"] {
            assert!(PairedGuardianBinding::decode(input).is_err());
        }
        assert!(PairedGuardianBinding::decode(&m.encode().unwrap()).is_err());
        assert!(PairedGuardianBinding::decode(&c.encode().unwrap()).is_err());
    }
    #[test]
    fn nested_deserialize_validates_and_cross_domain_file_aliases_reject() {
        let (m, c) = intents(ReaderStage::Metadata, 0);
        let valid = PairedGuardianBinding::from_intents(&m, 0, &c, 0, (10, 20)).unwrap();
        let nested: PairedGuardianBinding =
            serde_json::from_slice(&valid.encode().unwrap()).unwrap();
        assert_eq!(nested.creation(), (10, 20));
        let ma = m.anchors();
        for slot in [0, 1, 2, 3] {
            let mut anchors = c.anchors();
            let alias = CookieAnchor {
                volume_serial: ma.marker.volume_serial,
                file_id: ma.marker.file_id,
            };
            match slot {
                0 => anchors.copy = alias,
                1 => anchors.journal = alias,
                2 => anchors.receipts[0] = alias,
                _ => anchors.receipts[1] = alias,
            }
            let mut cookie = WindowsReaderRecord::new([4; 16], [5; 32], anchors).unwrap();
            cookie.begin(0, [6; 16]).unwrap();
            assert!(PairedGuardianBinding::from_intents(&m, 0, &cookie, 0, (10, 20)).is_err());
        }
        // Shared directory ancestors are permitted; no file alias or ancestry proof is inferred.
        let mut anchors = c.anchors();
        anchors.cache = CookieAnchor {
            volume_serial: ma.root.volume_serial,
            file_id: ma.root.file_id,
        };
        let mut cookie = WindowsReaderRecord::new([4; 16], [5; 32], anchors).unwrap();
        cookie.begin(0, [6; 16]).unwrap();
        assert!(PairedGuardianBinding::from_intents(&m, 0, &cookie, 0, (10, 20)).is_ok());
    }
}
