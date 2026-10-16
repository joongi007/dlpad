# S04: Windows 실제 관리 실행 creator lease와 GUI 재확인

- 상태: completed (실제 live creator GUI 인수; helper Busy/CloseWait는 후속)
- 분류: test
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당: metrics_recovery
- 수정 파일: scripts/windows_managed_live_recovery_gui_observer.cs, 이 문서
- 선행 작업: soak_recovery_audit의 공개 ToolManager managed-live holder, 최신 CPU exe

## 계약과 범위

기존 시작 GUI 관측기를 새 파일로 재사용하고 원본은 수정하지 않는다. private desktop과 원 Job에 suspended holder/app을 연결한 후 실행한다. CLI는 CPU exe, holder exe, 새 output이다. wrapper dlpad-managed-live-recovery-<nonce>만 관측기가 만들고 owned-fixture는 holder가 만든다. holder managed-live <root>의 ready/finished JSON과 원 parent_creation을 대조한다. stdin R 한바이트로만 정상 해제하고 원 HANDLE exit0을 확인한다.

공개 begin_execution creator lease가 실제 살아 있는 상태의 startup pending→Retry pending·원 marker/8receipt SHA 보존, 정상finish 뒤 Retry done·원 관리 기록 부재·외부 sentinel 보존을 실제 GUI에서 검사한다. 원 journal은 잠겨 content SHA를 읽을 수 없어 metadata/잠금 거절로 별도 검사한다. 독립 tool/helper Busy·실제 다운로드·CloseWait 전체 흐름으로 확대하지 않는다.

## 인수 조건

원 Process HANDLE creation, 정확 stdin/stdout/stderr3 HANDLE_LIST, 소유 Job·private desktop·input desktop 불변, queued partial handshake 상한과 자연 Job Active0을 확인한다. Retry는 실제 UIA Invoke를 사용한다. 원 byte/hash·metadata 및 release 후 삭제 이름은 fixture와 합의한 원 경로만 확인한다. 실패 cleanup을 성공으로 승격하지 않고 원 증거를 보존한다. 추정PID/globalinput/clipboard/가시desktop fallback은 없다.

## 검증과 인계

현재 계약·구현 중이며 native 실행/Cargo는 주 담당 전용이다. 실제 Windows 결과를 받은 뒤 기록한다.

## 게시된 정확한 관측기 인터페이스

holder CLI는 `managed-live <ownedroot>`다. ready/finished phase는 managed-live-ready/managed-live-finished이며 schema1과 원 parent_creation을 포함한다. ready protected_files는 absolute string[], protected_file_evidence는 path/bytes/sha256 배열이다. marker+8receipt+fixture.cap+source+foreign sentinel의 실제 SHA를 관측기에서 원 ready 증거와 대조한다. removed_files는 marker+journal+8receipt 정확10개다.

journal_evidence의 native_identity.Windows.volume_serial/file_id16은 실제 FILE_ID_INFO와 GetFileInformationByHandleEx(FileIdInfo=18)로 대조한다. 원 크기/regular/nlink1과 실제 ReadFile ERROR_LOCK_VIOLATION33을 앱 시작 전 및 살아 있는 retry 이후 재확인한다. journal content SHA를 읽었다고 주장하지 않는다. receipt 또는 PID만으로 삭제 권한을 새로 만들지 않는다.

holder stdout/stderr 각각64KiB, ready20초/finish10초/원 exit5초, UIA 상태20초씩·worker90초를 적용한다. stdin은 새4096B 원 pipe의 유일 write 한바이트 R이며 global input이 아니다. 정상 holder exit0을 확인한 뒤 기존 앱 retry로 ready 전이를 확인한다. 긴 .NET 경로는248자 미만 제한으로 명시 실패하고 원 삭제 이름의 부재는 GetFileAttributesW ERROR2/3만 인정한다. File.Exists false를 긴 경로 부재 증거로 사용하지 않는다.

실제 creator lease Busy 시험 준비본이며 독립 tool/helper의 journal Busy와 CloseWait는 별도다. 원 private desktop/Job/creation 및 실행 파일 SHA 전후 대조를 기록한다. 현재 C# native 컴파일/GUI 실행 미실행이며 root가 실제 fixture와 검증한다.

첫 실제 root mlg1/mlg2 실패 원시는 보존한다. mlg2는 fixture ready까지 통과했지만 Rust 원 canonical \\?\D:\ 경로를 .NET4 Path.GetFullPath가 거절한 collector 결함이었다. 관측기는 정확 verbatim drive prefix만 drive-letter/colon/backslash 검사 후 제거하고 기존248자 상한·원 containment/no-reparse 검사를 적용한다. extended UNC는 미지원 오류를 유지한다. 원 removed 이름과 protected key의 비교도 양쪽 같은 정규화를 적용하여 삭제된 marker를 보호 파일로 오인하지 않게 했다. 제품/Rust 정책을 완화하지 않았으며 수정본 native 재검증은 root 대기다.


## root 실제 Windows 통합 인수

첫 mlg1은 fixture 읽기 권한 오류, mlg2는 .NET4 verbatim 경로 처리 오류로 실패했고 원 실패를 보존했다. fixture의 실제 generic-read 권한과 observer의 엄격한 확장 드라이브 경로 정규화 후 [mlg3 원 결과](../validation/evidence/mlg3/result.json), [actual GUI](../validation/evidence/mlg3/g/native-result.json), [root 독립 대조](../validation/evidence/mlg3/root-review.json)는 PASS다. 원 creator HANDLE FILETIME/live 원 잠금33, 실제 GUI startup pending→Retry 원 SHA 유지→R 정상 finish/exit0→Retry done→원10이름 부재·foreign/source SHA 유지, 자연 Job0/private desktop 종료/input desktop 불변을 확인했다. root가 960x640 pending/done PNG를 직접 검토했다. 실제 tool/helper Busy나 CloseWait GUI 인수로 확대하지 않는다.

수정된 fixture 포함 최신 Windows fmt/clippy/test/release 네 명령은 [전체 검증](../validation/evidence/windows-managed-live-recovery-workspace/result.json)에서 exit0, [현재 source 대조](../validation/evidence/windows-managed-live-recovery-workspace/root-review.json) 확인. 새 의존성/Cargo/schema 변경 없음.
