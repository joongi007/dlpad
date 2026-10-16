# S03: 공개 startup deadline·취소 경로의 실제 native 검증

- 상태: done
- 분류: test
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery — 신규 scripts/public_startup_interrupt_probe.py, crates/engine/tests/startup_interrupt_driver.rs 및 본 문서만
- 선행 작업: root 최신 controlled startup/owned lifetime source freeze 및 integration driver Cargo READY

## 사용자 결과

helper가 readiness를 보내지 않거나 E 한 바이트만 보내도 공개 process::run/run_metadata가 전체150ms deadline·실제 launcher 관측 후100ms cancel을 startup 단계에서 받아들인다. reader 종료는 증명되지 않았으므로 reap 후에도 StartupUnconfirmed typed 오류를 유지한다. natural5초/kill/reap 정리는 짧은 요청 deadline과 분리해 관측한다.

## 기존 조사와 재사용 범위

guardian_startup_wait.rs는 nonexistent-tool public1case이며150/100ms noR case가 없었다. terminal_user_service.rs와 user_service_guardian.rs는 R후 worker에600ms timeout/500ms cancel을 적용한다. public_startup_helper_probe.py의 noR9초 readiness 및 EP원문error/errno 기대는 과거 계약이라 새collector로 교체하지 않고 원시 기존자료를 보존한다. terminal_user_service_probe.py는 현재 RGCLC raw/API/cgroup/typedmissingack 회귀의 독립 재실행 대상으로 유지한다.

## 신규 실행 계약

ignored exact controlled_startup_driver는 env의 owned /tmp folder와 whitelistcase만 받아 process::run 또는 run_metadata를 호출한다. noR150ms deadline, helper launcher.json 관측 뒤100ms cancel, partial E cancel 및 metadata150ms deadline을 포함한다. 출력은 max128KiB 고정 JSON이며 원문error/path/tooloutput을 기록하지 않고 error_kind와 OwnershipCleanupError typed marker/cleanup_unconfirmed만 기록한다. root가 compiled test+adjacent helper READY SHA를 제공한 뒤 신규collector를 실행한다.

collector는 복사한 driver와 생성한 adjacent Python helper만 사용한다. helper는 exact PID/start/parent·monotonic을 기록하고 boundedstdout/stderr, noR 또는 E partial, socket EOF 뒤30초 sleep으로 natural5초→ownedkill 필요성을 만든다. 외부env는 child별 direct development mode에만 지정하며 registered app/default backend를 약화하지 않는다. tool은 실제로 시작되지 않아야 한다. driver/helper pidfdready·exactidentitygone·FDstable·sentinelpreserved·tempcleanup·전후SHA를 확인한다. fake public scope에는 service/cgroup이 없으므로 cgroup PASS를 주장하지 않는다.

## 재실행 명령과 cgroup 독립 인수

root 빌드 명령: cargo test -p dlpad-engine --test startup_interrupt_driver --locked --no-run. READY native driver를 절대경로로 지정해 python3 scripts/public_startup_interrupt_probe.py DRIVER NEW_EVIDENCE를 실행한다. 기존 manager 증거는 python3 scripts/terminal_user_service_probe.py IMMUTABLE_TERMINAL_DRIVER NEW_MANAGER_EVIDENCE --case cancel --case timeout --case startup_after_start --case startup_after_get_unit --case startup_after_invocation로 별도 재실행한다. immutable driver 옆 actual same-snapshot helper가 필요하고 /run/user/UID bus·systemd/private 소유 socket을 확인한다. exact InvocationID/MainPID/start/cgroup과 모든 owned PID/pidfd, cgroup제거와고유unitGC, 별도 sentinel을 관측한다. unit명/PID만으로 기존 process/service를 종료하지 않는다.

## 인수 조건

startup EOF는150ms timeout 또는 launcher관측100ms cancel의 근처에서 발생하고, 반환은 natural5초 이후·finite upperbound 안이어야 한다. 오류kind는TimedOut/Interrupted와StartupUnconfirmed이며 socketclose/ownedhelperreap가 cookie reader종료확정으로 바뀌지 않는다. partial E도 동일하다. nativehelper/driver logs는128KiB 상한이며 원문 secret/media/network/사용자desktop 없이 합성fixture만 사용한다.

## 검증과 handoff

조사·신규collector/driver 작성 단계다. Cargo/product/process/기존scripts/tests는 변경하지 않는다. root compiled READY 전에는 실행하지 않는다. 실제 kernel nonresponsive/deferredreaper/managerrestart/GUI/OS/realmedia/8h 인수를 이단기시험으로 대신하지 않는다.

## source freeze 인계

driver ignored exact controlled_startup_driver와 신규collector를 작성했다. 4cases는 noR_timeout(150ms), noR_cancel(launcher marker 관측 뒤100ms), partial_cancel(E한바이트+같은취소), metadata_noR_timeout이다. StartupUnconfirmed downcast와 TimedOut/Interrupted kind를 고정JSON으로 기록한다. helper는 EOF뒤30초sleep하여 실제natural5초/kill/reap를 요구한다. EOF50ms~1초 미만, engine반환4.5~8초 미만, FDstable/pidfdready/exactidentitygone/tool미시작/sentinel 및copySHA 불변을 인수한다. helper/driver/log/JSON은bounded이며 실제 native 실행은 아직 하지 않았다. rustfmt 및 Python py_compile 성공, 문서374개 검사 통과. root source publication/build입력시점과 별개로 새 Rust파일은 동결했고 root Cargo READY를 기다린다.

## root 실제 실행과 독립 검토

Linux native driver `cargo test -p dlpad-engine --test startup_interrupt_driver --no-run --locked` exit0 후 immutable 임시 복사본으로 `scripts/public_startup_interrupt_probe.py`를 실행하여 4case 모두 PASS/exit0. [원시 결과](../validation/evidence/startup-interrupt-native-first/result.json)와 [root 재검토](../validation/evidence/startup-interrupt-native-first/root-review.json)는 EOF 0.111–0.143초, 전체 반환 5.17–5.22초, FD4→4, owned identity 부재·pidfd ready·sentinel 보존·temporary 제거를 확인한다. probe 현재 SHA와 결과 SHA도 일치한다. 5초 자연 정리 유예는 실행 deadline과 별도다. 쿠키 파일 보존/서비스 cgroup/GUI/다른OS 인수로 확대하지 않는다.
