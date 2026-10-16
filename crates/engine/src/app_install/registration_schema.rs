//! Bounded development-registration consistency records, never native ownership authority.
use super::{
    PreparedRecord,
    manifest::{BundleEntry, BundleManifest},
};
use crate::file_identity::NativeFileIdentity;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{self, Read};
const RECORD_LIMIT: usize = 65536;
const SEAL_LIMIT: usize = 4096;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistrationRecord {
    pub schema: u32,
    pub domain: String,
    pub nonce: String,
    pub root_identity: NativeFileIdentity,
    pub record_identity: NativeFileIdentity,
    pub seal_identity: NativeFileIdentity,
    pub ready_identity: NativeFileIdentity,
    pub prepared_sha256: String,
    pub prepared: PreparedRecord,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistrationSeal {
    pub schema: u32,
    pub domain: String,
    pub nonce: String,
    pub root_identity: NativeFileIdentity,
    pub record_identity: NativeFileIdentity,
    pub seal_identity: NativeFileIdentity,
    pub record_sha256: String,
}
fn bad() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "invalid development registration record",
    )
}
fn hex(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
}
fn anchors(
    root: &NativeFileIdentity,
    record: &NativeFileIdentity,
    seal: &NativeFileIdentity,
) -> io::Result<u64> {
    fn volume(id: &NativeFileIdentity) -> io::Result<u64> {
        match id {
            NativeFileIdentity::Windows {
                volume_serial,
                file_id,
            } if *volume_serial != 0 && file_id.iter().any(|b| *b != 0) => Ok(*volume_serial),
            _ => Err(bad()),
        }
    }
    let v = volume(root)?;
    if volume(record)? != v
        || volume(seal)? != v
        || root == record
        || root == seal
        || record == seal
    {
        return Err(bad());
    }
    Ok(v)
}
fn bounded<T: serde::de::DeserializeOwned>(reader: impl Read, limit: usize) -> io::Result<T> {
    let mut bytes = Vec::new();
    reader.take((limit + 1) as u64).read_to_end(&mut bytes)?;
    if bytes.is_empty() || bytes.len() > limit {
        return Err(bad());
    }
    serde_json::from_slice(&bytes).map_err(|_| bad())
}
impl RegistrationRecord {
    pub fn read(reader: impl Read) -> io::Result<Self> {
        let value: Self = bounded(reader, RECORD_LIMIT)?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> io::Result<()> {
        if self.schema != 1
            || self.domain != "dlpad-app-registration-v1"
            || !hex(&self.nonce)
            || self.nonce.bytes().all(|b| b == b'0')
            || !hex(&self.prepared_sha256)
        {
            return Err(bad());
        }
        let v = anchors(
            &self.root_identity,
            &self.record_identity,
            &self.seal_identity,
        )?;
        match &self.ready_identity {
            NativeFileIdentity::Windows {
                volume_serial,
                file_id,
            } if *volume_serial == v && file_id.iter().any(|b| *b != 0) => {}
            _ => return Err(bad()),
        }
        if self.ready_identity == self.root_identity
            || self.ready_identity == self.record_identity
            || self.ready_identity == self.seal_identity
        {
            return Err(bad());
        }
        let p = &self.prepared;
        if p.schema != 1
            || p.domain != "dlpad-prepared-app-v1"
            || p.trusted_provenance
            || !p.development_only
            || p.root_identity != self.root_identity
        {
            return Err(bad());
        }
        let descriptor = BundleManifest {
            schema: p.schema,
            version: p.version.clone(),
            target: p.target.clone(),
            development_only: p.development_only,
            entries: p
                .entries
                .iter()
                .map(|e| BundleEntry {
                    name: e.name.clone(),
                    bytes: e.bytes,
                    sha256: e.sha256.clone(),
                })
                .collect(),
        };
        descriptor.validate().map_err(|_| bad())?;
        for (i, e) in p.entries.iter().enumerate() {
            match &e.identity {
                NativeFileIdentity::Windows {
                    volume_serial,
                    file_id,
                } if *volume_serial == v && file_id.iter().any(|b| *b != 0) => {}
                _ => return Err(bad()),
            }
            if e.identity == self.ready_identity
                || e.identity == self.root_identity
                || e.identity == self.record_identity
                || e.identity == self.seal_identity
                || p.entries[..i]
                    .iter()
                    .any(|prior| prior.identity == e.identity)
            {
                return Err(bad());
            }
        }
        let bytes = serde_json::to_vec(p).map_err(|_| bad())?;
        if bytes.len() > RECORD_LIMIT
            || format!("{:x}", Sha256::digest(&bytes)) != self.prepared_sha256
        {
            return Err(bad());
        }
        Ok(())
    }
}
impl RegistrationSeal {
    pub fn read(reader: impl Read) -> io::Result<Self> {
        let value: Self = bounded(reader, SEAL_LIMIT)?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> io::Result<()> {
        if self.schema != 1
            || self.domain != "dlpad-app-registration-seal-v1"
            || !hex(&self.nonce)
            || self.nonce.bytes().all(|b| b == b'0')
            || !hex(&self.record_sha256)
        {
            return Err(bad());
        }
        anchors(
            &self.root_identity,
            &self.record_identity,
            &self.seal_identity,
        )?;
        Ok(())
    }
}
fn serialize<T: Serialize>(v: &T, limit: usize) -> io::Result<Vec<u8>> {
    let bytes = serde_json::to_vec(v).map_err(|_| bad())?;
    if bytes.len() > limit {
        Err(bad())
    } else {
        Ok(bytes)
    }
}
pub(super) fn serialize_record(record: &RegistrationRecord) -> io::Result<Vec<u8>> {
    record.validate()?;
    serialize(record, RECORD_LIMIT)
}
pub(super) fn serialize_seal(seal: &RegistrationSeal) -> io::Result<Vec<u8>> {
    seal.validate()?;
    serialize(seal, SEAL_LIMIT)
}
pub(super) fn record_digest(record: &RegistrationRecord) -> io::Result<String> {
    let mut hash = Sha256::new();
    hash.update(b"dlpad-app-registration-record-v1\0");
    hash.update(serialize_record(record)?);
    Ok(format!("{:x}", hash.finalize()))
}
pub(super) fn validate_binding(
    record: &RegistrationRecord,
    seal: &RegistrationSeal,
) -> io::Result<()> {
    record.validate()?;
    seal.validate()?;
    if record.nonce != seal.nonce
        || record.root_identity != seal.root_identity
        || record.record_identity != seal.record_identity
        || record.seal_identity != seal.seal_identity
        || record_digest(record)? != seal.record_sha256
    {
        return Err(bad());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn id(n: u8) -> NativeFileIdentity {
        NativeFileIdentity::Windows {
            volume_serial: 1,
            file_id: [n; 16],
        }
    }
    fn pair() -> (RegistrationRecord, RegistrationSeal) {
        let prepared = PreparedRecord {
            schema: 1,
            domain: "dlpad-prepared-app-v1".into(),
            root_identity: id(1),
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-gnu".into(),
            development_only: true,
            trusted_provenance: false,
            entries: vec![super::super::PreparedEntry {
                name: "dlpad.exe".into(),
                bytes: 256,
                sha256: "1".repeat(64),
                identity: id(4),
            }],
        };
        let r = RegistrationRecord {
            schema: 1,
            domain: "dlpad-app-registration-v1".into(),
            nonce: "a".repeat(64),
            root_identity: id(1),
            record_identity: id(2),
            seal_identity: id(3),
            ready_identity: id(5),
            prepared_sha256: format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&prepared).unwrap())
            ),
            prepared,
        };
        let s = RegistrationSeal {
            schema: 1,
            domain: "dlpad-app-registration-seal-v1".into(),
            nonce: r.nonce.clone(),
            root_identity: id(1),
            record_identity: id(2),
            seal_identity: id(3),
            record_sha256: record_digest(&r).unwrap(),
        };
        (r, s)
    }
    #[test]
    fn bounded_roundtrip_and_domain_separated_digest() {
        let (r, s) = pair();
        validate_binding(&r, &s).unwrap();
        let bytes = serialize_record(&r).unwrap();
        let rr = RegistrationRecord::read(bytes.as_slice()).unwrap();
        let ss = RegistrationSeal::read(serialize_seal(&s).unwrap().as_slice()).unwrap();
        validate_binding(&rr, &ss).unwrap();
        assert_ne!(
            record_digest(&r).unwrap(),
            format!("{:x}", Sha256::digest(&bytes))
        );
    }
    #[test]
    fn strict_unknown_duplicate_oversize_and_malformed() {
        let (r, s) = pair();
        let mut json = serde_json::to_value(&r).unwrap();
        json["unknown"] = true.into();
        assert!(RegistrationRecord::read(serde_json::to_vec(&json).unwrap().as_slice()).is_err());
        assert!(RegistrationRecord::read(&b"{\"schema\":1,\"schema\":1}"[..]).is_err());
        assert!(RegistrationRecord::read(vec![b' '; RECORD_LIMIT + 1].as_slice()).is_err());
        assert!(RegistrationSeal::read(vec![b' '; SEAL_LIMIT + 1].as_slice()).is_err());
        let mut seal = s;
        seal.domain = "other".into();
        assert!(seal.validate().is_err());
    }
    #[test]
    fn rejects_nonce_ids_privilege_and_prepared_tamper() {
        let (r, _) = pair();
        let mut candidates = Vec::new();
        let mut x = r.clone();
        x.nonce = "0".repeat(64);
        candidates.push(x);
        let mut x = r.clone();
        x.nonce = "A".repeat(64);
        candidates.push(x);
        let mut x = r.clone();
        x.record_identity = id(1);
        candidates.push(x);
        let mut x = r.clone();
        x.seal_identity = NativeFileIdentity::Windows {
            volume_serial: 2,
            file_id: [3; 16],
        };
        candidates.push(x);
        let mut x = r.clone();
        x.prepared.trusted_provenance = true;
        candidates.push(x);
        let mut x = r.clone();
        x.prepared.development_only = false;
        candidates.push(x);
        let mut x = r.clone();
        x.prepared.entries[0].bytes += 1;
        candidates.push(x);
        let mut x = r.clone();
        x.root_identity = NativeFileIdentity::Unix {
            device: 1,
            inode: 1,
        };
        candidates.push(x);
        let mut x = r.clone();
        x.prepared_sha256 = "0".repeat(64);
        candidates.push(x);
        let mut x = r.clone();
        x.ready_identity = id(4);
        candidates.push(x);
        let mut x = r.clone();
        x.ready_identity = id(2);
        candidates.push(x);
        let mut x = r.clone();
        x.ready_identity = NativeFileIdentity::Windows {
            volume_serial: 2,
            file_id: [5; 16],
        };
        candidates.push(x);
        for x in candidates {
            assert!(x.validate().is_err());
        }
    }
    #[test]
    fn seal_mismatched_original_binding_rejected() {
        let (r, s) = pair();
        let mut x = s.clone();
        x.nonce = "b".repeat(64);
        assert!(validate_binding(&r, &x).is_err());
        let mut x = s.clone();
        std::mem::swap(&mut x.record_identity, &mut x.seal_identity);
        assert!(validate_binding(&r, &x).is_err());
        let mut x = s;
        x.record_sha256 = "0".repeat(64);
        assert!(validate_binding(&r, &x).is_err());
    }
}
