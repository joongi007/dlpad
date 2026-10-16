//! Pure, fixed Windows managed-Deno environment policy. No native ownership proof.
use std::{fmt, path::PathBuf};

const MAX_BLOCK_UNITS: usize = 32_767;
const MAX_VARIABLES: usize = 4_096;
const MAX_KEY_UNITS: usize = 256;
const POLICY_VERSION: u8 = 1;
const CACHE_KEY: &str = "DENO_DIR";
const SHIM_KEY: &str = "DENO_DISABLE_NODE_SHIM";

#[cfg(windows)]
use std::sync::{Arc, Mutex};
#[cfg(windows)]
pub(crate) struct EnvironmentAuthority {
    pub(crate) cache: Mutex<crate::windows_owned_deno_cache::OwnedDenoCache>,
    _package: crate::tools::VerifiedManagedDeno,
}
#[derive(Clone)]
pub struct WindowsManagedDenoEnvironment {
    cache_path: PathBuf,
    policy_version: u8,
    #[cfg(any(windows, test))]
    binding: Option<crate::windows_deno_cache_binding::CacheBinding>,
    #[cfg(windows)]
    authority: Option<Arc<EnvironmentAuthority>>,
}
impl PartialEq for WindowsManagedDenoEnvironment {
    fn eq(&self, other: &Self) -> bool {
        let same =
            self.cache_path == other.cache_path && self.policy_version == other.policy_version;
        #[cfg(any(windows, test))]
        {
            same && self.binding == other.binding
        }
        #[cfg(not(any(windows, test)))]
        {
            same
        }
    }
}
impl Eq for WindowsManagedDenoEnvironment {}
impl fmt::Debug for WindowsManagedDenoEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WindowsManagedDenoEnvironmentRedacted")
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnvironmentError {
    InvalidPolicy,
    InvalidCachePath,
    InvalidKey,
    InvalidValue,
    DuplicateKey,
    BoundExceeded,
}
impl fmt::Debug for EnvironmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RuntimeEnvironmentError")
    }
}
impl fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("managed runtime environment rejected")
    }
}
impl std::error::Error for EnvironmentError {}

pub(crate) fn validate_cache_units(cache: &[u16], version: u8) -> Result<(), EnvironmentError> {
    if version != POLICY_VERSION {
        return Err(EnvironmentError::InvalidPolicy);
    }
    if cache.is_empty() || cache.len() > MAX_BLOCK_UNITS - 1 || cache.contains(&0) {
        return Err(EnvironmentError::InvalidCachePath);
    }
    let path = String::from_utf16(cache).map_err(|_| EnvironmentError::InvalidCachePath)?;
    let normalized = path.replace('/', "\\");
    let path = normalized.strip_prefix("\\\\?\\").unwrap_or(&normalized);
    // Verbatim disk paths are accepted; device/GLOBALROOT/extended-UNC domains are not.
    let disk = path.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
        && path.as_bytes().get(1) == Some(&b':')
        && path.as_bytes().get(2) == Some(&b'\\');
    let remainder = if disk {
        &path[3..]
    } else if !normalized.starts_with("\\\\?\\") && path.starts_with("\\\\") {
        let parts: Vec<_> = path[2..].split('\\').collect();
        if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(EnvironmentError::InvalidCachePath);
        }
        &path[2..]
    } else {
        return Err(EnvironmentError::InvalidCachePath);
    };
    if remainder
        .chars()
        .any(|c| c.is_control() || "<>\"|?*:".contains(c))
        || remainder
            .split('\\')
            .any(|part| part == "." || part == ".." || part.ends_with('.') || part.ends_with(' '))
    {
        return Err(EnvironmentError::InvalidCachePath);
    }
    Ok(())
}

