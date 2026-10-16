# S04: 공식 도구 release 조회 provider

- 상태: in-progress
- 분류: feature
- 우선순위: P1
- 요구사항: R02, R08, R11
- 담당/수정 범위: installer_batch — 새 release_discovery.rs/version_actions.rs 후보; root — 모듈 연결·정책·실제 검증
- 선행 작업: fixed approved install-only/verified selector; Cargo/schema 변경 없음

## 사용자 결과

공식 도구 release 조회 provider를 기존 Windows 네이티브 흐름에 연결하고 실제 실패/표시 동작을 검증한다.

## 계약과 범위

후보는 `.local-tools/session-recovery-20261013/` 새 namespace에 보존한다. 같은 파일을 동시에 수정하지 않으며 root가 공유 계약·제품 source freeze·통합 검증을 조정한다.

## 인수 조건

모든 redirect 요청 전에 HTTPS·고정 host·authority·userinfo·port 검증, 최대4 follow와 하나의 절대 deadline/cancel/body bounds 유지. malformed checksum·version·외부 URL·소유권·실패 시 기존 active를 보존한다.

## 검증과 handoff

v2 격리 실제 Windows pure backend6 회귀는 PASS. provider hardening v3의 총11 회귀는 작성됐고 실제 컴파일/HTTP 인수 전이다. 최신 파일 notice/receipt/archive materialization은 별도 후속 구현이며 사용자 승인 차단으로 표현하지 않는다.

## 2026-10-13 실제 실행 결과

`flow-backend-v3` 최초 호출은 WSL Windows interop socket 실패로 제품 시험에 진입하지 못했다. 승인된 네이티브 실행에서 새 `flow-backend-v3-retry` 증거로 11개 회귀가 통과했고 root가 원본 로그/자연 종료/source 불변을 확인했다. fake provider 기반 리다이렉트·취소·상한 검증이며 실제 공식 HTTP 다운로드나 최신 버전 설치 성공을 뜻하지 않는다. 최신 yt-dlp 공급 materialization은 installer_batch의 별도 `tool-update-dynamic-v1` 후보로 이어서 구현한다.

## 동적 yt-dlp 공급 후보 검증

`tool-update-dynamic-v1`은 최신/지정 tag의 체크섬·고지·정확한 source commit 증거를 보존한 뒤 inactive 설치하는 capability를 구현했다. `flow-dynamic-v1` Windows 실제 회귀는 18 PASS/0 FAIL/1 ignored이며 root가 원본 로그·source/candidate 불변·소유 Job 자연 종료를 확인했다. 공식 실제 다운로드 시험 1개는 실행하지 않았다. 제품 worker 연결, 총 receipt 개수/retention 및 재시작 표시, Deno/FFmpeg 동적 공급은 아직 남았다. source 고지 수집을 전체 binary 법적 완비/독립 publisher 인증으로 보고하지 않는다. 신규 dependency/Cargo/schema 변경은 없다.
