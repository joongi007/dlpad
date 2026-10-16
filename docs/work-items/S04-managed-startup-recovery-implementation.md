# S04: Windows 관리 실행 startup 복구 구현 staging

- 상태: in-progress
- 분류: safety
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: diagnostic_cache; `/tmp/dlpad-windows-managed-recovery.rs` 및 본 문서만
- 선행 작업: S04-managed-startup-recovery-contract; root의 단일권위 `execution_marker::RecoverySeed/recovery_seed`, module 선언

## 사용자 결과

실제 원 관리 reader receipt가 모두 남은 중단 세션을 회수할 Windows 내부 함수 초안을 작성했다. 프로젝트 Rust 게시 및 UI 자동 회수는 아직 하지 않는다.

## 계약과 범위

`pub(super) recover(manager: &ToolManager) -> Result<bool, ToolError>`이며 true는 원 journal/8 receipt 및 marker 실제 부재까지 확인한 경우다. false는 marker 없음 또는 Busy/증거 미완료 pending이다. unknown/손상은 오류로 보존한다. UI는 false를 pending 부재로 해석하면 안 된다. 호출자가 pending 여부를 별도 확인하거나 root가 typed summary로 통합해야 한다.

형식 복제를 피하기 위해 root의 `execution_marker::RecoverySeed {root_id,marker_id,anchors,session_nonce,digest}`와 `recovery_seed(&[u8])`를 참조한다. 이 helper는 기존 MarkerBinding 단일권위의 v3/nonce/anchors unique/outer native IDs/canonical v2 domain SHA 검증만 수행하며 삭제권한을 주지 않는다.

초안은 기존 manager root/pin을 재검증하고 marker bounded 원 read HANDLE/strict SID ACL/nlink/namespace를 확인한다. 원 lock을 no-create로 열고 immutable lock ID/namespace를 대조한 뒤 exclusive try-lock한다. 원 marker DELETE를 늦게 재open하여 bytes/ID/ACL을 재대조한다. creator의 FILE_SHARE_READ와 충돌하면 Busy로 보존한다. journal.reopen 원 anchors→실제 all receipts→cleanup_completed 소비/9개 부재→marker exact 재확인→원 marker disposition→모든 marker HANDLE close→실제 부재 확인 순서다. marker 부재 확인까지 원 exclusive lock을 보유한다. 기존 자동 startup/UI 호출부는 수정하지 않았다.

## 인수 조건

root native 통합 시험에서 완료 receipts를 가진 fresh-process recovery만 true를 반환해야 한다. legacy/partial/pristine/Intent/Bound missing receipt/foreign replacement/wrong digest/ACL widened는 삭제하지 않는다. live creator/guardian/lock은 Busy로 보존한다. 원 source·설치 tool·sentinel bytes 불변을 검사한다. journal 부분 disposition 실패나 marker deletion 실패는 전체 성공으로 보고하지 않으며 잔여 marker를 보존한다. paired 한 도메인 완료는 다른 cookie 도메인 회수 권한이 아니다.

## 검증과 handoff

2026-10-04 실제 Rust 초안 1개 작성 및 `/tmp` 파일 개별 rustfmt exit0. 프로젝트 Rust/Cargo/기존 문서 수정 및 컴파일/native 실행 없음. root helper `/tmp/dlpad-managed-recovery-seed.rs` 필드와 맞췄다. 원 journal API를 재사용하며 새 cleanup authority는 path 이름이나 self-claimed ID로 만들지 않는다. source freeze 종료 후 root가 검토·게시·native 테스트한다. 첫 native 시험 전 API가 actual PASS인 것으로 보고하지 않는다.

주 담당 실제 인수: `windows-managed-startup-recovery-workspace-first` Windows GNU CPU fmt/clippy/test/release PASS, 신규 보존 회귀 8개 실제 통과. `windows-paired-parent-crash-sentinel-retry`는 공개 Repository 실행 중 원 부모 HANDLE TerminateProcess73, 실제 두 도메인 receipt·원 관리/쿠키 복구·재사용·DB Interrupted·원 Job 자연 Active0·private desktop 종료/input desktop 불변을 통과했다. root-review.json에서 원 receipt 5개 SHA/256바이트, 원 cookie/tool/sentinel 보존, DB Interrupted/artifact0·인증 비밀 부재, 소유 journal/pending/owner 실제 부재 및 당시 source SHA를 독립 대조했다. 초기 fixture의 foreign sentinel rejected0 기대 오류는 기존 API 계약의 scanned2/recovered1/rejected1로 고쳤으며 두 원 실패를 보존했다. 관측기의 빠른 종료 뒤 미수신 pipe 출력을 먼저 읽도록 bounded drain을 보강했다. 실제 GUI 전체 crash·sleep/reboot와 전체 P0 인수는 별도다.
