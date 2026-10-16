//! Bounded managed-tool handoff; no native ID, receipt, or root ownership authority.
use super::windows_runtime_environment_config::RuntimeEnvironmentConfig;
use crate::tools::windows_managed_records::ManagedReaderRecord;
use serde::{Deserialize, Serialize};
use std::{fmt, time::Duration};

pub(super) const MAX_CONFIG_BYTES: usize = 256 * 1024;
const MAX_WIDE_UNITS: usize = 32766;
const MAX_TIMEOUT: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ManagedGuardianConfig {
    pub(super) version: u8,
    pub(super) executable: Vec<u16>,
    pub(super) args: Vec<Vec<u16>>,
    pub(super) root_path: Vec<u16>,
    pub(super) intent: ManagedReaderRecord,
    pub(super) index: u8,
    pub(super) nonce: [u8; 16],
    pub(super) timeout_secs: u64,
    pub(super) timeout_nanos: u32,
    pub(super) max_log_bytes: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) runtime_environment: Option<RuntimeEnvironmentConfig>,
}

impl fmt::Debug for ManagedGuardianConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ManagedGuardianConfigRedacted")
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct ConfigError;
impl fmt::Debug for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ManagedGuardianConfigError")
    }
}
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ManagedGuardianConfigError")
    }
}
impl std::error::Error for ConfigError {}

