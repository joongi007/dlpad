# S02: reader 종료 미확인과 worker unwind 시 쿠키 보존

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery — cookies.rs, tests/session_cookies.rs 및 본 문서; 이전 scoped 검증의 기존 기록 갱신
- 선행 작업: durable recovery records; root download/process backend의 실제 terminal-confirmed receipt 연결

## 사용자 결과

worker가 panic하거나 종료 확인을 잃으면 lease Drop이 아직 reader가 사용할 수 있는 쿠키 복사본을 지우지 않는다. private 기록과 copy를 남기고 모든 소유 핸들을 닫는다. 확인된 종료 뒤에는 기존 정상 정리를 유지한다.

## 계약과 범위

begin_reader(&self)->Result<SessionCookieReader<'_>,CookieImportError>는 lease 소유권 검증 후 AtomicBool CAS로 reader 하나만 예약한다. guard.confirm(self)는 root backend가 실제 terminal cleanup을 확인한 경우만 호출한다. guard Drop은 flag를 해제하지 않으며 Debug는 고정 redacted다. pending 상태의 cleanup은 defer_cleanup를 수행하고 fixed ReaderUnconfirmed를 반환한다. lease Drop도 pending을 먼저 defer하므로 OwnedCopy Drop의 unlink 경로를 막는다. 일반 no-reader cleanup은 유지한다. flag는 bounded atomic 하나이며 새 thread/dependency가 없다.

이 프로세스 내부 guard는 실제 OS crash에서 guardian 수명을 증명하지 않고 durable sameboot reader receipt도 기록하지 않는다. 이전boot-only recovery 계약을 그대로 유지하며 같은boot 삭제 인수는 완료로 쓰지 않는다. 실제 tool과 metadata/download panic 통합은 root 담당이다.

## 인수 조건

확인된 guard 정상cleanup/재예약, 두 번째 begin 거절, 미확인 guard explicitcleanup 반복과 copy/record/source 보존, catch_unwind로 guard+lease unwind 시 copy/record/source 보존 및 Linux 고유 cache/owner/file 경로 FD 모두 종료를 합성 fixture로 확인한다. 실제 tool을 시작하지 않았으며 시험의 PASS는 root Cargo 후 기록한다.

## 검증과 handoff

담당 두 Rust파일 rustfmt 및 문서 검사 후 source freeze로 인계한다. Cargo/module/UI/download/process/shared source는 수정하지 않는다. 신규 reader API 시험은 root 검증 대기다.

## root 통합 검증 (2026-10-04)

`cargo test -p dlpad-engine --locked` exit0: engine lib185PASS/2ignored, session_cookie_download7PASS, session_cookies20PASS. `cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings` exit0. [현재 소스 SHA와 실제 명령 결과](../validation/evidence/cookie-reader-guard-engine/result.json). 정상·실패·취소·guardian SIGKILL 경로와 reader unwind 보존 시험을 포함한다. 추가 실제 실행 guard 회귀는 별도 담당 검토 중이다. 전체 workspace/release/GUI/Windows/macOS 및 같은boot durable reader receipt는 미검증/미구현이므로 이 scoped 결과로 완료하지 않는다.
