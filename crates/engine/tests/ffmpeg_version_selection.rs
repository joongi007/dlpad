#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use dlpad_engine::tools::{
    ArchiveComponent, ArchiveComponentRole, ArchivePackage, InstallManifest, Ownership, ToolManager,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};
fn seed(root: &Path, version: &str, mode: &str) -> ArchivePackage {
    let path = root
        .join("ffmpeg")
        .join(version)
        .join("x86_64-unknown-linux-gnu");
    fs::create_dir_all(&path).unwrap();
    let mut components = Vec::new();
    for (name, role) in [
        ("ffmpeg", ArchiveComponentRole::Executable),
        ("ffprobe", ArchiveComponentRole::Auxiliary),
    ] {
        let reported = if mode == "probe-mismatch" && name == "ffprobe" {
            "0.0.0"
        } else {
            version
        };
        let mutation = if mode == "mutation" && name == "ffprobe" {
            "p.with_name('ffmpeg').write_bytes(p.with_name('ffmpeg').read_bytes()+b'\\n#mutated')"
        } else {
            "pass"
        };
        let code = if mode == "probe-nonzero" && name == "ffprobe" {
            7
        } else {
            0
        };
        let script = format!(
            "#!/usr/bin/python3\nimport pathlib,sys\nassert sys.argv[1:]==['-version']\np=pathlib.Path(__file__)\n{mutation}\nprint('{name} version {reported} Copyright (c) 2000-2026 the FFmpeg developers')\nprint('synthetic configuration')\nsys.exit({code})\n"
        );
        fs::write(path.join(name), script.as_bytes()).unwrap();
        fs::set_permissions(path.join(name), fs::Permissions::from_mode(0o700)).unwrap();
        components.push(ArchiveComponent {
            name: name.into(),
            bytes: script.len() as u64,
            sha256: format!("{:x}", Sha256::digest(script.as_bytes())),
            role,
        });
    }
    fs::write(path.join("LICENSE"), b"synthetic notice").unwrap();
    components.push(ArchiveComponent {
        name: "LICENSE".into(),
        bytes: 16,
        sha256: format!("{:x}", Sha256::digest(b"synthetic notice")),
        role: ArchiveComponentRole::Notice,
    });
    let package = ArchivePackage {
        notices: vec![],
        version: 1,
        layout: None,
        tool: "ffmpeg".into(),
        tool_version: version.into(),
        target: "x86_64-unknown-linux-gnu".into(),
        source_url: "https://example.invalid/synthetic.zip".into(),
        license: "synthetic only".into(),
        ownership: Ownership::Managed,
        archive_bytes: 123,
        archive_sha256: "a".repeat(64),
        components,
    };
    let manifest = InstallManifest {
        tool: package.tool.clone(),
        version: version.into(),
        target: package.target.clone(),
        source_url: package.source_url.clone(),
        sha256: package.components[0].sha256.clone(),
        license: package.license.clone(),
        ownership: Ownership::Managed,
        executable: "ffmpeg".into(),
    };
    fs::write(
        path.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    fs::write(
        path.join("archive-package.json"),
        serde_json::to_vec(&package).unwrap(),
    )
    .unwrap();
    package
}
#[test]
fn both_version_processes_select_and_failure_preserves_active() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("managed");
    let mut manager = ToolManager::new(&root).unwrap();
    let sentinel = temp.path().join("foreign");
    fs::write(&sentinel, b"preserve external").unwrap();
    let prior = seed(&root, "1.0.0", "success");
    let selected = manager
        .select_verified_ffmpeg(&prior, &Arc::new(AtomicBool::new(false)))
        .unwrap();
    let original = fs::read(selected.directory.join("ffmpeg")).unwrap();
    let probe_original = fs::read(selected.directory.join("ffprobe")).unwrap();
    assert_eq!(
        manager.active("ffmpeg").unwrap(),
        Some(("1.0.0".into(), prior.target.clone()))
    );
    for (i, mode) in [
        "probe-mismatch",
        "probe-nonzero",
        "mutation",
        "foreign",
        "cancel",
    ]
    .iter()
    .enumerate()
    {
        let p = seed(&root, &format!("2.0.{i}"), mode);
        let directory = root.join("ffmpeg").join(&p.tool_version).join(&p.target);
        if *mode == "foreign" {
            fs::write(directory.join("foreign-data"), b"do not delete").unwrap();
        }
        assert!(
            manager
                .select_verified_ffmpeg(&p, &Arc::new(AtomicBool::new(*mode == "cancel")))
                .is_err(),
            "accepted failure mode {mode}"
        );
        assert_eq!(
            manager.active("ffmpeg").unwrap(),
            Some(("1.0.0".into(), prior.target.clone()))
        );
        assert_eq!(
            fs::read(selected.directory.join("ffmpeg")).unwrap(),
            original
        );
        assert_eq!(
            fs::read(selected.directory.join("ffprobe")).unwrap(),
            probe_original
        );
        assert_eq!(fs::read(&sentinel).unwrap(), b"preserve external");
        if *mode == "foreign" {
            assert_eq!(
                fs::read(directory.join("foreign-data")).unwrap(),
                b"do not delete"
            );
        }
    }
    let next = seed(&root, "3.0.0", "success");
    manager
        .select_verified_ffmpeg(&next, &Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert_eq!(
        manager.active("ffmpeg").unwrap(),
        Some(("3.0.0".into(), next.target))
    );
}
