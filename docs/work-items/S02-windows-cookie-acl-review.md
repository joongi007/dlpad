# S02: Windows session cookie ACL 구현 검토

- 상태: done
- 분류: review
- 우선순위: P0
- 요구사항: R04, R07, R08, R11, R13
- 담당/수정 범위: metrics_recovery — 이 문서만; requirements/INTEGRATIONS/cookies/process/windows.rs 및 cached windows-sys 읽기 전용
- 선행 작업: root가 private Windows cache·공유 cookie parser/lease interface 조정; Linux/Windows 전체 입력 동결 검증 후 배정

## 사용자 결과와 현재 범위

선택한 Netscape 원본을 변경하지 않고 현재 사용자만 접근하는 Windows private copy로 실행할 수 있게 하는 계약이다. 현재 cookies.rs는 cfg(not(unix)) import/validate/cleanup을 Unavailable로 처리한다. Windows process backend의 suspended spawn→Job Object assign→resume 및 handle 소유성은 존재하지만 cookie ACL/file identity 인수를 대신하지 않는다. 기존 Windows 자동시험 통과를 cookie 지원 통과로 표현하지 않는다.

읽은 요구사항은 docs/REQUIREMENTS.md의 R04 extractor 실행, R08 취소/정리/복구, R11 OS별 실제 시험과 INTEGRATIONS의 세션 cookie·비밀 배제다. GUI나 system-wide 보안 정책을 변경하는 작업이 아니다. 일반 관리자/백업 권한으로 ACL을 우회할 수 없는 비밀 저장소를 약속하지 않는다.

## 현재 binding와 비용

cached windows-sys0.61.2에서 OpenProcessToken은 System::Threading, GetTokenInformation/TokenUser/SECURITY_ATTRIBUTES/보호 DACL/SID 함수는 Security, SetEntriesInAclW/GetSecurityInfo는 Security::Authorization, CreateDirectoryW/CreateFileW/FILE_ID_INFO/FileAttributeTagInfo/SetFileInformationByHandle/LockFileEx는 Storage::FileSystem에 있다. LockFileEx의 OVERLAPPED gating인 Win32_System_IO도 engine Cargo에 이미 있다. Cryptography도 이미 있어 nonce는 BCryptGenRandom 후보를 사용할 수 있다. 새 Rust dependency/Cargo feature 추가는 우선 필요 없어 보이나 실제 WindowsGNU compile/link로 확인해야 한다.

Win32 Advapi32/Kernel32/Bcrypt 호출과 몇 개 OwnedHandle, bounded SID/ACL buffer를 재사용한다. 별도 resident process/thread/GUI가 필요 없다. copy read 최대1MiB+1/line16KiB/entries4096, sessions128 상한을 유지한다. token/securitydescriptor 조회는 예시64KiB 상한, path는 UTF16길이/최대 component64 상한을 둔다. 실제 exe 증가/RSS/I/O 시간은 구현 후 동일 toolchain baseline과 측정하며 지금 수치를 invent하지 않는다.

## 사용자 SID와 생성 시 보호 ACL

OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY)→bounded GetTokenInformation(TokenUser), IsValidSid/GetLengthSid/CopySid로 현재 primary token SID를 독립 buffer에 고정한다. impersonation token이 있거나 실행 child의 primary user와 불일치하면 검증된 지원 전에 fail-closed한다. username 문자열·account lookup에 의존하지 않는다.

SetEntriesInAclW에 OldAcl=NULL과 정확한 사용자 SID의 단일 allow ACE를 주어 새 ACL을 만든다. owner는 같은 SID, DACL present/non-null, SE_DACL_PROTECTED, 상속 ACE 없음으로 SECURITY_DESCRIPTOR를 구성하고 SECURITY_ATTRIBUTES(bInheritHandle=false)를 CreateDirectoryW/CreateFileW 생성 시 전달한다. plain inherited ACL로 생성한 뒤 chmod처럼 수리하면 cookie 노출 창이 생기므로 금지한다. 권한 설정 실패나 이미 존재한 nonce leaf는 reuse/덮어쓰기하지 않는다. 반환 ACL buffer는 LocalFree RAII로 관리한다. [Microsoft SetEntriesInAclW](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-setentriesinaclw).

생성한 handle의 GetSecurityInfo(SE_FILE_OBJECT, OWNER_SECURITY_INFORMATION|DACL_SECURITY_INFORMATION)로 owner SID와 DACL을 재확인한다. READ_CONTROL 접근을 확보하고 GetSecurityDescriptorControl의 protected bit, GetSecurityDescriptorDacl present/non-null, ACE count/type/flags/target SID/권한을 정확 검증한다. Everyone/Users/AuthenticatedUsers/다른 SID·inherited/unknown ACE, null DACL은 private cache로 거절한다. GetEffectiveRights가 owner에게 권한이 있다고 말하는 것만으로 user-only인지는 판단하지 않는다. GetSecurityInfo 자체의 ACL 변경 race는 끝난 객체에 대한 원자 보증이 아니므로 실행 직전·삭제 직전 재검증을 유지한다. [Microsoft GetSecurityInfo](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-getsecurityinfo).

