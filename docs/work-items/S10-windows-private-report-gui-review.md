# S10: Windows private report GUI 읽기 검토

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R13, R10, R11
- 담당/수정 범위: soak_recovery_audit — scripts/windows_cookie_gui_probe.ps1, 이 문서
- 선행 작업: Windows private cookie GUI capability 및 root 실제 실패·취소 DB 인수

## 사용자 결과

고정 합성 인증 실패의 진단 preview와 새 파일 저장을 사용자 desktop/clipboard와 분리해 실제 검증할 후속 harness 계약을 정했다. 이 검토는 GUI 실행 또는 Windows clipboard 인수 완료가 아니다.

## 계약과 범위

README → PRODUCT → CONTEXT 및 기존 GUI workitem·source를 읽었다. `scripts/windows_cookie_gui_probe.ps1`, `accessible.rs`, `accessible_diagnostics.rs`, catalog, `reports.rs`, local AccessKit Windows 0.29.2/consumer 0.31.0 구현을 읽기만 했다. 제품/Cargo/스크립트 변경과 agent GUI 실행은 없다.

Root의 `windows-cookie-gui-auth-download-failure-key-envelope/terminal-uia.json`에는 실제 `실패 원인 보기` header가 있고 진단 본문은 아직 닫혀 있다. 기존 capability는 owned HWND의 UIA fields를 실제 확인했다. 이것만으로 report header 조작, preview readability, export 성공을 확인했다고 표현할 수 없다.

## 인수 조건과 구체 진입점

1. 동일 private desktop/MTA/owned Job/suspended-bind/정확 HWND·PID·start 계약으로 고정 download-failure를 실행한다. 실제 terminal DB Failed/Attempt 한 개와 toolcall metadata/download 동등 private copy·정리 proof를 root가 먼저 확인한다. 과거 failure 파일은 그대로 보존하고 fresh evidence를 쓴다.
2. owned HWND subtree에서 label `실패 원인 보기`인 유일 header를 찾는다. Catalog label과 역할을 함께 기록하고 label만으로 static text를 버튼으로 오인하지 않는다. `collapsing()`은 AccessKit expanded 상태를 붙인다. AccessKit consumer `is_invocable`은 expand/collapse 상태가 있는 control을 제외하고 Windows adapter source에는 ExpandCollapse provider가 없으므로 무조건 Invoke한다고 가정하면 안 된다. 실제 지원 pattern을 먼저 기록한다. 지원 pattern이 없으면 이전 입력과 같은 owned HWND의 현재 UIA bounds → ScreenToClient → client 내부 mouse down/up·paint만 허용한다. 전역 input/focus/desktop 전환은 없다.
3. 펼친 결과는 `진단 자료 미리보기` label의 실제 TextEdit 자손 출현과 readable text로 증명한다. Header 조작 API 반환만으로 통과시키지 않는다. 현재 panel은 readonly multiline TextEdit 한 페이지를 보여준다. ValuePattern/TextPattern 지원을 실제 읽고 read-only flag·값 byte bound와 SHA를 보존한다. Synthetic secret `windows-download-fixture-only`, original source path, metadata/download private copy path가 text에 없음을 literal 및 JSON escaping으로 검사한다. 인증 원시 로그 생략 안내가 존재해야 한다. Metadata 자유 필드 title의 secret도 검사한다. Empty preview는 실패다.
4. page 범위는 `미리보기 페이지`, `이전 페이지`, `다음 페이지` label로 실제 기록한다. 작은 고정 fixture가 1페이지이면 그 사실과 현재 전체 text를 보존한다. 여러 페이지이면 각각 bounded 읽기 또는 전체 새 파일 저장으로 확인한다. 현재 harness UIA text 상한64KiB와 제품 preview 상한2MiB가 다르므로 잘린 한 페이지를 전체 privacy 검증으로 쓰지 않는다.
5. `진단 자료 저장 경로` Edit를 exact owned evidence 하위의 아직 없는 `report.json`/`report.md`로 채우고 actual readback을 확인한다. 알려진 `JSON` 또는 `Markdown` control의 지원 pattern/selected 상태를 실제 확인한 뒤 `새 파일로 저장` Button Invoke를 실행한다. Native file dialog나 Documents 기본경로는 사용하지 않는다. Busy·close-wait가 없고 preview_ok/absolute valid_path일 때만 button enabled다.
6. Engine `save_report`는 write/create_new로 저장한다. 새 파일 출현·`선택한 새 파일로 저장했습니다.` UI label·size≤2MiB·SHA·JSON parse 또는 Markdown 내용을 실제 보존하고 root가 preview와 전체 export privacy를 독립 검토한다. 기존 owned sentinel을 대상으로 한 두 번째 저장은 고정 `진단 자료 작업을 완료하지 못했습니다. 입력 파일과 기존 저장 파일은 유지됩니다.`와 sentinel SHA 불변으로 overwrite 거부를 인수한다. 취소·진단 worker 정리·Job active0·input desktop 불변도 확인한다.

