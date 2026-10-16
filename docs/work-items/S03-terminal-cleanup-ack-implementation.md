# S03: guardian와 service terminal 정리 확인 구현

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R06, R08, R11, R13
- 담당/수정 범위: metrics_recovery — process/guardian.rs, process/user_service.rs, 이 문서
- 선행 작업: terminal-cleanup-ack-review; root process.rs 통합, stress terminal parser

## 사용자 결과

실제 tool exit126과 보호 정리 실패를 stderr 없이 구분할 수 있도록 guardian GC/GU와 launcher LC/LU를 고정 소유 socket에 전달한다.

## 계약과 범위

GC는 supervise run/killed/reaped/status 모두 성공한 경우만, 그 외 소유 endpoint가 있으면 GU. launcher run 성공은 LC, 실패는 LU 뒤 기존 EP/shutdown. genuine tool exit126은 GC와 함께 원status 유지. command은 동일 validated backend marker로 expects_service bool을 반환한다. known cgroup evidence가 없으면 LC를 성공으로 내보내지 않는다. frame2byte/write1초 상한, FD CLOEXEC, 새 stderr 해석/PID 신호 없음. parser/process/Cargo는 root와 stress 소유다.

## 인수 조건

root 직렬 Cargo/native 회귀에서 genuine126, guardian failure, LC/LU, frame 누락, partial startup, cgroup 증거 부재, endpoint/pipe/lease 수명과 기존17+2case를 확인해야 한다. 단위/문서 통과로 실제 native 완료를 대체하지 않는다.

## 검증과 handoff

구현 전 범위 기록이며 현재 미구현·미검증이다. freeze 후 root 통합/실제 시험을 요청한다.


## 동결 handoff

supervise는 소유 socket 확보 뒤 전체 setup/run/killed/reaped/status 결과를 closure로 묶고 모두 Ok면 GC, 하나라도 Err이면 GU를 고정2byte로 보낸다. genuine tool exit126은 Ok(status)이므로 GC와 원status를 유지한다. launcher는 alias를 run 전체에 유지하고 Ok에 LC, Err에 LU 뒤 기존 EP/shutdown을 수행한다. 각 terminal write는1초 timeout이다. 실패 상세/경로/PID/쿠키를 frame에 넣지 않는다.

command의 반환은 (Command, app, guard, expects_service)로 확장했다. 기대 bool은 같은 validated backend marker의 LAUNCHER 여부에서 결정하며 환경을 다시 읽지 않는다. wait_unit은 known_group가 전혀 관측되지 않았다면 terminal에서도 fail-closed하고, 관측된 same-unit cgroup이 empty/removal이어야 정상LC가 가능하다. 너무 빠른 version 실행에서 group identity를 한번도 관측하지 못하는 경우는 native 실제 회귀로 판단할 남은 gate다.

read_readiness는 R 이전 GU/LU도 root OwnershipCleanupError::TerminalUnconfirmed로 typed 반환한다. EP 기존 fixture 계약을 보존한다. source-only unit3개(frame2byte/write상한/CLOEXEC, missing peer, preR GU/LU typed/downcast/secret 미출력)를 작성했다. 실행하지 않았다.

현재 guardian SHA256 `b37d41cb5f43bcbf644409ffcab6bd01195b1c2710be853ad845b3c9aa4029af`, user_service SHA256 `ab8699d3b5a99930049d61e4864c8088eb946086001c8b691d4b5b85aa0c6040`, rustfmt exit0. Cargo와 native fixture는 root 직렬 통합 대상이다. root process와 stress parser가 각각 기대state/프레임 생략/terminal typed 결과를 연결해야 하며 이 두 파일의 변경만으로 전체 기능이 완성되지 않는다.
