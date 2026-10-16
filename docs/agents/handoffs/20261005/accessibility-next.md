# Windows S00/R06 접근성 다음 인수 — 읽기 감사

상태: Proposed/실행 전. 프로젝트 수정·native 실행·사용자 설정 변경0. 기존 Windows UIA tree/Invoke/Value/Text/Bounds/PNG 성공은 실제 screen reader 사용성 PASS가 아니다. Linux Orca/AT-SPI 및 WINIT_X11_SCALE_FACTOR=2는 Windows reader/DPI를 대체하지 않는다. DESIGN.md77의 100/150/200%, 키보드만, 실제reader 제목·상태·버튼·알림상한 인수를 축소하지 않는다.

## 가장 작은 다음 private 수직 인수

**앱 session200% zoom에서 키보드 Tab/Enter/Space만으로 원 focused control과 실제 행동을 관측**한다. 새 `/tmp/windows_accessibility_zoom_keyboard_probe.cs`→root 게시시 `scripts/windows_accessibility_zoom_keyboard_probe.cs`, 새Python wrapper를 별도담당한다. 기존 settings privateobserver의 생성·원HANDLE/FILETIME/HWND·Job-beforeResume·boundedUIA·PNG·WM_CLOSE/natural0/Job0/input불변을 그대로 재사용한다. Dependency0/Cargo0. freshowneddata/media 합성빈library, noURL/noCookies/noactualvideo. 최초 UIA SetFocus는 측정 시작field에만 허용하고 이후 Tab/Space/Enter를 원HWND message로 전달하여 synthetic native keyboard transport임을명시한다. UIA SetFocus/Invoke로 모든step을대신하면 keyboard-only로쓰지않는다. 글로벌SendInput/GetAsyncKeyState변경/foreground/layout/clipboard0.

제품 accessible_settings.rs19의75/100/125/150/175/200 선택과136/249의ctx.set_zoom_factor로 앱zoom200%를 설정한다. 원 userOS DPI변경이 아니다. 설정200% 선택 후 다운로드/도구/파일목록/작업이력 탭과 중앙footer를 실제Tab로 접근하는 짧은 한cycle을 먼저인수한다. 원focusUIA HasKeyboardFocus/role/name/IsOffscreen/bounds,Tab전후PNG,Enter/Space활성후actualstate를검사한다. 시작버튼 actualdownload는빈URL이라실행거절되는안전실패만확인하거나 비작업toggle을사용한다. scroll이필요한field는 keyboard포커스이동이실제scroll을일으키는지관측; pointerwheel로대신하고keyboardPASS라고하지않는다. Shift+Tab은기존PostMessage만으로modifier실제state가전달되는지미확정이므로일방Tab최소cycle우선, modifier는원appthreadcapability별도검증.

10~64개의boundedTabstep 각focus를기록하고 원실제focusednode가핵심controls에 도달해야한다. UIA SetFocus 초기성공만으로tabtraversal완료아님. focus실종/함정/중복label/960x640clip은 FAIL과PNG보존. 앱zoom100→200→100후설정원persist와freshreopen을검증하고외부data/sentinel불변. editorcaret/IME는별도actual IMM결과 재사용 범위만유지한다.

## 실제 OS DPI200%는 별도 게이트

native original GetDpiForWindow/awareness/physicalclientrect를read-only 기록하여현재host DPI를확인한다. SetThreadDpiAwarenessContext는awareness만바꾸며192DPI환경을만드는API가아니다. observerthread context를바꿔windowOSDPI200%라고표현하면안된다. private desktop은 monitor DPI 설정을독립소유하지않는다. OS scaling/monitor전환·WM_DPICHANGED메시지모사는사용자설정변경없이actualOStransition이아님. 실제192DPI로구성된격리WindowsVM/별도테스트session이필요하다. 현재없는경우정직히미검증으로남기고visiblefallback0.

공식 GetDpiForWindow와 SetThreadDpiAwarenessContext API최소client는Windows10 version1607이다. [GetDpiForWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdpiforwindow), [SetThreadDpiAwarenessContext](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setthreaddpiawarenesscontext). 이는collector관측API요구이며dlpad의최소OS실행지원증명이아니다. API불가/조회0은Unavailable/FAIL raw보존하며96DPI를추정값으로대체하지않는다. 기존Windows실행build만으로Windows10 1607또는모든Windows버전을시험했다고하지않는다.

## actual reader feasibility

현재재사용가능한UIAutomation client는providersemantic/clientevent 검증이며reader가아니다. 별도C# boundedAutomationFocusChanged/TextChanged/PropertyChanged handler를원PID subtree에만등록하면실제focus·상태events/알림frequency를관측할수있다. rootownedobserver lifespan에만구독/해제하며다른앱event내용저장0. Progress과다알림은합성job로별도빈도상한시험이필요하다. 이런client를'무음reader'라고새로이름붙여도actualNVDA/Narrator인수가닫히지않는다.

실제NVDA portable을project/user전용설치해originalprivateDesktop process로띄우는후보는있지만,전역hook/foreground/audio·기존reader중복/profileroutes를실제로검증전안전하다확정할수없다. Narrator globaltoggle은사용자desktop/system설정을건드릴수있어실행하지않는다. root가ownedprivateDesktop-only reader지원/설정scope/동작증거를입증할수없으면actualreader를미검증으로남긴다. 실제reader필요시격리VM에서제목/상태/버튼읽기/Tab/과다진행률알림/음성또는reader의실제speechlog를인수한다. UIA tree접근만으로reader완료0.

## 남은 근거

기존 settings/theme observer는 주로 pointer/Invoke/SetFocus 및 F24-envelope-WM_CHAR이며 nativeTabtraversal과actual200%DPI를증명하지않는다. software_host는winit원scale_factor와eguioutputpixels_per_point로renderer를갱신하지만 source존재가DPItransition증거아님. 최종스택은ADR0001의Proposed 유지, Windows-only릴리스우선인수만추진한다. R06의SystemOS실시간theme변경은UI Light/Dark선택과분리하며사용자globaltheme변경허용으로해석하지않는다.