## Clipboard와 호스트 계약

`검토한 자료 복사`는 절대 누르지 않는다. Product는 그 button에서 `ctx.copy_text()`를 호출한다. 현재 private desktop은 같은 window station에 속하므로 clipboard 분리를 보장하지 않는다. Clipboard read/write/sequence 조회 모두 금지한다.

기존 [station 결과](../validation/evidence/windows-clipboard-station/result.json)는 create-only NULL station 생성에서 handle NULL/Win32 183 ERROR_ALREADY_EXISTS로 실패했고 `WinSta0_clipboard_accessed=false`, borrowed handle close=false, station/desktop 복귀=true였다. 이 실패를 clipboard roundtrip PASS로 바꾸거나 기존 station 재사용·ACL 변경·관리자 권한·사용자 station fallback으로 우회하지 않는다. [작업 기록](S10-windows-clipboard-station-capability.md)과 같은 gate를 유지한다.

Root 후속 호스트는 immutable CPU+tool SHA, fresh native output, OS temp fixed synthetic cap/source, outer hidden CLI timeout을 제공해야 한다. Report opening/preview/save는 현재 owned HWND/MTA host 위에 별도 후속 collector로 구현 가능하지만 실제 지원 pattern·readback·새 파일·privacy proof가 있어야 인수된다. 현재 template에 report 조작을 추가한 구현이나 실행은 없으며 별도 소유 파일 계약 후 진행한다. Windows clipboard copy actual gate는 독립 station capability가 해결되기 전 미검증이다.

## 검증과 handoff

실제 수행은 source/evidence 읽기 검토뿐이다. 현재 cookie download-failure/cancel 통과는 root의 DB/file 독립 결과이며 이번 report 검토의 runtime PASS가 아니다. 남은 작업은 실제 header 펼침·readonly preview·새 파일 저장·overwrite 거부 시험과 root 원시 자료 검토다. 사용자 desktop fallback은 없다.

## 후속 harness 구현

Root가 구현 scope를 승인한 후 기존 소유 script에 opt-in `-VerifyReport`를 구현했다. Switch는 metadata-failure/download-failure에만 허용하며 기존 기본 auth 흐름은 유지한다. Header·preview·export field·save button의 실제 supported patterns를 먼저 별도 evidence로 보존한다. Header가 Invoke를 지원하지 않으면 owned bounds click으로만 펼친 뒤 실제 preview Edit를 기다린다. Preview에는 actual ValuePattern.IsReadOnly와 nonempty text를 요구한다. 한 페이지 text와 새 Markdown export 전체 모두 고정 secret·source·toolcalls에 기록된 private cookie path 및 JSON escaped 형태의 부재와 `[authenticated tool output omitted]`를 확인한다.

Export field는 새 `root/report-export.md`로 actual readback을 확인한 뒤 save Invoke와 saved status/새 파일 byte bound≤2MiB/hash를 확인한다. 두 번째 입력은 기존 root/sentinel로 바꾸고 actual empty/readback을 확인하며 저장 실패 label과 sentinel hash 보존을 요구한다. 입력 중간 단계는 별도 report input evidence directory에 보존한다. Copy button·clipboard API·global input 호출은 없다. 기존 outer worker90초 deadline 및 owned Job/input desktop cleanup 계약은 유지한다.

