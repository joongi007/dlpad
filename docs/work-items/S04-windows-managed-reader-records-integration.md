# S04: Windows 관리 reader 기록 공통 계약 통합

- 상태: in-progress
- 분류: safety
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: root — tools.rs module/API 가시성, 순수검증과 향후native통합; diagnostic_cache — windows_managed_records.rs
- 선행 작업: Windows originalmarker/nativeID/privateACL 기반

## 사용자 결과와 계약

실행stage의single-useIntent/Bound/Resolved와원receipt일치를검증하는bounded형식을통합한다. root가process/native-storage 재사용을위해crate내가시성을조정한다. receipt encoding/Resolved 상태만으로삭제권한은없다. native originalID/ACL/Jobempty/terminalwriter증거를별도검증해야한다. 최대8reader/256byte receipt8개/16KiBjournal,고정12anchor,새dependency없음.

## 인수 조건

잘못된nonce/stage/identitydigest/anchoralias/transition/indexreuse/oversize/unknownfields/receiptprefix/reserved/길이를거절하며Resolved재열기후실제receipt없으면matchingfalse다. 순수시험PASS를native자동회수완료로주장하지않는다. Windowsprocess/native저장통합이후actualcrash/미확인reader보존/receipt후회수인수는필수다.

## 검증과 handoff

root초안코드읽기검토후module/API연결했다. 실제순수회귀및strictclippy를실행하고native저장capability를후속연결한다.

### 실제 순수 계약 통합 결과

root실행tools::windows_managed_records4시험PASS,enginealltargetsstrictClippyexit0. module정렬fmtcheck첫실패는cargofmt로수정했다. 원8reader single-use/stage/nonce/anchor/digest/receipt손상/actualreceipt없는Resolved거절을확인한다. 이것은순수계약인수이며nativejournal/Job/guardian기록쓰기/자동회수는아직구현중이다.
