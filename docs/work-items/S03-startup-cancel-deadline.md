# S03: startup 취소와 전체 process deadline 구현

- 상태: in-progress
- 분류: fix
- 우선순위: P0
- 요구사항: R06, R08, R11, R13
- 담당/수정 범위: metrics_recovery — crates/engine/src/process.rs 및 이 문서; root — Windows checkpoint와 통합 검증. guardian/UI/worker/Cargo 수정 금지.
- 선행 작업: guardian-startup-wait 구현 및 native17/public2 회귀

## 사용자 결과

readiness 이전에도 취소·timeout을 관찰하고 소유 트리 정리를 시작한다. Linux 구현은 아래 동결 handoff에 기록하며 전체 회귀는 진행 중이다. 앞선 검토와 당시 근거는 역사적 범위로 보존한다. 실제 검증 완료 전 P0 완료로 표현하지 않는다.

## 현재 근거와 한계

[실제 회귀](../validation/guardian-startup-wait-regression.md)의 manager strict13+startup4=17case와 public EP/noR2case 증거를 보존한다. 공개 noR는 15.127초(engine15089ms), EP-sleep은 5.054초(engine5036ms)에 반환했다. noR는 고정 readiness10초 이후 자연 정리5초·kill/reap이며 cancel/ProcessSpec timeout latency 증거가 아니다. [기존 구현 작업](S03-guardian-startup-wait-implementation.md)의 초기 cleanup5+2초와 유한8슬롯 singleton reaper 정책을 이번 검토가 삭제하거나 검증 범위를 확대하지 않는다.

process.rs는 함수 진입 시 cancel을 한 번 확인한 뒤 readiness까지 재확인하지 않는다. Ownership::attach의 고정 read timeout10초가 ProcessSpec.timeout과 독립이고, 현재 실행 start Instant는 readiness 및 drain thread 구성 뒤에 설정된다. 따라서 spec timeout에 launcher startup이 포함되지 않는다. 정상 ready 뒤 취소/timeout 분기의 child.wait()와 Ownership::drop waitpid(...,0)도 무한 대기 경계다. 소유성 확보는 대기 유한성을 뜻하지 않는다.

## 제안: 단일 실행 deadline과 단계별 결과

1. ProcessSpec bounds 검증 후, command 구성/admission/spawn 전에 checked_add로 단일 absolute execution deadline을 만든다. 각 단계에서 같은 cancel Arc와 deadline을 확인하며 normal run loop도 이 deadline을 재사용한다. spawn 전 취소는 기존 Cancelled/no-spawn 동작을 유지한다. std spawn/dlopen 같은 OS 호출 자체의 커널 정체까지 이 폴링 정책이 hard realtime으로 제한한다고 주장하지 않는다.
2. readiness에는 min(execution deadline, startup 시작+10초)를 적용한다. 실행 deadline이 먼저 만료되면 TimedOut, cancel이면 Cancelled, 독립 startup10초만 만료하면 보호 초기화 오류다. 두 중단 의도가 동시에 관찰되면 기존 cancel 우선 정책을 일관되게 유지한다. R을 읽은 직후에도 같은 token/deadline을 확인한다.
3. socket read_exact를 긴 단일 timeout으로 호출하지 않는다. poll 최대20ms와 유한 상태 머신을 사용해 R 또는 EP 두 byte를 부분 수신 누적한다. Interrupted/WouldBlock마다 token/deadline을 재확인한다. E만 수신한 채 멈춘 fixture도 취소 가능해야 하며 partial byte를 버리거나 다시 marker로 해석하지 않는다. 미지 marker/code, truncation, EOF는 기존 고정 protocol/보호 오류를 보존한다. raw argv/env/manager 메시지는 전달하지 않는다.
4. attach의 내부 결과는 Ready(Ownership), Interrupted(Cancelled/TimedOut), Failed(io::Error)로 분리한다. startup 취소를 ordinary io error로만 반환하여 DB가 Failed로 오판하지 않도록, 상위 MetadataResult/ExecutionResult의 기존 ProcessOutcome을 사용한다. 취소 시 stdout metadata는 빈 값이며 partial 파일 삭제 정책을 바꾸지 않는다. cleanup이 입증되지 않은 경우 Cancelled/TimedOut 성공으로 덮지 않고 고정 protection-incomplete 오류가 우선한다.
5. 취소/timeout 인지를 execution deadline의 의미로 정의한다. 종료 정리는 별도 최대5+2초를 허용한다. 따라서 공개 API 반환 상한은 관찰 지연≤20ms + cleanup≤7초 + 유한 drain 정리이며 spec.timeout 자체와 같다고 문서화하지 않는다. 초기 cleanup5+2 정책은 유지한다. 정상 실행 중 기존 T→200ms→K 의도도 유지하되 그 뒤의 blocking child.wait 대신 유한 try_wait 정리를 사용한다.

