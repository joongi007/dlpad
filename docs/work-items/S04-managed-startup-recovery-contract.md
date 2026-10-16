# S04: Windows 관리 실행 startup 복구 후보 계약

- 상태: proposed
- 분류: safety
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: diagnostic_cache; 본 신규 문서만. 제품 Rust는 읽기 전용.
- 선행 작업: Windows v3 execution marker, native managed journal, original Job terminal receipts, paired guardian

## 사용자 결과

제안: 앱이 중단된 뒤 관리 실행의 실제 종료 증거가 모두 남아 있으면 원 소유 pending 파일만 회수한다. 증거가 불완전하거나 실행 중이면 관리 도구 변경을 계속 차단한다. 이 감사는 자동 회수를 구현하거나 UI에 활성화하지 않는다.

## 현재 코드와 권한

- `tools/execution_marker.rs`의 `MarkerBinding`은 bounded 4096-byte denyunknown JSON이다. v3은 원 생성 HANDLE에서 얻은 root/marker ID, session nonce 및 `ManagedBinding {anchors,digest}`를 갖는다. `create_journal()`이 먼저 v2의 정확한 canonical bytes에 `dlpad-managed-execution-base-v1\0` 도메인을 붙여 SHA256을 만들고 journal 생성 후 v3를 sync한다. `verify_journal()`은 v2로 되돌린 canonical serialization으로 base digest를 재검증한다. v2→v3 중단은 자동 회수 권한을 만들지 않는다.
- `tools/windows_managed_journal.rs::reopen()`은 원 root/lock/marker와 원 anchors를 받아 journal/8 receipt의 실제 native ID, strict current-SID protected ACL, 종류·namespace 및 journal lock을 확인한다. 현재 root/lock ACL은 private ACL로 요구하지 않는다. 기존 사용자 root 권한을 자동 변경해서는 안 된다.
- `all_receipts_match()`는 lock을 가진 상태에서 실제 원 receipt bytes를 읽는다. persisted Resolved, FILETIME/PID, 예전 boot, mtime 또는 ID만으로 종료를 추론하지 않는다. `cleanup_completed(self)`는 전원 DELETE HANDLE 검증 후 disposition하고 HANDLE을 닫아 9개 이름의 실제 부재를 확인한다. `cleanup_pristine(self)`는 created_here 원 live creator만 허용하므로 startup에서 사용할 수 없다.
- `execution_lease.rs::finish()`는 원 scope의 journal 완료 정리 후 marker.clear를 호출한다. 현재 marker에는 안전한 startup 후보 재open API가 없다. `clear(&self)` Windows 성공은 disposition 설정이며 consuming close/부재 확인 API가 아니다.
- `cookies/windows/recovery.rs::recover_one()`은 immutable marker→원 native chain→원 journal reopen→실제 receipts→원 OwnedCopy cleanup을 사용한다. pristine/Busy는 회수하지 않는다. 이 구조를 재사용하되 managed 루트에 cookie allowlist/권한을 그대로 이식하지 않는다.

## 제안하는 최소 인터페이스와 순서

root 담당 수정 파일은 `tools/execution_marker.rs`, `tools/execution_lease.rs` 및 필요 시 독립 `tools/windows_managed_recovery.rs`다. 공유 공개 summary/callsite는 통합 담당이 결정한다.

