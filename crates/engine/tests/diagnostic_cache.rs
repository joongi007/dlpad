//! P0 diagnostics persist only explicit exports and never expire user-owned files.
use dlpad_core::diagnostics::{DiagnosticReport, FailureStage};
use dlpad_engine::reports::{MAX_MPD_BYTES, ReportFormat, attach_local_mpd, save_report};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
enum Entry {
    Directory,
    File(Vec<u8>),
}

fn files(root: &Path) -> BTreeMap<PathBuf, Entry> {
    fn collect(root: &Path, path: &Path, result: &mut BTreeMap<PathBuf, Entry>) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                result.insert(
                    entry.path().strip_prefix(root).unwrap().to_owned(),
                    Entry::Directory,
                );
                collect(root, &entry.path(), result);
            } else {
                result.insert(
                    entry.path().strip_prefix(root).unwrap().to_owned(),
                    Entry::File(fs::read(entry.path()).unwrap()),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    collect(root, root, &mut result);
    result
}

#[test]
fn session_report_and_attachment_never_create_cache_or_change_user_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let attachment = root.join("external-evidence.mpd");
    fs::write(&attachment, br#"<MPD><BaseURL>https://invalid.test/PRIVATE?token=PRIVATE</BaseURL><SegmentList><Initialization range="0-5"/></SegmentList></MPD>"#).unwrap();
    fs::write(root.join("previous-user-export.md"), b"user-managed export").unwrap();
    fs::create_dir(root.join("job-fixture")).unwrap();
    fs::write(root.join("job-fixture/journal"), b"synthetic job state").unwrap();
    fs::write(root.join("job-fixture/video.part"), b"synthetic partial").unwrap();
    let before = files(root);

    let mut report = DiagnosticReport::new(7, FailureStage::Metadata, Some(1));
    report.add_log("synthetic metadata failure\nfailed https://invalid.test/PRIVATE?token=PRIVATE");
    attach_local_mpd(&mut report, &attachment).unwrap();
    assert!(!report.markdown_preview().contains("PRIVATE"));
    assert!(!report.json_preview().unwrap().contains("PRIVATE"));
    assert_eq!(files(root), before);
    assert!(report.edit_log(0, "updated sanitized observation"));
    assert!(report.omit_log(0));
    assert!(report.omit_mpd_attachment(0));
    drop(report);
    assert_eq!(files(root), before);
}

#[test]
fn only_explicit_exports_are_persisted_and_remain_user_owned() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let existing = root.join("existing.json");
    fs::write(&existing, b"existing user data").unwrap();
    let before = files(root);
    let mut report = DiagnosticReport::new(8, FailureStage::Download, Some(1));
    report.add_log("synthetic offline failure https://invalid.test/SECRET?token=SECRET");
    assert_eq!(
        save_report(&report, &existing, ReportFormat::Json)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::AlreadyExists
    );
    assert_eq!(files(root), before);
    let markdown = root.join("user-choice.md");
    let json = root.join("user-choice.json");
    save_report(&report, &markdown, ReportFormat::Markdown).unwrap();
    save_report(&report, &json, ReportFormat::Json).unwrap();
    let persisted = files(root);
    assert_eq!(persisted.len(), before.len() + 2);
    assert_eq!(
        persisted[Path::new("existing.json")],
        Entry::File(b"existing user data".to_vec())
    );
    assert!(
        !String::from_utf8(fs::read(&markdown).unwrap())
            .unwrap()
            .contains("SECRET")
    );
    let serialized = fs::read_to_string(&json).unwrap();
    assert!(!serialized.contains("SECRET"));
    assert!(serde_json::from_str::<serde_json::Value>(&serialized).is_ok());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&json).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    drop(report);
    // Creating another session report does not inspect or expire earlier exports.
    let next = DiagnosticReport::new(9, FailureStage::Unknown, None);
    let _ = next.markdown_preview();
    drop(next);
    assert_eq!(files(root), persisted);
}

#[test]
fn rejected_attachments_and_export_failure_leave_sources_and_files_unchanged() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let malformed = root.join("malformed.mpd");
    let entity = root.join("entity.mpd");
    let oversized = root.join("oversized.mpd");
    fs::write(&malformed, b"<MPD><SegmentList></MPD>").unwrap();
    fs::write(
        &entity,
        b"<!DOCTYPE MPD [<!ENTITY secret SYSTEM 'file:///PRIVATE'>]><MPD>&secret;</MPD>",
    )
    .unwrap();
    fs::write(&oversized, vec![b'x'; MAX_MPD_BYTES + 1]).unwrap();
    let before = files(root);
    for source in [&malformed, &entity, &oversized] {
        let mut report = DiagnosticReport::new(10, FailureStage::Metadata, Some(1));
        assert!(attach_local_mpd(&mut report, source).is_err());
        assert!(report.mpd_attachments().is_empty());
        assert!(!report.markdown_preview().contains("PRIVATE"));
        assert!(
            save_report(
                &report,
                &root.join("absent/report.md"),
                ReportFormat::Markdown
            )
            .is_err()
        );
        drop(report);
        assert_eq!(files(root), before);
    }
}
