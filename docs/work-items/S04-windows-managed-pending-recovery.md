# S04: Windows managed pending recovery original authority

- 상태: proposed
- 분류: safety-contract
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: diagnostic_cache; 이 새 문서만
- 선행 작업: root execution marker v2, Windows guardian/Job/native cookie receipt foundation

## 사용자 결과와 현재 상태

최초 릴리스 Windows 범위에 따라 Windows의 앱 crash 후 managed pending 자동 회수부터 구현한다. Linux 서비스 receipt 구현은 보류한다. 현재 execution_marker.rs v2는 실제 create handles의 root_id/marker_id/session_nonce를 4096-byte bounded denyunknown JSON으로 기록한다. lock/journal/receipt/ACL binding이 없으므로 이 정보를 자동 삭제 권한으로 확대하지 않는다. v1/unknown/partial는 계속 보존한다. 이 문서는 구현 제안이며 자동 회수 완료가 아니다.

## 최소 Windows utility 범위

`cookies/windows.rs`에서 current_sid(:86), Descriptor::private/attributes(:130–158), acl(:159), open(:310)/kind(:344)/ntfs(:376), chain(:424)/check_pin(:436)가 현재 private이다. root의 도메인중립 `windows_owned` 또는 `file_identity::windows` 내부 helper로 최소 capability API를 노출할 수 있다:

- `current_user_sid()`와 protected current-SID descriptor create-only builder.
- `validate_private_acl(&File,sid)`는 owner/protectedDACL/허용ACE·bounded security bytes를 검증한다. 기존 전체managed root/사용자 부모 ACL을 임의변경하지 않는다.
- `open_owned_directory/read/delete` 대신 용도별 retained NativeAnchor 제공; 원FD, NativeFileIdentity, no-reparse/nlink1/NTFS+현재namespacechain을 재검증한다. 임의path재귀삭제 API는 노출하지 않는다.
- marker/journal/receipt 생성은 CREATE_NEW+원handle sync/원IDs이며 reopen은 persistedoriginalIDs와 실제File IDs를 비교한다. 새로연현재IDs를 original로 다시저장하지 않는다.

cookie-specific `Journal` 전체 공개는 피한다. `cookies/windows/journal.rs:84,171,241–432`의 original record/receipt anchoring·exclusive release/reacquire·actualreceipt reading을 도메인중립 저장 capability로 분리할 수 있으나 cookiecopy/cache SessionAnchors 자체는 관리 도구에 적용하지 않는다. 관리 domain SHA prefix와 새 typed binding은 root schema 조정이다.

## 최소 immutable chain와 쓰기 순서

새 버전 binding에는 v2 root/marker 원IDs 외 원execution-lock, journal 및 고정 boundedreceipt 원IDs, sessionnonce, SID, domain-separated immutableDigest를 포함한다. 원create markerhandle을 유지하면서 baseDigest(root/lock/nonce)→journal/receipt CREATE_NEW와actualIDs→finalmarker(originalallIDs+markerownID+digest) sync로 확장한다. partial 실패는 gate 보존한다. 현v2를 새권한으로 소급해석하지 않으며 readerreceipt 없는기록은 회수하지 않는다. 공유 file schema/버전은 root 담당이다.

## stage별 receipt 권한

| reader stage | 등록/실행/완료 조건 |
|---|---|
| yt-dlp/Deno/FFmpeg/ffprobe version probe | 관리 원component hash/nativepin 이후 Intent, 정확 원process creation identity Bound, 실제 helper Job전체cleanup/Terminal+EOF 뒤 receipt |
| metadata 조회 | 원선택tool 및 managedruntime scope등록; cookie readerreceipt는별도domain 유지; 종료 시 각각원증거 |
| download | 원typed snapshot managedtools/component를등록,자손FFmpeg 포함한원Jobempty증거후완료 |
| 직접 FFmpeg 처리 | 별도 ProcessSpec 실행이면새stage Intent/Bound/원receipt; yt-dlp 자손이면상위Job전체종료receipt범위이며개별PID 추측금지 |

한session의모든준비된Intent가actualmatchingreceipt로해결되어야자동회수한다. stage counter/slot 상한과 reuse nonce는명시한다. versionprobe 성공만으로 뒤metadata/download의terminal권한을대체하지않는다. pre-cancel/launch전 crash/Empty도startup에서no-reader를추측해삭제하지않는다. 정상live object의명시finish와startupreceipt authority는분리한다.

## process 적용 우선 파일/interface

1. `process/windows_guardian_config.rs`에 optional typed managed binding/receipt index/expected original IDs(비밀/path raw debug 없음)를 root가 추가.
2. `windows_helper_launch` original inherited handle allowlist에관리receipt 원writer+원lockcap 전달. `windows_guardian_entry.rs:89–132` validation/CLOEXEC-equivalent inherit clear 적용; child에게managementreceipt를implicitinherit하지않음.
3. `windows_guardian`/Job authority는 실제원Job empty 확인 뒤에만원receipt를쓴다. helper exit/kill요청/PID없음/이전boot만으로resolve하지않는다.
4. `windows_cookie_capture.rs:124–368`의 exactTerminal+EOF+reader.confirm_terminal 패턴을 관리reader 공용callback에 분리한다. parent originalreceiptwriter는helperlaunch직후drop하여EOF barrier 유지. 오류시 관리gate pending을유지한다.
5. root `ManagedExecutionContext::prepare_reader(stage)->ManagedReaderLaunch` / `bind(nativecreation)` / `confirm_terminal()` / `recover_pending(root)->{cleared,pending,rejected}` 최소 API. typedProcessSpec 연결이며TLS/globalenv authority 없음.

## 첫 Windows 수직 기능

새 private owned pending-session에 원SIDACL 보호된 marker/journal/receipt를생성하고 관리 합성tool1개만등록→실제nativeWindowshelper launch/Jobcleanup receipt→callerapp 강제종료→같은임시root의새앱startup가originalchain+allreceipts+exclusive원lock 확인 후 exactmarker만 회수한다. 실제GUI는pending안내/회수count와그후관리선택가능상태를검증한다. registeredWindows nativeprocess 경로 사용, Linux/mock결과로대체하지않는다. 다중stage는account구현전자동회수대상아님/gate보존.

## 인수 조건

앱create전/partial/Intent/Bound-live/terminalreceipt뒤/clear중killfixture; 실제원Jobdescendantalive동안회수거절; receipt뒤재실행만성공. foreign/nativeIDswap/wrongnonce/ACLwiden/nlink/reparse/oversize/truncated/v1/v2unknown/priorboot/noactualreceipt 보존. 현재SIDprotectedACL 없는managedroot는자동변경하지말고pending/error. 원package/source/외부sentinel보존. guard acquisition과exactDELETEhandle전원chain재검증. clear중failed는잔여proof보존하며무조건재귀정리없음.

## 검증과 handoff

현재소스읽기감사완료, Rust/schema/Cargo/기존문서변경없음. root에필요utility와process파일순서를전달했다. 실제nativeWindows crash/GUI/자동회수 구현 및 시험은 다음수직작업이며 아직미완료이다.
