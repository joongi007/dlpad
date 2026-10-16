# S00: 최신 Windows 유휴 CPU 예산 실패 진단

- 상태: in-progress
- 분류: performance
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: windows_auto — /tmp source·원시로그 진단; root — 계측/실행/공유 source 통합
- 선행 작업: 최신 CPU776d17c…3×60초 actual측정

## 사용자 결과

유휴상태의 CPU 예산 초과를 원인에 맞게 수정한다. 첫구간 폐기·평균대체·threshold완화로 성공표시하지 않는다.

## 계약과 범위

원 current-ui-focus-plan-idle-cost-20261013은18창중2CPU FAIL(1.090280335%,1.256076155%),memory18PASS다. 원 source/raw/lifecycle/currentCPU 및3×10000 합성자료를보존한다. 병렬담당은프로젝트읽기/tmp만수행하여화면담당의accessible_metrics·공통UI파일과충돌하지않는다. 기존first-paint-fixture 계측이UI/tess/paint/present·wake/metricscheck를구분할수있는지검토하며필요하면root가cfgfeature 국소후보를통합한다. 제품schema/DB/OS설정/새dependency를변경하지않는다.

## 인수 조건

3–5반증가설을기록하고빠른실제재현계측으로 UI/renderer/library/metrics/초기활동을구분한다. 원18창과원인수정뒤fresh조건동일18창을연결한다. threadchurn 귀속실패를어느worker의범인증명으로표시하지않는다. 평균0.729/0.625/0.494%만으로통과를주장하지않는다.

## 검증과 handoff

원 strictcollection은exit2, rootwrapperexit1 FAIL이며ownerJob자연0/privateDesktopClosed/InputDesktopUnchanged를확인했다. source-only진단과existing단일60초/6창/170초boundedargv는/tmp/dlpad-idle-cost-diagnosis-20261013에준비했다. 추가계측/featurebuild/native는미실행이며userUI전수수정뒤sourcefreeze가선행한다. CI후보와원FAIL을보존한다.

최신 준비/실행: whole-v7 source654/CPUc69a1e…를 기준으로 cpu,first-paint-fixture4검사와immutable018e20…/9890304B를 생성·독립 대조했다. 첫observer는legacy Startenabled 기대값으로sample전에실패, v2는legacy InvokePattern만요구해sample전에실패했다. 원FAIL을보존하고제품gate를완화하지않았다. v3 observer는실제Startdisabled/URLempty·enabled/복구확인·시작조건/취소없음과nativeToggle/selectedRuntimeID를확인한후같은10kLoadedReady/six10s측정에진입했다. 현재측정진행중이며default18창인수또는원CPU2FAIL해결로승격하지않는다.

진단v3실제결과: 60초/6고정구간 CPU0.315~0.623%·메모리최대78,680,064B로6/6예산PASS, 자연close0/Job0/source/rawpin독립검토PASS. feature진단이며직전default18창2FAIL을대체하지않는다. MetricsChecks/Unchanged합계2·MetricsRedraws0; 모든metrics관측0이라는표현은사용하지않는다. RootDue/CallbackDue와cachehit관측에서callbackgeneration폐기후보를준비했으나인과는미확정이다.
