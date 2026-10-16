//! Bounded managed-tool handoff; no native ID, receipt, or root ownership authority.
use super::windows_paired_guardian_binding::PairedGuardianBinding;
use super::windows_runtime_environment_config::RuntimeEnvironmentConfig;
use crate::{
    cookies::windows_records::WindowsReaderRecord,
    tools::windows_managed_records::ManagedReaderRecord,
};
use serde::{Deserialize, Serialize};
use std::{fmt, time::Duration};

pub(super) const MAX_CONFIG_BYTES: usize = 256 * 1024;
pub(super) const CONFIG_DOMAIN: &str = "dlpad-windows-paired-guardian-config-v1";
const MAX_WIDE_UNITS: usize = 32766;
const MAX_TIMEOUT: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PairedGuardianConfig {
    pub(super) version: u8,
    pub(super) domain: String,
    pub(super) executable: Vec<u16>,
    pub(super) args: Vec<Vec<u16>>,
    pub(super) managed_root_path: Vec<u16>,
    pub(super) cookie_owner_path: Vec<u16>,
    pub(super) managed_intent: ManagedReaderRecord,
    pub(super) managed_index: u8,
    pub(super) managed_nonce: [u8; 16],
    pub(super) cookie_intent: WindowsReaderRecord,
    pub(super) cookie_index: u8,
    pub(super) cookie_nonce: [u8; 16],
    pub(super) timeout_secs: u64,
    pub(super) timeout_nanos: u32,
    pub(super) max_log_bytes: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) runtime_environment: Option<RuntimeEnvironmentConfig>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigWire {
    version: u8,
    domain: String,
    executable: Vec<u16>,
    args: Vec<Vec<u16>>,
    managed_root_path: Vec<u16>,
    cookie_owner_path: Vec<u16>,
    managed_intent: ManagedReaderRecord,
    managed_index: u8,
    managed_nonce: [u8; 16],
    cookie_intent: WindowsReaderRecord,
    cookie_index: u8,
    cookie_nonce: [u8; 16],
    timeout_secs: u64,
    timeout_nanos: u32,
    max_log_bytes: usize,
    #[serde(default)]
    runtime_environment: Option<RuntimeEnvironmentConfig>,
}
impl<'de> Deserialize<'de> for PairedGuardianConfig {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let w = ConfigWire::deserialize(deserializer)?;
        let value = Self {
            version: w.version,
            domain: w.domain,
            executable: w.executable,
            args: w.args,
            managed_root_path: w.managed_root_path,
            cookie_owner_path: w.cookie_owner_path,
            managed_intent: w.managed_intent,
            managed_index: w.managed_index,
            managed_nonce: w.managed_nonce,
            cookie_intent: w.cookie_intent,
            cookie_index: w.cookie_index,
            cookie_nonce: w.cookie_nonce,
            timeout_secs: w.timeout_secs,
            timeout_nanos: w.timeout_nanos,
            max_log_bytes: w.max_log_bytes,
            runtime_environment: w.runtime_environment,
        };
        value
            .validate()
            .map_err(|_| serde::de::Error::custom("PairedGuardianConfigError"))?;
        Ok(value)
    }
}
impl fmt::Debug for PairedGuardianConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PairedGuardianConfigRedacted")
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct ConfigError;
impl fmt::Debug for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PairedGuardianConfigError")
    }
}
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PairedGuardianConfigError")
    }
}
impl std::error::Error for ConfigError {}

impl PairedGuardianConfig {
    pub(super) fn validate(&self) -> Result<(), ConfigError> {
        if let Some(policy) = &self.runtime_environment {
            policy.validate().map_err(|_| ConfigError)?;
        }
        let wide = |v: &[u16]| v.len() <= MAX_WIDE_UNITS && !v.contains(&0);
        if self.version != 1
            || self.domain != CONFIG_DOMAIN
            || self.cookie_index > 1
            || self.cookie_nonce == [0; 16]
            || self.cookie_owner_path.is_empty()
            || !wide(&self.cookie_owner_path)
            || self.managed_index > 7
            || self.managed_nonce == [0; 16]
            || self.timeout_nanos >= 1_000_000_000
            || (self.timeout_secs == 0 && self.timeout_nanos == 0)
            || self.timeout_secs > MAX_TIMEOUT.as_secs()
            || (self.timeout_secs == MAX_TIMEOUT.as_secs() && self.timeout_nanos != 0)
            || !(1..=super::MAX_LOG_BYTES).contains(&self.max_log_bytes)
            || self.executable.is_empty()
            || !wide(&self.executable)
            || self.managed_root_path.is_empty()
            || !wide(&self.managed_root_path)
            || self.args.len() > 256
        {
            return Err(ConfigError);
        }
        let mut units = 0usize;
        for arg in &self.args {
            if !wide(arg) {
                return Err(ConfigError);
            }
            units = units.checked_add(arg.len()).ok_or(ConfigError)?;
            if units > MAX_WIDE_UNITS {
                return Err(ConfigError);
            }
        }
        let pair = PairedGuardianBinding::from_intents(
            &self.managed_intent,
            self.managed_index,
            &self.cookie_intent,
            self.cookie_index,
            (1, 1),
        )
        .map_err(|_| ConfigError)?;
        if pair.managed_identity().nonce != self.managed_nonce
            || pair.cookie_identity().nonce != self.cookie_nonce
        {
            return Err(ConfigError);
        }
        Ok(())
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, ConfigError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| ConfigError)?;
        if bytes.len() > MAX_CONFIG_BYTES {
            return Err(ConfigError);
        }
        Ok(bytes)
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Self, ConfigError> {
        if bytes.len() > MAX_CONFIG_BYTES {
            return Err(ConfigError);
        }
        let config: Self = serde_json::from_slice(bytes).map_err(|_| ConfigError)?;
        config.validate()?;
        Ok(config)
    }

