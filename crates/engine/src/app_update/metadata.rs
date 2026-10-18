use ring::signature::{ED25519, UnparsedPublicKey};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

pub const MAX_METADATA_BYTES: usize = 64 * 1024;
pub const MAX_PACKAGE_BYTES: u64 = 256 * 1024 * 1024;
const MAX_KEY_COUNT: usize = 8;
const DOMAIN: &str = "dlpad.app-update.v1";
const REPOSITORY: &str = "joongi007/dlpad";
const TARGET: &str = "x86_64-pc-windows-msvc";
const MAX_VALIDITY_SECONDS: u64 = 31 * 24 * 60 * 60;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Stable,
    Preview,
}

/// Compiled/caller-pinned verification keys, never fetched from the candidate.
pub struct TrustedKey<'a> {
    pub id: &'a str,
    pub public_key: &'a [u8; 32],
}

pub struct Policy<'a> {
    pub installed_version: &'a str,
    pub installed_sequence: u64,
    pub channel: Channel,
    pub now_unix: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    MetadataSize,
    UnknownKey,
    InvalidKeyring,
    Signature,
    Metadata,
    Policy,
    Cancelled,
    TimedOut,
    PackageSize,
    PackageHash,
    PackageRead,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "app update validation failed ({self:?})")
    }
}
impl std::error::Error for Error {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Metadata {
    schema: u32,
    domain: String,
    key_id: String,
    sequence: u64,
    version: String,
    channel: Channel,
    target: String,
    repository: String,
    asset_name: String,
    asset_url: String,
    asset_bytes: u64,
    asset_sha256: String,
    issued_at_unix: u64,
    expires_at_unix: u64,
}

/// A signed, policy-checked observation. It is neither installation nor publisher Authenticode authority.
pub struct VerifiedCandidate {
    metadata: Metadata,
    digest: [u8; 32],
    signed_payload: Box<[u8]>,
    signature_hex: String,
}
impl std::fmt::Debug for VerifiedCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerifiedCandidate")
            .field("metadata", &self.metadata)
            .field("digest", &self.digest)
            .finish_non_exhaustive()
    }
}
impl VerifiedCandidate {
    /// Original public evidence for later signature verification, never a launch capability.
    pub fn signed_payload(&self) -> &[u8] {
        &self.signed_payload
    }
    pub fn signature_hex(&self) -> &str {
        &self.signature_hex
    }
    pub fn key_id(&self) -> &str {
        &self.metadata.key_id
    }
    pub fn version(&self) -> &str {
        &self.metadata.version
    }
    pub fn sequence(&self) -> u64 {
        self.metadata.sequence
    }
    pub fn channel(&self) -> Channel {
        self.metadata.channel
    }
    pub fn asset_name(&self) -> &str {
        &self.metadata.asset_name
    }
    pub fn asset_url(&self) -> &str {
        &self.metadata.asset_url
    }
    pub fn asset_sha256(&self) -> &[u8; 32] {
        &self.digest
    }
    pub fn asset_bytes(&self) -> u64 {
        self.metadata.asset_bytes
    }
    pub fn expires_at_unix(&self) -> u64 {
        self.metadata.expires_at_unix
    }

    /// Hash a bounded stream without buffering, writing, installing or executing it.
    /// The caller must retain/pin the original file before using this observation.
    /// Cancellation is cooperative between Read calls; the transport must bound each blocking call.
    pub fn verify_package(
        &self,
        reader: &mut impl Read,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<VerifiedPackage, Error> {
        let mut digest = Sha256::new();
        let mut total = 0u64;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            checkpoint(cancel, deadline)?;
            // At most one byte past the signed count is ever consumed.
            let remaining = self.metadata.asset_bytes - total;
            let limit = usize::try_from(remaining.saturating_add(1))
                .unwrap_or(buffer.len())
                .min(buffer.len());
            let count = reader
                .read(&mut buffer[..limit])
                .map_err(|_| Error::PackageRead)?;
            checkpoint(cancel, deadline)?;
            if count == 0 {
                break;
            }
            total = total.checked_add(count as u64).ok_or(Error::PackageSize)?;
            if total > self.metadata.asset_bytes {
                return Err(Error::PackageSize);
            }
            digest.update(&buffer[..count]);
        }
        if total != self.metadata.asset_bytes {
            return Err(Error::PackageSize);
        }
        let observed: [u8; 32] = digest.finalize().into();
        if observed != self.digest {
            return Err(Error::PackageHash);
        }
        Ok(VerifiedPackage {
            bytes: total,
            sha256: observed,
        })
    }
}