## 제안: 정상 ready 이후 소유 Child 수명

raw cached PID를 Ownership::drop에서 waitpid하여 다시 blocking하는 경로를 없애야 한다. Child를 실제로 소유한 RAII guard에서 모든 early return/정상 종료/취소에 동일 유한 cleanup을 적용하고, Ownership Drop은 소유 control로 K/EOF를 전달하는 역할만 갖게 한다. 다른 global SIGCHLD/waitpid(-1) reaper가 Child를 소비하지 않는 단일 회수 계약을 유지한다.

최종 try_wait deadline에도 회수되지 않으면 기존 bounded reaper로 Child를 이관해야 한다. 정상 attach 시 기존 startup permit을 해제하면 나중의 normal deferred Child가 사용할 예약 슬롯이 보장되지 않는 점을 먼저 해결해야 한다. 가장 작은 보장안은 permit을 전체 Child 수명 동안 유지하고 최대8개 process 호출 admission으로 명시하는 것이다. 이는 기존 startup-only8슬롯보다 강한 동시성 정책 변경이므로 root가 pipeline 병렬성과 비용을 대조해야 한다. 또는 별도의 정상수명 예약 풀을 만들 수 있지만 여전히 spawn 전에 예약·유한 상한·thread 실패를 확인해야 한다. 정상 실패 순간 임의로 슬롯을 얻거나 무제한 reaper thread를 만드는 fallback은 금지다. 기존 startup 예약 해제 정책을 이 문서만으로 변경하지 않았다.

stdout/stderr drain도 child/Ownership guard를 먼저 잃지 않도록 구성한다. Linux의 기존1초 drain grace/stop mechanism을 유지하고 interrupted drain은 보호 오류로 분류한다. Pipe 구성 실패, thread 생성 실패, evidence lock 오류 같은 readiness 이후 early return도 같은 Child 정리 guard를 지나야 한다. 반환이 유한하더라도 deferred service 정리가 미확인되면 UI/DB는 완료로 확대하지 않는다.

## CloseWait와 partial service EOF 소유권

accessible Worker::request_stop은 pause를 먼저 기록하고 동일 cancel Arc를 Release로 설정한다. CloseWait는 worker completion/None과 다른 panel stopped를 기다리며 새 작업을 막는다. startup 대기에도 이 같은 Arc를 계속 보유·Acquire 관찰해야 한다. 완료 전에 token을 false로 재설정하거나 별도 token으로 교체하지 않는다. 다운로드 pause/partial 유지 전이를 startup Cancelled와 연결하는 기존 execute_with_control 계약을 실패시험으로 확인해야 한다.

앱 control peer drop은 부분 시작된 service guardian에 EOF를 전달한다. launcher가 복제한 guardian-side endpoint가 존재해도 앱 반대쪽 peer 종료의 EOF를 막지 않는다. launcher만 kill하는 것은 guardian/service/tool tree 종료의 증거가 아니며 AddRef bus drop은 unit stop이 아니다. root가 구현한 launcher EP-before-SHUT_RDWR와 app-peer-drop는 소유 socket 기반 정리이며, InvocationID 미확인 unit을 Stop하거나 numeric PID/group을 신호하는 우회가 필요 없다. service가 R 직전에/직후에 취소되는 race도 실제 guardian/worker/setsid leaf 부재, owned cgroup empty/removal, launcher reap/pipe EOF로 검증해야 한다.

