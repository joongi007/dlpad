# S03: ready 이후 I/O 오류의 소유권 분류

- 상태: in-progress
- 분류: fix
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: root — process.rs 및 이 기록
- 선행 작업: terminal GC/GU LC/LU 실제22case

## 사용자 결과

도구가 시작된 뒤 pipe/thread/child 관측 오류로 terminal 확인을 건너뛰어도 인증 파일을 조기에 삭제하지 않는다.

## 계약과 범위

Linux ready 실행의 모든 반환 오류를 terminal-confirmed 상태와 함께 분류한다. 확인 전 일반 I/O 오류도 고정 typed 미확인으로 전환하고 기존 io kind를 보존한다. 기존 typed 오류는 유지한다. terminal 확인 뒤 오류는 기존 종류를 유지한다. panic unwinding과 sameboot durable receipt는 별도 남은 계약이다.

## 인수 조건

일반 I/O 실패 전/후 분류, 비밀 원문 제외, 기존 typed marker/kind 유지 및 실제 engine 성공·취소·다운로드 회귀.

## 검증과 handoff

구현 중.

## 최초 실제 시험

Linux ready 단계의전체I/O반환을terminal-confirmed flag로분류하고확인전일반오류도비밀없는TerminalUnconfirmed로변환했다. `cargo test -p dlpad-engine --lib --locked cleanup_marker_tests` actual2PASS/0FAIL/exit0. [결과](../validation/evidence/ready-error-ownership/unit.json). 이전29ce8cGUI/release는이후변경을포함하지않으며engine/clippy전체회귀와panic시lease보존은추가인수다.

## reader unwind 보강 준비

pre-ready Linux guardian/launcher가reap되어도모든reader정리ack가없으므로startupattach실패는direct/service모두typed미확인으로보강한다. readiness자체단위parser오류는유지하고공개process반환의소유권해석만강화한다. 별도lease readerguard는begin→명시confirm까지cleanup/Drop을보류하여panicunwind시에도복사본을보존하도록담당분리했다. 이추가변경은아직재검증전이다.

## root 통합 검증 (2026-10-04)

`cargo test -p dlpad-engine --locked` exit0: engine lib185PASS/2ignored, session_cookie_download7PASS, session_cookies20PASS. `cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings` exit0. [현재 소스 SHA와 실제 명령 결과](../validation/evidence/cookie-reader-guard-engine/result.json). 정상·실패·취소·guardian SIGKILL 경로와 reader unwind 보존 시험을 포함한다. 추가 실제 실행 guard 회귀는 별도 담당 검토 중이다. 전체 workspace/release/GUI/Windows/macOS 및 같은boot durable reader receipt는 미검증/미구현이므로 이 scoped 결과로 완료하지 않는다.

## 현재 소스 재감사: direct startup 반환 보강

현재 process.rs의 attach실패/launcher회수 성공 분기는 expects_service일때만typed미확인을반환하고direct는일반오류를반환했다. readerguard가일반오류를확인된종료로해석할수있으므로root가이분기를수정한다. direct에서도launcher회수만으로guardian의모든readerterminalack를대체하지않는다. 실제공개absenttool시험과marker/reader다운로드회귀 및clippy를다시실행한다.

담당파일추가: root는기존 crates/engine/tests/guardian_startup_wait.rs의공개absenttool실패에 cleanup_is_unconfirmed 및실행파일경로비노출assert를추가한다. readerguard가일반오류를확인된것으로취급하지않는회귀다.

## direct startup 보강 실제 결과

`cargo test -p dlpad-engine --test guardian_startup_wait --test session_cookie_download --locked` root session37918 exit0: 공개absenttool1PASS/0.20초, cookie reader8PASS/1.88초, 모두0FAIL/0ignored. 최신fmt/engineall-targetclippy/docs도exit0. [소스SHA와원시검증로그](../validation/evidence/direct-startup-cookie-ownership/result.json). 정상수명의blocking Child.wait/waitpid와socketwrite무한경계는별도 [Child 수명 작업](S03-owned-child-lifetime.md)으로이어진다. 이번반환분류수정은해당수명정책/전체P0완료가아니다.
