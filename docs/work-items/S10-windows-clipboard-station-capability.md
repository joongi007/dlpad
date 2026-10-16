# S10 Windows owned noninteractive clipboard station capability

- 상태: completed
- 분류: experiment
- 우선순위: P0
- 요구사항: R13, R11
- 담당/수정 범위: guardian_fault; 신규 scripts/windows_clipboard_station_probe.ps1 및 전용validation/evidence
- 선행 작업: Windows clipboard station analysis 및 investigation workitem 읽기완료

## 사용자 결과

비특권 새소유 nativehelper에서 새 noninteractive station의clipboard를사용자 WinSta0와분리할수있는지 합성roundtrip으로시험한다. 제품GUI는실행하지않는다.

## 계약과 범위

60초deadline/freshprotectedcurrentSID fixture/nativehelper process. CreateWindowStation(NULL,CWF_CREATE_ONLY,protectedcurrentSID DACL); name nonWinSta0/flags nonvisible/ownerACL exact 확인 전clipboardAPI금지. initial borrowedhandles close금지. station/Desktopassociation복원후 ownedhandlesclose. 기존station/ACL/globalinput/WinSta0clipboard조회·sequence조회도금지. 제품/Cargo/sharedharness불변.

## 인수 조건

create-onlysuccess 혹은 actualWin32failure; station/Desktop ACL/name/flags 검증; 소유stationclipboard syntheticUTF16roundtrip≤4MiB; error와cleanup/results보존. Capability성공후root보고만,제품CPU/UIA는새계약전금지.

## 검증과 handoff

actualboundedhelperexit/cleanup/JSON/sourcehash전용evidence; existingstation재사용이나권한승격fallback없음. 기존8h시험유지.

실제capability experiment 완료: CREATE_ONLY NULLname 실패 Win32ERROR_ALREADY_EXISTS183/handleNULL. stationname/ACL/desktop/clipboard단계진행없음, WinSta0clipboard호출0, initialborrowedhandleclose0. ownedhelperexit1및rawsource/executable/hash/result보존. 기존station재사용/삭제/ACL변경/승격우회없음. 제품WindowsS10copy는gate미검증으로남음.