1. 내부 `ExecutionRecoveryCandidate::open(root) -> Result<Option<Self>, RecoveryFailure>`: 원 root no-reparse directory HANDLE 및 marker read-only 원 HANDLE을 잡고 bounded read, strict ACL/native IDs/nlink/namespace를 검증한다. marker 없음만 None이다. v3 외/unknown/부분 JSON은 보존하고 Pending 또는 Rejected를 보고한다. root/marker anchors가 outer actual IDs와 일치하고 nonce가 nonzero인지 확인한다. canonical v2 base digest를 재구성한다. 이 시점 DELETE 권한을 부여하지 않는다.
2. candidate의 원 `.install-transfer-lock`을 **no-create**로 열고 expected lock ID·regular/no-reparse·root namespace를 확인한 뒤 `try_lock_exclusive`한다. 정상 install gate는 pending 자체를 거절하므로 이를 우회하는 전용 내부 recovery capability가 필요하다. 전역 flag로 gate를 해제하지 않는다. Busy는 보존한다. 같은 잠금을 marker 삭제/부재 확인까지 보유한다.
3. `candidate.acquire_original_marker_delete()`는 exact original marker ID/bytes/ACL을 다시 대조하며 DELETE capability를 늦게 얻는다. 기존 marker writer가 FILE_SHARE_READ만 허용하므로 live creator와 충돌하는 sharing violation은 Busy로 분류한다. 새 HANDLE을 현재 이름의 ID를 원 ID로 채택하는 데 사용하지 않는다.
4. 원 anchors/nonce/base digest 및 보유 root/marker/lock을 `ManagedJournal::reopen()`에 제공한다. root/lock path를 별도로 original identity에 묶는다. Busy, pristine, Intent, incomplete Bound, receipt 불일치/oversize/foreign replacement는 모두 보존한다. 마지막 unused Empty slots는 원 receipt가 실제 empty인 경우에만 허용된다.
5. `candidate.verify_journal(record)` 재검증→`journal.all_receipts_match()`→`journal.cleanup_completed()`→`candidate.clear_completed(self)` 순서다. 마지막 API는 원 marker DELETE HANDLE만 disposition한 뒤 닫고 이름 부재를 확인한다. journal 부분 삭제/close 실패 시 marker를 유지한다. orphaned marker만 남은 partial cleanup은 원 receipt 증거가 없어 재시도 회수 권한이 없다.
6. 결과는 bounded `ManagedRecoverySummary {recovered: bool, pending: bool, rejected: bool}` 또는 동등한 typed status로 보고한다. 내부 권한 객체를 UI에 넘기지 않는다. 첫 실제 native 인수 전 `accessible.rs` startup worker 및 관리 도구 Refresh에서 자동 호출하지 않는다. 활성화 시 긴 I/O는 startup worker에서 수행하고 기존 cookie pending과 OR하여 시작 gate를 유지한다.

## 부모 crash와 paired 권한 한계

paired guardian은 한 원 Job의 terminal 확인 뒤 managed 및 cookie 각 원 journal에 receipt를 sync한다. pair Terminal은 두 complete 성공 및 HANDLE close 후에만 보낸다. 한 receipt sync 뒤 crash면 한 도메인만 종료 증거를 가질 수 있다. 관리 회수는 관리 도메인의 실제 receipt로 그 관리 파일 정리만 허용하며 cookie 파일을 삭제할 권한은 얻지 않는다. 두 도메인을 모두 회수했다고 보고하려면 각 원 immutable chain/실제 receipts가 독립 통과해야 한다. pairdigest는 live protocol의 결합 검증이며 durable journal에 별도 저장된 pair receipt가 아니므로 디스크에서 두 recovery를 원자적 완료로 주장할 수 없다. helper가 두 journal을 acquire하기 전 부모가 죽으면 Intent/미완료 Bound가 남아 보존된다. guardian alive는 journal Busy 또는 marker sharing/lock 상태로 차단하며 PID만으로 판단하지 않는다.

## 실패 경로 인수 조건

- 실제 Windows app-kill 실행 중: 원 guardian Job 진행 동안 Busy/pending이며 원 tool/cookie/sentinel을 보존한다. 두 receipts sync 후 fresh process에서 원 managed journal·8 receipts·marker만 사라지고 새 managed execution이 가능하다.
- marker 생성 전/후, journal 일부 생성, v3 sync 중, Intent, Bound missing receipt, pristine, receipt 한 개 sync 후 중단: 보존·gate 유지.
- legacy v1/v2, unknown fields/oversize, zero nonce, wrong base digest/anchor, marker replacement, root/lock replacement, reparse/hardlink/ACL widened, receipt foreign replacement: 삭제 0, source/sentinel bytes 불변.
- live journal/execution lock contention: bounded Busy 결과, 기다리며 UI를 멈추지 않는다.
- cleanup disposition/handle close/부재 확인 실패: marker 유지, 전체 recovered=false; partial 삭제를 성공으로 숨기지 않는다.
- 정상 종료와 fresh reopen 회수 모두 실제 native compiler/Job/receipt 시험으로 검증한다. 순수 parser·합성 Resolved 값만으로 native startup PASS를 주장하지 않는다.

## 검증과 handoff

2026-10-04 읽기 전용 source 감사. 실행한 명령은 지정 파일 `rg -n` 및 `sed` 읽기만이다. Rust/Cargo 실행·변경, GUI 실행 및 자동 회수 활성화 없음. 이 문서는 proposed 계약이며 실제 구현 완료가 아니다. 원 native ACL/identity와 actual Job receipt를 확보한 뒤 첫 vertical slice는 Windows command-line fresh-process startup 회수이며 GUI wiring은 그 다음이다.