CloseWait가 worker를 회수했다고 해도 deferred reaper Child/service가 남아 있으면 process 정리 완료를 자동으로 뜻하지 않는다. 보호 정리 오류를 보고하고 앱이 종료하더라도 native manager의 KillMode=control-group/guardian EOF 책임이 남는 범위를 문서화해야 한다. 일반 close의 성공 인수는 실제 소유 프로세스/pipe 정리까지 확인해야 한다.

## 실패 fixture 인수 조건

- 기존 manager17/public2 결과를 당시 snapshot으로 보존하고 변경 snapshot에서 별도로 다시 실행한다.
- public noR + spec.timeout150ms: timeout 인지는 deadline+20ms 오차 내, API 반환은 cleanup5+2+drain 상한 내; tool 미시작, exact helper identity reap, FD baseline 복귀, sentinel 보존.
- public noR 중100ms 후 cancel: Cancelled 전달 및 동일 유한 정리. 기시작 Cancelled는 child/tool spawn0.
- E byte만 보내고 sleep, R 직전취소, R 직후취소, R과 cancel 동시 fixture: 부분 frame 유지·취소우선·일관된 outcome.
- independent startup10초만 만료(spec deadline 더 큼)는 보호 setup 오류이며 TimedOut과 구분.
- normal R 뒤 control 명령을 무시하고 pipe를 붙잡는 owned helper: 기존 T→200ms→K 후 유한 Child 정리, Ownership Drop이 blocking하지 않음.
- 실제 partial service 전후 취소: started identity/InvocationID를 먼저 확보한 case만 인수; guardian+escaped descendant/cgroup cleanup, pipe EOF/unit GC와 unrelated unit 보존.
- CloseWait 가상 GUI: startup noR 중 종료 의도, UI thread 응답/새 작업 차단, 같은 cancel token 수명, worker 회수, 임시 journal 상태/partial 유지, 원래 소유 프로세스 부재까지 확인. 사용자 desktop/실영상/실계정은 사용하지 않는다.
- deferred adapter/예약 포화/early-return fixture: 전체수명 보장 선택에 맞게 상한, 단일 Child 소유, idle blocking, 슬롯 복구 검증. 실제 D-state/manager 재시작을 임의 유발하지 않는다.

## 검증과 handoff

실제 명령은 sed/rg의 코드·문서 읽기와 이 문서 작성이다. 제품 소스/Cargo/기존 원시 증거를 수정하지 않았다. 구현은 root가 위 outcome/예약수명 공유 계약을 결정한 뒤 별도로 배정해야 한다. 최신 전체 workspace 검증이 진행 중이므로 이 제안은 현재 제품 snapshot의 동작으로 표현하지 않는다.

## 후속 Linux 구현 동결 (2026-10-04)

이 문서의 앞선 review 근거와 검증 당시 범위를 보존한다. [현재 재검토](S03-startup-cancel-deadline-review.md)에 따라 process.rs만 변경했다. bounds 직후 checked_add로 ExecutionControl absolute deadline을 만들고 cancel 우선 outcome을 확인한다. Linux admission은 기존8 슬롯/1초 자체상한을 보존하면서 Condvar 대기를 min20ms/실행remaining으로 나눈다. reservation 직전·직후 및 실제 spawn 직전 checkpoint를 확인한다. spawn 이전 중단은 empty Cancelled/TimedOut 결과로 반환한다.

readiness는 최대2byte partial frame을 실제 UnixStream에서 누적한다. 각 read timeout은 min20ms/실행deadline/독립startup10초다. R/EP/GU/LU 문법과 typed GU/LU 오류는 기존 guardian::read_readiness에 완성된 frame을 넘겨 보존한다. partial E/G/L을 버리지 않으며 WouldBlock/TimedOut/Interrupted 때 동일 cancel/deadline을 재확인한다. R 직후에도 확인한다. normal loop timeout은 readiness 이전부터 생성된 같은 absolute deadline을 사용한다.

