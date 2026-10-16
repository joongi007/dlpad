# S03: StopWhenUnneeded runtime 전환으로 parent release 경합 제거 실험

- 상태: blocked (manager runtime property 거절; 후속 후보 필요)
- 분류: native-experiment
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery; 신규 scripts/cookie_slice_release_probe.py 및 이 문서, 전용 evidence
- 선행 작업: dedicated slice native empty 수명 root 독립 확인

## 사용자 결과

strict empty 관측과 parent StopUnit 사이의 sibling/start 경합을 피하는 후보로, exact parent runtime StopWhenUnneeded=true→Unref만 사용한다.

## 계약과 범위

random dedicated user slice와 고정2초 합성 service만 생성한다. empty 증명 뒤 empty/live Typeexec sibling/pending Typeoneshot sibling 세 case를 구분한다. pending은 실제 activating 및 nonzero Job을 확인한다. 모든 worker는2초 정상 종료, TimeoutStart/RuntimeMax3초 상한이다. sameInvocation runtime SetProperties 및 samebus Unref만 수행하며 parent/service StopUnit, PID signal, 외부unit검색, global설정, filesystemcgroupwrite, Cargo/Rust/기존fixture/8h 변경은 없다. 기존 lifetime/dedicated probe는 읽기 import만 재사용한다.

## 인수 조건

빈 parent 정상autostop/GC. 실제 sibling PID/start/pidfd 및 serviceInvocation과 parentInvocation/nativeidentity를 release 전후 확인하고, sibling 또는 pending Job이 있을 때 parent가 활성으로 유지된다. sibling 정상 종료 및 own AddRef 해제 뒤 GetUnitENOENT/두cgroup부재/FD baseline 확인. populated1+빈parentprocs를 empty로 승인하지 않는다. beforeI/source/start 실패는 identity없는unit을추정Stop하지 않고 신규raw에미확인범위를남긴다.

## 검증과 handoff

실제 명령: `python3 scripts/cookie_slice_release_probe.py /mnt/d/project/dlpad/docs/validation/evidence/cookie-slice-release-first` → exit 2. Python AST 파싱 PASS. Cargo 및 제품 변경 없음.

첫 empty case에서 정상 합성 worker exit0, PID/start 소멸, pidfd readiness, parent의 held/new-openat populated0+procs0, proof service Unref 후 GetUnitENOENT를 확인했다. 그러나 Unit.SetProperties(runtime=true, StopWhenUnneeded=true)는 고정 errno30(EROFS)으로 실패했다. 런타임 변경과 parent Unref 단계는 완료되지 않았으며 sibling/pending 두 case는 실행하지 않았다. FD4→4. 성공한 앞부분으로 release 후보를 PASS 처리하지 않는다.

원시 `evidence/cookie-slice-release-first/empty.json` 및 `result.json`은 변경하지 않는다. 정확 부모 `dlpadcookie43d7dc21120c4a96bb83629eea9421a0.slice`는 후속 읽기 조회에서도 Invocation `48b9f7539a544041b00338f6a300f5a0`, active, StopWhenUnneeded=no, 원래 ControlGroup을 유지했다. 시험은 직접 parent StopUnit을 금지하므로 별도 identity+strict empty 검증 cleanup을 통합 담당에게 요청했다. 이 시점에 모든 owned unit 정리를 완료했다고 주장하지 않는다.

현재 manager에서 runtime setter 후보는 사용할 수 없음이 관찰됐다. 생성 시 StopWhenUnneeded=true로 시작하는 별도 후보의 parent 수명/receipt proof 확보 가능성, queued start 보존 정책은 root가 검토하며 본 시험으로 구현 완료나 제품 채택을 주장하지 않는다.

## 실패 잔여물 정리와 공식 소스 대조

통합 담당 승인 후 실패 cleanup만 별도로 수행했다. 정확 기존 Invocation/ControlGroup과 pinned mount121/dev24/inode18110을 재검증하고 held/new-openat strict empty를 다시 확인한 뒤 이번 부모만 Stop했다. `evidence/cookie-slice-release-first/cleanup.json`은 실험 directStop=false와 실패 잔여물 cleanupStop=true를 분리한다. 최종 GetUnitENOENT 및 parent cgroup 부재 확인. 실제 worker 정상 종료와 proof service GC는 첫 원시에 보존돼 있다.

현 환경은 systemd249(249.11-0ubuntu3.22)이다. [공식 v249 dbus-unit.c](https://raw.githubusercontent.com/systemd/systemd/v249/src/core/dbus-unit.c)의 bus_unit_set_transient_property는 StopWhenUnneeded를 생성 시에만 처리하며, bus_unit_set_properties는 transient && UNIT_STUB일 때만 이 경로를 사용한다. 이미 loaded인 본 부모는 read-only property 오류로 귀결되는 구조이며 실제 errno30과 일치한다. 파일시스템이나 보안 설정을 변경할 근거가 아니다. v255 공식 소스에서도 동일 생성 시 제한을 확인했다. Ubuntu downstream 소스 자체는 직접 대조하지 않았으므로 errno의 상세 오류문자열까지 수집했다고 주장하지 않는다. 공식 man 페이지 fetch는403으로 실패했고 공식 tagged source로 대조했다.

문서 검사 `python3 scripts/check_docs.py` 실제425문서 PASS. script SHA256 `f48a29717477cd4cf375be4bf6ae04b25846f69498e045cfa033bf9e3eb55f28`. 소스 freeze; 후속 후보는 통합 담당 계약 전 실행하지 않는다.
