//! Fixed developer-only large diagnostic fixture; no external payload is accepted.
use dlpad_core::diagnostics::{DiagnosticReport, FailureStage};

pub(crate) fn stress_report() -> DiagnosticReport {
    let mut report = DiagnosticReport::new(1, FailureStage::Metadata, Some(1));
    let line = ("\"".repeat(48) + " ").repeat(83) + &"\"".repeat(29);
    for _ in 0..128 {
        report.add_log(&line);
    }
    let range = format!("{}-{}", "1".repeat(23), "9".repeat(24));
    let node = format!(
        r#"<Initialization sourceURL="https://fixture.invalid/one" media="https://fixture.invalid/one" range="{range}" mediaRange="{range}"/>"#
    );
    let mpd = format!(
        "<MPD>{}{}{}</MPD>",
        "<Period>".repeat(62),
        node.repeat(256),
        "</Period>".repeat(62)
    );
    for _ in 0..8 {
        // All data above is fixed and within the production parser's limits.
        report
            .add_mpd_attachment(mpd.as_bytes())
            .expect("fixed valid MPD fixture");
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use dlpad_core::diagnostics::MAX_REPORT_BYTES;

    #[test]
    fn fixed_fixture_uses_valid_bounded_report_api() {
        let report = stress_report();
        assert_eq!(report.logs().len(), 128);
        assert!(report.logs().iter().all(|line| line.len() == 4096));
        assert_eq!(report.mpd_attachments().len(), 8);
        assert!(
            report
                .mpd_attachments()
                .iter()
                .all(|mpd| mpd.nodes.len() == 256)
        );
        assert!(report.attachment_omissions().is_empty());
        let json = report.json_preview().unwrap();
        let markdown = report.markdown_preview();
        // Dynamic report ID/time can change a few bytes, not the workload size.
        assert!((1_800_000..1_900_000).contains(&json.len()));
        assert!((1_900_000..2_000_000).contains(&markdown.len()));
        assert!(json.len() <= MAX_REPORT_BYTES && markdown.len() <= MAX_REPORT_BYTES);
        assert!(!json.contains("https://fixture.invalid"));
    }
}
