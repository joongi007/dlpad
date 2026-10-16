# S00: 실행 파일 입력의 focus 문맥 공개

- 상태: proposed scratch / actual RED·GREEN 미실행
- 분류: bug
- 우선순위: P0
- 요구사항: R08 접근성·키보드·입력·clip
- 담당/수정 범위: B/executable-field-context-reveal-v2 accessible.rs 후보만; Root가 Live/S 통합과 실제검증 담당
- 선행: 현재 Root 검색/Help native·source guard 완료 및 채택 ACK. Live/S/docs/E/Cargo 변경0.

## 사용자 결과

실행 파일 도움말·입력·찾아보기 중 하나에 실제 focus가 들어오면, 라벨+도움말+입력+찾아보기 전체 그룹이 viewport에 들어갈 수 있는 경우 그룹을 공개한다. 그룹이 더 크면 focused control만 공개한다. 기존 LabelledBy와 input globalID, help stableID/tooltip/pinned 기능, browse availability/클릭 반환은 유지한다. focus를 새로 요청하거나 path를 수정하지 않는다.

## 계약과 범위

Settings reveal_setting_context/reveal_setting_row의 실제 group-fit→Center 및 oversized→response.scroll_to_me(None) 정책을 같은 private helper로 적용한다. 전체 그룹은 실제 response.rect들의 union이며 guessed spacing 또는 text-mirror가 아니다. helper는 enabled&&gained_focus 때만 호출된다. common control 기존 reveal보다 나중에 group 공개를 요청해 전체 의미가 마지막 scroll target이 되게 한다. 기존 ui.horizontal/with_layout/widget 호출순서 유지, 반복focus/wheel 때 매프레임 재공개하지 않는다. DB/worker/controller/catalog/deps/schema/lock 변경0.

## 인수 조건

두 actual Context 시험은 도움말/input/browse 각각100/200 focus를 actual current AccessKit NodeId로 주입한다. 실제 viewport height를 max_height96/56으로 확인한다. 들어맞는320폭/96높이 그룹은 모든 actualresponse와 한글label painter의 fullclip, focus/IDs/LabelledBy 및 original bool 반환을 유지한다. 큰200폭/56높이 그룹은 focused control fullclip만 요구한다. disabledparent 및 unavailableBrowse는 실제focus/offset/문자열 변경을 거부한다. focus 유지 중 실제6Point wheel을 받은 뒤 후속frames snap없음을 검사한다. 이어 별도의 실제 TextEdit에 current AccessKit Focus를 전달하여 새 focus와 원 ScrollArea offset 불변을 함께 확인한다. Context는 전체 Main/nativeOS/Reader/IME/물리포인터 인수가 아니다.

## 검증과 handoff

필터 actual_executable_focus_reveals_fitting_label_help_input_browse_group_once 및 actual_executable_oversized_group_reveals_only_focused_control_and_refuses_disabled. 원 production에서 RED probe/tests 실행→GREEN 동일필터+existing tool_browser_fields_fit/입력caret/help/strict/fullwhole를 Root가 실행한다. compile/setup FAIL은 semantic RED가 아니다. 현재 파일 rustfmt0, Cargo/native0, Source는 scratch3(before/red/green)만 생성했다. 새 지속state/DB I/O/thread/의존성0, 추가 비용은 focus transition의 작은 rect union/fit 비교와 기존 scroll animation이며 실제runtime·binary cost 미측정이다.

Root: 실제 Windows selected RED/GREEN 통합 대기. Live/S 수정 없이 별도 후보만 검토했습니다.

Root actual RED-v1: compile 성공, fitting Context는 group clip semantic FAIL을 재현했습니다. oversized Context는 ScrollArea 기본 최소 높이가56보다 커 setup guard FAIL이므로 동작 재현으로 집계하지 않습니다. v3는 RED/GREEN 시험 둘 다 min_scrolled_height0으로 실제56px fixture를 만들며 동작/조건은 유지합니다.