Agent는 새 Windows 실행·Cargo·제품 source 변경을 하지 않았다. Root가 immutable binary/tool로 fresh output에 failure mode와 `-VerifyReport`를 추가해 실행한다. Actual unsupported header/readonly preview/pattern/scroll/input/save/privacy는 FAIL이며 root 재실행 전 Windows S10 인수 완료로 표시하지 않는다. Export DB/privacy 원시 자료의 독립 root 검토는 계속 필요하다.

첫 실제 `windows-cookie-gui-auth-report-export`는 header patterns Invoke=false/ScrollItem=false였고 `Report control outside owned client`로 FAIL했다. Source와 private screenshot을 대조하면 긴 download form 아래 header가 아직 가시 content로 scroll되지 않았다. 후속 수집기는 bounds 검증을 약화하지 않고, owned HWND subtree의 현재 가시 Edit 중심을 anchor로 삼아 해당 HWND에만 mouse move/WM_MOUSEWHEEL(-120)을 보내고 최대32회 대상 bounds를 재확인한다. 다운로드 시작 button의 실제 UIA top을 footer 경계로 사용해 footer 뒤의 가짜 visible target도 거절한다. Header와 export field 모두 이 경계를 통과해야 클릭/입력하며 가시 anchor가 없거나 scrolling으로 가시화되지 않으면 원시 시도 JSON/private screenshot과 FAIL을 남긴다. ClientToScreen은 wheel lParam의 screen 좌표 인코딩용이며 global input이나 desktop 전환을 호출하지 않는다. Root fresh retry 전 실제 scroll/preview/export 인수는 미검증이다.

실제 `windows-cookie-gui-auth-report-export-scroll`은 32회 모두 header screen top992/anchor 영상 URL client480,107로 불변이었고 가시화 FAIL이었다. Winit Windows WM_MOUSEWHEEL source는 wParam high16을 signed i16로 해석해120으로 나누므로 0xff880000은 정확히 -1 line이다. lParam은 owned ClientToScreen 좌표이며 winit wheel branch 자체는 이를 읽지 않고 별도 pointer move 상태를 사용한다. Egui-winit은 LineDelta를 MouseWheel Line event로 전달한다. Move에서 TrackMouseEvent(TME_LEAVE)를 등록하므로 private 창에서 대기 동안 pointer 상태가 사라지는 가능성은 있으나 실제 event trace 없이 원인으로 확정하지 않는다. 최신 시험은 move와 wheel 사이 paint/100ms 대기를 제거해 연속 enqueue하고 raw signed delta/예상line/screen좌표를 추가한다.

별도 `-ReportClientHeight 1700` opt-in 시험도 준비했다. VerifyReport와640~2048에서만 허용하며 exact owned HWND/start 검증 후 SetWindowPos(SWP_NOMOVE|SWP_NOZORDER|SWP_NOACTIVATE)로 그 창만 높인다. 실제 client 높이를 확인하고 원래 width/height와 `original_960x640_report_scroll_verified=false`를 기록한다. 시스템 해상도·사용자 창·desktop·focus를 바꾸지 않는다. 큰 소유 창에서 preview/export가 통과하더라도 제품960×640 scrolling 검증을 대체하지 않는다. Root는 정상 크기 재시험 또는 별도 fresh evidence의 opt-in 시험을 선택하고 원시 결과를 구분한다.

첫 tall 시험은 Add-Type C# CS0170 컴파일 실패로 앱을 실행하지 못했다. Resize `GetClientRect && GetWindowRect` 단락 평가 후 outer RECT를 사용하는 경로가 definite assignment를 만족하지 않았다. 후속 source는 client/outer를 명시적 `new Rect()`로 초기화하고 API 반환값 요구를 그대로 유지한다. 이는 수집기 compile 수정이며 tall GUI 통과가 아니다. Root의 최초 compile 원시는 보존하고 fresh retry를 실행한다.

