# S00: optional 설명의 공통 도움말 버튼

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R08 기본화면·접근성·가벼운UX
- 담당/수정 범위: windows_auto ui_components.rs 및이문서/B/context-help-button-v1; Root catalog/accessible.rs 통합
- 선행 작업: Root source671 gate종료/편집해제; freshwhole/native인수Root담당

## 사용자 결과

필수상태는화면에남기고선택적긴설명을32px ?버튼의hover/keyboardtooltip및고정가능설명으로읽는다. 독립적인purposeName과description,보이는focus,Enter/Space/click로열기와Escape/outsideclick으로닫기를제공한다.

## 계약과 범위

PRODUCT/DESIGN의native컴포넌트/필수경고inline/keyboardtooltip계약을따른다. ui_components::help(ui,id,title,text)->Response는논리key를명시하고기존Ghost팔레트/context_help/eguiPopup을재사용한다. Rootcatalog HELP_BUTTON='?' 추가. appearance/worker/schema/Cargo/performancebudget/기존필수warning오류와disabledreason을변경하지않는다. disabled부모는자기popup을닫고클릭을거절한다.

## 인수 조건

28px이상hitarea(현재32),중립hover/명확focus,ButtonpurposeName+description,실제Contextpointer/keyboard열기/닫기/disabledrefusal/개체재배치identity보존을확인한다. actualheading/bodyglyph가popup/tooltipviewportclip내있는지Light/Dark와100/200합성논리viewport에서관측한다. Root의800폭200배율caller/footer본문충돌과private OSUIA/reader/physicalfocus는별도native인수이며Context통과로대체하지않는다.

## 검증과 handoff

1.94rustfmt파일별exit0. Cargo/native시험미실행. 새actualContext2filters actual_optional_help_keyboard_pinned_close_and_stable_identity, actual_optional_help_pointer_hover_open_and_outside_close; 기존keyboard_context_help_tests/command_state_tests 재사용. 정확basepin/fullfiles/patch/catalog요청/HANDOFF는B에동결. Root통합freshwhole/payload/GUI후판정하며준비를제품UI완료로표현하지않는다. 새dependency0/font0/motion0/workerI/O0;popup갈레이와닫힌Button/shape비용은native미측정.


Root 실제Context 후속: 초기match Help 누락컴파일실패/positional child ID 재배치실패/Escape후focus없음/tooltip첫sizing frame/1초press-release를모두원실패로보존했다. global UiBuilder.id scope와Escape 자기owner focus반환(외부clickfocus불간섭)을최소수정했다. hover본문이실제owner밖/viewportclip내paint되는≤4frame setup관측후pointer50ms click을사용한다. egui 기본max_click_duration0.8s보다원1초가길어click이아니었던source규칙과실제v9 release5.10 clicked=true/pinned=true/1pass를확인했다(Root관측). 다중pass가설은이번click실패원인으로확인되지않았다. Root v9 실제2PASS/0FAIL·source/currentcandidate변경없음/Job/privateguardPASS이며합성Context범위다. 실제제품private 도움말popup/reader/physicalfocus는freshwhole후Root별도. 도움말source최종동결상태유지.
