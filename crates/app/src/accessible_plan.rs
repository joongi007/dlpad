//! Pure, bounded argv preview. This module never executes or expands arguments.
use crate::catalog as c;
use dlpad_core::options::DownloadOptions;
use std::io::{self, Write};

pub(crate) const MAX_PREVIEW_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PreviewError {
    InvalidOptions,
    NonUnicodeArgument,
    TooLarge,
    Serialization,
}

struct BoundedWriter {
    bytes: Vec<u8>,
    exceeded: bool,
}

impl Write for BoundedWriter {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if input.len() > MAX_PREVIEW_BYTES.saturating_sub(self.bytes.len()) {
            self.exceeded = true;
            return Err(io::ErrorKind::FileTooLarge.into());
        }
        self.bytes.extend_from_slice(input);
        Ok(input.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) fn preview(options: &DownloadOptions) -> Result<String, PreviewError> {
    // Reject an oversized path before the core plan clones it. JSON escaping can
    // still exceed the limit; the writer below enforces the actual output bound.
    if options.output_dir.as_os_str().len() > MAX_PREVIEW_BYTES
        || options
            .ffmpeg
            .as_ref()
            .is_some_and(|path| path.as_os_str().len() > MAX_PREVIEW_BYTES)
    {
        return Err(PreviewError::TooLarge);
    }
    let plan = options.plan().map_err(|_| PreviewError::InvalidOptions)?;
    let mut args = Vec::with_capacity(plan.args.len());
    for (index, arg) in plan.args.iter().enumerate() {
        args.push(if index + 1 == plan.args.len() {
            c::OMITTED_SOURCE
        } else {
            arg.to_str().ok_or(PreviewError::NonUnicodeArgument)?
        });
    }
    let mut writer = BoundedWriter {
        bytes: Vec::new(),
        exceeded: false,
    };
    if serde_json::to_writer_pretty(&mut writer, &args).is_err() {
        return Err(if writer.exceeded {
            PreviewError::TooLarge
        } else {
            PreviewError::Serialization
        });
    }
    String::from_utf8(writer.bytes).map_err(|_| PreviewError::Serialization)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dlpad_core::options::{AdvancedOption, Quality, SourceUrl, TimeRange};

    fn options() -> DownloadOptions {
        DownloadOptions {
            url: SourceUrl::parse(
                "https://example.invalid/watch?v=id&token=fixture-secret#private",
            )
            .unwrap(),
            output_dir: std::env::temp_dir().join("한글 공백 $literal 'quote'"),
            quality: Quality::Height(720),
            time_range: Some(TimeRange::new(1250, 9500, true).unwrap()),
            advanced: vec![AdvancedOption {
                key: "--retries".into(),
                value: "2".into(),
            }],
            ffmpeg: None,
            deno: None,
            final_name_style: None,
            subtitles: None,
        }
    }

    #[test]
    fn preview_redacts_source_and_preserves_literal_argv() {
        let options = options();
        let rendered = preview(&options).unwrap();
        let parsed: Vec<String> = serde_json::from_str(&rendered).unwrap();
        let expected = options.plan().unwrap().args;
        assert_eq!(parsed.len(), expected.len());
        for (actual, original) in parsed[..parsed.len() - 1].iter().zip(&expected) {
            assert_eq!(actual, original.to_str().unwrap());
        }
        assert_eq!(parsed.last().unwrap(), c::OMITTED_SOURCE);
        for secret in ["fixture-secret", "example.invalid", "#private", "token="] {
            assert!(!rendered.contains(secret));
        }
        assert!(SourceUrl::parse("https://user:fixture-secret@example.invalid/").is_err());
        assert_eq!(parsed[7], options.output_dir.to_str().unwrap());
    }

    #[test]
    fn invalid_policy_is_rejected_without_raw_error_output() {
        let mut options = options();
        options.quality = Quality::Height(1);
        assert_eq!(preview(&options), Err(PreviewError::InvalidOptions));
        options.quality = Quality::Best;
        options.advanced.push(AdvancedOption {
            key: "--exec".into(),
            value: "secret".into(),
        });
        assert_eq!(preview(&options), Err(PreviewError::InvalidOptions));
    }

    #[test]
    fn escaping_and_oversized_paths_cannot_exceed_output_bound() {
        let mut options = options();
        options.output_dir = "\"".repeat(MAX_PREVIEW_BYTES / 2).into();
        assert!(options.plan().is_ok());
        assert_eq!(preview(&options), Err(PreviewError::TooLarge));
        options.output_dir = "x".repeat(MAX_PREVIEW_BYTES + 1).into();
        assert_eq!(preview(&options), Err(PreviewError::TooLarge));
        options.output_dir = std::env::temp_dir();
        options.ffmpeg = Some(std::env::temp_dir().join("x".repeat(MAX_PREVIEW_BYTES + 1)));
        assert_eq!(preview(&options), Err(PreviewError::TooLarge));
        let mut writer = BoundedWriter {
            bytes: Vec::new(),
            exceeded: false,
        };
        assert_eq!(
            writer.write(&vec![b'x'; MAX_PREVIEW_BYTES]).unwrap(),
            MAX_PREVIEW_BYTES
        );
        assert!(writer.write(b"x").is_err());
        assert_eq!(writer.bytes.len(), MAX_PREVIEW_BYTES);
    }

    #[cfg(unix)]
    #[test]
    fn non_unicode_path_is_rejected_instead_of_lossy_preview() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};
        let mut options = options();
        options.output_dir = OsString::from_vec(vec![b'/', 0xff]).into();
        assert_eq!(preview(&options), Err(PreviewError::NonUnicodeArgument));
    }
}
