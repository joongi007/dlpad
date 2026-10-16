# S00: Windows GUI idle의 10초 thread 비용 구분

- 상태: in-progress
- 분류: verification
- 우선순위: P0
- 요구사항: R01, R08
- 담당/수정 범위: root scripts/windows_library_thread_window_cost60_probe.cs collector 통합·native/원결과검증·문서; metrics_recovery /tmp C# per-window thread counter 초안
- 선행 작업: current thread budget GUI3회 첫10초 원FAIL

## 사용자 결과

주기 요청 전 발생한 GUI CPU peak를 실제 원thread counter로 구분해 근거에 따른 수정으로 이어간다.

## 계약과 범위

고정 현재CPU와 source·원 PI/Job/private desktop 유지. 기존 ThreadCostPins의 원HANDLE/creation마다6개10초 구간의 누적CPU delta를 bounded JSON으로 기록한다. thread churn/OSdescription/observer timing의 불확실성은 보존하고 label을immutable authority로 표현하지 않는다. 제품 코드/Cargo/schema/사용자데스크톱 변경0.

## 인수 조건

각interval actual kernel+user/duration/creation/원Job delta 대조; 총 bound thread128/windows6·누락과 exited 명시. 자연 Job0/desktopclosed/inputunchanged/원1만synthetic 데이터 보존. 관측값만으로 startup/periodic/metrics phase를 확정하지 않는다.

## 검증과 handoff

current b2bd9d58 GUI avg .5463/.3381/.6768% 및 memoryPASS지만 round3첫10초1.2487% FAIL. 주기30초 요청 전 구간이어서 readerbudget만의 원인으로 단정하지 않는다. 아직 새계측구현/실행 전이다.

root가기존collector를보존하고별도새filename으로6창원thread초안을통합했다. JSON총byte상한은반복계측필드의증가로1MiB이며원8MiB hostlog상한·thread128/window6/name128UTF16을유지한다. 최대thread수에서도항상성공한다는보장은없고상한초과는실패로보존한다. 아직C#compile/native미검증이다.

### 2ms 후보의 실제 3회 측정

현재 기본CPU71ec7e489a1e0d7f0ae4301c179353a1ccdf4a0c37d63cd35e1c345121c16c94/source543에서 새collector compile과60초3회가 종료했다. 평균CPU0.598209/0.727867/0.364378%, round1 첫10초1.090614%로전체FAIL이며기준을완화하지않는다. 해당창 원main thread781250ticks, library312500ticks, restore0ticks다. Thread와Job의조회시간은비원자적이므로정확동일interval로합산하지않는다. 각3회자연Job0/privateclosed/inputunchanged 원결과를확인했으며전체payload와수식독립검토는진행중이다.

병렬담당: metrics_recovery는/tmp/dlpad-2ms-window-independent-review.json 및md로원18창독립감사; soak_recovery_audit는/tmp/dlpad-main-idle-redraw-audit.md로main UI재그리기경로read-only 조사; diagnostic_cache는/tmp/dlpad-restore-stop-event-design.md로등록/취소경쟁설계검토. 모두프로젝트/native실행/대량fixture읽기금지, root만제품수정/통합/실제native검증을담당한다. restore설계검토는성능원인확정이나채택을뜻하지않는다.

root가원18창수식/원source와compiledpins/Jobguards를직접재검증하고독립감사를보관했다. [root metadata](../validation/evidence/windows-periodic-thread-window-gui-60sec-20261006-continue/root-metadata-review.json). 3만synthetic payload와3개DB각1만행/정확membership/size/present/identity형식/integrity/CAP보존도실제완료했다. [root payload](../validation/evidence/windows-periodic-thread-window-gui-60sec-20261006-continue/root-payload-review.json). allnativeID재열기/quiet-host보장은없다. 원performanceFAIL을유지하고S00-bounded-identical-frame-raster-reuse 후속의新GUI를원strict기준으로직렬측정한다. 현재실험소스변경으로以上source543/71ecGUI는당시snapshot증거다.
