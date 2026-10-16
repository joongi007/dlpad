# S03: 정상 실행과 unwind의 유한 Child 소유 수명

- 상태: in-progress
- 분류: fix
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: metrics_recovery — crates/engine/src/process.rs 및 이 문서; root — 통합 검증, 공유 계약 문서
- 선행 작업: S03-ready-process-error-ownership; 기존 bounded startup reaper

## 사용자 결과

취소·오류·worker panic에서 무한 wait 없이 소유 프로세스 정리를 요청하고, 회수되지 않은 Child는 제한된 reaper에 넘긴다. 원 Child를 회수한 뒤 재사용된 숫자 PID로 다른 자식을 기다리지 않는다.

## 계약과 범위

Linux의 raw PID waitpid Drop을 제거하고 Child와 기존8개 reaper permit을 RAII로 함께 소유한다. spawn 직후 guard를 구성한다. 제어 socket write에 유한 timeout을 적용한다. 취소 Child.wait를 bounded try_wait로 바꾸며, 확인이 부족하면 typed 미확인을 유지한다. unwind Drop에서는 무한 wait/긴 grace/락 대기 없이 소유 peer shutdown과 정확한 Child만 처리한다. 미회수 Child 이관은 이미 예약된 슬롯을 사용한다. 새 schema/Cargo/상시 daemon/thread를 추가하지 않는다. 전체 실행 수명동안8슬롯을 유지하는 정책 변경은 병렬 회귀로 검증한다.

## 인수 조건

정상 실행·취소·timeout·outputlimit·pipe 오류의 기존 결과 유지, 미확인 종료는 쿠키 보존, 원 Child 회수 후 숫자 PID wait 없음, panic시 pending Child 이관과 정확한 슬롯 반환, admission 상한 및 병렬시험 통과. 실제 native service22case와 private GUI는 root 후속검증이며 unit PASS로 대체하지 않는다.

## 검증과 handoff

읽기 감사에서 정상 Drop의 blocking waitpid, 취소의 Child.wait, 무제한 socket write를 확인했다. 실제 코드는 구현·검증 대기다. 직접 guardian startup 소유권 분류 수정은 먼저 root가 회귀했다. 동일 process.rs를 root와 동시 수정하지 않는다.


## 구현 동결 handoff

Linux OwnedChildLifetime은 spawn 직전 준비한 전용 app-side shutdown alias와 Child/기존 StartupPermit을 spawn 직후 함께 소유한다. 정상 ready 후 permit을 해제하지 않고 전체 Child 수명동안 유지한다. alias는 control mutex를 얻지 않고 unwind에서 동일 소유 peer를 SHUT_RDWR할 수 있다. startup attach의 기존 모든 오류 typed StartupUnconfirmed 정책과 5초/2초 cleanup은 보존했다.

Ownership의 cached PID 필드와 Drop waitpid를 완전히 제거했다. 따라서 원 Child를 try_wait로 회수한 뒤 다른 자식에 재사용된 PID를 기다리지 않는다. control write timeout은1초다. 정상 실행 결과는 원 try_wait status→pipe drain→terminal GC/LC confirmation→기존 결과 구성 순서를 유지한다. 함수 반환 시 이미 회수된 Child의 cached status를 확인하고 permit을 해제한다.

Linux 취소 경로의 Child.wait는5초 try_wait 자연 정리, 그 뒤 owned peer EOF와 정확한 unreaped Child kill 및2초 최종 try_wait로 바꿨다. 최종 미회수 또는 관측 오류는 즉시 기존 예약으로 reaper transfer하여 추가 grace를 반복하지 않는다. ready 이후 ordinary error에는 StopDrains scope 종료 뒤 소유 peer shutdown과5+2 cleanup을 적용한다. terminal 미확인의 root 분류는 유지한다. 정상 confirmation 이후 결과 구성 오류는 이미 Child가 회수돼 있어 원래 오류를 유지할 수 있다.

unwind Drop은 긴 grace나 blocking wait를 하지 않는다. alias shutdown, try_wait1회, None인 정확한 unreaped Child의 best-effort kill 뒤 transfer, 상태 오류면 추측 신호 없이 transfer한다. guardian control mutex를 얻지 않는다. 기존 reaper queue transfer/permit 반환의 짧은 내부 Mutex critical section은 재사용한다. 그 구간은 user I/O/terminal read를 포함하지 않지만 kernel scheduling까지 hard realtime이라고 주장하지 않는다. 새 queue/thread/schema/dependency는 없다.

