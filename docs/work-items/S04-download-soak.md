# S04: DownloadRepository 합성 장시간·실패 복구 검증

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R04, R10, R13
- 담당/수정 범위: guardian_fault; 신규 crates/engine/tests/download_soak.rs, scripts/download_soak.py (필요 시), docs/validation/download-soak.md
- 선행 작업: DownloadRepository pause/resume/publication/SQLite 및 guardian adapter

## 사용자 결과

엔진과 DB의 실제 prepare/download/pause/resume/new Attempt/final publication을 합성 실행 파일로 연결해 장시간·단절·만료 오류를 검증한다. 실제 사이트/영상/GUI 또는 모든 OS 성공으로 표시하지 않는다.

## 계약과 범위

AGENTS→README→PRODUCT→CONTEXT→R04→ADR-0002 및 ARCHITECTURE를 읽었다. 기존 library/core/download/process/app/Cargo/schema 변경 금지. library_identity 담당 Windows 동일 snapshot 검증과 sourcehash 변경 시점을 먼저 조정한다. Cargo 실행은 주 담당 승인 신호 이후. 임시 폴더·stdlib Python synthetic executable만 사용하고 shell 실행·사용자 Documents·실제 미디어·외부 네트워크를 사용하지 않는다. 기존 guardian 8h session57344는 유지하며 새 시험이 대체하지 않는다. 기존 빈 .git은 보존한다.

## 인수 조건

- 짧은 실제 engine+DB 연결 시험이 prepare, progress, stderr bound, pause partial 보존, reopen, 동일 ID 재개·새 Attempt·final artifact 검증을 통과한다.
- 합성 네트워크 단절과 만료 URL 오류는 Failed 및 partial 보존으로 귀결되고, 재개는 원본 URL을 사용하며 새로운 실행 증거를 남긴다.
- 8h mode는 ignored/명시 env opt-in, bounded disk overwrite/progress/log 및 상태 proof를 사용한다.
- 장시간 시작/진행/최종 결과를 구분하고 8시간 실제 경과를 성공 조건으로 둔다. UI 없는 범위는 엔진+DB로 표시한다.

## 검증과 handoff

Windows snapshot 담당자에게 test source 변경 조정을 요청했다. 현재 source/빌드 변경 전 조사 단계이며 성공을 주장하지 않는다.

## 구현과 실제 실행

신규 tests/download_soak.rs와 scripts/download_soak.py 구현. 실제 short 시험 1 passed/1 long ignored,7.88초. 대상clippy/Pythoncompile 통과. 8h session76382를 별도로 시작했으며 최종완료아님. 기존guardian57344 6250초/105samples6개PIDstart 일치 확인, 계속유지. 전용 docs/validation/download-soak.md에 범위·명령·proof 경로와 실제 transport/GUI 미검증을 기록했다.

## 2026-10-02 재개 시 증거 소실

현재 boot ID794988eb-539b-437e-8396-330738b565bb에서 이전 /tmpstatus와 소유 PID가 존재하지 않는다. 위 running 관측은 당시 중간 기록이며 현재 실행 상태가 아니다. session57344/76382의 8시간 완료 결과는 미검증이다. 기존 시험을 성공으로 처리하지 않는다. 신규 실행과 영구 증거는 S04-soak-restart 작업으로 구분한다.
