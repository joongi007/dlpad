# S02: Windows private session cookie backend

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: stress_recovery — 신규 crates/engine/src/cookies/windows.rs 및 이 문서; root — cookies.rs backend wiring, app cache 준비, Cargo/native 통합 검증
- 선행 작업: S02-windows-cookie-acl-review, S02-cookie-shared-parser

## 사용자 결과

Windows에서 사용자가 선택한 Netscape 쿠키를 현재 사용자 전용 private 복사본으로 도구에 전달하고, 확인된 reader 종료 후 소유 handle로만 정리한다.

## 계약과 범위

기존windows-sys feature를사용하며공유format helper를재사용한다. 새dependency/schema/thread없이boundedSID/ACL/path/token조회, source1MiB+1/line16KiB/entries4096/sessions128상한. cache/owner/file SID·보호된nonnull DACL·단일현재SID ACE, component별reparse거절과heldparent로소유권을검증한다. 원본읽기전후nativefileidentity/size/time확인, 신규owner/file명create-only, retainedDELETEhandlecleanup, extra/changed항목보존. 자동원본·외부cache삭제나보안설정변경없음. deferred는handle닫고copy보존, recovery는OS증거미확인으로Unavailable유지한다.

공유interface는 windows::import(source,cache)->Result<(OwnedCopy,PathBuf,usize,usize),CookieImportError>; OwnedCopy의 validate_for_tool/cleanup/defer_cleanup 메서드. cache생성은별도 prepare_cache(&Path)->Result<(),CookieImportError>로신규생성시에만privateACL설정, 기존cache는검증만한다. root가SessionCookieLease에이를연결한다. 구현중추가계약이필요하면root와합의한다.

## 인수 조건

실제WindowsGNU compile/test/clippy,syntheticoriginal/sentinel보존,privateACL/identity검증,unicode/space/size/format,hardlink/reparse/broadenedACL/extra항목거절,도구정상read/write와cancel·Job종료뒤정확cleanup,unconfirmed보존. 확인하지못한case/OS는미검증기록하고releasegate유지. root가actualOS/GUI검증하며소스완료보고만으로인수하지않는다.

## 검증과 handoff

Windowsbackend구현대기. agent는공유cookies.rs/Cargo/UI/tests를수정하지않고신규module과자체unit시험에만작업한다. root sourcefreeze 후wiring/nativecompile,검증실패는원시증거보존후수정한다.

## tool 일반 open 공유 호환성에 따른 root 계약 수정

DELETE access를가진copyhandle을실행중유지하면tool일반open이FILE_SHARE_DELETE를주지않아충돌할수있다. 초기copyhandle은DELETEaccess없이identity를pin하고FILE_SHARE_READ|WRITE|DELETE를사용한다. 모든parent/ownerheldchain은rename을막는다. 정상cleanup시reader종료후별도DELETEcapablehandle을OPEN_EXISTING/no-reparse로얻어retainedcopy와exactFileId/type/link/ACL을대조한뒤그handle에만disposition을적용한다. path기반DeleteFile은사용하지않는다. 검증전교체는거절하고검증후rename도disposition대상은정확한원filehandle이므로다른파일을삭제하지않는다. 실제nativePython/yt-dlpreadwrite및rename/failurefixture가필요하며악성sameSID경합을모두방지한다고주장하지않는다. 기존검토의DELETEaccessretention은이공식share계약대조로수정한제안이다.

## 공식 공유 규칙에 따른 계약 수정

root가 [CreateFileW 공식 share 계약](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)을 근거로 초기 retained copy에 DELETE access를 요구하지 않도록 승인했다. 초기 copy는 read/write와 FILE_SHARE_READ|WRITE|DELETE를 허용한다. cache/owner parent chain은 no-share-delete 핸들로 고정한다. 실제 terminal 확인 후 별도 DELETE handle을 OPEN_EXISTING/no-reparse로 열고 retained copy와 FileId/type/link/DACL을 비교한 뒤 [SetFileInformationByHandle](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-setfileinformationbyhandle)의 일반 FileDispositionInfo만 사용한다. path DeleteFile/readonly/POSIX override나 자동 약화 fallback은 없다. 같은SID의 동시 rename이 있더라도 delete handle은 검증한 원본 객체를 가리키며 foreign identity는 삭제하지 않는다. 실제 yt-dlp cookie read/write 호환성은 native fixture 인수가 필요하다.

## 작성 상태와 공개 interface

신규 windows.rs에 import(source,cache)->Result<(OwnedCopy,PathBuf,usize,usize),CookieImportError>, OwnedCopy validate_for_tool/cleanup/defer_cleanup 및 prepare_cache(&Path)를 작성했다. cache 신규 생성 시만 owner-currentSID/protected 단일ACE DACL을 적용하고 기존cache는 엄격 검증만 한다. bounded token/SD/ACL, UTF16 path32767/component64, fixed local NTFS, component별 non-reparse held chain, 1MiB+1 sourcebuffer/4096scratch, common format와 zeroization을 사용한다. copy source전후 identity/size/time을 확인하며 cache foreign entries도128 cap에 포함한다. startup recovery나 durable Windows boot ownership은 구현하지 않고 root Unavailable 정책을 유지한다. 부분 생성 실패의 private leaf 잔존도 release gate다.

