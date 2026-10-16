# S03: CPU 이력 실제 GUI 검증 준비

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R08, S03
- 담당/수정 범위: diagnostic_cache; 신규 scripts/cpu_history_probe.py 및 전용 validation/evidence
- 선행 작업: root HistoryPanel entry/bounded engine validated_resume 통합 및 새 READY

## 사용자 결과

새 journal 없는 화면,128페이지·Next,실제Paused작업의원snapshot같은id재개,terminal/손상옵션재개거절과scan대기종료를검증한다.

## 계약과 범위

READY전GUI금지/제품소스freeze. ownedXvfb/privatebus/합성ytfixture/임시data/media만. guardian cpu_options_probe.py/FIXTURE/lookup은read-only재사용하며수정안함. 실제Pause/Resume를먼저수행하고페이지/terminalfixture는이번소유SQLite에syntheticrows260개를seed(실제260다운로드로주장금지). localtoolargv만/외부네트워크없음. appSQLAPI직접구동으로GUI재개를대체하지않음. rowstate/optionstamper만ownedfixture. lock실험은HISTORY_LOADING namedactualText로pending를확인하고ClosePromptStopExit를actualaction으로보낸다.

## 인수 조건

sameid/attempt2/원Height720argv+typedoptions;잘못된UIcurrentoptionAudio로바꿔도snapshot보존. resume전validation의실제workerstarted없음. Next128→256names+PNG. DBhistoryMAX128계약과GUIrows가일치하는범위만관측. SQLitelock중HISTORY_LOADING존재/StopExit後생존/release後exit0,모든child/tempcleanup. 原failurepreserve/기존binary증거SHA분리.

## 검증과 handoff

harness준비만,실행은새READY필수SHA인자와root허가후. 제품단위시험만으로GUI완료주장안함.

## 실제 완료 결과

root 통합 CPU59tests/alltargetclippy/boundedAPI3tests/release PASS 보고. actualGUI SHA522e2304fc38618df696cfca1f126e08bc77634f48ac1db631d3c797fe8f8cbd에서 missing/corrupt/원snapshot재개·128페이지·종료대기 PASS. 최초physicalPause timeout raw보존, actualPauseaction 계측 대조로 같은후보 재검증. docs/validation/cpu-history-validation.md 및 cpu-history/summary.json. 모든소유GUI종료,source/Cargo추가변경없음. rawfailure상세journal미저장/다른OS·전체reader는미검증.
