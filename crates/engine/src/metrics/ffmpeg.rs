//! Bounded FFmpeg `-progress` parsing, independent of process ownership and UI.
use super::{MAX_METRIC_LINE_BYTES, MetricsParseError, parse_ffmpeg_speed};

const MAX_RECORD_LINES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FfmpegMetrics {
    pub output_time_us: Option<u64>,
    pub speed: Option<f64>,
    /// End of the progress stream, never evidence of a valid completed artifact.
    pub ended: bool,
}
impl FfmpegMetrics {
    /// Estimate only for a known output duration and a measured positive speed.
    pub fn eta_seconds(self, duration_us: Option<u64>) -> Option<f64> {
        if self.ended {
            return None;
        }
        let remaining = duration_us?.checked_sub(self.output_time_us?)?;
        let speed = self.speed.filter(|s| s.is_finite() && *s > 0.0)?;
        let eta = remaining as f64 / 1_000_000.0 / speed;
        eta.is_finite().then_some(eta)
    }
}

#[derive(Debug, Default)]
struct Record {
    us: Option<Option<u64>>,
    legacy_us: Option<Option<u64>>,
    speed: Option<Option<f64>>,
    lines: usize,
    invalid: Option<MetricsParseError>,
}

/// One parser per dedicated progress pipe and per FFmpeg process. Feed only the
/// machine pipe, never merged human logs. Errors invalidate the consumer's old
/// sample; consumers must display unknown until the next successful record.
#[derive(Debug, Default)]
pub struct FfmpegProgressParser {
    line: Vec<u8>,
    dropping_line: bool,
    record: Record,
    last_time_us: Option<u64>,
    ended: bool,
}
impl FfmpegProgressParser {
    pub fn push(
        &mut self,
        bytes: &[u8],
        mut emit: impl FnMut(Result<FfmpegMetrics, MetricsParseError>),
    ) {
        for byte in bytes {
            if *byte == b'\n' {
                self.complete_line(&mut emit);
            } else if !self.dropping_line {
                if self.line.len() == MAX_METRIC_LINE_BYTES {
                    self.line.clear();
                    self.dropping_line = true;
                    self.record.invalid = Some(MetricsParseError::TooLarge);
                } else {
                    self.line.push(*byte);
                }
            }
        }
    }

    /// Flush EOF. An unfinished record is unknown, not the last valid sample.
    pub fn finish(&mut self, mut emit: impl FnMut(Result<FfmpegMetrics, MetricsParseError>)) {
        if !self.line.is_empty() || self.dropping_line {
            self.complete_line(&mut emit);
        }
        if self.record.lines != 0 || self.record.invalid.is_some() {
            emit(Err(self
                .record
                .invalid
                .unwrap_or(MetricsParseError::Malformed)));
            self.record = Record::default();
        }
    }

    fn complete_line(&mut self, emit: &mut impl FnMut(Result<FfmpegMetrics, MetricsParseError>)) {
        let bytes = std::mem::take(&mut self.line);
        if self.dropping_line {
            self.dropping_line = false;
            self.record.lines = self.record.lines.saturating_add(1);
            return;
        }
        let line = std::str::from_utf8(&bytes)
            .ok()
            .map(|s| s.strip_suffix('\r').unwrap_or(s));
        self.record.lines = self.record.lines.saturating_add(1);
        if self.record.lines > MAX_RECORD_LINES {
            self.record.invalid = Some(MetricsParseError::TooLarge);
        }
        let Some((key, value)) = line.and_then(|line| line.split_once('=')) else {
            self.record.invalid = Some(MetricsParseError::Malformed);
            return;
        };
        if key.is_empty()
            || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || value.bytes().any(|b| b.is_ascii_control())
        {
            self.record.invalid = Some(MetricsParseError::Malformed);
            return;
        }
        if self.ended {
            emit(Err(MetricsParseError::Malformed));
            self.record = Record::default();
            return;
        }
        let result = match key {
            "out_time_us" => set_time(&mut self.record.us, value),
            // Despite the name FFmpeg 4.4 emits microseconds in out_time_ms.
            "out_time_ms" => set_time(&mut self.record.legacy_us, value),
            "speed" => {
                if self.record.speed.is_some() {
                    Err(MetricsParseError::Malformed)
                } else if unknown(value) {
                    self.record.speed = Some(None);
                    Ok(())
                } else if let Some(speed) = parse_ffmpeg_speed(line.unwrap()) {
                    self.record.speed = Some(Some(speed));
                    Ok(())
                } else {
                    Err(MetricsParseError::InvalidNumber)
                }
            }
            "progress" => {
                self.publish(value, emit);
                return;
            }
            _ => Ok(()), // Forward-compatible unrelated bounded machine keys.
        };
        if let Err(error) = result {
            self.record.invalid = Some(error);
        }
    }

