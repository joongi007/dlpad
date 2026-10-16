//! User-requested local report I/O. These functions belong on a worker, not a UI thread.
//!
//! P0 retention policy: reports live only in session memory; previews and attachments
//! never create an automatic disk cache. The automatic diagnostic cache budget is zero.
//! Only `save_report` persists a report, at the exact user-selected path. That export is
//! user-owned, not a cache entry: it has no application expiry or automatic cleanup.
//! Attachments are read-only sources, never copied into a raw disk cache or removed.
//! Cleanup is restricted to the same newly created export file if its write fails.
use dlpad_core::diagnostics::{DiagnosticReport, FailureStage, MAX_REPORT_BYTES, ToolComponent};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

pub const MAX_MPD_BYTES: usize = 1024 * 1024;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Markdown,
    Json,
}

/// Serialize only already sanitized report state, then create a new user-selected export.
/// Existing files are never overwritten. No network or job-state changes occur.
pub fn save_report(report: &DiagnosticReport, path: &Path, format: ReportFormat) -> io::Result<()> {
    save_report_inner(report, path, format, |_| Ok(()))
}
/// Explicit feature-only fixture boundary; production save_report never activates a hook.
#[cfg(feature = "report-closewait-fixture")]
pub fn save_report_after_create(
    report: &DiagnosticReport,
    path: &Path,
    format: ReportFormat,
    after_create: impl FnOnce(&File) -> io::Result<()>,
) -> io::Result<()> {
    save_report_inner(report, path, format, after_create)
}
fn save_report_inner(
    report: &DiagnosticReport,
    path: &Path,
    format: ReportFormat,
    after_create: impl FnOnce(&File) -> io::Result<()>,
) -> io::Result<()> {
    let preview = match format {
        ReportFormat::Markdown => report.markdown_preview(),
        ReportFormat::Json => report.json_preview().map_err(io::Error::other)?,
    };
    save_preview_inner(&preview, path, after_create)
}

// Defensive serialized-byte boundary, independent of bounded report-domain construction.
// This private writer does not add a public arbitrary-payload export API.
fn save_preview_inner(
    preview: &str,
    path: &Path,
    after_create: impl FnOnce(&File) -> io::Result<()>,
) -> io::Result<()> {
    if preview.len() > MAX_REPORT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Diagnostic report exceeds 2 MiB",
        ));
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    let result = after_create(&file)
        .and_then(|()| file.write_all(preview.as_bytes()))
        .and_then(|()| file.sync_all());
    if result.is_err() {
        cleanup_owned_partial(&file, path);
    }
    result
}

fn cleanup_owned_partial(file: &File, path: &Path) {
    // A path may have been replaced while writing. Remove only the same regular file.
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let (Ok(owned), Ok(current)) = (file.metadata(), fs::symlink_metadata(path))
            && current.is_file()
            && owned.dev() == current.dev()
            && owned.ino() == current.ino()
        {
            let _ = fs::remove_file(path);
        }
    }
    // std has no portable handle identity/removal API. Keep an uncertain partial rather
    // than risk deleting another user's file; Windows handle identity remains a test gate.
    #[cfg(not(unix))]
    {
        let _ = (file, path);
    }
}

/// Read at most 1 MiB + one detection byte; raw MPD never falls back into a report.
pub fn attach_local_mpd(report: &mut DiagnosticReport, path: &Path) -> io::Result<()> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "MPD attachment must be a regular local file",
        ));
    }
    let file = File::open(path)?;
    let mut bytes = Vec::with_capacity(MAX_MPD_BYTES.min(metadata.len() as usize));
    file.take((MAX_MPD_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    report.add_mpd_attachment(&bytes).map_err(|reason| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("MPD omitted: {reason:?}"),
        )
    })
}