// Windows inherited pseudoentries have one leading equals sign; embedded equals remains invalid.
fn key_name(key: &[u16]) -> Result<String, EnvironmentError> {
    if key.is_empty()
        || key.len() > MAX_KEY_UNITS
        || (key.len() == 1 && key[0] == u16::from(b'='))
        || key.iter().enumerate().any(|(index, unit)| {
            !(33..=126).contains(unit) || (*unit == u16::from(b'=') && index != 0)
        })
    {
        return Err(EnvironmentError::InvalidKey);
    }
    Ok(key
        .iter()
        .map(|unit| (*unit as u8 as char).to_ascii_uppercase())
        .collect())
}
fn valid_value(value: &[u16]) -> Result<(), EnvironmentError> {
    if value.len() > MAX_BLOCK_UNITS || value.contains(&0) {
        return Err(EnvironmentError::InvalidValue);
    }
    String::from_utf16(value).map_err(|_| EnvironmentError::InvalidValue)?;
    Ok(())
}

/// UTF-16 core, testable without a Windows process. Caller values are never logged.
fn encode_units<I>(cache: &[u16], version: u8, ambient: I) -> Result<Vec<u16>, EnvironmentError>
where
    I: IntoIterator<Item = (Vec<u16>, Vec<u16>)>,
{
    validate_cache_units(cache, version)?;
    let mut entries: Vec<(String, Vec<u16>, Vec<u16>)> = Vec::new();
    let mut input_count = 0usize;
    // Includes the final block terminator and both fixed variables before any ambient input.
    let mut units = 1 + CACHE_KEY.len() + 1 + cache.len() + 1 + SHIM_KEY.len() + 1 + 1 + 1;
    if units > MAX_BLOCK_UNITS {
        return Err(EnvironmentError::BoundExceeded);
    }
    for (key, value) in ambient {
        input_count += 1;
        if input_count > MAX_VARIABLES {
            return Err(EnvironmentError::BoundExceeded);
        }
        let name = key_name(&key)?;
        valid_value(&value)?;
        if name == CACHE_KEY || name == SHIM_KEY {
            continue;
        }
        if entries.iter().any(|(existing, _, _)| existing == &name) {
            return Err(EnvironmentError::DuplicateKey);
        }
        units = units
            .checked_add(key.len() + value.len() + 2)
            .ok_or(EnvironmentError::BoundExceeded)?;
        if units > MAX_BLOCK_UNITS || entries.len() + 3 > MAX_VARIABLES {
            return Err(EnvironmentError::BoundExceeded);
        }
        entries.push((name, key, value));
    }
    entries.push((
        CACHE_KEY.to_owned(),
        CACHE_KEY.encode_utf16().collect(),
        cache.to_vec(),
    ));
    entries.push((
        SHIM_KEY.to_owned(),
        SHIM_KEY.encode_utf16().collect(),
        vec![u16::from(b'1')],
    ));
    entries.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    let mut block = Vec::with_capacity(units);
    for (_, key, value) in entries {
        block.extend(key);
        block.push(u16::from(b'='));
        block.extend(value);
        block.push(0);
    }
    block.push(0);
    debug_assert_eq!(block.len(), units);
    Ok(block)
}

