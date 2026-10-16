# S02: cookie reader 실행 guard

- 상태: in-progress
- 분류: fix
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — download.rs, 이 문서만
- 선행 작업: stress begin_reader/SessionCookieReader::confirm; root process 모든 미확인 terminal 오류 typed

## 사용자 결과

metadata/download의 실제 reader 사용 직전에 lease guard를 시작한다. terminal 확인된 반환에서만 guard를 confirm하며 미확인 반환이나 unwind에서는 복사본 보존 의도를 유지한다.

## 계약과 범위

version/FFmpeg version에는 cookie reader guard를 넣지 않는다. 기존 결과/취소/로그 생략과 lease borrow 수명을 유지한다. metadata io 결과는 cleanup_is_unconfirmed로, download 결과는 SessionCleanupDeferred로 정확 판정한다. 새 DB/schema/Cargo/thread 없음. 이 작업은 unwind 안전성이며 같은boot invocation identity receipt의 완료가 아니다.

## 인수 조건

성공·확인된 실패/취소/outputlimit는 confirm, typed미확인·panic은 active 보존, prelaunch cookie 거절은 guard생성0. 원본/sentinel/secret 결과/기존 download API를 보존한다. 실제시험은 root 직렬 Cargo와 별도 fixture가 담당한다.

## 검증과 handoff

구현전 계약 기록. source freeze 후 root/stress API 통합검증을 요청한다.


## source freeze handoff

metadata argv와 ProcessSpec를 먼저 구성·cookie validation한 뒤 begin_reader를 즉시 호출하고 process::run_metadata를 실행한다. 반환 io 오류가 cleanup_is_unconfirmed typed marker가 아니면 guard.confirm()을 호출하고, 그 뒤 metadata exit/outcome 분류·JSON parsing을 수행한다. download는 validated argv와 Downloading state 뒤 begin_reader→checked 실행이며 SessionCleanupDeferred에서는 confirm하지 않는다. checked의 확인된 Execution 오류(Cancelled/TimedOut/exit failure/OutputLimit)는 confirm하여 기존 DB/outcome 전이를 유지한다. confirm은 process 반환 직후, optional processing endpoint finish 이전이다.

guard는 cookie lease를 borrow하고 기존 전체 execute_inner borrow와 함께 유지된다. typed 미확인 반환 또는 unwind에서는 pending Drop을 유지한다. version/FFmpeg version에는 guard를 넣지 않는다. begin_reader 오류는 고정 private-cookie-reader 오류로만 보고하며 raw import/path/error를 노출하지 않는다. root/stress가 pending lease Drop의 record/FD 보존을 구현하므로 reader guard를 임의 mem::forget하지 않는다.

download source SHA256 `832140d986992c0b5aea5b4325be49687a55280d06cffc600efe6d4e885a81d3`, rustfmt exit0. Cargo/source guard fixture는 root 직렬 인수로 아직 실행하지 않았다. stress의 begin_reader(&self)->Result<SessionCookieReader<'_>,CookieImportError>, confirm(self)->() API와 확인했다. 기존 secret omission/typed SessionCleanupDeferred/upfront cookie validation은 유지했다.

이 변경은 pending-reader unwind safety다. 같은boot exact unit/InvocationID/cgroup reader receipt와 Begin 이전 durability는 구현하지 않았으므로 해당 recovery 계약이나 전체 P0 완료로 확대하지 않는다. 정상 실패/취소 cleanup과 실제 guard pending 보존을 root integration에서 대조해야 한다.

## root 통합 검증 (2026-10-04)

`cargo test -p dlpad-engine --locked` exit0: engine lib185PASS/2ignored, session_cookie_download7PASS, session_cookies20PASS. `cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings` exit0. [현재 소스 SHA와 실제 명령 결과](../validation/evidence/cookie-reader-guard-engine/result.json). 정상·실패·취소·guardian SIGKILL 경로와 reader unwind 보존 시험을 포함한다. 추가 실제 실행 guard 회귀는 별도 담당 검토 중이다. 전체 workspace/release/GUI/Windows/macOS 및 같은boot durable reader receipt는 미검증/미구현이므로 이 scoped 결과로 완료하지 않는다.

추가 root 검증: `cargo test -p dlpad --features cpu --locked` exit0, main16/accessible69/CPU74PASS·0FAIL.실제GUI시험이아닌Rust UI통합검증이다. `python3 scripts/check_docs.py` 359개PASS. 소유8h시험은host의11개PID/startticks와동일boot를UTC16:45:23에재확인하여live이며terminal은아직없다.

## root 전체 Linux 통합 결과

[별도 재시도 원시 결과](../validation/evidence/owned-child-reader-workspace-retry/workspace-result.json)에서fmt/defaultClippy/defaulttest/CPUClippy/CPUtest/defaultrelease/CPUrelease모두exit0. 대상소스시작끝SHA일치, 원래병렬조건유지. 공통parser6시험과Childunwind/reapedDrop2시험을root가로그대조했다. CPUbinary12515320B/SHA`d0451487a8127f6891ccf377e8aa1826fe9261b4c554e6278c791b23a3d9c3a5`를[immutable copy](../validation/evidence/owned-child-reader-workspace-retry/immutable-app.json)에고정했다. 신규unwiredWindowsbackend는컴파일/검증범위밖이다. 실제native22/GUI/Windows/macOS와readiness전체deadline은후속인수다. 최초Clippy실패raw는보존했다.
