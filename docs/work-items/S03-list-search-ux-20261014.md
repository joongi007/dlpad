# S03: 파일 목록과 작업 이력 전체 DB 검색

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R03 파일 목록, R08 작업 이력·접근성·bounded I/O
- 담당/수정 범위: windows_auto accessible_library.rs, accessible_history.rs 및 private list_search.rs; installer_batch engine/library.rs + engine/download.rs; Root catalog/main module 등록
- 선행 작업: Root 승인 DB query API/공유 progress template 보존. Root isolated source/기존 evidence 수정 금지, Live overlay는 원 실행 terminal ACK 후.

## 사용자 결과

파일명·경로와 작업 ID·실제 저장된 원 URL을 전체 현재 DB에서 검색하고128개씩 다음 페이지를 확인한다. 제목은 현재 이력 DB에 저장하지 않아 제목 검색을 약속하지 않는다. 입력 label/Enter·검색·지우기/결과 없음/조회 오류/이전 결과 유지 상태를 구분한다.

## 계약과 범위

README→PRODUCT→CONTEXT와 R03/R08·ADR0002 bounded readonly paging·DESIGN/UX를 따른다. sqlite schema/lock/dependencies/core job는 변경하지 않는다. query trim/최대256 UTF8 bytes/NUL거절/literal substring/parameter binding/keyset128을 유지한다. 파일 목록은 현재 성공 reconciliation snapshot을 검색하며 현재 파일권한이나 새 fullscan 완료를 뜻하지 않는다. history snapshot은 argv envelope이고 원URL은 options_json의 StoredOptions $.url(version1)에서만 검색한다. NULL/legacy에서 URL을 추측하지 않는다. 전체jobJSON/옵션내부text를검색하지 않는다. 모든 DB I/O는 한 worker/capacity1 mailbox, 요청 generation로 늦은 결과 게시를 거절한다. 기존 export/open/resume/CloseWait/cancel/ACK 수명은 유지한다.

## 인수 조건

DB128페이지 경계를 넘어 한글/공백/literal %·_·quote/clear 검색, keyset 중복 없음,256byte/NUL/NULL·malformedoptions/DBbusy/corrupt 실패를 검사한다. 실제Context 입력/Enter/Clear/label/결과없음·오류·stale표시,late/outofordergeneration 게시거절을 관측한다. 이전 결과 액션은 기존 exact worker 재검증/개체 ID를 사용한다. OS IME/physicalreader/nativeGUI는 Root actual 후별도이며단위시험으로완료표현하지않는다.

## 검증과 handoff

구현/회귀 준비 중. Cargo/native0. Rootcatalog8문자열/private 모듈등록/전체검증 담당. dependency/schema0,querypayload256B/결과128행/상시worker추가0. substring DB keyset scan 비용은 데이터수에비례할수있고UI I/O로옮기지않는다. B/list-search-v1 exactbeforepins/fullfiles/patch/HANDOFF로전달하며Rootwhole후비용·실제GUI인수필요.


UI/worker3파일 최종동결: B/list-search-v1/final-source-pins.json·HANDOFF. 명시검색/CtrlF/Enter/Clear,게시된검색어와초안stale,128page cursor/단일worker/latestqueue1/serial+generation 게시거절/실패기존rows보존/Stop gate를연결했다. 실제Context/채널회귀3개작성,1.94파일rustfmt0,UI Cargo/native미실행. backend5회귀RootactualPASS보고는별도범위. Root main모듈등록뒤실제app전체/strict와freshwhole/native를인수한다. Live source추가편집중단/엔진이관확인. 원before/scratch를최종engine후보로승격하지않는다.


첫v26 whole 실제3FAIL(Root fmt/clippy0, 첫app402/3/6ignored,release미실행)을원증거로보존했다. B/list-search-ui-v2 최소교정은빈폴더필수안내를동일scroll경로뒤/검색조작앞으로공개하여원246clip/wheel인수를유지한다. readonlylocation시험은원rooteditor NodeID부재와새검색TextInput존재를구분한다. scope시험은Label value/name과실제clip내갈레이paint를동시관측한다. 1.94rustfmt0, childCargo/native0. 새인수proof는windows-msvc-ui-feedback-v26-whole-v2-20261015이고actual결과전성공주장없다. Windows native검색helper는별도B/v26-search-private-v1에준비하며whole-v2Rootreview없이실행불가;원도구private helper/실패pins는보존한다.


Root 새whole-v2-20261015 실제1570/0/42ignored/43harness·source673/current3/raw 독립검토PASS 보고를받았다. 새 B/v26-search-private-v1 helper는그exactproof+동일root-review/currentmap을gate하며준비만완료했다. Root단독명령은argv.json, freshoutput current-ui-feedback-v26-search-private-v1-20261015. 원도구privatehelper와Run ownedDesktop/Job/MTA/natural/input/256node/64KiB guard를보존한다. native-created freshroot에140syntheticfile·owned3DB/Cancelled2job를seed,첫128밖파일target·clear·noresult·ID/실제optionsJSONv1.URL query를ValuePattern writable+actualquery/행ACK로확인한다. missingValueSetter는FAIL/fallback0. 예상6PNG/800Light200 한조건이며실제수집성공을주장하지않는다. 이력native128경계/Enter/물리IME·reader는미검증;원nodebound를늘리지않았다. capsCSC45+prepare30+GUI230/MTA180,Source/current3/schema/hash·140media/Jobsbyte/3DBintegrity/nativeparentIDs를검토하며Scope수행과pixel가독성판정은Root몫. AST+CS lexicalbrace만확인,Cargo/native/seed실행0. 원whole26FAIL/최초helpFAIL/성능FAIL을그대로보존한다.

2026-10-15 검색 사용자 문의에 대한 현재 상태: 검색 구현은 작업 소스에 포함됐지만 README의 배포 후보 v25에는 없다. v26 전체 검사 두 번째 실행은 실제 1,570 통과였으나 이후 Windows UIA 문자열 입력 무반영을 발견했다. 새 vendor 수정 후 전체 검사와 private 검색의 실제 입력·결과 ACK를 다시 확인하기 전에는 새 설치본 검수가 가능하다고 보고하지 않는다. 검색 범위는 파일명·경로와 작업 ID·저장된 URL이며, 설정/도구의 전역 검색은 이번 작업에 포함되지 않는다.
