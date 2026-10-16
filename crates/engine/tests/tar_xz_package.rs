use dlpad_engine::tools::{
    ArchiveComponent, ArchiveComponentRole, ArchiveLayout, ArchiveMemberMapping, ArchivePackage,
    Ownership, ToolError, ToolManager,
};
use sha2::{Digest, Sha256};
use std::{fs, sync::atomic::AtomicBool};
const VALID: &[u8] = &[
    253, 55, 122, 88, 90, 0, 0, 4, 230, 214, 180, 70, 2, 0, 33, 1, 22, 0, 0, 0, 116, 47, 229, 163,
    224, 7, 255, 0, 62, 93, 0, 57, 27, 236, 232, 47, 37, 78, 155, 192, 90, 38, 131, 209, 31, 67,
    231, 82, 39, 237, 224, 103, 128, 2, 144, 210, 59, 249, 111, 12, 195, 29, 20, 65, 207, 19, 57,
    209, 136, 92, 105, 20, 114, 2, 176, 202, 9, 249, 153, 240, 145, 2, 67, 106, 19, 180, 64, 16, 0,
    103, 9, 100, 0, 0, 0, 0, 49, 122, 82, 147, 212, 17, 103, 149, 0, 1, 90, 128, 16, 0, 0, 0, 65,
    24, 191, 143, 177, 196, 103, 251, 2, 0, 0, 0, 0, 4, 89, 90,
];

fn package(version: &str, bytes: &[u8]) -> ArchivePackage {
    ArchivePackage {
        notices: vec![],
        version: 2,
        tool: "ffmpeg".into(),
        tool_version: version.into(),
        target: "fixture-linux".into(),
        source_url: "https://example.invalid/synthetic.tar.xz".into(),
        license: "synthetic fixture".into(),
        ownership: Ownership::Managed,
        archive_bytes: bytes.len() as u64,
        archive_sha256: format!("{:x}", Sha256::digest(bytes)),
        components: vec![ArchiveComponent {
            name: "tool".into(),
            bytes: 3,
            sha256: format!("{:x}", Sha256::digest(b"abc")),
            role: ArchiveComponentRole::Executable,
        }],
        layout: Some(ArchiveLayout::TarXz {
            root: "root".into(),
            mappings: vec![ArchiveMemberMapping {
                member: "root/bin/tool".into(),
                component: "tool".into(),
            }],
        }),
    }
}
#[test]
fn inactive_inventory_select_remove_and_preserve() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.tar.xz");
    fs::write(&source, VALID).unwrap();
    let sentinel = temp.path().join("foreign");
    fs::write(&sentinel, b"preserve").unwrap();
    let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
    let first = manager
        .install_verified_archive(&source, package("1.0.0", VALID), &AtomicBool::new(false))
        .unwrap();
    assert!(manager.active("ffmpeg").unwrap().is_none());
    let inventory = manager
        .package_installed(&first.main.manifest)
        .unwrap()
        .unwrap();
    assert_eq!(inventory.directory, first.directory);
    assert_eq!(fs::read(first.directory.join("tool")).unwrap(), b"abc");
    manager.select_package(&first.package).unwrap();
    assert!(matches!(
        manager.remove_package(&first.package),
        Err(ToolError::ActiveVersion)
    ));
    let second = manager
        .install_verified_archive(&source, package("2.0.0", VALID), &AtomicBool::new(false))
        .unwrap();
    assert_eq!(
        manager.active("ffmpeg").unwrap(),
        Some(("1.0.0".into(), "fixture-linux".into()))
    );
    manager.remove_package(&second.package).unwrap();
    assert!(!second.directory.exists());
    assert!(first.directory.exists());
    drop(manager);
    let manager = ToolManager::new(&temp.path().join("managed")).unwrap();
    assert!(
        manager
            .package_installed(&first.main.manifest)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        manager.active("ffmpeg").unwrap(),
        Some(("1.0.0".into(), "fixture-linux".into()))
    );
    assert_eq!(fs::read(&source).unwrap(), VALID);
    assert_eq!(fs::read(sentinel).unwrap(), b"preserve");
}
#[test]
fn schema_versions_layout_mapping_and_size_caps() {
    let mut old = package("1.0.0", VALID);
    old.version = 1;
    old.layout = None;
    assert!(old.validate().is_ok());
    let json = serde_json::to_value(&old).unwrap();
    assert!(json.get("layout").is_none());
    let old_bytes = serde_json::to_vec(&json).unwrap();
    assert!(
        !String::from_utf8(old_bytes.clone())
            .unwrap()
            .contains("layout")
    );
    let decoded: ArchivePackage = serde_json::from_slice(&old_bytes).unwrap();
    let mut unknown = serde_json::to_value(package("1.0.0", VALID)).unwrap();
    unknown["layout"]["TarXz"]["mappings"][0]["sha256"] = serde_json::json!("not a mapping field");
    assert!(serde_json::from_value::<ArchivePackage>(unknown).is_err());
    assert!(decoded.layout.is_none());
    let mut bad = old.clone();
    bad.layout = package("1.0.0", VALID).layout;
    assert!(bad.validate().is_err());
    bad = package("1.0.0", VALID);
    bad.layout = None;
    assert!(bad.validate().is_err());
    bad = package("1.0.0", VALID);
    let Some(ArchiveLayout::TarXz { mappings, .. }) = &mut bad.layout else {
        unreachable!()
    };
    mappings[0].component = "absent".into();
    assert!(bad.validate().is_err());
    bad = package("1.0.0", VALID);
    bad.components[0].bytes = 384 * 1024 * 1024 + 1;
    assert!(bad.validate().is_err());
    old.components[0].bytes = 256 * 1024 * 1024 + 1;
    assert!(old.validate().is_err());
    let mut allowed = package("1.0.0", VALID);
    allowed.components[0].bytes = 256 * 1024 * 1024 + 1;
    assert!(allowed.validate().is_ok());
}
#[test]
fn bad_hash_truncated_and_component_hash_preserve_prior_active() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.tar.xz");
    fs::write(&source, VALID).unwrap();
    let sentinel = temp.path().join("foreign");
    fs::write(&sentinel, b"preserve").unwrap();
    let mut manager = ToolManager::new(&temp.path().join("managed")).unwrap();
    let prior = manager
        .install_verified_archive(&source, package("1.0.0", VALID), &AtomicBool::new(false))
        .unwrap();
    manager.select_package(&prior.package).unwrap();
    for kind in 0..3 {
        let bytes = if kind == 1 {
            &VALID[..VALID.len() - 1]
        } else {
            VALID
        };
        fs::write(&source, bytes).unwrap();
        let mut p = package(&format!("2.0.{kind}"), bytes);
        if kind == 0 {
            p.archive_sha256 = "0".repeat(64);
        }
        if kind == 2 {
            p.components[0].sha256 = "0".repeat(64);
        }
        assert!(
            manager
                .install_verified_archive(&source, p, &AtomicBool::new(false))
                .is_err()
        );
        assert_eq!(
            manager.active("ffmpeg").unwrap(),
            Some(("1.0.0".into(), "fixture-linux".into()))
        );
        assert_eq!(fs::read(prior.directory.join("tool")).unwrap(), b"abc");
        assert_eq!(fs::read(&source).unwrap(), bytes);
        assert_eq!(fs::read(&sentinel).unwrap(), b"preserve");
    }
}