## path·reparse·소유 file identity

초기 지원은 local fixed volume의 검증한 ACL-capable filesystem(예: 실제 NTFS fixture)으로 한정한다. UNC/SMB/device namespace/ADS/상대경로/drive-relative/NUL/예약 device leaf는 거절한다. Unicode·공백·정상 long path는 typed UTF16로 처리하며 shell을 사용하지 않는다. 사용자 Documents나 OneDrive 전체 ACL을 바꾸지 않고 앱 private cache leaf만 생성한다.

CreateFileW는 source와 directory를 FILE_FLAG_OPEN_REPARSE_POINT, directory는 FILE_FLAG_BACKUP_SEMANTICS로 연다. FileAttributeTagInfo의 FILE_ATTRIBUTE_REPARSE_POINT가 있으면 종류와 관계없이 거절한다. 마지막 leaf flag 하나가 모든 intermediate component를 안전하게 만들었다고 주장하지 않는다. bounded component walk로 각 상위 directory를 먼저 열어 no-reparse 검증하고, rename/replace를 막는 share-delete 제외 handle을 유지하면서 다음 component를 연다. reparse 변경까지 차단할 share-write 정책과 실제 junction race를 Windows에서 검증해야 한다. canonical string/최종path equality만으로 부모 경합을 해결하지 않는다. Win32만으로 목표 인수를 만족하지 못하면 rooted NT handle-relative open을 별도 검토하거나 Windows import를 계속 Unavailable로 둔다. [Microsoft CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew).

cache/owner/file handle마다 FileIdInfo의 VolumeSerialNumber+128bit FileId를 고정하고 FileStandardInfo의 directory/size/numberOfLinks를 확인한다. cookie는 regular disk file/link1, owner/cache는 검증한 directory다. file ID만으로 다음 시작 재사용 inode를 오인하지 않도록 nonce/marker 및 별도 recovery 계약이 필요하다. 지원 filesystem에서 이 정보가 없거나 identity가 바뀌면 fail-closed한다. [Microsoft FILE_ID_INFO](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_id_info).

## 원본 읽기·copy·tool lifetime·정확 삭제

원본은 OPEN_EXISTING read-only/no-reparse와 제한된 공유 모드로 읽고 length/time/ID 전후를 확인한다. 원본 ACL·mtime·내용·이름을 변경하지 않으며 실패한 read도 source를 삭제하지 않는다. 현재 Unix 내부에 있는 pure Netscape validate와 SensitiveBytes/zeroization을 root가 공통 helper로 이동해 동일 bounds/parser를 재사용해야 한다. parser를 새 Windows module에 복사하여 다른 의미로 갈라지지 않는다.

copy는 OS nonce의 create-new owner와 CREATE_NEW file, explicit protected ACL에서만 쓰고 flush한다. retained file handle은 tool read/write를 허용할 FILE_SHARE_READ|FILE_SHARE_WRITE와 share-delete 제외 정책을 검토한다. yt-dlp가 cookie contents를 업데이트하는 것은 허용할 수 있지만 rename/identity replacement는 허용하지 않는다. 실제 공식 tool save 동작이 share-delete 제외와 충돌하면 명시 실패로 처리하고 넓은 자동 fallback을 하지 않는다. LockFileEx를 cookie bytes에 exclusive로 걸어 tool의 정상 read를 막는 방식은 금지다. optional owner.marker lock은 동시 import/cleanup 조정에만 사용하며 자손 종료 증거가 아니다.

lease OwnedHandle은 metadata/download child 및 Job Object 종료 확인을 포함한 worker 수명 동안 유지한다. validate_for_tool은 file/cache/owner handle identity, reparse/type/owner/DACL/link count를 직전에 확인한다. `--cookies`만 execution argv로 전달하고 version/options/snapshot/DB에는 넣지 않는 기존 adapter를 재사용한다. raw auth logs/metadata 자유 텍스트의 생략을 유지한다.