/// Content observation only; does not retain a file identity or confer mutation authority.
#[derive(Debug)]
pub struct VerifiedPackage {
    bytes: u64,
    sha256: [u8; 32],
}
impl VerifiedPackage {
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
    pub fn sha256(&self) -> &[u8; 32] {
        &self.sha256
    }
}
fn checkpoint(cancel: &AtomicBool, deadline: Instant) -> Result<(), Error> {
    if cancel.load(Ordering::Acquire) {
        return Err(Error::Cancelled);
    }
    if Instant::now() >= deadline {
        return Err(Error::TimedOut);
    }
    Ok(())
}
fn key_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|v| v.is_ascii_alphanumeric() || matches!(v, b'-' | b'_'))
}
fn hex<const N: usize>(value: &str) -> Option<[u8; N]> {
    if value.len() != N * 2 {
        return None;
    }
    let mut result = [0u8; N];
    for (i, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        fn digit(v: u8) -> Option<u8> {
            match v {
                b'0'..=b'9' => Some(v - b'0'),
                b'a'..=b'f' => Some(v - b'a' + 10),
                _ => None,
            }
        }
        result[i] = digit(pair[0])? * 16 + digit(pair[1])?;
    }
    Some(result)
}
fn version(value: &str) -> Option<[u32; 3]> {
    if value.len() > 32 {
        return None;
    }
    let mut result = [0u32; 3];
    let mut parts = value.split('.');
    for item in &mut result {
        let part = parts.next()?;
        if part.is_empty()
            || part.len() > 9
            || (part.len() > 1 && part.starts_with('0'))
            || !part.bytes().all(|v| v.is_ascii_digit())
        {
            return None;
        }
        *item = part.parse().ok()?;
    }
    parts.next().is_none().then_some(result)
}

/// Verify the exact unmodified payload bytes before parsing any metadata fields.
/// `key_id_hint` selects only a pre-pinned key; its signed counterpart must match.
pub fn verify_metadata(
    payload: &[u8],
    key_id_hint: &str,
    signature_hex: &str,
    keys: &[TrustedKey<'_>],
    policy: &Policy<'_>,
) -> Result<VerifiedCandidate, Error> {
    if payload.is_empty() || payload.len() > MAX_METADATA_BYTES {
        return Err(Error::MetadataSize);
    }
    if keys.is_empty()
        || keys.len() > MAX_KEY_COUNT
        || keys
            .iter()
            .enumerate()
            .any(|(i, key)| !key_id(key.id) || keys[..i].iter().any(|other| other.id == key.id))
    {
        return Err(Error::InvalidKeyring);
    }
    let key = keys
        .iter()
        .find(|key| key.id == key_id_hint)
        .ok_or(Error::UnknownKey)?;
    let signature = hex::<64>(signature_hex).ok_or(Error::Signature)?;
    UnparsedPublicKey::new(&ED25519, key.public_key)
        .verify(payload, &signature)
        .map_err(|_| Error::Signature)?;
    let metadata: Metadata = serde_json::from_slice(payload).map_err(|_| Error::Metadata)?;
    let installed = version(policy.installed_version).ok_or(Error::Policy)?;
    let candidate = version(&metadata.version).ok_or(Error::Policy)?;
    let digest = hex::<32>(&metadata.asset_sha256).ok_or(Error::Policy)?;
    let expected_name = format!("dlpad-{}-windows-x86_64-installer.zip", metadata.version);
    let expected_url = format!(
        "https://github.com/{REPOSITORY}/releases/download/v{}/{expected_name}",
        metadata.version
    );
    if metadata.schema != 1
        || metadata.domain != DOMAIN
        || metadata.key_id != key_id_hint
        || metadata.sequence <= policy.installed_sequence
        || candidate <= installed
        || metadata.channel != policy.channel
        || metadata.target != TARGET
        || metadata.repository != REPOSITORY
        || metadata.asset_name != expected_name
        || metadata.asset_url != expected_url
        || metadata.asset_bytes == 0
        || metadata.asset_bytes > MAX_PACKAGE_BYTES
        || metadata.issued_at_unix > policy.now_unix
        || metadata.expires_at_unix <= policy.now_unix
        || metadata
            .expires_at_unix
            .checked_sub(metadata.issued_at_unix)
            .is_none_or(|v| v == 0 || v > MAX_VALIDITY_SECONDS)
    {
        return Err(Error::Policy);
    }
    Ok(VerifiedCandidate {
        metadata,
        digest,
        signed_payload: payload.into(),
        signature_hex: signature_hex.into(),
    })
}

#[cfg(test)]
mod tests;
