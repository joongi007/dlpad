//! Worker-only, unauthenticated HTTPS transport for pinned signed update candidates.
//! Reading a package confers no installation authority. The staging owner verifies it.
use super::{
    Error as ValidationError, MAX_METADATA_BYTES, Policy, TrustedKey, VerifiedCandidate,
    verify_metadata,
};
use std::{
    io::{self, Read},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidUrl,
    RedirectLimit,
    Http(u16),
    Network,
    Cancelled,
    TimedOut,
    BodyLimit,
    SignatureFormat,
    Validation(ValidationError),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "app update transport failed ({self:?})")
    }
}
impl std::error::Error for Error {}

/// A single response. Implementations must not follow redirects or send credentials.
pub struct Response {
    pub status: u16,
    pub location: Option<String>,
    pub body: Box<dyn Read>,
}
/// Implementations bound blocking calls by the supplied remaining global time.
pub trait Transport {
    fn request(&mut self, url: &str, remaining: Duration) -> Result<Response, Error>;
}
pub struct OfficialHttps;
impl Transport for OfficialHttps {
    fn request(&mut self, url: &str, remaining: Duration) -> Result<Response, Error> {
        let config = ureq::Agent::config_builder()
            .https_only(true)
            .proxy(None)
            .max_redirects(0)
            .http_status_as_error(false)
            .max_response_header_size(16384)
            .max_idle_connections(0)
            .timeout_global(Some(remaining))
            .timeout_connect(Some(remaining))
            .timeout_recv_response(Some(remaining))
            .timeout_recv_body(Some(remaining))
            .build();
        let response = ureq::Agent::new_with_config(config)
            .get(url)
            .header("User-Agent", "dlpad-app-update")
            .header("Accept", "application/octet-stream")
            .call()
            .map_err(|e| match e {
                ureq::Error::Timeout(_) => Error::TimedOut,
                _ => Error::Network,
            })?;
        let status = response.status().as_u16();
        let mut values = response.headers().get_all("location").iter();
        let location = values
            .next()
            .map(|v| v.to_str().map(str::to_owned).map_err(|_| Error::InvalidUrl))
            .transpose()?;
        if values.next().is_some() {
            return Err(Error::InvalidUrl);
        }
        Ok(Response {
            status,
            location,
            body: Box::new(response.into_body().into_reader()),
        })
    }
}
fn checkpoint(cancel: &AtomicBool, deadline: Instant) -> Result<Duration, Error> {
    if cancel.load(Ordering::Acquire) {
        return Err(Error::Cancelled);
    }
    deadline
        .checked_duration_since(Instant::now())
        .filter(|v| !v.is_zero())
        .ok_or(Error::TimedOut)
}
fn canonical_version(version: &str) -> bool {
    if version.len() > 32 {
        return false;
    }
    let mut count = 0;
    for component in version.split('.') {
        if component.is_empty()
            || component.len() > 9
            || (component.len() > 1 && component.starts_with('0'))
            || !component.bytes().all(|v| v.is_ascii_digit())
            || component.parse::<u32>().is_err()
        {
            return false;
        }
        count += 1;
    }
    count == 3
}
/// The only permitted GitHub path is the exact initial release asset path.
fn checked_url(url: &str, initial: &str) -> Result<(), Error> {
    if url.len() > 8192 || url.contains(['\r', '\n', '#', '\\']) {
        return Err(Error::InvalidUrl);
    }
    let uri: ureq::http::Uri = url.parse().map_err(|_| Error::InvalidUrl)?;
    let host = uri.host().ok_or(Error::InvalidUrl)?;
    let authority = uri.authority().ok_or(Error::InvalidUrl)?.as_str();
    if uri.scheme_str() != Some("https")
        || authority.contains(['@', '%'])
        || (authority != host && authority != format!("{host}:443"))
    {
        return Err(Error::InvalidUrl);
    }
    match host {
        "github.com" if url == initial && uri.query().is_none() => Ok(()),
        "release-assets.githubusercontent.com"
            if uri.path().starts_with("/github-production-release-asset/") =>
        {
            Ok(())
        }
        "objects.githubusercontent.com" | "github-releases.githubusercontent.com"
            if uri.query().is_none()
                && uri.path().starts_with("/github-production-release-asset/") =>
        {
            Ok(())
        }
        _ => Err(Error::InvalidUrl),
    }
}
fn open(
    transport: &mut impl Transport,
    initial: &str,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<Box<dyn Read>, Error> {
    let mut current = initial.to_owned();
    for hop in 0..=4 {
        let remaining = checkpoint(cancel, deadline)?;
        checked_url(&current, initial)?;
        let response = transport.request(&current, remaining)?;
        checkpoint(cancel, deadline)?;
        if response.status == 200 {
            return Ok(response.body);
        }
        if !matches!(response.status, 301 | 302 | 303 | 307 | 308) {
            return Err(Error::Http(response.status));
        }
        if hop == 4 {
            return Err(Error::RedirectLimit);
        }
        let next = response.location.ok_or(Error::InvalidUrl)?;
        checked_url(&next, initial)?;
        current = next;
    }
    Err(Error::RedirectLimit)
}
fn read_bounded(
    reader: &mut dyn Read,
    maximum: usize,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<Vec<u8>, Error> {
    let mut data = Vec::with_capacity(maximum.min(4096));
    let mut buffer = [0u8; 4096];
    loop {
        checkpoint(cancel, deadline)?;
        let limit = (maximum + 1 - data.len()).min(buffer.len());
        let read = reader.read(&mut buffer[..limit]);
        checkpoint(cancel, deadline)?;
        let count = read.map_err(|error| {
            if error.kind() == io::ErrorKind::TimedOut
                || matches!(
                    error
                        .get_ref()
                        .and_then(|inner| inner.downcast_ref::<ureq::Error>()),
                    Some(ureq::Error::Timeout(_))
                )
            {
                Error::TimedOut
            } else {
                Error::Network
            }
        })?;
        if count == 0 {
            return Ok(data);
        }
        data.extend_from_slice(&buffer[..count]);
        if data.len() > maximum {
            return Err(Error::BodyLimit);
        }
    }
}
/// Fetch immutable metadata and signature; package requests are impossible before verification.
/// key_id_hint selects a caller-pinned key; no public key is fetched from the network.
pub fn fetch_candidate(
    transport: &mut impl Transport,
    version: &str,
    key_id_hint: &str,
    keys: &[TrustedKey<'_>],
    policy: &Policy<'_>,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<VerifiedCandidate, Error> {
    checkpoint(cancel, deadline)?;
    if !canonical_version(version) {
        return Err(Error::InvalidUrl);
    }
    let base = format!(
        "https://github.com/joongi007/dlpad/releases/download/v{version}/update-windows-x86_64.json"
    );
    let mut reader = open(transport, &base, cancel, deadline)?;
    let payload = read_bounded(&mut reader, MAX_METADATA_BYTES, cancel, deadline)?;
    let mut reader = open(transport, &format!("{base}.sig"), cancel, deadline)?;
    let signature = read_bounded(&mut reader, 129, cancel, deadline)?;
    let signature = signature.strip_suffix(b"\n").unwrap_or(&signature);
    if signature.len() != 128
        || !signature
            .iter()
            .all(|v| v.is_ascii_digit() || matches!(v, b'a'..=b'f'))
    {
        return Err(Error::SignatureFormat);
    }
    let signature = std::str::from_utf8(signature).map_err(|_| Error::SignatureFormat)?;
    let candidate = verify_metadata(&payload, key_id_hint, signature, keys, policy)
        .map_err(Error::Validation)?;
    if candidate.version() != version {
        return Err(Error::InvalidUrl);
    }
    checkpoint(cancel, deadline)?;
    Ok(candidate)
}
struct GuardedReader {
    body: Box<dyn Read>,
    cancel: Arc<AtomicBool>,
    deadline: Instant,
    remaining: u64,
}
impl Read for GuardedReader {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        checkpoint(&self.cancel, self.deadline).map_err(io::Error::other)?;
        if self.remaining == 0 {
            return Err(io::Error::other(Error::BodyLimit));
        }
        let limit = usize::try_from(self.remaining)
            .unwrap_or(usize::MAX)
            .min(output.len());
        let result = self.body.read(&mut output[..limit]);
        checkpoint(&self.cancel, self.deadline).map_err(io::Error::other)?;
        let count = result?;
        self.remaining -= count as u64;
        Ok(count)
    }
}
/// Stream into worker-owned staging. The staging owner MUST verify the signed length/hash
/// and retain original file identity before publication. At most signed length + 1 is read.
pub fn open_package(
    transport: &mut impl Transport,
    candidate: &VerifiedCandidate,
    cancel: Arc<AtomicBool>,
    deadline: Instant,
) -> Result<Box<dyn Read>, Error> {
    let body = open(transport, candidate.asset_url(), &cancel, deadline)?;
    Ok(Box::new(GuardedReader {
        body,
        cancel,
        deadline,
        remaining: candidate.asset_bytes() + 1,
    }))
}

#[cfg(test)]
mod tests;
