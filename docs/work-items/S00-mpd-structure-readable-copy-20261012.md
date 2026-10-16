# S00: MPD 구조 상세의 사용자용 표현

- 상태: completed
- 분류: UX
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: root — accessible_diagnostics.rs/catalog.rs(현재native source동결해제후), 이 문서; diagnostic_cache — /tmp 구조상세native후보만
- 선행 작업: MPD 선택/첨부/제외 양테마byte검증과구조상세원캡처

## 사용자 결과

MPD 상세를펼치면초기화·미디어구간과바이트범위를개발용Some/None표현없이읽는다.

## 계약과 범위

현재UI가format Debug로MpdNodeKind/Optionrange를직접그리는것을source확인했다. 중앙catalog명칭과있는범위만표시하는읽기표현으로교체한다. 원JSON/report/구조정책/privacy/MPD원파일/worker/입력ID불변. 현재1195source actual시험이끝나기전제품source를바꾸지않는다.

## 인수 조건

4kind를명확한명칭으로구별하고range/media_range가있으면원값을보이며없으면None을그리지않는다. rawURL을새로표시하지않는다. actual첨부상세/제외PNG와원3report·피커·privacy회귀유지.

## 검증과 handoff

source에서Debug직접출력을발견해actual구조상세캡처를위임했다. 아직제품수정/시각인수미완료다.

- 원visual후보copy tuple형식을root AST사전검사로거절해native미시작. 새v2clone에서selection_review/visual_review를정확2pair로분리·모든pairlen2검사통과후Light93146실행중. 원1195source의Debug구조actual캡처후사용자표현수정할예정이며제품source동결유지.

- visualLight93146 exit2:compileCS0170로Rect client.bottom/right definite-assignment오류가나GUI미시작. root가새v3에서MpdBodyGeometry의Rect초기화만보정·원Win32성공Require유지하고Light40154fresh실행. actual구조픽셀인수미완료/원v2보존.

- visualv3Light40154 exit2:compile통과/actualMPDattached-structure provider5행wholebody와PNG는생성됐지만3번째export완료119386ms뒤omittedcapture전에120sworker timeout. 원전체workflow는FAIL/natural미인수로보존. root가생성된attachedPNG를직접읽어RustDebug Some/None표현을확인하는좁은실패run의시각근거로만사용한다. functional0/1/0양theme성공과합쳐이실패를통과로쓰지않는다.

- 모든원native/독립handleterminal후source동결해제. root가Kind4개중앙catalog명칭과있는range/media_range만바이트범위로표시하도록제품UI수정했다. Some/None·Debugenum은없애고rawURL필드는새로읽거나표시하지않는다. 원MPDJSON/worker/schema/Cargo/입력ID불변. 단순읽기문구변경에구현복제test를추가하지않으며기존진단/privacy회귀·새whole·actualUI검증을이어간다.

- 새 전체 검사31007과 독립 원 로그·소스·artifact 대조20413이 모두 exit0으로 종료했다. fmt/clippy/test/release 근거는 `windows-msvc-mpd-readable-copy-whole-20261012/root-review.json`이며1195성공/0실패/30ignored 범위다. CPU는9,762,304바이트, SHA-256 `58f8c2aa43281040edf65fb11c8474eadb85ee1b2199bb09993e2ae8d3eb9715`이다. 이전 설치 ZIP은 별도1195 snapshot이며 이 문구 변경을 포함한다고 쓰지 않는다. 새 문구의 actual 구조·제외 캡처는 아직 미인수다.

- root는 새 문구의 실제 구조/제외 캡처 전에 `/tmp/dlpad-diagnostic-input-batch-capability-native-20261012` 입력 시험을 선행한다. 담당 파일은 이 작업 기록과 새 evidence이며 제품 Rust/Cargo 변경 없음. 인수 조건: 16scalar 단위 입력의 원 PID/focus/UTF16 pair/각 prefix/full readback, 선택 해제 ACK, 원 fixture/privacy/readonly DB, private desktop 및 자연 종료를 유지한다. Light11714 실행 중이며 성공 전 MPD visual에 채택하지 않는다. 원120/150초 한도와 실패 로그 보존 계약을 유지한다.

- 선행 batch capability Light11714 exit0. root가 원 raw compile/gui guard, 현재 source map, original SHA, 실제2개 prefix/fullreadback receipt·1개 selection clear, readonly DB/자연 종료를 독립 대조해 `windows-msvc-diagnostic-input-batch-capability-light-native-20261012/root-independent-review.json`에 기록했다. physicalIME/MPD/export 인수는 이 시험 범위가 아니다. 그 뒤 새 catalog MPD Light24326을 fresh output으로 실행 중이며 제품source 동결을 유지한다.

- 새 catalog Light24326 exit0. root가 원 로그/source/hash/natural/fixture/privacy/3report[0,1,0]/clear/provider를 독립 대조해 PASS를 기록했다. attached PNG를 직접 확인해4개 한국어 명칭과range0-31/32-99, Some/None 없는 표현을 확인했다. Dark89215 실행 중.

- Dark89215 exit0. root가 양테마 원로그/source/originals/natural/3report/privacy/clear/provider 독립 검토와 첨부·제외2PNG씩 직접 검토를 기록했다. 이 작업은 사용자용 명칭의960x640100 표시와 기존 기능 회귀 범위로 완료한다. 확대200/MPD cancel/owner close/전체UI 인수는 별도 잔여이며 제외 후 모든 버튼 fullpaint는 주장하지 않는다.
