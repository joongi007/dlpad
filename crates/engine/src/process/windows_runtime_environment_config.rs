//! Strict environment policy handoff only; cache pathname is not ownership authority.
use crate::windows_deno_cache_binding::CacheBinding;
use serde::{Deserialize, Serialize};
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RuntimeEnvironmentConfig {
    version: u8,
    cache_path: Vec<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cache_binding: Option<CacheBinding>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    version: u8,
    cache_path: Vec<u16>,
    #[serde(default)]
    cache_binding: Option<CacheBinding>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct EnvironmentConfigError;
impl std::fmt::Debug for RuntimeEnvironmentConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RuntimeEnvironmentConfigRedacted")
    }
}
impl std::fmt::Debug for EnvironmentConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RuntimeEnvironmentConfigError")
    }
}
impl std::fmt::Display for EnvironmentConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RuntimeEnvironmentConfigError")
    }
}
impl std::error::Error for EnvironmentConfigError {}
impl<'de> Deserialize<'de> for RuntimeEnvironmentConfig {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let wire = Wire::deserialize(d)?;
        let value = Self::new(wire.cache_path, wire.version)
            .map_err(|_| serde::de::Error::custom("RuntimeEnvironmentConfigError"))?;
        match wire.cache_binding {
            Some(binding) => value
                .with_binding(binding)
                .map_err(|_| serde::de::Error::custom("RuntimeEnvironmentConfigError")),
            None => Ok(value),
        }
    }
}
impl RuntimeEnvironmentConfig {
    pub(super) fn new(cache_path: Vec<u16>, version: u8) -> Result<Self, EnvironmentConfigError> {
        let value = Self {
            version,
            cache_path,
            cache_binding: None,
        };
        value.validate()?;
        Ok(value)
    }
    pub(super) fn validate(&self) -> Result<(), EnvironmentConfigError> {
        crate::runtime_environment::validate_cache_units(&self.cache_path, self.version)
            .map_err(|_| EnvironmentConfigError)?;
        if let Some(binding) = &self.cache_binding {
            binding.validate().map_err(|_| EnvironmentConfigError)?;
            let mut expected = binding.root_path.clone();
            if !matches!(expected.last(), Some(92 | 47)) {
                expected.push(92);
            }
            expected.extend(".runtime-cache\\deno".encode_utf16());
            if expected != self.cache_path {
                return Err(EnvironmentConfigError);
            }
        }
        Ok(())
    }
    pub(super) fn cache_binding(&self) -> Option<&CacheBinding> {
        self.cache_binding.as_ref()
    }
    pub(super) fn with_binding(
        mut self,
        binding: CacheBinding,
    ) -> Result<Self, EnvironmentConfigError> {
        self.cache_binding = Some(binding);
        self.validate()?;
        Ok(self)
    }
    #[cfg(windows)]
    pub(super) fn from_native(
        policy: &crate::runtime_environment::WindowsManagedDenoEnvironment,
    ) -> Result<Self, EnvironmentConfigError> {
        use std::os::windows::ffi::OsStrExt;
        policy.validate().map_err(|_| EnvironmentConfigError)?;
        let value = Self::new(
            policy
                .cache_path()
                .as_os_str()
                .encode_wide()
                .take(32767)
                .collect(),
            policy.policy_version(),
        )?;
        match policy.cache_binding() {
            Some(binding) => value.with_binding(binding.clone()),
            None => Ok(value),
        }
    }
    #[cfg(windows)]
    pub(super) fn to_native(
        &self,
    ) -> Result<crate::runtime_environment::WindowsManagedDenoEnvironment, EnvironmentConfigError>
    {
        use std::{ffi::OsString, os::windows::ffi::OsStringExt, path::PathBuf};
        self.validate()?;
        crate::runtime_environment::WindowsManagedDenoEnvironment::from_handoff(
            PathBuf::from(OsString::from_wide(&self.cache_path)),
            self.cache_binding.clone(),
        )
        .map_err(|_| EnvironmentConfigError)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn valid() -> RuntimeEnvironmentConfig {
        RuntimeEnvironmentConfig::new(r"C:\owned 한글 cache".encode_utf16().collect(), 1).unwrap()
    }
    #[test]
    fn original_cache_attachment_requires_exact_path_and_strict_nested_ids() {
        use crate::file_identity::NativeFileIdentity;
        let id = |n| NativeFileIdentity::Windows {
            volume_serial: 7,
            file_id: [n; 16],
        };
        let binding = CacheBinding {
            version: 1,
            root_path: r"C:\owned".encode_utf16().collect(),
            root: id(1),
            runtime: id(2),
            cache: id(3),
            owner: id(4),
            lock: id(5),
            owner_sha256: [6; 32],
        };
        let value = RuntimeEnvironmentConfig::new(
            r"C:\owned\.runtime-cache\deno".encode_utf16().collect(),
            1,
        )
        .unwrap()
        .with_binding(binding.clone())
        .unwrap();
        assert!(valid().with_binding(binding).is_err());
        let wire = serde_json::to_vec(&value).unwrap();
        let decoded: RuntimeEnvironmentConfig = serde_json::from_slice(&wire).unwrap();
        assert_eq!(decoded, value);
        assert!(decoded.cache_binding().is_some());
        let base = serde_json::to_value(&value).unwrap();
        for mode in 0..4 {
            let mut v = base.clone();
            match mode {
                0 => v["cache_binding"]["foreign"] = true.into(),
                1 => v["cache_binding"]["version"] = 2.into(),
                2 => v["cache_binding"]["owner"] = v["cache_binding"]["lock"].clone(),
                _ => {
                    v["cache_path"] =
                        serde_json::json!(r"C:\foreign".encode_utf16().collect::<Vec<_>>())
                }
            }
            assert!(
                serde_json::from_value::<RuntimeEnvironmentConfig>(v).is_err(),
                "mode={mode}"
            );
        }
        assert!(
            serde_json::to_value(valid())
                .unwrap()
                .get("cache_binding")
                .is_none()
        );
    }
    #[test]
    fn typed_roundtrip_and_redaction() {
        let value = valid();
        let bytes = serde_json::to_vec(&value).unwrap();
        let decoded: RuntimeEnvironmentConfig = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value, decoded);
        assert_eq!(format!("{value:?}"), "RuntimeEnvironmentConfigRedacted");
        assert_eq!(
            format!("{EnvironmentConfigError:?} {EnvironmentConfigError}"),
            "RuntimeEnvironmentConfigError RuntimeEnvironmentConfigError"
        );
    }
    #[test]
    fn nested_deserialize_unknown_version_field_and_malformed_values_rejected() {
        let value = serde_json::to_value(valid()).unwrap();
        for bad in [0, 2, 255] {
            let mut v = value.clone();
            v["version"] = bad.into();
            assert!(serde_json::from_value::<RuntimeEnvironmentConfig>(v).is_err());
        }
        let mut v = value.clone();
        v["foreign"] = true.into();
        assert!(serde_json::from_value::<RuntimeEnvironmentConfig>(v).is_err());
        let mut v = value;
        v["cache_path"] = "raw-secret".into();
        assert!(serde_json::from_value::<RuntimeEnvironmentConfig>(v).is_err());
    }
    #[test]
    fn bounded_utf16_and_windows_path_policy_rejected() {
        for path in [
            "",
            r"relative\cache",
            r"C:cache",
            r"C:\owned\..\foreign",
            r"C:\bad?cache",
            r"\\?\GLOBALROOT\Device\cache",
        ] {
            assert!(RuntimeEnvironmentConfig::new(path.encode_utf16().collect(), 1).is_err());
        }
        for units in [vec![0], vec![0xd800], vec![1; 32767]] {
            assert!(RuntimeEnvironmentConfig::new(units, 1).is_err());
        }
    }
    #[cfg(windows)]
    #[test]
    fn native_roundtrip_preserves_policy_exactly() {
        let policy = valid().to_native().unwrap();
        let wire = RuntimeEnvironmentConfig::from_native(&policy).unwrap();
        assert_eq!(wire, valid());
        assert_eq!(policy.policy_version(), 1);
    }
}
