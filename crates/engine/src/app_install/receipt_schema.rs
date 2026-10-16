//! Bounded external registration consistency receipt. No delete or publisher authority.
use super::registration_schema::{RegistrationRecord, RegistrationSeal, validate_binding};
use crate::file_identity::NativeFileIdentity;
use serde::{Deserialize, Serialize};
use std::io::{self, Read};
const LIMIT: usize = 65536;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptRecord {
    pub schema: u32,
    pub domain: String,
    pub receipt_root_identity: NativeFileIdentity,
    pub receipt_identity: NativeFileIdentity,
    pub registration: RegistrationRecord,
    pub seal: RegistrationSeal,
}
fn bad() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid registration receipt")
}
fn volume(id: &NativeFileIdentity) -> io::Result<u64> {
    match id {
        NativeFileIdentity::Windows {
            volume_serial,
            file_id,
        } if *volume_serial != 0 && file_id.iter().any(|b| *b != 0) => Ok(*volume_serial),
        _ => Err(bad()),
    }
}
impl ReceiptRecord {
    pub fn read(reader: impl Read) -> io::Result<Self> {
        let mut bytes = Vec::new();
        reader.take((LIMIT + 1) as u64).read_to_end(&mut bytes)?;
        if bytes.is_empty() || bytes.len() > LIMIT {
            return Err(bad());
        }
        let value: Self = serde_json::from_slice(&bytes).map_err(|_| bad())?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> io::Result<()> {
        if self.schema != 1 || self.domain != "dlpad-app-registration-receipt-v1" {
            return Err(bad());
        }
        validate_binding(&self.registration, &self.seal)?;
        if volume(&self.receipt_root_identity)? != volume(&self.receipt_identity)?
            || self.receipt_root_identity == self.receipt_identity
        {
            return Err(bad());
        }
        let r = &self.registration;
        for id in [
            &r.root_identity,
            &r.record_identity,
            &r.seal_identity,
            &r.ready_identity,
        ]
        .into_iter()
        .chain(r.prepared.entries.iter().map(|e| &e.identity))
        {
            if id == &self.receipt_root_identity || id == &self.receipt_identity {
                return Err(bad());
            }
        }
        if serde_json::to_vec(self).map_err(|_| bad())?.len() > LIMIT {
            return Err(bad());
        }
        Ok(())
    }
}
pub(super) fn serialize_receipt(record: &ReceiptRecord) -> io::Result<Vec<u8>> {
    record.validate()?;
    serde_json::to_vec(record).map_err(|_| bad())
}
#[cfg(test)]
mod tests {
    use super::super::registration_schema::record_digest;
    use super::super::{PreparedEntry, PreparedRecord};
    use super::*;
    use sha2::{Digest, Sha256};
    fn id(n: u8, volume: u64) -> NativeFileIdentity {
        NativeFileIdentity::Windows {
            volume_serial: volume,
            file_id: [n; 16],
        }
    }
    fn record() -> ReceiptRecord {
        let p = PreparedRecord {
            schema: 1,
            domain: "dlpad-prepared-app-v1".into(),
            root_identity: id(1, 1),
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-gnu".into(),
            development_only: true,
            trusted_provenance: false,
            entries: vec![PreparedEntry {
                name: "dlpad.exe".into(),
                bytes: 256,
                sha256: "1".repeat(64),
                identity: id(5, 1),
            }],
        };
        let r = RegistrationRecord {
            schema: 1,
            domain: "dlpad-app-registration-v1".into(),
            nonce: "a".repeat(64),
            root_identity: id(1, 1),
            record_identity: id(2, 1),
            seal_identity: id(3, 1),
            ready_identity: id(4, 1),
            prepared_sha256: format!("{:x}", Sha256::digest(serde_json::to_vec(&p).unwrap())),
            prepared: p,
        };
        let seal = RegistrationSeal {
            schema: 1,
            domain: "dlpad-app-registration-seal-v1".into(),
            nonce: r.nonce.clone(),
            root_identity: r.root_identity.clone(),
            record_identity: r.record_identity.clone(),
            seal_identity: r.seal_identity.clone(),
            record_sha256: record_digest(&r).unwrap(),
        };
        ReceiptRecord {
            schema: 1,
            domain: "dlpad-app-registration-receipt-v1".into(),
            receipt_root_identity: id(6, 2),
            receipt_identity: id(7, 2),
            registration: r,
            seal,
        }
    }
    #[test]
    fn bounded_receipt_cross_volume_and_nested_binding() {
        let mut r = record();
        r.receipt_root_identity = id(1, 2);
        r.receipt_identity = id(2, 2);
        let bytes = serialize_receipt(&r).unwrap();
        let decoded = ReceiptRecord::read(bytes.as_slice()).unwrap();
        assert_eq!(serialize_receipt(&decoded).unwrap(), bytes);
        let mut changed = r;
        changed.seal.nonce = "b".repeat(64);
        assert!(changed.validate().is_err());
    }
    #[test]
    fn rejects_alias_invalid_identity_and_privilege_tamper() {
        for n in 1..=5 {
            let mut r = record();
            r.receipt_identity = id(n, 1);
            r.receipt_root_identity = id(6, 1);
            assert!(r.validate().is_err());
        }
        let mut r = record();
        r.receipt_identity = r.receipt_root_identity.clone();
        assert!(r.validate().is_err());
        let mut r = record();
        r.receipt_identity = id(0, 2);
        assert!(r.validate().is_err());
        let mut r = record();
        r.receipt_root_identity = id(6, 0);
        assert!(r.validate().is_err());
        let mut r = record();
        r.registration.prepared.trusted_provenance = true;
        assert!(r.validate().is_err());
        let mut r = record();
        r.registration.prepared.development_only = false;
        assert!(r.validate().is_err());
    }
    #[test]
    fn rejects_unknown_duplicate_oversize_and_wrong_domain() {
        let mut v = serde_json::to_value(record()).unwrap();
        v["unknown"] = true.into();
        assert!(ReceiptRecord::read(serde_json::to_vec(&v).unwrap().as_slice()).is_err());
        let valid = String::from_utf8(serialize_receipt(&record()).unwrap()).unwrap();
        let duplicate = valid.replacen("{", "{\"schema\":1,", 1);
        assert!(ReceiptRecord::read(duplicate.as_bytes()).is_err());
        assert!(ReceiptRecord::read(vec![b' '; LIMIT + 1].as_slice()).is_err());
        let mut r = record();
        r.domain = "other".into();
        assert!(r.validate().is_err());
    }
}
