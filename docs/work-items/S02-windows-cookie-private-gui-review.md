# S02: Windows 인증 private GUI harness 읽기 검토와 진입 계약

- 상태: done
- 분류: review
- 우선순위: P0
- 요구사항: R04, R08, R10, R11
- 담당/수정 범위: soak_recovery_audit — 이 문서만; 후속 신규 harness/fixture 코드와 빌드·실행은 별도 담당 계약 필요
- 선행 작업: root Windows authenticated engine 실제4mode PASS, 최신 immutable CPU app/example READY와 별도 GUI 실행 지시

## 사용자 결과

사용자 화면·포커스·클립보드를 건드리지 않고 실제 Windows 네이티브 쿠키 입력과 완료/실패/보존 안내를 검증할 다음 실행 경로를 정한다. 이 작업은 읽기 검토 완료이며 GUI 검증 완료가 아니다.

## 계약과 확인된 근거

읽은 소스는 scripts/windows_background_probe.ps1, scripts/windows_native_probe.py, crates/app/src/accessible.rs/accessible_cpu_main.rs/catalog.rs, 새 합성 example, 현재 lock의 accesskit_windows0.29.2와 egui/egui-winit0.33.3이다. 기존 background script는 새 GUID desktop을 CreateDesktop으로 만들고 lpDesktop 지정 CreateProcess를 suspended로 시작해 kill-on-close Job Object에 넣은 뒤 resume한다. EnumDesktopWindows와 HWND PID 검증, 실제 process handle/start FILETIME, input desktop 이름 before/after, PrintWindow, owned job/desktop 종료는 재사용할 수 있는 설계 근거다. **실행 바이너리 고정 SHA와 fixed pointer 좌표/--downloads/C# synthetic worker는 과거 화면 계약**이므로 최신 CPU 인증 harness에 그대로 실행하지 않는다. 기존 script는 수정하지 않는다.

현재 AccessKit Windows provider에는 WM_GETOBJECT 활성화, UIA_Edit/Button 매핑, InvokePattern, ValuePattern과 ScrollItem/TextPattern 구현이 있다. egui-winit은 요청을 Event::AccessKitActionRequest로 넘긴다. 앱 field는 label을 labelled_by로 연결한다. 이는 실제 private desktop에서 UIA가 작동했다는 증거는 아니다.

중요한 입력 경계: 현재 egui TextEdit 소스는 text selection과 focus/scroll을 처리하지만 text edit용 Action::SetValue 처리는 발견하지 못했다. AccessKit ValuePattern.SetValue의 반환 성공만으로 입력 성공을 가정하지 않는다. 첫 capability 시험에서 값을 읽어 실제 반영 여부를 확인한다. 미반영이면 아래 owned-window PostMessage 입력 경로를 명시적으로 구분하며 제품 변경이나 사용자 화면 fallback으로 우회하지 않는다.

