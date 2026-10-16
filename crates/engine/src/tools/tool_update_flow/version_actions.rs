//! Explicit install-only and installed-version rollback; no startup/version auto switch.
use super::release_discovery::Tool;
use crate::tools::{
    ArchivePackage, InstallManifest, InstalledTool, Ownership, ToolError, ToolManager,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
#[derive(Debug)]
pub enum FlowError {
    Cancelled,
    UnapprovedVersion,
    InvalidCatalog,
    ArchiveMaterializationRequired,
    Tool(ToolError),
}
impl From<ToolError> for FlowError {
    fn from(e: ToolError) -> Self {
        match e {
            ToolError::Cancelled => Self::Cancelled,
            e => Self::Tool(e),
        }
    }
}
#[derive(Clone)]
pub enum Supply {
    Direct {
        manifest: InstallManifest,
        bytes: u64,
    },
    Archive(ArchivePackage),
}
impl Supply {
    fn identity(&self) -> (&str, &str) {
        match self {
            Self::Direct { manifest, .. } => (&manifest.tool, &manifest.version),
            Self::Archive(p) => (&p.tool, &p.tool_version),
        }
    }
}
/// App compiled supply inventory. This is deliberately NOT Deserialize from release JSON.
pub struct Catalog {
    entries: Vec<Supply>,
}
impl Catalog {
    pub fn from_compiled(entries: Vec<Supply>) -> Result<Self, FlowError> {
        if entries.len() > 32 {
            return Err(FlowError::InvalidCatalog);
        }
        let mut unique = std::collections::HashSet::new();
        for e in &entries {
            let (t, v) = e.identity();
            if !matches!(t, "yt-dlp" | "deno" | "ffmpeg") || !unique.insert((t, v)) {
                return Err(FlowError::InvalidCatalog);
            }
            match e {
                Supply::Direct { manifest, bytes } => {
                    super::super::validate(manifest)?;
                    if manifest.ownership != Ownership::Managed
                        || manifest.tool != "yt-dlp"
                        || *bytes == 0
                        || *bytes > crate::tools::MAX_DOWNLOAD_BYTES
                    {
                        return Err(FlowError::InvalidCatalog);
                    }
                }
                Supply::Archive(p) => {
                    if p.ownership != Ownership::Managed
                        || !matches!(p.tool.as_str(), "deno" | "ffmpeg")
                    {
                        return Err(FlowError::InvalidCatalog);
                    }
                    p.validate()?;
                }
            }
        }
        Ok(Self { entries })
    }
    fn find(&self, tool: Tool, version: &str) -> Result<&Supply, FlowError> {
        self.entries
            .iter()
            .find(|s| s.identity() == (tool.id(), version))
            .ok_or(FlowError::UnapprovedVersion)
    }
}
/// No implicit fallback from unknown latest to some unrelated approved version.
pub fn install_approved(
    manager: &mut ToolManager,
    catalog: &Catalog,
    tool: Tool,
    version: &str,
    cancel: &Arc<AtomicBool>,
) -> Result<InstalledTool, FlowError> {
    if cancel.load(Ordering::Acquire) {
        return Err(FlowError::Cancelled);
    }
    let supply = catalog.find(tool, version)?;
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(120))
        .ok_or(FlowError::Tool(ToolError::TimedOut))?;
    match supply {
        Supply::Direct { manifest, bytes } => Ok(manager.download_and_install_inactive_until(
            manifest.clone(),
            *bytes,
            cancel,
            deadline,
        )?),
        Supply::Archive(package) => {
            manager.download_archive_and_install_until(package.clone(), cancel, deadline)?;
            let verified = match tool {
                Tool::Deno => manager.inspect_verified_deno(package, cancel)?,
                Tool::Ffmpeg => manager.inspect_verified_ffmpeg(package, cancel)?,
                Tool::YtDlp => return Err(FlowError::InvalidCatalog),
            };
            Ok(verified.main)
        }
    }
}
/// Request carries the exact owned installed manifest, never just a row index/version substring.
/// Whole bytes/runtime verification occurs in the same root lock before active commit.
pub fn rollback_verified(
    manager: &mut ToolManager,
    manifest: &InstallManifest,
    cancel: &Arc<AtomicBool>,
) -> Result<InstalledTool, FlowError> {
    if cancel.load(Ordering::Acquire) {
        return Err(FlowError::Cancelled);
    }
    match manifest.tool.as_str() {
        "yt-dlp" => Ok(manager.select_verified_existing(manifest, cancel)?),
        "deno" | "ffmpeg" => {
            let package = manager
                .recorded_package_for_manifest(manifest)?
                .ok_or(FlowError::InvalidCatalog)?;
            let result = if manifest.tool == "deno" {
                manager.select_verified_deno(&package.package, cancel)?
            } else {
                manager.select_verified_ffmpeg(&package.package, cancel)?
            };
            Ok(result.main)
        }
        _ => Err(FlowError::InvalidCatalog),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_release_does_not_become_an_installable_catalog_entry() {
        let catalog = Catalog::from_compiled(vec![]).unwrap();
        assert!(matches!(
            catalog.find(Tool::YtDlp, "9999.01.01"),
            Err(FlowError::UnapprovedVersion)
        ));
    }
    #[test]
    fn duplicate_or_external_manifest_catalog_fails_before_any_io() {
        let m = InstallManifest {
            tool: "yt-dlp".into(),
            version: "1".into(),
            target: "x86_64-pc-windows-msvc".into(),
            source_url: "https://github.com/yt-dlp/yt-dlp/releases/download/1/yt-dlp.exe".into(),
            sha256: "a".repeat(64),
            license: "synthetic".into(),
            ownership: Ownership::Managed,
            executable: "yt-dlp.exe".into(),
        };
        let supply = Supply::Direct {
            manifest: m.clone(),
            bytes: 10,
        };
        assert!(matches!(
            Catalog::from_compiled(vec![supply.clone(), supply]),
            Err(FlowError::InvalidCatalog)
        ));
        let mut ext = m;
        ext.ownership = Ownership::External;
        assert!(
            Catalog::from_compiled(vec![Supply::Direct {
                manifest: ext,
                bytes: 10
            }])
            .is_err()
        );
    }
    #[test]
    fn complete_valid_catalog_requires_managed_ownership_for_both_supplies() {
        let manifest = InstallManifest {
            tool: "yt-dlp".into(),
            version: "2026.08.19".into(),
            target: "x86_64-pc-windows-msvc".into(),
            source_url: "https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/yt-dlp.exe"
                .into(),
            sha256: "a".repeat(64),
            license: "Unlicense".into(),
            ownership: Ownership::Managed,
            executable: "yt-dlp.exe".into(),
        };
        let package=ArchivePackage{version:1,tool:"deno".into(),tool_version:"2.9.7".into(),target:"x86_64-pc-windows-msvc".into(),source_url:"https://github.com/denoland/deno/releases/download/v2.9.7/deno-x86_64-pc-windows-msvc.zip".into(),license:"MIT".into(),ownership:Ownership::Managed,archive_bytes:100,archive_sha256:"b".repeat(64),components:vec![crate::tools::ArchiveComponent{name:"deno.exe".into(),bytes:10,sha256:"c".repeat(64),role:crate::tools::ArchiveComponentRole::Executable}],layout:None,notices:vec![]};
        assert!(
            Catalog::from_compiled(vec![
                Supply::Direct {
                    manifest: manifest.clone(),
                    bytes: 10
                },
                Supply::Archive(package.clone())
            ])
            .is_ok()
        );
        let mut external = manifest;
        external.ownership = Ownership::External;
        assert!(
            Catalog::from_compiled(vec![Supply::Direct {
                manifest: external,
                bytes: 10
            }])
            .is_err()
        );
        let mut external = package;
        external.ownership = Ownership::External;
        assert!(Catalog::from_compiled(vec![Supply::Archive(external)]).is_err());
    }
}
