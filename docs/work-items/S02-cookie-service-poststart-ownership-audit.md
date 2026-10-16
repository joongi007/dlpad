# S02: gated transaction 수락 직후 소유권 확보 실패 감사

- 상태: reviewed (읽기 감사, 후속 구현 미완료)
- 분류: security-review
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery; 이 문서만
- 선행 작업: gated auxiliary parent 의존성 통합 source freeze

## 사용자 결과와 계약

RemainAfterExit=true로 확보한 receipt proof 수명이 identity capture 전 실패에서 service 잔여물로 바뀌는 범위를 구분한다. unit 이름만으로 추정 Stop하지 않는다. Rust/Cargo/native 실행 및 시스템 설정 변경 없음. 근거 source는 user_service.rs SHA2d44325f79ea678a1924bd62a2b69fa4f5f0d0bff4a4fc0c54b400085880ae49와 현재 guardian.rs의 launcher endpoint shutdown 및 before-B gate다.

## 확인된 소유권 경계

run_gated의 start 성공 뒤 get_unit 또는 bus.invocation 실패까지 ReaderService guard가 없다. start 자체도 manager call 수락 후 job reply framing/read 실패하면 accepted 여부를 확정하지 못하고 Err가 될 수 있다. 이 구간에는 service/auxiliary parent AddRef가 manager에 생성됐을 수 있으나 신뢰할 service Invocation이 없다. 무작위 unit nonce만으로 정리 권한을 대체해서는 안 된다.

서비스 Invocation을 확보하여 ReaderService를 구성한 이후 parent capture 실패 또는 native PID capture 실패는 기존 guard Drop이 exact Invocation service Stop→wait_cleanup→Unref를 시도한다. parent capture 실패 전에는 ReaderContainer guard가 없지만 bus close는 같은 client AddRef를 해제하며 SWU parent는 service 의존성이 사라진 경우 자동 종료 후보가 된다. 이 fallback의 실제 manager 실패 인수는 아직 별도다.

launcher는 실패 때 유지한 CLOEXEC startup_control alias로 LU/EP를 기록하고 shutdown(Both)한다. 단순 close와 달리 manager가 복제해 보유한 동일 endpoint에도 shutdown 상태를 전달하도록 기존 코드가 구성됐다. guardian beforeB는 sole wait_begin_only의10초 absolute budget/EOF에서 실패하며 도구는 시작되지 않는다. 따라서 이 경계에서 tool 실행 권한은 부여되지 않지만, ignore-failure+RemainAfterExit=true가 guardian 실패를 관리 active(exited)로 유지할 수 있어 service 잔여물이 별도 문제다.

## AddRef와 parent 자동 종료가 해결하지 않는 범위

bus close는 client 참조를 제거하지만 active(exited) service의 관리 active 상태를 자체적으로 Stop하지 않는다. CollectMode inactive-or-failed는 active service의 자동 회수를 보장하지 않는다. parent SWUtrue도 이 active dependency가 남아 있으면 이를 보존한다. 따라서 FD 해제/guardian EOF/tool0만으로 service/parent GC를 완료했다고 주장하면 안 된다. 첫 identity 전 bus 장애나 accepted reply 손상은 안전하게 보존하되 명확한 미확인 cleanup 상태로 보고할 필요가 있다.

## 후속 최소 해결 후보와 인수

우선 accepted reply 직후 service GetUnit/Invocation capture를 짧은 absolute budget 안에서 bounded retry하여, 일시적인 조회 실패로 cleanup guard가 없는 구간을 줄이는 후보가 적합하다. 재시도 성공 시 원 service Invocation을 guard에 고정하고 그 뒤 parent/native capture를 수행한다. object path/name이 재사용됐다면 무조건 같은 작업으로 승인하지 말고 transaction job과 service 실행 command/nonce, native guardian PID/start 및 control 관계 등 신뢰 가능한 binding을 root 계약으로 정해야 한다. 조회 실패 자체가 안전한 재시도 가능 여부와 신뢰 근거는 구현 전에 구분한다. 조회가 영구 실패하면 무명 identity로 Stop하는 fallback은 없다.

더 강한 후보는 service guardian이 before-B에서 자신이 속한 service Invocation/native PID identity를 bounded protocol로 일찍 알려주어 launcher가 manager 조회를 교차 확인하는 것이다. 현재 I receipt schema는 launcher가 제공하므로 이 역방향 binding은 새 공유 계약 없이는 구현하지 않는다. 독립 manager/guardian 보조 cleanup이 필요하면 정확 Invocation과 own endpoint lifetime에 묶여야 하며 임의 unit pattern 검색 cleanup은 금지한다.

고정 RuntimeMaxSec를 단순 추가하면 실제8시간 이상 작업의 제품 timeout 정책을 훼손할 수 있다. 또한 active(exited)에서 그 timer가 동작하는지는 본 감사로 실증하지 않았으므로 잔여물 해결책으로 주장하지 않는다. ExecStop/ExecStopPost는 Stop/종료 lifecycle에 연결되는 후처리이며 service가 active(exited)에 남는 최초 문제를 자동으로 해결한다고 확인하지 않았다. service StopWhenUnneeded를 처음부터 켜는 후보는 active 실행까지 필요한 dependency가 없으면 조기 Stop할 위험이 있어 native 대조 없이 채택하지 않는다. global watchdog·재부팅·보안 설정 변경은 후속 해결 후보가 아니다.

인수 fixture는 accepted StartTransientUnit 뒤 GetUnit/Invocation 각각의 일시/영구 failure, job reply parsing failure, parent/native capture failure를 구분한다. beforeB tool0, 원 소유 PID/start 종료와 guardianEOF, unknown identity의 Stop0, known Invocation service만 정리, foreign/queued sibling 보존, unit/cgroup GC 또는 정확한 보존 상태를 원시로 확인해야 한다. 정상 long 작업 timeout/기존8h 경로는 변경하지 않는다.

## 검증과 handoff

읽기 source 감사만 수행했다. 실제 failure injection/native 재실행 및 보완 구현은 미완료다. root 통합 검증 session68320과 병렬로 코드/source freeze를 유지했다.
