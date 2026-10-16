# S00: 현재 진단 로그 multiline 편집 인수

- 상태: done
- 분류: verification
- 우선순위: P0
- 요구사항: R07, R08, R13
- 담당/수정 범위: root crates/app/src/accessible_diagnostics.rs·scripts 실행·통합·입력표; diagnostic_inputs와 timeline_precision은 /tmp observer/collector·편집 UI 후보만 작성한다. 공유 Rust/Cargo/schema/lock 변경은 root가 조정한다.
- 선행 작업: 현재 whole1013/0/26·S10 정제 보고서·singleline 인수

## 사용자 결과

실제 진단 편집 로그에서 줄 이동·Home/End/Delete·선택 편집 후 내용과 caret가 읽을 수 있는 위치에 유지되고 정제 반영이 정확한지 확인한다.

## 계약과 범위

DESIGN·진단 요구사항·ADR의 bounded/private 정제 계약을 유지한다. 별도 가상 화면·합성 Attempt/로그/도구만 사용하고 실제 계정·미디어·clipboard·global focus를 사용하지 않는다. readonly preview와 editable multiline 규격을 구분한다. 물리IME/reader/포인터를 synthetic action으로 완료 처리하지 않는다.

## 인수 조건

현재 binary/source/helper SHA와 원 소유process/Job/private desktop 종료를 확인한다. 편집 전후 정확한 합성 내용·selection/caret 관측·bounded viewport 및 실제 glyph PNG를 검토한다. 정제 반영 후 민감한 합성 marker 보존/삭제 정책을 확인한다. helper 작성만으로 완료 처리하지 않는다.

## 검증과 handoff

현재 source의 editable multiline은 진단 edited_log이며 존재하지 않는 추가인자 입력을 시험 대상으로 만들지 않는다.

두 번째 실제 실행은 native compile와 GUI 명령이 자연 종료0/Job0로 끝났고, 여러 줄·Home/End/Delete·선택 교체·잘못된 로그 번호의 보고서 보존 assertion을 통과했다. Python collector는 원 observer가 기록하지 않는 `original_handles_close_safe` 키를 요구해 최종 실패했다. [원 raw](../validation/evidence/windows-msvc-diagnostic-multiline-current-second-20261011/native/result.json)와 실패 result를 보존하며 전체 인수 PASS로 바꾸지 않는다. Root가 vertical-last/up-home-delete PNG를 직접 확인했다. 편집 영역은 기본288px 폭이라 나머지 입력과 정렬이 다르고 줄 수에 따라276px 높이로 늘어난다. collapsed caret의 provider rectangle은 empty/Infinity이므로 caret 표시 검증은 미완료다. 안정 ID·전체 폭·높이 제한 및 긴 로그의 커서/스크롤 검증을 후속 구현한다. 현재 기본 binary/source는 변경되지 않았다.

Root는 원 명령 로그·8개 정확한 편집 readback·invalid index 전후 JSON 동일성·원 SHA/source와 두 SQLite integrity_check=ok를 독립 대조했다. [범위 관측](../validation/evidence/windows-msvc-diagnostic-multiline-current-second-20261011/root-review.json)은 SCOPED_OBSERVATIONS_WITH_GAPS이며 collector의 FAIL을 유지한다. `python3 -m unittest discover -s scripts -p 'test_ci_*.py'`25tests PASS, `python3 scripts/check_docs.py`889documents/114보존공지 PASS는 별도 검사다.

현재 구현: edited log에 안정 ID·전체 가용 폭·120~200px scroll viewport·12px 가로 여백을 적용했다. 실제 TextEdit의 cursor reveal을 내부 scroll에서 처리하고 포커스/선택 이동·편집/ScrollIntoView의 parent에는 bounded viewport를 전달한다. AccessKit의 전체 문서 bounds를 가짜 viewport bounds로 바꾸지 않는다. readonly 보고서도 가용 폭에 맞추고 기존320px 높이·page/export/privacy 계약을 유지했다. 추가 의존성·schema·lock 변경 없음.

실제 Windows RawInput 회귀: 첫 실행은 테스트의 NodeId dereference 컴파일 오류이며 인수 RED가 아니다. 수정 후 기존 제품에서 ancestor reveal assertion이 실패(1failed/exit101)했고, UI 교정 후1passed/exit0. 100줄+긴 문단의 실제paintedgalley와 clip에서 CtrlEnd+입력/CtrlHome+Delete cursor·wrapping·focus·paste byte상한·ScrollIntoView를 검증했다. [root RED/GREEN](../validation/evidence/windows-msvc-diagnostic-editor-green-20261011/root-review.json). 이후 readonly 폭 정렬과 새 ignored soak 파일을 포함한 whole 검사를 실행 중이다. 새 release의 native glyph/provider·설치ZIP 인수 전이며 물리IME/reader·전체P0 성공은 주장하지 않는다.