spawn 후 readiness 중단은 socket alias shutdown→기존5초/2초 bounded cleanup→기존 typed StartupUnconfirmed 또는 Deferred를 유지한다. launcher만 reap했다고 cookie reader guard를 confirm하지 않는다. preR cancel intent의 별도 UI 표시나 새 typed payload는 구현하지 않았다. Windows preResume checkpoint는 root의 별도 담당이며 이 변경으로 완료됐다고 주장하지 않는다.

새 내부 회귀3개는 실제 socket partial EP/GU/LU semantics와 E-only50ms deadline, G-only중30ms cancel 및 cancel/deadline동시 cancel우선, local 포화8슬롯에서50ms deadline/30ms cancel과 reserved 불변을 확인한다. 각각500ms 느슨한 wallclock 상한으로 고정10초 readiness/1초 admission regression을 구분한다. 실제 성공 실행은 아직 없으며 root Cargo 통합 대기다. 기존 singleton/reaper/native fixture 시험과 default 병렬도는 보존한다.

process.rs SHA256 `dff8b9e7c1f57aa681a0fc33a7b2aed9134ed0ef55f83173e2b2e0465c024c79`, rustfmt exit0. Cargo/guardian.rs/windows.rs/shared schema 수정 없음. OS CreateProcess/spawn syscall 자체가 멈추는 경우를 hard timeout으로 보장하지 않는다. cleanup/pipe/terminal 예산은 execution deadline에 추가되므로 API 총 반환시간이 spec.timeout과 같다는 주장은 하지 않는다.

## 최초 Linux 실행 실패와 시험 동기화 수정

root actual engine test는195PASS/2FAIL/2ignored exit101이었다. 첫 실패 raw (`/tmp/dlpad-startup-linux-test.log`; durable link는 root 보존 후 연결)의 cancellation 및 observed_output_limit_and_cancellation_remain_bounded는50/100ms 고정 timer가 readiness 이전에 cancel을 요청하여 기존 typed StartupUnconfirmed/Interrupted를 받았다. 제품의 reader 보존 정책을 완화하지 않는다. 이 임시 raw는 root가 durable evidence로 보존해야 한다.

두 postready 시험은 실제 ProgressFeed.snapshot 관찰 후 취소하도록 바꿨다. stdout drain은 attach 이후에 시작하므로 parsed tool progress는 실제 ready 이후 증거다. 관찰은5ms poll/3초 절대상한이며 무작정 sleep을 늘리거나 미확인 Err를 허용하지 않는다. 추가 내부 startup_partial_cancel_keeps_owned_cleanup_unconfirmed_after_reap는 실제 E-only helper socket·owned child에서30ms cancel, alias shutdown, bounded exact Child reap 후에도 GC/LC 없는 typed StartupUnconfirmed를 확인한다. 이 내부 시험은 공개 API/native service proof를 대체하지 않는다.

현재 source SHA256 `04fd175181c244db0f9359d12e7914fec38ce30a4142093708c310710359cc38`, rustfmt exit0, Cargo 미실행. root가 통합한 Windows checkpoint/공유 enum을 보존하고 테스트 부분만 변경했다. metadata_cancellation도 같은 작업에서 tempdir의 literal argv marker를 실제 fixture가 stdout flush 후 작성하도록 변경했다. marker 관찰을5ms poll/3초 절대상한으로 기다린 후 취소한다. 원본 사용자 데이터는 사용하지 않는다. marker는 실제 tool 시작 근거이며 native GC/LC 종료 확인은 기존 제품 경로가 수행한다.

문서 검사 첫 시도는 비이식 `/tmp` absolute link로 실패했다. 해당 경로를 raw 위치 텍스트로 바꾼 뒤 재실행372 documents PASS. root durable evidence 경로가 정해지면 portable link로 갱신한다.