내부 회귀2개를 작성했다: owned_child_unwind_closes_peer_and_reaps_exact_child는 실제 합성30초 child를 catch_unwind에서 guard Drop한 뒤 peer EOF와 원 PID/startticks 부재를5초 poll로 확인한다. owned_reaped_child_drop_releases_its_lifetime_reservation은 원 Child 회수 후 local reserved1→0/pendingempty를 확인하고 globalreserved=0을 가정하지 않는다. 실제 시험은 아직 하지 않았다.

source SHA256 `f8698f7a6ec5fced098f7d3630a70bcca915077643512d7855c21beaa5f2afb4`, rustfmt exit0. Cargo는 root 독점이며 미실행이다. whole-lifetime8 슬롯이 병렬 longprocess fixture에서1초 admission과 경합할 가능성을 root에 보고했다. 상한을 임의로 늘리거나 테스트 병렬도를 줄이지 않고 실제 default workspace 실패/성공으로 판단해야 한다. native terminal22/private GUI도 별도 통합 인수다.

## root 최초 통합 결과

전체fmt exit0, defaultClippy exit101: cfgLinux block의불필요return한개로실패했다. [원시 실패](../validation/evidence/owned-child-reader-workspace/default-clippy.log)를보존한다. root가tail-expression으로수정하고전체동일병렬조건재시도한다. 병렬작성중인unwiredWindowscookiebackend는컴파일/검증범위밖으로명시하며사용된공유sourceSHA는별도고정한다.

## root 전체 Linux 통합 결과

[별도 재시도 원시 결과](../validation/evidence/owned-child-reader-workspace-retry/workspace-result.json)에서fmt/defaultClippy/defaulttest/CPUClippy/CPUtest/defaultrelease/CPUrelease모두exit0. 대상소스시작끝SHA일치, 원래병렬조건유지. 공통parser6시험과Childunwind/reapedDrop2시험을root가로그대조했다. CPUbinary12515320B/SHA`d0451487a8127f6891ccf377e8aa1826fe9261b4c554e6278c791b23a3d9c3a5`를[immutable copy](../validation/evidence/owned-child-reader-workspace-retry/immutable-app.json)에고정했다. 신규unwiredWindowsbackend는컴파일/검증범위밖이다. 실제native22/GUI/Windows/macOS와readiness전체deadline은후속인수다. 최초Clippy실패raw는보존했다.

## startup alias EOF 보강 동결

spawn 전 복제한 lifetime shutdown alias가 살아 있으므로 attach 오류에서 원 control descriptor만 drop하면 guardian peer에 EOF가 전달되지 않는다. Linux attach Err 분기에서 cleanup_startup_child 직전에 child.shutdown_peer()를 명시하여 동일 endpoint 전체를 SHUT_RDWR한다. 자연 정리5초/kill2초 및 typed StartupUnconfirmed 정책은 유지한다.

신규 startup_failure_shutdown_reaches_peer_with_lifetime_alias_alive는 동일 실제 socket을 상속한 합성 Python child가 EP를 전송한 뒤 recv EOF를 기다린다. attach 원 descriptor가 닫힌 뒤에도 child가 살아 있음을 확인하고 lifetime alias shutdown 후2초 자연 정리로 exit0을 확인한다. 강제 kill은 성공 exit가 아니므로 EOF 전달을 대신하여 시험을 통과할 수 없다. 사용자 데이터·desktop·서비스 manager를 사용하지 않는다.

process.rs SHA256 `65cc90ebea49b9a94010d93b206e6231e43ce23fc94c8dd7e869f3365e2abaf9`, rustfmt exit0. Cargo는 담당 통합 root가 수행할 예정이며 신규 회귀의 실행 성공을 아직 주장하지 않는다. 앞선 전체7명령/immutable CPU 및 진행 중 native22는 이전 snapshot 근거이므로 이번 변경의 검증으로 확대하지 않는다.

## root shutdown alias 실제 시험

`cargo test -p dlpad-engine --lib --locked startup_failure_shutdown_reaches_peer_with_lifetime_alias_alive` exit0 및 engineall-targetClippy exit0. [원시로그](../validation/evidence/startup-shutdown-alias/result.json). 동일socket의실제child가EOF뒤정상exit0으로종료한시험이며kill성공으로대체하지않았다. 이전native22는patch전불변snapshot이다. [root22독립검토](../validation/evidence/owned-child-terminal-native-20261004/root-review.json)에서원시RGCLC/원exit0·42·126,typed미확인,78PID·14cgroup부재를대조했다.
