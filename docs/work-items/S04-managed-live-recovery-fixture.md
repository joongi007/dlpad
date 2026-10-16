# S04: Windows live creator lease startup 회수 fixture

- 상태: staged; publication-native-validation-pending
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; 신규 examples/windows_managed_live_recovery_fixture.rs와 본 문서만
- 선행 작업: root public managed recovery/native API, metrics private observer/stdin contract

## 사용자 결과

살아 있는 creator의 원 ExecutionLease가 startup 회수를 차단하고 명시 release 후 pristine finish로만 원 pending을 제거하는지 확인한다.

## 계약과 범위

CLI managed-live namespace/dlpad-managed-live-recovery-nonce/owned-fixture. fresh CAP/sentinel/synthetic source와 공개 begin_execution. stdout bounded schema1 ready/finished + GetProcessTimes parentcreation. stdin ASCII R 1byte/max90초; EOF/timeout는 실패/pending 보존. journal은 원 byte-range lock 때문에 metadata identity/size·실제 read denial만 기록하고 SHA를 만들지 않는다. Rust /tmp staging, root 게시 GO 전 프로젝트 source 미추가. shared source/Cargo/실제 도구 helper/PIDguesskill 제외.

## 인수 조건

live public recover recoveredfalse/pendingtrue 및 marker+8receipt/CAP/source/sentinel SHA 불변. 원 journal read ERROR_LOCK_VIOLATION33/metadata 불변. release 후 public finish/원 files 제거 및 source/sentinel 보존, reader thread join/exit0. creator lease Busy 범위이며 살아 있는 tool/helper Busy 인수가 아니다.

## 검증과 handoff

root/metrics와 CLI/schema/R 계약 합의. protected_files absolute string[]와 별도 path/bytes/SHA evidence, removed_files와 journal_path를 제공한다. native build/private GUI 실행은 root 담당, 아직 미검증.

완성 staging `/tmp/dlpad-windows-managed-live-recovery-fixture.rs` SHA256 `b230d29c9586ecbeaecdf19aa987dda571309c2809c0e22f5c76ab61a5446d90`. protected12files는 marker+8receipts+CAP/synthetic source/foreign sentinel; 원 journal은 public NativeFileIdentity의 Windows volume_serial/file_id128와 size 및 실제 payload read ERROR_LOCK_VIOLATION33를 기록한다. journal payload hash는 측정하지 않았다고 명시한다. self public recover false/pendingtrue와 before/after SHA·journal metadata 일치를 확인한 뒤 readiness를 발신한다. 성공 release 전에 같은 pins를 다시 확인하고 public finish→marker/journal/8receipts 부재→source/CAP/sentinel bytes 보존을 요구한다. 성공 stdin reader thread는 join한다; EOF/잘못된 byte/90초 timeout은 pending 보존 실패로 반환한다.

staging rustfmt 실제 exit0. 프로젝트 Rust 미게시, Cargo/native GUI 미실행. root publication GO 후 신규 파일만 게시 예정이며 actual Busy/Windowsread33/GUI Retry는 미검증이다. 실제 live 도구/helper guardian scope로 확대하지 않는다.

### 실제 최초 실패 후 읽기 권한 수정 staging

root가 최초 source를 게시/build/observer compile 성공한 뒤 actual GUI mlg1은 handshake 이전 ERROR_ACCESS_DENIED5로 실패했다. 원시 실패를 보존했으며 observer 원 Job은 자연 Active0이었다. 원인은 fixture가 metadata-only public `open_regular_no_reparse` handle을 payload SHA/read probe에 사용한 것임을 root가 확인했다. 이는 원 journal lock33 관측이나 GUI 회수 성공이 아니었다.

수정 staging `/tmp/dlpad-windows-managed-live-recovery-read-access-fix.rs`, SHA256 `dd5dd69fac030e0ad750d76c3019b8125e4aacd8fd09fc75495ffd86228325a9`. fixture 전용 normal FILE_GENERIC_READ/shareALL/no-reparse open을 추가했다. root-contained path/ancestor reparse 거절, 원 root metadata handle과 public ensure_regular_under, FILE_STANDARD_INFO NumberOfLinks1/Directoryfalse/DeletePendingfalse를 검증한 뒤 bounded payload read를 한다. ACL/privilege/core 공유 함수는 변경하지 않는다. journal의 실제 read33 요구 및 hash 미측정 표기는 유지한다. staging rustfmt exit0, project 게시 GO·actual retry는 대기다.

Root reviewed the staged source and published its exact SHA after the preceding Windows checks terminated. Native build and GUI execution remain pending; source publication is not acceptance.

Root actual Windows build exit0 and C# observer compile exit0 in evidence/mlg1. First GUI run failed before ready: fixture attempted content reads from the public file_identity read-attributes-only handle, causing ERROR_ACCESS_DENIED5. Original managed files remain and observer confirms natural owned Job0/private desktop closed/input desktop unchanged. This is a fixture access-rights bug; actual live GUI acceptance is not complete. Corrected generic-read fixture handles are staged for review; core shared API is unchanged.

Root published reviewed read-access correction SHA dd5dd69fac030e0ad750d76c3019b8125e4aacd8fd09fc75495ffd86228325a9. Fresh mlg2 actual build exit0 and holder ready passed public live recovery pending plus actual journal lock33; GUI observer failed on .NET4 verbatim path normalization before UI acceptance. Original failure and naturally empty owned Job/input unchanged are preserved. Latest Windows GNU CPU fmt/clippy/test/release all exit0 in windows-managed-live-recovery-workspace; root-review confirms current source. GUI observer path fix and fresh execution remain pending.


## root 실제 Windows 통합 인수

첫 mlg1은 fixture 읽기 권한 오류, mlg2는 .NET4 verbatim 경로 처리 오류로 실패했고 원 실패를 보존했다. fixture의 실제 generic-read 권한과 observer의 엄격한 확장 드라이브 경로 정규화 후 [mlg3 원 결과](../validation/evidence/mlg3/result.json), [actual GUI](../validation/evidence/mlg3/g/native-result.json), [root 독립 대조](../validation/evidence/mlg3/root-review.json)는 PASS다. 원 creator HANDLE FILETIME/live 원 잠금33, 실제 GUI startup pending→Retry 원 SHA 유지→R 정상 finish/exit0→Retry done→원10이름 부재·foreign/source SHA 유지, 자연 Job0/private desktop 종료/input desktop 불변을 확인했다. root가 960x640 pending/done PNG를 직접 검토했다. 실제 tool/helper Busy나 CloseWait GUI 인수로 확대하지 않는다.

수정된 fixture 포함 최신 Windows fmt/clippy/test/release 네 명령은 [전체 검증](../validation/evidence/windows-managed-live-recovery-workspace/result.json)에서 exit0, [현재 source 대조](../validation/evidence/windows-managed-live-recovery-workspace/root-review.json) 확인. 새 의존성/Cargo/schema 변경 없음.
