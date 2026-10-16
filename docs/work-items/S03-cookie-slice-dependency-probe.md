# S03: transient service 의존성에 따른 slice 수명 실험

- 상태: in-progress
- 분류: spike
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — scripts/cookie_slice_dependency_probe.py 및 이 문서
- 선행 작업: runtime StopWhenUnneeded setter 거절 원시 증거

## 사용자 결과와 계약

service를 주 transient unit으로, StopWhenUnneeded=true slice를 auxiliary unit으로 한 transaction에 생성하는 대안을 시험한다. RemainAfterExit=true 서비스는 worker 종료 뒤 active(exited)로 남아 원래 parent의 native empty proof를 읽을 시간을 확보한다. 정확한 소유 service Stop/Unref 뒤 parent의 자동 unneeded Stop/GC를 관찰하며 정상 경로에서 parent Stop을 직접 요청하지 않는다.

합성 owned worker는 2초이며 RuntimeMax 상한은 3초다. live/pending sibling을 새 소유 nonce fixture로 생성하고 owned reader service 종료 후 sibling 생존과 parent 보존을 확인한다. sibling 자연 종료 뒤 parent 자동 GC를 확인한다. 모든 PID/startticks/pidfd, unit/Invocation, job, parent native FD 및 strict empty 원시 증거를 기록한다. 사용자의 기존 unit·데스크톱·미디어는 접근하지 않는다.

## 인수와 정리

empty/live/pending 모두 성공해야 대안을 채택할 근거가 된다. 실패를 성공으로 완화하지 않는다. 실패 잔여물만 정확한 Invocation과 엄격한 empty proof를 확인하고 별도 cleanup Stop을 요청할 수 있다. proof를 얻지 못하면 보존하여 root에 보고한다. 실제 namespace 변경을 수행하는 이 probe는 제품 구현 완료가 아니다.

## 검증과 handoff

실행 전 작업 기록. 결과와 명령은 실행 후 추가한다. Cargo 및 제품 소스는 변경하지 않는다.

## 실제 결과

`python3 -m py_compile scripts/cookie_slice_dependency_probe.py` exit 0. `python3 scripts/cookie_slice_dependency_probe.py /mnt/d/project/dlpad/docs/validation/evidence/cookie-slice-dependency-first` 실제 exit 0, empty/live sibling/pending sibling 3/3 PASS. [실행 source 및 재사용 helper hash와 판정](../validation/evidence/cookie-slice-dependency-first/result.json).

[empty 원시 결과](../validation/evidence/cookie-slice-dependency-first/empty.json)에서 service 주 unit과 auxiliary slice의 transaction을 수락했다. worker 정상 exit0 뒤 service는 active/exited, MainPID0이며 parent는 원 Invocation/native inode로 유지됐다. held cgroup.events/procs 및 fresh openat 모두 populated0/빈 procs를 확인했다. reader service만 Stop/Unref한 뒤 parent는 자동 inactive/dead가 되고 Unref 뒤 GetUnit ENOENT 및 cgroup 경로 부재를 확인했다.

[live sibling](../validation/evidence/cookie-slice-dependency-first/sibling.json)과 [pending sibling](../validation/evidence/cookie-slice-dependency-first/pending.json)은 reader service Stop 뒤 sibling의 원 Invocation/PID/startticks 생존 및 pidfd 미종료와 parent active를 유지했다. pending Start job도 그대로 보존됐다. sibling 자연 종료 후 원 PID 부재/pidfd 종료를 확인하고 sibling Unref, parent 자동 inactive 및 Unref 뒤 GC를 확인했다. parent를 직접 Stop한 정상 요청은 0이며 실패 정리 경로도 실행되지 않았다. 각 case 자체 FD 수는 4→4다.

이 실험은 합성 2초 worker와 3초 runtime cap의 제한된 fixture다. 제품 timeout 정책을 3초로 채택한 것이 아니며 실제 guardian/C/I/B/receipt/앱 crash 경로의 통합 인수는 root 후속 작업이다. sibling은 시험이 소유한 별도 합성 unit으로 사용자 기존 unit을 변경하지 않았다. 정확한 unit/Invocation 대조와 Stop 호출 사이의 manager namespace 경쟁 자체를 원자적 권한 경계로 주장하지 않는다. 기존 runtime setter 실패를 성공으로 변경하지 않고 생성 시 속성을 사용하는 독립 대안을 입증했다.

## root 독립 실행 원시 재검토

stress_recovery가 root의 `cookie-slice-dependency-root-first/result.json`과 empty/sibling/pending 개별 원시 및 `/tmp/dlpad-cookie-slice-dependency-root-first.log`를 읽기 대조했다. 로그는3case true, 원시 모두 completed/passed=true, FD4→4, normal_parent_stop=false, failure_cleanup=[]다. source `e1265df52a182a38b49cd9cb6f254198db43837bbe5b26fd57e80a396959e33c`와 result의 재사용 helper4개 SHA를 현재 실제 파일 SHA와 각각 대조해 일치를 확인했다. 재실행이나 source 변경은 하지 않았다.

모든 reader terminal은 원 ExecMainCode1/Status0, active/exited/MainPID0이며 원 PID 소멸/pidfd 종료를 확인했다. strict empty는 held/fresh openat 모두 populated0와 빈 procs를 동시에 만족한다. reader와 parent GetUnitENOENT 및 parent cgroup 부재가 원시에 있다. live sibling은 release 전후 active, 동일 Invocation/MainPID 유지; pending은 activating과 동일 Job5187 유지다. parent는 동일 Invocation 및 active를 유지했고 sibling 자연 종료 뒤 sibling/parent GC를 확인했다. 원시 PID/start 및 native identity와 probe의 실제 assertion도 대조했다. 이는 root 독립3case를 재검토한 결과이며 사용자 기존 foreign unit 시험을 새로 수행한 것이 아니다.

제품 채택 인수는 생성 transaction에서 auxiliary slice StopWhenUnneeded=true와 reader RemainAfterExit=true를 함께 적용하여 native parent proof 수명을 확보하고, terminal 원 종료값과 pidfd 및 strict native empty를 확인한 뒤 durable receipt를 먼저 확정하는 것이다. 이후 정확한 reader service만 Stop/Unref하고 parent 직접 Stop을 요청하지 않아야 한다. foreign 또는 queued sibling은 Stop 대상이 아니며 같은 manager dependency로 보존되어야 한다. 합성 owned sibling의3case는 이 정책의 실험 근거지만 실제 제품 C/I/B/receipt/취소/강제종료/same-boot crash 및 진짜 외부 queued sibling 통합을 완료했다고 표현하지 않는다. 같은 unit 이름에 대한 Invocation 조회와 Stop 사이의 경쟁은 별도 소유권 검토가 필요하다.