첫 실제 실행은 observer 컴파일에서 `TextPatternRangeEndpoint`와 `TextUnit` namespace 누락으로 실패했다. [원 실패 기록](../validation/evidence/windows-msvc-diagnostic-multiline-current-20261011/compile.log)을 보존했다. 제품 실행 전 실패이며 입력 인수 성공으로 계산하지 않는다. 원 소유 compiler는 자연 종료하고 Job0·private desktop 종료·input desktop 불변을 확인했다. 후보에 `System.Windows.Automation.Text` import 한 줄을 추가한 뒤 새 evidence 경로에서 다시 실행 중이다. 제품 Rust·Cargo·lock 변경은 없다.

## 2026-10-11 ScrollIntoView의 실제 커서 회귀

현재 1,015개 통과 후보의 native GUI에서 CtrlEnd 이후 접근성 ScrollIntoView가 전체 문서의 가운데로 이동하여 ROW35 커서를 가리는 것을 캡처했다. `windows-msvc-diagnostic-editor-current-gui-20261011/root-review.json`은 알려진 UI 버그를 남기며 화면 인수를 완료로 표시하지 않는다.

실제 TextEdit galley의 커서와 clip을 검사하도록 기존 회귀를 강화했다. `windows-msvc-diagnostic-scrollinto-caret-red-20261011`에서 컴파일 성공 후 해당 assertion이 실패(exit101)했고 소유 Job이 자연 종료했다. exact editor의 ScrollIntoView만 소비하여 부모의 제한된 viewport를 표시하고 실제 내부 커서 위치를 유지하도록 수정했다. Focus/선택/키보드 요청과 전체 접근성 문서 좌표는 유지한다. viewport에는 focus/interaction 외곽선을 추가했다. `windows-msvc-diagnostic-scrollinto-caret-green-20261011`에서 동일 실제 테스트 1개가 통과했다. 전체 검증과 새 바이너리의 native PNG 확인은 아직 진행 중이다.

교정 소스의 whole fmt/clippy/test/release는1,015/0/28(43harness) 통과했다. `windows-msvc-diagnostic-caret-current-gui-20261011/root-review.json`에서 정확한 입력9건·noncollapsed LINE-C 실제 선택·invalid index 원본 유지·두DB integrity·원본/소스SHA·natural exit/Job0를 확인했다. root는 실제 PNG에서 ROW35가 보이는 것과 선택 표시를 확인했다. collapsed provider caret geometry는 미지원으로 유지하고 실제 painted galley/clip 회귀 근거와 구분한다. 물리IME/입력·reader 및 모든 disabled/action 조합의 전수 인수로 확대하지 않는다.

## 수동 도구 설정을 포함한 최신 설치 후보 재검증

windows-msvc-diagnostic-caret-tool-preferences-fifth-20261011/root-review.json에서 현재 CPU SHA3d7c4f5f69b4f8857b13c80c6f8e92b538a616060c8c92513299da6c0f627c78로 실제9개 입력 readback·LINE-C 선택·invalid index 원본 유지·두DB integrity·source620·원SHA·natural Job0를 root 재검증했다. root PNG에서 마지막ROW35를 직접 확인했다. 기존4줄+추가36줄의 총40줄이며 전체36줄이라고 계산하지 않는다. collapsed provider caret geometry·물리IME 인수는 여전히 미지원/미검증이다.

현재 CPU806bf의 실제 private Windows 재실행 root-review PASS_SCOPED: 9입력 readback·LINE-C noncollapsed선택·40줄에서ROW35 실제PNG·invalid999999 보고서byte보존·closedSQLite integrity/단일Failed job·원source620/원SHA/raw natural0/Job0를 대조했다. windows-diagnostic-current806bf-20261011/root-review.json. 물리IME/reader 및 providercaret geometry는 미검증이다. 기존3d7c… 결과와 별도snapshot이며 source 수정 후 소급하지 않는다.

최신 picker-fixed CPU9653760 SHA2f45fd... 실제진단 private재시험0 및 scopedreceipt/root검토통과. 9primary exactinput/readback, LINE-C selected geometry, ROW35 bottom·ROW34 Home/Delete 캡처를 root직접확인했고 actualDB download_jobs 1 Failed/integrity ok, 잘못된index 전후report 원bytes불변을검토했다. physicalIME/reader 및collapsedcaretgeometry는미검증유지. 최신binary/scope623로기록하며 이전806bf판정을재사용하지않음.
