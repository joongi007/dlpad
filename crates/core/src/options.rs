//! Typed, bounded download inputs. No shell tokenization or expansion.
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, path::PathBuf};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError(pub &'static str);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceUrl(String);
impl SourceUrl {
    pub fn parse(input: &str) -> Result<Self, ValidationError> {
        if input.len() > 8192 || input.chars().any(|c| c.is_control() || c.is_whitespace()) {
            return Err(ValidationError(
                "URL에는 공백 또는 제어 문자를 사용할 수 없습니다",
            ));
        }
        let rest = input
            .strip_prefix("https://")
            .or_else(|| input.strip_prefix("http://"))
            .ok_or(ValidationError("HTTP 또는 HTTPS URL이 필요합니다"))?;
        let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
        if authority.is_empty() || authority.contains('@') || authority.starts_with(':') {
            return Err(ValidationError(
                "호스트 없는 URL 또는 인증 정보가 포함된 URL입니다",
            ));
        }
        Ok(Self(input.into()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeRange {
    pub start_ms: u64,
    pub end_ms: u64,
    pub accurate: bool,
}
impl TimeRange {
    pub fn new(start_ms: u64, end_ms: u64, accurate: bool) -> Result<Self, ValidationError> {
        if end_ms <= start_ms {
            return Err(ValidationError("끝 시간은 시작 시간보다 뒤여야 합니다"));
        }
        Ok(Self {
            start_ms,
            end_ms,
            accurate,
        })
    }
    pub fn parse_time(input: &str) -> Result<u64, ValidationError> {
        let (whole, fraction) = match input.split_once('.') {
            Some((whole, fraction))
                if !fraction.is_empty()
                    && fraction.len() <= 3
                    && fraction.bytes().all(|b| b.is_ascii_digit()) =>
            {
                (whole, fraction)
            }
            Some(_) => return Err(ValidationError("밀리초는 소수점 뒤 1~3자리입니다")),
            None => (input, ""),
        };
        let parts: Vec<_> = whole.split(':').collect();
        if parts.is_empty() || parts.len() > 3 {
            return Err(ValidationError("시간 형식이 올바르지 않습니다"));
        }
        let mut seconds = 0u64;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
                return Err(ValidationError("시간은 초 또는 HH:MM:SS.mmm입니다"));
            }
            let n = part
                .parse::<u64>()
                .map_err(|_| ValidationError("시간 범위를 초과했습니다"))?;
            if i > 0 && n >= 60 {
                return Err(ValidationError("분과 초는 60 미만이어야 합니다"));
            }
            seconds = seconds
                .checked_mul(60)
                .and_then(|v| v.checked_add(n))
                .ok_or(ValidationError("시간 범위를 초과했습니다"))?;
        }
        let milliseconds = if fraction.is_empty() {
            0
        } else {
            fraction
                .parse::<u64>()
                .map_err(|_| ValidationError("밀리초 형식 오류"))?
                * 10u64.pow(3 - fraction.len() as u32)
        };
        seconds
            .checked_mul(1000)
            .and_then(|v| v.checked_add(milliseconds))
            .ok_or(ValidationError("시간 범위를 초과했습니다"))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Quality {
    Best,
    Height(u16),
    Audio,
}
impl Quality {
    /// Shared policy for form feedback and the executable plan.
    pub fn validate(self) -> Result<(), ValidationError> {
        if let Self::Height(height) = self
            && !(144..=4320).contains(&height)
        {
            return Err(ValidationError("품질 높이는 144~4320 범위입니다"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdvancedOption {
    pub key: String,
    pub value: String,
}
impl AdvancedOption {
    pub fn validate(&self) -> Result<(), ValidationError> {
        let (max, zero, message) = match self.key.as_str() {
            "--limit-rate" => (
                1_000_000_000_000u64,
                false,
                "속도 제한은 1~1000000000000 B/s 정수입니다",
            ),
            "--retries" => (20, true, "재시도는 0~20 정수입니다"),
            "--concurrent-fragments" => (16, false, "조각 동시성은 1~16 정수입니다"),
            _ => return Err(ValidationError("예약 또는 검증되지 않은 옵션입니다")),
        };
        if self.value.is_empty()
            || self.value.len() > 20
            || !self.value.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(ValidationError(message));
        }
        let value = self
            .value
            .parse::<u64>()
            .map_err(|_| ValidationError(message))?;
        if value > max || (value == 0 && !zero) {
            return Err(ValidationError(message));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubtitleLanguage {
    Korean,
    English,
    KoreanAndEnglish,
}
impl SubtitleLanguage {
    pub const ALL: [Self; 3] = [Self::Korean, Self::English, Self::KoreanAndEnglish];
    pub fn label(self) -> &'static str {
        match self {
            Self::Korean => "한국어",
            Self::English => "영어",
            Self::KoreanAndEnglish => "한국어와 영어",
        }
    }
    pub fn codes(self) -> &'static str {
        match self {
            Self::Korean => "ko",
            Self::English => "en",
            Self::KoreanAndEnglish => "ko,en",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubtitleOptions {
    pub language: SubtitleLanguage,
    pub auto_generated: bool,
}
/// Separate typed-control help: the numeric advanced-option catalog stays intact.
pub const SUBTITLE_HELP: OptionHelp = OptionHelp {
    id: "subtitles",
    label: "자막",
    summary: "선택 언어의 자막을 영상 옆 별도 파일로 저장합니다",
    example: "한국어와 영어, 자동 생성 자막은 별도 선택",
    default: "저장 안 함, 자동 생성 자막 허용 안 함",
    tradeoff: "추가 요청과 디스크 공간이 필요하며 제공되지 않은 언어는 저장되지 않습니다",
    conflicts: "구간 저장과 함께 사용할 수 없습니다; 원본 언어 자동 선택과 자막 삽입은 미지원",
    version: "yt-dlp 설치 버전별 검증되지 않음",
};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinalNameStyle {
    TitleWithJobId,
}
#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub url: SourceUrl,
    pub output_dir: PathBuf,
    pub quality: Quality,
    pub time_range: Option<TimeRange>,
    pub advanced: Vec<AdvancedOption>,
    pub ffmpeg: Option<PathBuf>,
    pub deno: Option<PathBuf>,
    pub subtitles: Option<SubtitleOptions>,
    pub final_name_style: Option<FinalNameStyle>,
}
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub args: Vec<OsString>,
}
#[derive(Debug, Clone, Copy)]
pub struct OptionHelp {
    pub id: &'static str,
    pub label: &'static str,
    pub summary: &'static str,
    pub example: &'static str,
    pub default: &'static str,
    pub tradeoff: &'static str,
    pub conflicts: &'static str,
    pub version: &'static str,
}
pub const CATALOG: &[OptionHelp] = &[
    OptionHelp {
        id: "--limit-rate",
        label: "속도 제한",
        summary: "초당 다운로드 바이트 수를 제한합니다",
        example: "1048576",
        default: "제한 없음",
        tradeoff: "다른 작업의 대역폭을 확보하지만 오래 걸립니다",
        conflicts: "없음",
        version: "검증되지 않음",
    },
    OptionHelp {
        id: "--retries",
        label: "재시도",
        summary: "네트워크 재시도 횟수를 제한합니다",
        example: "3",
        default: "3",
        tradeoff: "횟수가 크면 실패 안내가 늦어집니다",
        conflicts: "앱 재시도 정책과 별개",
        version: "검증되지 않음",
    },
    OptionHelp {
        id: "--concurrent-fragments",
        label: "조각 동시성",
        summary: "동시에 받는 조각 수를 지정합니다",
        example: "1",
        default: "1",
        tradeoff: "증가하면 서버와 디스크 부하가 커집니다",
        conflicts: "없음",
        version: "검증되지 않음",
    },
];
const STORED_OPTIONS_LIMIT: usize = 128 * 1024;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredOptions {
    version: u8,
    url: String,
    output_dir: PathBuf,
    quality: Quality,
    time_range: Option<TimeRange>,
    advanced: Vec<AdvancedOption>,
    ffmpeg: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    deno: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    subtitles: Option<SubtitleOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    final_name_style: Option<FinalNameStyle>,
}
impl DownloadOptions {
    /// Versioned local journal data; decoding always revalidates domain policy.
    pub fn to_stored_json(&self) -> Result<String, ValidationError> {
        self.plan()?;
        let value = serde_json::to_string(&StoredOptions {
            version: 1,
            url: self.url.as_str().into(),
            output_dir: self.output_dir.clone(),
            quality: self.quality,
            time_range: self.time_range,
            advanced: self.advanced.clone(),
            ffmpeg: self.ffmpeg.clone(),
            deno: self.deno.clone(),
            subtitles: self.subtitles,
            final_name_style: self.final_name_style,
        })
        .map_err(|_| ValidationError("저장할 옵션을 인코딩할 수 없습니다"))?;
        if value.len() > STORED_OPTIONS_LIMIT {
            return Err(ValidationError("저장 옵션 크기 제한을 초과했습니다"));
        }
        Ok(value)
    }
    pub fn from_stored_json(value: &str) -> Result<Self, ValidationError> {
        if value.len() > STORED_OPTIONS_LIMIT {
            return Err(ValidationError("저장 옵션 크기 제한을 초과했습니다"));
        }
        let stored: StoredOptions = serde_json::from_str(value)
            .map_err(|_| ValidationError("저장 옵션이 손상되었습니다"))?;
        if stored.version != 1 {
            return Err(ValidationError("지원하지 않는 저장 옵션 버전입니다"));
        }
        let options = Self {
            url: SourceUrl::parse(&stored.url)?,
            output_dir: stored.output_dir,
            quality: stored.quality,
            time_range: stored.time_range,
            advanced: stored.advanced,
            ffmpeg: stored.ffmpeg,
            deno: stored.deno,
            subtitles: stored.subtitles,
            final_name_style: stored.final_name_style,
        };
        options.plan()?;
        Ok(options)
    }

    pub fn plan(&self) -> Result<ExecutionPlan, ValidationError> {
        if self.output_dir.as_os_str().is_empty() {
            return Err(ValidationError("저장 위치가 필요합니다"));
        }
        if self.advanced.len() > 32 {
            return Err(ValidationError("옵션 개수 제한을 초과했습니다"));
        }
        let mut args: Vec<OsString> = [
            "--ignore-config",
            "--no-plugin-dirs",
            "--newline",
            "--no-playlist",
            "--retries",
            "3",
            "--paths",
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        args.push(self.output_dir.clone().into_os_string());
        self.quality.validate()?;
        args.extend([
            "--format".into(),
            match self.quality {
                Quality::Best => "bv*+ba/b".into(),
                Quality::Audio => "bestaudio/best".into(),
                Quality::Height(h) => format!("bv*[height<={h}]+ba/b[height<={h}]").into(),
            },
        ]);
        if let Some(path) = &self.ffmpeg {
            let value = path
                .to_str()
                .ok_or(ValidationError("FFmpeg 경로는 Unicode여야 합니다"))?;
            if !path.is_absolute()
                || value.len() > 32768
                || value.chars().any(char::is_control)
                || path
                    .components()
                    .any(|part| matches!(part, std::path::Component::ParentDir))
            {
                return Err(ValidationError("FFmpeg 실행 파일의 절대 경로를 입력하세요"));
            }
            args.extend(["--ffmpeg-location".into(), path.clone().into_os_string()]);
        }
        if let Some(path) = &self.deno {
            let value = path
                .to_str()
                .ok_or(ValidationError("Deno 경로는 Unicode여야 합니다"))?;
            if !path.is_absolute()
                || value.len() > 32768
                || value.chars().any(char::is_control)
                || path
                    .components()
                    .any(|part| matches!(part, std::path::Component::ParentDir))
            {
                return Err(ValidationError("Deno 실행 파일의 절대 경로를 입력하세요"));
            }
            let mut runtime = OsString::from("deno:");
            runtime.push(path.as_os_str());
            args.extend([
                OsString::from("--no-js-runtimes"),
                OsString::from("--js-runtimes"),
                runtime,
            ]);
        }
        if let Some(subtitles) = self.subtitles {
            if self.time_range.is_some() {
                return Err(ValidationError(
                    "자막 저장은 현재 전체 영상에서만 사용할 수 있습니다",
                ));
            }
            args.extend([
                "--write-subs".into(),
                "--sub-langs".into(),
                subtitles.language.codes().into(),
                "--sub-format".into(),
                "vtt/best".into(),
            ]);
            if subtitles.auto_generated {
                args.push("--write-auto-subs".into());
            }
        }
        if let Some(r) = self.time_range {
            TimeRange::new(r.start_ms, r.end_ms, r.accurate)?;
            args.extend([
                "--download-sections".into(),
                format!(
                    "*{}.{:03}-{}.{:03}",
                    r.start_ms / 1000,
                    r.start_ms % 1000,
                    r.end_ms / 1000,
                    r.end_ms % 1000
                )
                .into(),
            ]);
            if r.accurate {
                args.push("--force-keyframes-at-cuts".into());
            }
        }
        let mut seen = std::collections::HashSet::new();
        for option in &self.advanced {
            if !seen.insert(&option.key) {
                return Err(ValidationError("중복 옵션입니다"));
            }
            option.validate()?;
            // Explicit typed values replace the app default with no silent duplicate.
            if option.key == "--retries" {
                args[5] = option.value.clone().into();
            } else {
                args.extend([option.key.clone().into(), option.value.clone().into()]);
            }
        }
        args.extend(["--".into(), self.url.as_str().into()]);
        Ok(ExecutionPlan { args })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn options() -> DownloadOptions {
        DownloadOptions {
            url: SourceUrl::parse("https://example.org/한글?q='quoted'").unwrap(),
            output_dir: "C:\\한글 폴더\\$literal".into(),
            quality: Quality::Best,
            time_range: None,
            advanced: vec![],
            ffmpeg: None,
            deno: None,
            final_name_style: None,
            subtitles: None,
        }
    }
    #[test]
    fn typed_subtitles_are_finite_explicit_and_roundtrip() {
        for language in SubtitleLanguage::ALL {
            for auto_generated in [false, true] {
                let mut selected = options();
                selected.subtitles = Some(SubtitleOptions {
                    language,
                    auto_generated,
                });
                let args = selected.plan().unwrap().args;
                let index = args.iter().position(|a| a == "--sub-langs").unwrap();
                assert_eq!(args[index + 1], language.codes());
                assert_eq!(args.iter().filter(|a| *a == "--write-subs").count(), 1);
                assert_eq!(
                    args.iter().filter(|a| *a == "--write-auto-subs").count(),
                    usize::from(auto_generated)
                );
                let format = args.iter().position(|a| a == "--sub-format").unwrap();
                assert_eq!(args[format + 1], "vtt/best");
                assert_eq!(args[args.len() - 2], "--");
                let stored = selected.to_stored_json().unwrap();
                let restored = DownloadOptions::from_stored_json(&stored).unwrap();
                assert_eq!(restored.subtitles, selected.subtitles);
                assert_eq!(restored.plan().unwrap().args, args);
            }
        }
        for help in [SUBTITLE_HELP] {
            for field in [
                help.label,
                help.summary,
                help.example,
                help.default,
                help.tradeoff,
                help.conflicts,
                help.version,
            ] {
                assert!(!field.is_empty());
                assert!(field.chars().any(|c| ('가'..='힣').contains(&c)));
            }
        }
    }
    #[test]
    fn legacy_subtitle_default_preserves_snapshot_and_invalid_contracts_fail() {
        let baseline = options();
        let stored = baseline.to_stored_json().unwrap();
        let value: serde_json::Value = serde_json::from_str(&stored).unwrap();
        assert!(value.get("subtitles").is_none());
        assert_eq!(
            DownloadOptions::from_stored_json(&stored)
                .unwrap()
                .subtitles,
            None
        );
        assert_eq!(
            DownloadOptions::from_stored_json(&stored)
                .unwrap()
                .plan()
                .unwrap()
                .args,
            baseline.plan().unwrap().args
        );
        for invalid in [
            serde_json::json!({"language":"all","auto_generated":false}),
            serde_json::json!({"language":"Korean","auto_generated":"yes"}),
            serde_json::json!({"language":"Korean","auto_generated":false,"exec":"evil"}),
            serde_json::json!({"language":"Korean"}),
        ] {
            let mut altered = value.clone();
            altered["subtitles"] = invalid;
            assert!(DownloadOptions::from_stored_json(&altered.to_string()).is_err());
        }
        let mut cut = options();
        cut.subtitles = Some(SubtitleOptions {
            language: SubtitleLanguage::Korean,
            auto_generated: false,
        });
        cut.time_range = Some(TimeRange::new(0, 1000, false).unwrap());
        assert!(cut.plan().is_err());
        for key in [
            "--write-subs",
            "--sub-langs",
            "--sub-format",
            "--write-auto-subs",
            "--embed-subs",
        ] {
            let mut selected = options();
            selected.advanced.push(AdvancedOption {
                key: key.into(),
                value: "1".into(),
            });
            assert!(selected.plan().is_err());
        }
    }
    #[test]
    fn stored_options_revalidate_policy_and_preserve_argv() {
        let mut original = options();
        original.time_range = Some(TimeRange::new(100, 900, true).unwrap());
        original.advanced.push(AdvancedOption {
            key: "--retries".into(),
            value: "2".into(),
        });
        let json = original.to_stored_json().unwrap();
        assert_eq!(
            DownloadOptions::from_stored_json(&json)
                .unwrap()
                .plan()
                .unwrap()
                .args,
            original.plan().unwrap().args
        );
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        for (key, replacement) in [
            ("url", serde_json::json!("file:///secret")),
            ("version", serde_json::json!(2)),
            (
                "advanced",
                serde_json::json!([{"key":"--exec","value":"1"}]),
            ),
            (
                "time_range",
                serde_json::json!({"start_ms":900,"end_ms":100,"accurate":true}),
            ),
            ("unexpected", serde_json::json!(true)),
        ] {
            let mut altered = value.clone();
            altered[key] = replacement;
            assert!(DownloadOptions::from_stored_json(&altered.to_string()).is_err());
        }
        assert!(DownloadOptions::from_stored_json(&" ".repeat(STORED_OPTIONS_LIMIT + 1)).is_err());
    }
    #[test]
    fn typed_option_boundaries_and_non_decimal_values() {
        for (key, min, max) in [
            ("--limit-rate", 1, 1_000_000_000_000u64),
            ("--retries", 0, 20),
            ("--concurrent-fragments", 1, 16),
        ] {
            for value in [min.to_string(), max.to_string()] {
                assert!(
                    AdvancedOption {
                        key: key.into(),
                        value
                    }
                    .validate()
                    .is_ok()
                );
            }
            for value in [
                String::new(),
                "+1".into(),
                "-1".into(),
                " 1".into(),
                "1.0".into(),
                "١".into(),
                "18446744073709551616".into(),
                (max + 1).to_string(),
            ] {
                assert!(
                    AdvancedOption {
                        key: key.into(),
                        value
                    }
                    .validate()
                    .is_err()
                );
            }
            if min > 0 {
                assert!(
                    AdvancedOption {
                        key: key.into(),
                        value: "0".into()
                    }
                    .validate()
                    .is_err()
                );
            }
        }
    }
    #[test]
    fn explicit_ffmpeg_is_one_literal_argument_and_reserved_raw_override_stays_blocked() {
        let mut options = options();
        let directory = std::env::temp_dir();
        let selected = directory
            .join("한글 FFmpeg $literal 'quote'")
            .join("ffmpeg");
        options.ffmpeg = Some(selected.clone());
        let args = options.plan().unwrap().args;
        let index = args
            .iter()
            .position(|arg| arg == "--ffmpeg-location")
            .unwrap();
        assert_eq!(args[index + 1], selected.as_os_str());
        for path in [
            PathBuf::from("relative/ffmpeg"),
            directory.join("../ffmpeg"),
            directory.join("ffmpeg\n"),
        ] {
            options.ffmpeg = Some(path);
            assert!(options.plan().is_err());
        }
        options.ffmpeg = None;
        options.advanced.push(AdvancedOption {
            key: "--ffmpeg-location".into(),
            value: "1".into(),
        });
        assert!(options.plan().is_err());
    }
    #[test]
    fn argv_preserves_paths_and_quotes() {
        let o = options();
        let p = o.plan().unwrap();
        assert_eq!(p.args[7], o.output_dir.as_os_str());
        assert_eq!(p.args.last().unwrap(), o.url.as_str());
        assert_eq!(p.args[p.args.len() - 2], "--");
    }
    #[test]
    fn forbidden_controls() {
        for key in [
            "--exec",
            "--config-locations",
            "--plugin-dirs",
            "-o",
            "--print",
            "--proxy",
            "--",
        ] {
            let mut o = options();
            o.advanced.push(AdvancedOption {
                key: key.into(),
                value: "1".into(),
            });
            assert!(o.plan().is_err());
        }
        for u in [
            "file:///x",
            "--exec",
            "https://",
            "https://user:secret@example.org",
            "https://host/a\n",
        ] {
            assert!(SourceUrl::parse(u).is_err());
        }
    }
    #[test]
    fn ranges_and_overflow() {
        assert_eq!(TimeRange::parse_time("08:00:00").unwrap(), 28_800_000);
        for t in ["1:60:00", "-1", "1.0001", "18446744073709551615"] {
            assert!(TimeRange::parse_time(t).is_err());
        }
        assert!(TimeRange::new(1, 1, false).is_err());
    }
    #[test]
    fn duplicates_and_bounds() {
        let mut o = options();
        o.advanced = vec![
            AdvancedOption {
                key: "--retries".into(),
                value: "2".into()
            };
            2
        ];
        assert!(o.plan().is_err());
        o.advanced.pop();
        assert_eq!(o.plan().unwrap().args[5], "2");
        o.advanced[0].value = "infinite".into();
        assert!(o.plan().is_err());
    }
}

#[cfg(test)]
mod fractional_tests {
    use super::tests::options;
    use super::*;
    #[test]
    fn milliseconds() {
        assert_eq!(TimeRange::parse_time("00:00:00.001").unwrap(), 1);
        assert_eq!(TimeRange::parse_time("00:00:01.999").unwrap(), 1999);
        assert_eq!(TimeRange::parse_time("1.5").unwrap(), 1500);
        for v in ["1.", ".001", "1.0001", "1.1.1", "18446744073709552.999"] {
            assert!(TimeRange::parse_time(v).is_err());
        }
    }
    #[test]
    fn deno_old_snapshot_and_selected_roundtrip_preserve_policy() {
        let original = options();
        let old = original.to_stored_json().unwrap();
        assert!(!old.contains("\"deno\""));
        let restored = DownloadOptions::from_stored_json(&old).unwrap();
        assert!(restored.deno.is_none());
        assert_eq!(restored.plan().unwrap().args, original.plan().unwrap().args);
        let mut selected = original;
        selected.deno = Some(std::env::temp_dir().join("한글 runtime").join("deno"));
        let encoded = selected.to_stored_json().unwrap();
        assert_eq!(
            DownloadOptions::from_stored_json(&encoded)
                .unwrap()
                .plan()
                .unwrap()
                .args,
            selected.plan().unwrap().args
        );
    }
    #[test]
    fn deno_path_is_one_literal_unicode_argument() {
        let mut selected = options();
        let path = std::env::temp_dir()
            .join("한글 $literal 'quote' runtime")
            .join("deno");
        selected.deno = Some(path.clone());
        let args = selected.plan().unwrap().args;
        let index = args
            .iter()
            .position(|arg| arg == "--no-js-runtimes")
            .unwrap();
        assert_eq!(args[index + 1], "--js-runtimes");
        let mut literal = OsString::from("deno:");
        literal.push(path.as_os_str());
        assert_eq!(args[index + 2], literal);
        assert_eq!(args.last().unwrap(), selected.url.as_str());
        #[cfg(windows)]
        {
            selected.deno = Some(PathBuf::from(r"C:\한글 폴더\deno.exe"));
            assert!(
                selected
                    .plan()
                    .unwrap()
                    .args
                    .iter()
                    .any(|arg| arg == r"deno:C:\한글 폴더\deno.exe")
            );
        }
    }
    #[test]
    fn deno_invalid_paths_and_raw_runtime_overrides_are_rejected() {
        let mut selected = options();
        for path in [
            PathBuf::from("relative/deno"),
            std::env::temp_dir().join("../deno"),
            std::env::temp_dir().join("deno\n"),
            std::env::temp_dir().join("x".repeat(32769)),
        ] {
            selected.deno = Some(path);
            assert!(selected.plan().is_err());
        }
        selected.deno = None;
        for key in ["--js-runtimes", "--no-js-runtimes", "--remote-components"] {
            selected.advanced = vec![AdvancedOption {
                key: key.into(),
                value: "1".into(),
            }];
            assert!(selected.plan().is_err());
        }
    }
}

#[cfg(test)]
mod final_name_style_tests {
    use super::*;
    #[test]
    fn legacy_snapshot_new_policy_and_unknown_style_preserve_exact_execution_plan() {
        let original = super::tests::options();
        let old = original.to_stored_json().unwrap();
        assert!(!old.contains("final_name_style"));
        assert!(
            DownloadOptions::from_stored_json(&old)
                .unwrap()
                .final_name_style
                .is_none()
        );
        let mut named = original.clone();
        named.final_name_style = Some(FinalNameStyle::TitleWithJobId);
        let json = named.to_stored_json().unwrap();
        let restored = DownloadOptions::from_stored_json(&json).unwrap();
        assert_eq!(restored.final_name_style, named.final_name_style);
        assert_eq!(restored.plan().unwrap().args, original.plan().unwrap().args);
        for unknown in ["Other", "../../escape", "title"] {
            let mut invalid: serde_json::Value = serde_json::from_str(&json).unwrap();
            invalid["final_name_style"] = unknown.into();
            assert!(DownloadOptions::from_stored_json(&invalid.to_string()).is_err());
        }
    }
}
