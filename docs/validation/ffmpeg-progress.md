# FFmpeg progress parser 검증

2026-10-01 Linux/WSL, Rust 1.94.0, 기존 외부 FFmpeg 4.4.2-0ubuntu0.22.04.1로 검증했다. 사용자 미디어·Documents·네트워크를 사용하지 않았다.

## 계약과 출처

[FFmpeg 공식 -progress 문서](https://ffmpeg.org/ffmpeg.html#toc-Main-options)는 key=value stream과 progress=continue/end record 경계를 설명한다. [FFmpeg n4.4.2 소스](https://raw.githubusercontent.com/FFmpeg/FFmpeg/n4.4.2/fftools/ffmpeg.c)의 print_report는 out_time_us와 out_time_ms 양쪽에 같은 pts를 출력한다. 이 버전의 out_time_ms는 이름과 달리 microseconds이다. 두 필드가 함께 있으면 일치해야 하며 out_time_us를 우선한다.

`metrics::ffmpeg::FfmpegProgressParser::push(bytes, callback)`는 chunk/CRLF를 처리하고 record 경계마다 Result를 발행한다. `finish(callback)`는 마지막 newline 없는 record를 flush하고 미완결 record를 오류로 처리한다. **consumer는 Err에서 이전 sample을 None으로 지우며 다음 유효 record까지 측정 불가로 표시해야 한다.** 이전 숫자를 재사용해서는 안 된다. key 순서는 자유롭지만 중요 key 중복, 시간 역행, 단위 필드 불일치, nonfinite/negative/overflow, malformed/invalid UTF-8는 해당 record를 거절한다. N/A는 unknown이며 zero는 관측값이다.

한 line 64KiB, 한 record 64행이다. parser는 한 line과 고정 numeric state만 보관하며 시간에 비례한 이력/queue를 쌓지 않는다. 추가 dependency/schema는 없다. consumer도 최신 sample 하나만 유지해야 한다. durationunknown, speedunknown/zero, 역전된 범위, ended이면 ETA None이다. progress=end는 출력 관측 종료이며 artifact 검증이나 Downloaded 상태를 대신하지 않는다.

## 실제 시험

- `rustfmt --edition 2024 crates/engine/src/metrics.rs crates/engine/src/metrics/ffmpeg.rs`: 성공.
- `cargo test -p dlpad-engine --lib metrics::ffmpeg --locked`: 기본 parser 회귀 통과. 실제 FFmpeg 시험은 환경변수 없을 때 실행하지 않는다.
- `DLPAD_TEST_FFMPEG_PROGRESS=/usr/bin/ffmpeg cargo test -p dlpad-engine --lib metrics::ffmpeg --locked -- --nocapture`: 실제 시험 포함 6개 통과, 0개 실패, 1.40초. 실제 두 record, final out_time_us=896000, speed=421x.
- `cargo clippy -p dlpad-engine --lib --locked -- -D warnings`: 성공, 6.90초.

실제 FFmpeg는 기존 소유 process adapter의 run_metadata로 argv 실행했다. `-hide_banner -loglevel error -nostdin -nostats -progress pipe:1 -f lavfi -i sine=frequency=440:sample_rate=8000 -t 1 -f null -`를 사용하며 파일을 만들지 않는다. 10초 timeout, 64KiB logs, bounded metadata stdout을 사용한다. pipe 출력을 받은 뒤 3byte chunk로 parser에 전달했다. **이 시험은 live UI/provider 연결 시험이 아니다.**

첫 실제 시험에서 final timestamp가 정확히 1000000이라는 fixture 가정이 실패했다(실제 896000). packet timestamp를 요청 duration로 단정할 수 없음을 확인하고 약 1초의 microsecond 범위 및 legacy 필드 동등성으로 수정한 뒤 통과했다. 처리 배속은 이 합성 null sink에서의 FFmpeg 관측이며 사용자 작업 성능 예산이 아니다.

## 미완료

yt-dlp가 소유한 FFmpeg 자손의 전용 machine pipe 연결, live latest/stale feed, native UI 소비는 주 담당 후속이다. 사람용 stderr와 progress를 섞어서 파싱하지 않는다. OS별 pipe/FFmpeg 버전 smoke, 실제 병합/정확 자르기 진행률, 8시간 안정성은 이번 검증에 포함되지 않는다. R01 전체 완료를 주장하지 않는다.
