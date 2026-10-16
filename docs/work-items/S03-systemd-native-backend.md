# S03: Linux native 사용자 서비스 launcher

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당/수정 범위: metrics_recovery — 신규 crates/engine/src/process/user_service.rs, 전용 작업 문서
- 선행 작업: native contract review; root process/guardian marker·module 연결

## 사용자 결과

guardian을소유nonce사용자service로실행하는launcher가직접Child stdout/stderr/wait계약을유지한다.

## 계약과 범위

run(OwnedFd, &Path, &[OsString])->io::Result<ExitStatus>. root공유files/Cargo/schema수정없음. existinglibc dynload/RAII/sd-bus typedcontainers/no-env/AddRef/identity/cgroup/terminalproxy. release고정UIDbus/debugonlymissingbusoverride. 명시boundedenv/argv/응답/각calltimeout; jobtimeout는기존engine T/K. weakfallback없음. Cargo/build/GUI/native통합은root직렬.

## 인수 조건

lib/symbol/input/bus/socketUID/FD/property/status/identity실패 및정상exit/signal/빠른완료/cleanup/pipeEOF actualnative. 단위fixture는실제manager/crashcleanup인수를대신하지않는다.

## 검증과 handoff

작성후rustfmt/sourcefreeze/rootcompile요청. rootREADY뒤native제품시험과failclosed integration필수. 아직미연결이며현재backend완료아님.


## 최초 구현 handoff

신규 user_service.rs는 미선언 상태로 root에 freeze를 전달했다. run API, libc dynamic loader/RAII, bounded UTF8 argv/env, 고정UID bus/debug-only override, Unix control/CLOEXEC, no-env/AddRef/transientservice, 같은invocation/cgroup/MainPID 관측, terminal proxy/cleanup을 구현했다.5회귀를 작성하고rustfmt exit0. 실제Cargo/native integration은 미실행이다. 제품공유파일/Cargo는변경하지않았다.

Start call 뒤 message/reply scope drop으로복제FD를해제하고controlOwnedFd를drop한다. stdout1/stderr2는launcherexit까지유지해parent drain이뒤에EOF를받는계약이다. fastterminal은AddRef 아래InvocationID와historicalExecMainPID/CodeStatus로읽는다. per-call2초/transition15초/cleanup5초, jobtimeout는root기존T/K이며임의RuntimeMax제한없음. property문자열4096B/input각1MiB/4096개상한. managerOOM/librarysymbol/UTF8/NUL/FD/socketidentity 실패를정상성공으로바꾸지않는다.


## PID 식별 보강 handoff

MainPID는 숫자 외에 `/proc/PID/stat`의 startticks와 소유 pidfd를 함께 유지한다. pidfd capture 전후와 cgroup membership 읽기 전후에 생존·startticks를 확인하며, terminal ExecMainPID는 관측한 PID와 대조한다. PID<=1, proc 소실, 종료된 pidfd는 같은 연속 5초 deadline과 20ms 대기를 사용한다. timer는 완전한 identity/membership 검증 성공 후에만 초기화한다. PID를 직접 signal하거나 임의 cgroup을 수정하지 않는다. pidfd 미지원/권한 실패는 fail-closed이다.

보강 source SHA256: `088ac27424e3488a56f654a3913a7a4f478ebde6f667255a7b9181fc0996b90b`. rustfmt exit0; Cargo 또는 native 실행은 아직 하지 않았다. root가 module 연결·컴파일·통합을 이어간다.

한계: StartTransientUnit 성공 후 GetUnit/InvocationID 조회에서 실패하는 경로는 control socket EOF와 bus AddRef 해제를 통한 guardian 정리에 의존한다. 실제 이 구간의 EOF/cgroup cleanup은 아직 입증되지 않았으며, 전체 backend 완료 판정 전 실패 주입 통합시험이 필요하다. terminal status, launcher pipe EOF, 앱+guardian 동시 종료, manager 부재, no-env 미지원도 actual native 검증 대상으로 남아 있다.


## 부분 초기화 실패 검토와 fixture hook

Start 성공 뒤 control descriptor drop만으로 service의 동일 socket endpoint에 EOF가 생기지 않는 결함 후보를 읽기 검토로 확인했다. root가 launcher에서 CLOEXEC control alias를 보유하고 startup 실패에 고정 `[E,P]` frame을 보낸 뒤 SHUT_RDWR하여, 확인하지 않은 unit/PID에 신호를 보내지 않고 소유 endpoint로 guardian 정리를 시작하는 공유 구현을 담당한다. module은 중복 alias/frame을 추가하지 않았다. manager AddRef 해제는 unit 정리 보장이 아니므로 실제 EOF/cgroup/pipe 검증을 생략할 수 없다. manager 부재가 기존 UnexpectedEOF로만 전달되는 문제는 root의 고정 readiness 오류 mapping 대상으로 보고했다.

전용 debug fixture 환경 `DLPAD_TEST_SYSTEMD_STARTUP_FAULT`는 정확한 값 `after-start`, `after-get-unit`, `after-invocation`에서 각각 Start 성공/control drop, GetUnit 성공, InvocationID 성공 직후 고정 오류를 낸다. release는 이 환경을 읽지 않고 무시한다. hook 선택의 pure unit 1개를 추가했다(총 6개 작성, 아직 실행하지 않음). 미지 값은 주입을 하지 않으며 환경값을 오류/로그에 넣지 않는다.

hook source SHA256 `d3db54b86e153ce4dcb5044a7f9cc3f7cefddcfdc56c29f4b64e5a4a7642b709`, rustfmt exit0. module 동결을 root에 전달했으며 Cargo/native fault 시험은 root 직렬 조정 대상이다. 성공했다고 보고하지 않는다.