Microsoft primary 문서는 [HWND에서 UIA element 얻기](https://learn.microsoft.com/en-us/dotnet/api/system.windows.automation.automationelement.fromhandle?view=netframework-4.8.1), [작은 application subtree 검색](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-obtainingelements), [창을 소유하지 않는 별도 MTA thread](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-threading)를 설명한다. [SetThreadDesktop](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setthreaddesktop)은 현재 window station의 desktop을 thread에 연결하며 기존 window/hook이 있으면 실패할 수 있다. 이 자료에서 private desktop UIA 성공을 추론해 완료로 쓰지 않는다. 실제 capability 인수는 후속 시험이다.

## 후속 신규 harness의 구체 진입점

신규 scripts/windows_session_cookie_gui_probe.ps1를 별도 소유권으로 만들 수 있다. 기존 hidden windows-cli-host가 PowerShell을 argv -File로 실행하게 하고, controller는 고유 dlpad-cookie GUID desktop과 kill-on-close owned job을 생성한다. 새 CPU app를 suspended+lpDesktop으로 시작하고 assign/resume한다. 공개 desktop SwitchDesktop/SendInput/SendKeys/SetForegroundWindow/전역 cursor 또는 clipboard API는 사용하지 않는다. Input desktop은 query-only before/after로 비교한다.

UIA controller는 새 window/hook 없는 MTA thread에서 SetThreadDesktop(owned HDESK)을 **UIA 초기화 전** 실행한다. UIAutomationClient/UIAutomationTypes managed assemblies 또는 native UIAutomationCore COM만 이용하며 새 외부 의존성은 필요 없다. Thread/COM apartment는 전체 시험 동안 유지하고 원래 thread desktop 복귀와 owned handles 정리를 finally에서 수행한다. capability 실패는 그 GUI 검증만 미인수로 기록한다.

EnumDesktopWindows로 얻은 실제 owned HWND를 GetWindowThreadProcessId+primary process handle/start FILETIME으로 검증한 뒤 AutomationElement.FromHandle(HWND)를 root로 사용한다. Desktop RootElement/FromPoint/사용자 앱 전체 검색은 하지 않는다. Root descendants는 최대256node/JSON64KiB/시간10초로 제한하고 각 action 전에 HWND/PID/start를 재검증한다. Name+ControlType으로 다음 catalog를 찾는다:

| 컨트롤 | 이름과 동작 |
|---|---|
| URL Edit | 영상 URL; ValuePattern 또는 TextPattern으로 실제 값 확인 |
| Cookie Edit | 이번 작업의 Netscape 쿠키 파일 (선택); source path만 입력하고 시작 뒤 실제 빈값 확인 |
| Start Button | 다운로드 시작; enabled과 InvokePattern으로 실행 |
| Cancel Button | 작업 취소; InvokePattern으로 실제 Cancelled 인수 |
| Report header/preview | 실패 원인 보기 / 진단 자료 미리보기; Invoke와 TextPattern.DocumentRange.GetText로 정제 자료 확인 |
| Pending notice | 이전 쿠키 복사본의 종료·소유권 확인이 남아 안전하게 보존했습니다. 새 작업은 새 쿠키 선택으로 실행합니다. |

ValuePattern.SetValue가 실제 text edit에 반영되지 않으면 UIA ScrollItem.ScrollIntoView/현재 BoundingRectangle로 위치를 찾고 ScreenToClient(owned HWND) 변환 후 같은 HWND에만 PostMessage mouse-down/up·WM_CHAR를 보낸다. Global input/clipboard를 쓰지 않는다. 반환값으로 입력 성공을 판단하지 않고 UIA 실제 텍스트와 호출 기록을 다시 확인한다. 기존 Linux stale bounds 문제처럼 반환 좌표만 믿지 않으며 fresh frame 후 bounds를 재조회한다. 이 경우 UIA setter PASS라고 표현하지 않는다. Diagnostic preview는 Edit와 label의 동일 이름을 ControlType으로 구분하고 최대64KiB만 읽는다. Copy report 버튼은 누르지 않는다.

## 합성 도구와 앱 경로의 호환 계약

이미 native engine4mode를 통과한 example은 current_exe parent가 OS temp 바로 아래 `dlpad-windows-cookie-download-owned-*`이며 fixture.cap/source.cookies/mode가 고정 합성값이어야 한다. 각 GUI case마다 이런 임시 root에 immutable example을 복사한다. Source bytes는 정확히 해당 example의 windows-download-fixture-only Netscape 고정값이다. Mode success/metadata-failure/download-failure/cancel을 그대로 사용한다.

현재 example의 private copy prefix=root/private-cache 제약을 제품 수정 없이 만족시키려면 **DLPAD_TEST_DATA=root/private-cache**, 실제 앱 session cache=root/private-cache/session-cookies로 둔다. DLPAD_YTDLP=root/synthetic-ytdlp.exe, DLPAD_LIBRARY_ROOT=root/media 한글, OS config/cache는 별도 owned temp 하위로 지정한다. 이렇게 하면 실제 UI가 선택한 세션 copy가 tool의 허용 prefix 아래이며 journal과 cookies cache가 제품 규칙대로 분리된다. 프로젝트/사용자 설정·Documents는 사용하지 않는다. Parent PowerShell의 TEMP/TMP를 root로 바꾸면 example의 OS-temp-parent 검증을 깨뜨리므로 **원래 OS temp를 유지**하고 case root만 그 아래 생성한다.

## 인수 순서와 추가 fixture 결정

1. Capability: 최신 immutable app/example SHA before/after, owned HWND/Job/start identity, private desktop에서 URL/Cookie Edit와 Start Invoke/Value/Text 가시성을 확인한다. Controller deadline180초, app readiness15초, 각 operation10초. Hard bound를 넘는 UIA COM 호출은 controller process를 owned outer job/hidden CLI host deadline으로 정리하며 thread abort나 사용자 앱 kill을 하지 않는다.
2. Success: 실제 source path 입력→시작→Cookie Edit empty→Downloaded/1Attempt/artifact bytes+size+SHA, version no cookies와 metadata/download 동일 literal SHA/path, source/sentinel 보존, copy directory 제거, raw SQLite backup/reopen/integrity/privacy를 보존한다.
3. Failure: metadata/download failure 또는 malformed source를 구분한다. FailureStage/고정 omission preview/no secret/source/copy paths, Failed 또는 no job, 정상 cleanup/source sentinel를 확인한다. Malformed는 이 fixture tool을 시작할 필요가 없다.
4. Pending restart: 실제 Windows prepare_session_cache+import_session+defer_cleanup로 만든 **private native copy**가 필요하다. Linux fixture/record 또는 임의 folder를 actual import 증거로 쓰지 않는다. 후속에 별도 ignored Windows seed driver 또는 기존 example의 명시 dev-only seed-deferred argv를 추가하는 소유 계약이 필요하다. 아직 새 seed 코드는 작성하지 않았다. Seed는 root/private-cache/session-cookies의 실제 copy를 보존하고 raw credential을 출력하지 않는다. App startup의 pending label+start enabled+source field empty/no auto-start를 인수한 뒤 새 쿠키를 선택해 새 copy로 Downloaded, 새 copy만 정리되고 이전 copy identity/ACL/bytes hash가 그대로인지 확인한다. 종료·동일 data 재실행에서도 이전 copy 보존/1Attempt·artifact 불변을 확인한다. Windows startup은 아직 자동 recovery/deletion이 아니므로 pending을 cleanup 완료로 쓰지 않는다.

Windows worker 소유권은 outer owned Job의 process list와 query/synchronize process handles에서 PID/start/exe identity를 확인한다. 기존 Linux /proc/PID 신호를 적용하지 않는다. 성공의 짧은 worker를 live 관측하지 못하면 live Job ownership 인수는 별도 cancel-mode의 worker-ready 30초 창에서 수행한다. Exact handle 종료와 whole owned Job 정리만 사용하고 stale PID를 추측하지 않는다. 다른 owned sentinel process/file은 그대로 유지한 뒤 자신의 handles로 정리한다.

## 검증과 handoff

이 문서 외 파일·Rust/Cargo/기존 script를 수정하지 않았고 Windows GUI/clipboard 실행도 하지 않았다. Root가 보고한 [native engine4mode evidence](../validation/evidence/windows-authenticated-download-first/root-review.json)는 GUI 인수와 구분한다. 현재 준비된 것은 신규 harness 구현 계약이며 actual UIA private-desktop capability, 최신 GUI 성공/실패/pending restart는 미검증이다. 후속 실행 전에 root의 정확한 app/example snapshot과 seed 코드 담당 범위를 고정한다. UIA 실패가 있어도 독립 engine/backend 개발은 계속할 수 있다.
