//! Explicit archive member layouts; component hashes remain in ArchivePackage.
#[cfg(test)]
use super::ArchiveComponentRole;
use super::archive_manifest::component_name;
use super::{ArchiveComponent, ToolError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ArchiveLayout {
    Zip {
        root: String,
        mappings: Vec<ArchiveMemberMapping>,
    },
    TarXz {
        root: String,
        mappings: Vec<ArchiveMemberMapping>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveMemberMapping {
    pub member: String,
    pub component: String,
}

impl ArchiveLayout {
    pub fn validate(&self, components: &[ArchiveComponent]) -> Result<(), ToolError> {
        let (root, mappings) = match self {
            Self::TarXz { root, mappings } | Self::Zip { root, mappings } => (root, mappings),
        };
        if root.len() > 100
            || !component_name(root)
            || components.is_empty()
            || components.len() > 8
            || mappings.len() != components.len()
        {
            return Err(ToolError::InvalidManifest);
        }
        let mut declared = BTreeSet::new();
        for component in components {
            if !component_name(&component.name)
                || !declared.insert(component.name.to_ascii_lowercase())
            {
                return Err(ToolError::InvalidManifest);
            }
        }
        let prefix = format!("{root}/");
        let mut names = BTreeSet::new();
        let mut sources = BTreeSet::new();
        for mapping in mappings {
            if !component_name(&mapping.component)
                || !components.iter().any(|c| c.name == mapping.component)
                || !names.insert(mapping.component.as_str())
                || mapping.member.len() > 255
                || !mapping.member.starts_with(&prefix)
                || !mapping.member.is_ascii()
                || mapping.member.contains('\\')
                || !mapping.member.split('/').all(component_name)
                || !sources.insert(mapping.member.to_ascii_lowercase())
            {
                return Err(ToolError::InvalidManifest);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    fn components() -> Vec<ArchiveComponent> {
        vec![ArchiveComponent {
            name: "ffmpeg".into(),
            bytes: 3,
            sha256: "0".repeat(64),
            role: ArchiveComponentRole::Executable,
        }]
    }
    fn layout(member: &str, component: &str) -> ArchiveLayout {
        ArchiveLayout::TarXz {
            root: "supply-root".into(),
            mappings: vec![ArchiveMemberMapping {
                member: member.into(),
                component: component.into(),
            }],
        }
    }
    #[test]
    fn exact_mapping_and_unsafe_names() {
        assert!(
            layout("supply-root/bin/ffmpeg", "ffmpeg")
                .validate(&components())
                .is_ok()
        );
        for member in [
            "/supply-root/ffmpeg",
            "supply-root/../ffmpeg",
            "supply-root//ffmpeg",
            "supply-root/./ffmpeg",
            "supply-root/bin\\ffmpeg",
            "other/ffmpeg",
            "supply-root/CON",
            "supply-root/ffmpeg.",
        ] {
            assert!(layout(member, "ffmpeg").validate(&components()).is_err());
        }
        assert!(
            layout("supply-root/ffmpeg", "FFmpeg")
                .validate(&components())
                .is_err()
        );
        assert!(
            layout("supply-root/ffmpeg", "missing")
                .validate(&components())
                .is_err()
        );
        assert!(
            layout("supply-root/ffmpeg", "ffmpeg")
                .validate(&[])
                .is_err()
        );
    }
    #[test]
    fn alias_duplicate_and_cardinality() {
        let mut components = components();
        components.push(ArchiveComponent {
            name: "ffprobe".into(),
            ..components[0].clone()
        });
        let mut value = layout("supply-root/bin/ffmpeg", "ffmpeg");
        assert!(value.validate(&components).is_err());
        let ArchiveLayout::TarXz { mappings, .. } = &mut value else {
            unreachable!();
        };
        mappings.push(ArchiveMemberMapping {
            member: "supply-root/bin/FFmpeg".into(),
            component: "ffprobe".into(),
        });
        assert!(value.validate(&components).is_err());
        let ArchiveLayout::TarXz { mappings, .. } = &mut value else {
            unreachable!();
        };
        mappings[1].member = "supply-root/bin/ffprobe".into();
        mappings[1].component = "ffmpeg".into();
        assert!(value.validate(&components).is_err());
    }
    #[test]
    fn zip_mapping_rejects_traversal_aliases_and_unknown_components() {
        for (member, component, valid) in [
            ("supply-root/bin/ffmpeg.exe", "ffmpeg", true),
            ("supply-root/../ffmpeg.exe", "ffmpeg", false),
            ("other/bin/ffmpeg.exe", "ffmpeg", false),
            ("supply-root/bin/ffmpeg.exe", "missing", false),
        ] {
            let value = ArchiveLayout::Zip {
                root: "supply-root".into(),
                mappings: vec![ArchiveMemberMapping {
                    member: member.into(),
                    component: component.into(),
                }],
            };
            assert_eq!(value.validate(&components()).is_ok(), valid);
            let decoded: ArchiveLayout =
                serde_json::from_slice(&serde_json::to_vec(&value).unwrap()).unwrap();
            assert_eq!(value, decoded);
        }
    }
    #[test]
    fn serde_unknown_fields_rejected() {
        for json in [
            r#"{"TarXz":{"root":"supply-root","mappings":[],"limit":999}}"#,
            r#"{"TarXz":{"root":"supply-root","mappings":[{"member":"supply-root/a","component":"ffmpeg","sha256":"secret"}]}}"#,
            r#"{"Zip":{"root":"supply-root","mappings":[],"limit":999}}"#,
        ] {
            assert!(serde_json::from_str::<ArchiveLayout>(json).is_err());
        }
        let original = layout("supply-root/bin/ffmpeg", "ffmpeg");
        let decoded: ArchiveLayout =
            serde_json::from_slice(&serde_json::to_vec(&original).unwrap()).unwrap();
        assert_eq!(original, decoded);
    }
}
