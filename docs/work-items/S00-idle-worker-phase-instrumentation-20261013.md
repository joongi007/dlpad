# S00: 유휴 비용 실패 구간의 worker 작업 구분

- 상태: in-progress
- 분류: diagnosis
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: advanced_units — /tmp/dlpad-idle-worker-phase-candidate-20261013 후보만; root — frame_phase_fixture.rs/accessible_library.rs/accessible_metrics.rs/새 cfg fixture module 계약과 통합
- 선행 작업: default18 실제CPU2FAIL 보존·현재CI 및 source freeze 종료

## 사용자 결과

유휴 CPU 초과 구간에 파일 재조사·지표 갱신·화면 다시 그리기가 겹쳤는지 확인하여 원인에 맞게 수정한다.

## 계약과 범위

기존 default-OFF first-paint-fixture 계측만 확장하는 후보를 준비한다. default binary의 정책·poll/reconcile/sample 간격·CPU 예산·stop/실패 gate·schema·새 의존성을 바꾸지 않는다. 숫자 counter만 고정 상한 메모리에 보관하며 hotpath 파일로그/새 상시timer를 만들지 않는다. worker counter snapshot이 main frame에 publish될 경우 시점 지연을 기록하고 atomic native interval/인과 증명으로 확대하지 않는다. root가 공유 counter 이름·순서·수명·Cargo 변경 여부를 조정한다.

## 인수 조건

비활성 계측의 무동작·overflow/상한·worker start/end/failure 및 snapshot 구분을 실제 테스트한다. owner/private desktop/원 TID+creation·로그 상한을 유지한다. 실제 trace가 수집되기 전 원인 확정·default 예산 통과·P0 완료로 쓰지 않는다.

## 검증과 handoff

최신 기본18구간은CPU16PASS/2FAIL, memory18PASS이며 앱3자연종료/Job0이다. 모든 round thread set churn으로 특정 worker 작업 귀속은 불가하다. LoadedReady는 scan 부재 증명이 아니며 원 실패를 유지한다. 현재는 계측 후보 준비 단계다.

## 2026-10-13 통합 검증

root가 worker_phase_fixture.rs와 frame_phase_fixture.rs 및 두 main의 cfg 선언, Library/Metrics worker hook을 통합했다. default-OFF 기능으로 기존36 native property에16 숫자 property를 추가하며 고정128B registry/상한/CAS/epoch/weak snapshot·마지막 main publish 지연을 기록한다. 새 상시timer/파일로그/의존성 없음. 기본 whole 1278/0/30 PASS는 feature 자체의 실행 검증이 아니다. 별도 feature fmt/clippy/test/release와 7개 registry 회귀·실제 private trace를 진행 중이다. counters는 worker 작업 종류의 관측이며 TID CPU 인과 귀속이나 기본 예산 성공을 증명하지 않는다.

별도 cpu,first-paint-fixture 실제 fmt/clippy/test/release 및 root source/원본/immutable artifact 대조 PASS: **1312/0/30**,43 harness. worker registry7회귀가 두 harness에서 실제 성공했다. 진단CPU9902592B, 기본CPU9836032B, lock/dependency/version불변이다. 실제60초52property trace는 준비되었고 실행은 다른native/Cargo작업이끝난뒤 진행한다. [feature 원검토](../validation/evidence/windows-msvc-ui-feedback-library-worker-feature-20261013/root-review.json).

실제52property/private60초 진단 및 독립 raw/pin/수식/6구간/수명 검토 PASS다. 평균0.6765594609%, 고정6구간CPU0.309953~0.940532%와memory6PASS. LibraryPeriodicBegins는3구간에서1, 종료는측정끝까지0; CPU-debt wait는4/6구간43/56회, MetricsObserve각구간9~10회였다. 이것은관측된작업종류이며특정TID CPU원인이나기본18구간성공은아니다. 마지막publish뒤작업suffix·weak snapshot·quiet host미검증을유지한다. [실제 진단검토](../validation/evidence/current-worker-phase-cost-20261013/root-review.json). 이후 기본앱8ab7의18구간을별도로측정한다.
