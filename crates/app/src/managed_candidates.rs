//! Fixed development supply; installation remains inactive and does not establish publisher trust.
use crate::managed::ToolId;
use dlpad_engine::tools::{
    ArchiveComponent, ArchiveComponentRole, ArchiveLayout, ArchiveMemberMapping, ArchivePackage,
    Ownership, PinnedArchiveNotice,
};
pub(crate) fn archive_candidate(tool: ToolId) -> Option<ArchivePackage> {
    if !cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        return None;
    }
    match tool {
        ToolId::YtDlp => None,
        ToolId::Deno => Some(deno()),
        ToolId::Ffmpeg => Some(ffmpeg()),
    }
}
fn deno() -> ArchivePackage {
    ArchivePackage {
 version:3, tool:"deno".into(),tool_version:"2.9.7".into(),target:"x86_64-pc-windows-msvc".into(),
 source_url:"https://github.com/denoland/deno/releases/download/v2.9.7/deno-x86_64-pc-windows-msvc.zip".into(),
 license:"MIT; exact upstream LICENSE.md only; transitive notice/trust adoption unverified".into(),ownership:Ownership::Managed,
 archive_bytes:42_630_221,archive_sha256:"a0c3101b4158d1dfb7d6a78a7bf0f3de80c96bb423c152beec8beb22786f2238".into(),layout:None,
 components:vec![
ArchiveComponent { name: "deno.exe".into(), bytes: 97462048, sha256: "e020f3e232bd16e33768dee528e5983349c962952051ced0a5d58ad42f5d9b33".into(), role: ArchiveComponentRole::Executable },
ArchiveComponent { name: "LICENSE.md".into(), bytes: 1074, sha256: "f62497fffecc0852960c8d3e6934b9db86d16396e9b604072e923892cae3a588".into(), role: ArchiveComponentRole::Notice }
 ],notices:vec![PinnedArchiveNotice {component:"LICENSE.md".into(),revision:"0c071246a412575e07423263404a5d13e7ed6aa2".into(),source_url:"https://raw.githubusercontent.com/denoland/deno/0c071246a412575e07423263404a5d13e7ed6aa2/LICENSE.md".into()}],
}
}
fn ffmpeg() -> ArchivePackage {
    ArchivePackage {
 version:2,tool:"ffmpeg".into(),tool_version:"n9.0.2-17-g2a571b6068-20260930".into(),target:"x86_64-pc-windows-gnu".into(),ownership:Ownership::Managed,
 license:"LGPL-3.0-or-later; supply adoption pending".into(),
source_url:"https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-09-30-13-08/ffmpeg-n9.0.2-17-g2a571b6068-win64-lgpl-9.0.zip".into(),archive_bytes:171535354,archive_sha256:"6b264b9e6019103f601d98c292bd332fd87acf1c5e941ddff4fb71760fe63432".into(),components:vec![
ArchiveComponent { name: "LICENSE.txt".into(), bytes: 7651, sha256: "da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768".into(), role: ArchiveComponentRole::Notice },
ArchiveComponent { name: "ffmpeg.exe".into(), bytes: 134497280, sha256: "0e482b853f2fd4821055d20cdce510f0eea6027037c04b994f454a2136e8868e".into(), role: ArchiveComponentRole::Executable },
ArchiveComponent { name: "ffprobe.exe".into(), bytes: 134296064, sha256: "b9627185ffbad1d4fc6ef953868ca9ad8df75fc8dd035fcab58aca78caa2d23e".into(), role: ArchiveComponentRole::Auxiliary }
],notices:vec![],layout:Some(ArchiveLayout::Zip {root:"ffmpeg-n9.0.2-17-g2a571b6068-win64-lgpl-9.0".into(),mappings:vec![
ArchiveMemberMapping {member:"ffmpeg-n9.0.2-17-g2a571b6068-win64-lgpl-9.0/LICENSE.txt".into(),component:"LICENSE.txt".into()},
ArchiveMemberMapping {member:"ffmpeg-n9.0.2-17-g2a571b6068-win64-lgpl-9.0/bin/ffmpeg.exe".into(),component:"ffmpeg.exe".into()},
ArchiveMemberMapping {member:"ffmpeg-n9.0.2-17-g2a571b6068-win64-lgpl-9.0/bin/ffprobe.exe".into(),component:"ffprobe.exe".into()}
] }),
}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_packages_and_external_notice_validate() {
        let d = deno();
        let f = ffmpeg();
        d.validate().unwrap();
        f.validate().unwrap();
        assert_eq!(d.archive_components().count(), 1);
        assert_eq!(f.archive_components().count(), 3);
        let mut bad = d.clone();
        bad.notices[0].component = "missing.txt".into();
        assert!(bad.validate().is_err());
        assert_eq!(
            f.components
                .iter()
                .filter(|c| c.role == ArchiveComponentRole::Auxiliary)
                .count(),
            1
        );
    }
    #[test]
    fn platform_and_tool_gate_are_explicit() {
        assert!(archive_candidate(ToolId::YtDlp).is_none());
        for t in [ToolId::Deno, ToolId::Ffmpeg] {
            assert_eq!(
                archive_candidate(t).is_some(),
                cfg!(all(target_os = "windows", target_arch = "x86_64"))
            );
        }
    }
}
