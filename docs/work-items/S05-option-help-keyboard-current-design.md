# S05: 현재 옵션 도움말 키보드 인수 설계

- 상태: completed (설계/읽기 감사; keyboard 실행 별도)
- 분류: validation-design
- 우선순위: P0
- 요구사항: R07, R01
- 담당/수정 파일: soak_recovery_audit — 이 문서만
- 의존성: root CPU help5 pointer/AT-SPI 인수 및 기본Iced스크롤 인수, 기존plan keyboard/nativeX11 helpers

## 범위와 인수 조건

CPU help5 Enter/Space/focus/expanded실제검증을기존ownedprivateGUI harness에추가할위치를읽기설계하고기본Iced tab순서후속scope를제시한다. Rust/Cargo/기존script/GUI실행은없다. 현재pointer인수를reader/keyboard인수완료로확대하지않는다. Root가현재scripts의독점owner다.

## 현재 실제 근거와 실행 범위

[CPU third help result](../validation/evidence/p0-option-help-gui-third/help/result.json)은root pointer/AT-SPI expanded state와fulltail bounds 인수다. FFmpeg·cookie·quality·range·cut 다섯header를click→state→tail/screenshot→clickclose한다. 이것은키보드Enter/Space나screenreader발화검증이아니다. 기본IcedFFmpeg/range/cut스크롤화면인수도기존sourcehelp출력가시성이고tabnavigation인수를대체하지않는다.

읽은harness SHA256: cpu_options_probe.py `feaabb04dcfb70bdb8ccd67afa85c8cde1a2f960f9b6bddf8f3ac7230a93b0ac`,cpu_plan_toggle_probe.py `69ae3eab6ac7810de2895df4ebb27a0f6aa8e619ad8c84ea3cbe4fd70a09d846`,ime_probe.py `b87d9fbc40e25ae53ef717c01ccfec20c884d825b2f06265f2e019695e177518`。Root는scriptowner이며이번작업은읽기만했다.

## CPU 최소 추가 위치와 엄격 assertion

현재cpu_options_probe.py87–95 help case loop를확장하거나별도 help-keyboard case로분리하면기존freshprivatefixture/lookup/wheel/sourcecatalogtail을재사용할수있다. 기존 pointercase원시증거를덮어쓰지않고새output에실행한다.

각header에서초기collapsed/EXPANDABLE bit9를실제확인한다. `Component.GrabFocus` exit0만으로성공하지말고다시GetState의FOCUSED bit12와actualwindowfocus/ownedapp PID-start를확인한다. Accessible state전체2개uint32를파싱해64bits로조합한다. Unknown/unavailable이면FAIL이며action성공을focus로대체하지않는다.

같은focusedheader에Return을down/up한번보내고boundedpoll로EXPANDED bit10=true+expectedtailtext/bounds/privatePNG를확인한다. 다시Return으로collapsed=false를확인한다. 새GrabFocus/FOCUSED를확인후space도open/close를반복한다. 닫힘은해당tail이tree에서제거되거나비가시semantic이며headerexpanded=false이어야한다. 다른header열림으로인한tail오인/같은표시이름중복을피하도록destination/path/exactlabel을매단계재검증한다.

기존cpu_plan_toggle_probe.py150이후키보드loop는GrabFocus reply와key후관측을기록하지만opened가false여도조건분기를지나최종passed=true가가능하다. 이를복사하면keyboard동작실패를PASS할수있다. 신규5header인수는반드시각keyopen==true/keyclose==false와FOCUSED를명시assert해야한다. 포인터reset은실패후cleanup수단일뿐키보드PASS의대체물이아니다.

Native key_sequence는명시DISPLAY에XTestFakeKeyEvent를보낸다. 오직해당harness가생성한Xvfb/ownedappwindow에만사용하고defaultDISPLAY/visible desktop으로fallback하지않는다. root의appPID/start/전용display/windowidentity검증이선행되어야한다. 기존focus_owned_window는제목검색을사용하므로privateXvfb내단일ownedapp이라는전제가필요하며userdesktop에서사용할계약이아니다. No clipboard/tool/network;job DB·worker미생성도매case확인한다.

## Scroll와 keyboard-only scope 구분

Lookup/wheel로header까지이동후GrabFocus+Enter/Space를보낸시험은**키보드activation**인수다. 처음부터마지막까지Tab만으로탐색한인수라고표현하지않는다. Headerfocus의scroll_to_me는actualbounds/footerclip/PNG로확인하고offscreen논리action성공을가시성PASS로대체하지않는다. 960x640에서tail은fullwidth/body검증과각stage스크롤뒤에확인한다.

추가Tab순서는별도CPU범위로URL→tool→output→FFmpeg→FFmpeghelp→cookie→cookiehelp→quality/range/cut/help→자막/advanced/Start등실제AccessKitfocusabletree순서를먼저기록해야한다. 단순하드코딩N번Tab로목표에도달했다고추정하지않는다. Max64 Tab/singlecase15초같은상한과focusednode를매step기록하고job/tool동작버튼은activate하지않는다.

## 기본 Iced 후속 범위

main새help는noninteractivetext이므로입력focus순서를추가하지않는다. URL/root/applyroot/defaultroot/tool/FFmpeg/quality/rangestart/rangeend/cutpick_list/자막/advanced/Start등기존focusablecontrols를privateXvfb에서기록하고Tab/ShiftTab뒤focusedcontrol/caret/bounds로원순서보존을인수한다. Height/FFmpeg/구간field는합성입력만사용하고readback/plan검증을종료조건으로쓴다. 도움말추가로Start/Plan이아래로밀리므로scroll와focus도달을별도로확인한다.

기본Iced는CPUAT-SPIprovider지원을자동가정할수없다. 실제focusednode/caret추출이불가능하면privatePNG+정확ownedwindow의입력readback으로지원범위를밝히고screenreaderPASS로표현하지않는다. native_probe/x11_diagnostics의기존pointer스크롤시험만으로Tab검증이라고하지않는다. Source/main/corepolicy는후속keyboardfixture에서수정하지않고실제제품버그가보이면root에돌려보낸다.

## 검증과 잔여

이번작업은scripts/result/source계약읽기및신규문서만작성했다. Rust/Cargo/기존scripts/GUI/keyboard실행없음. 현재R07help내용/포인터가시성범위는root의기존증거로인정하며5headerkey activation/Tab/reader실제발화·Windows/macOSkeyboard는잔여다. 신규fixture의source/binarySHA·정확ownedPID/start·privatePNG/rawstates·cleanup결과를보존해root가독립검토한다.