정상 cleanup은 retained DELETE-capable owned handle의 FileDispositionInfo로 delete-on-close를 설정한다. 이 API는 DELETE 권한이 필요하며 삭제는 CloseHandle 뒤 완료된다. path 문자열로 DeleteFileW를 호출하여 교체된 파일을 삭제하지 않는다. child/tool handle이 닫힌 뒤 수행하고 실패하면 고정 오류·보존으로 끝낸다. owner directory도 exact identity와 예상한 own 항목만 있고 비어 있는 경우에만 handle 기반 정리를 인수한다. 원본·cache root·extra sentinel·다른 file은 삭제하지 않는다. [Microsoft SetFileInformationByHandle](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-setfileinformationbyhandle).

FILE_FLAG_DELETE_ON_CLOSE를 import 즉시 지정하면 app crash/cleanup-unconfirmed 때도 copy가 조기에 없어질 수 있다. explicit 정상 종료 시점까지 delete disposition을 설정하지 않는다. POSIX/ignore-readonly disposition flags로 외부 readonly 데이터를 강제삭제하는 우회도 하지 않는다. 바뀐 security/identity·hardlink·reparse·extra 파일이면 cleanup을 거절하고 모두 보존한다. same-SID 악성 프로세스나 관리자가 ACL을 위조하는 위협을 완전히 막는다는 보장은 범위 밖이다.

## Windows startup recovery 미지원 경계

Linux /proc bootUUID를 Windows record에 쓰거나 GetTickCount64 uptime·wallclock 차이·사용자 logon을 reboot signature로 추정하지 않는다. Fast Startup/hibernate/suspend/reboot 간 의미와 신뢰한 boot identity API가 실제 검증되기 전 Windows recover_sessions는 Unavailable로 유지한다. cached public bindings에서 Linux와 동등한 검증된 durable reboot UUID를 확인하지 못했다. NtQuerySystemInformation의 내부 boot struct를 추측해 같은boot/이전boot 삭제를 허용하지 않는다.

Windows Job Object의 살아있는 owned handle과 ActiveProcesses=0 같은 종료 확인은 현재 session의 정상 정리 근거로 실제 시험할 수 있다. process creation time/PID 파일만으로 다음 시작 detached reader가 없다고 추정하지 않는다. crash/deferred copies의 durable record/128 capacity/secret-free owner proof가 없는 import 구현은 startup 복구 요구사항 전체 완료가 아니며 release gate로 명시해야 한다. 초기 세션 import 지원과 recovery Unavailable 범위를 native UI가 설명하도록 root catalog에서 정한다.

## 미래 파일 분업

독립 backend 담당은 신규 crates/engine/src/cookies/windows.rs의 token/ACL/path/handle lifetime/cleanup만 소유한다. root는 cookies.rs의 cfg backend enum·shared parser/zeroized buffers·SessionCookieLease interface/privatecache native UI를 단독 조정한다. 시험 담당은 신규 tests/windows_session_cookies.rs 및 합성 native driver만 소유한다. Cargo/lock/shared schema 변경이 필요해도 root가 조정한다. 현재 동결 source는 이 검토에서 바꾸지 않는다.

## 실제 Windows 인수 fixture

- WindowsGNU compile/test/clippy와 동일 actual Windows binary/input hash 전후, temporary source/cache만 사용.
- 생성 직후 owner SID/current token 일치, protected/non-null DACL·정확1ACE, inherited Everyone ACL parent에서도 copy가 broad ACE를 물려받지 않음. 다른 SID를 만드는 실제 계정 생성 없이 ACE 구조 판정/negative fixtures로 확인.
- source/cache/owner/copy 각각 reparse/junction/hardlink/rename replacement·DACL broaden/owner change·extra sentinel에서 import/validate/cleanup 거절과 외부 bytes 보존. symlink 권한이 없으면 그 case를 미검증으로 기록하고 관리자 설정을 바꾸지 않음.
- Unicode/공백/long path, bounded format/size/line/entry, concurrent source write/share conflict, FD/handle baseline, create-new collision/partial-write failure.
- fake tool metadata/download cookie 동일·version 제외, genuine success/failure/cancel/timeout/Job Object kill 뒤 정확 copy 정리; unconfirmed 종료는 preserve. tool cookie-save가 identity 유지함을 actual synthetic integration으로 확인.
- full128/foreign entry 보존, crash/defer recovery는 Unavailable 안내와 잔여 gate이며 reboot가 검증됐다고 주장하지 않음.
- native GUI는 root의 windows_native_probe를 통한 private desktop에서만 실행하며 user desktop/clipboard/actual account cookie를 사용하지 않음.

## 검증과 handoff

실제 수행은 rg/sed cached bindings·제품 읽기와 Microsoft primary docs 확인, 이 전용 문서 작성뿐이다. source/Cargo/ACL/system settings/Windows GUI는 변경·실행하지 않았다. root가 초기 filesystem/path 지원 범위, shared parser/API, normal cleanup/Windows recovery release gate를 선택한 뒤 구현·actual OS 시험을 배정해야 한다.
