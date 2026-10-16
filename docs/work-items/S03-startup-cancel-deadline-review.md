# S03: 현재 Linux/Windows startup 취소·deadline 최소 계약

- 상태: done
- 분류: review
- 우선순위: P0
- 요구사항: R06, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 이 문서만; process.rs/windows.rs/guardian.rs 읽기 전용
- 선행 작업: [이전 startup 검토](S03-startup-cancel-deadline.md), [Child 전체 수명](S03-owned-child-lifetime.md), [Windows Job 확인](S03-windows-terminal-ownership.md), [Windows pipe 중단](S03-windows-interruptible-pipes.md)

## 사용자 결과와 현재 증거

startup/admission에서 취소 및 전체 실행 deadline을 관찰하고 새 tool 실행을 차단한다. done은 검토 완료이며 제품 구현 완료가 아니다. 이전 public noR 15.127초/EP 5.054초는 [당시 회귀](../validation/guardian-startup-wait-regression.md) 근거이며 현재 취소 latency 시험으로 확대하지 않는다. 최근 owned lifetime/native Job/interruptible pipe 작업이 blocking 종료 경계를 줄였지만 startup deadline 자체는 미구현이다. root의 진행 중 전체 Clippy snapshot은 수정하지 않았다.

현재 run_capture는 bounds와 cancel을 함수 진입에서만 검사한다. Linux guardian command→StartupPermit::reserve→spawn→Ownership::attach 순서이며 admission의 자체1초 Condvar 대기는 token을 받지 않는다. readiness는 고정10초 read timeout이고 read_readiness의 두 번째 byte read_exact도 별도 대기할 수 있다. 따라서 E/G/L만 수신하는 상대가 총 실행 timeout보다 오래 정체시킬 수 있다. normal execution timer는 readiness와 두 drain thread 구성 뒤에 생성돼 이전 비용을 제외한다.

Windows에는 Linux admission/readiness가 없다. Ownership::new→원자적 suspended CreateProcessW→membership verification→ResumeThread가 하나의 spawn 함수이며 cancel/deadline을 전달받지 않는다. 취소가 process 생성 중 발생해도 ResumeThread 전 재검사가 없으므로 보호된 tool이 실행될 수 있다. OS API 자체가 반환하지 않는 커널 정체는 thread polling으로 제한할 수 없으며 별도 thread timeout으로 소유 handle을 잃는 우회는 제안하지 않는다.

## 최소 공유 인터페이스

root process.rs에 내부 ExecutionControl<'a> { cancel: &'a AtomicBool, deadline: Instant }와 checkpoint()->Option<ProcessOutcome>를 둔다. bounds 확인 직후 checked_add(spec.timeout)로 deadline을 만들고 overflow는 spawn 전 InvalidInput으로 거절한다. cancel 우선, 이후 deadline을 검사한다. ProcessSpec 공개 필드 변경이나 새 Arc/thread/schema는 필요 없다. 함수 진입·command/admission 전후·spawn 직전·ready 직후·drain 구성 직후·normal loop에서 동일 checkpoint를 사용한다. 기존 output-limit 우선순위는 ready 이후 보존한다.

Linux StartupPermit::reserve_with_control 및 reserve_on_with_control은 기존8 슬롯과1초 admission 한계를 유지한다. wait_timeout은 min(20ms, admission remaining, execution remaining)으로 잘라 취소를 주기적으로 관찰한다. 슬롯을 얻기 직전에도 checkpoint를 확인하고 중단이면 예약하지 않는다. capacity 포화1초만 먼저 끝나면 기존 admission 오류이며 실행 deadline이 먼저면 TimedOut이다. 예약을 획득한 직후 checkpoint 실패 시 permit을 반환하고 spawn하지 않는다. 기존 reserve_on은 내부 local tests를 위한 wrapper로 보존하거나 시험 호출을 명시적으로 바꾼다. 실제 pending child를 버리거나 상한/병렬조건을 변경하지 않는다.

Linux attach에는 별도 내부 StartupOutcome::Ready(Ownership)/Interrupted(ProcessOutcome)/Failed(io::Error)가 필요하다. socket을 nonblocking으로 읽는 최대2byte 상태 머신 또는20ms timeout read를 사용한다. R/EP/GU/LU 문법과 typed GU/LU를 보존하고 partial prefix를 누적한다. WouldBlock/TimedOut/Interrupted에서 checkpoint를 재검사하며 startup 독립10초는 min(execution deadline, attach 시작+10초)다. read_exact를 재시도해 이미 소비한 prefix를 잃어서는 안 된다. R 직후에도 checkpoint를 확인한다. EOF·invalid frame는 고정 오류다. 독립 startup10초만 만료하면 setup 오류이고 실행 deadline과 혼동하지 않는다. ready를 잃은 상태에서는 raw tool exit만으로 reader cleanup을 확인하지 않는다.

