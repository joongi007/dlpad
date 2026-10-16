# S00: 선택 요약의 실제 글자 정렬과 파일명 우선 너비

- 상태: done / v24 scoped 통합 검증
- 분류: ui-polish
- 우선순위: P0
- 요구사항: R03, R08, R11
- 담당/수정 범위: installer_batch — accessible_library.rs 분리 후보; Root — 통합/전체 검증/새설치본/문서
- 선행 작업: v23 source670/전체1536/0/42 및 실제960·100/150/200 selected PNG Root 직접 검토

## 사용자 결과

선택 파일명과 metadata를 각 열의 왼쪽에 정렬해 목록·도움말의 기준선과 맞춘다. 작은 창에서 파일명 열을 우선하고 secondary metadata는 truncate하되 전체 내용은 기존 hover/accessibility description으로 확인한다.

## 계약과 범위

실제 w960-z100/library-selected.png에서 add_sized Label의 가운데 정렬을 확인했다. selection_summary의 두 Label은 halign Min과 명시적인 mainAlign Min/crossCenter·main/cross justify 셀을 사용하고 metadata width cap을70%→45%로 줄인다. add_sized의 intrinsic 가운데 배치를 실제 수정하면서32px height/ID/전체경로·정보description/기존선택/virtualrows/worker/API/gates/Cargo/schema를 유지한다. 현재v23 Source/ZIP/proof를 보존하고 Root가 새whole v24로 구분한다.

## 인수 조건

짧은 한글 이름과 실제 TextShape/Labelbounds로 두열 글자의 left alignment를 확인한다(800/960·앱100/150/200·Light/Dark). 기존49 Library 회귀 유지, 새 Windows 전체4검사/actual currentprivate selected images/원sourcefile·naturalJob guard 및 새패키지인수. 물리input/IME/reader/fullUX인수와구분한다.

## 검증과 handoff

v23의 실제 selection32 geometry와 Context tests만으로 glyph alignment 인수를 대신하지 않는다. Root 원픽셀 검토에서 추가 개선을 확인했고 v5/v6 분리 후보를 실제 검증했다. metadata 전체값/hover/description은 유지한다. 새 native24 픽셀·패키지·설치 인수가 끝나기 전에는 최신 UI 완료로 보고하지 않는다.

Root v5 Windows 원결과49PASS1FAIL: 새실제paint시험에서800·100%·Light TextShape.x226.7 vsLabelcell.left8을관측했다. 단순halignMin이add_sized의셀중앙배치를해결하지않아 **제품 정렬 미해결**로분류하고 원FAIL을보존했다. primary egui ui.rs1798/layout.rs171을Root가직접확인했다. v6는두cell을기존동일allocation API의explicitmainAlignMin/crossCenter/두justifytrue로바꾸고metadata45·같은회귀인수를유지한다. frozenpins·v5formattedSbaseline대조후 S만overlay, Live23변경없음. 그 뒤 동일50시험을 실제 수행한 결과는 아래 v6 기록과 구분한다.

Root v6 실제 Windows Library module **50/0/0** 통과, 원로그·SourceS/Live분리·후보맵/Job0를 [독립 검토](../validation/evidence/managed-tool-isolated-summary-alignment-green-v6-20261014/root-review.json)했다. Root가원source23backup과before/after SHA를 root-v24-source-adoption-v1에기록하고Live한파일만채택(source670 유지)했다. 새 source의 Windows whole24 네 명령(fmt/clippy/test/release)은 [Root 독립 검토](../validation/evidence/windows-msvc-ui-feedback-v24-whole-20261014/root-review.json)를 통과했다: source670, 43 harness, 1,538 PASS/0 FAIL/42 ignored. 현재 CPU는10,243,072 B이며 현재 installer/CLI와 함께 원 artifact SHA·raw log·source beforeafter를 검토했다. 원 source23/ZIP/17ticks 진단은 각 frozen 범위로 보존하며 새 source에 원23 whole/GUI를 재사용하지 않는다. 현재 native24 6scope 및 직접 픽셀·새 패키지/설치 검증은 pending이므로 상태는 in-progress다. 물리 입력/IME/reader와 전체 UI/정식 배포 완료를 뜻하지 않는다.

### 통합 완료 범위

새actualsource24 전체1538/0/42, 선택24원화면,설치88원화면을Root/child가직접분담검수하고Root가전체SHA/bytes/dimensions를재검산했다. 새두ZIP/normal/long/negative검증과문서체크1094문서/114원notice는통과했다. 물리사용자검수/cleanOS/정식배포/전체P0완료는아니다. 최신로컬CI와비용은별도 [CI작업](S00-windows-v24-local-ci-20261014.md)/[비용작업](S00-windows-v24-default-cost-20261014.md)에서진행한다.
