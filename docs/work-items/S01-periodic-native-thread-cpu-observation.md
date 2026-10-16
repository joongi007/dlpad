# S01: 실제 주기 스캔의 원 Windows 스레드 CPU 관측

- 상태: in-progress
- 분류: diagnosis
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root — scripts/windows_library_periodic_thread_cpu_probe.cs, scripts/windows_library_periodic_functional_native_probe.py 및 검증 문서. metrics_recovery — /tmp C# helper 초안만.
- 선행 작업: CPU-first 정책의 실제 완료 180초 실패와 원 DB 보존 기록.

## 사용자 결과

실제 앱이 주기 스캔 중 완료 기한을 넘는 원인을 CPU 사용량과 대기 구간으로 좁힌다. 관측 도구이며 제품 예산을 변경하지 않는다.

## 계약과 범위

기존 표준 collector를 보존하고 명시적인 진단용 collector 선택만 추가한다. 원 app HANDLE·PID·생성 시각과 원 library thread HANDLE·owner·생성 시각을 고정한다. thread 이름은 역할 선택 근거이며 작업 권한으로 사용하지 않는다. 첫 INSERT 이전/도달/10초 구간/종료의 kernel+user 100ns 카운터와 실제 경과 시간을 최대 32회·128KiB로 저장한다. 추가 제품 스레드나 CPU 예산 변경은 없다. 기존 75초 선행 관측·180초 완료·5초 Next/Stop·4event capability·private desktop 및 synthetic fixture 계약을 유지한다.

## 인수 조건

원 스레드 HANDLE로만 샘플링하고 counter 역행·identity·권한·종료 오류를 별도로 기록한다. 성공·기능 실패 모두 종료 전 원 샘플을 보존한다. 계측 실패는 기능 성공이나 CPU 기준 통과로 확대하지 않는다. source/compiled pins와 원 Job·private/input cleanup을 root가 대조한다. 계측 추가 비용을 기존 무계측 GUI 측정과 동일하게 표현하지 않는다.

## 검증과 handoff

초안은 읽기 검토만 했으며 실제 Windows 컴파일·계측 전이다. CPU 0.4%의 180초 누적 허용량은 0.72초이므로 실제 스캔 CPU를 먼저 확인한다. wall 작업 구간 변경이나 CPU idle credit 제거는 이번 작업에서 채택하지 않는다.

root가 기존 기능 collector를 보존하고 별도 C# 진단 collector와 고정 `--thread-cpu-observer` 선택을 통합했다. 원 입력 C# source의 프로젝트 원본 SHA도 종료 후 대조한다. 원 snapshot thread 열거는 32,768개, 소유 thread는 128개, 샘플은 32개로 제한한다. UIA timeout에는 원 Job 종료·bounded join 이후에만 원 event/HANDLE/desktop을 닫고, 종료하지 않는 worker가 있으면 FAIL과 HANDLE 보존 상태를 기록한다. 새 선택은 기본 OFF이며 제품 CPU 예산과 기존 기능 인수 기한을 변경하지 않는다.

표준 CLI 순수 테스트는 새 기본 OFF 진단 선택을 포함해 12개 통과했다. 최초 진단 C# 컴파일은 stop field를 다른 class명에 삽입하려 했던 root 통합 오류로 CS0103 두 건이 발생했다. 실제 class에 명시적으로 field를 선언해 교정했으며 원 실패를 보존한다. 제품 스캔 결과나 CPU 계측 결과로 집계하지 않는다. 새 fresh 경로에서 실제 진단을 이어간다.

수정 후 진단 C# 컴파일은 exit 0, 원 thread 계측은 measurement_failed=false와 동일 HANDLE owner/creation·25개 단조 샘플을 확인했다. 실제 기능은 첫 INSERT 42.179초 후 commit 180초 실패로 유지된다. 첫 gate부터 마지막 관측까지 약 180.034초 동안 library thread CPU 0.765625초를 관측했다(kernel 0.703125초·user 0.0625초). 이 값은 원 단일 스캔 시작과 정확히 맞춘 baseline이 아니며 rollback/watch/retry 등 전체 library thread 활동을 포함할 수 있다. 실제 스캔 자체의 CPU가 0.72초를 넘는다고 확정하지 않는다. [root 계측 대조](../validation/evidence/windows-composed-budget-thread-cpu-fixed-completion-20261006-continue/root-thread-cpu-review.json). 추가 private phase 비교로 실제 scan과 대기 비용을 분리하며 제품 예산을 완화하지 않는다.
