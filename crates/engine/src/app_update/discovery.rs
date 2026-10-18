//! Bounded public GitHub release observations. No signature or execution authority.
use super::Channel;
use serde::Deserialize;
use std::{
    io::Read,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
const MAX_BODY: usize = 256 * 1024;
const MAX_RELEASES: usize = 20;
const MAX_NOTES_BYTES: usize = 16 * 1024;
const MAX_ASSETS: usize = 64;
const BUDGET: Duration = Duration::from_secs(20);
pub const RELEASES_URL: &str = "https://github.com/joongi007/dlpad/releases";
const API: &str = "https://api.github.com/repos/joongi007/dlpad/releases";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscoveryError {
    Cancelled,
    TimedOut,
    Offline,
    RateLimited,
    NoRelease,
    InvalidResponse,
    Limit,
}
impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "app release observation failed ({self:?})")
    }
}
impl std::error::Error for DiscoveryError {}
#[derive(Clone, Debug)]
pub struct ReleaseObservation {
    version: String,
    url: String,
    newer: bool,
    channel: Channel,
    notes: String,
    notes_truncated: bool,
    signed_metadata_available: bool,
}
impl ReleaseObservation {
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn is_newer(&self) -> bool {
        self.newer
    }
    pub fn channel(&self) -> Channel {
        self.channel
    }
    /// Untrusted plain release text, never parsed as markup or an execution policy.
    pub fn notes(&self) -> &str {
        &self.notes
    }
    pub fn notes_truncated(&self) -> bool {
        self.notes_truncated
    }
    /// Exact public assets were observed. This does not verify a signature or grant authority.
    pub fn signed_metadata_available(&self) -> bool {
        self.signed_metadata_available
    }
}
#[derive(Deserialize)]
struct Release {
    id: u64,
    tag_name: String,
    draft: bool,
    prerelease: bool,
    html_url: String,
    published_at: Option<String>,
    body: Option<String>,
    #[serde(default)]
    assets: Vec<Asset>,
}
#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    size: u64,
    state: String,
}
fn signed_assets(assets: &[Asset], release_version: &str) -> bool {
    let base = format!("https://github.com/joongi007/dlpad/releases/download/v{release_version}");
    [
        ("update-windows-x86_64.json", 1, 64 * 1024),
        ("update-windows-x86_64.json.sig", 128, 129),
    ]
    .into_iter()
    .all(|(name, minimum, maximum)| {
        let mut matched = assets.iter().filter(|asset| asset.name == name);
        let Some(asset) = matched.next() else {
            return false;
        };
        matched.next().is_none()
            && asset.state == "uploaded"
            && (minimum..=maximum).contains(&asset.size)
            && asset.browser_download_url == format!("{base}/{name}")
    })
}
fn version(value: &str) -> Option<[u32; 3]> {
    if value.len() > 32 {
        return None;
    }
    let mut parts = value.split('.');
    let mut parsed = [0u32; 3];
    for n in &mut parsed {
        let s = parts.next()?;
        if s.is_empty()
            || s.len() > 9
            || (s.len() > 1 && s.starts_with('0'))
            || !s.bytes().all(|v| v.is_ascii_digit())
        {
            return None;
        }
        *n = s.parse().ok()?;
    }
    parts.next().is_none().then_some(parsed)
}
fn checkpoint(cancel: &AtomicBool, deadline: Instant) -> Result<Duration, DiscoveryError> {
    if cancel.load(Ordering::Acquire) {
        return Err(DiscoveryError::Cancelled);
    }
    deadline
        .checked_duration_since(Instant::now())
        .filter(|d| !d.is_zero())
        .ok_or(DiscoveryError::TimedOut)
}
fn observe(
    release: Release,
    channel: Channel,
    installed: [u32; 3],
) -> Result<ReleaseObservation, DiscoveryError> {
    let value = release
        .tag_name
        .strip_prefix('v')
        .ok_or(DiscoveryError::InvalidResponse)?;
    let number = version(value).ok_or(DiscoveryError::InvalidResponse)?;
    let expected = format!("{RELEASES_URL}/tag/v{value}");
    if release.id == 0
        || release.draft
        || release.prerelease != (channel == Channel::Preview)
        || release.html_url != expected
        || release.published_at.as_ref().is_none_or(|s| {
            s.is_empty() || s.len() > 64 || !s.bytes().all(|v| v.is_ascii_graphic())
        })
    {
        return Err(DiscoveryError::InvalidResponse);
    }
    if release.assets.len() > MAX_ASSETS {
        return Err(DiscoveryError::Limit);
    }
    let signed_metadata_available = signed_assets(&release.assets, value);
    let mut notes = release.body.unwrap_or_default();
    let notes_truncated = notes.len() > MAX_NOTES_BYTES;
    if notes_truncated {
        let mut end = MAX_NOTES_BYTES;
        while !notes.is_char_boundary(end) {
            end -= 1;
        }
        notes.truncate(end);
    }
    Ok(ReleaseObservation {
        version: value.to_owned(),
        url: expected,
        newer: number > installed,
        channel,
        notes,
        notes_truncated,
        signed_metadata_available,
    })
}
fn parse(
    body: &[u8],
    channel: Channel,
    installed_version: &str,
) -> Result<ReleaseObservation, DiscoveryError> {
    if body.len() > MAX_BODY {
        return Err(DiscoveryError::Limit);
    }
    let installed = version(installed_version).ok_or(DiscoveryError::InvalidResponse)?;
    match channel {
        Channel::Stable => observe(
            serde_json::from_slice(body).map_err(|_| DiscoveryError::InvalidResponse)?,
            channel,
            installed,
        ),
        Channel::Preview => {
            let releases: Vec<Release> =
                serde_json::from_slice(body).map_err(|_| DiscoveryError::InvalidResponse)?;
            if releases.len() > MAX_RELEASES {
                return Err(DiscoveryError::Limit);
            }
            let mut selected: Option<ReleaseObservation> = None;
            for r in releases.into_iter().filter(|r| !r.draft && r.prerelease) {
                let observed = observe(r, channel, installed)?;
                if selected
                    .as_ref()
                    .is_none_or(|old| version(observed.version()) > version(old.version()))
                {
                    selected = Some(observed);
                }
            }
            selected.ok_or(DiscoveryError::NoRelease)
        }
    }
}