`windows-cookie-gui-auth-report-export-tall-retry`는 실제 client1700을 확인했지만 mouse header 조작 후 preview가 출현하지 않아 FAIL했다. Winit WM_LBUTTONDOWN/UP source는 button state만 보내고 lParam 위치를 읽지 않는다. Egui-winit `on_mouse_button_input`은 pointer_pos가 None이면 button event를 보내지 않고 CursorLeft는 pointer_pos를 None으로 만든다. 실제 CursorLeft trace가 없으므로 이 조건이 이번 원인이라고 확정하지 않는다. 후속 source는 mouse 미반영 screenshot/pattern을 보존한 뒤 exact owned header UIA SetFocus → 실제 HasKeyboardFocus 확인 → 해당 HWND Space down/up을 한 번 추가로 시험한다. AccessKit Windows SetFocus는 Action::Focus만 요청하고 egui context는 focus된 clickable widget의 Space/Enter를 primary click으로 처리함을 source로 확인했다. Global focus·foreground·keyboard state API는 호출하지 않는다. Keyboard focus 미반영/preview 미출현은 그대로 FAIL이다. Tall 경로는960×640 scroll 실패를 대체하지 않는다.

실제 focus retry는 owned UIA Focus/Space로 preview를 펼쳤다. 다음 실패의 원시 patterns는 TextPattern=true/ValuePattern=true/IsReadOnly=false/keyboard_focusable=false였다. 따라서 이전 오류문구 `ValuePattern unsupported`는 사실을 정확히 설명하지 못했다. Egui interactive(false)는 편집·선택 불가지만 upstream AccessKit TextEdit builder는 role/text를 보내고 read_only flag를 붙이지 않는 것을 읽기 확인했다. 현재 수집기는 TextPattern DocumentRange.GetText를 우선해 실제 multiline 값을 읽고 TextPattern2 available 속성도 조회·기록한다. ValuePattern 존재 시 별도로 값 privacy를 확인하고 fixed `[synthetic-readonly-probe]` SetValue 후 Value/Text가 그대로인지 확인한다. 실제 keyboard focus 불가도 요구한다. Provider IsReadOnly=false는 경고로 그대로 보존하며 이 runtime 비편집 증거를 UIA readonly flag PASS로 바꾸지 않는다. Explicit 제품 read_only metadata 수정·새 immutable 인수는 root 담당 후속 결정이다. Actual 값이 바뀌거나 text 비가용/empty/overbound이면 FAIL이며 clipboard를 쓰지 않는다.

Root는 실제 IsReadOnly=false를 제품 접근성 버그로 판정하고 preview node에 명시 read_only metadata를 추가한다. 최종 수집기 계약은 actual ValuePattern.IsReadOnly=true 필수로 유지한다. 패턴 부재와 supported provider의 false를 정확히 다른 오류로 표시하며 TextPattern-only·비편집 probe 결과만으로 통과하지 않는다. 새 immutable READY 이후에만 root가 실제 readonly metadata와 TextPattern text를 함께 인수한다. 제품 파일은 root 단독 소유이며 수집기는 바꾸지 않았다.

Root 새 readonly-fixed binary의 첫 native 실행은 수집기 Add-Type CS0160 compileFAIL이었다. ElementNotEnabledException은 InvalidOperationException 하위 형식인데 일반 catch가 먼저 있어 도달 불가능했다. 후속 source는 하위 catch를 먼저 두는 순서만 수정했다. 앱은 실행되지 않았고 root 원시 compile 실패를 보존한다. 제품 readonly fix actual GUI 검증은 fresh retry 전 미검증이다.

다음 실제 compile은 CS0136으로 실패했다. 같은 VerifyDiagnostic 메서드의 nested preview before와 outer sentinel before가 C# local scope 규칙을 위반해 `previewBefore`/`sentinelBefore`로 이름을 구분했다. 전체 C# source의 local/foreach/lambda/catch scope와 out 변수 경로를 읽기 점검하고 brace balance, 기존 RECT 명시 초기화, 하위 exception catch 우선 순서를 정적으로 확인했다. 정적 점검은 Windows C# compile 성공을 대체하지 않는다. 이번 변경은 두 변수와 참조 이름뿐이며 API/인수 조건은 그대로다. Root가 보존한 실제 compile 실패는 앱 미실행 결과이며 새 actual compile/GUI는 fresh retry 대기다.