impl WindowsManagedDenoEnvironment {
    #[cfg(windows)]
    pub(crate) fn new(cache_path: PathBuf) -> Result<Self, EnvironmentError> {
        let policy = Self {
            cache_path,
            policy_version: POLICY_VERSION,
            binding: None,
            authority: None,
        };
        policy.validate()?;
        Ok(policy)
    }
    #[cfg(windows)]
    pub(crate) fn from_owned(
        cache: crate::windows_owned_deno_cache::OwnedDenoCache,
        verified: crate::tools::VerifiedManagedDeno,
    ) -> Result<Self, EnvironmentError> {
        cache
            .verify()
            .map_err(|_| EnvironmentError::InvalidPolicy)?;
        let binding = cache
            .binding()
            .map_err(|_| EnvironmentError::InvalidPolicy)?;
        if binding.root
            != verified
                .root_identity()
                .map_err(|_| EnvironmentError::InvalidPolicy)?
            || binding
                .root_path()
                .map_err(|_| EnvironmentError::InvalidPolicy)?
                != verified.root_path()
            || cache.path() != verified.root_path().join(".runtime-cache").join("deno")
        {
            return Err(EnvironmentError::InvalidPolicy);
        }
        let policy = Self {
            cache_path: cache.path().to_owned(),
            policy_version: POLICY_VERSION,
            binding: Some(binding),
            authority: Some(Arc::new(EnvironmentAuthority {
                cache: Mutex::new(cache),
                _package: verified,
            })),
        };
        policy.validate()?;
        Ok(policy)
    }
    #[cfg(windows)]
    pub(crate) fn from_handoff(
        cache_path: PathBuf,
        binding: Option<crate::windows_deno_cache_binding::CacheBinding>,
    ) -> Result<Self, EnvironmentError> {
        let mut policy = Self::new(cache_path)?;
        policy.binding = binding;
        policy.validate()?;
        Ok(policy)
    }
    #[cfg(any(windows, test))]
    pub(crate) fn cache_binding(&self) -> Option<&crate::windows_deno_cache_binding::CacheBinding> {
        self.binding.as_ref()
    }
    #[cfg(windows)]
    pub(crate) fn verify_original(
        &self,
        cancel: &Arc<std::sync::atomic::AtomicBool>,
        deadline: std::time::Instant,
    ) -> Result<(), crate::tools::ToolError> {
        self.validate()
            .map_err(|_| crate::tools::ToolError::UnsafePath)?;
        self.authority
            .as_ref()
            .ok_or(crate::tools::ToolError::UnsafePath)?
            ._package
            .verify_retained_until(cancel, deadline)
    }
    #[cfg(windows)]
    pub(crate) fn authority(&self) -> Option<&Arc<EnvironmentAuthority>> {
        self.authority.as_ref()
    }
    #[cfg(windows)]
    pub(crate) fn selected_executable(&self) -> Option<&std::path::Path> {
        self.authority.as_ref().map(|a| a._package.executable())
    }
    #[cfg(windows)]
    pub(crate) fn cache_path(&self) -> &std::path::Path {
        &self.cache_path
    }
    #[cfg(windows)]
    pub(crate) fn policy_version(&self) -> u8 {
        self.policy_version
    }

