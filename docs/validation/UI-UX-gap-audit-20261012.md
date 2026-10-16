# 현재 UI/UX 잔여 감사 — 2026-10-12

읽기만 수행. 프로젝트/native/Cargo/하위agent 실행0. README→PRODUCT→CONTEXT 및 DESIGN 현재판, final-ui workitem, GTMUX 실제 Input/Button/Tooltip/tokens, 현재 Rust source를대조했다. 최신 feedback-scaled Light markdown-explicit-save.png/save-action-final-app.png를직접view했다. root가인수한Save feedback100/200양테마/기존파일거절·pagination18matrix/10k·choicepressed/checkbox/focus/첫diagnostic공개는재발견으로세지않는다. 좁은진단성공으로전체UI인수를대체하지않는다.

## P0-UX 1 — 잘못된 로그 번호의 오류가 동작 문맥에서 떨어진다

사용자행동: 잘못된로그번호입력→선택한로그수정/제외. accessible_diagnostics.rs490..495가INVALID_LOG_INDEX를FeedbackScope::Logs로설정하지만실제show_feedback는613에서 editorviewport/apply 뒤에출력된다. editor는확장시120..200logical높이+labels/actions를소비한다. 800200의작은본문에서는사용자가누른index/action과오류가동시에보이지않을수있다. 이는코드상actual배치차이이며최신invalid-logPNG재현은아직없다. 실제oversizedbody/지원하지않는virtualkeyboard제품버그라고확정하지않는다.

기준: DESIGN 입력옆오류와고치는방법, GTMUX Input.svelte의control바로뒤error rolealert. 최소수정: Logsfeedback를index/actions바로뒤에두고apply결과가필요하면apply인접feedbackscope를UI-only로구분(공유schema변경없이root결정). 같은오류문구두번그리지않는다. invalidindex는report/draft를바꾸지않아야한다.

인수: actualDiagnosticPanel.show Context invalidindex→actualaction에서오류labelrect와index/actionrect가같은viewport범위, AccessKitdescription/invalid 연결, reportbytes불변; native800200Light/Dark실제invalidaction/오류PNG. 성공Save/Attach scope에영향없음 확인. 담당accessible_diagnostics.rs/catalog필요시root.

## P0-UX 2 — 높이를제외한고급입력은invalid상태를입력에전달하지않는다

사용자행동: 고급옵션열기→재시도21/조각동시성0/속도제한0입력. accessible_options.rs294..301은실제AdvancedOption.validate 오류를colored_label로출력한다. height분기185..188은node.set_invalid/clear_invalid와description을연결했지만고급field_response305..339는labelled_by만설정한다. 따라서기존native숫자오류문구/plan시험통과가고급Edit의Windowsvalidity전달을증명하지않는다. Windowsprovider30103mapping은공통준비됐으나호출부가invalid를설정해야한다.

기준: GTMUX Input.svelte aria-invalid/errorborder/alert; DESIGNerror상태정의. 최소수정: 각AdvancedOption의실제validation결과를그fieldresponse에연결해invalid/description을set/clear. core검증/입력ID/revision/planflags/disabled정책불변. Height코드를검증과독립한상수predicate로복제하지않는다. visualdangerborder는focusaccent와별도로검토하며전역theme변경금지.

인수: actualOptions.show invalid→valid의해당EditAccessKitinvalid/description→currentWindows30103false→true/30159정확core오류→제거, receiptmatchedopaqueID와실제plan오류/복구검증. scopedpure+native회귀. 담당accessible_options.rs, catalog새문구필요없음.

## P1-UX 3 — 잘린전체경로를keyboardfocus에서공개하지못한다

사용자행동: Tab으로긴진단저장/MPD/폴더pathfocus,전체경로를확인하려고마우스없이머문다. diagnostic_path_field accessible_diagnostics.rs728 및path_picker.rs214는on_hover_text(fullpath)만제공. vendor/egui/tooltip.rs338은enabledresponse에hovered&&pointer.has_pointer를필수로하여focus만으로tooltip이열리지않는다. GTMUX Tooltip.svelte31..34는mouseenter와focusin/focusout둘지원하며DESIGN tooltip은hover와keyboardfocus둘지원한다. 최신SavePNG에긴경로가한줄폭에서잘리는것자체는정상TextEdithorizontalviewport지만fullcontext의keyboard공개경로가없다는codegap과구분한다. ValuePatternfullreadback은screenreader정보이며육안keyboardhelp를대체하지않는다.

최소수정: 특정경로field의focus에작은fullpathread-onlydisclosure/tooltip을한정하고bounded문자/줄/높이로표시. active편집caret/selection/가로scroll을뺏지않음. essentialdisabledpickerreason도hoveronly로남기지않는정책은같이검토하되이번항목에기능추가를조용히묶지않는다.

인수: 실제ContextPointerGone+Tab에서전체경로helperlabelpaint/입력focusID/selection안정;focusoff닫힘,hover동일경로,disabled에서pointer별도fallback없이이유읽기. nativeLongUnicode800200 Light/Dark실제PNG와Home/End/caret회귀보존. 담당ui_components.rs/path_picker.rs/accessible_diagnostics.rs(root파일조정필요).

## 명확한미검증경계

실제OSphysicalhover/IME전체인수는syntheticegui/windowmessage성공과구분한다. 위3개중1의native문맥clipping은재현전,2·3은현재소스의semantic/trigger차이를확인한것이다. 색상/selected/caret가일반적으로부족하다고추측하지않는다. 진단path피커JSON/MPD/cancel/close개별인수는root진행중이며새소스/캡처확정없이미완료제품결함으로중복제시하지않는다.

### 읽은직접캡처 SHA
- docs/validation/evidence/windows-msvc-diagnostic-feedback-scaled-light-native-20261012/gui-single/markdown-explicit-save.png: `f2ee0259aec307f28fa4c770812878a966b579b508d0471286f5b5922d261963`
- docs/validation/evidence/windows-msvc-diagnostic-feedback-scaled-light-native-20261012/gui-single/save-action-final-app.png: `4e0a57d249790fab1babaefbbaa3252d31ce0f072a9cda2231967a11b22fa00c`
