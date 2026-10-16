# S03: Owned disk writer failure and resume

- 상태: completed (bounded writer regression; actual ENOSPC gate remains open)
- 분류: regression
- 우선순위: P0
- 요구사항: R04, R10
- 담당/수정 범위: guardian_fault; 신규 crates/engine/tests/disk_failure.rs, 전용 validation/workitem
- 선행 작업: native process guardian, DownloadRepository failure/resume; Windows snapshot 완료 후 신규 Rust 파일 작성

## 사용자 결과

다운로드 writer 실패 시 실패 이력과 partial을 유지하고, DB 재열기 후 같은 작업의 새 Attempt에서 재개되는지 검증한다.

## 계약과 범위

README→PRODUCT→CONTEXT→REQUIREMENTS R04 및 ADR0002/0003, ARCHITECTURE 계약 유지. 제품/Cargo/schema는 root 소유로 변경 금지. Linux 테스트의 tempfile 합성 executable만 사용한다. 시스템 mount, shared disk filling, 사용자 Documents/media, 기존 device 쓰기를 사용하지 않는다. 각 subprocess에만 resource RLIMIT_FSIZE를 설정하고 부모/guardian 제한은 바꾸지 않는다. RLIMIT_FSIZE 실제 errno는 EFBIG(27); deterministic OSError ENOSPC(28) 주입은 kernel storage exhaustion으로 주장하지 않는다. 기존 8시간 실행/아카이브 유지.

## 인수 조건

실제 소유 file RLIMIT_FSIZE EFBIG 관측; injected ENOSPC writer 경로 별도 표시; 제품 process download 실패가 Repository Failed로 저장; partial 보존과 final/artifact 부재; SQLite reopen 후 sameJob ID/options/newAttempt/resume 성공, 오류 로그 상한 유지. 실제 공간 부족과 DB/publication ENOSPC는 미검증 상태 유지.

## 검증과 handoff

Windows source snapshot freeze 해제 후 신규 테스트만 작성/직접 rustfmt 및 개별 cargo test. 실행 결과와 실제 ENOSPC 한계를 전용 validation에 기록한다.

Actual: 신규 test 2개 PASS(1.59s), 개별 clippy -D warnings PASS. 자세한 결과와 실제 ENOSPC 미검증은 [검증 문서](../validation/disk-failure-validation.md). 제품/Cargo 변경 없음.
