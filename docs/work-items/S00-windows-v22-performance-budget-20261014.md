# S00: 최신 Windows 유휴 성능 예산 검증과 개선

- 상태: in-progress
- 분류: performance
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당/수정 범위: root — 실행·통합·최종 검증 및 문서; advanced_units — 분리된 감사/진단 후보; 제품 소스는 현재 동결
- 선행 작업: v22 source669 전체 Windows 검사, native 화면 시험과 분리된 측정

## 사용자 결과

합성 파일 10,000개를 가진 앱이 유휴 상태에서 과도하게 CPU와 메모리를 사용하지 않도록 한다. 평균과 순간 구간의 실패를 구분하여 개선한다.

## 계약과 범위

사용자 Documents 대신 새 소유 fixture 세 개를 사용한다. 세 앱은 차례로 별도 private desktop/Job에서 실행한다. 60초 측정마다 겹치지 않는 10초 구간 여섯 개를 검사한다. 제품의 제안 목표는 단일 코어 기준 평균 CPU 1% 미만이다. 추가 진단은 각10초 구간에 더 엄격한1% 미만 기준을 적용한다. working set과 private usage의 Job 합계는 각각80MiB 이하가 기준이다. 원 기준을 완화하거나 실패가 사라질 때까지 같은 측정을 반복하지 않는다. 기존 GNU 실행 파일과 잠금파일을 보존한다. 실제 thread churn으로 완전한 측정이 불가능한 thread 표는 인과 근거로 승격하지 않는다.

## 인수 조건

현재 binary/소스/원로그 핀을 확인하고 18구간의 counter 차이·시간·메모리를 독립 재계산한다. 세 앱의 자연 종료·Job0·private desktop 종료와 입력 desktop 불변을 확인한다. 30,000개 원 합성 파일과 SQLite 세 개를 읽기 전용으로 검토한다. 실패를 수정할 경우 먼저 좁은 진단으로 원인을 확인하고 새 전체 검사와 새 측정으로 인수한다.

## 검증과 handoff

실제 명령: `.local-tools/session-recovery-20261013/final-current-ci-cost-readiness-v1/cost-collector.py`에 현재 whole proof, fresh evidence와 `--no-concurrent-dlpad-probes`를 전달했다. 최종 exit1/strict budget FAIL이다. 세 평균 CPU는 0.598640/0.884629/0.650723%였지만 18구간 중 두 구간이 1.093468/1.092684%로 실패했다. 메모리 18구간은 통과했다. Root는 원18 counter 차이·시간과 계산을 별도로 재검산했다. 두 실패 구간의 thread 표는 churn으로 불완전하다.

전체 raw/pins·SQLite·합성 파일과 여섯 실제 PNG는 별도 독립 감사 중이다. 원 실패는 [측정 결과](../validation/evidence/current-ui-feedback-v22-default18-cost-20261014/collector-result.json)에 보존한다. 이것은 전체 성능 인수나 물리 GUI 검수를 완료했다는 뜻이 아니다.

Root 독립 감사 완료: source669/현재3실행파일/원pins/GNU, 실제compile0·측정기2·자연Job0·세앱0를 대조했다.18구간 raw수식과 세가중평균을 재계산했고 세SQLite integrity/migration1·2/각10,000행 및 원30,000파일3byte/열린FD신원·namespace를 다시 확인했다.6PNG는 팀 직접검토와Root6해시대조·중요2원화면을 확인했다. 보고된60초 평균의 원최종counter가 저장되지 않아 exactraw재계산 완료로표현하지 않는다. [Root 결과](../validation/evidence/current-ui-feedback-v22-default18-cost-20261014/root-review.json). 좁은metrics API operation profiling을 준비하며 같은기본측정의 무의미한반복은 수행하지 않는다.

좁은API실측/Root검토완료: 같은원engine의volume/automatic-route/sample을30회1초주기로호출했다. 각각30회정상관측,combined총wall15.463ms/회평균0.515433ms였다. callerCPU원counter delta0은유한분해능관측일뿐실제비용0을뜻하지않는다. 이결과로live UI/메트릭worker 전체원인을단정하거나관측갱신주기를늘리지않는다. 원18구간2FAIL을유지하며main repaint/library watch/discovery/IPC의경계별진단이후속이다. [API 원검토](../validation/evidence/v22-metrics-operation-cost-v4-20261014/root-review.json). Root가compiledbinarySHA를추가로대조하고자신이새로추가한예제1파일만해시확인후제거해원provider2014를복원했다. 보존된B원예제와실제compiledbinary/raw는남겼다.
