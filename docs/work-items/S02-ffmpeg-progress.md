# S02: FFmpeg machine progress parser

- 상태: implemented
- 분류: feature
- 우선순위: P0
- 요구사항: R01
- 담당/수정 범위: library_identity 에이전트; crates/engine/src/metrics.rs, crates/engine/src/metrics/ffmpeg.rs, 본 문서, docs/validation/ffmpeg-progress.md
- 선행 작업: S02 bounded metrics, 외부 FFmpeg 선택; live provider 연결은 주 담당 후속

## 사용자 결과

후처리 배속과 출력 미디어 시간을 다운로드 속도와 구분할 수 있는 provider 계약을 준비한다. duration을 모르면 ETA를 만들지 않는다. 이번 작업만으로 앱 UI가 실시간 FFmpeg 관측을 표시한다고 주장하지 않는다.

## 계약과 범위

FFmpeg 공식 progress key=value stream을 progress=continue/end 경계에서만 sample로 발행한다. line 64KiB, record 64행 상한과 callback 스트리밍을 사용한다. shell 문자열, process/download/UI/schema/Cargo 변경은 없다. FFmpeg 실제 시험은 기존 소유 process adapter와 합성 lavfi/null output만 사용한다. Git 저장소 부재로 branch/worktree 불가, 기존 .git 보존.

## 인수 조건

- chunk 분할, CRLF, 마지막 newline 없음과 중요한 key 순서 차이를 처리한다.
- N/A, malformed, overflow, duplicate, 시간 역행, 긴 line/record는 오래된 관측을 새 값으로 발행하지 않는다.
- out_time_us와 legacy out_time_ms 단위 차이를 실제 FFmpeg 4.4.2로 검증한다.
- durationunknown 또는 배속unknown/zero이면 ETA None, progress=end로 완료 파일을 추정하지 않는다.
- actual pipe 검증과 live UI provider 연결 미완료를 구분한다.

## 검증과 handoff

FFmpeg 4.4.2 합성 machine pipe 포함 parser 시험 6개 통과(1.40초), engine lib clippy 통과. [검증 기록](../validation/ffmpeg-progress.md)에 실제 명령, 최초 fixture 가정 수정, 단위·unknown·오류 계약을 기록했다. Linux 외 OS, yt-dlp 자손 FFmpeg live pipe와 UI 연결은 미검증이다. 통합 담당에게 인계한다.