Windows spawn 내부에는 spawn_controlled(spec,&ExecutionControl)->Result<SpawnOutcome>를 추가하거나 현재 private spawn을 변경한다. CreateProcess 전 checkpoint는 no-child Cancelled/TimedOut, suspended 반환 후 checkpoint는 **ResumeThread 전에** 검사한다. 이미 중단이면 Job TerminateJobObject→Child bounded wait→Job ActiveProcesses0 확인 후에만 Interrupted(outcome)을 반환한다. Query/wait/kill 실패는 typed StartupUnconfirmed/TerminalUnconfirmed로 보존한다. ownership과primary/Child handle은 정리 확인까지 살아 있어야 한다. ResumeThread 직후 중단 race는 공통 normal stop 흐름으로 이어지고 spawn 직전 checkpoint만으로 완전 차단했다고 주장하지 않는다.

## 중단 intent와 cleanup의 구분

spawn 전 중단은 기존 empty MetadataResult + Cancelled/TimedOut이며 cookie guard를 confirm할 수 있다(아직 tool reader가 없다). Linux spawn 후 preR 중단은 현재 어떤 attach 오류도 StartupUnconfirmed로 처리하는 정책을 유지한다. intent를 일반 Cancelled로 반환하면 쿠키 guard가 confirm하여 미확인 reader copy를 삭제할 위험이 있다. 최소 안전 구현은 즉시 peer shutdown→기존5초/2초 bounded Child 정리 후 typed 미확인 반환이다. Cancelled/TimedOut을 정확히 UI에 표시하려면 fixed typed cleanup error에 선택적 interrupt intent를 추가하는 별도 root 계약이 필요하다. 단순 errno/문구 매칭은 금지다.

Windows interrupted suspended child는 user code 미실행과 동일 Job terminal 확인이 함께 입증되면 outcome을 안전하게 반환할 수 있다. 일반 ready 이후 취소/timeout은 현재 bounded Child/drain/terminal 확인 흐름을 그대로 사용한다. 실행 deadline은 중단 의도 관찰 상한이며 API 전체 반환 상한과 같지 않다. Linux startup cleanup5+2초 및 유한 drain/ack, Windows2초 Child/Job/drain 등 단계별 정리 예산은 별도다. callback이나 token reset 없이 같은 Acquire flag를 CloseWait까지 유지한다.

## 실패 시험 위치와 인수 조건

- process.rs startup_wait_tests local reaper:8 예약 포화 중 cancel/짧은 execution deadline이1초 admission보다 먼저 반환, reserved 수 변동 없음; 예약 반환 notification 기존 시험 보존. default 병렬도와8 슬롯 상한 유지.
- guardian.rs read_readiness 인접 parser 시험 또는 process.rs UnixStream pair: noR/E-only/G-only/L-only, R 전후 cancel, cancel+deadline 동시, GU/LU typed·EP fixed·unknown/truncated 거절. 실제 socket partial read와20ms 관찰을 확인하며 단순 byte slice 시험으로 latency를 대체하지 않는다.
- 공개 synthetic helper fixture: spec timeout150ms/noR,100ms cancel, spawn 전 cancel/tool0. 원 owned PID/startticks 회수·sentinel·FD/permit 복귀와 반환의 cleanup 예산을 확인. 기존 typed 미확인 copy 보존 시험을 outcome 개선 때문에 약화하지 않는다.
- windows.rs: suspended fixture 생성 뒤 이미 cancelled control로 resume helper 호출→literal output 없음/Child 회수/Job0; deadline 만료도 동일. test current executable만 사용한다. 필요한 seam은 suspended result와 resume의 private 함수 분리이며 제품 환경 fault hook은 불필요하다. Query 실패는 기존 non-Job owned event fixture로 typed 보존 확인.
- actual Windows native 실행에서 normal exit/126,cancel/timeout/Job자손/pipe stop/쿠키 unknown 보존을 기존 회귀와 함께 수행한다. Linux native partial-service/R race는 실질 시작한 InvocationID/cgroup 증거를 확보한 case만 PASS로 인정한다.
- CloseWait private GUI: startup 중취소, UI 응답/새 작업 차단, 동일 token 수명, pending cookie 유지와 worker 회수. 사용자desktop·실영상·실계정은 사용하지 않는다.

## 실행 및 한계

실제 수행은 rg/sed/cat 소스·기존문서 읽기와 이 문서 작성뿐이다. Rust source/Cargo/기존 evidence를 수정하지 않았고 Cargo는 실행하지 않았다. 구현 배정과 typed intent/미확인 정책은 root가 결정한다. Windows/macOS GUI·새 latency 회귀·전체 deadline 완료를 주장하지 않는다.
