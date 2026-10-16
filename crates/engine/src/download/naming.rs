//! Human final names only; staging/attempt/process identity remains the internal ID.
use super::DownloadError;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinalNameStyle {
    TitleWithJobId,
}
pub(super) fn style_from_options_json(json: &str) -> Result<Option<FinalNameStyle>, DownloadError> {
    #[derive(Deserialize)]
    struct Stored {
        #[serde(default)]
        version: Option<u8>,
        #[serde(default)]
        final_name_style: Option<FinalNameStyle>,
    }
    if json.len() > 128 * 1024 {
        return Err(DownloadError::Policy("stored filename options limit"));
    }
    let parsed: Stored = serde_json::from_str(json)?;
    if parsed.version.is_some_and(|v| v != 1)
        || (parsed.final_name_style.is_some() && parsed.version != Some(1))
    {
        return Err(DownloadError::Policy("stored filename options version"));
    }
    Ok(parsed.final_name_style)
}
pub(super) fn snapshot_argv(
    argv: &[&str],
    style: Option<FinalNameStyle>,
) -> Result<String, DownloadError> {
    #[derive(Serialize)]
    struct NamedSnapshot<'a> {
        version: u8,
        argv: &'a [&'a str],
        final_name_style: FinalNameStyle,
    }
    Ok(match style {
        None => serde_json::to_string(argv)?,
        Some(style) => serde_json::to_string(&NamedSnapshot {
            version: 2,
            argv,
            final_name_style: style,
        })?,
    })
}
const COMPONENT_LIMIT: usize = 240;
const SUFFIX_RESERVE: usize = 32;
fn reserved(name: &str) -> bool {
    let base = name
        .split('.')
        .next()
        .unwrap_or("")
        .trim_end()
        .to_ascii_uppercase();
    matches!(
        base.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$" | "CONIN$" | "CONOUT$"
    ) || ["COM", "LPT"].iter().any(|prefix| {
        base.strip_prefix(prefix).is_some_and(|tail| {
            matches!(
                tail,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    })
}
pub(super) fn stem(title: &str, id: &str) -> Result<String, DownloadError> {
    if title.len() > 4096
        || id.is_empty()
        || id.len() > 80
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
    {
        return Err(DownloadError::Policy("readable filename binding limit"));
    }
    let mut clean = String::new();
    for ch in title.chars() {
        if ch.is_control() || matches!(ch,'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}') {
            continue;
        }
        let ch = if matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') {
            '_'
        } else if ch.is_whitespace() {
            ' '
        } else {
            ch
        };
        if ch == ' ' && clean.ends_with(' ') {
            continue;
        }
        clean.push(ch);
    }
    clean = clean
        .trim_matches(|c: char| c == ' ' || c == '.')
        .to_owned();
    if clean.is_empty() {
        clean = "영상".into();
    }
    if reserved(&clean) {
        clean.insert(0, '_');
    }
    let tail = format!(" [{id}]");
    let budget = COMPONENT_LIMIT - SUFFIX_RESERVE - tail.len();
    let mut short = String::new();
    let mut units = 0;
    for ch in clean.chars() {
        if short.len() + ch.len_utf8() > budget || units + ch.len_utf16() > budget {
            break;
        }
        short.push(ch);
        units += ch.len_utf16();
    }
    short = short.trim_end_matches([' ', '.']).to_owned();
    if short.is_empty() {
        short = "영상".into();
    }
    Ok(format!("{short}{tail}"))
}
pub(super) fn suffix<'a>(id: &str, source: &'a str) -> Result<&'a str, DownloadError> {
    let tail = source
        .strip_prefix(id)
        .filter(|s| s.starts_with('.'))
        .ok_or(DownloadError::Policy("readable filename source ID"))?;
    let pieces: Vec<_> = tail[1..].split('.').collect();
    let valid = match pieces.as_slice() {
        [ext] => {
            !ext.is_empty()
                && ext.len() <= 12
                && ext.bytes().all(|b| b.is_ascii_alphanumeric())
                && !matches!(ext.to_ascii_lowercase().as_str(), "part" | "ytdl")
        }
        [language, ext] => {
            matches!(*language, "ko" | "en") && matches!(*ext, "vtt" | "srt" | "ttml")
        }
        _ => false,
    };
    if !valid || tail.len() > SUFFIX_RESERVE {
        return Err(DownloadError::Policy("readable filename artifact suffix"));
    }
    Ok(tail)
}
pub(super) fn target_name(stem: &str, id: &str, source: &str) -> Result<String, DownloadError> {
    let name = format!("{stem}{}", suffix(id, source)?);
    if name.len() > COMPONENT_LIMIT
        || name.encode_utf16().count() > COMPONENT_LIMIT
        || name.ends_with([' ', '.'])
        || reserved(&name)
    {
        return Err(DownloadError::Policy("readable filename component limit"));
    }
    Ok(name)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn readable_unicode_reserved_and_forbidden_names_preserve_id_and_suffix() {
        for title in [
            "CON",
            "con.txt",
            "COM1",
            "LPT³",
            "NUL ",
            "..",
            "한글 강의 🎬",
            "bad:/\\<>?*\"|\n\u{202e}",
        ] {
            let base = stem(title, "job_1").unwrap();
            let name = target_name(&base, "job_1", "job_1.ko.ttml").unwrap();
            assert!(name.ends_with(" [job_1].ko.ttml"));
            assert!(!name.chars().any(|c| c.is_control()
                || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')));
            assert!(!reserved(&name));
        }
        assert_eq!(
            stem("한글 강의 🎬", "job_1").unwrap(),
            "한글 강의 🎬 [job_1]"
        );
    }
    #[test]
    fn long_hangul_and_emoji_titles_fit_both_components_without_splitting_codepoints() {
        for title in ["한".repeat(1000), "🎬".repeat(1000), "x".repeat(4096)] {
            let base = stem(&title, &"x".repeat(80)).unwrap();
            let source = format!("{}.mp4", "x".repeat(80));
            let name = target_name(&base, &"x".repeat(80), &source).unwrap();
            assert!(name.len() <= 240);
            assert!(name.encode_utf16().count() <= 240);
            assert!(std::str::from_utf8(name.as_bytes()).is_ok());
        }
        assert!(stem(&"x".repeat(4097), "job").is_err());
    }
    #[test]
    fn paths_partial_foreign_ids_and_unknown_subtitle_suffixes_fail() {
        for source in [
            "other.mp4",
            "job.part",
            "job.en.ass",
            "job../file.mp4",
            "job.mp4.part",
            "job.",
            "job.ko.vtt:stream",
        ] {
            assert!(suffix("job", source).is_err());
        }
    }
    #[test]
    fn same_argv_different_naming_cannot_share_immutable_job_snapshot() {
        let argv = ["--paths", "D:\\owned", "https://example.org/fixture"];
        let legacy = snapshot_argv(&argv, None).unwrap();
        assert_eq!(legacy, serde_json::to_string(&argv).unwrap());
        let named = snapshot_argv(&argv, Some(FinalNameStyle::TitleWithJobId)).unwrap();
        assert_ne!(legacy, named);
        let mut options = serde_json::json!({"version":1});
        assert_eq!(style_from_options_json(&options.to_string()).unwrap(), None);
        options["final_name_style"] = "TitleWithJobId".into();
        let decoded = style_from_options_json(&options.to_string()).unwrap();
        assert_ne!(snapshot_argv(&argv, decoded).unwrap(), legacy);
        assert!(
            style_from_options_json(r#"{"version":2,"final_name_style":"TitleWithJobId"}"#)
                .is_err()
        );
    }
}