/// No caller URL, credential, cookie, external process or file mutation arguments.
/// Preview examines at most the first20 public releases; absence is not global history proof.
pub fn discover_release(
    channel: Channel,
    installed_version: &str,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<ReleaseObservation, DiscoveryError> {
    let deadline = deadline.min(Instant::now() + BUDGET);
    let timeout = checkpoint(cancel, deadline)?;
    version(installed_version).ok_or(DiscoveryError::InvalidResponse)?;
    let url = match channel {
        Channel::Stable => format!("{API}/latest"),
        Channel::Preview => format!("{API}?per_page=20&page=1"),
    };
    let config = ureq::Agent::config_builder()
        .https_only(true)
        .proxy(None)
        .max_redirects(0)
        .http_status_as_error(false)
        .max_response_header_size(64 * 1024)
        .input_buffer_size(16 * 1024)
        .output_buffer_size(4096)
        .max_idle_connections(0)
        .timeout_global(Some(timeout))
        .timeout_resolve(Some(timeout.min(Duration::from_secs(5))))
        .timeout_connect(Some(timeout.min(Duration::from_secs(5))))
        .timeout_recv_response(Some(timeout.min(Duration::from_secs(5))))
        .timeout_recv_body(Some(timeout.min(Duration::from_secs(5))))
        .build();
    let response = ureq::Agent::new_with_config(config)
        .get(&url)
        .header("User-Agent", "dlpad-app-release-observer")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2026-03-10")
        .call()
        .map_err(|error| {
            if cancel.load(Ordering::Acquire) {
                DiscoveryError::Cancelled
            } else if matches!(error, ureq::Error::Timeout(_)) || Instant::now() >= deadline {
                DiscoveryError::TimedOut
            } else {
                DiscoveryError::Offline
            }
        })?;
    checkpoint(cancel, deadline)?;
    match response.status().as_u16() {
        200 => (),
        404 => return Err(DiscoveryError::NoRelease),
        403 | 429 => return Err(DiscoveryError::RateLimited),
        _ => return Err(DiscoveryError::Offline),
    }
    let mut reader = response.into_body().into_reader();
    let mut body = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        checkpoint(cancel, deadline)?;
        let remaining = (MAX_BODY + 1 - body.len()).min(chunk.len());
        let count = reader.read(&mut chunk[..remaining]).map_err(|error| {
            if cancel.load(Ordering::Acquire) {
                DiscoveryError::Cancelled
            } else if error.kind() == std::io::ErrorKind::TimedOut || Instant::now() >= deadline {
                DiscoveryError::TimedOut
            } else {
                DiscoveryError::Offline
            }
        })?;
        checkpoint(cancel, deadline)?;
        if count == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..count]);
        if body.len() > MAX_BODY {
            return Err(DiscoveryError::Limit);
        }
    }
    parse(&body, channel, installed_version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn release(v: &str, preview: bool) -> serde_json::Value {
        json!({"id":1,"tag_name":format!("v{v}"),"draft":false,"prerelease":preview,"html_url":format!("{RELEASES_URL}/tag/v{v}"),"published_at":"2026-10-18T00:00:00Z"})
    }
    #[test]
    fn stable_compares_numeric_version_and_canonical_release_only() {
        let r = parse(
            &serde_json::to_vec(&release("0.10.0", false)).unwrap(),
            Channel::Stable,
            "0.9.0",
        )
        .unwrap();
        assert!(r.is_newer());
        assert_eq!(r.version(), "0.10.0");
        let r = parse(
            &serde_json::to_vec(&release("0.1.0", false)).unwrap(),
            Channel::Stable,
            "0.2.0",
        )
        .unwrap();
        assert!(!r.is_newer());
        for field in ["html_url", "tag_name"] {
            let mut r = release("0.2.0", false);
            r[field] = json!("https://attacker.invalid/");
            assert_eq!(
                parse(&serde_json::to_vec(&r).unwrap(), Channel::Stable, "0.1.0").unwrap_err(),
                DiscoveryError::InvalidResponse
            );
        }
    }
    #[test]
    fn preview_selects_highest_supported_numeric_version_in_bounded_public_page() {
        let page = json!([
            release("0.2.0", true),
            release("0.9.0", false),
            release("0.3.0", true)
        ]);
        let r = parse(
            &serde_json::to_vec(&page).unwrap(),
            Channel::Preview,
            "0.1.0",
        )
        .unwrap();
        assert_eq!(r.version(), "0.3.0");
        assert_eq!(
            parse(b"[]", Channel::Preview, "0.1.0").unwrap_err(),
            DiscoveryError::NoRelease
        );
        let list = vec![release("0.2.0", true); 21];
        assert_eq!(
            parse(
                &serde_json::to_vec(&list).unwrap(),
                Channel::Preview,
                "0.1.0"
            )
            .unwrap_err(),
            DiscoveryError::Limit
        );
    }
    #[test]
    fn malformed_duplicate_draft_and_overlong_never_become_observations() {
        for b in [b"{".as_slice(), b"{\"id\":1,\"id\":2}"] {
            assert!(parse(b, Channel::Stable, "0.1.0").is_err());
        }
        let mut r = release("0.2.0", false);
        r["draft"] = json!(true);
        assert!(parse(&serde_json::to_vec(&r).unwrap(), Channel::Stable, "0.1.0").is_err());
        assert_eq!(
            parse(&vec![b' '; MAX_BODY + 1], Channel::Stable, "0.1.0").unwrap_err(),
            DiscoveryError::Limit
        );
    }
    fn observe_fixture(value: serde_json::Value) -> Result<ReleaseObservation, DiscoveryError> {
        parse(
            &serde_json::to_vec(&value).unwrap(),
            Channel::Stable,
            "0.1.0",
        )
    }
    fn asset(name: &str, size: u64) -> serde_json::Value {
        json!({"name":name,"size":size,"state":"uploaded","browser_download_url":format!("https://github.com/joongi007/dlpad/releases/download/v0.2.0/{name}")})
    }
    fn signed_release() -> serde_json::Value {
        let mut value = release("0.2.0", false);
        value["assets"] = json!([
            asset("update-windows-x86_64.json", 1024),
            asset("update-windows-x86_64.json.sig", 129),
        ]);
        value
    }
    #[test]
    fn notes_are_plain_bounded_text_with_utf8_safe_truncation() {
        let mut value = release("0.2.0", false);
        let notes = format!("{}한글🙂", "x".repeat(MAX_NOTES_BYTES - 1));
        value["body"] = json!(notes);
        let observed = observe_fixture(value).unwrap();
        assert_eq!(observed.notes(), "x".repeat(MAX_NOTES_BYTES - 1));
        assert!(observed.notes_truncated());
        let mut value = release("0.2.0", false);
        value["body"] = json!("<script>fixture</script> [link](https://invalid.test) 한글");
        let observed = observe_fixture(value).unwrap();
        assert_eq!(
            observed.notes(),
            "<script>fixture</script> [link](https://invalid.test) 한글"
        );
        assert!(!observed.notes_truncated());
        assert!(
            observe_fixture(release("0.2.0", false))
                .unwrap()
                .notes()
                .is_empty()
        );
    }
    #[test]
    fn only_the_exact_uploaded_metadata_and_signature_pair_is_observed() {
        assert!(
            observe_fixture(signed_release())
                .unwrap()
                .signed_metadata_available()
        );
        assert!(
            !observe_fixture(release("0.2.0", false))
                .unwrap()
                .signed_metadata_available()
        );
        let mut value = signed_release();
        value["assets"][1]["size"] = json!(128);
        assert!(observe_fixture(value).unwrap().signed_metadata_available());
        let mut value = signed_release();
        value["assets"].as_array_mut().unwrap().pop();
        assert!(!observe_fixture(value).unwrap().signed_metadata_available());
    }
    #[test]
    fn aliases_duplicate_foreign_version_url_and_invalid_size_do_not_advertise_support() {
        for (field, bad) in [
            ("name", json!("UPDATE-windows-x86_64.json")),
            ("state", json!("starter")),
            ("size", json!(0)),
            ("size", json!(64 * 1024 + 1)),
            (
                "browser_download_url",
                json!("https://attacker.invalid/update-windows-x86_64.json"),
            ),
            (
                "browser_download_url",
                json!(
                    "https://github.com/joongi007/dlpad/releases/download/v0.3.0/update-windows-x86_64.json"
                ),
            ),
        ] {
            let mut value = signed_release();
            value["assets"][0][field] = bad;
            assert!(
                !observe_fixture(value).unwrap().signed_metadata_available(),
                "{field}"
            );
        }
        let mut value = signed_release();
        value["assets"][1]["size"] = json!(130);
        assert!(!observe_fixture(value).unwrap().signed_metadata_available());
        let mut value = signed_release();
        value["assets"]
            .as_array_mut()
            .unwrap()
            .push(asset("update-windows-x86_64.json", 100));
        assert!(!observe_fixture(value).unwrap().signed_metadata_available());
    }
    #[test]
    fn asset_count_is_bounded_and_preview_carries_only_selected_release_notes() {
        let mut value = release("0.2.0", false);
        value["assets"] = json!(vec![asset("unrelated", 1); MAX_ASSETS + 1]);
        assert_eq!(observe_fixture(value).unwrap_err(), DiscoveryError::Limit);
        let mut old = release("0.2.0", true);
        old["body"] = json!("old notes");
        let mut new = release("0.3.0", true);
        new["body"] = json!("new notes");
        let observed = parse(
            &serde_json::to_vec(&json!([old, new])).unwrap(),
            Channel::Preview,
            "0.1.0",
        )
        .unwrap();
        assert_eq!(observed.notes(), "new notes");
        assert_eq!(observed.version(), "0.3.0");
        assert!(!observed.signed_metadata_available());
    }
    #[test]
    fn stop_and_expired_deadline_prevent_network() {
        assert_eq!(
            discover_release(
                Channel::Stable,
                "invalid version",
                &AtomicBool::new(false),
                Instant::now() + BUDGET
            )
            .unwrap_err(),
            DiscoveryError::InvalidResponse
        );
        assert_eq!(
            discover_release(
                Channel::Stable,
                "0.1.0",
                &AtomicBool::new(true),
                Instant::now() + BUDGET
            )
            .unwrap_err(),
            DiscoveryError::Cancelled
        );
        assert_eq!(
            discover_release(
                Channel::Stable,
                "0.1.0",
                &AtomicBool::new(false),
                Instant::now()
            )
            .unwrap_err(),
            DiscoveryError::TimedOut
        );
    }
}
