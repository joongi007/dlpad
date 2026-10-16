# S03: 실패 및 signal 종료의 slice 의존성 실험

- 상태: in-progress
- 분류: spike
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — scripts/cookie_slice_failure_dependency_probe.py 및 이 문서
- 선행 작업: auxiliary slice 의존성 실제 3case 성공

## 계약과 인수

ExecStartEx의 no-env-expand 및 ignore-failure로 service 관리 상태와 원래 프로세스 종료 상태를 분리한다. [공식 upstream flag parser](https://github.com/systemd/systemd/blob/main/src/shared/exec-util.c)는 두 문자열을 각각 지원한다. 실제 설치된 manager가 이를 지원하는지는 별도 native 실행으로 판정한다.

Python worker는 2초 뒤 exit7 또는 자기 PID에 SIGTERM을 보내며 RuntimeMax는 3초다. 종료 이후 ExecMainCode/Status의 원래 1/7 또는 2/15, service active/exited/MainPID0 및 원 parent의 held/fresh openat strictEmpty를 요구한다. service만 정확한 Invocation에서 Stop/Unref하고 parent의 자동 종료/GC를 확인한다. 원 exit 값이나 제품 core 결과를 성공으로 바꾸지 않는다. 정상 parent 직접 Stop은 금지이며 실패 잔여물만 same Invocation+strictEmpty 확인 뒤 정리할 수 있다. 기존 unit·데스크톱·Cargo·제품 코드는 변경하지 않는다.

## 검증과 handoff

실행 전 계약 기록. 이전 probe/evidence는 동결 유지한다. 결과를 성공 조건에 맞춰 완화하지 않는다.

## 실제 결과

`python3 -m py_compile scripts/cookie_slice_failure_dependency_probe.py` exit 0. `python3 scripts/cookie_slice_failure_dependency_probe.py /mnt/d/project/dlpad/docs/validation/evidence/cookie-slice-failure-dependency-first` 실제 exit 0, 2/2 PASS. [source/helper hash와 판정](../validation/evidence/cookie-slice-failure-dependency-first/result.json).

[exit7](../validation/evidence/cookie-slice-failure-dependency-first/exit7.json)은 실제 ExecMainCode=1/Status=7, [self-SIGTERM](../validation/evidence/cookie-slice-failure-dependency-first/selfsigterm.json)은 Code=2/Status=15를 보존했다. 두 case 모두 service의 관리 Result는 success, ActiveState=active/SubState=exited/MainPID=0이다. 따라서 앱 결과는 Result=success가 아니라 ExecMainCode/Status 원값과 guardian terminal ack로 판정해야 한다. Python 실행 자체의 종료값을 변경하지 않았다.

원 parent Invocation/native identity를 유지하면서 held cgroup.events/procs 및 fresh openat 모두 populated0/빈 procs를 확인했다. exact reader service Stop/Unref 뒤 parent는 자동 inactive/dead, Unref 뒤 GetUnit ENOENT 및 cgroup 경로 부재다. 정상 parent Stop 요청과 실패 cleanup은 0이며 두 case 모두 FD 4→4다. 스크립트 SHA256은 `fc27bf66914c46c865f77af414f99fdbee95bc2d19c3cea005a343e4d6893dd4`다.

이 결과는 실제 installed manager가 두 ExecStartEx flag를 받아 합성 failure/signal fixture의 parent 수명을 보장한 증거다. 제품 guardian 실패/취소/kill 및 앱 crash, authenticated C/I/B/receipt 통합 인수는 root 후속 범위다. 기존 대안 probe와 기존 실패 원시 증거는 변경하지 않았다.

## root 독립 실행 원시 재검토

stress_recovery가 root의 `cookie-slice-failure-dependency-root-first/result.json`, exit7/selfsigterm 원시와 `/tmp/dlpad-cookie-slice-failure-dependency-root-first.log`를 읽기 대조했다. 로그2case true와 원시 completed/passed=true, normal_parent_stop=false, failure_cleanup=[], FD4→4를 확인했다. source `fc27bf66914c46c865f77af414f99fdbee95bc2d19c3cea005a343e4d6893dd4` 및 result 재사용 helper5개 SHA 모두 현재 실제 파일과 일치한다. 재실행/소스 수정은 없다.

exit7은 ExecMainCode1/Status7, 자기 SIGTERM은2/15를 유지한다. 두 service는 관리 Result=success, active/exited/MainPID0이므로 ignore-failure는 원 tool 성공을 뜻하지 않는다. 실제 worker PID/start 소멸과 pidfd 종료, 동일 parent의 held/fresh-openat strict empty, reader/parent GetUnitENOENT 및 parent cgroup 부재를 각각 확인했다.

제품에서 원 ExecMainCode/Status를 종료·실패·signal 결과에 그대로 연결하고 guardian typed terminal/receipt proof와 구분해야 한다. 관리 success를 다운로드 성공으로 승격해서는 안 된다. durable native proof 뒤 reader service만 release하고 parent 직접 Stop을 제거하는 채택 후보이며, guardian RuntimeMax/취소/kill·앱 crash·queued foreign service 보존의 실제 제품 통합은 아직 별도 인수다. 이2case를 제품 구현 완료로 기록하지 않는다.
