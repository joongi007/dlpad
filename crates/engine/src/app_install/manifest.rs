//! A bounded description, not a provenance or release trust grant.
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    io::{self, Read},
};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleManifest {
    pub schema: u32,
    pub version: String,
    pub target: String,
    pub entries: Vec<BundleEntry>,
    pub development_only: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleEntry {
    pub name: String,
    pub bytes: u64,
    pub sha256: String,
}
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid app bundle manifest")
}
impl BundleManifest {
    pub fn read(reader: impl Read) -> io::Result<Self> {
        let mut bytes = Vec::new();
        reader.take(65537).read_to_end(&mut bytes)?;
        if bytes.len() > 65536 {
            return Err(invalid());
        }
        let result: Self = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> io::Result<()> {
        if self.version.len() > 64 {
            return Err(invalid());
        }
        let parts: Vec<_> = self.version.split('.').collect();
        if self.schema != 1
            || !matches!(
                self.target.as_str(),
                "x86_64-pc-windows-gnu" | "x86_64-pc-windows-msvc"
            )
            || self.version.len() > 64
            || parts.len() != 3
            || parts
                .iter()
                .any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()))
            || self.entries.is_empty()
            || self.entries.len() > 64
        {
            return Err(invalid());
        }
        let mut names = BTreeSet::new();
        let mut sum = 0u64;
        let mut exe = 0;
        for e in &self.entries {
            if !name(&e.name)
                || !names.insert(e.name.to_ascii_lowercase())
                || e.bytes > 128 * 1024 * 1024
                || e.sha256.len() != 64
                || !e
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(invalid());
            }
            sum = sum.checked_add(e.bytes).ok_or_else(invalid)?;
            if sum > 256 * 1024 * 1024 {
                return Err(invalid());
            }
            if e.name == "dlpad.exe" {
                exe += 1;
            }
        }
        if exe != 1 {
            return Err(invalid());
        }
        Ok(())
    }
}
fn name(s: &str) -> bool {
    if s.is_empty()
        || s.len() > 96
        || !s.is_ascii()
        || s.ends_with(['.', ' '])
        || s.bytes().any(|b| {
            b.is_ascii_control()
                || matches!(
                    b,
                    b'/' | b'\\' | b':' | b'<' | b'>' | b'"' | b'|' | b'?' | b'*'
                )
        })
    {
        return false;
    }
    let stem = s
        .split('.')
        .next()
        .unwrap_or("")
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$")
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
    {
        return false;
    }
    s == "dlpad.exe"
        || s == "LICENSE"
        || s == "THIRD-PARTY-NOTICES.txt"
        || (s.ends_with(".txt") && s.len() > 4 && !s.starts_with('.'))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn base() -> BundleManifest {
        BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-gnu".into(),
            entries: vec![BundleEntry {
                name: "dlpad.exe".into(),
                bytes: 3,
                sha256: "a".repeat(64),
            }],
            development_only: true,
        }
    }
    #[test]
    fn roundtrip_targets() {
        let mut b = base();
        assert!(BundleManifest::read(serde_json::to_vec(&b).unwrap().as_slice()).is_ok());
        b.target = "x86_64-pc-windows-msvc".into();
        assert!(b.validate().is_ok());
    }
    #[test]
    fn unsafe_and_devices() {
        for n in [
            "../a.txt",
            "a/b.txt",
            "a\\b.txt",
            "a:b.txt",
            "CON.txt",
            "nul.txt",
            "COM1.txt",
            "lpt9.txt",
            "NUL .txt",
            "x.txt.",
            "x.txt ",
            "x\n.txt",
            ".txt",
            "extra.dll",
        ] {
            let mut b = base();
            b.entries.push(BundleEntry {
                name: n.into(),
                bytes: 1,
                sha256: "a".repeat(64),
            });
            assert!(b.validate().is_err(), "{n}");
        }
    }
    #[test]
    fn duplicates_digest_missing() {
        let mut b = base();
        b.entries.push(b.entries[0].clone());
        assert!(b.validate().is_err());
        b = base();
        b.entries[0].name = "DLPAD.exe".into();
        assert!(b.validate().is_err());
        for sha in ["A".repeat(64), "g".repeat(64), "a".repeat(63)] {
            let mut b = base();
            b.entries[0].sha256 = sha;
            assert!(b.validate().is_err());
        }
    }
    #[test]
    fn schema_version_target() {
        for v in ["1.2", "1..3", "1.2.3.4", "1.2.x", "1.2.3-beta", "１.2.3"] {
            let mut b = base();
            b.version = v.into();
            assert!(b.validate().is_err());
        }
        let mut b = base();
        b.schema = 2;
        assert!(b.validate().is_err());
        b = base();
        b.target = "linux".into();
        assert!(b.validate().is_err());
    }
    #[test]
    fn counts_and_sizes() {
        let mut b = base();
        b.entries.clear();
        assert!(b.validate().is_err());
        b = base();
        b.entries = vec![b.entries[0].clone(); 65];
        assert!(b.validate().is_err());
        for bytes in [128 * 1024 * 1024 + 1, u64::MAX] {
            let mut b = base();
            b.entries[0].bytes = bytes;
            assert!(b.validate().is_err());
        }
        let mut b = base();
        b.entries[0].bytes = 128 * 1024 * 1024;
        b.entries.push(BundleEntry {
            name: "notice.txt".into(),
            bytes: 128 * 1024 * 1024,
            sha256: "a".repeat(64),
        });
        assert!(b.validate().is_ok());
        b.entries.push(BundleEntry {
            name: "other.txt".into(),
            bytes: 1,
            sha256: "a".repeat(64),
        });
        assert!(b.validate().is_err());
    }
    #[test]
    fn unknown_oversize_malformed() {
        assert!(BundleManifest::read(vec![b' '; 65537].as_slice()).is_err());
        let mut j = serde_json::to_value(base()).unwrap();
        j["unknown"] = true.into();
        assert!(BundleManifest::read(serde_json::to_vec(&j).unwrap().as_slice()).is_err());
        j.as_object_mut().unwrap().remove("unknown");
        j["entries"][0]["unknown"] = true.into();
        assert!(BundleManifest::read(serde_json::to_vec(&j).unwrap().as_slice()).is_err());
        assert!(BundleManifest::read(b"{}".as_slice()).is_err());
    }
}
