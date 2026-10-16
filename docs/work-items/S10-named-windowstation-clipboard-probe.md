# S10: named WindowStation 격리 clipboard capability

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R13, R08
- 담당/수정 범위: diagnostic_cache — scripts/windows_named_clipboard_probe.cs 및 이 문서
- 선행 작업: root native CLI host/Job/deadline/compiler; 기존 NULL station ERROR183 원시 보존

## 사용자 결과

GUID CREATE_ONLY 새 station/desktop에서만 합성 clipboard Set/Get/Clear capability를 시험한다. 사용자/WinSta0 clipboard 접근 및 visible fallback 금지. probe PASS는 제품 clipboard UI 완료가 아니다.

## 계약과 범위

caller-owned fresh evidence directory 하나. original process station/worker desktop handles는 borrowed이며 닫지 않는다. 소유 station 이름/handle·desktop이 확인된 뒤만 OpenClipboard. currentSID protectedACL 신규객체만. 실제 STA/nativeWndProc hiddenHWND·WM_CLOSE cleanup. fixedstructuredJSON<=64KiB/원OSerror·operation/원복과close결과. Rust/Cargo/기존runner 수정금지.

## 인수 조건

원inputdesktop 이름 read-only before/after 같음, owned syntheticUnicode roundtrip+clear, original station/threaddesktop 원복, 원HWND/desktop/stationclose 모두확인. 이름생성거절/권한/handle mismatch는 clipboard열지않고failclosed.

## 검증과 handoff

[CreateWindowStationW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowstationw), [SetThreadDesktop](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setthreaddesktop) 공식API확인. namedstation은 Administrators멤버 제한을 문서가명시하므로 actual권한실패면승격/우회없이실패증거로남긴다. Threaddesktop복귀는processstation먼저원복후수행. source READY/frozen. currentSID protected descriptor, GUID explicit CREATE_ONLY, nonvisible UOI_FLAGS, exact station/desktop names와 원 handles 검증, STA/native custom WndProc hiddenstyle0 HWND, synthetic Unicode roundtrip/clear, WM_CLOSE/class unregister, process station→thread desktop 복귀·close를 연결했다. 결과 Dictionary는 lock으로 bounded worker와 동기화하고 첫 원 OSfailure와 cleanup 별도 error를 보존한다. worker10초 초과 시 다른 thread가 사용 중인 owned handles를 닫지 않고 outer native Job cleanup 필요를 기록한다. borrowed original handles는 닫지 않는다.

C# compiler/native 실행은 root 담당으로 아직 수행하지 않았다. 원 inputdesktop name 읽기 실패도 clipboard에 접근하지 않고 failclosed로 기록한다. 능력 probe 성공을 actual 제품 보고서 복사 UI/사용자 clipboard 인수 완료로 표시하지 않는다.

## 통합 담당 실제 실행

2026-10-04 root가 Windows CLI host/private desktop에서 C# compile을 실행해 exit0을 확인했다. 실제 실행은 create_only_named_station에서 OS ERROR_ACCESS_DENIED(5)로 exit1이며 [원 결과](../validation/evidence/named-clipboard-capability-first/probe-result/result.json)에 기록했다. user_clipboard_accessed=false, visible_desktop_used=false, process_station_restored=true, input_desktop_unchanged=true 및 소유 자원 정리를 확인했다. 권한 상승이나 사용자 station fallback을 하지 않았다. 제품 복사 기능의 실제 Windows 검증은 미완료다.
