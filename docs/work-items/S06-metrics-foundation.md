# S06: 제한된 실제 지표 adapter 준비

- 상태: implemented-foundation, integration-pending
- 분류: feature
- 우선순위: P0
- 요구사항: R01
- 담당/수정 범위: metrics 담당 — `crates/engine/src/metrics.rs`, 이 작업 기록
- 선행 작업: S02의 고정 progress template와 UI worker 연결; main의 Windows Storage_FileSystem feature 조정

## 사용자 결과

다운로드 B/s와 ETA, FFmpeg 처리 배속, 선택한 네트워크 interface의 수신/송신 delta, 대상 volume의 현재 사용자 이용 가능 bytes를 서로 다른 타입과 API로 관측한다. 알 수 없는 값은 None이며 0으로 대체하지 않는다. 지표만으로 인터넷 전체 정상이나 작업 성공을 판단하지 않는다. UI 연결은 주 담당 범위이므로 이 작업만으로 R01/S06 완료를 주장하지 않는다.

## 계약과 범위

AGENTS, REQUIREMENTS R01, ARCHITECTURE와 PERFORMANCE의 지표 정의를 따른다. 수정 파일과 공개 API는 다음과 같다.

- `volume_free_bytes(&Path) -> io::Result<Option<u64>>`: Unix statvfs의 f_bavail × f_frsize, Windows GetDiskFreeSpaceExW의 호출 사용자 available bytes. 실패는 io error, unsupported target/overflow는 None. 경로 NUL을 거절한다.
- `sample_network_interface(&str) -> io::Result<Option<NetworkSample>>`: Linux `/proc/net/dev`를 요청할 때만 128 KiB+1 byte 상한으로 읽는다. 기타 OS는 None. all-interface polling loop나 합산이 없다.
- `parse_network_snapshot(&str, &str, Instant)`: selected interface 한 개, 16개의 u64 counter를 검증한다. 최대 256 interface rows + header 2개, 행 64 KiB, 이름 64 byte의 restricted grammar이다.
- `NetworkSample::rate_since`: 동일 interface와 양수 monotonic interval에서 counter delta를 계산한다. reset, 역방향/동일 timestamp, interface 변경은 None. VPN과 다른 앱 traffic을 job B/s로 표시하지 않는다.
- `parse_ytdlp_progress(&str)`: 고정 JSON template의 speed/eta 숫자 필드를 읽는다. missing/null/NA/N/A/Unknown은 None. malformed JSON, negative/nonfinite/타입 불일치, 64 KiB 초과는 error. 실제 tool version/template 검증은 S02 통합 범위다.
- `parse_ffmpeg_speed(&str)`: machine `speed=...x`의 finite nonnegative f64만 허용한다. network B/s·ETA와 섞지 않는다.

Cargo·lib export·schema를 바꾸지 않았다. 추가 crate 의존성은 없다. Windows feature 확장은 주 담당이 기존 windows-sys에 조정한다. I/O와 지표 sampling은 worker에서 호출해야 하며 자동 speed test·benchmark·정기 전체 시스템 polling을 구현하지 않는다.

## 인수 조건

- 측정 0과 unknown None을 구분한다.
- counter reset·타임스탬프 변화·다른 interface는 rate를 만들지 않는다.
- FFmpeg NaN/Infinity/negative와 yt-dlp invalid numeric values를 거절한다.
- parser 크기·행·counter 수에 상한을 둔다.
- temp volume 조회는 실제 값을 반환하고 missing path는 실패한다.
- Windows/macOS/Linux 검증 결과를 섞지 않는다.

## 검증과 handoff

- 실제 실행: `/tmp/dlpad-cargo/bin/rustfmt --edition 2024 crates/engine/src/metrics.rs`: 성공.
- 실제 실행: `rustc --edition 2024 --test ... -D warnings`로 기존 /tmp workspace libc/serde_json/tempfile rlib를 직접 링크한 `/tmp/dlpad-metrics-tests`: 5 passed. 공유 Cargo/lockfile 변경이나 concurrent Cargo 호출 없음.
- 테스트: temp-only target volume/missing path, 합성 progress JSON·FFmpeg 출력, 합성 interface counters/rate/reset/time/interface 변화. 실제 전체 시스템 polling·벤치마크 없음.
- 미검증: Windows GetDiskFreeSpaceExW 빌드/런타임, macOS statvfs 런타임, 실제 yt-dlp/FFmpeg template 출력, GUI 표시. 선택 interface 관측 adapter와 UI 연결, unsupported OS 표시를 통합 담당이 완료한다.
