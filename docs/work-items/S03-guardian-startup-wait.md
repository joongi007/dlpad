# S03: guardian 초기화 실패의 유한 launcher 대기 검토

- 상태: done
- 분류: review
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당/수정 범위: metrics_recovery — 이 전용 검토 문서만; process.rs, process/guardian.rs, process/user_service.rs 읽기 전용
- 선행 작업: user-service launcher와 고정 EP readiness 오류 통합

## 사용자 결과

현재는 readiness 실패 이후 자식 launcher가 종료되지 않으면 작업이 계속 막힐 수 있다. 아래 개선안은 구현 전 검토이며, 제품의 유한 종료를 검증 완료로 선언하지 않는다.

## 계약과 범위

Ownership::attach 내부 실패 분기의 child.wait()와 호출자의 재차 child.wait()가 둘 다 무한 대기다. read timeout 설정/해제 오류도 호출자 wait에 들어간다. 초기화 실패는 Ownership을 만들기 전이므로 Drop의 waitpid 정리와는 별개다. 제품 파일, Cargo, manager 및 다른 프로세스는 수정·실행하지 않았다.

권고: attach는 readiness 검증만 담당하고 오류면 control을 drop하여 반환한다. Linux 호출자는 하나의 cleanup_startup_child(&mut Child)에서만 유한 정리를 한다. 예시 정책은 자연 종료 5초, 정확한 unreaped Child kill, 마지막 try_wait 2초, poll 간격 20ms다. readiness 10초에 더하는 최대 7초이며 실제 선택한 값은 상수/인수에 명시해야 한다. try_wait Interrupted는 같은 절대 deadline 안에서 재시도하고, 그 외 오류는 정리 실패로 전달한다. kill 직전 try_wait가 Some이면 kill하지 않으며, None인 자신의 아직 unreaped child에만 Child::kill을 사용한다. 음수 process group, manager 전체, numeric cached PID에 신호를 보내지 않는다. Child 소유권이 단일하고 다른 waitpid(-1)/SIGCHLD reaper가 이 child를 회수하지 않는 계약이 PID 재사용 안전성의 전제다.

호출자는 Linux 오류 분기에서 다시 wait를 하지 않는다. 성공시 기존 정상 process/Ownership 경로를 유지한다. startup readiness 오류를 유지하되, cleanup 실패도 고정 typed context로 함께 나타내며 일반 성공으로 바꾸지 않는다. read timeout(None) 실패도 동일 정리 경로에 들어간다.

최종 try_wait 상한 뒤에도 reap이 불가능한 경우가 남는다(예: 커널 uninterruptible 상태). 마지막에 wait를 호출하면 상한이 사라지고, Child를 그냥 drop하면 zombie 회수 보장이 없다. 구현은 이 경우 Child를 별도 소유 reaper로 이관하거나 소유권을 계속 보존하는 후속 정리 계약을 명시해야 한다. reaper는 제한 없는 실패당 thread 생성 대신 기존 동시성 상한 또는 유한 큐를 사용해야 한다. 이 드문 경계에서 유한 API 반환과 동기 reap 완료를 동시에 무조건 보장한다고 쓰면 안 된다.

## 서비스 정리와의 관계

앱 control peer를 먼저 drop하면 guardian service의 소유 endpoint reader는 EOF를 관찰하여 tool 트리 정리를 시작한다. launcher가 보유한 같은 guardian-side alias는 앱 반대쪽 peer가 닫혔을 때의 EOF를 막지 않는다. service stdin socket과 pipe writer는 manager/service가 보유하므로 launcher만 kill하는 것은 service/tool 트리를 직접 kill하지 않는다. 따라서 순서는 앱 peer drop → guardian EOF 정리 시간을 부여 → 아직 살아 있는 정확한 launcher만 kill이다. manager는 guardian 종료 후 KillMode=control-group으로 잔류 descendants를 정리한다. 이 경로의 실제 cgroup/pipe EOF는 별도 native 증거가 필요하며 launcher reap만으로 대체할 수 없다. manager bus AddRef 해제는 unit Stop 명령이 아니고, 미확인 unit을 이름만으로 Stop해서는 안 된다.

## 인수 조건

1. 개발 helper 모조의 소유 자식이 EP를 보낸 뒤 무기한 sleep: 고정 readiness 오류가 반환되고 정해진 상한 내 child reap; tool/service를 만들지 않는 fixture.
2. 아무 readiness를 보내지 않는 소유 자식: readiness timeout + cleanup 상한 내 반환, 이후 정확한 child 부재.
3. EOF 직후 자연 종료: try_wait Some를 관찰하고 불필요한 kill 없이 원래 오류 반환.
4. malformed/truncated frame, timeout 설정 실패 주입: 같은 cleanup 경로; caller의 추가 blocking wait 없음.
5. 이미 종료/reaped child: cached ExitStatus 경로이며 PID signal 없음. source 계약상 중복 reaper 없음.
6. 실제 service startup fault: 앱 control drop, guardian/worker/setsid leaf 부재, owned cgroup populated=0 또는 제거, pipe EOF를 별도로 확인. unit/PID 식별은 시작 당시 InvocationID/startticks로 동결하고 다른 unit을 건드리지 않는다.
7. 최종 reap 불능: injected Child adapter로 deadline·오류·소유 reaper 이관을 시험한다. 실제 사용자 프로세스를 D-state에 넣지 않는다.

## 검증과 handoff

실제 실행은 sed/rg 읽기만이며 Cargo/제품/fixture 실행은 하지 않았다. root에 유한 정리 helper와 단일 reaping 담당 개선안을 보고했다. 이 문서의 done은 검토 완료이며 구현/실제 테스트 완료가 아니다. root가 상수, 최종 reap 소유권 정책, failure harness 및 통합 검증을 담당한다.
