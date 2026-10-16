# S01: 주기 스캔 중 감시 조회 비용 제한

- 상태: in-progress
- 분류: performance
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root — crates/app/src/accessible_library.rs, CONTEXT.md, docs/adr/0002-runtime-and-storage.md의 관련 정책, 검증 문서. 하위는 /tmp 독립 검토만.
- 선행 작업: 동일 CPU/wall 예산의 독립 A/C 비교 종료와 결과·보존 검토. 실제 제품 반영 전까지 기존 정책을 유지한다.

## 사용자 결과

작업 중 반복하는 감시 root metadata 조회를 줄여 CPU 예산 안에서 주기 스캔이 완료될 수 있도록 한다. CPU 기준이나 완료 기한을 늘리지 않는다.

## 계약과 범위

주기 스캔 중 watch.poll 간격만 100ms에서 500ms로 변경하는 후보다. quiet receive 500ms, Stop·명령 확인과 CPU 샘플·스캔 대기 상한 100ms, wall 2ms/500ms·누적 CPU 4/1000·전체 180초·감시 30초 보완/1초 ordinary gate/2초 재연결 정책은 유지한다. 원 native watch 이벤트를 버리거나 root identity 확인을 생략하지 않는다. R01 별도 지표 cadence와 UI 상태 스키마는 변경하지 않는다. 추가 의존성·스레드·mutex·Cargo 변경은 없다.

## 인수 조건

독립 C 후보의 완료/CPU/9단계 profile과 보존을 검토한다. 반영 후 Windows 전체 fmt/Clippy/test/release·fixture feature 회귀, 실제 완료 180초·Next 5초·Stop ACK 5초와 이벤트·root 복구를 검증한다. 현재 소스의 GUI 60초 3회는 각 10초 CPU 1%·메모리 80MiB 기준으로 확인한다. 이벤트/root 복구 인식은 최대 500ms+checkpoint/OS로 느려질 수 있으며 hard 500ms 또는 1초 내 DB 반영을 보장한다고 표현하지 않는다. 단일 I/O 중단은 기존 cooperative 한계를 유지한다.

## 검증과 handoff

C1은 단독 engine scan에서 137.071초·CPU 0.546875초로 전체 10,000개를 완료했다. watch poll 264회, CPU 결정 대기 31.408초·wall 결정 대기 105.158초를 관측했다. 별도 warmup은 0.345초·CPU 0.359375초이다. 실제 GUI periodic/Next/Stop/이벤트 복구를 통과한 결과가 아니며 현재 A2 대조가 진행 중이다. 후보는 아직 제품에 반영하지 않았다.

## 비교 결과와 구현

동일 실험 바이너리에서 C1(500ms)은 137.071초·thread CPU 0.546875초, A2(100ms)는 151.399초·thread CPU 0.593750초로 모두 완료했다. 이전 A1/B1 실패도 보존한다. 단일 비교로 원인을 확정하지 않는다. root는 watch 조회만 SCAN_WATCH_POLL=500ms로 반영하고 실제 제품 회귀 검증을 시작한다. 전체 P0 완료는 아니다.

### 현재 소스 전체 검증

Windows GNU fmt/Clippy/workspace tests/release 모두 통과했다. source 543개 고정과 자연 Job 종료를 root가 원 로그로 확인했다. 기본 CPU 실행 파일은 SHA256 59448518d46ae8f3da4bf18540ae8ed031497584f13799e231966dd28f754848, 7,917,568 bytes다. [전체 검증](../validation/evidence/windows-scan-watch500-workspace-20261006-continue/root-review.json). 문서 checker 758개와 Python periodic 12개·resize 4개 회귀도 통과했다. fixture feature와 실제 GUI 인수는 진행 중이며 전체 P0 완료가 아니다.

fixture feature Clippy/workspace all-targets와 dlpad 앱 테스트도 통과했다. [회귀 기록](../validation/evidence/windows-scan-watch500-functional-tests-20261006-continue/result.json). 독립 담당이 원 test.log에서 CPU-first 3개·StopWake 3개·frame reuse 3개 실행을 대조했다. ignored와 GUI/MSVC 미검증은 유지한다. 실제 completion native 시험은 현재 진행 중이다.

실제 completion이 PASS했다. 첫 INSERT 34.075초, gate release 이후 commit 145.160초다. 전후 DB 10,000 rows exact·전체 payload/CAP/sentinel 보존, 원 앱 자연 exit0/Job0/private/input을 root가 원 결과로 대조했다. [완료 인수](../validation/evidence/windows-scan-watch500-completion-20261006-continue/root-completion-review.json). Next/Stop과 GUI CPU 등 남은 인수는 계속 진행한다.

실제 Next 응답 2.031초로 5초 기준을 통과했다. 첫 INSERT 35.341초, 자연 exit0/Job0와 전후 1만 DB·payload 보존 확인을 완료했다. [Next 인수](../validation/evidence/windows-scan-watch500-next-20261006-continue/root-next-review.json). Stop 시험을 이어서 실행한다.

실제 Stop 인수가 PASS했다. 첫 INSERT 35.911초, Stop 동안 목록 controls 비활성·release 이전 ACK false·release 뒤 ACK 5초 및 commit 금지, 자연 exit0/Job0, 전후 1만 rows/payload/CAP/sentinel 보존을 확인했다. [Stop 인수](../validation/evidence/windows-scan-watch500-stop-20261006-continue/root-stop-review.json).

Windows watch 정책 회귀 11개가 실제 실행돼 통과했다. missing root 재연결·metadata 단일 observation·backend failure/urgent priority 등을 포함한다. Linux 전용 native_write 및 replacement-directory 시험은 Windows 실행에 포함되지 않았다. [범위 제한 기록](../validation/evidence/windows-scan-watch500-engine-watch-tests-20261006-continue/result.json). 기본 제품 복원 SHA가 전체 검증과 일치하며 실제 watch-only GUI 시험을 진행 중이다.

현재 GUI CPU 60초×3 수집 및 root 18개 구간 재계산이 strict CPU<1%/whole Job memory<80MiB를 통과했다. 평균 0.5201/0.3902/0.4163%, 구간 최대 0.9379/0.7805/0.9371%이며 전체 fixture 보존 검토는 별도로 진행 중이다. [CPU 대조](../validation/evidence/windows-scan-watch500-thread-window-gui-60sec-20261006-continue/root-cost-review.json). startup missing root 자동 회복도 2.010초로 통과했다. paced scan 중 실제 higher-watch preemption은 기존 command 시험으로 대체하지 않는다.

GUI cost fixture 3회의 최종 DB 10k exact membership/integrity와 원 source/pins를 root가 대조했다. 독립 reader는 전체 30k payload를 검사했고 root는 DB 전체와 9개 payload spotcheck로 통합 확인했다. cost fixture에는 sentinel이 없고 CAP 최초 pin이 없어 이 시험에서 해당 before/after 보존은 주장하지 않는다. 전체 cost thread churn으로 완전한 작업 원인 귀속/quiet host도 미검증이다. 실제 completion/Next/Stop 별도 fixture에는 전후 CAP/sentinel 보존 증거가 있다.
