# S03: guardian startup 유한 대기 구현

- 상태: in-progress
- 분류: fix
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당/수정 범위: metrics_recovery — crates/engine/src/process.rs, crates/engine/tests/guardian_startup_wait.rs, 이 문서
- 선행 작업: S03-guardian-startup-wait.md 읽기 검토; root guardian EP readiness/partial startup socket shutdown

## 사용자 결과

Linux guardian readiness 실패 후 launcher가 살아 있어도 worker 호출이 무한 wait에 들어가지 않도록 한다. startup에 한해 5초 자연 종료 대기, 정확한 unreaped Child kill, 마지막 2초 try_wait를 적용한다. 재회수 불능이면 고정 오류를 반환하고 Child 소유권을 별도 유한 reaper에 이관한다. 정상 작업의 timeout이나 다른 OS 정책은 이 변경으로 재정의하지 않는다.

## 계약과 범위

attach 내부 wait를 제거하고 호출자 Linux 실패 분기에서만 cleanup한다. read timeout 설정/해제 실패도 caller cleanup으로 들어간다. 실제 child spawn 전에 8슬롯을 예약한다. singleton reaper thread 생성 실패나 슬롯 포화는 tool/launcher spawn 전 고정 오류다. 정상 attach 또는 spawn 실패는 예약을 해제한다. transfer는 이미 예약한 슬롯을 사용하며 큐 overflow 시 Child를 버리는 분기가 없다. pending+active+in-flight 예약의 합은 최대 8이다.

thread는 하나이고 스택 요청은 128KiB다. pending VecDeque와 active Vec는 각각 최대 8 Child 보관 상한을 가진다. 회수할 자식이 없으면 Condvar에서 무기한 blocking하며 idle poll하지 않는다. deferred Child가 있을 때만 try_wait를 20ms 간격으로 확인한다. reap 완료 또는 ECHILD로 더 이상 자신이 회수할 자식이 없으면 슬롯을 반환한다. 영구적인 커널 정체/회수 오류는 슬롯을 계속 차지해 추가 startup을 fail-closed하며 무제한 큐/스레드를 만들지 않는다. 기존 libc 외 새 의존성은 없다. 실제 RSS/성능 영향은 아직 측정하지 않았다.

kill은 Child.try_wait가 None인 unreaped 단일 소유 Child에만 수행한다. 다른 global waitpid(-1)/SIGCHLD reaper가 이 Child를 소비하지 않는 전제가 필요하다. startup 실패에는 Ownership 객체가 생성되지 않으므로 기존 정상 경로 Ownership::drop의 waitpid와 중복 회수하지 않는다. 종료상태가 이미 있으면 kill하지 않는다. PID/group/unit 이름 기반 신호를 추가하지 않았다.

service가 이미 시작되었으면 앱 control peer drop이 guardian EOF 정리를 시작한다. launcher kill/reap은 service cgroup 정리 완료를 뜻하지 않는다. root의 실제 partial-start fixture에서 guardian/descendants/cgroup/pipe EOF를 별도로 확인해야 한다. 정상 attach 후 기존 Ownership::drop 등의 다른 무한 대기 경계는 이번 startup 작업 범위 밖이다.

## 인수 조건

- 고정 EP 후 sleep하는 실제 합성 자식은 오류 반환 후 5초 자연 대기→kill→reap; blocking wait 없음.
- noR fixture는 짧은 테스트 read timeout 후 같은 cleanup; EOF 자연 종료 fixture는 try_wait 회수.
- adapter는 Interrupted/deadline 후 kill, kill 이후 회수 성공, 최종 deferred 경계를 시험.
- 실제 예약 함수의 8슬롯 포화 거부, release 이후 복구, 최종 reserved=0 확인.
- 공개 process API의 임시 경로 absent-tool 초기화 실패가 유한 시간 안에 반환.
- 실제 service startup 실패에서 소유 guardian/worker/setsid leaf/cgroup 정리와 pipe EOF는 root 통합시험에서 검증.
- singleton Thread spawn 실패는 spawn전 에러 분기가 구현되었으나 실제 OS thread 생성 실패 주입은 아직 시험하지 않았다.

## 검증과 handoff

rustfmt --edition 2024 process.rs 및 새 integration 파일 exit0. source SHA256 `0f424c15a2331724862068f133b150fed716db3f900a9d3723fd1798a4a07824`, integration SHA256 `9a2751232ce3fb4863e71afba9dd1ec7e2f3bcf2700cec954758de76223150f8`. unit 4개와 integration 1개를 작성했으나 Cargo/fixture 실행은 아직 하지 않았다. root 직렬 compile/clippy/tests가 필요하다. guardian.rs/user_service.rs/Cargo/공유 문서를 수정하지 않았다. 기존 검토 done 문서는 보존했다.

