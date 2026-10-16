# S10: Windows private clipboard station 사전 점검 감사

- 상태: done
- 분류: review
- 우선순위: P0
- 요구사항: R08, R11, R13
- 담당/수정 범위: metrics_recovery — 이 문서만; script/제품/Rust/Cargo/기존 evidence 읽기 전용
- 선행 작업: [station capability](S10-windows-clipboard-station-capability.md), [station 분석](S10-windows-clipboard-station-analysis.md)

## 실제 실패의 유형

[원시 결과](../validation/evidence/windows-clipboard-station/result.json)는 create_error=183, failure_stage=create station, CreateOnlyNULL Win32=183, passed=false다. station name/ACL/desktop/clipboard 필드는 없다. [실행한 helper source](../validation/evidence/windows-clipboard-station/helper.cs)의 반환 handle NULL 검사에서 실패했고 CreateWindowStationW(NULL,1,0x2002f,protected SID SA)를 호출했다. 따라서 nonzero handle 성공 뒤 남은 stale GetLastError를 실패로 오판한 사건이 아니라, **NULL handle과 즉시 capture한183의 실제 create-only 실패**다. desktop/clipboard 단계에는 도달하지 않았다.

NULL 이름은 임의 고유 이름이 아니다. Windows는 호출자의 logon session identifier로 이름을 구성하고, CWF_CREATE_ONLY는 해당 station이 이미 존재하면 실패한다. CWF_CREATE_ONLY를 제거하면 기존 station handle을 열 수 있으므로 이번 격리 요구에서는 제거하지 않는다. 이 API는 이름을 지정할 수 있는 호출자를 Administrators로 제한한다고 명시한다. 따라서 일반 사용자에서 GUID 이름만 넣으면 해결된다고 주장하지 않는다. [CreateWindowStationW 공식 계약](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowstationw)

기존 station이 정확히 무엇인지는 원시 결과만으로 알 수 없다. WinSta0라고 추정하거나 OpenWindowStation/clipboard/sequence 조회로 조사하지 않는다. 기존 handle을 재사용·ACL 변경·삭제하지 않는다. 원시 실패는 안전하게 중단한 capability gate이고 제품 copy 실패나 사용자 clipboard 오염의 증거가 아니다.

## 최소 수정 제안

script 담당자가 향후 preflight를 분리한다. Create 반환 handle의 zero/nonzero와 바로 캡처한 error를 별도 필드로 기록하고, 성공 여부는 반환값으로 판단한다. 성공 경로의 LastError는 API 계약이 별도 의미를 정의하지 않는 한 실패판단에 사용하지 않는다. CWF_CREATE_ONLY+nonzero+183처럼 계약과 불일치하는 결과는 명확한 unknown으로 격리하고 clipboard 단계로 진행하지 않는다.

비권한 current token의 enabled Administrators membership/elevation 상태를 읽기 전용으로 기록할 수 있다. 일반 사용자 NULL-name/create-only가183이면 현재 세션에서 새 station 생성 capability가Unavailable이라는 결과를 유지한다. 이름을 지정하는 생성은 문서상 권한 조건과 사용자 승인 범위를 먼저 확인해야 하며, 권한 상승·새 logon token·service 생성·ACL/보안 설정 변경은 이 preflight의 자동 fallback이 아니다. 승인된 이름 지정 실험이 가능하다면 `dlpad-owned-`+GUID와CWF_CREATE_ONLY를 함께 사용하고 충돌 시 최대 작은 고정 횟수만 새 이름으로 재시도한다. 기존 station open은 재시도가 아니다.

private station을 만들 수 없으면 Windows copy의 격리 검증만 미완료로 남긴다. 같은 WinSta0의 private desktop은 clipboard 분리 증거가 아니고 사용자 desktop/default station으로 fallback하지 않는다. Linux isolated selection PASS나 Windows private report-save GUI PASS를 Windows clipboard PASS로 확대하지 않는다.

## 반환 후 검증 및 권한