/// Synthetic evidence exclusively for fixture mode; no real account or media.
pub fn demo_report() -> DiagnosticReport {
    let mut report = DiagnosticReport::new(1, FailureStage::Metadata, Some(1));
    report.add_tool(ToolComponent::YtDlp, "2026.09.30", false);
    report.add_log("KeyError sourceURL\nMetadata extraction failed\nrequest https://media.invalid/synthetic?signature=fixture");
    let _ = report.add_mpd_attachment(br#"<MPD><BaseURL>https://media.invalid/synthetic?signature=fixture</BaseURL><SegmentList><Initialization range="0-59062"/><SegmentURL mediaRange="59063-124807"/></SegmentList></MPD>"#);
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn existing_export_is_preserved_and_json_is_valid() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("report.json");
        fs::write(&path, "user data").unwrap();
        assert_eq!(
            save_report(&demo_report(), &path, ReportFormat::Json)
                .unwrap_err()
                .kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), "user data");
        let output = temp.path().join("new.json");
        save_report(&demo_report(), &output, ReportFormat::Json).unwrap();
        let json = fs::read_to_string(&output).unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
        assert!(!json.contains("signature=fixture"));
    }
    #[cfg(feature = "report-closewait-fixture")]
    #[test]
    fn callback_failure_uses_original_partial_policy_and_preserves_existing() {
        let temp = tempfile::tempdir().unwrap();
        let fresh = temp.path().join("partial.md");
        let error =
            save_report_after_create(&demo_report(), &fresh, ReportFormat::Markdown, |file| {
                assert!(file.metadata()?.is_file());
                assert_eq!(file.metadata()?.len(), 0);
                Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "fixture stopped before write",
                ))
            })
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        #[cfg(unix)]
        assert!(!fresh.exists());
        #[cfg(not(unix))]
        assert_eq!(fs::read(&fresh).unwrap(), b"");
        let sentinel = temp.path().join("existing.md");
        fs::write(&sentinel, b"preserve").unwrap();
        let invoked = std::cell::Cell::new(false);
        assert_eq!(
            save_report_after_create(&demo_report(), &sentinel, ReportFormat::Markdown, |_| {
                invoked.set(true);
                Ok(())
            })
            .unwrap_err()
            .kind(),
            io::ErrorKind::AlreadyExists
        );
        assert!(!invoked.get());
        assert_eq!(fs::read(sentinel).unwrap(), b"preserve");
    }
    #[test]
    fn defensive_preview_exact_cap_utf8_and_overflow_preserve_foreign_files() {
        let temp = tempfile::tempdir().unwrap();
        // Valid UTF-8 JSON string, but intentionally not a bounded DiagnosticReport.
        let exact = format!("\"{}\"", "한".repeat(699_050));
        assert_eq!(exact.len(), MAX_REPORT_BYTES);
        let source = temp.path().join("synthetic-source");
        let sentinel = temp.path().join("foreign-sentinel");
        fs::write(&source, b"synthetic source stays read-only").unwrap();
        fs::write(&sentinel, b"foreign user export remains unchanged").unwrap();
        let output = temp.path().join("exact.json");
        save_preview_inner(&exact, &output, |_| Ok(())).unwrap();
        let mut reader = File::open(&output).unwrap();
        let mut actual = vec![0; MAX_REPORT_BYTES];
        reader.read_exact(&mut actual).unwrap();
        assert_eq!(actual, exact.as_bytes());
        assert!(std::str::from_utf8(&actual).is_ok());
        let mut detection = [0u8; 1];
        assert_eq!(reader.read(&mut detection).unwrap(), 0);
        assert_eq!(reader.metadata().unwrap().len(), MAX_REPORT_BYTES as u64);

        let invoked = std::cell::Cell::new(false);
        let overflow = format!("{exact} ");
        assert_eq!(overflow.len(), MAX_REPORT_BYTES + 1);
        let rejected = temp.path().join("overflow.json");
        let error = save_preview_inner(&overflow, &rejected, |_| {
            invoked.set(true);
            Ok(())
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(!invoked.get());
        assert!(!rejected.exists());
        assert_eq!(
            save_preview_inner(&exact, &sentinel, |_| {
                invoked.set(true);
                Ok(())
            })
            .unwrap_err()
            .kind(),
            io::ErrorKind::AlreadyExists
        );
        assert!(!invoked.get());
        assert_eq!(
            fs::read(&source).unwrap(),
            b"synthetic source stays read-only"
        );
        assert_eq!(
            fs::read(&sentinel).unwrap(),
            b"foreign user export remains unchanged"
        );
    }

    #[test]
    fn defensive_preview_hook_failure_preserves_original_partial_policy() {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("owned-partial.json");
        let error = save_preview_inner("\"합성\"", &output, |file| {
            assert_eq!(file.metadata()?.len(), 0);
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "synthetic write boundary failure",
            ))
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        #[cfg(unix)]
        assert!(!output.exists());
        #[cfg(not(unix))]
        assert_eq!(fs::read(output).unwrap(), b"");
    }

    #[test]
    fn oversized_report_creates_no_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("huge.md");
        let mut report = demo_report();
        report.residual_risk = Box::leak("x".repeat(MAX_REPORT_BYTES + 1).into_boxed_str());
        assert!(save_report(&report, &path, ReportFormat::Markdown).is_err());
        assert!(!path.exists());
    }
    #[test]
    fn local_attachment_is_bounded_and_secret_free() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("input.mpd");
        fs::write(&path, br#"<MPD><BaseURL>https://invalid.test/PRIVATE?token=PRIVATE</BaseURL><SegmentList><Initialization range="0-5"/></SegmentList></MPD>"#).unwrap();
        let mut report = demo_report();
        attach_local_mpd(&mut report, &path).unwrap();
        let export = temp.path().join("report.md");
        save_report(&report, &export, ReportFormat::Markdown).unwrap();
        assert!(!fs::read_to_string(&export).unwrap().contains("PRIVATE"));
        fs::write(&path, vec![b'x'; MAX_MPD_BYTES + 1]).unwrap();
        let count = report.mpd_attachments().len();
        assert!(attach_local_mpd(&mut report, &path).is_err());
        assert_eq!(report.mpd_attachments().len(), count);
        assert_eq!(
            fs::metadata(&path).unwrap().len(),
            (MAX_MPD_BYTES + 1) as u64
        );
    }
    #[cfg(unix)]
    #[test]
    fn cleanup_removes_owned_file_and_keeps_replacement() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("partial");
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        cleanup_owned_partial(&file, &path);
        assert!(!path.exists());
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        fs::rename(&path, temp.path().join("owned")).unwrap();
        fs::write(&path, "replacement user data").unwrap();
        cleanup_owned_partial(&file, &path);
        assert_eq!(fs::read_to_string(&path).unwrap(), "replacement user data");
    }
}
