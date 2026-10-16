//! Bounded original cache identity attachment. Syntax is not native ownership authority.
use crate::file_identity::NativeFileIdentity;
use serde::{Deserialize, Serialize};
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CacheBinding {
    pub(crate) version: u8,
    pub(crate) root_path: Vec<u16>,
    pub(crate) root: NativeFileIdentity,
    pub(crate) runtime: NativeFileIdentity,
    pub(crate) cache: NativeFileIdentity,
    pub(crate) owner: NativeFileIdentity,
    pub(crate) lock: NativeFileIdentity,
    pub(crate) owner_sha256: [u8; 32],
}
impl std::fmt::Debug for CacheBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DenoCacheBindingRedacted")
    }
}
impl CacheBinding {
    pub(crate) fn validate(&self) -> Result<(), BindingError> {
        if self.version != 1 || self.owner_sha256 == [0; 32] {
            return Err(BindingError);
        }
        crate::runtime_environment::validate_cache_units(&self.root_path, 1)
            .map_err(|_| BindingError)?;
        let ids = [
            &self.root,
            &self.runtime,
            &self.cache,
            &self.owner,
            &self.lock,
        ];
        let mut volume = None;
        for (index, identity) in ids.iter().enumerate() {
            let NativeFileIdentity::Windows {
                volume_serial,
                file_id,
            } = identity
            else {
                return Err(BindingError);
            };
            if *file_id == [0; 16]
                || volume.is_some_and(|v| v != *volume_serial)
                || ids[..index].contains(identity)
            {
                return Err(BindingError);
            }
            volume = Some(*volume_serial);
        }
        Ok(())
    }
    #[cfg(windows)]
    pub(crate) fn root_path(&self) -> Result<std::path::PathBuf, BindingError> {
        use std::os::windows::ffi::OsStringExt;
        self.validate()?;
        Ok(std::ffi::OsString::from_wide(&self.root_path).into())
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct BindingError;
impl std::fmt::Debug for BindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DenoCacheBindingError")
    }
}
impl std::fmt::Display for BindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("runtime cache identity attachment rejected")
    }
}
impl std::error::Error for BindingError {}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> CacheBinding {
        let id = |n| NativeFileIdentity::Windows {
            volume_serial: 7,
            file_id: [n; 16],
        };
        CacheBinding {
            version: 1,
            root_path: r"C:\owned".encode_utf16().collect(),
            root: id(1),
            runtime: id(2),
            cache: id(3),
            owner: id(4),
            lock: id(5),
            owner_sha256: [6; 32],
        }
    }
    #[test]
    fn exact_attachment_roundtrip_and_redacted_debug() {
        let b = fixture();
        b.validate().unwrap();
        let wire = serde_json::to_vec(&b).unwrap();
        let decoded: CacheBinding = serde_json::from_slice(&wire).unwrap();
        decoded.validate().unwrap();
        assert_eq!(decoded, b);
        assert_eq!(format!("{b:?}"), "DenoCacheBindingRedacted");
        let mut unknown = serde_json::to_value(&b).unwrap();
        unknown["foreign"] = serde_json::json!(true);
        assert!(serde_json::from_value::<CacheBinding>(unknown).is_err());
    }
    #[test]
    fn malformed_domain_version_alias_zero_digest_and_path_rejected() {
        for mode in 0..7 {
            let mut b = fixture();
            match mode {
                0 => b.version = 2,
                1 => b.owner_sha256 = [0; 32],
                2 => b.owner = b.lock.clone(),
                3 => {
                    b.root = NativeFileIdentity::Unix {
                        device: 1,
                        inode: 1,
                    }
                }
                4 => {
                    b.lock = NativeFileIdentity::Windows {
                        volume_serial: 8,
                        file_id: [5; 16],
                    }
                }
                5 => {
                    b.cache = NativeFileIdentity::Windows {
                        volume_serial: 7,
                        file_id: [0; 16],
                    }
                }
                _ => b.root_path = r"C:\owned\..\foreign".encode_utf16().collect(),
            }
            assert!(b.validate().is_err(), "mode={mode}");
        }
    }
}