현재 stationRights0x2002f는READ_CONTROL+ENUMDESKTOPS+READATTRIBUTES+ACCESSCLIPBOARD+CREATEDESKTOP+ACCESSGLOBALATOMS다. 이 값은clipboards/create-desktop에 필요한 권한을 포함하고 EXITWINDOWS/SWITCHDESKTOP/WRITE_DAC/WRITE_OWNER를 추가하지 않는다. create-time protected current SID DACL/owner 및nonvisible UOI_FLAGS/name notWinSta0를 확인한 뒤에만 station association을 진행한다. default ACL의 문서 차이를 이유로 NULL security descriptor로 완화하지 않는다. [Window station 권한](https://learn.microsoft.com/en-us/windows/win32/winstation/window-station-security-and-access-rights)

SetProcessWindowStation은 process 전체의 station을 지정하고 이후 desktop/clipboard/atom 작업에 영향을 준다. 같은 session의 handle이어야 한다. 따라서 fresh bounded helper에서만 수행하고 concurrent GUI/clipboard thread가 있는 기존 collector process의 station을 바꾸지 않는다. 반환 BOOL과 immediate error를 기록하고 association 후 GetProcessWindowStation의exact handle/name/nonvisible를 다시 확인한다. 이 검사가 성공하기 전 OpenClipboard/EmptyClipboard/GetClipboardData/clipboard sequence 호출은0이다. [SetProcessWindowStation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setprocesswindowstation)

현재 desktopRights0x200c7는READ_CONTROL+READOBJECTS+CREATEWINDOW+CREATEMENU+ENUMERATE+WRITEOBJECTS를 포함한다. READ_CONTROL 요청에는READOBJECTS/WRITEOBJECTS도 필요한데 이미 충족한다. named desktop GUID는 current owned station 내부에서 만들고0 flags와protected current SID ACL을 유지한다. desktopName만 보지 말고 station/desktop 조합이 실제owned라는association을 검증한다. [CreateDesktopW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createdesktopw)

기존 helper는 station 생성 전에 GetProcessWindowStation/GetThreadDesktop borrowed handles를 저장했고 owner window를 만들기 전에 SetThreadDesktop을 호출한다. 정상 종료에서는 owner window/clipboard 작업을 끝내고 process station과threaddesktop association을 복구한 뒤 새 owned desktop/station handle만 닫아야 한다. 초기 borrowed handles는 닫지 않는다. 실제 create 실패에서는 owned station/desktop이 없어 restore BOOL만 기록됐고 사용자clipboard 호출0이었다. 복구/Close 실패는 숨기지 않고 helper 종료 후 잔여 소유 handle 여부를 별도로 기록한다.

## 후속 실험 인수 조건

- fresh bounded current-SID helper, create-only 유지, raw 반환 handle/error/name/ACL/nonvisible/association을 각각 기록한다.
- NULL-name183은 실패 그대로이며 old station 재사용0/ACL 수정0/default fallback0이다.
- 승인된 named 생성이 성공하는 환경만 독립owned station clipboard의 합성UTF-16 marker roundtrip을 수행한다. WinSta0/read-write/sequence 조회0이다.
- station/desktop association 확인 전 clipboard 함수0, partial create/desktop/association 실패는 소유handle만 정리한다.
- 실제 capability 성공 전 제품 copy 버튼/GUI collector를 실행하지 않는다. 현재 사용자작업/desktop/clipboard와기존8h시험은시험대상이아니다.
- source/executable SHA와 새 raw 결과를 고정하고 기존183 실패 evidence는 보존한다. 이름 충돌 실패를 대기 시간 증가나 권한 확대만으로 숨기지 않는다.

## 수행과 미검증

실제 수행은rg/cat/sed로기존source와raw를읽고Microsoft primary API문서를조회한것과이문서작성뿐이다. script/제품/Rust/Cargo/기존evidence를수정하지않았다. WindowsGUI/station생성/clipboard조회·읽기·쓰기·sequence조회/권한변경을실행하지않았다. named station capability는제안이며성공으로보고하지않는다.
