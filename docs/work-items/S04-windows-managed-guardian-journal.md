# S04: Windows 관리 실행 guardian의 원 receipt journal

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당: metrics_recovery
- 수정 파일: 신규 crates/engine/src/tools/windows_managed_guardian_journal.rs, 이 문서
- 의존성: root ManagedReaderRecord/SessionAnchors/v3 marker binding, 다른 담당 ManagedJournal 및 native runtime

## 공유 API 및 권한

ManagedGuardianJournal::acquire(root_path, original_intent, slot, nonce, actual_guardian_and_tool_creation, original_receipt_file) -> io::Result<Self>. complete(&mut self) -> io::Result<()>는 실제 호출자가 원 Job ActiveProcesses 0을 확인한 뒤에만 호출한다. root가 원 marker v3와 session/anchors/baseDigest의 바인딩을 launcher/startup에서 검증한다. 이 facade는 cookie 타입을 참조하지 않는다.

원 intent clone의 bind와 실제 디스크 Bound를 bounded 16KiB JSON으로 읽고 encoded exact 비교한다. root/marker/lock/journal/8receipt의 원 native file ID, non-reparse regular/type/nlink1/namespace 포함 관계를 재검증한다. marker/journal/receipt는 current SID protected private DACL이다. 기존 관리 root와 install lock의 ACL을 변경하거나 private ACL을 임의 요구하지 않으며 원 ID/kind/contained 계약을 유지한다.

journal은 FILE_GENERIC_READ로만 독립 open하고 nonblocking try_lock을 얻는다. 삭제나 journal 쓰기 권한은 없다. 선택 receipt만 원 write capability로 소유한다. metadata READ_ATTRIBUTES/READ_CONTROL은 허용하지만 FILE_READ_DATA/DELETE/ACL 변경 등 확대 권한은 허용하지 않는다. [Microsoft NtQueryObject 계약](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryobject)에 따라 이미 로드된 ntdll의 함수 주소를 얻고 고정56바이트 ObjectBasicInformation의 GrantedAccess를 확인한다. API/형태 실패는 fixed error로 보존한다. 새 Cargo feature/dependency/도구 설치는 없다.

complete는 원 Bound와 모든 원 파일을 재검증하고 원 receipt가 비었을 때만 exact256 encode/write/sync를 한 번 시도한다. 쓰기 시도 이후 독립 journal lock을 즉시 반환한다. 부분 write 실패를 재시도로 성공 receipt로 위장하지 않는다. drop은 handle close만 하며 unlink/rmdir/재부팅/PID 기반 권한은 없다. native filesystem syscall은 하드 실시간이 아니며 bounded parse와 nonblocking lock을 실제 Job/상위 deadline과 함께 사용한다.

## 인수 및 실패 시험

실제 Windows read-only journal의 independent exclusive lock 획득 및 Busy, complete 후 facade 생존 중 app reacquire, 원 receipt256 및 resolve를 시험한다. wrong creation/nonce, read-only/read-write payload capability 거절, 다른 native receipt ID, partial receipt 바이트 보존을 시험한다. 외부 합성 sentinel/원 marker는 보존한다. 해당 fixture는 native 파일 계약이며 실제 Job terminal/독립 helper/app crash 증거로 확대하지 않는다.

## 검증 상태

개별 rustfmt exit 0. Cargo/module/reexport 및 실제 native 시험은 root 통합 담당이며 현재 PASS를 주장하지 않는다.

### 실제 Windows 통합 검증

주 담당이 `python3 scripts/windows_current_workspace_probe.py .../windows-managed-lifecycle-lock-capability-retry --execute --seconds 1800`을 실행했다. fmt, 전체 workspace Clippy(-D warnings), workspace test, release build가 모두 통과했다. result.json 및 최종 소스 SHA를 현재 파일과 독립 대조했다. native journal 7개, 삭제권한 없는 guardian journal 3개, runtime 제어 pipe 2개 및 v3 실제 정리 후 재사용 시험이 통과했다. 사용자 input desktop은 그대로였다. 문서 검사는 587개 PASS다. 실제 관리 도구 실행 adapter 및 startup 자동 회수는 아직 연결되지 않았으며 이 시험을 실제 다운로드/앱 crash 전체 인수로 확대하지 않는다.
