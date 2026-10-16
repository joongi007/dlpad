//! Validate only explicitly requested sidecar metadata; never scan arbitrary files.
use dlpad_core::options::SubtitleOptions;
use serde::{
    Deserialize,
    de::{self, MapAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs, io,
    path::{Component, Path, PathBuf},
};

const MAX_EVIDENCE_BYTES: usize = 128 * 1024;
const MAX_SUBTITLE_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug)]
pub(super) enum SubtitleError {
    Policy(&'static str),
    Metadata(serde_json::Error),
    Io(io::Error),
}
impl fmt::Display for SubtitleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Policy(message) => f.write_str(message),
            Self::Metadata(error) => write!(f, "subtitle metadata: {error}"),
            Self::Io(error) => write!(f, "subtitle filesystem: {error}"),
        }
    }
}
impl std::error::Error for SubtitleError {}
impl From<io::Error> for SubtitleError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct SubtitleStagingEvidence {
    pub path: PathBuf,
    pub bytes: u64,
    pub language: String,
    pub extension: String,
}
#[derive(Deserialize)]
struct Metadata {
    id: String,
    #[serde(default, deserialize_with = "requested")]
    requested_subtitles: BTreeMap<String, Requested>,
}
#[derive(Deserialize)]
struct Requested {
    #[serde(default, alias = "filepath")]
    filename: String,
    #[serde(default)]
    ext: String,
}
fn requested<'de, D: de::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Requested>, D::Error> {
    struct Requests;
    impl<'de> Visitor<'de> for Requests {
        type Value = BTreeMap<String, Requested>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most two requested subtitles or null")
        }
        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(BTreeMap::new())
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut result = BTreeMap::new();
            let mut seen = BTreeSet::new();
            while let Some(language) = map.next_key::<String>()? {
                if seen.len() == 2
                    || !matches!(language.as_str(), "ko" | "en")
                    || !seen.insert(language.clone())
                {
                    return Err(de::Error::custom(
                        "unsupported, duplicate, or excessive subtitle language",
                    ));
                }
                let Some(value) = map.next_value::<Option<Requested>>()? else {
                    continue;
                };
                if value.filename.is_empty() && value.ext.is_empty() {
                    continue;
                }
                if value.filename.len() > 32768 || value.ext.len() > 4 {
                    return Err(de::Error::custom("subtitle field limit exceeded"));
                }
                result.insert(language, value);
            }
            Ok(result)
        }
    }
    deserializer.deserialize_any(Requests)
}

