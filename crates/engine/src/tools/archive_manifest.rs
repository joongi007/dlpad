//! Bounded, explicit package contract; source catalog trust is a separate gate.
use super::{ArchiveLayout, MANIFEST_LIMIT, Ownership, ToolError, identifier};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_EXPANDED_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_TAR_INSTALLED_BYTES: u64 = 384 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveComponentRole {
    Executable,
    Auxiliary,
    Notice,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveComponent {
    pub name: String,
    pub bytes: u64,
    pub sha256: String,
    pub role: ArchiveComponentRole,
}

/// Exact revision HTTPS evidence installed as a Notice, never treated as an archive member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinnedArchiveNotice {
    pub component: String,
    pub source_url: String,
    pub revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchivePackage {
    pub version: u8,
    pub tool: String,
    pub tool_version: String,
    pub target: String,
    pub source_url: String,
    pub license: String,
    pub ownership: Ownership,
    pub archive_bytes: u64,
    pub archive_sha256: String,
    pub components: Vec<ArchiveComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<ArchiveLayout>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notices: Vec<PinnedArchiveNotice>,
}

fn digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

pub(super) fn component_name(value: &str) -> bool {
    if !identifier(value) || value.ends_with('.') {
        return false;
    }
    let lower = value.to_ascii_lowercase();
    let stem = lower.split('.').next().unwrap_or_default();
    if matches!(stem, "con" | "prn" | "aux" | "nul") {
        return false;
    }
    if stem.len() == 4
        && (stem.starts_with("com") || stem.starts_with("lpt"))
        && matches!(stem.as_bytes()[3], b'1'..=b'9')
    {
        return false;
    }
    !matches!(
        lower.as_str(),
        "manifest.json" | "archive-package.json" | ".dlpad-owner"
    )
}

impl ArchivePackage {
    /// Only components physically supplied by the archive; external notices are separate capabilities.
    pub fn archive_components(&self) -> impl Iterator<Item = &ArchiveComponent> {
        self.components.iter().filter(|component| {
            !self
                .notices
                .iter()
                .any(|notice| notice.component == component.name)
        })
    }

    /// Flat member names deliberately match a small immutable supply allowlist.
    pub fn validate(&self) -> Result<(), ToolError> {
        if self.ownership != Ownership::Managed {
            return Err(ToolError::ExternalOwnership);
        }
        let expanded_limit = match (self.version, &self.layout) {
            (1, None) if self.notices.is_empty() => MAX_EXPANDED_BYTES,
            (3, None) => MAX_EXPANDED_BYTES,
            (2, Some(layout)) if self.notices.is_empty() => {
                layout.validate(&self.components)?;
                MAX_TAR_INSTALLED_BYTES
            }
            _ => return Err(ToolError::InvalidManifest),
        };
        if ![1, 2, 3].contains(&self.version)
            || ![&self.tool, &self.tool_version, &self.target]
                .into_iter()
                .all(|v| identifier(v))
            || self.source_url.len() > 8192
            || self.source_url.is_empty()
            || self
                .source_url
                .chars()
                .any(|c| c.is_control() || c.is_whitespace())
            || self.license.is_empty()
            || self.license.len() > 4096
            || self.license.chars().any(|c| c.is_control())
            || !digest(&self.archive_sha256)
            || self.archive_bytes == 0
            || self.archive_bytes > MAX_ARCHIVE_BYTES
            || self.components.is_empty()
            || self.components.len() > 8
        {
            return Err(ToolError::InvalidManifest);
        }
        let uri: ureq::http::Uri = self
            .source_url
            .parse()
            .map_err(|_| ToolError::InvalidManifest)?;
        if uri.scheme_str() != Some("https")
            || self.source_url.contains(['?', '#'])
            || uri
                .authority()
                .is_none_or(|a| a.host().is_empty() || a.as_str().contains(['@', '%']))
        {
            return Err(ToolError::InvalidManifest);
        }
        let mut names = BTreeSet::new();
        let mut total = 0u64;
        let mut executables = 0;
        for component in &self.components {
            if !component_name(&component.name)
                || !names.insert(component.name.to_ascii_lowercase())
                || !digest(&component.sha256)
                || component.bytes == 0
            {
                return Err(ToolError::InvalidManifest);
            }
            total = total
                .checked_add(component.bytes)
                .ok_or(ToolError::DownloadLimit)?;
            if total > expanded_limit {
                return Err(ToolError::DownloadLimit);
            }
            executables += usize::from(component.role == ArchiveComponentRole::Executable);
        }
        if self.notices.len() > 2 {
            return Err(ToolError::InvalidManifest);
        }
        let mut notice_names = BTreeSet::new();
        let mut notice_bytes = 0u64;
        for notice in &self.notices {
            let component = self
                .components
                .iter()
                .find(|c| c.name == notice.component)
                .ok_or(ToolError::InvalidManifest)?;
            if component.role != ArchiveComponentRole::Notice
                || component.bytes > 1024 * 1024
                || !notice_names.insert(&notice.component)
                || notice.revision.len() != 40
                || !notice.revision.bytes().all(|b| b.is_ascii_hexdigit())
                || notice.source_url.is_empty()
                || notice.source_url.len() > 8192
                || notice
                    .source_url
                    .chars()
                    .any(|c| c.is_control() || c.is_whitespace())
                || notice.source_url.contains(['?', '#'])
            {
                return Err(ToolError::InvalidManifest);
            }
            let uri: ureq::http::Uri = notice
                .source_url
                .parse()
                .map_err(|_| ToolError::InvalidManifest)?;
            if uri.scheme_str() != Some("https")
                || uri
                    .authority()
                    .is_none_or(|a| a.host().is_empty() || a.as_str().contains(['@', '%']))
            {
                return Err(ToolError::InvalidManifest);
            }
            notice_bytes = notice_bytes
                .checked_add(component.bytes)
                .ok_or(ToolError::DownloadLimit)?;
            if notice_bytes > 2 * 1024 * 1024 {
                return Err(ToolError::DownloadLimit);
            }
        }
        if executables != 1 || serde_json::to_vec(self)?.len() as u64 > MANIFEST_LIMIT {
            return Err(ToolError::InvalidManifest);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn package() -> ArchivePackage {
        ArchivePackage {
            version: 1,
            notices: vec![],
            layout: None,
            tool: "deno".into(),
            tool_version: "fixture-1".into(),
            target: "linux-x64".into(),
            source_url: "https://example.invalid/runtime.zip".into(),
            license: "MIT".into(),
            ownership: Ownership::Managed,
            archive_bytes: 100,
            archive_sha256: "a".repeat(64),
            components: vec![ArchiveComponent {
                name: "deno".into(),
                bytes: 10,
                sha256: "b".repeat(64),
                role: ArchiveComponentRole::Executable,
            }],
        }
    }
    #[test]
    fn rejects_foreign_or_unbounded_package() {
        let mut p = package();
        assert!(p.validate().is_ok());
        p.ownership = Ownership::External;
        assert!(matches!(p.validate(), Err(ToolError::ExternalOwnership)));
        p = package();
        p.archive_bytes = MAX_ARCHIVE_BYTES + 1;
        assert!(p.validate().is_err());
        p = package();
        p.components[0].bytes = MAX_EXPANDED_BYTES + 1;
        assert!(matches!(p.validate(), Err(ToolError::DownloadLimit)));
    }
    #[test]
    fn rejects_path_alias_and_duplicate_component() {
        for name in [
            "../deno",
            "a/deno",
            "C:deno",
            "deno\\x",
            "manifest.json",
            "archive-package.json",
            ".dlpad-owner",
            "CON",
            "nul.txt",
            "COM1.exe",
            "lpt9",
            "deno.",
        ] {
            let mut p = package();
            p.components[0].name = name.into();
            assert!(p.validate().is_err(), "{name}");
        }
        let mut p = package();
        let mut duplicate = p.components[0].clone();
        duplicate.name = "DENO".into();
        duplicate.role = ArchiveComponentRole::Auxiliary;
        p.components.push(duplicate);
        assert!(p.validate().is_err());
    }
    #[test]
    fn rejects_source_credentials_and_ambiguous_schema() {
        for url in [
            "http://example.invalid/r.zip",
            "https://u:p@example.invalid/r.zip",
            "https://example.invalid/r.zip?secret=a",
            "https://example.invalid/r.zip#x",
        ] {
            let mut p = package();
            p.source_url = url.into();
            assert!(p.validate().is_err());
        }
        let mut data = serde_json::to_value(package()).unwrap();
        data["unexpected"] = true.into();
        assert!(serde_json::from_value::<ArchivePackage>(data).is_err());
        let mut p = package();
        p.components[0].role = ArchiveComponentRole::Notice;
        assert!(p.validate().is_err());
    }
    fn with_notice() -> ArchivePackage {
        let mut p = package();
        p.version = 3;
        p.components.push(ArchiveComponent {
            name: "LICENSE".into(),
            bytes: 100,
            sha256: "c".repeat(64),
            role: ArchiveComponentRole::Notice,
        });
        p.notices.push(PinnedArchiveNotice {
            component: "LICENSE".into(),
            source_url: "https://example.invalid/revision/LICENSE".into(),
            revision: "d".repeat(40),
        });
        p
    }
    #[test]
    fn v3_notice_is_explicit_and_old_serialization_stays_compatible() {
        let old = package();
        let json = serde_json::to_string(&old).unwrap();
        assert!(!json.contains("notices"));
        let parsed: ArchivePackage = serde_json::from_str(&json).unwrap();
        assert!(parsed.notices.is_empty());
        assert!(parsed.validate().is_ok());
        let p = with_notice();
        assert!(p.validate().is_ok());
        assert_eq!(
            p.archive_components()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>(),
            vec!["deno"]
        );
        let parsed: ArchivePackage =
            serde_json::from_slice(&serde_json::to_vec(&p).unwrap()).unwrap();
        assert!(parsed.validate().is_ok());
        let mut bad = p.clone();
        bad.version = 1;
        assert!(bad.validate().is_err());
        bad.version = 2;
        assert!(bad.validate().is_err());
    }
    #[test]
    fn rejects_wrong_notice_identity_transport_revision_and_bounds() {
        for url in [
            "http://example.invalid/LICENSE",
            "https://user@example.invalid/LICENSE",
            "https://example.invalid/LICENSE?q=x",
            "https://example.invalid/LICENSE#x",
            "https://example.invalid/a b",
        ] {
            let mut p = with_notice();
            p.notices[0].source_url = url.into();
            assert!(p.validate().is_err());
        }
        for revision in ["a".repeat(39), "z".repeat(40)] {
            let mut p = with_notice();
            p.notices[0].revision = revision;
            assert!(p.validate().is_err());
        }
        let mut p = with_notice();
        p.notices[0].component = "deno".into();
        assert!(p.validate().is_err());
        p = with_notice();
        p.notices[0].component = "license".into();
        assert!(p.validate().is_err());
        p = with_notice();
        p.notices.push(p.notices[0].clone());
        assert!(p.validate().is_err());
        p = with_notice();
        p.components[1].bytes = 1024 * 1024 + 1;
        assert!(p.validate().is_err());
        p = with_notice();
        p.notices
            .extend([p.notices[0].clone(), p.notices[0].clone()]);
        assert!(p.validate().is_err());
        let encoded = serde_json::to_string(&with_notice())
            .unwrap()
            .replace("\"revision\":", "\"unknown\":0,\"revision\":");
        assert!(serde_json::from_str::<ArchivePackage>(&encoded).is_err());
    }
}
