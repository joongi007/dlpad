# S01: Windows native directory 목록 scanner 통합

- 상태: in-progress
- 분류: performance
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root 새 examples/windows_library_all_identities.rs·library.rs 통합·CONTEXT/ADR0002·전체검증/문서; diagnostic_cache 신규 library/windows_scan.rs /tmp 초안; metrics_recovery 전용 Windows 실패경로 테스트 /tmp 초안
- 선행 작업: bounded directory identity membership 실험과 Windows 기존 library 취소/rename/ID 회귀

## 사용자 결과

파일1만개 주기 재확인 비용을 줄이면서30초 fallback·외부 추가/삭제/rename·취소/rollback을 유지한다. native backend의 관측 시점과 namespace 계약을 문서화하고 실제 회귀 및 동일GUI 예산으로 채택 여부를 결정한다.

## 계약과 범위

Windows만 원 directory HANDLE·native128bitID·volume·finalpath/canonical·no reparse 디렉터리 검증으로 bounded64KiB batch를 stream한다. regular observations는 kernel이 반환한 name/size/FILETIME/opaqueID snapshot이며 개별fileHANDLE 전체검증과 같다고 주장하지 않는다. 개별 관측시간 이후 변경은 watch/30초 재확인으로 조정하는 기존 snapshot 계약을 유지한다. directory descent는 원batch childID와 실제 child directoryHANDLE ID를 대조하며 mismatch/API/namespace실패는 transaction rollback이다. root class unsupported는 첫관측전만 기존 conservative scanner로 fallback하며 traversal중 unsupported는 실패보존한다. reparse는 원보수경로 또는 명확한 bounded거절로 처리하고 namespace containment를 완화하지 않는다. MAX_SCAN_DEPTH·상한·after_first 취소ACK·schema/deps0·기존publication SQL을 유지한다. root가공유interface/게시를조정하고하위는/tmp만수정한다.

## 인수 조건

parser malformed/offset/UTF16·zeroID/native APIunsupported·원directory교체/ancestor·childrename·reparse·file교체·duplicate·cancelcheckpoint·callbacktimeout 실제 Windows 회귀, 원10000파일 전ID/size/mtime와 DB 대조, 기존hardlink/rename/취소회귀와 fmt/clippy/test/release, 원defaultCPU3회60초 strictCPU1%/memory80MiB·동일source/Job0/private/input 검증. 미지원filesystem/물리OS미검증을채택완료로표현하지않는다.

## 검증과 handoff

실험은 약2.4~2.9ms API+parse wall/약16ms wholeCPU였다. 생산적용전 안전성회귀와 원전ID 비교가 필요하다. 이 작업현재구현준비이며 GUI예산과 생산완료는 미검증이다.

호환성보강: 관측publication전native지원preflight를stream하고 knownclassUnsupported/reparse/zeroID만기존보수scanner를선택한다. 원rootguard를유지한restart로실제관측을시작하며 late namespace/permission/parse/API오류는transaction실패다. first-observation fixturecallback은실제SQL INSERT뒤한번만수행한다. preflight2회비용은실측으로확인하며clock/watch주기는변경0.

원공개Store3회CPU0.078125/0.125/0.109375초·전체1만nativeID/size/mtime실제대조PASS. rename후관측callback에서다른원file로교체하는경합을actualnative red1failed로재현했다. root가 unique rename 후보만 publication직전 원perfile metadata/canonical/nativeID를재검증해 mismatch는rollback하도록 보강하고, unchanged UPSERT는실제변경이없으면skip하도록했다. lateUnsupported callback 실패가preflight fallback으로오인되지않는회귀를추가했다. original SQL rename유일성/hardlink정책과schema/deps는그대로다.

보강후Windowstest20개PASS(원rename replacement red→green·lateUnsupported 포함) 및1만전체ID/size/mtime/bytes 대조 재PASS. publicAPI CPU0.078125/0.140625/0.171875초이며 기존1.17초보다작지만추가candidate재검증 비용도포함한다. unchanged SQL조건은실제측정상 CPU추가감소를입증하지않았으므로과장하지않는다. 기존복수newhardlinks와실패깊이스캔 snapshot시험2개를Windows에도실행하도록cfg만확장했다. 추가deps/Cargo/schema변경0.

unchanged UPSERT skip는추가개선미입증으로철회하고원publication SQL을복원했다. rename 후보검증은missing previous path가있을때만수행하도록boundedSQL EXISTS로gate한다(이조건은원rename SQL의필수조건이라후보누락없음). 마지막source공통검증과같은GUI60초가후속이다.

최종Windows feature회귀22개PASS. 첫전체검증fmtPASS/Clippy FAIL은새module의collapsible_if와malformedtest useless_vec 두lint였으며원raw보존했다. root가두코드표현만교정했고최신전체검증을새evidence로재실행한다. 아직test/release未実행이므로첫full을PASS로표시하지않는다.

최신 표현교정 뒤 540소스 Windows 전체 fmt/clippy/test/release가 실제 PASS했다. [원 결과](../validation/evidence/windows-native-library-workspace-lints-fixed-20261005-continue/result.json)와 root 원로그/source 검토를 보존했다. CPU630e63e38444…의 가상 데스크톱 60초×3회 GUI 검증을 실행 중이며 측정 종료까지 보호소스를 동결한다. 최초 Clippy 실패 이력은 그대로 보존한다.

최신GUI60초×3회 실제종료: 평균CPU0.728/1.118/0.910%, 모든memory PASS, strict6개10초구간은매회CPU FAIL. 전체30,000합성파일 원3B·각DB10,000행/무결성·원 Job자연0·입력desktop불변·동일source 검토를 진행한다. 기준을 완화하지 않고 [단계별 계측](S01-windows-native-library-phase-cost.md)을 이어간다.
