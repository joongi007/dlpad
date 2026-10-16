# S01: Windows hard-link alias의 최신 메타데이터 복구

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root sharednative scanner/library·Cargo/인터페이스·재현/수정/전체검증·문서; metrics_recovery /tmp 최소fixture; diagnostic_cache /tmp primaryAPI자료; soak_recovery_audit /tmp 원실패DB/코드검토
- 선행 작업: WatchSession burst fixture의 실제실패

## 사용자 결과

다른 hard-link 경로를 통해 파일을 수정해도 목록의 크기/수정시간과 DB가 최신 원 파일 관측으로 조정된다.

## 계약과 범위

Windows actual burst에서완료파일12→13B뒤DB12가남아nativeexit101을보존했다. 경로별directorycachedmetadata/WatchSession타이밍/정확동일SQL판정3가설을분리한다. 먼저동일fixture재현후파일1개/Watcher없는최소공개Store시험으로줄인다. 새observations방식채택은재현/실측/128ID·reparse·rootnamespace·취소rollback 근거후root만조정한다. 시간초과·위조metadata로시험통과를만들지않는다.

## 인수 조건

original실패반복과실제최소red→green, alias inside/outside/retainedHANDLE/metadata조회없는경로·같은크기mtime변경·hardlink일반행보존·실패rollback·원byte/ID검증, latestfull4명령/10kGUI비용/연속eventfixture 재실행. Windows API 최소OS/fallback·크기/runtime비용과미검증범위기록.

## 검증과 handoff

원windows-watchsession-burst-20261005-continue는lock/clippy/build PASS후experimentexit101(DB12 expected13); Job는자연0/private/input복원PASS. coalescing완료인수/최신제품release는보류했다. diagnose loop를적용하며아직원인확정·수정완료가아니다.


### 原 실패 반복·최소 재현

`windows-watchsession-burst-repeat-red-20261005-continue`는 동일 원 바이너리에서 다시 DB12/expected13로 exit101을 냈다. `windows-alias-rescan-red-20261005-continue`는 offline lock/clippy/release를 통과한 후 watcher 없는 각1파일3freshfixture를 실제 실행했다. 사전 final metadata 조회 없음의 nohold·retained HANDLE 두 경우는 DB12이고 사전 metadata 조회 경우만 DB13이었다. 모든 원 native ID는 동일했고 시험 후 실제 두 alias는13B였다. 실패 exit1/원로그를 보존했다. 따라서 watch 타이밍 없이도 재현되며, 조회 유무가 결과를 바꾸는 관측 경로를 우선 교정한다. 원failed evidence를 성공으로 수정하지 않는다.


### 후보 API 실제 관측

`windows-by-name-api-20261005-continue`의 offline lock/clippy/release/실험4명령은 PASS다. System32 API-set 동적조회/Basic class3/104B align8을 사용했고, 사전 fs metadata 없이 두 hard-link 경로 모두12→13B·원volume/full128ID 일치를 실제 확인했다. 단일 pair wall134/93µs는 제품1만파일 성능 근거가 아니다. 최신 API는 hard import하지 않고, missing DLL/symbol·preflight의 정확한 unsupported class에는 원 no-reparse HANDLE StandardInfo/BasicInfo/full128ID 조회로 보정한다. permission/notfound/identity/reparse 오류는 보존/rollback한다. 현재 scanner 두 pass 최대2만queries 비용은 후속 실제 측정하며 지원 최소OS·reparse 의미·race/longpath 검증은 별도다.


### 첫 수정 회귀와 의미 보강

원제품 새회귀는 수정 전에 actual exit101(DB12 expected13)을 보존했다. 첫수정은 FILE_STANDARD_INFO.Directory bool compile오류를 보존하고 교정했다. 다음실행은 alias8조합 PASS 및 watcher9 PASS, 목록28개중27PASS/원multi-rename1FAIL이었다. 후자는 z를 첫관측 후 수정하더라도 z가 아직 관측 전이면 fresh7B가 정상값이라 old batch전제가 성립하지 않는 시험이었다. 모든 후보를 첫관측 뒤 변경하여 이미SQL INSERT된 후보 하나의 불일치를 확정하는 회귀로 보강했으며 InvalidData/원2행전체rollback/원각ID·bytes·재열기 assertion을 유지했다. 원failure를 덮지 않고 전체재실행 중이다.

