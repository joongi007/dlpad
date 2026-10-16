# S03: Linux 사용자 manager native backend 계약 검토

- 상태: done
- 분류: architecture-review
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당/수정 범위: metrics_recovery — 이 workitem 및 docs/validation/systemd-native-backend-contract.md만
- 선행 작업: systemd user readiness/transient/no-env prototype 실제 근거; root 공유 인터페이스 결정

## 사용자 결과

guardian 자체 고장에도 소유 자손을 독립 사용자 manager의 cgroup으로 정리할 후보의 실제 구현·인수 계약을 정한다.

## 계약과 범위

제품source/Cargo/schema/다른문서는읽기전용. 직접child launcher와 실제서비스MainPID guardian 분리, 기존 std::process::Child/stdout/stderr/wait/R·T·K/EOF 계약 유지. libsystemd dynamic호출은Proposed이며새Rustdeps없음. no-manager/no-env/권한실패는약한fallback없이failclosed. 관리자변경/기존서비스변경/새unit실험/하위spawn없음.

## 인수 조건

FD소유/중복해제·메시지ABI·literalargv·환경/credentials·nonce/invocation·statusGC race·유한timeout·OOM·동시죽음과두job/무관sentinel 보호의실제시험조건을작성한다. 기존prototype가증명한것과추가제품미검증을분리한다. contract검토완료를backend구현완료로쓰지않는다.

## 검증과 handoff

cached제품process/guardian 및실제prototype JSON/source와systemd v249 primary API를읽고 계약문서화한다. 제품코드작성/빌드/nativebackend검증은root 인터페이스채택후별도배정이다.


## 검토 결과

docs/validation/systemd-native-backend-contract.md에 실제CLI false/no-env true 경계, 직접Child/MainPID 분리,dupFD/EOF/AddRef GC/status proxy/typed failure/환경/dynloadABI/deadline 및native 실패시험을기록했다. root가공유계약과생산구현을별도조정한다. 핵심새위험: AddRef 없이fast-exitstatus GC race, message h 복제FD 누수와managerretainedstdio에의한EOF 지연, 기존Child.wait무한대기, backend126/tool126혼동. 이번검토에서는실제unit/제품code/빌드/새native시험없음.
