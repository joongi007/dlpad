# S00: Windows 앱 200% 배율의 키보드 경로

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R06, R08 및 S00 접근성 인수
- 담당/수정 범위: metrics_recovery는 /tmp/dlpad-zoom-keyboard-* 새 observer/runner 초안; root는 이 문서, 새 scripts/windows_zoom_keyboard_private_probe.cs/.py 게시와 실제 검증. 기존 제품/shared/Cargo는 root만 담당한다.
- 선행 작업: 현재 first-paint feature 전체검증 및 root source/binaryfreeze. native 실행은 root serial 조정.

## 사용자 결과

앱 session 200% 배율에서도 보이는 포커스와 Tab/Enter/Space 경로로 주요 탭·설정에 접근하고 안전한 동작을 수행한다. 실제 OS192DPI나screen reader 완료를 의미하지 않는다.

## 계약과 범위

AGENTS/README/PRODUCT/CONTEXT/관련요구/ADR0001/DESIGN을 따른다. 원privateDesktop/PID/FILETIME/HWND/Job만 조작하며 globalSendInput/foreground/clipboard/System설정/사용자데이터변경은 금지한다. 빈 합성fixture에서 초기 UIA focus를한번만설정하고 이후 ownHWND native키보드메시지에 실제focus/행동을대조한다. zoom선택준비단계UIA와keyboard시험단계를구분한다. stdlibC#기존UIAutomation참조만쓰고 제품dependency추가없다.

## 인수 조건

실제200%sessionzoom 및 원nativeDPI값·clientgeometry 기록,10~64 boundedTab 이동/Enter 또는Space 실행과실제control/state/보이는포커스 PNG,필수controls접근·함정/clip 실패보존,원source/exe해시·자연exit0/Job0/desktop원복. OS실제DPI변경/physical키보드/실제reader를합성메시지성공으로대체하지않는다.

## 검증과 handoff

구현·native미실행. 읽기계획은 docs/agents/handoffs/20261005/accessibility-next.md에보관한다. runtime fixture 증거와 root독립검토후에만완료한다.

### 재개: 실제 포커스 잘림 수정

`windows-zoom-keyboard-first-20261005-resume`은 step23에서 고급 옵션 펼치기 버튼의 실제 focus가 client 바깥(top743)에 위치하여 FAIL했다. 원 PNG에서도 해당 버튼이 보이지 않았다. root 담당 파일에 `crates/app/src/accessible_options.rs`를 추가한다. 기존 도움말 header와 동일하게 focus 획득 시 scroll 요청을 보내도록 수정한다. 초기 실패 증거를 보존하고 같은 200% native 시험을 재실행한다. 단위시험만으로 OS 가시성 인수를 대체하지 않는다.

두 번째 실제 시험은 고급 옵션 버튼을 포함한 step23을 통과한 뒤, 설정 화면의 한글 입력 검증 필드(step36, top662)에서 동일한 잘림을 발견했다. root 담당 accessible.rs의 해당 필드에도 focus scroll 요청을 추가한다. 실패 증거는 windows-zoom-keyboard-focus-fixed-20261005-resume에 유지한다.

세 번째 실제 시험은 IME field step36을 통과하고 focus traversal 순환에서 화면 테마 ComboBox(step44, top -85)가 위로 잘림을 발견했다. root 담당 accessible_settings.rs의 테마·배율 ComboBox에도 동일한 focus scroll을 적용한다. 수집기 조건은 변경하지 않는다.

네 번째 시험은 64step 모두 client내 focus였지만 START 방문이 누락됐다. 추가 실제 UIA 관측에서 START는 enabled/keyboardfocusable/notOffscreen이었다. egui 생성순서(nav→footer→form)와 시험의 조기 설정 활성화가 footer를 제거한 coverage 결함이다. 초기 UIA focus를 원 다운로드 nav 한 번으로 설정하고 이후 실제 Tab으로 START/폼을 방문한 뒤 설정Enter→다운로드Space 전환을 검사하도록 수정한다. 64상한·START Tab방문·clipping 조건은 유지하며 초기 SetFocus만으로 START 접근을 성공 처리하지 않는다. 변경 파일은 기존 root담당 observer.cs뿐이다.

최종 nav coverage 수정의 actual native47steps PASS, START step5 실제Tab 접근·설정Enter35→다운로드Space46·모든focusclient내·자연exit0/Job0/privateclose/inputsame를root가원JSON/PNG와대조했다. 증거 windows-zoom-keyboard-navigation-coverage-fixed-20261005-resume/root-review.json. 초기4종실패는보존했다. root는START outline을확인했지만다른privatePrintWindow PNG만으로모든control의명확한focusoutline을확정하지않아visible-focus전체인수는열어둔다. Physicalkeyboard/OS192DPI/reader도미확인.

visibleoutline 후속 실험 담당root파일 scripts/windows_zoom_keyboard_active_private_probe.cs를추가한다. egui Response.has_focus는viewportfocused&&memoryfocus이며winit은is_active&&is_focused가모두true일때Focused를전달한다. 원private HWND에만WM_ACTIVATEAPP/WM_SETFOCUS 메시지로이조건의합성경로를검사한다. 원GlobalSetForeground/SendInput/desktop변경없음,actualOSforeground나physicalfocus증명아님. 기존observer와실험변형을별도보존하며원memoryfocus·모든clipping·Tab접근인수는유지한다.

합성viewport활성실험에서IME outline이보였으나advancedPNG는scroll애니메이션중footer에가려졌다. 기존client내조건만으로실제viewport가시성을인수할수없다. observer두개에원focusedbounds3회동일(최대8×100ms)대기와actualUIA Custom scroll조상의bounds내포검증을추가한다. 인수강화이며최대64step/90초worker/원process만유지한다. 이전47PASS는client내·키보드전환scope로만보존한다.

후속 원queue focus 실험(root 새파일 scripts/windows_zoom_keyboard_native_focus_probe.cs): helper/GUI 원TID·PID·FILETIME·HWND 및 원privateDesktop 이름 일치 후 AttachThreadInput→SetActiveWindow/SetFocus→GetFocus/GetActiveWindow/GetGUIThreadInfo로 로컬queue focus를검증하고 finally detach한다. Global SetForegroundWindow/SendInput/SwitchDesktop/clipboard0, 다른desktopattach실패시중단이다. callerForeground 조회는원collector desktop(outerhost private)범위이며userDesktopfocus의독립증명으로표현하지않는다. 실제native 실행후outlines를별도검토하며physical키보드/reader/OSzoom을대체하지않는다.

2026-10-05 native owner focus 첫 실측은 desktop 이름 조회에서 FAIL했다. 원 natural Job0/private desktop close/input desktop 불변을 확인했다. Win32 오류 보존 누락을 수정하고 created/caller/target 조회를 각각 bounded JSON으로 계측한다. 첫 실패 원시는 windows-zoom-native-owner-focus-20261005-resume에 보존하며 실제 focus 성공으로 집계하지 않는다.

계측 재시험 windows-zoom-native-desktop-instrumented-20261005-resume은 created/caller 동일private 이름을 확인했으나 target GetThreadDesktop이 handle0/ERROR_ACCESS_DENIED5로 실패했다. 권한·보안 설정을 변경하거나 거절된 API를 우회하지 않는다. native queue focus 및outline 인수는 미완료로 유지하고 독립적으로 가능한 개발을 계속한다. 원실패와 자연종료/desktop정리 증거를 보존했다.
