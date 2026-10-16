# S00: 사용자 서비스 기본 backend 비용 표본

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: stress_recovery — scripts/user_service_cost_probe.py, 본 work-item, 전용 validation/evidence
- 선행 작업: root의 기본 user-service CPU immutable release READY 경로/SHA 및 정상 기본 backend native 검증

## 사용자 결과

실제 Linux 네이티브 GUI의 idle/합성 다운로드 중 앱·launcher·service guardian 비용과 단독 실행파일 크기를 분리 기록한다. 전체 스택/3OS/8h 비용 합격으로 확대하지 않는다.

## 계약과 범위

AGENTS, ADR-0001, 사용자 서비스 guardian 계약, 기존 user_service_app_probe/cpu_background_probe/systemd_user_guardian_probe를 읽는다. 현재 launcher는 systemd-run이 아닌 동일 실행파일의 --dlpad-internal-user-service 경로이며 active 작업 동안 유지된다. guardian은 같은 immutable executable을 manager가 실행한다. 제품·Cargo·기존 harness 수정 없음. READY 이전에는 binary/GUI/service를 실행하지 않고 준비 기록만 작성한다.

독립 Python process에서 기존 GUI/helper를 readonly import해 실제 active-hidden 또는 stop-exit 합성 흐름에 관측을 덧붙인다. privateXvfb/private AT-SPI bus와 임시앱data/media만 사용한다. 기본 backend env override는 제거하고 native 실제 다운로드 시작을 사용한다. 테스트가 만든 고유 unit의 InvocationID/MainPID/start/cgroup과 worker PPID를 확인한 뒤만 guardian 표본 대상으로 등록한다. launcher는 소유 앱 thread children 중 동일 exe dev/inode/SHA 및 정확 marker로 식별한다. 기존8h PID/unit은 관측 집계·종료 대상으로 삼지 않는다.

## 인수 조건

- immutable binary SHA/bytes 전후 동일과 helper 없는 단독 artifact를 기록한다.
- startup warmup 뒤 app idle10초와 작업 중 active10초 이상의 perPID 표본을 기록한다. 선택적 hidden 표본은 실제 hidden 확인 뒤 별도 phase로 구분한다.
- /proc stat의 utime+stime(자식 CPU 제외), starttime, RSS/HWM을 최대10Hz·전체2000행 안에서 기록한다. launcher/guardian 부재는0MiB라는 측정값으로 만들지 않고 not-present/미포착을 구분한다.
- CPU는 monotonic elapsed/SC_CLK_TCK로 단일코어 %를 계산한다. app/launcher/guardian 각각과 동일시점 RSS 합계를 구분하며 공유 페이지 중복 합산 가능성을 명시한다. 합성 외부 worker 비용을 infrastructure 비용과 혼합하지 않는다.
- 실제 native 작업/DB attempt가 유지되고 종료 후 소유 PID/start·unit/cgroup 회수가 확인되어야 측정값을 유효 표본으로 보고한다.
- 짧은 표본은 peak·누수·시작1초·숨김40MiB·3OS·최종renderer 인수를 대체하지 않는다. 병렬 장기시험/host load 조건을 기록한다.

## 검증과 handoff

READY ea75955... 12,438,904B immutablecopy에서 current-1/current-2 실제 nativeGUI idle/active 비용을 실행했다. 두run exit0/native service proof와독립PID/unit/cgroup/DB 검증PASS. current-2 idle98개 자식관측에서 ownedmarker0, app RSS54.023MiB/CPU0.7017%; active infrastructure 동시RSS69.746MiB/CPU근사합2.2292%. 표본1 값도 원시로 유지한다. docs/validation/user-service-cost-validation.md 및 evidence/user-service-cost/ 참조. Python구문/문서검사 exit0. 전체OS/실제media/8h누수/hidden비용/전체스택결정은 남는다. 실행파일 정리는 root에게 맡기고 소유 GUI는 모두terminal이다.

최신 후속인수: a67d722 startup alias/EP 정리 immutablecopy12,439,160B에서 current-3동일collector 실행exit0. idle app54.219MiB/0.7027%, active infrastructure동시70.758MiB/CPU근사합2.0237%. 모든ownedrole/PID/unit/cgroup/DB 독립검증PASS, GUI모두terminal/sharedcopy유지. 원ea759표본두개보존. Python구문/문서검사exit0.

## 공용probe후속인자호환

root의latest97cfbcec current-4 첫측정은공용app_probe에추가된release_fault_env가costnamespace에없어실행전exit1/0samples로실패했다. raw보존. costnamespace에명시None을전달해일반실행계약을복원하고current-5로재시험한다. source추가변경은이compatibility필드뿐이며제품회귀/0RSS측정으로오인하지않는다.
