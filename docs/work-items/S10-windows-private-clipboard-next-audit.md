# S10: Windows 격리 clipboard 다음 인수 감사

- 상태: done (읽기 감사만; 제품 복사 인수 미완료)
- 분류: validation
- 우선순위: P0
- 요구사항: R13, R08
- 담당/수정 범위: diagnostic_cache; 이 문서만
- 선행 작업: P0-status 최신 Windows 보고서·station capability 기록, ADR-0004, 기존 private native CLI/GUI host

## 사용자 결과

사용자 화면·clipboard를 건드리지 않고 검증할 수 있는 저장 인수와, 현재 host에서 막힌 실제 clipboard 인수를 구별한다. S10은 최초 Windows 릴리스 필수 게이트다.

## 계약과 범위

AGENTS의 README→PRODUCT→CONTEXT→R13/R08→ADR-0004 순서와 INTEGRATIONS/기존 작업 기록을 대조했다. Rust/Cargo/scripts 수정·native 실행·권한 변경 없음. named station probe와 NULL station probe는 실패 원시를 보존한다. 기존 station 채택·WinSta0 clipboard 접근·visible fallback·관리자/DeveloperMode 변경·새 계정 인증은 허용하지 않는다. ADR-0004는 Proposed이며 사용자 확정으로 표현하지 않는다.

## 현재 실제 근거

- `named-clipboard-capability-first/probe-result/result.json`: fresh GUID CWF_CREATE_ONLY가 ERROR_ACCESS_DENIED(5), passed=false. process_station_restored/input_desktop_unchanged=true, user_clipboard_accessed/visible_desktop_used=false. C# compile PASS는 native clipboard roundtrip PASS가 아니다.
- 기존 NULL CWF_CREATE_ONLY ERROR_ALREADY_EXISTS(183): 새 station 소유권을 입증하지 못했다. CREATE_ONLY를 빼고 기존 logon station을 여는 대안은 채택하지 않는다.
- [CreateWindowStationW 공식 계약](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowstationw)은 explicit name에 Administrators membership 제한을 명시한다. rights mask 축소만으로 named 생성 권한이 생긴다고 추정할 수 없다. 현재 ERROR5의 별도 token/Job 원인까지 확정한 것은 아니다.
- [Windows station 계약](https://learn.microsoft.com/en-us/windows/win32/winstation/window-stations)에 clipboard는 station 자원이며 desktop을 새로 만드는 것만으로 분리되지 않는다. 기존 CLI host는 CreateDesktopW만 수행하므로 사용자의 station clipboard를 시험용으로 사용할 수 없다.
- 최신 Windows 정상 960x640 보고서 GUI는 preview/readonly/1478B export/정제/덮어쓰기 거절과 정상 종료를 인수했다(`S10-report-focus-scroll.md`, `windows-cookie-gui-auth-report-readonly-scope-retry/root-review.json`). Clipboard는 이 결과에 포함되지 않는다.

## 실제 제품 경로와 다음 작업

`accessible_diagnostics.rs:237–240`의 복사 버튼은 전체 preview를 `ctx.copy_text`에 전달하고 `software_host.rs:228`은 egui-winit `handle_platform_output`을 호출한다. 저장은 같은 panel `Operation::Save`→worker `reports::save_report`다. 둘을 메모리 mock으로 바꾼 시험은 실제 Windows clipboard 인수가 아니다.

현재 권한에서 **새 noninteractive station 생성 성공 근거가 없으므로 실제 clipboard 최종 GUI 실행은 시작할 수 없다**. 안전하게 가능한 다음 독립 수직 작업은 기존 private desktop에서 최신 immutable 제품 binary의 Markdown/JSON 저장·로그 edit/omit·MPD attach/omit·정제·create-only 충돌·CloseWait를 합성 fixture로 재인수하는 collector다. 복사/Paste 버튼·keyboard shortcut을 실행하지 않고 임시 export 파일을 직접 비교한다. 수정 후보는 신규 `scripts/windows_s10_report_save_probe.ps1`/전용 작업문서이며 기존 private GUI host를 읽기 재사용한다. 제품 변경 없이 실행 가능한 범위다.

실제 복사 최종 인수의 선행 조건은 별도 격리 Windows 시험 환경에서 **새 station의 원 HANDLE/name/nonvisible/protected ACL 및 owned desktop capability가 성공**하는 것이다. 현재 machine의 권한 거절을 우회해 확보하지 않는다. 독립적으로 provision된 시험 환경이 제공되면 새 작업으로 launcher/collector를 구현한다. cloud/계정/관리자 설정이 필요한 provisioning은 별도 결정 사항이며 이번 감사는 이를 수행하지 않는다.

그 뒤 구현 후보는 새 native launcher: station CREATE_ONLY→원 ACL/name/nonvisible 확인→새 desktop→station/desktop에 연결된 원 child process 생성→제품 HWND/UIA identity 확인→실제 copy click→같은 원 station의 hidden owner HWND에서 bounded CF_UNICODETEXT readback→same report MD/JSON bytes와 secret omission 대조→owned clipboard만 clear→원 child Job0/원 handle close 및 input desktop 불변. noninteractive station의 제품 software renderer/UIA 동작은 별도 실제 capability gate다. SwitchDesktop/foreground 조작 없이 렌더링·UIA를 검증하지 못하면 미검증으로 멈춘다.

## 인수 조건

저장 collector는 새 파일 전체 MD/JSON와 preview가 일치하고 로그 수정 재정제·개별/전체 생략·MPD 원본 보존·unsafe/oversize 실패·기존 export sentinel 보존·job 상태 불변·종료 ack를 입증한다. 복사 collector는 원 station/desktop/child identity 검증 전 clipboard API를 한 번도 호출하지 않으며 mismatch·ERROR5·ERROR183·UIA 부재는 failclosed다. 결과/로그는 bounded, binary/source SHA 전후 일치와 실패 원시·owned cleanup을 기록한다. capability나 synthetic roundtrip만으로 제품 복사 GUI 또는 S10 전체 완료를 표시하지 않는다.

## 검증과 handoff

2026-10-04 실제 코드/원 JSON·작업 문서 읽기와 Microsoft primary 문서 조회만 수행했다. 새 native/GUI 시험 없음. 현재 안전 환경에서는 저장 수직 기능 인수를 진행할 수 있고 clipboard는 환경 capability 제공 전 보류하는 것이 구체적 결론이다. 전체 P0 완료 주장은 하지 않는다.