    pub(super) fn timeout(&self) -> Duration {
        Duration::new(self.timeout_secs, self.timeout_nanos)
    }

    #[cfg(windows)]
    pub(super) fn process_spec(&self) -> Result<super::ProcessSpec, ConfigError> {
        use std::{ffi::OsString, os::windows::ffi::OsStringExt};
        self.validate()?;
        Ok(super::ProcessSpec {
            #[cfg(windows)]
            runtime_environment: self
                .runtime_environment
                .as_ref()
                .map(RuntimeEnvironmentConfig::to_native)
                .transpose()
                .map_err(|_| ConfigError)?,
            executable: std::path::PathBuf::from(OsString::from_wide(&self.executable)),
            args: self
                .args
                .iter()
                .map(|arg| OsString::from_wide(arg))
                .collect(),
            timeout: self.timeout(),
            max_log_bytes: self.max_log_bytes,
        })
    }

    #[cfg(windows)]
    pub(super) fn managed_root_path(&self) -> Result<std::path::PathBuf, ConfigError> {
        use std::{ffi::OsString, os::windows::ffi::OsStringExt};
        self.validate()?;
        Ok(OsString::from_wide(&self.managed_root_path).into())
    }
    #[cfg(windows)]
    pub(super) fn cookie_owner_path(&self) -> Result<std::path::PathBuf, ConfigError> {
        use std::{ffi::OsString, os::windows::ffi::OsStringExt};
        self.validate()?;
        Ok(OsString::from_wide(&self.cookie_owner_path).into())
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
            ReaderStage, SessionAnchors as ManagedAnchors, WindowsAnchor as ManagedAnchor,
        },
    };
    fn config(stage: ReaderStage, slot: u8) -> PairedGuardianConfig {
        let ma = |id| ManagedAnchor {
            volume_serial: 1,
            file_id: [id; 16],
        };
        let ca = |id| CookieAnchor {
            volume_serial: 2,
            file_id: [id; 16],
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
        if slot == 1 {
            c.bind(0, 10, 20).unwrap();
            let r = c.receipt_for(0).unwrap();
            c.resolve(&r).unwrap();
            c.begin(1, [7; 16]).unwrap();
        }
        PairedGuardianConfig {
            version: 1,
            domain: CONFIG_DOMAIN.into(),
            executable: "C:\\한글 tool.exe".encode_utf16().collect(),
            args: vec![
                "literal credential=synthetic only".encode_utf16().collect(),
                vec![],
            ],
            managed_root_path: "C:\\owned-managed".encode_utf16().collect(),
            cookie_owner_path: "C:\\owned-cookie".encode_utf16().collect(),
            managed_intent: m,
            managed_index: 0,
            managed_nonce: [3; 16],
            cookie_intent: c,
            cookie_index: slot,
            cookie_nonce: [if slot == 0 { 6 } else { 7 }; 16],
            timeout_secs: 1,
            timeout_nanos: 1,
            max_log_bytes: 4096,
            runtime_environment: None,
        }
    }
    #[test]
    fn metadata_download_roundtrip_and_fixed_debug() {
        for (stage, slot) in [(ReaderStage::Metadata, 0), (ReaderStage::Download, 1)] {
            let c = config(stage, slot);
            let bytes = c.encode().unwrap();
            assert!(bytes.len() <= MAX_CONFIG_BYTES);
            let d = PairedGuardianConfig::decode(&bytes).unwrap();
            assert_eq!(d.encode().unwrap(), bytes);
            assert_eq!(d.timeout(), Duration::new(1, 1));
            assert_eq!(format!("{d:?}"), "PairedGuardianConfigRedacted");
            assert_eq!(
                format!("{ConfigError:?} {ConfigError}"),
                "PairedGuardianConfigError PairedGuardianConfigError"
            );
        }
    }
    #[test]
    fn only_exact_two_intents_and_stage_mapping_permitted() {
        for (stage, slot) in [
            (ReaderStage::Metadata, 1),
            (ReaderStage::Download, 0),
            (ReaderStage::VersionProbe, 0),
            (ReaderStage::DirectProcessing, 0),
        ] {
            assert!(config(stage, slot).validate().is_err());
        }
        for field in 0..4 {
            let mut c = config(ReaderStage::Metadata, 0);
            match field {
                0 => c.managed_nonce = [4; 16],
                1 => c.cookie_nonce = [8; 16],
                2 => c.managed_index = 1,
                _ => c.cookie_index = 1,
            }
            assert!(c.validate().is_err());
        }
        let mut c = config(ReaderStage::Metadata, 0);
        c.managed_intent.bind(0, 1, 1).unwrap();
        assert!(c.validate().is_err());
        let mut c = config(ReaderStage::Metadata, 0);
        c.cookie_intent.bind(0, 1, 1).unwrap();
        assert!(c.validate().is_err());
    }
    #[test]
    fn finite_timeout_and_log_boundaries() {
        let mut c = config(ReaderStage::Metadata, 0);
        for (secs, nanos) in [
            (0, 0),
            (1, 1_000_000_000),
            (86400, 1),
            (86401, 0),
            (u64::MAX, 0),
        ] {
            c.timeout_secs = secs;
            c.timeout_nanos = nanos;
            assert!(c.validate().is_err());
        }
        for (secs, nanos) in [(0, 1), (86399, 999_999_999), (86400, 0)] {
            c.timeout_secs = secs;
            c.timeout_nanos = nanos;
            assert!(c.validate().is_ok());
        }
        for limit in [0, super::super::MAX_LOG_BYTES + 1] {
            c.max_log_bytes = limit;
            assert!(c.validate().is_err());
        }
        c.max_log_bytes = super::super::MAX_LOG_BYTES;
        assert!(c.validate().is_ok());
    }
    #[test]
    fn utf16_lengths_literal_units_and_argument_total() {
        let c = config(ReaderStage::Metadata, 0);
        for selector in 0..3 {
            let mut bad = c.clone();
            let target = match selector {
                0 => &mut bad.executable,
                1 => &mut bad.managed_root_path,
                _ => &mut bad.cookie_owner_path,
            };
            target.clear();
            assert!(bad.validate().is_err());
            let target = match selector {
                0 => &mut bad.executable,
                1 => &mut bad.managed_root_path,
                _ => &mut bad.cookie_owner_path,
            };
            *target = vec![0xd800; MAX_WIDE_UNITS];
            assert!(bad.validate().is_ok());
            let target = match selector {
                0 => &mut bad.executable,
                1 => &mut bad.managed_root_path,
                _ => &mut bad.cookie_owner_path,
            };
            target.push(1);
            assert!(bad.validate().is_err());
        }
        let mut bad = c.clone();
        bad.args = vec![vec![]; 256];
        assert!(bad.validate().is_ok());
        bad.args.push(vec![]);
        assert!(bad.validate().is_err());
        bad.args = vec![vec![1; MAX_WIDE_UNITS]];
        assert!(bad.validate().is_ok());
        bad.args.push(vec![1]);
        assert!(bad.validate().is_err());
        for selector in 0..4 {
            let mut bad = c.clone();
            match selector {
                0 => bad.executable.push(0),
                1 => bad.managed_root_path.push(0),
                2 => bad.cookie_owner_path.push(0),
                _ => bad.args.push(vec![0]),
            };
            assert!(bad.validate().is_err());
        }
    }
    #[test]
    fn nested_deserialize_unknown_domains_bounds_and_malformed() {
        let c = config(ReaderStage::Metadata, 0);
        let original: serde_json::Value = serde_json::from_slice(&c.encode().unwrap()).unwrap();
        for path in [
            "/version",
            "/managed_index",
            "/cookie_index",
            "/managed_nonce/0",
            "/cookie_nonce/0",
            "/timeout_nanos",
        ] {
            let mut value = original.clone();
            let field = value.pointer_mut(path).unwrap();
            let n = field.as_u64().unwrap();
            *field = (if path == "/timeout_nanos" {
                1_000_000_000
            } else {
                n + 1
            })
            .into();
            let bytes = serde_json::to_vec(&value).unwrap();
            assert!(PairedGuardianConfig::decode(&bytes).is_err(), "{path}");
            assert!(
                serde_json::from_slice::<PairedGuardianConfig>(&bytes).is_err(),
                "nested {path}"
            );
        }
        for path in [
            "/managed_intent/anchors/root/file_id",
            "/cookie_intent/anchors/copy/file_id",
        ] {
            let mut value = original.clone();
            *value.pointer_mut(path).unwrap() = serde_json::to_value([0u8; 16]).unwrap();
            let bytes = serde_json::to_vec(&value).unwrap();
            assert!(PairedGuardianConfig::decode(&bytes).is_err());
            assert!(serde_json::from_slice::<PairedGuardianConfig>(&bytes).is_err());
        }
        for domain in [
            "dlpad-windows-cookie-guardian-v1",
            "dlpad-windows-managed-guardian-v1",
        ] {
            let mut value = original.clone();
            value["domain"] = domain.into();
            assert!(PairedGuardianConfig::decode(&serde_json::to_vec(&value).unwrap()).is_err());
        }
        let mut value = original;
        value["foreign"] = true.into();
        assert!(PairedGuardianConfig::decode(&serde_json::to_vec(&value).unwrap()).is_err());
        let bytes = c.encode().unwrap();
        for b in [&bytes[..bytes.len() - 1], b"{}", b"\xff"] {
            assert!(PairedGuardianConfig::decode(b).is_err());
        }
        assert!(PairedGuardianConfig::decode(&vec![b' '; MAX_CONFIG_BYTES + 1]).is_err());
        let mut large = c;
        large.executable = vec![65535; MAX_WIDE_UNITS];
        large.managed_root_path = vec![65535; MAX_WIDE_UNITS];
        large.cookie_owner_path = vec![65535; MAX_WIDE_UNITS];
        large.args = vec![vec![65535; MAX_WIDE_UNITS]];
        assert!(large.validate().is_ok());
        assert!(large.encode().is_err());
    }
    #[cfg(windows)]
    #[test]
    fn native_spec_and_paths_preserve_literal_wide_units() {
        use std::os::windows::ffi::OsStrExt;
        let c = config(ReaderStage::Metadata, 0);
        let spec = c.process_spec().unwrap();
        assert_eq!(
            spec.executable
                .as_os_str()
                .encode_wide()
                .collect::<Vec<_>>(),
            c.executable
        );
        assert_eq!(spec.args[0].encode_wide().collect::<Vec<_>>(), c.args[0]);
        assert!(spec.args[1].is_empty());
        assert_eq!(
            c.managed_root_path()
                .unwrap()
                .as_os_str()
                .encode_wide()
                .collect::<Vec<_>>(),
            c.managed_root_path
        );
        assert_eq!(
            c.cookie_owner_path()
                .unwrap()
                .as_os_str()
                .encode_wide()
                .collect::<Vec<_>>(),
            c.cookie_owner_path
        );
    }
    #[test]
    fn optional_runtime_environment_old_none_and_some_wire() {
        let mut c = config(ReaderStage::Metadata, 0);
        let baseline = c.encode().unwrap();
        assert!(
            serde_json::from_slice::<serde_json::Value>(&baseline)
                .unwrap()
                .get("runtime_environment")
                .is_none()
        );
        assert_eq!(
            PairedGuardianConfig::decode(&baseline)
                .unwrap()
                .encode()
                .unwrap(),
            baseline
        );
        c.runtime_environment = Some(
            RuntimeEnvironmentConfig::new(r"C:\owned 한글 cache".encode_utf16().collect(), 1)
                .unwrap(),
        );
        let bytes = c.encode().unwrap();
        let decoded = PairedGuardianConfig::decode(&bytes).unwrap();
        assert_eq!(decoded.runtime_environment, c.runtime_environment);
        assert_eq!(decoded.encode().unwrap(), bytes);
        for bad in [
            serde_json::json!({"version":2,"cache_path":[67,58,92,120]}),
            serde_json::json!({"version":1,"cache_path":[0]}),
            serde_json::json!({"version":1,"cache_path":[55296]}),
            serde_json::json!({"version":1,"cache_path":[67,58,92,120],"foreign":true}),
        ] {
            let mut value: serde_json::Value = serde_json::from_slice(&baseline).unwrap();
            value["runtime_environment"] = bad;
            assert!(PairedGuardianConfig::decode(&serde_json::to_vec(&value).unwrap()).is_err());
        }
    }
    #[cfg(windows)]
    #[test]
    fn process_spec_restores_optional_environment() {
        let mut c = config(ReaderStage::Metadata, 0);
        c.runtime_environment = Some(
            RuntimeEnvironmentConfig::new(r"C:\owned cache".encode_utf16().collect(), 1).unwrap(),
        );
        let native = c.process_spec().unwrap().runtime_environment.unwrap();
        assert_eq!(
            RuntimeEnvironmentConfig::from_native(&native).unwrap(),
            c.runtime_environment.unwrap()
        );
    }
}
