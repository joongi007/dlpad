# Windows P0 복사의 clipboard 격리 조사

2026-10-03, R13/S10. 조사만완료했고station/desktop/clipboardAPI실행·GUI실험·제품수정은하지않았다. 현재WindowsCPUprobe는CreateDesktop 후lpDesktop에desktop이름만지정한다. 같은WinSta0안의desktop끼리는clipboard를분리하지못하므로사용자clipboard를읽거나쓰는copy시험은금지상태다.

별도station은유망한실험경로다. Microsoft는clipboard/atomtable을windowstation의객체로설명하며WinSta0만interactive이고나머지는noninteractive라고명시한다. 따라서ownednoninteractiveStation의clipboard는WinSta0의사용자clipboard와다른객체라는근거가있다. 이것은현재CPU앱/UIA/GDI가그station에서정상동작한다는실증은아니다. [Window Stations](https://learn.microsoft.com/en-us/windows/win32/winstation/window-stations)

## 생성·권한·process 범위

SetProcessWindowStation은callingprocess의stationassociation과그process의clipboard/desktop접근을변경한다. 별도ownedprobe process에만호출하고WSL·사용자앱·사용자station자체설정은변경하지않는다. 같은sessionstation만허용된다. [SetProcessWindowStation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setprocesswindowstation)

중요제약: CreateWindowStationW의명시stationname은Administrators그룹에만허용된다. 비특권후보는NULLname+ CWF_CREATE_ONLY이며generatedlogonsessionname을UOI_NAME으로얻는다. generatedstation이이미있으면실패기록후종료하고재사용·삭제·ACL변경하지않는다. 이경로의실제비특권성공은미실험이다. namedstation을만들기위한승격/새user/logonsession/서비스생성은이번실험범위외다. [CreateWindowStationW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowstationw)

같은CreateAPI문서는SECURITY_ATTRIBUTES=NULL일때allusers GENERIC_ALL이라고명시하지만일반security문서는defaultACL이creatortoken에서유래한다고설명한다. 이문서차이를추측으로완화하지않고create-time명시protectedcurrent-userDACL을필수로한다. station/desktop원래ACL자동수정은없다. ownedobject requestedrights후보는station ACCESSCLIPBOARD/ACCESSGLOBALATOMS/CREATEDESKTOP/READATTRIBUTES/ENUMDESKTOPS/ENUMERATE/READ_CONTROL,desktop READOBJECTS/WRITEOBJECTS/CREATEWINDOW/CREATEMENU/ENUMERATE/READ_CONTROL이다. EXITWINDOWS/SWITCHDESKTOP/JOURNAL/HOOKCONTROL/WRITE_OWNER/기존objectWRITE_DAC는요청하지않는다. 최소subset에서실제UIA/CPU실패시권한확대는새ownedobject에한정해근거를기록한다. [Window Station Security](https://learn.microsoft.com/en-us/windows/win32/winstation/window-station-security-and-access-rights), [Desktop Security](https://learn.microsoft.com/en-us/windows/win32/winstation/desktop-security-and-access-rights)

## child·UIA·정리 계약

STARTUPINFO.lpDesktop에generatedStationName\\ownedDesktopName전체이름을지정할수있다. child와UIAcollector는같은ownedstation/desktop에시작한다. station/desktophandle의무차별inheritance를피하고stdioallowlist만상속한다. child가첫USER/GDI호출때연결되는station규칙에는SetProcessWindowStation·parentinheritance·lpDesktop우선순위가있으므로parent/child각stationname/desktopname실제증거를별도확인해야한다. [STARTUPINFO](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/ns-processthreadsapi-startupinfow), [Process Connection](https://learn.microsoft.com/en-us/windows/win32/winstation/process-connection-to-a-window-station), [Thread Connection](https://learn.microsoft.com/en-us/windows/win32/winstation/thread-connection-to-a-desktop)

SetThreadDesktop은현재thread에window/hook이있으면실패한다. probe는COM/UIA/clipboardownerwindow초기화전에station/desktop을연결해야하며기존PowerShell/GUIthread를그대로switch하는설계보다새nativehelperprocess가안전한후보다. UIA의invoke/getvalue는inputinjection과달리headless사용가능하다는Microsoft자료가있지만noninteractive다른station의pinnedAccessKit provider까지보장하지않는다. 같은station의collector ElementFromHandle/이름기반InvokePattern을실험해야하며실패시사용자desktop·SendInput·SwitchDesktop fallback금지다. 실제reader/IME경험과분리한다. [SetThreadDesktop](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setthreaddesktop), [UI Automation headless/injection boundary](https://learn.microsoft.com/en-us/windows/apps/dev-tools/winapp-cli/ui-automation)

ownedJob의child/collector를deadline/cancel로회수하고해당station의모든clipboardownerwindow/COM/eventthread를종료한뒤probe의기존processstation/threadDesktopassociation을복원한다. GetProcessWindowStation/GetThreadDesktop의borrowedinitialhandles는close하지않는다. 현재process에assignedstation또는thread가사용중인Desktop은Close API가실패할수있으므로순서를기록하고handleclose실패증거도보존한다. 최종probeprocess종료가남은ownedhandle을회수하며모든childJob종료증거가필수다. [CloseWindowStation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-closewindowstation), [CloseDesktop](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-closedesktop)

## 후속실험 인수조건(아직미실행)

1. 제품SHA/source/runner와ownedhelpercodeSHA고정. bounded60초nativehelper,보호DACL,current-userToken,CREATE_ONLYstation성공을먼저증명한다. UOI_NAME이WinSta0가아니고UOI_FLAGS의VISIBLE이false임을확인하기전clipboard호출금지.
2. station이이미있거나ACL/name/flags확인이불확실하면failclosed. WinSta0clipboard사전내용/sequence/format도읽지않는다. 사용자clipboardunchanged를내용대조했다고허위주장하지않는다.
3. 생성한ownedstation에만syntheticmarker를쓰고동일stationhelper로roundtrip한다. 모든OpenClipboard직전에processstation증거를재확인한다. retry/deadline및GlobalSize UTF16최대약4MiB를확인하고CloseClipboard/GlobalUnlock소유권정책을지킨다. [OpenClipboard](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-openclipboard)
4. frozenCPU앱과same-stationUIAcollector를full lpDesktop으로실행해실제진단reportpreview를읽고namedCOPY_REPORT를Invoke한다. clipboardtext가그검토preview와같고PRIVATE/서명/토큰원본이없으며Markdown/JSON각copy를비교한다. backendAPI를직접호출해서제품copy버튼인수로대체하지않는다.
5. AccessKit/provider/GDI/clipboardunsupported를각단계오류로기록한다. childstation명확증거가불가능하면copy단계진행금지. 다른station/process/사용자foreground/inputdesktop은변경하지않는다. ownedjob의exit/cancel/failurecleanups와objectclose결과를남긴다.

결론: 공식문서상station별clipboard격리와비특권NULLname경로가존재하므로추가승격없이ownedcapability실험을검토할수있다. 명시namedstation은관리자제약으로현재범위외이며자동생성station재사용도허용하지않는다. noninteractiveCPU/UIA호환과실제WindowsP0copy gate는미검증이다.
