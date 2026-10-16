# S04: Windows 관리 실행 복구 통합

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08, R11, R13
- 담당/수정 범위: root; crates/engine/src/tools.rs, tools/execution_marker.rs, tools/windows_managed_recovery.rs, scripts/windows_paired_parent_crash_probe.py, 본 문서 및 P0-status/CONTEXT/ADR
- 선행 작업: v3 marker 원 anchors와 base digest, 실제 managed/paired guardian receipt, 복구 계약/초안

## 사용자 결과

Windows에서 앱이 중단된 뒤 실제 소유 reader가 종료됐음을 원 기록과 실제 receipt로 확인한 경우만 관리 실행 보류를 해제한다. 실행 중·손상·미완료 기록은 보존한다.

## 계약과 범위

기존 MarkerBinding을 단일 권위로 v3 recovery seed를 파싱한다. 신규 모듈은 원 파일 identity/ACL/namespace와 no-create install lock을 검증하고 late marker DELETE 권한 및 원 journal lock을 얻는다. 실제 전체 receipts를 검증한 뒤 9개 journal namespace를 소비 정리하고 marker를 마지막에 삭제·닫아 부재를 확인한다. public ToolManager::recover_execution은 recovered/pending을 구분한다. UI 자동 호출은 최초 실제 부모 종료 인수 뒤 별도 연결한다. 새 dependency/Cargo/schema 없음. root 공유 인터페이스 담당, 하위 담당은 독립 tests/example/observer 파일만 수정한다.

## 인수 조건

완료 실제 receipt 세션은 원 journal/8 receipts/marker 부재와 재사용을 확인한다. absent/live/pristine/Intent/legacy/unknown/oversize/foreign/wrongdigest/ACL 실패는 보존하고 원 source/sentinel을 유지한다. 정리 일부 실패를 전체 성공으로 표시하지 않는다. paired cookie 회수는 독립 원 cookie 권한 검증을 사용한다. Windows fmt/clippy/workspace test/release 및 actual parent-only kill/Jobempty 인수를 통과한다.

## 검증과 handoff

초안 검토 후 구현 시작. Git invalid 상태를 유지하며 기존 .git을 변경하지 않는다. Linux/macOS 제품 인수는 최초 릴리스 후속이다.

주 담당 실제 인수: `windows-managed-startup-recovery-workspace-first` Windows GNU CPU fmt/clippy/test/release PASS, 신규 보존 회귀 8개 실제 통과. `windows-paired-parent-crash-sentinel-retry`는 공개 Repository 실행 중 원 부모 HANDLE TerminateProcess73, 실제 두 도메인 receipt·원 관리/쿠키 복구·재사용·DB Interrupted·원 Job 자연 Active0·private desktop 종료/input desktop 불변을 통과했다. root-review.json에서 원 receipt 5개 SHA/256바이트, 원 cookie/tool/sentinel 보존, DB Interrupted/artifact0·인증 비밀 부재, 소유 journal/pending/owner 실제 부재 및 당시 source SHA를 독립 대조했다. 초기 fixture의 foreign sentinel rejected0 기대 오류는 기존 API 계약의 scanned2/recovered1/rejected1로 고쳤으며 두 원 실패를 보존했다. 관측기의 빠른 종료 뒤 미수신 pipe 출력을 먼저 읽도록 bounded drain을 보강했다. 실제 GUI 전체 crash·sleep/reboot와 전체 P0 인수는 별도다.

앱 호출 최종 검증: `windows-managed-startup-app-workspace`에서 Windows GNU CPU fmt/clippy/workspace test/release 전부 PASS, 신규 시작/보류/변조/재확인/CloseWait 호출 테스트 5개도 실제 통과했다. 주 담당 final source SHA와 현재 파일을 독립 대조했다. Windows 앱의 기존 단일 worker에 자동 회수를 연결했고 managed pending을 cookie pending과 구분하며 관리 경로의 실행/변경 차단과 외부 경로 허용 정책을 유지한다. 실제 최신 GUI 시작·클릭·시각 동작은 후속이며 전체 P0 완료가 아니다.
