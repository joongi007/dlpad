# S01: FFmpeg 처리 배속 live feed

- 상태: partial
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R05, R10
- 담당/수정 범위: guardian_fault; metrics.rs, metrics/ffmpeg.rs, 신규 scripts/ffmpeg_progress_probe.py 및 전용 validation
- 선행 작업: bounded FFmpeg parser, ProgressFeed

## 사용자 결과

다운로드 B/s와 FFmpeg 처리 배속·조건부 ETA를 서로 독립된 최신 관측으로 제공한다. stale/미측정 숫자는 unknown이다.

## 계약과 범위

R01 및 ADR-0002/ARCHITECTURE를 따르며 기존 download/process/app/Cargo 파일 변경 금지. main이 reserved postprocessor argv와 stderr 전용 feed 연결을 담당한다. 실제 소유 합성 fixture의 yt-dlp 빠른/정확 자르기·병합 출력으로 pipe:2 수신 가능 여부를 검증한다. 사용자 Documents/실제 영상/외부 네트워크 금지, loopback temporary server만 허용. 기존 두 8h 시험을 종료/재시작하지 않는다.

## 인수 조건

- 각 stdout/stderr의 독립 parser state, frame 시작과 progress 종료를 가진 제한 record만 처리한다.
- latest processing sample·unknown·3초 stale 및 duration 없는 ETA unknown을 제공하며 download speed와 독립이다.
- 실제 FFmpeg 실행의 종료마커·microsecond·배속 및 stderr 전달을 관측한다. yt-dlp 억제로 수신 불가하면 제품 성공으로 표시하지 않고 증거/다음 interface를 보고한다.
- 실제 명령·결과와 비용·미검증OS를 전용 문서에 기록한다.

## 검증과 handoff

기존 FFmpeg 4.4.2 및 yt-dlp 2026.09.27.232945가 발견되었다. 실제 실행과 interface 합의 전 성공으로 기록하지 않는다.

## 실제 결과와 남은 gate

다운로드와 분리된 최신 processing feed/framed stderr parser 및 stale/unknown 회귀 구현. 실제 FFmpeg와 yt-dlp fast/accurate에서 stderr→Rustfeed 통과. 일반merge는 yt-dlp 내부stderr흡수로 machine record0이며 미측정을 유지한다. main이 finite downloaderargv, processstderrobserver, metadata/range duration·UI를 연결한다. 상세 실제수치/실패원리/명령은 docs/validation/processing-feed.md. 기존두8h job보존.
