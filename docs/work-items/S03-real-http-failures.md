# S03: 실제 도구 HTTP 실패와 같은 작업 재개

- 상태: done (synthetic actual HTTP transport; R04 전체 게이트는 남음)
- 분류: test
- 우선순위: P0
- 요구사항: R04, R10
- 담당/수정 범위: library_identity — 신규 crates/engine/tests/local_media_failures.rs, 전용 validation 문서
- 선행 작업: DownloadRepository 실패/재개 API, 실제 yt-dlp/FFmpeg opt-in 실행 환경

## 사용자 결과

만료된 미디어 HTTP 403 및 전송 중 연결 단절에서 작업을 Downloaded로 오인하지 않고, 같은 원본 URL을 다시 추출해 같은 Job ID/새 Attempt로 복구하는 동작을 실제 도구로 검증한다.

## 계약과 범위

README → PRODUCT → CONTEXT → R04/R10 → ADR-0002 및 ARCHITECTURE/INTEGRATIONS를 읽었다. 기존 local_media.rs/제품 source/Cargo/schema는 변경하지 않는다. 신규 시험은 opt-in ignored이며 DLPAD_SMOKE_YTDLP/FFMPEG의 명시 경로를 사용한다. FFmpeg 합성 fixture와 127.0.0.1 owned HTTP server만 사용한다. root 공유 코드에는 영향을 주지 않는다. shell 문자열을 실행하지 않고 process adapter/argv를 사용한다.

## 인수 조건

- 403과 짧게 끊긴 실제 HTTP 응답을 yt-dlp가 소비하고 Failed를 저장한다.
- 실패에는 artifact가 없고 Downloaded가 아니며, 전송 단절에는 비어 있지 않은 소유 partial이 유지된다.
- 원본 페이지 재추출, 같은 Job ID/새 Attempt, 정상 완료 bytes/DB reopen을 확인한다.
- HTTP 요청·fixture 크기·취소 watchdog·socket I/O·서버 종료가 bounded다.
- 실제 ENOSPC/외부 사이트/8시간/다른 OS를 이 시험으로 완료하지 않는다. 시스템 mount나 /dev/shm 채우기를 하지 않는다.

## 검증과 handoff

실제 opt-in 1 pass/24.71초 및 고정 source 재실행 1 pass/23.74초. HTTP 403과 전송 단절의 Failed/no artifact, 31,744-byte partial/실제 Range 재개, 원본 재추출/같은 ID 새 Attempt/DB 재개방을 확인했다. narrow clippy 경고 0, 기본 실행 1 ignored다. [검증 결과](../validation/local-media-failures.md)와 [원시 증거](../validation/local-media-failures.json)에 기록했다. Git 상태 확인은 정상 저장소가 아니라는 오류이며 기존 .git을 보존한다. 제품 변경은 필요하지 않았다. 실제 ENOSPC·8시간·외부 사이트·다른 OS는 별도 미검증 게이트다.