Root 새 readonly-fixed binary의 높이1700 시험은 실제 readonly preview·전체 export/privacy·DB/file·sentinel overwrite 거부를 독립 인수했다. 이 통과는 정상960×640 scroll 통과가 아니다. 후속 opt-in `-VerifyReportReachability`는 VerifyReport와 original size에서만 허용한다. Header owned UIA Focus/Space를 먼저 실행해 offscreen 논리 펼침 가능 여부를 기록하고, export writable Edit에 owned UIA Focus/End를 요청한다. 각 전후 actual bounds/client center/IsOffscreen/in-owned-content/focus 및 private screenshot을 보존한다. Export focus 후 path/header/preview 각각 실제 content 안에 있어야 visual reachability를 통과한다. 밖에 남아 있으면 논리 API 성공·preview text readability와 별개로 FAIL이며 제품960×640 scroll 미검증을 유지한다.

Source 근거: egui Context의 AccessKit Focus와 ScrollIntoView는 별도 action이며 latter만 scroll_target를 설정한다. CollapsingHeader는 focus gain에 scroll_to_me를 호출하지 않는다. 반면 writable TextEdit는 focused cursor rect에 scroll_to_rect를 요청하므로 export Focus/End의 실제 scroll 변화를 시험할 수 있다. 이 경로에서도 화면 밖 header가 남으면 제품 header의 focus scroll/ScrollIntoView advertisement 보강을 root가 검토해야 한다. Product source는 수정하지 않았고 agent native 실행은 없다. 기존 tall/default auth 동작은 유지한다.

Normal 최초 actual reachability는 export Focus/End 가시화 실패였다. Root는 제품 header/export gained_focus에서 scroll_to_me Center를 추가한다. Collector의 header/export/preview 동시 가시 요구는 nested300px 영역에 부적합하므로 순차 조건으로 교정했다: header Focus 직후 실제 content bounds와 screenshot → Space로 펼침 → readonly preview ScrollItem 실제 지원·전후 bounds/screenshot과 가시화 → export Focus/End 직후 실제 bounds/screenshot 가시화. Preview에 ScrollItem이 없으면 현재 이미 visible인 경우만 허용하고 outside text readability를 가시 PASS로 대체하지 않는다. Windows ScrollItem provider는 AccessKit Action::ScrollIntoView 지원이 있을 때만 노출되고 egui Context는 그 요청을 scroll_target으로 처리한다. Product metadata 광고는 root 담당이며 새 immutable READY 전 actual 인수는 미검증이다. 최종 reachability flag는 서로 다른 stage에서 증명한 true들을 결합하며 같은 시점 전체 가시를 요구하지 않는다.

실제 scroll-action 원시는 preview height905, screen top1242→507, IsOffscreen=false였고 center client824가640 화면 밖이라 기존 판정이 실패했다. Oversized readonly text 전체/중심 가시 요구는 과도하다. 다만 after private PNG에는 subtitle/form과 body 최하단 header만 보이고 실제 preview 글자는 보이지 않았다. AccessKit Windows GetVisibleRanges는 source에서 NULL을 반환하므로 API로 글자 가시를 증명할 수 없다. Source상 Action::ScrollIntoView는 egui scroll_target 요청으로 처리되지만 nested panel clip과 최종 repaint는 독립 검증해야 한다.

후속 collector는 readonly preview에만 actual IsOffscreen=false와 owned client/footer 영역의 교집합을 검사하고 그 교집합 private PrintWindow crop·정확한 bounds JSON을 보존한다. Mouse/입력 control의 기존 center/client 검증은 바꾸지 않는다. Crop 존재나 교집합 자체를 visible glyph PASS로 표현하지 않는다. `report_preview_actual_visible=false`, glyph/root-image-review pending을 남기고 기존 REVIEW_REQUIRED 흐름으로 export 후속 진단을 계속한다. Root가 실제 preview 일부 글자가 읽히는 raw image를 확인해야 별도 인수 기록에서 정상크기 가시성 완료가 된다. 이번 script의 final reachability flag는 그 검토 전 false이며 full-page 동시 가시를 요구하지 않는다. Actual screenshot에 여전히 header만 보이면 root 제품 nested scroll 보강이 필요하다.
