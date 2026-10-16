# UI: 스크롤 도움말 native provider 관측 준비

- 상태: proposed — 실행 전 관측 후보, 전체 인수 미검증
- 분류: validation
- 우선순위: P0 UI 검수 후속
- 요구사항: R08, 사용자 UI/UX 전수 검수
- 담당/수정 범위: field_reveal; help-scroll-native-v1 scratch helper 파생본/기록만
- 선행 작업: Root small-popup 실제 Context RED 교차 실패 / GREEN 통과; 새 source whole 완료 후 명시 proof 인자

## 사용자 결과

좁은 화면에서 입력을 덮지 않는 스크롤 도움말을 실제 native 화면으로 검수할 근거를 수집한다. UIA provider가 제공하지 않는 viewport/frame을 임의로 만들지 않는다.

## 계약과 범위

기존 help-protected-native-v2는 그대로 보존하고 새 namespace에 관측만 추가했다. 기존 current exact whole/source/3PE proof gate와 private Desktop/소유 PID·HWND/Job/natural0/input 불변, 동일 960×640 physical/100% 및 800×600 physical/200% Light 두 scene, 세 원 도움말 Invoke, 각 Escape6, 원 provider IDs와 원 fullclip guard를 유지한다. 새 helper는 원 raw title/body bbox를 그대로 원자료로 보존하고 이를 clipped viewport로 쓰지 않는다.

locked egui ScrollArea는 scroll UiStackInfo만 달고 GenericContainer 자식 Ui를 만든다. Ui의 bounds는 min_rect 기반의 content bounds이며 inner viewport나 frame/shadow bounds가 아니다. locked accesskit_windows0.29.2는 ScrollPattern/IScrollProvider를 구현하지 않고 ScrollItemPattern만 제공한다. TextPattern.GetVisibleRanges는 미구현으로 null을 반환한다. 따라서 원문 rawbounds 및 ancestor를 viewport/보이는 글리프로 승격할 수 없다.

새 observer는 pinned title/body 각각 RawWalker ancestor chain(최대16, 동일 owned PID와 exact HWND 도달), 각 Bounds 및 Scroll/ScrollItemPattern 지원 여부·스크롤 속성을 저장한다. TextPattern 원 DocumentRange가 원 Help FullDescription과 일치하는지 확인하고 visible ranges가 있으면 raw로 기록한다. bounds semantics/glyph clip 검증은 false다. 원 associated Edit 값 및 Edit/browse 좌표도 pinned 전후/Escape 뒤 불변을 확인한다. 값을 로그에 쓰지 않는다.

## 인수 조건과 한계

원 helper의 lifetime/IDs/Escape/fullclip 인수는 그대로다. 새 관측 계약 help-scroll-native-observation-v1은 clipped popup ROI/frame/shadow를 식별했다고 주장하지 않는다. 실제 신뢰할 popup ROI가 없어 WM_MOUSEWHEEL을 보내지 않고, 마지막 문장 glyph 접근·처음/끝 PNG 쌍은 false로 남긴다. 원 first pinned PNG와 raw source/provider 관측은 Root가 직접 검수한다. 결과 상태는 REVIEW_REQUIRED_SCROLL_PROVIDER_UNAVAILABLE이며 전체 native UI 통과가 아니다.

## 검증과 handoff

Python ast.parse만 모든 파생 .py 파일에서 통과. C# compile/native 실행 없음. Root가 실제 whole/result와 독립 root-review를 명시 인자로 넣어 빌드/실행한다. 미래 source count/hash/실행 종료값 예측 없음. 기존 증거/Live/S/Cargo 수정 없음.

비용: 제품 코드/의존성/PE 크기 변화 없음. observer raw ancestor 최대32개와 최대16개 visible text ranges(각 원문32768자/rect256double 한도)를 추가 관측한다. 이는 테스트 관측 비용이며 제품 상주 비용이 아니다. 신뢰할 viewport provider 추가나 전역 입력은 하지 않는다.

Root가provider미지원범위를읽고가상Frame/viewport로성공시키지않는관측범위를채택했다. v2에서원본문첫줄좌표와소유HWND의boundedwheel로실제glyph검수를보완할수있는지준비중이며아직미실행이다.

v2실제compile1/GUI미실행: managed UIA GetBoundingRectangles는Rect[]인데double[]로선언한fixture타입오류다. v3에서이타입과같은최대256좌표투영만수정한후새출력current-help-scroll-private-v2-20261016에서실행중이다. 원컴파일실패namespace는보존하며인수조건을완화하지않는다.

Root 실제 v3는 compile0/100%3행통과 뒤200% FFmpeg 첫 wheel에서 본문 raw top이 움직이지 않아 실패했다. 원 보호input/browse/값/ID는 유지됐고 Root 처음/실패PNG에서 popup이 입력을 가리지 않는 것은 확인했다. native마지막문장·Deno/나머지Escape는 미실행이다. wheel원인은 미확정이며 별도 제품 키보드 접근성 회귀와 native키 경로를 준비한다.
