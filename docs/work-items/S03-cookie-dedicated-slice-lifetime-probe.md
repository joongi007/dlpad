# S03: 소유 dedicated slice의 reader container 수명 실험

- 상태: completed (container 수명 실험 완료, 제품 채택 별도)
- 분류: native-experiment
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery; 신규 scripts/cookie_dedicated_slice_probe.py, 기존 cookie_cgroup_lifetime_probe.py의 선택적 Slice 인수, 이 문서 및 신규 evidence
- 선행 작업: service AddRef/RemainAfterExit 두 실제 관측에서 cgroup 삭제 및 empty 읽기 불가

## 사용자 결과

다른 공유 slice를 쓰지 않고 random dedicated transient user slice가 정상 reader service 종료 후 native empty 증거를 유지하는지 확인한다.

## 계약과 범위

같은 retained user sd-bus에서 random slice(AddRef=true/StopWhenUnneeded=false)와 그 Slice에 고정2초 service(RuntimeMax3초/Delegate=false)를 생성한다. unit 이름에 implicit shared parent slice를 추가하지 않도록 slice 이름은 hyphen 없는 random prefix다. slice/service Invocation, 서비스 Slice 속성·membership·하위 cgroup과 부모 mount identity를 전후 확인한다. cgroup 쓰기/신호/foreignunit/설정/관리자/Rust/Cargo/기존8h 변경은 없다.

## 인수 조건

active 동안 parent/service dir/events/procs를 고정한다. service가 정상 inactive이고 exact savedPID가 종료한 뒤에도 sameInvocation parent active에서 populated0+빈procs를 동시에 읽을 수 있는지 기록한다. parentprocs가 비어도 populated1이면 승인하지 않는다. service cgroup 삭제와 parent 수명을 분리한다. same identity serviceUnref→parentStop→parentUnref→두unitGC/FD baseline을 확인한다. 실패도 신규 raw로 보존한다.

## 검증과 handoff

작성 중. 실험 성공은 container 수명 가능성만 확인하며 현재 service 중심 receipt shape에 container identity를 bind하는 정책은 root ADR/인터페이스 결정 후 구현한다.

## 실제 관측과 정리

`python3 scripts/cookie_dedicated_slice_probe.py /mnt/d/project/dlpad/docs/validation/evidence/cookie-dedicated-slice-first`의 첫 실제 실행은 exit2였다. service 정상 종료 뒤 sameInvocation active dedicated slice의 held/new-openat 모두 populated0+빈procs를 확인했지만, slice Stop/Unref 뒤 systemctl show의 LoadState=not-found를5초 내 확인하지 못했다. slice 이름은 모두 hyphen 없는 dlpadcookie<32hex>.slice이고 implicit shared parent slice를 새로 만들지 않았다. 첫 실패 raw를 수정하지 않았다.

slice는 이름으로 조회하면서 implicit unit이 로드될 수 있어 GC 판정에 systemctl show를 쓰는 한계가 있다. 생성 없이 조회하는 같은 bus Manager.GetUnit→ENOENT 기준으로 바꿔 fresh `cookie-dedicated-slice-second`를 실행했고 실제 exit0/PASS였다. 현재 실험은 기존 service 특성을 바꾸지 않으며 선택적 Slice 지정에만 Delegate=false를 추가한다.

두 번째 active parent는 populated1, procs0bytes였고 strict empty=false였다. 이는 부모 direct PID목록만 비었다고 자손을 정리 완료로 오인하지 않는 실제 증거다. 정상 service exit0/Resultsuccess/Delegate=no/같은 service Invocation 이후 부모는 같은 slice Invocation 및 active 상태를 유지했다. parent native dev24/inode16317/mount121 및 경로가 그대로이고 held events/procs와 pinned dir 새 openat 모두 populated0/procs0bytes가 읽혔다.100ms 뒤에도 동일했다. service cgroup 자체는 삭제되어 held ENODEV19/newopenat ENOENT2였으며 parent 수명과 분리됐다.

saved worker PID/start 부재·pidfdready, service Unref→GetUnitENOENT, sameInvocation slice Stop→inactive→Unref→GetUnitENOENT 및 FD4→4를 확인했다. 첫/둘째 실험 모두 별도 create-only cleanup-follow-up.json에 exact unit GetUnitENOENT와 parent/service cgroup 경로 부재를 재확인했다. 원시 result/properties/두 Invocation/source SHA는 각 fresh evidence에 보존했다. 외부 unit 검색/Stop/프로세스 신호/cgroup 쓰기/설정/Rust/Cargo/기존8h 변경은 없다.

최종 신규 script SHA `515d6c3e93eae56dcc8dbaf415d1ff745556271c1ec423d05f1f1c34a03022ac`, reused lifetime script SHA `c186feec418e8d8c5ddff5e91f5c8ea493e5d0691502fe0a3695b8994259e950`로 인계한다. 이 실험은 현재 사용자 manager에서 dedicated container 수명 가능성을 입증한 범위다. 실제 launcher/app crash·reader receipt·container/service identity 동시 binding·재접속 복구·다른 OS 검증은 미수행이다. 현재 receipt shape 변경 및 ADR 채택은 root가 독립 검증한 뒤 결정한다.