cached windows-sys0.61.2의 실제 함수/구조체를 확인했다. FILE_STANDARD_INFO/FILE_DISPOSITION_INFO bool fields 및 CreateDirectory/CreateFile/GetSecurityInfo/BCryptGenRandom 기존feature binding을 사용한다. feature 밖 두 값 DRIVE_FIXED=3/FILE_PERSISTENT_ACLS=0x8은 local named constants로 제한해 Cargo feature를 추가하지 않는다. [GetSecurityInfo](https://learn.microsoft.com/en-us/windows/win32/api/aclapi/nf-aclapi-getsecurityinfo)의 protected/non-null/current-owner 구조 검사를 수행한다. 새 Cargo dependency/thread 없음. sourcefreeze 전 standalone rustfmt만 수행하며 native compile/test는 root wiring 뒤 검증 대기다. 신규 unit2는 namespace/device/ADS/traversal 및 reservedname·unicode 정책이고 실제 ACL/handles/tool 인수를 대신하지 않는다.

## root engine wiring

root가cfgWindows OwnedCopy와import/validate/cleanup/defer를공유SessionCookieLease에연결했다. prepare_session_cache는Windows신규cache생성/기존검증entry이며기존ACL을수정하지않는다. GUI입력지원과startupgate정책은실제native빌드후별도연결한다. Windows복구Unavailable은유지한다. backend/helper를연결했으므로공통parser의notUnix deadcode예외를notUnix+notWindows로좁혔다. 현재actualWindowscompile검증전이다.

root 담당 시험 추가: tests/session_cookies.rs의 기존 nonUnixUnavailable 조건을 미지원 OS에만 적용하고, Windows에는 상대 source를 SourceUnavailable로 cache 생성 전에 거절하는 시험을 추가했다. 이 실패 시험만으로 private ACL·복사 성공을 인수하지 않는다.

## root actual Windows compile

`python3 scripts/windows_native_probe.py clippy --seconds 900` exit0/9.07초. actual Windows GNU workspace all-target lint 및 private-desktop=true/inputdesktopunchanged를 확인했다. [입력 SHA 독립 대조](../validation/evidence/windows-cookie-backend-first/root-review.json)에서 before/after/current 일치를 확인했다. ACL/tool read-write/cleanup 실제 동작·GUI·회수·P0 완료를 뜻하지 않는다. 다음은 합성 fixture 시험이다.

## 쓰기 실패 시 소유 RAII 보강

root 검토를 반영해 create-new copy의 ACL/nativeidentity 확인 직후 OwnedCopy를 구성하고, retained copy를 통해 write_all/sync_all을 수행하도록 순서를 옮겼다. 이후 쓰기 실패는 Drop의 exact identity/ACL/link/type/extra 검증을 거쳐 확인한 copy와 빈 owner만 정리한다. 검증 실패의 foreign/changed 대상은 보존한다. initial create/security/identity 실패의 객체는 추정 삭제하지 않으며 해당 partial gate는 남는다. 이 보강은 Cargo/native 재검증 대기이고 source freeze 뒤 root가 실제 Windows fixture를 다시 실행한다. 앞선 root 보고의 fixture3 PASS는 이 순서 변경 이전 snapshot이며 hardlink fixture 실패는 별도 시험 담당이 수정 중이다.

## root actual Windows runtime 최초 결과

`python3 scripts/windows_native_probe.py test --seconds 900` session18167 exit101/31.12초. [원시 실패](../validation/evidence/windows-cookie-runtime-first/test.log)와inputs를보존했다. Windowscookiefixture3PASS/1FAIL: privateACL·ordinaryread/write·확인후cleanup, pending/defer보존, 기존broadcacheACL불변은통과했다. extra/hardlink/ACL통합시험은hardlink생성단계의Windows공유위반32로실패했으므로해당검출과뒤ACL분기를완료로쓰지않는다. 시험담당이정확한32거절/생성성공경계를구분해재검증한다. 별도writer실패의partialcopyRAII보강은backend담당이구현중이다.

## root 동결 Windows 재검증

[전체 actual Windows 결과](../validation/evidence/windows-cookie-runtime-stable/root-review.json) session3271 exit0/19.84초, before/after/current271입력SHA일치. cookiefixture4PASS와Jobterminal3PASS를확인했다. 중간실행은fixture최종수정으로wrapperexit3이며 [원시관측](../validation/evidence/windows-cookie-runtime-retry-observed/test-inputs.json)을보존했다. [명시nocapture결과](../validation/evidence/windows-cookie-runtime-nocapture/root-review.json) exit0/fixture4PASS에서hardlink생성은OS32공유거절이며link검출증명이아니다. 뒤broadenedACLvalidate/cleanup거절분기는실제실행했다. 실제productauthGUI,writer실패주입,Windowspipe유한drain,재시작회수는남은인수다.
