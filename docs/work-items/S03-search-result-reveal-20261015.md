# S03: 검색 완료 결과의 한 번 공개

- 상태: in-progress / 후보 실제 검증 중
- 요구사항: R08 접근성·입력·스크롤, S03 전체DB 목록검색 UX
- 소유: B/search-empty-reveal-v1의 list_search.rs/accessible_library.rs/accessible_history.rs scratch와시험·문서draft만. Root Live/S/source/E/Cargo/lock/schema 수정0.
- 근거: actual v8은 Library 초기목록·128 밖 일치검색·clear를 통과했으나 no-result Label이 body 아래에 있어 unsupportedFocus로 실패했다. 원증거와5PNG를 보존한다. Reader/물리wheel/physicalIME PASS로 표현하지 않는다.

## 사용자 행동

Library 성공 빈 검색 결과는 기존 ScrollArea 안의 정확한 안내를 한 번 공개한다. populated first-row 자동이동은 도입하지 않는다. History는 header/search/paging/result가 분리돼 800/200%에서 records 영역이 거의 남지 않는 source 구조를 하나의 body ScrollArea로 묶는다. 새 nonempty query의 matching success에서 최초 recordLabel 또는 emptyLabel을 한 번 공개한다. Label은 그대로 비포커스이고 입력 포커스를 요청하지 않는다. 초기전체목록 generation0·동일generation refresh/page·clear 전체목록 첫행은 자동이동하지 않는다.

## 안전 계약

공통 Search에 Option<u64> 한 슬롯의 pending result-reveal ticket만 추가한다. accepts matching request, first publication, generation>0, current draft.trim()==publishedquery일 때만 생성한다. 새 enqueue(실패입력도 포함)/새 request/worker failure/nonempty Library/History whole-list/Stop/disabled/stale에서 폐기한다. result response render 때 토큰을 take하고 같은generation·!stale·fullclip 미충족인 경우만 기존 scroll_to_me(None)를 한 번 요청한다. 중복 success는 재생성하지 않는다. visible Label description에 실제 published query를 연결하며 새 draft를 새 결과로 표기하지 않는다. worker serial/generation/queue1/rows128/DB읽기/오류시 이전rows/Resume entityID/프로세스수명 권한은 그대로다.

## 인수 조건과 실제 시험 후보

실제 Context/panel.show + sync_channel commands/outcomes 4개 회귀를 제공한다. Library125logical body100/200의 빈 안내 actual complete painter/AccessKit Label noFocus·inputfocus 불변·실제wheel +6 direct smooth 후snap없음; stale draft/old query/failure/Stop의 actualoffset 무이동·기존rows 보존; History 같은100/200 body의 Cancelled 비포커스record/빈Label actualfullpaint·inputfocus 유지; History stale/old/failure/Stop의 이전결과 무공개. 실제 source event와 worker envelope를 이용하며 토큰값을 복제해 성공을 주장하지 않는다. RED-only variants는 production baseline에 시험+fixture 관측만 추가한다. baseline compile/setup 실패는 semantic RED가 아니다. Root 실제 MSVC 전까지 결과 미검증이다.

## 비용·후속

추가per-panel UI token 최대 Option<u64>16B, 큐/worker/DB I/O/thread/deps0. History는 기존128 rows 렌더 상한 유지, 새 nested scroll 제거 및 once-scroll animation만 있으며 CPU/size 실제비용은 미측정이다. component125logical Clip Context가 전체Main/footer/native800200을 대체하지 않는다. Root가 actual RED→GREEN/기존History action geometry/입력caret/workerStop/strict/newwhole와freshprivate native empty+History result를 검증한다. 광범위한 스타일·타이머·물리입력 변경은 없다.

Root 실제 RED: managed-tool-isolated-search-reveal-red-v1-20261015, 4필터 컴파일 성공. Library empty reveal 및 History stale/Stop 회귀가 각각 0 PASS/1 FAIL, 나머지 2필터는 각각 1 PASS. Live 및 후보 스냅샷 변경 없음. GREEN 3파일은 S에만 반영하여 실행 중이며 제품 실행 파일은 아직 whole27입니다.

GREEN-v1/v2는 WSL UtilBindVsockAnyPort 소켓 오류로 Cargo가 시작되지 않았습니다. 권한 검토를 거친 동일 private Windows 실행 GREEN-v3는 컴파일 후 실제 Library empty 회귀 0 PASS/1 FAIL(offset 변화 검증)이며 후보는 미채택입니다. 별도 수정 후보와 재시험이 필요합니다. Live는 whole27 유지.

Root viewport 교정 RED-v2: Library empty 및 History result positive는 실제 각각0 PASS/1 FAIL, stale/error/Stop negative 두 필터는 각각1 PASS였습니다. combined app GREEN-v1에서는 검색4개 실제 모두 PASS, 전체 앱은458 PASS/2 FAIL/6 ignored로 field wheel 및 Help outside focus 시험이 남았습니다. 아직 Live 미통합입니다.
