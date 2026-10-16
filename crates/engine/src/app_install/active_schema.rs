//! Pure explicit-path development selection. IDs/digests are consistency facts,
//! never native ownership, publisher trust, discovery, launch or deletion rights.
use super::{
    PreparedEntry, receipt_schema::ReceiptRecord, registration_schema::RegistrationRecord,
};
use crate::file_identity::NativeFileIdentity;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{self, Read};
const LIMIT: usize = 65536;
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActiveSelectionRecord {
    pub schema: u32,
    pub domain: String,
    pub development_only: bool,
    pub active_root_identity: NativeFileIdentity,
    pub marker_identity: NativeFileIdentity,
    pub active_path: String,
    pub registered_path: String,
    pub receipt_path: String,
    pub registration_root_identity: NativeFileIdentity,
    pub registration_record_identity: NativeFileIdentity,
    pub registration_seal_identity: NativeFileIdentity,
    pub registration_ready_identity: NativeFileIdentity,
    pub receipt_root_identity: NativeFileIdentity,
    pub receipt_identity: NativeFileIdentity,
    pub registration_sha256: String,
    pub receipt_sha256: String,
    pub selected_executable: PreparedEntry,
}
impl std::fmt::Debug for ActiveSelectionRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ActiveSelectionRecord(<redacted>)")
    }
}
fn bad() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "invalid explicit active selection",
    )
}
fn digest<T: Serialize>(v: &T) -> io::Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(v).map_err(|_| bad())?)
    ))
}
fn hex(v: &str) -> bool {
    v.len() == 64
        && v.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
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
fn path(v: &str) -> bool {
    let v = if let Some(rest) = v.strip_prefix("\\\\?\\") {
        rest
    } else {
        v
    };
    let bytes = v.as_bytes();
    // Native extended disk prefix is accepted; UNC/device namespaces remain refused.
    if bytes.len() < 3
        || bytes.len() > 32768
        || !bytes[0].is_ascii_alphabetic()
        || bytes[1] != b':'
        || bytes[2] != b'\\'
        || v.chars().any(|c| c.is_control() || c == '/' || c == '\0')
    {
        return false;
    }
    if v.encode_utf16().count() > 32767 {
        return false;
    }
    v[3..].split('\\').all(|part| {
        !part.is_empty()
            && part != "."
            && part != ".."
            && !(part.ends_with('.') || part.ends_with(' '))
            && !part
                .chars()
                .any(|c| matches!(c, ':' | '<' | '>' | '"' | '|' | '?' | '*'))
    })
}
impl ActiveSelectionRecord {
    pub fn read(reader: impl Read) -> io::Result<Self> {
        let mut raw = Vec::new();
        reader.take((LIMIT + 1) as u64).read_to_end(&mut raw)?;
        if raw.is_empty() || raw.len() > LIMIT {
            return Err(bad());
        }
        let record: Self = serde_json::from_slice(&raw).map_err(|_| bad())?;
        record.validate()?;
        if record.canonical_bytes()? != raw {
            return Err(bad());
        }
        Ok(record)
    }
    pub fn canonical_bytes(&self) -> io::Result<Vec<u8>> {
        let bytes = serde_json::to_vec(self).map_err(|_| bad())?;
        if bytes.len() > LIMIT {
            return Err(bad());
        }
        Ok(bytes)
    }
    pub fn validate(&self) -> io::Result<()> {
        if self.schema != 1
            || self.domain != "dlpad-explicit-active-selection-v1"
            || !self.development_only
            || !path(&self.active_path)
            || !path(&self.registered_path)
            || !path(&self.receipt_path)
            || !hex(&self.registration_sha256)
            || !hex(&self.receipt_sha256)
            || self.selected_executable.name != "dlpad.exe"
            || self.selected_executable.bytes < 64
            || self.selected_executable.bytes > 128 * 1024 * 1024
            || !hex(&self.selected_executable.sha256)
        {
            return Err(bad());
        }
        let paths = [&self.active_path, &self.registered_path, &self.receipt_path];
        for i in 0..paths.len() {
            for j in i + 1..paths.len() {
                let normalize = |p: &str| p.strip_prefix("\\\\?\\").unwrap_or(p).to_lowercase();
                let a = normalize(paths[i]);
                let b = normalize(paths[j]);
                if a == b || a.starts_with(&(b.clone() + "\\")) || b.starts_with(&(a + "\\")) {
                    return Err(bad());
                }
            }
        }
        let ids = [
            &self.active_root_identity,
            &self.marker_identity,
            &self.registration_root_identity,
            &self.registration_record_identity,
            &self.registration_seal_identity,
            &self.registration_ready_identity,
            &self.receipt_root_identity,
            &self.receipt_identity,
            &self.selected_executable.identity,
        ];
        for (i, id) in ids.iter().enumerate() {
            volume(id)?;
            if ids[..i].contains(id) {
                return Err(bad());
            }
        }
        if volume(&self.active_root_identity)? != volume(&self.marker_identity)?
            || volume(&self.registration_root_identity)?
                != volume(&self.selected_executable.identity)?
            || volume(&self.receipt_root_identity)? != volume(&self.receipt_identity)?
        {
            return Err(bad());
        }
        for id in [
            &self.registration_record_identity,
            &self.registration_seal_identity,
            &self.registration_ready_identity,
        ] {
            if volume(id)? != volume(&self.registration_root_identity)? {
                return Err(bad());
            }
        }
        self.canonical_bytes()?;
        Ok(())
    }
    /// Caller separately verifies original native capabilities and explicit paths.
    pub fn verify_pair(
        &self,
        registration: &RegistrationRecord,
        receipt: &ReceiptRecord,
    ) -> io::Result<()> {
        self.validate()?;
        registration.validate()?;
        receipt.validate()?;
        for entry in &registration.prepared.entries {
            if entry.identity == self.active_root_identity || entry.identity == self.marker_identity
            {
                return Err(bad());
            }
        }
        if serde_json::to_vec(registration).map_err(|_| bad())?
            != serde_json::to_vec(&receipt.registration).map_err(|_| bad())?
            || self.registration_sha256 != super::registration_schema::record_digest(registration)?
            || self.receipt_sha256 != digest(receipt)?
            || self.registration_root_identity != registration.root_identity
            || self.registration_record_identity != registration.record_identity
            || self.registration_seal_identity != registration.seal_identity
            || self.registration_ready_identity != registration.ready_identity
            || self.receipt_root_identity != receipt.receipt_root_identity
            || self.receipt_identity != receipt.receipt_identity
            || registration
                .prepared
                .entries
                .iter()
                .filter(|e| e.name == "dlpad.exe")
                .count()
                != 1
            || serde_json::to_vec(
                registration
                    .prepared
                    .entries
                    .iter()
                    .find(|e| e.name == "dlpad.exe")
                    .ok_or_else(bad)?,
            )
            .map_err(|_| bad())?
                != serde_json::to_vec(&self.selected_executable).map_err(|_| bad())?
        {
            return Err(bad());
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn id(n: u8) -> NativeFileIdentity {
        NativeFileIdentity::Windows {
            volume_serial: 9,
            file_id: [n; 16],
        }
    }
    fn record() -> ActiveSelectionRecord {
        ActiveSelectionRecord {
            schema: 1,
            domain: "dlpad-explicit-active-selection-v1".into(),
            development_only: true,
            active_root_identity: id(1),
            marker_identity: id(2),
            active_path: "C:\\owned\\active".into(),
            registered_path: "C:\\owned\\registered".into(),
            receipt_path: "C:\\owned\\receipt".into(),
            registration_root_identity: id(3),
            registration_record_identity: id(4),
            registration_seal_identity: id(5),
            registration_ready_identity: id(6),
            receipt_root_identity: id(7),
            receipt_identity: id(8),
            registration_sha256: "a".repeat(64),
            receipt_sha256: "b".repeat(64),
            selected_executable: PreparedEntry {
                name: "dlpad.exe".into(),
                bytes: 128,
                sha256: "c".repeat(64),
                identity: id(9),
            },
        }
    }
    #[test]
    fn canonical_roundtrip_unknown_and_boolean_fail() {
        let r = record();
        assert!(ActiveSelectionRecord::read(r.canonical_bytes().unwrap().as_slice()).is_ok());
        let mut v = serde_json::to_value(&r).unwrap();
        v["development_only"] = false.into();
        assert!(ActiveSelectionRecord::read(serde_json::to_vec(&v).unwrap().as_slice()).is_err());
        v = serde_json::to_value(r).unwrap();
        v["publisher_trust"] = true.into();
        assert!(ActiveSelectionRecord::read(serde_json::to_vec(&v).unwrap().as_slice()).is_err());
    }
    #[test]
    fn paths_and_cross_anchor_aliases_fail() {
        for p in [
            "relative",
            "C:\\owned\\..\\bad",
            "\\\\?\\UNC\\server\\share",
            "C:\\owned\\active\\child",
            "C:\\owned\\ACTIVE",
        ] {
            let mut r = record();
            r.receipt_path = p.into();
            assert!(r.validate().is_err());
        }
        let mut r = record();
        r.marker_identity = r.registration_record_identity.clone();
        assert!(r.validate().is_err());
    }
    #[test]
    fn malformed_digest_size_noncanonical_and_bounds_fail() {
        let mut r = record();
        r.receipt_sha256 = "g".repeat(64);
        assert!(r.validate().is_err());
        let r = record();
        let mut b = r.canonical_bytes().unwrap();
        b.push(b'\n');
        assert!(ActiveSelectionRecord::read(b.as_slice()).is_err());
        assert!(ActiveSelectionRecord::read(vec![b' '; LIMIT + 1].as_slice()).is_err());
        let mut r = record();
        r.selected_executable.name = "notice.txt".into();
        assert!(r.validate().is_err());
    }
    #[test]
    fn actual_pair_digest_and_selected_binding_mismatch_fail() {
        let mut r = record();
        let registration = RegistrationRecord {
            schema: 1,
            domain: "dlpad-app-registration-v1".into(),
            nonce: "1".repeat(64),
            root_identity: r.registration_root_identity.clone(),
            record_identity: r.registration_record_identity.clone(),
            seal_identity: r.registration_seal_identity.clone(),
            ready_identity: r.registration_ready_identity.clone(),
            prepared_sha256: String::new(),
            prepared: super::super::PreparedRecord {
                schema: 1,
                domain: "dlpad-prepared-app-v1".into(),
                root_identity: r.registration_root_identity.clone(),
                version: "0.1.0".into(),
                target: "x86_64-pc-windows-gnu".into(),
                development_only: true,
                trusted_provenance: false,
                entries: vec![r.selected_executable.clone()],
            },
        };
        let mut registration = registration;
        registration.prepared_sha256 = digest(&registration.prepared).unwrap();
        let mut hash = Sha256::new();
        hash.update(b"dlpad-app-registration-record-v1\0");
        hash.update(serde_json::to_vec(&registration).unwrap());
        let seal = super::super::registration_schema::RegistrationSeal {
            schema: 1,
            domain: "dlpad-app-registration-seal-v1".into(),
            nonce: registration.nonce.clone(),
            root_identity: registration.root_identity.clone(),
            record_identity: registration.record_identity.clone(),
            seal_identity: registration.seal_identity.clone(),
            record_sha256: format!("{:x}", hash.finalize()),
        };
        let receipt = ReceiptRecord {
            schema: 1,
            domain: "dlpad-app-registration-receipt-v1".into(),
            receipt_root_identity: r.receipt_root_identity.clone(),
            receipt_identity: r.receipt_identity.clone(),
            registration: registration.clone(),
            seal,
        };
        r.registration_sha256 =
            super::super::registration_schema::record_digest(&registration).unwrap();
        r.receipt_sha256 = digest(&receipt).unwrap();
        assert!(r.verify_pair(&registration, &receipt).is_ok());
        let mut collision_registration = registration.clone();
        collision_registration.prepared.entries.push(PreparedEntry {
            name: "notice.txt".into(),
            bytes: 2,
            sha256: "d".repeat(64),
            identity: r.active_root_identity.clone(),
        });
        collision_registration.prepared_sha256 = digest(&collision_registration.prepared).unwrap();
        let mut collision_receipt = receipt.clone();
        collision_receipt.registration = collision_registration.clone();
        collision_receipt.seal.record_sha256 =
            super::super::registration_schema::record_digest(&collision_registration).unwrap();
        collision_registration.validate().unwrap();
        collision_receipt.validate().unwrap();
        let mut collision = r.clone();
        collision.registration_sha256 =
            super::super::registration_schema::record_digest(&collision_registration).unwrap();
        collision.receipt_sha256 = digest(&collision_receipt).unwrap();
        assert!(
            collision
                .verify_pair(&collision_registration, &collision_receipt)
                .is_err()
        );
        r.registration_sha256 = "0".repeat(64);
        assert!(r.verify_pair(&registration, &receipt).is_err());
        r.registration_sha256 =
            super::super::registration_schema::record_digest(&registration).unwrap();
        r.selected_executable.bytes += 1;
        assert!(r.verify_pair(&registration, &receipt).is_err());
    }

    #[test]
    fn native_extended_disk_and_literal_domain_digest() {
        let mut r = record();
        r.active_path = "\\\\?\\C:\\owned\\active".into();
        r.registered_path = "\\\\?\\C:\\owned\\registered".into();
        r.receipt_path = "\\\\?\\C:\\owned\\receipt".into();
        assert!(r.validate().is_ok());
        for value in [
            "\\\\.\\C:\\owned",
            "\\\\?\\UNC\\server\\share",
            "\\\\?\\GLOBALROOT\\Device\\disk",
            "\\\\?\\C:relative",
        ] {
            r.receipt_path = value.into();
            assert!(r.validate().is_err());
        }
        let raw = b"{}";
        let mut domain = Sha256::new();
        domain.update(b"dlpad-app-registration-record-v1\0");
        domain.update(raw);
        assert_eq!(
            format!("{:x}", domain.finalize()),
            "8283ea898874fd980df78ae066d1047a059416510d9d5ebdac367ad011818529"
        );
    }
}