선택 API의 실제 소유 junction 시험은 tag0xa0000003/attrs reparse/원junction128ID/target sentinel 보존으로 raw4native PASS다. collector는 inputs가 fmt전 fresh.rs를 가리켜 false였고 원결과를 유지했다. 모든native operation 내부/사이는 같은 source SHA였음을 root review가 대조했다. Symbolic-link·ancestor 경쟁·구형OS 실제 실행을 이 junction 인수로 확대하지 않는다.


### 전체 targeted green과 원 burst green

`windows-alias-freshness-all-green-20261005-continue`에서 watcher9·library30·두UI각14·fixture 포함Clippy가 실제 PASS했고 root현재541 source SHA/원Job/private/input을 대조했다. 독립담당도원로그/회귀의scope를 검토했다. `windows-watchsession-burst-green-20261005-continue`는 최초와동일source fixture를수정engine으로 rebuild하여4native PASS다. dirty 시작4회는약1초간격, 완료12B→alias13B/원ID/원1만bytes/sentinel/readonly10001행이검증됐다. scan CPU0.61~0.64초이므로성능완료는아니다.

preflight는능력/namespace만확인하고 SQL pass만freshmetadata를생성하도록 Step::PreflightFile/Observation을분리했다. 첫regular에서class확인1회외preflight는filequery를생략하고 원전체directory/depth/reparse/ID/cancel 검사를유지한다. API부재에는transaction의원HANDLE조회로정확성을보존하고lateunsupported는rollback한다. 실제중복축소회귀·비용측정진행중이며현재제품release/fullGUI는후속이다.


### 현재 default 전체·GUI 비용

중복축소후 `windows-alias-freshness-preflight-green-20261005-continue` targeted회귀/Clippy PASS, `windows-library-fresh-cold-cost-20261005-continue` 원Storescan3회CPU각0.328125초 PASS, 최신541 source의 `windows-alias-freshness-full-20261005-continue` fmt/clippy/test/release4명령 PASS다. root가4원Job 자연0/private/input/SHA 및defaultprofilemarker부재를대조했다. CPU0478aa51a152ed454ccdce1946ac952f07b9eeb977084ef7eaeec5adc873db66/7892992B는이전3f9보다4608B늘었고dependency/Cargo변경0이다.

새CPU격리GUI60초3회평균1.6145/1.5865/1.6131%와strict10초max4.206/4.057/4.225%는FAIL,80MiB메모리는PASS다. 정확성보정을되돌리거나예산/관측지연을늘려통과로표시하지않는다. 전체DB/원3만bytes독립검증과optionalrelative원directoryHANDLE query·실제스레드귀속을후속측정한다. 현재P0·최종S00선정·새패키지·클립보드/물리IME/reader/설치/공급/실제site/remoteCI는완료하지않았다.


### 상대경로 조회 독립실험 범위

기존Frame의원no-reparse directory HANDLE을RootDirectory로사용하는optional NtQueryInformationByName class77 후보를별도evidence crate에서실험한다. root는relative조회가absolute조회보다먼저실행되도록순서를보강하여alias수정뒤캐시refresh를가리지않고unexpected STATUS_PENDING에는임시시험process를buffer해제전에종료하도록했다. DDI의Nt API는user-mode ntdll export의장기지원보장과같지않으므로실제ABI·원전체ID/volume·EOF/mtime·reparse/cancel/unsupported·지연·비용근거전production채택0. 새로운Cargo dependency/권한/driver/admin변경은없다.


### 상대 조회 실험 결과와 미채택

원relative class77/ABI/alias12→13 시험4native PASS다. 1만파일3round정·역순12측정은원Process/ThreadCPU·full128ID/volume/size/time 전부대조하여PASS이고logical fixture vector1,560,000B(제품scanner정책아님)다. relative6표본CPU0.265625~0.28125초,absolute6표본0.265625~0.28125초로실질이득없다. 평균/모든표본/원15.625ms CPU회계해상도를root review에보존했다. 따라서더낮은지원안정성의Ntdll API를제품에채택하지않는다. 현재지원Win32 optional조회+원HANDLEfallback을유지하고실제worker 귀속·주기실행방법의별도experiment를진행한다.