    fn publish(
        &mut self,
        value: &str,
        emit: &mut impl FnMut(Result<FfmpegMetrics, MetricsParseError>),
    ) {
        let record = std::mem::take(&mut self.record);
        let result = (|| {
            if let Some(error) = record.invalid {
                return Err(error);
            }
            if !matches!(value, "continue" | "end") {
                return Err(MetricsParseError::Malformed);
            }
            if let (Some(us), Some(legacy)) = (record.us, record.legacy_us)
                && us != legacy
            {
                return Err(MetricsParseError::InvalidNumber);
            }
            let time = record.us.or(record.legacy_us).flatten();
            if let (Some(previous), Some(current)) = (self.last_time_us, time)
                && current < previous
            {
                return Err(MetricsParseError::InvalidNumber);
            }
            if let Some(time) = time {
                self.last_time_us = Some(time);
            }
            self.ended = value == "end";
            Ok(FfmpegMetrics {
                output_time_us: time,
                speed: record.speed.flatten(),
                ended: self.ended,
            })
        })();
        emit(result);
    }
}

fn unknown(value: &str) -> bool {
    matches!(value.trim(), "N/A" | "NA" | "Unknown")
}
fn set_time(slot: &mut Option<Option<u64>>, value: &str) -> Result<(), MetricsParseError> {
    if slot.is_some() {
        return Err(MetricsParseError::Malformed);
    }
    *slot = Some(if unknown(value) {
        None
    } else {
        if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
            return Err(MetricsParseError::InvalidNumber);
        }
        Some(
            value
                .parse()
                .map_err(|_| MetricsParseError::InvalidNumber)?,
        )
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(input: &[u8]) -> Vec<Result<FfmpegMetrics, MetricsParseError>> {
        let mut parser = FfmpegProgressParser::default();
        let mut samples = Vec::new();
        for chunk in input.chunks(3) {
            parser.push(chunk, |sample| samples.push(sample));
        }
        parser.finish(|sample| samples.push(sample));
        samples
    }
    #[test]
    fn fragmented_crlf_reordered_fields_and_legacy_microseconds() {
        let samples = parse(b"speed= 2.5x\r\nout_time_ms=1500000\r\nout_time_us=1500000\r\nprogress=continue\r\nout_time_us=2000000\nspeed=N/A\nprogress=end");
        let first = samples[0].as_ref().unwrap();
        assert_eq!(first.output_time_us, Some(1_500_000));
        assert_eq!(first.speed, Some(2.5));
        assert_eq!(first.eta_seconds(Some(4_000_000)), Some(1.0));
        assert_eq!(first.eta_seconds(None), None);
        assert_eq!(first.eta_seconds(Some(1)), None);
        let final_sample = samples[1].as_ref().unwrap();
        assert!(final_sample.ended);
        assert_eq!(final_sample.speed, None);
        assert_eq!(final_sample.eta_seconds(Some(4_000_000)), None);
        assert_eq!(
            parse(b"out_time_ms=1000000\nprogress=continue\n")[0]
                .as_ref()
                .unwrap()
                .output_time_us,
            Some(1_000_000)
        );
    }
    #[test]
    fn unknown_zero_and_duration_are_not_invented() {
        let samples = parse(b"out_time_us=N/A\nspeed=N/A\nprogress=continue\nout_time_us=0\nspeed=0x\nprogress=continue\nprogress=continue\n");
        assert_eq!(samples[0].as_ref().unwrap().output_time_us, None);
        assert_eq!(samples[1].as_ref().unwrap().output_time_us, Some(0));
        assert_eq!(samples[1].as_ref().unwrap().speed, Some(0.0));
        assert_eq!(samples[1].as_ref().unwrap().eta_seconds(Some(10)), None);
        assert_eq!(samples[2].as_ref().unwrap().speed, None);
    }
    #[test]
    fn invalid_records_do_not_publish_old_numbers() {
        for invalid in [
            "out_time_us=-1",
            "out_time_us=18446744073709551616",
            "out_time_us=1.5",
            "speed=NaNx",
            "speed=infx",
            "speed=-2x",
            "speed=1x\nspeed=2x",
            "out_time_us=1\nout_time_us=1",
            "out_time_us=1\nout_time_ms=2",
            "broken",
            "speed=1x\u{0}",
        ] {
            let input = format!(
                "out_time_us=100\nspeed=2x\nprogress=continue\n{invalid}\nprogress=continue\nout_time_us=200\nspeed=3x\nprogress=continue\n"
            );
            let samples = parse(input.as_bytes());
            assert!(samples[0].is_ok(), "{invalid}");
            assert!(samples[1].is_err(), "{invalid}");
            assert_eq!(samples[2].as_ref().unwrap().speed, Some(3.0), "{invalid}");
        }
    }
    #[test]
    fn time_reversal_incomplete_eof_and_post_end_fail_closed() {
        let samples = parse(b"out_time_us=20\nprogress=continue\nout_time_us=10\nprogress=continue\nout_time_us=30\nprogress=end\nprogress=continue\n");
        assert!(samples[0].is_ok());
        assert_eq!(samples[1], Err(MetricsParseError::InvalidNumber));
        assert!(samples[2].as_ref().unwrap().ended);
        assert!(samples[3].is_err());
        assert_eq!(
            parse(b"out_time_us=20"),
            vec![Err(MetricsParseError::Malformed)]
        );
        assert_eq!(
            parse(&[
                0xff, b'\n', b'p', b'r', b'o', b'g', b'r', b'e', b's', b's', b'=', b'e', b'n',
                b'd', b'\n'
            ]),
            vec![Err(MetricsParseError::Malformed)]
        );
    }
    #[test]
    fn lines_and_record_counts_are_bounded_and_recover_at_boundary() {
        let mut parser = FfmpegProgressParser::default();
        let giant = vec![b'x'; MAX_METRIC_LINE_BYTES + 900];
        parser.push(&giant, |_| panic!("no boundary"));
        assert!(parser.line.len() <= MAX_METRIC_LINE_BYTES);
        let mut samples = Vec::new();
        parser.push(b"\nprogress=continue\nspeed=2x\nprogress=continue\n", |s| {
            samples.push(s)
        });
        assert_eq!(samples[0], Err(MetricsParseError::TooLarge));
        assert_eq!(samples[1].as_ref().unwrap().speed, Some(2.0));
        let input = format!(
            "{}progress=continue\n",
            "frame=1\n".repeat(MAX_RECORD_LINES)
        );
        assert_eq!(
            parse(input.as_bytes()),
            vec![Err(MetricsParseError::TooLarge)]
        );
    }
    /// Opt-in real FFmpeg smoke; no download, network, media file, or user data.
    #[test]
    fn actual_ffmpeg_machine_pipe_microsecond_units() {
        let Some(executable) = std::env::var_os("DLPAD_TEST_FFMPEG_PROGRESS") else {
            return;
        };
        let args = [
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-nostats",
            "-progress",
            "pipe:1",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=8000",
            "-t",
            "1",
            "-f",
            "null",
            "-",
        ];
        let captured = crate::process::run_metadata(
            &crate::process::ProcessSpec {
                #[cfg(windows)]
                runtime_environment: None,
                executable: executable.into(),
                args: args.into_iter().map(Into::into).collect(),
                timeout: std::time::Duration::from_secs(10),
                max_log_bytes: 64 * 1024,
            },
            &std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        )
        .unwrap();
        assert_eq!(
            captured.execution.exit_code,
            Some(0),
            "{}",
            captured.execution.logs
        );
        let samples = parse(&captured.stdout);
        assert!(samples.len() >= 2);
        assert!(samples.iter().all(Result::is_ok), "{samples:?}");
        let final_sample = samples.last().unwrap().as_ref().unwrap();
        assert!(final_sample.ended);
        assert!((800_000..=1_100_000).contains(&final_sample.output_time_us.unwrap()));
        assert!(final_sample.speed.is_some());
        let text = std::str::from_utf8(&captured.stdout).unwrap();
        assert!(
            text.contains(&format!(
                "out_time_ms={}",
                final_sample.output_time_us.unwrap()
            )),
            "{text}"
        );
        println!(
            "actual FFmpeg samples={}, output_time_us={:?}, speed={:?}",
            samples.len(),
            final_sample.output_time_us,
            final_sample.speed
        );
    }
}