## 주 담당 실제 검증

root가startupunit4/공개absenttool1/clippy를실행해exit0을확인했다. 추가actualdeferredsingleton회수unit1도exit0/0.34초. [단기검증](../validation/evidence/guardian-startup-wait/result.json), [실제transfer및nativebuild](../validation/evidence/guardian-startup-wait/transfer-and-native-build.json). root가8예약/큐/active경계와단일소유Childkill·emptyCondvar를코드대조했다. 최신driver/helper에서strict13+startup4와공개EP/noR실패시험은하위에이전트가실행중이며native결과·전체Linux/Windows회귀/비용은미완료다.

## 실제 공개 API와 사용자 서비스 회귀

root가 [공개EP/noR2case](../validation/evidence/guardian-startup-wait-public-20261003/root-review.json)의socketEOF→naturalgrace→정확launcher회수·FD4→4·유한반환을독립대조했다(약5.05/15.13초). [manager17case](../validation/evidence/guardian-startup-wait-native-20261003/root-review.json)의기록된PID/cgroup부재·unitGC·sentinel보존도확인했다. 새전체workspace/Windows/release/비용검증은미완료다. 시작대기중cancel/전체ProcessSpecdeadline반영은후속검토가필요하며현재고정readiness10초와cleanup상한을완료latency로표현하지않는다.

## 전체 병렬 회귀 발견

최초전체workspace fmt/clippy는exit0이지만defaulttest는exit101. engine 병렬시험 running_pause_ack_preserves_partial_and_reuses_job이일시적인8startup예약포화로정상작업을거절했다(164PASS/1FAIL/2ignored 해당testbinary). [원시실패](../validation/evidence/guardian-startup-wait-full/default-test.log). 전체검증runner는실패시후속CPU/release를실행하지않았다. 슬롯상한을유지하면서boundedadmission대기/예약해제통지로일시초기화겹침을처리하도록수정중이다. 테스트병렬도를낮춰실패를숨기거나실패로그를덮어쓰지않는다.


## 병렬 startup admission 보완 handoff

원시 default workspace 실패를 보존하고, 순간 8슬롯 겹침은 최대1초 absolute deadline의 Condvar 대기로 처리하도록 수정했다. 대기 중에도 queue/thread/슬롯을 추가하지 않으며 deadline 뒤에도8슬롯 full이면 기존 고정 admission 오류로 fail-closed한다. 정상 attach 예약 release와 worker reap 슬롯 release는 notify_all하며, transfer 역시 같은 Condvar의 admission waiter가 worker 통지를 독점하지 않도록 notify_all한다. 대기자가 없고 deferred child가 없는 idle worker는 기존 blocking 정책을 유지한다. 전체 ProcessSpec timeout/cancel에 admission1초를 포함하는 개선은 별도 읽기 검토 대상이며 이 수정으로 해결됐다고 쓰지 않는다.

실제8슬롯 포화 unit을1초 유한 실패+release 후 재예약으로 보완하고, 새 `startup_admission_waiter_wakes_when_reserved_slot_releases`는 local8예약을 보유한 상태에서 waiter 미진행을 확인하고 release 통지 후500ms 안에 진행하는 경로를 시험한다. 이 시험은 globalreserved=0을 가정하지 않는다. source SHA256 `00c29001af1a5842d951ad75d7eca7dcaadedf13b78c375c4e1eefa459eb26f9`, rustfmt exit0. Cargo는 root 독점이므로 아직 실행하지 않았으며, default parallel workspace regression을 다시 통과해야 한다. 슬롯수 증가/테스트 병렬도 축소는 하지 않았다.

## admission 수정 후 재검증

유한1초admission대기/해제통지후원래parallel조건의defaultworkspace test가exit0으로통과했다. fmt/defaultclippy도exit0. [별도재시도로그](../validation/evidence/guardian-startup-wait-admission-retry/workspace-result.json)를사용하고첫실패raw는보존했다. CPU/clippy/test/release후속은이기록시점진행중이며완료로확대하지않는다.

CPU전체test도exit0/359PASS/7ignored, 기본220PASS/7ignored. RUST_TEST_THREADS는미설정으로기존parallel조건을유지했다. defaultrelease도exit0, CPUrelease는이기록시점진행중이다. [count/조건](../validation/evidence/guardian-startup-wait-admission-retry/test-counts.json).

CPUrelease도exit0완료해7명령모두PASS. immutableCPU97cfbcec/12449656B를고정했고새GUI·비용표본은별도인수한다.
