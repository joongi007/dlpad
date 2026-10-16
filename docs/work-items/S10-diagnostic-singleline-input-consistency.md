# S10: 진단 단일 행 입력의 공통 상호작용 규격

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R08, R10, R11
- 담당/수정 범위: 미배정; crates/app/src/accessible_diagnostics.rs, actual diagnostics private observer와 관련 문서/회귀. 공유 schema/Cargo/lock은 root 조정이며 변경 필요 없음.
- 선행 작업: S00-network-observation-input의공통hint/ID helper및전체검증

## 사용자 결과

진단 log index·MPD attachment path·export path 입력의 높이/여백/caret/focus/논리 ID를 다른 단일 행 입력과 일치시킨다. export ID 기반 AccessKit ScrollIntoView와원본보존/create-only 계약을유지한다.

## 계약과 범위

현재 직접 TextEdit 3곳의 공통geometry누락을전입력읽기감사에서확인했다. multiline log editor·readonly paged preview·DragValue는별도규격이며단일행helper로억지변환하지않는다. 이전실제S10성공증거를새geometry의전수클릭/입력인수로소급하지않는다. 임시syntheticMPD/log/privateWindows만사용한다.

## 인수 조건

실제진단Form의각필드를클릭/편집/Tab/hover/selection하며Light/Dark/좁은창/앱배율에서캡처한다. 잘못된index·수정로그취소/적용·첨부missing/oversize/unsafe·exportexisting/실패preserve 및실제새파일내용/privacy와workerACK를재검증한다. shared helper로바꾸되ancestorScroll/AccessKit ID·MAX_INPUT_BYTES/4 bounds를유지한다. whole4검사와문서·새사용자후보를검증한다.

## 검증과 handoff

미구현이다. docs/development/input-coverage-audit.md의읽기전용감사가누락근거이며,네트워크교정을진단입력완료로표현하지않는다.

2026-10-11 사용자 전체 진행 지시에 따라 구현 착수. 담당/공유 계약과 통합은 P0-quality-completion-integration.md에서 조정한다. Git 불가로 하위 초안은/tmp에 분리하며root가현재 파일에 통합한다.

2026-10-11 root 통합: 공통 helper/고정 ID·32px·12px padding 적용. actual panel 회귀3개를 Windows CPU에서 실행해 PASS했다(`windows-msvc-diagnostic-input-corrected-20261011`). 첫 whole/debug의2 FAIL은 AccessKit node 순서를 시각 순서로 가정한 테스트 오류였으며 원 실패 로그를 보존했다. 실제 bounds 정렬로 필드를 식별하고 원 glyph 정렬 허용 오차는 유지했다. 물리 IME/reader·current native GUI 교정은 아직 별도 인수다.

2026-10-11 추가 native collector 재시험은 진단에 도달하기 전 legacy fixture 준비에서 실패했다. private provider Focus로 cookie input이 y933→507로 실제 scroll 공개됨을 관측하여 기존 wheel-route 실패와 구분했다. 현재 preview bypass Toggle readback은 실패해 actual 진단3input 완료를 주장하지 않는다. 원 실패 evidence를 보존하며 새 UI stable identity와 collector를 교정한다.

2026-10-11: fresh-field-focus 시험은 저장 경로·로그 번호·MPD의 세 singleline readback과 로그 편집 입력을 확인했으나, 정제 후 반영 전에 기존90초 UIA 총 기한을 넘겨 전체FAIL이다. 추가 필드 PNG·settle 비용을 반영해 worker180초/host240초로 총 기한만 조정하고 단일 단계 상한은 유지했다. 요약 명칭은 native_readback으로 바꿔 provider focus를 pointer·물리 검증으로 표현하지 않는다.

root actual PNG 감사에서중첩스크롤제품결함확인: full-budget수집기의nativefocus/readback은통과했으나diagnostic-controlsviewport가download-content하단에남아focusinput32px중수pixel만paint됐다. rootPNG2장을직접확인했고focusJSON은clientcenter531/height32였다. nativeproviderfocus를입력가시성인수로대체하지않는다. 실제Candidate중첩스크롤회귀/ancestorreveal교정을후속통합범위로추가하며현설치ZIP은이결함수정전후보이다.


### 2026-10-11 실제 부모 스크롤 회귀와 수정

root는 private GUI PNG에서 32px 입력의 실제 표시가 하단 몇 픽셀로 잘리는 것을 확인했다. UIA Focus/값 readback 성공은 표시 인수로 사용하지 않는다. `accessible.rs`의 중복 diagnostic-controls ScrollArea를 제거하여 기존 download-content가 전체 controls 스크롤을 소유하고, `accessible_diagnostics.rs`는 widget이 소비하기 전에 exact Focus/ScrollIntoView를 포착해 실제 field rect를 부모에 전달한다. readonly preview의 자체 제한·보고서/입력 상한은 유지한다. root 담당 파일에 실제 Candidate960×640 회귀를 추가하며 schema/Cargo/lock 변경은 없다.

`windows-msvc-diagnostic-viewport-red-resolved-20261011` 실제 Windows 회귀는 로그 번호 Focus의32px rect가514..546으로 visiblecontent544를 넘는 실패를 재현했다. 앞선 compiler/selector 실패는 재현 성공으로 계산하지 않는다. 초기 GREEN 시도는 text의 의도된 내부clip에 전체field32px를 요구한 테스트 오류로 실패했으며 수정해 전체배경clip과 실제glyphclip을 따로 검사한다. 최종 새whole/GUI/package 결과로 인수할 때까지 in-progress다.


새 실제 Candidate 회귀 `windows-msvc-diagnostic-viewport-green-background-glyph-20261011`은1passed/0failed다. 세필드×Focus/ScrollIntoView의 fullfield backgroundclip과 입력glyphclip을 같은 프레임으로 확인했다. 기존실제 DiagnosticPanel 회귀 `windows-msvc-diagnostic-viewport-panel-regressions-20261011`도14passed/0failed다. `patch`의 절대경로거절 및 상대경로EROFS 시도는 실제 적용으로 계산하지 않았으며 root는 허용된 lowercase absolute project파일에 동일hunk를 적용했다. 이후rustfmt를실행했다.


`windows-msvc-s10-unclipped-final-ui-20261011` 새CPU의 실제privateedit시나리오PASS. root는3field Focus PNG에서전체32px를직접검토하고 fresh UIA y299/310·높이32·focus를대조했다. actualJSONinitial은Metadata/exit7·인증출력생략, edited는public observation URLtoken정제임을확인했다. 실제readonlyDB Failed1attempt/artifacts0·snapshot쿠키옵션없음, mediaempty·원cookies/sentinelSHA보존·원Job자연0/inputdesktop불변을인수했다. 전체S10시나리오/physical pointer·IME·reader를검증완료로주장하지않는다. readonly multiline73px는singleline32px인수와분리한다.
