//! Worker-only official release observation and same-origin checksum capture.
//! No executable authority from metadata alone; no publisher-trust or active changes.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
const META_LIMIT: usize = 256 * 1024;
const CHECKSUM_LIMIT: usize = 64 * 1024;
const NETWORK_BUDGET: Duration = Duration::from_secs(20);
const ASSET_LIMIT: usize = 128;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Tool {
    YtDlp,
    Deno,
    Ffmpeg,
}
impl Tool {
    pub fn id(self) -> &'static str {
        match self {
            Self::YtDlp => "yt-dlp",
            Self::Deno => "deno",
            Self::Ffmpeg => "ffmpeg",
        }
    }
    fn repository(self) -> &'static str {
        match self {
            Self::YtDlp => "yt-dlp/yt-dlp",
            Self::Deno => "denoland/deno",
            Self::Ffmpeg => "BtbN/FFmpeg-Builds",
        }
    }
    fn latest_url(self) -> String {
        format!(
            "https://api.github.com/repos/{}/releases/latest",
            self.repository()
        )
    }
    fn asset(self, name: &str) -> bool {
        match self {
            Self::YtDlp => name == "yt-dlp.exe",
            Self::Deno => name == "deno-x86_64-pc-windows-msvc.zip",
            Self::Ffmpeg => name.starts_with("ffmpeg-") && name.ends_with("-win64-lgpl-9.0.zip"),
        }
    }
    fn checksum(self, asset: &str) -> String {
        match self {
            Self::YtDlp => "SHA2-256SUMS".into(),
            Self::Deno => format!("{asset}.sha256sum"),
            Self::Ffmpeg => "checksums.sha256".into(),
        }
    }
}
#[derive(Debug)]
pub enum DiscoveryError {
    Cancelled,
    TimedOut,
    Http(u16),
    Transport,
    Malformed,
    Limit,
    MissingAsset,
    MissingChecksum,
    ChecksumMismatch,
    Io(io::Error),
}
impl From<io::Error> for DiscoveryError {
    fn from(e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::TimedOut {
            Self::TimedOut
        } else {
            Self::Io(e)
        }
    }
}
#[derive(Debug, Deserialize)]
struct Asset {
    id: u64,
    name: String,
    browser_download_url: String,
    size: u64,
    digest: Option<String>,
}
#[derive(Debug, Deserialize)]
struct Release {
    id: u64,
    tag_name: String,
    draft: bool,
    prerelease: bool,
    published_at: String,
    #[serde(deserialize_with = "assets")]
    assets: Vec<Asset>,
}
fn assets<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Asset>, D::Error> {
    struct Bounded;
    impl<'de> serde::de::Visitor<'de> for Bounded {
        type Value = Vec<Asset>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most128 release assets")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(v) = seq.next_element()? {
                if out.len() == ASSET_LIMIT {
                    return Err(serde::de::Error::custom("asset bound"));
                }
                out.push(v);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Bounded)
}
#[derive(Debug, Clone, Serialize)]
pub struct Observation {
    pub tool: Tool,
    pub release_id: u64,
    pub tag: String,
    pub published_at: String,
    pub checked_unix_seconds: u64,
    pub asset_id: u64,
    pub asset_name: String,
    pub asset_url: String,
    pub asset_bytes: u64,
    pub metadata_sha256: String,
    pub publisher_trust_verified: bool,
}
/// Private fields prevent constructing installation pins by deserializing arbitrary JSON.
#[derive(Debug, Clone, Serialize)]
pub struct CapturedPin {
    observation: Observation,
    checksum_url: String,
    checksum_body_sha256: String,
    artifact_sha256: String,
    metadata_digest_matched: bool,
    publisher_trust_verified: bool,
}
impl CapturedPin {
    pub fn observation(&self) -> &Observation {
        &self.observation
    }
    pub fn artifact_sha256(&self) -> &str {
        &self.artifact_sha256
    }
}
fn token(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        && s != "."
        && s != ".."
}
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn checkpoint(cancel: &Arc<AtomicBool>, deadline: Instant) -> Result<Duration, DiscoveryError> {
    if cancel.load(Ordering::Acquire) {
        return Err(DiscoveryError::Cancelled);
    }
    deadline
        .checked_duration_since(Instant::now())
        .filter(|d| !d.is_zero())
        .ok_or(DiscoveryError::TimedOut)
}
fn parse(tool: Tool, raw: &[u8]) -> Result<Observation, DiscoveryError> {
    if raw.len() > META_LIMIT {
        return Err(DiscoveryError::Limit);
    }
    let release: Release = serde_json::from_slice(raw).map_err(|_| DiscoveryError::Malformed)?;
    if release.id == 0
        || release.draft
        || release.prerelease
        || !token(&release.tag_name)
        || release.published_at.len() > 64
        || !release.published_at.bytes().all(|b| b.is_ascii_graphic())
    {
        return Err(DiscoveryError::Malformed);
    }
    let mut ids = std::collections::HashSet::new();
    let mut names = std::collections::HashSet::new();
    if release
        .assets
        .iter()
        .any(|a| a.id == 0 || !ids.insert(a.id) || !names.insert(a.name.as_str()))
    {
        return Err(DiscoveryError::Malformed);
    }
    let mut matching = release.assets.into_iter().filter(|a| tool.asset(&a.name));
    let selected = matching.next().ok_or(DiscoveryError::MissingAsset)?;
    if matching.next().is_some()
        || selected.id == 0
        || !token(&selected.name)
        || selected.size == 0
        || selected.size > 256 * 1024 * 1024
    {
        return Err(DiscoveryError::Malformed);
    }
    let expected = format!(
        "https://github.com/{}/releases/download/{}/{}",
        tool.repository(),
        release.tag_name,
        selected.name
    );
    if selected.browser_download_url != expected {
        return Err(DiscoveryError::Malformed);
    }
    // Digest is checked during capture against the same-origin checksum, never called signature trust.
    if selected.digest.as_ref().is_some_and(|d| !valid_digest(d)) {
        return Err(DiscoveryError::Malformed);
    }
    Ok(Observation {
        tool,
        release_id: release.id,
        tag: release.tag_name,
        published_at: release.published_at,
        checked_unix_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| DiscoveryError::Malformed)?
            .as_secs(),
        asset_id: selected.id,
        asset_name: selected.name,
        asset_url: selected.browser_download_url,
        asset_bytes: selected.size,
        metadata_sha256: sha(raw),
        publisher_trust_verified: false,
    })
}
fn valid_digest(d: &str) -> bool {
    d.strip_prefix("sha256:")
        .is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
}
fn checksum(raw: &[u8], wanted: &str) -> Result<String, DiscoveryError> {
    if raw.len() > CHECKSUM_LIMIT {
        return Err(DiscoveryError::Limit);
    }
    let text = std::str::from_utf8(raw).map_err(|_| DiscoveryError::Malformed)?;
    let mut matched = None;
    let mut lines = 0;
    for row in text.lines() {
        lines += 1;
        if lines > 512 || row.len() > 1024 {
            return Err(DiscoveryError::Limit);
        }
        let parts: Vec<_> = row.split_ascii_whitespace().collect();
        if parts.len() != 2 {
            return Err(DiscoveryError::Malformed);
        }
        let name = parts[1].strip_prefix('*').unwrap_or(parts[1]);
        if !token(name) || parts[0].len() != 64 || !parts[0].bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(DiscoveryError::Malformed);
        }
        if name == wanted {
            if matched.is_some() {
                return Err(DiscoveryError::Malformed);
            }
            matched = Some(parts[0].to_ascii_lowercase());
        }
    }
    matched.ok_or(DiscoveryError::MissingChecksum)
}
/// Production only requests derived fixed public endpoints; no caller URL/auth/cookie arguments.
pub(super) trait Transport {
    fn open(
        &mut self,
        url: &str,
        timeout: Duration,
        metadata: bool,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Box<dyn Read>, DiscoveryError>;
}
struct OfficialHttps;
fn delivery_host(host: &str) -> bool {
    matches!(
        host,
        "api.github.com"
            | "github.com"
            | "release-assets.githubusercontent.com"
            | "objects.githubusercontent.com"
            | "github-releases.githubusercontent.com"
    )
}
/// Checked before each request, not after the redirect target has received a request.
fn checked_uri(url: &str, initial: bool) -> Result<(), DiscoveryError> {
    if url.len() > 8192 || url.contains(['\r', '\n', '#', '\\']) {
        return Err(DiscoveryError::Malformed);
    }
    let uri: ureq::http::Uri = url.parse().map_err(|_| DiscoveryError::Malformed)?;
    let authority = uri.authority().ok_or(DiscoveryError::Malformed)?;
    let host = uri.host().ok_or(DiscoveryError::Malformed)?;
    if uri.scheme_str() != Some("https")
        || authority.as_str().contains(['@', '%'])
        || (authority.as_str() != host && authority.as_str() != format!("{host}:443"))
        || !delivery_host(host)
        || (initial && (!matches!(host, "api.github.com" | "github.com") || uri.query().is_some()))
    {
        return Err(DiscoveryError::Malformed);
    }
    Ok(())
}
struct Hop {
    status: u16,
    location: Option<String>,
    body: Box<dyn Read>,
}
trait HopTransport {
    fn request(
        &mut self,
        url: &str,
        timeout: Duration,
        metadata: bool,
    ) -> Result<Hop, DiscoveryError>;
}
struct UreqHop;
impl HopTransport for UreqHop {
    fn request(
        &mut self,
        url: &str,
        timeout: Duration,
        metadata: bool,
    ) -> Result<Hop, DiscoveryError> {
        // ureq 3.4.2 config.rs:325: zero returns the redirect response without following it.
        let cfg = ureq::Agent::config_builder()
            .https_only(true)
            .proxy(None)
            .max_redirects(0)
            .http_status_as_error(false)
            .max_response_header_size(16384)
            .max_idle_connections(0)
            .timeout_global(Some(timeout))
            .timeout_connect(Some(timeout))
            .timeout_recv_response(Some(timeout))
            .build();
        let response = ureq::Agent::new_with_config(cfg)
            .get(url)
            .header("User-Agent", "dlpad-tool-release-observer")
            .header(
                "Accept",
                if metadata {
                    "application/vnd.github+json"
                } else {
                    "text/plain"
                },
            )
            .header("X-GitHub-Api-Version", "2026-03-10")
            .call()
            .map_err(|e| match e {
                ureq::Error::Timeout(_) => DiscoveryError::TimedOut,
                _ => DiscoveryError::Transport,
            })?;
        let status = response.status().as_u16();
        let locations = response.headers().get_all("location");
        let mut rows = locations.iter();
        let location = rows
            .next()
            .map(|v| {
                v.to_str()
                    .map(str::to_owned)
                    .map_err(|_| DiscoveryError::Malformed)
            })
            .transpose()?;
        if rows.next().is_some() {
            return Err(DiscoveryError::Malformed);
        }
        Ok(Hop {
            status,
            location,
            body: Box::new(response.into_body().into_reader()),
        })
    }
}
fn follow_checked(
    transport: &mut dyn HopTransport,
    url: &str,
    timeout: Duration,
    metadata: bool,
    cancel: &Arc<AtomicBool>,
) -> Result<Box<dyn Read>, DiscoveryError> {
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or(DiscoveryError::TimedOut)?;
    let mut current = url.to_owned();
    checked_uri(&current, true)?;
    // Four actual follow-ups; a fifth redirect cannot trigger another request.
    for hop in 0..=4 {
        let remaining = checkpoint(cancel, deadline)?;
        checked_uri(&current, hop == 0)?;
        let response = transport.request(&current, remaining, metadata)?;
        checkpoint(cancel, deadline)?;
        if response.status == 200 {
            return Ok(response.body);
        }
        if !matches!(response.status, 301 | 302 | 303 | 307 | 308) {
            return Err(DiscoveryError::Http(response.status));
        }
        if metadata || hop == 4 {
            return Err(DiscoveryError::Limit);
        }
        let next = response.location.ok_or(DiscoveryError::Malformed)?;
        // Only absolute HTTPS delivery URLs. Relative/ambiguous locations fail closed.
        checked_uri(&next, false)?;
        current = next;
        // Redirect bodies are never read; agent has no retained idle connections.
    }
    Err(DiscoveryError::Limit)
}
impl Transport for OfficialHttps {
    fn open(
        &mut self,
        url: &str,
        timeout: Duration,
        metadata: bool,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Box<dyn Read>, DiscoveryError> {
        follow_checked(&mut UreqHop, url, timeout, metadata, cancel)
    }
}
/// Source evidence only. Callers are sibling policy code, not public arbitrary URL APIs.
pub(super) fn read_ytdlp_source(
    url: &str,
    limit: usize,
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
) -> Result<Vec<u8>, DiscoveryError> {
    let uri: ureq::http::Uri = url.parse().map_err(|_| DiscoveryError::Malformed)?;
    let source = uri.host() == Some("raw.githubusercontent.com")
        && uri.path().starts_with("/yt-dlp/yt-dlp/");
    let api =
        uri.host() == Some("api.github.com") && uri.path().starts_with("/repos/yt-dlp/yt-dlp/git/");
    if !(source || api)
        || uri.scheme_str() != Some("https")
        || uri.query().is_some()
        || url.len() > 8192
        || url.contains(['#', '\\', '\r', '\n'])
        || uri.authority().is_none_or(|a| {
            a.as_str() != uri.host().unwrap_or("")
                && a.as_str() != format!("{}:443", uri.host().unwrap_or(""))
        })
        || limit == 0
        || limit > 1024 * 1024
    {
        return Err(DiscoveryError::Malformed);
    }
    let remaining = checkpoint(cancel, deadline)?;
    let hop = UreqHop.request(url, remaining, api)?;
    checkpoint(cancel, deadline)?;
    // No redirects for immutable source bytes or git references.
    if hop.status != 200 {
        return Err(DiscoveryError::Http(hop.status));
    }
    bounded_read(hop.body, limit, cancel, deadline)
}
fn bounded_read(
    mut reader: Box<dyn Read>,
    limit: usize,
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
) -> Result<Vec<u8>, DiscoveryError> {
    let mut out = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        checkpoint(cancel, deadline)?;
        let n = reader.read(&mut chunk)?;
        checkpoint(cancel, deadline)?;
        if n == 0 {
            break;
        }
        if out.len() + n > limit {
            return Err(DiscoveryError::Limit);
        }
        out.extend_from_slice(&chunk[..n]);
    }
    Ok(out)
}
pub fn discover_latest(
    tool: Tool,
    cancel: &Arc<AtomicBool>,
) -> Result<Observation, DiscoveryError> {
    discover_with(tool, cancel, &mut OfficialHttps).map(|(o, _)| o)
}
fn discover_with(
    tool: Tool,
    cancel: &Arc<AtomicBool>,
    transport: &mut dyn Transport,
) -> Result<(Observation, Option<String>), DiscoveryError> {
    let deadline = Instant::now()
        .checked_add(NETWORK_BUDGET)
        .ok_or(DiscoveryError::TimedOut)?;
    let url = tool.latest_url();
    let reader = transport.open(&url, checkpoint(cancel, deadline)?, true, cancel)?;
    let raw = bounded_read(reader, META_LIMIT, cancel, deadline)?;
    let o = parse(tool, &raw)?;
    let release: Release = serde_json::from_slice(&raw).map_err(|_| DiscoveryError::Malformed)?;
    let digest = release
        .assets
        .into_iter()
        .find(|a| a.id == o.asset_id)
        .and_then(|a| a.digest);
    Ok((o, digest))
}
/// User-invoked capture, not auto install: exact public release body+checksum are receipt-pinned.
pub fn capture_latest(tool: Tool, cancel: &Arc<AtomicBool>) -> Result<CapturedPin, DiscoveryError> {
    capture_at(tool, None, cancel, &mut OfficialHttps)
}
pub fn capture_version(
    tool: Tool,
    version: &str,
    cancel: &Arc<AtomicBool>,
) -> Result<CapturedPin, DiscoveryError> {
    let tag = if tool == Tool::Deno && !version.starts_with('v') {
        format!("v{version}")
    } else {
        version.to_owned()
    };
    if !token(&tag) {
        return Err(DiscoveryError::Malformed);
    }
    capture_at(tool, Some(&tag), cancel, &mut OfficialHttps)
}
#[cfg(test)]
pub(super) fn capture_with(
    tool: Tool,
    cancel: &Arc<AtomicBool>,
    transport: &mut dyn Transport,
) -> Result<CapturedPin, DiscoveryError> {
    capture_at(tool, None, cancel, transport)
}
fn capture_at(
    tool: Tool,
    requested_tag: Option<&str>,
    cancel: &Arc<AtomicBool>,
    transport: &mut dyn Transport,
) -> Result<CapturedPin, DiscoveryError> {
    let deadline = Instant::now()
        .checked_add(NETWORK_BUDGET)
        .ok_or(DiscoveryError::TimedOut)?;
    let metadata_url = match requested_tag {
        Some(tag) => format!(
            "https://api.github.com/repos/{}/releases/tags/{tag}",
            tool.repository()
        ),
        None => tool.latest_url(),
    };
    let reader = transport.open(&metadata_url, checkpoint(cancel, deadline)?, true, cancel)?;
    let meta = bounded_read(reader, META_LIMIT, cancel, deadline)?;
    let observation = parse(tool, &meta)?;
    if requested_tag.is_some_and(|tag| tag != observation.tag) {
        return Err(DiscoveryError::Malformed);
    }
    let release: Release = serde_json::from_slice(&meta).map_err(|_| DiscoveryError::Malformed)?;
    let checksum_name = tool.checksum(&observation.asset_name);
    let checksum_url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        tool.repository(),
        observation.tag,
        checksum_name
    );
    let checksum_asset = release
        .assets
        .iter()
        .find(|a| {
            a.name == checksum_name
                && a.browser_download_url == checksum_url
                && a.size > 0
                && a.size <= CHECKSUM_LIMIT as u64
        })
        .ok_or(DiscoveryError::MissingChecksum)?;
    let reader = transport.open(&checksum_url, checkpoint(cancel, deadline)?, false, cancel)?;
    let raw = bounded_read(reader, CHECKSUM_LIMIT, cancel, deadline)?;
    if raw.len() as u64 != checksum_asset.size {
        return Err(DiscoveryError::ChecksumMismatch);
    }
    let artifact_sha256 = checksum(&raw, &observation.asset_name)?;
    let digest = release
        .assets
        .iter()
        .find(|a| a.id == observation.asset_id)
        .and_then(|a| a.digest.as_ref());
    if digest.is_some_and(|d| {
        d.strip_prefix("sha256:")
            .is_none_or(|h| !h.eq_ignore_ascii_case(&artifact_sha256))
    }) {
        return Err(DiscoveryError::ChecksumMismatch);
    }
    checkpoint(cancel, deadline)?;
    Ok(CapturedPin {
        observation,
        checksum_url,
        checksum_body_sha256: sha(&raw),
        artifact_sha256,
        metadata_digest_matched: digest.is_some(),
        publisher_trust_verified: false,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    fn metadata() -> Vec<u8> {
        serde_json::json!({"id":17,"tag_name":"2026.08.19","draft":false,"prerelease":false,"published_at":"2026-08-19T00:00:00Z","assets":[{"id":1,"name":"yt-dlp.exe","size":1024,"browser_download_url":"https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/yt-dlp.exe","digest":format!("sha256:{}","a".repeat(64))},{"id":2,"name":"SHA2-256SUMS","size":77,"browser_download_url":"https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/SHA2-256SUMS","digest":null}]}).to_string().into_bytes()
    }
    struct Fake(Vec<Vec<u8>>);
    impl Transport for Fake {
        fn open(
            &mut self,
            _: &str,
            _: Duration,
            _: bool,
            _: &Arc<AtomicBool>,
        ) -> Result<Box<dyn Read>, DiscoveryError> {
            Ok(Box::new(Cursor::new(self.0.remove(0))))
        }
    }
    #[test]
    fn same_origin_capture_does_not_establish_publisher_trust() {
        let meta = metadata();
        let sums = format!("{}  yt-dlp.exe\n", "a".repeat(64)).into_bytes();
        let pin = capture_with(
            Tool::YtDlp,
            &Arc::new(AtomicBool::new(false)),
            &mut Fake(vec![meta, sums]),
        )
        .unwrap();
        assert_eq!(pin.artifact_sha256(), "a".repeat(64));
        assert!(!pin.publisher_trust_verified);
        assert!(pin.metadata_digest_matched);
    }
    #[test]
    fn release_duplicate_missing_asset_foreign_url_prerelease_and_bounds_fail() {
        for mode in ["foreign", "draft", "asset", "id"] {
            let mut v: serde_json::Value = serde_json::from_slice(&metadata()).unwrap();
            match mode {
                "foreign" => {
                    v["assets"][0]["browser_download_url"] = "https://evil.invalid/tool.exe".into()
                }
                "draft" => v["draft"] = true.into(),
                "asset" => v["assets"] = serde_json::json!([]),
                _ => v["id"] = 0.into(),
            };
            assert!(parse(Tool::YtDlp, v.to_string().as_bytes()).is_err());
        }
        assert!(parse(Tool::YtDlp, &vec![b'x'; META_LIMIT + 1]).is_err());
        let duplicate =
            String::from_utf8(metadata())
                .unwrap()
                .replacen("\"id\":17", "\"id\":17,\"id\":18", 1);
        assert!(parse(Tool::YtDlp, duplicate.as_bytes()).is_err());
    }
    #[test]
    fn checksum_duplicate_mismatch_traversal_and_wrong_filename_rejected() {
        for sums in [
            format!("{} ../yt-dlp.exe\n", "a".repeat(64)),
            format!("{} another.exe\n", "a".repeat(64)),
            format!(
                "{} yt-dlp.exe\n{} yt-dlp.exe\n",
                "a".repeat(64),
                "a".repeat(64)
            ),
        ] {
            assert!(checksum(sums.as_bytes(), "yt-dlp.exe").is_err());
        }
        let sums = format!("{}  yt-dlp.exe\n", "b".repeat(64));
        assert!(matches!(
            capture_with(
                Tool::YtDlp,
                &Arc::new(AtomicBool::new(false)),
                &mut Fake(vec![metadata(), sums.into_bytes()])
            ),
            Err(DiscoveryError::ChecksumMismatch)
        ));
    }
    #[test]
    fn pre_cancel_never_opens_transport() {
        struct Never;
        impl Transport for Never {
            fn open(
                &mut self,
                _: &str,
                _: Duration,
                _: bool,
                _: &Arc<AtomicBool>,
            ) -> Result<Box<dyn Read>, DiscoveryError> {
                panic!("cancel must precede network")
            }
        }
        assert!(matches!(
            capture_with(Tool::YtDlp, &Arc::new(AtomicBool::new(true)), &mut Never),
            Err(DiscoveryError::Cancelled)
        ));
    }
    struct Hops {
        replies: std::collections::VecDeque<Result<Hop, DiscoveryError>>,
        requested: Vec<String>,
        cancel_on_request: Option<Arc<AtomicBool>>,
    }
    impl HopTransport for Hops {
        fn request(&mut self, url: &str, _: Duration, _: bool) -> Result<Hop, DiscoveryError> {
            self.requested.push(url.into());
            if let Some(c) = &self.cancel_on_request {
                c.store(true, Ordering::Release);
            }
            self.replies
                .pop_front()
                .expect("unexpected network request")
        }
    }
    struct UnreadRedirect;
    impl Read for UnreadRedirect {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            panic!("redirect body must not be read")
        }
    }
    fn redirect(url: &str) -> Result<Hop, DiscoveryError> {
        Ok(Hop {
            status: 302,
            location: Some(url.into()),
            body: Box::new(UnreadRedirect),
        })
    }
    fn success() -> Result<Hop, DiscoveryError> {
        Ok(Hop {
            status: 200,
            location: None,
            body: Box::new(Cursor::new(b"ok".to_vec())),
        })
    }
    fn hops(replies: Vec<Result<Hop, DiscoveryError>>) -> Hops {
        Hops {
            replies: replies.into(),
            requested: vec![],
            cancel_on_request: None,
        }
    }
    const CHECKSUM_URL: &str =
        "https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/SHA2-256SUMS";
    #[test]
    fn forbidden_redirect_authorities_never_receive_a_request() {
        for target in [
            "http://github.com/a",
            "https://evil.invalid/a",
            "https://user:password@github.com/a",
            "https://github.com:444/a",
            "https://github.com:/a",
            "https://github.com:0443/a",
            "https://github.com:99999/a",
            "https://%67ithub.com/a",
            "//github.com/a",
            "/relative",
            "https://github.com/a#fragment",
        ] {
            let mut t = hops(vec![redirect(target)]);
            assert!(matches!(
                follow_checked(
                    &mut t,
                    CHECKSUM_URL,
                    NETWORK_BUDGET,
                    false,
                    &Arc::new(AtomicBool::new(false))
                ),
                Err(DiscoveryError::Malformed)
            ));
            assert_eq!(t.requested, vec![CHECKSUM_URL]);
        }
        let mut t = hops(vec![]);
        assert!(
            follow_checked(
                &mut t,
                "https://github.com:80/a",
                NETWORK_BUDGET,
                false,
                &Arc::new(AtomicBool::new(false))
            )
            .is_err()
        );
        assert!(t.requested.is_empty());
    }
    #[test]
    fn four_delivery_hops_allowed_fifth_blocked_and_metadata_never_redirects() {
        let target = "https://release-assets.githubusercontent.com:443/asset?signature=opaque";
        let mut t = hops(vec![
            redirect(target),
            redirect(target),
            redirect(target),
            redirect(target),
            success(),
        ]);
        let mut body = follow_checked(
            &mut t,
            CHECKSUM_URL,
            NETWORK_BUDGET,
            false,
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        let mut bytes = vec![];
        body.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"ok");
        assert_eq!(t.requested.len(), 5);
        let mut t = hops((0..5).map(|_| redirect(target)).collect());
        assert!(matches!(
            follow_checked(
                &mut t,
                CHECKSUM_URL,
                NETWORK_BUDGET,
                false,
                &Arc::new(AtomicBool::new(false))
            ),
            Err(DiscoveryError::Limit)
        ));
        assert_eq!(t.requested.len(), 5);
        let mut t = hops(vec![redirect(target)]);
        assert!(matches!(
            follow_checked(
                &mut t,
                "https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest",
                NETWORK_BUDGET,
                true,
                &Arc::new(AtomicBool::new(false))
            ),
            Err(DiscoveryError::Limit)
        ));
        assert_eq!(t.requested.len(), 1);
    }
    #[test]
    fn provider_timeout_failure_and_cancel_after_headers_remain_distinct() {
        for error in [
            DiscoveryError::TimedOut,
            DiscoveryError::Transport,
            DiscoveryError::Http(503),
        ] {
            let expected = std::mem::discriminant(&error);
            let mut t = hops(vec![Err(error)]);
            let err = follow_checked(
                &mut t,
                CHECKSUM_URL,
                NETWORK_BUDGET,
                false,
                &Arc::new(AtomicBool::new(false)),
            )
            .err()
            .unwrap();
            assert_eq!(std::mem::discriminant(&err), expected);
            assert_eq!(t.requested.len(), 1);
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let mut t = hops(vec![success()]);
        t.cancel_on_request = Some(cancel.clone());
        assert!(matches!(
            follow_checked(&mut t, CHECKSUM_URL, NETWORK_BUDGET, false, &cancel),
            Err(DiscoveryError::Cancelled)
        ));
        let mut t = hops(vec![]);
        assert!(matches!(
            follow_checked(
                &mut t,
                CHECKSUM_URL,
                Duration::ZERO,
                false,
                &Arc::new(AtomicBool::new(false))
            ),
            Err(DiscoveryError::TimedOut)
        ));
        assert!(t.requested.is_empty());
    }
    #[test]
    fn cancelled_body_stops_after_one_bounded_chunk_and_limits_fail_closed() {
        struct Cancelling {
            cancel: Arc<AtomicBool>,
            reads: Arc<std::sync::atomic::AtomicUsize>,
        }
        impl Read for Cancelling {
            fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
                assert!(out.len() <= 4096);
                self.reads.fetch_add(1, Ordering::Relaxed);
                out.fill(b'x');
                self.cancel.store(true, Ordering::Release);
                Ok(out.len())
            }
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let reads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        assert!(matches!(
            bounded_read(
                Box::new(Cancelling {
                    cancel: cancel.clone(),
                    reads: reads.clone()
                }),
                META_LIMIT,
                &cancel,
                Instant::now() + NETWORK_BUDGET
            ),
            Err(DiscoveryError::Cancelled)
        ));
        assert_eq!(reads.load(Ordering::Relaxed), 1);
        assert!(matches!(
            bounded_read(
                Box::new(Cursor::new(vec![b'x'; CHECKSUM_LIMIT + 1])),
                CHECKSUM_LIMIT,
                &Arc::new(AtomicBool::new(false)),
                Instant::now() + NETWORK_BUDGET
            ),
            Err(DiscoveryError::Limit)
        ));
    }
}