/// Source ID is observed metadata for the selected URL, not a cryptographic ID.
/// Returned candidates require create-only publication and identity revalidation
/// by the caller before they can become final artifacts.
pub(super) fn collect_staging_subtitles(
    staging_root: &Path,
    job_id: &str,
    source_id: &str,
    selected: SubtitleOptions,
    evidence_json: &[u8],
) -> Result<Vec<SubtitleStagingEvidence>, SubtitleError> {
    if evidence_json.len() > MAX_EVIDENCE_BYTES {
        return Err(SubtitleError::Policy("subtitle metadata limit exceeded"));
    }
    if job_id.is_empty()
        || job_id.len() > 128
        || !job_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
    {
        return Err(SubtitleError::Policy("invalid subtitle job identity"));
    }
    if source_id.is_empty() || source_id.len() > 256 || source_id.chars().any(char::is_control) {
        return Err(SubtitleError::Policy("invalid subtitle source identity"));
    }
    let metadata: Metadata =
        serde_json::from_slice(evidence_json).map_err(SubtitleError::Metadata)?;
    if metadata.id != source_id {
        return Err(SubtitleError::Policy("subtitle source identity mismatch"));
    }
    let root_metadata = fs::symlink_metadata(staging_root)?;
    if !staging_root.is_absolute()
        || !root_metadata.is_dir()
        || root_metadata.file_type().is_symlink()
        || fs::canonicalize(staging_root)? != staging_root
    {
        return Err(SubtitleError::Policy(
            "subtitle staging root must be canonical directory",
        ));
    }
    let mut candidates = Vec::with_capacity(metadata.requested_subtitles.len());
    for (language, subtitle) in metadata.requested_subtitles {
        if !selected
            .language
            .codes()
            .split(',')
            .any(|code| code == language)
        {
            return Err(SubtitleError::Policy("subtitle language was not selected"));
        }
        if !matches!(subtitle.ext.as_str(), "vtt" | "srt" | "ttml") {
            return Err(SubtitleError::Policy("unsupported subtitle extension"));
        }
        let path = PathBuf::from(subtitle.filename);
        let expected = format!("{job_id}.{language}.{}", subtitle.ext);
        if !path.is_absolute()
            || path.parent() != Some(staging_root)
            || path.file_name().and_then(|n| n.to_str()) != Some(expected.as_str())
            || path
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
            || path
                .to_str()
                .is_none_or(|s| s.chars().any(char::is_control))
        {
            return Err(SubtitleError::Policy(
                "subtitle path is not its owned direct staging path",
            ));
        }
        let observed = fs::symlink_metadata(&path)?;
        if !observed.is_file()
            || observed.file_type().is_symlink()
            || observed.len() == 0
            || observed.len() > MAX_SUBTITLE_BYTES
        {
            return Err(SubtitleError::Policy(
                "subtitle must be regular, nonempty, and within size limit",
            ));
        }
        if fs::canonicalize(&path)? != path {
            return Err(SubtitleError::Policy("subtitle canonical path mismatch"));
        }
        candidates.push(SubtitleStagingEvidence {
            path,
            bytes: observed.len(),
            language,
            extension: subtitle.ext,
        });
    }
    Ok(candidates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dlpad_core::options::SubtitleLanguage;
    fn selected(language: SubtitleLanguage) -> SubtitleOptions {
        SubtitleOptions {
            language,
            auto_generated: false,
        }
    }
    fn evidence(path: &Path, extension: &str, language: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "id": "source", "requested_subtitles": {
                language: {"filename": path, "ext": extension}
            }
        }))
        .unwrap()
    }
    #[test]
    fn selected_source_and_owned_files_yield_staging_evidence_only() {
        let temp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temp.path()).unwrap();
        let path = root.join("job.ko.vtt");
        fs::write(&path, b"WEBVTT\nsynthetic").unwrap();
        let found = collect_staging_subtitles(
            &root,
            "job",
            "source",
            selected(SubtitleLanguage::Korean),
            &evidence(&path, "vtt", "ko"),
        )
        .unwrap();
        assert_eq!(
            found,
            vec![SubtitleStagingEvidence {
                path,
                bytes: 16,
                language: "ko".into(),
                extension: "vtt".into()
            }]
        );
    }
    #[test]
    fn missing_requested_metadata_is_empty_not_a_directory_scan() {
        let temp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temp.path()).unwrap();
        fs::write(root.join("job.ko.vtt"), b"unclaimed file").unwrap();
        for input in [
            br#"{"id":"source"}"#.as_slice(),
            br#"{"id":"source","requested_subtitles":null}"#,
        ] {
            assert!(
                collect_staging_subtitles(
                    &root,
                    "job",
                    "source",
                    selected(SubtitleLanguage::Korean),
                    input
                )
                .unwrap()
                .is_empty()
            );
        }
        assert!(
            collect_staging_subtitles(
                &root,
                "job",
                "source",
                selected(SubtitleLanguage::Korean),
                b"{}"
            )
            .is_err()
        );
    }
    #[test]
    fn both_languages_are_bounded_and_source_selection_is_required() {
        let temp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temp.path()).unwrap();
        let korean = root.join("job.ko.vtt");
        let english = root.join("job.en.srt");
        fs::write(&korean, b"synthetic ko").unwrap();
        fs::write(&english, b"synthetic en").unwrap();
        let json = serde_json::to_vec(&serde_json::json!({"id":"source","requested_subtitles":{
            "ko":{"filename":korean,"ext":"vtt"},"en":{"filename":english,"ext":"srt"}}}))
        .unwrap();
        assert_eq!(
            collect_staging_subtitles(
                &root,
                "job",
                "source",
                selected(SubtitleLanguage::KoreanAndEnglish),
                &json
            )
            .unwrap()
            .len(),
            2
        );
        assert!(
            collect_staging_subtitles(
                &root,
                "job",
                "different",
                selected(SubtitleLanguage::KoreanAndEnglish),
                &json
            )
            .is_err()
        );
        assert!(
            collect_staging_subtitles(
                &root,
                "job",
                "source",
                selected(SubtitleLanguage::Korean),
                &json
            )
            .is_err()
        );
        for json in [
            br#"{"id":"source","requested_subtitles":{"ko":{"filename":"x","ext":"vtt"},"ko":{"filename":"x","ext":"vtt"}}}"#.as_slice(),
            br#"{"id":"source","requested_subtitles":{"all":{"filename":"x","ext":"vtt"}}}"#,
            br#"{"id":"source","requested_subtitles":[]}"#,
            br#"{"id":"source","id":"other"}"#,
        ] {
            assert!(collect_staging_subtitles(&root, "job", "source", selected(SubtitleLanguage::KoreanAndEnglish), json).is_err());
        }
    }
    #[test]
    fn unknown_extension_empty_directory_and_oversize_fail_closed() {
        let temp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temp.path()).unwrap();
        for (name, extension) in [("job.ko.ass", "ass"), ("job.ko.vtt", "vtt")] {
            let path = root.join(name);
            fs::write(&path, []).unwrap();
            assert!(
                collect_staging_subtitles(
                    &root,
                    "job",
                    "source",
                    selected(SubtitleLanguage::Korean),
                    &evidence(&path, extension, "ko")
                )
                .is_err()
            );
        }
        let huge = root.join("job.ko.srt");
        fs::File::create(&huge)
            .unwrap()
            .set_len(MAX_SUBTITLE_BYTES + 1)
            .unwrap();
        assert!(
            collect_staging_subtitles(
                &root,
                "job",
                "source",
                selected(SubtitleLanguage::Korean),
                &evidence(&huge, "srt", "ko")
            )
            .is_err()
        );
        let directory = root.join("job.ko.ttml");
        fs::create_dir(&directory).unwrap();
        assert!(
            collect_staging_subtitles(
                &root,
                "job",
                "source",
                selected(SubtitleLanguage::Korean),
                &evidence(&directory, "ttml", "ko")
            )
            .is_err()
        );
        assert!(
            collect_staging_subtitles(
                &root,
                "job",
                "source",
                selected(SubtitleLanguage::Korean),
                &vec![b' '; MAX_EVIDENCE_BYTES + 1]
            )
            .is_err()
        );
    }
    #[test]
    fn supported_formats_size_boundary_and_field_limits() {
        let temp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temp.path()).unwrap();
        for extension in ["vtt", "srt", "ttml"] {
            let path = root.join(format!("job.ko.{extension}"));
            fs::File::create(&path)
                .unwrap()
                .set_len(MAX_SUBTITLE_BYTES)
                .unwrap();
            let found = collect_staging_subtitles(
                &root,
                "job",
                "source",
                selected(SubtitleLanguage::Korean),
                &evidence(&path, extension, "ko"),
            )
            .unwrap();
            assert_eq!(found[0].bytes, MAX_SUBTITLE_BYTES);
        }
        let enormous_path = PathBuf::from("x".repeat(32769));
        assert!(
            collect_staging_subtitles(
                &root,
                "job",
                "source",
                selected(SubtitleLanguage::Korean),
                &evidence(&enormous_path, "vtt", "ko")
            )
            .is_err()
        );
        assert!(
            collect_staging_subtitles(
                &root,
                "job",
                &"s".repeat(257),
                selected(SubtitleLanguage::Korean),
                br#"{"id":"source"}"#
            )
            .is_err()
        );
    }

    #[test]
    fn arbitrary_relative_outside_and_nested_paths_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temp.path()).unwrap();
        let nested = root.join("nested");
        fs::create_dir(&nested).unwrap();
        let other = tempfile::tempdir().unwrap();
        for path in [
            PathBuf::from("job.ko.vtt"),
            root.join("other.ko.vtt"),
            nested.join("job.ko.vtt"),
            other.path().join("job.ko.vtt"),
        ] {
            if path.is_absolute() {
                fs::write(&path, b"synthetic").unwrap();
            }
            assert!(
                collect_staging_subtitles(
                    &root,
                    "job",
                    "source",
                    selected(SubtitleLanguage::Korean),
                    &evidence(&path, "vtt", "ko")
                )
                .is_err()
            );
        }
        assert!(
            collect_staging_subtitles(
                &root,
                "../job",
                "source",
                selected(SubtitleLanguage::Korean),
                br#"{"id":"source"}"#
            )
            .is_err()
        );
    }
    #[cfg(unix)]
    #[test]
    fn symlink_file_and_root_do_not_claim_external_file() {
        let temp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temp.path()).unwrap();
        let external = tempfile::tempdir().unwrap();
        let target = external.path().join("target.vtt");
        fs::write(&target, b"preserved outside fixture").unwrap();
        let linked = root.join("job.ko.vtt");
        std::os::unix::fs::symlink(&target, &linked).unwrap();
        assert!(
            collect_staging_subtitles(
                &root,
                "job",
                "source",
                selected(SubtitleLanguage::Korean),
                &evidence(&linked, "vtt", "ko")
            )
            .is_err()
        );
        let linked_root = root.join("link-root");
        std::os::unix::fs::symlink(external.path(), &linked_root).unwrap();
        assert!(
            collect_staging_subtitles(
                &linked_root,
                "job",
                "source",
                selected(SubtitleLanguage::Korean),
                br#"{"id":"source"}"#
            )
            .is_err()
        );
        assert_eq!(fs::read(&target).unwrap(), b"preserved outside fixture");
    }
}