impl ManagedGuardianConfig {
    pub(super) fn validate(&self) -> Result<(), ConfigError> {
        if let Some(policy) = &self.runtime_environment {
            policy.validate().map_err(|_| ConfigError)?;
        }
        let wide = |v: &[u16]| v.len() <= MAX_WIDE_UNITS && !v.contains(&0);
        if self.version != 1
            || self.index > 7
            || self.nonce == [0; 16]
            || self.timeout_nanos >= 1_000_000_000
            || (self.timeout_secs == 0 && self.timeout_nanos == 0)
            || self.timeout_secs > MAX_TIMEOUT.as_secs()
            || (self.timeout_secs == MAX_TIMEOUT.as_secs() && self.timeout_nanos != 0)
            || !(1..=super::MAX_LOG_BYTES).contains(&self.max_log_bytes)
            || self.executable.is_empty()
            || !wide(&self.executable)
            || self.root_path.is_empty()
            || !wide(&self.root_path)
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
        self.intent.encode().map_err(|_| ConfigError)?;
        let identity = self
            .intent
            .clone()
            .bind(self.index, 1, 1)
            .map_err(|_| ConfigError)?;
        if identity.nonce != self.nonce {
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
    pub(super) fn root_path(&self) -> Result<std::path::PathBuf, ConfigError> {
        use std::{ffi::OsString, os::windows::ffi::OsStringExt};
        self.validate()?;
        Ok(OsString::from_wide(&self.root_path).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::windows_managed_records::{ReaderStage, SessionAnchors, WindowsAnchor};

    fn config() -> ManagedGuardianConfig {
        let anchor = |id| WindowsAnchor {
            volume_serial: 1,
            file_id: [id; 16],
        };
        let mut intent = ManagedReaderRecord::new(
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
        .unwrap();
        intent.begin(0, ReaderStage::VersionProbe, [4; 16]).unwrap();
        ManagedGuardianConfig {
            version: 1,
            executable: "C:\\secret-tool.exe".encode_utf16().collect(),
            args: vec!["credential=secret".encode_utf16().collect(), vec![]],
            root_path: "C:\\private-owner".encode_utf16().collect(),
            intent,
            index: 0,
            nonce: [4; 16],
            timeout_secs: 1,
            timeout_nanos: 1,
            max_log_bytes: 1,
            runtime_environment: None,
        }
    }

    #[test]
    fn roundtrip_and_fixed_redaction() {
        let c = config();
        let bytes = c.encode().unwrap();
        let decoded = ManagedGuardianConfig::decode(&bytes).unwrap();
        assert_eq!(decoded.encode().unwrap(), bytes);
        assert_eq!(decoded.timeout(), Duration::new(1, 1));
        assert_eq!(format!("{decoded:?}"), "ManagedGuardianConfigRedacted");
        assert_eq!(
            format!("{ConfigError:?} {ConfigError}"),
            "ManagedGuardianConfigError ManagedGuardianConfigError"
        );
    }

    #[test]
    fn invalid_scalars_and_timeout_boundaries() {
        let mut c = config();
        c.version = 2;
        assert!(c.validate().is_err());
        c.version = 1;
        c.index = 8;
        assert!(c.validate().is_err());
        c.index = 0;
        c.nonce = [0; 16];
        assert!(c.validate().is_err());
        c.nonce = [4; 16];
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
        c.max_log_bytes = 0;
        assert!(c.validate().is_err());
        c.max_log_bytes = super::super::MAX_LOG_BYTES + 1;
        assert!(c.validate().is_err());
        c.max_log_bytes -= 1;
        assert!(c.validate().is_ok());
    }

    #[test]
    fn wide_and_argument_bounds() {
        let mut c = config();
        c.executable.clear();
        assert!(c.validate().is_err());
        c.executable = vec![1; MAX_WIDE_UNITS];
        assert!(c.validate().is_ok());
        c.executable.push(1);
        assert!(c.validate().is_err());
        c = config();
        c.root_path.clear();
        assert!(c.validate().is_err());
        c = config();
        c.root_path.push(0);
        assert!(c.validate().is_err());
        c = config();
        c.executable.push(0);
        assert!(c.validate().is_err());
        c = config();
        c.args = vec![vec![]; 256];
        assert!(c.validate().is_ok());
        c.args.push(vec![]);
        assert!(c.validate().is_err());
        c.args = vec![vec![1; MAX_WIDE_UNITS]];
        assert!(c.validate().is_ok());
        c.args.push(vec![1]);
        assert!(c.validate().is_err());
        c.args = vec![vec![0]];
        assert!(c.validate().is_err());
    }

    #[test]
    fn only_matching_intent_can_be_handed_off() {
        let mut c = config();
        c.intent = ManagedReaderRecord::new(
            c.intent.session_nonce(),
            c.intent.record_digest(),
            c.intent.anchors(),
        )
        .unwrap();
        assert!(c.validate().is_err());
        c = config();
        c.nonce = [5; 16];
        assert!(c.validate().is_err());
        c = config();
        c.index = 1;
        assert!(c.validate().is_err());
        c = config();
        c.intent.bind(0, 1, 1).unwrap();
        assert!(c.validate().is_err());
        let receipt = c.intent.receipt_for(0).unwrap();
        c.intent.resolve(&receipt).unwrap();
        assert!(c.validate().is_err());
        c.intent.begin(1, ReaderStage::Metadata, [5; 16]).unwrap();
        c.index = 1;
        c.nonce = [5; 16];
        assert!(c.validate().is_ok());
    }

    #[test]
    fn all_eight_typed_stages_require_fresh_matching_intent() {
        let mut c = config();
        for index in 0..8 {
            c.index = index;
            c.nonce = [index + 4; 16];
            if index > 0 {
                c.intent
                    .begin(index, ReaderStage::DirectProcessing, c.nonce)
                    .unwrap();
            }
            assert!(c.validate().is_ok());
            c.intent.bind(index, 1, 1).unwrap();
            assert!(c.validate().is_err());
            let receipt = c.intent.receipt_for(index).unwrap();
            c.intent.resolve(&receipt).unwrap();
            assert!(c.validate().is_err());
        }
    }

    #[test]
    fn unknown_malformed_and_oversized_json_rejected() {
        let mut value: serde_json::Value =
            serde_json::from_slice(&config().encode().unwrap()).unwrap();
        value["credential"] = "secret".into();
        assert!(ManagedGuardianConfig::decode(&serde_json::to_vec(&value).unwrap()).is_err());
        assert!(ManagedGuardianConfig::decode(b"secret invalid json").is_err());
        assert!(ManagedGuardianConfig::decode(&vec![b' '; MAX_CONFIG_BYTES + 1]).is_err());
        let mut c = config();
        c.executable = vec![65535; MAX_WIDE_UNITS];
        c.root_path = vec![65535; MAX_WIDE_UNITS];
        c.args = vec![vec![65535; MAX_WIDE_UNITS]];
        assert!(c.validate().is_ok());
        assert!(c.encode().is_err());
    }
    #[test]
    fn optional_runtime_environment_old_none_and_some_wire() {
        let mut c = config();
        let baseline = c.encode().unwrap();
        assert!(
            serde_json::from_slice::<serde_json::Value>(&baseline)
                .unwrap()
                .get("runtime_environment")
                .is_none()
        );
        assert_eq!(
            ManagedGuardianConfig::decode(&baseline)
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
        let decoded = ManagedGuardianConfig::decode(&bytes).unwrap();
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
            assert!(ManagedGuardianConfig::decode(&serde_json::to_vec(&value).unwrap()).is_err());
        }
    }
    #[cfg(windows)]
    #[test]
    fn process_spec_restores_optional_environment() {
        let mut c = config();
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
