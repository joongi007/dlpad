# S03: Linux 사용자 systemd/cgroup 준비 상태 읽기 전용 조사

- 상태: done
- 분류: research
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: soak_recovery_audit; 이 workitem, docs/validation/systemd-user-readiness.md, 전용 evidence/systemd-user-readiness-20261003/
- 선행 작업: S03-app-subreaper-fallback-review, guardian fault 실제 실패

## 사용자 결과

guardian SIGKILL 보완 후보가 현재 사용자 권한에서 실험 가능한지 확인하고 stdin FD/EOF/cgroup 종료 정책의 남은 계약을 명시한다.

## 계약과 범위

기존 Proposed 검토와 ADR-0002/ARCHITECTURE/R08을 읽었다. 현재 boot/본인 프로세스 cgroup, known user runtime socket, systemctl version 및 선택 manager/user@UID property, sysfs metadata만 조회한다. 전체 unit 목록/환경/credentials 출력, transient unit 생성, cgroup write, process signal/stop/kill/restart, 기존 8시간 시험 변경은 없다. 공식 systemd primary 자료로 후보를 검토한다. 제품/Cargo/sharedsource는 root 담당이다.

## 인수 조건

sandbox view와 실제 host view를 구분한다. readonly query 실제 exit/metadata를 보관하고 준비 상태와 권한 미검증을 분리한다. Proposed를 채택/구현으로 보고하지 않는다.

## 검증과 handoff

host read-only 5개systemctl/systemd-run query 모두exit0. systemd249 user manager running/Delegate=yes 및사용자cgroupwrite metadata를확인했고현재/non-systemd와cgroup.kill 직접쓰기불가는구분했다. 공식v249 stdFD direct/MainPID/service-stop/KillMode계약과literalargv 환경확장경계를문서화했다. 설치·unit생성·cgroupwrite·signal·기존8h변경없음. readiness 연구만완료이며실제 crash/cleanup/FD/literalargv와보호기능인수는후속합성실험대상이다.