    /// Does not open/create/delete the cache or prove its owner. None callers keep old behavior.
    #[cfg(windows)]
    pub(crate) fn encode<I>(&self, ambient: I) -> Result<Vec<u16>, EnvironmentError>
    where
        I: IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
    {
        use std::os::windows::ffi::OsStrExt;
        let Self {
            cache_path,
            policy_version,
            ..
        } = self;
        let cache: Vec<u16> = cache_path
            .as_os_str()
            .encode_wide()
            .take(MAX_BLOCK_UNITS + 1)
            .collect();
        encode_units(
            &cache,
            *policy_version,
            ambient.into_iter().map(|(key, value)| {
                (
                    key.encode_wide().take(MAX_KEY_UNITS + 1).collect(),
                    value.encode_wide().take(MAX_BLOCK_UNITS + 1).collect(),
                )
            }),
        )
    }
    #[cfg(windows)]
    pub(crate) fn validate(&self) -> Result<(), EnvironmentError> {
        use std::os::windows::ffi::OsStrExt;
        let Self {
            cache_path,
            policy_version,
            ..
        } = self;
        let cache: Vec<u16> = cache_path
            .as_os_str()
            .encode_wide()
            .take(MAX_BLOCK_UNITS + 1)
            .collect();
        validate_cache_units(&cache, *policy_version)?;
        if let Some(binding) = &self.binding {
            binding
                .validate()
                .map_err(|_| EnvironmentError::InvalidPolicy)?;
            let expected = binding
                .root_path()
                .map_err(|_| EnvironmentError::InvalidPolicy)?
                .join(".runtime-cache")
                .join("deno");
            if self.cache_path != expected {
                return Err(EnvironmentError::InvalidPolicy);
            }
        }
        // Native authority is checked at capture under its own bounded lock; no nested lock here.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }
    fn ambient(key: &str, value: &str) -> (Vec<u16>, Vec<u16>) {
        (wide(key), wide(value))
    }
    fn lines(block: &[u16]) -> Vec<String> {
        assert!(block.ends_with(&[0, 0]));
        block[..block.len() - 1]
            .split(|unit| *unit == 0)
            .filter(|line| !line.is_empty())
            .map(|line| String::from_utf16(line).unwrap())
            .collect()
    }
    #[test]
    fn inherited_leading_equal_pseudoentry_is_preserved() {
        let block = encode_units(
            &wide(r"C:\owned cache"),
            1,
            [ambient("=SYNTHETIC_PSEUDO", "owned synthetic value")],
        )
        .unwrap();
        assert!(lines(&block).contains(&"=SYNTHETIC_PSEUDO=owned synthetic value".to_string()));
        assert!(encode_units(&wide(r"C:\owned cache"), 1, [ambient("KEY=NAME", "value")]).is_err());
        assert!(encode_units(&wide(r"C:\owned cache"), 1, [ambient("==", "value")]).is_err());
    }
    #[test]
    fn reserved_duplicates_replaced_and_unicode_literal_preserved() {
        let block = encode_units(
            &wide(r"C:\소유 폴더\cache"),
            1,
            [
                ambient("deno_dir", "external-secret"),
                ambient("DENO_DIR", "other-secret"),
                ambient("DeNo_Disable_Node_Shim", "0"),
                ambient("DENO_DISABLE_NODE_SHIM", "false"),
                ambient("ProgramFiles(x86)", r"C:\Program Files (x86)"),
                ambient("PATH", r"C:\도구 폴더"),
            ],
        )
        .unwrap();
        let values = lines(&block);
        assert_eq!(
            values.iter().filter(|v| v.starts_with("DENO_DIR=")).count(),
            1
        );
        assert_eq!(
            values
                .iter()
                .filter(|v| v.starts_with("DENO_DISABLE_NODE_SHIM="))
                .count(),
            1
        );
        assert!(values.contains(&r"DENO_DIR=C:\소유 폴더\cache".to_owned()));
        assert!(values.contains(&"DENO_DISABLE_NODE_SHIM=1".to_owned()));
        assert!(values.contains(&r"PATH=C:\도구 폴더".to_owned()));
        assert!(!values.iter().any(|value| value.contains("secret")));
    }
    #[test]
    fn duplicate_key_and_invalid_key_domains_rejected() {
        assert_eq!(
            encode_units(
                &wide(r"C:\cache"),
                1,
                [ambient("Path", "a"), ambient("PATH", "b")]
            ),
            Err(EnvironmentError::DuplicateKey)
        );
        for key in ["", "=C:=", "a=b", "white space", "한글", "line\n"] {
            assert_eq!(
                encode_units(&wide(r"C:\cache"), 1, [ambient(key, "x")]),
                Err(EnvironmentError::InvalidKey)
            );
        }
        assert_eq!(
            encode_units(&wide(r"C:\cache"), 1, [(vec![b'A' as u16; 257], wide("v"))]),
            Err(EnvironmentError::InvalidKey)
        );
    }
    #[test]
    fn malformed_values_and_reserved_malformed_values_rejected() {
        for key in ["VALUE", "DENO_DIR"] {
            for value in [
                vec![0],
                vec![0xd800],
                vec![0xdc00],
                vec![b'x' as u16; MAX_BLOCK_UNITS + 1],
            ] {
                assert_eq!(
                    encode_units(&wide(r"C:\cache"), 1, [(wide(key), value)]),
                    Err(EnvironmentError::InvalidValue)
                );
            }
        }
    }
    #[test]
    fn path_policy_rejects_relative_parent_device_stream_and_surrogate() {
        for path in [
            "relative",
            r"C:cache",
            r"\root",
            r"C:\safe\..\cache",
            r"C:\safe\.\cache",
            r"\\.\GLOBALROOT\cache",
            r"\\?\GLOBALROOT\cache",
            r"\\?\UNC\server\share",
            r"C:\cache:stream",
            r"C:\bad?\cache",
            r"C:\trailing.\cache",
            r"\\server",
            "C:\\cache\n",
        ] {
            assert_eq!(
                validate_cache_units(&wide(path), 1),
                Err(EnvironmentError::InvalidCachePath)
            );
        }
        assert_eq!(
            validate_cache_units(&[0xd800], 1),
            Err(EnvironmentError::InvalidCachePath)
        );
        assert_eq!(
            validate_cache_units(&wide(r"C:\cache"), 2),
            Err(EnvironmentError::InvalidPolicy)
        );
        assert!(validate_cache_units(&wide(r"\\?\C:\소유 cache"), 1).is_ok());
        assert!(validate_cache_units(&wide(r"\\server\share\cache"), 1).is_ok());
    }
    #[test]
    fn exact_block_size_and_variable_count_bounds() {
        let cache = wide(r"C:\cache");
        let base = encode_units(&cache, 1, std::iter::empty()).unwrap().len();
        let remaining = MAX_BLOCK_UNITS - base - 3;
        let block = encode_units(&cache, 1, [(wide("K"), vec![b'x' as u16; remaining])]).unwrap();
        assert_eq!(block.len(), MAX_BLOCK_UNITS);
        assert_eq!(
            encode_units(&cache, 1, [(wide("K"), vec![b'x' as u16; remaining + 1])]),
            Err(EnvironmentError::BoundExceeded)
        );
        let many = (0..4097).map(|_| ambient("DENO_DIR", "ignored"));
        assert_eq!(
            encode_units(&cache, 1, many),
            Err(EnvironmentError::BoundExceeded)
        );
        let retained = (0..4095).map(|i| ambient(&format!("K{i}"), ""));
        assert_eq!(
            encode_units(&cache, 1, retained),
            Err(EnvironmentError::BoundExceeded)
        );
    }
    #[test]
    fn policy_and_errors_never_debug_values() {
        let policy = WindowsManagedDenoEnvironment {
            cache_path: PathBuf::from("secret-cache"),
            policy_version: 1,
            binding: None,
            #[cfg(windows)]
            authority: None,
        };
        assert!(policy.cache_binding().is_none());
        assert_eq!(
            format!("{policy:?}"),
            "WindowsManagedDenoEnvironmentRedacted"
        );
        assert!(
            !EnvironmentError::InvalidValue
                .to_string()
                .contains("secret")
        );
    }
    #[cfg(windows)]
    #[test]
    fn windows_osstring_encoder_rejects_unpaired_surrogate() {
        use std::os::windows::ffi::OsStringExt;
        let policy = WindowsManagedDenoEnvironment {
            cache_path: PathBuf::from(r"C:\소유 cache"),
            policy_version: 1,
            binding: None,
            #[cfg(windows)]
            authority: None,
        };
        assert!(policy.validate().is_ok());
        assert_eq!(
            policy.encode([(
                std::ffi::OsString::from("VALUE"),
                std::ffi::OsString::from_wide(&[0xd800])
            )]),
            Err(EnvironmentError::InvalidValue)
        );
        let block = policy
            .encode([(
                std::ffi::OsString::from("dEnO_dIr"),
                std::ffi::OsString::from("external"),
            )])
            .unwrap();
        assert_eq!(
            lines(&block),
            vec![
                r"DENO_DIR=C:\소유 cache".to_owned(),
                "DENO_DISABLE_NODE_SHIM=1".to_owned()
            ]
        );
    }
}
