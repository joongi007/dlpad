# S04: Windows paired 부모 종료·회수 fixture

- 상태: implemented; native-validation-pending
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; 신규 examples/windows_paired_parent_crash_fixture.rs와 본 문서만
- 선행 작업: root managed recover_execution API/paired runtime, metrics observer 소유 handle 종료

## 사용자 결과

실제 인증 다운로드 중 부모 원 프로세스 종료 후 helper 원 Job 정리·양 receipt 기반 공개 회수를 확인한다.

## 계약과 범위

paired-parent/paired-inspect 명시 owned-root 한정. 원 private cookie 실제 read 뒤만 준비 이벤트, observer가 원 parent handle로 종료. helper PID 조회/종료·debug hook·실제 계정/네트워크/영상 없음. create-only fixture/evidence와 bounded read/poll. 공개 회수 API만 완료 권한을 판정하며 receipt 사본은 증거 보존 목적이다. Cargo/다른 source 변경 제외.

## 인수 조건

부모 creation identity와 actual reader ready, 종료 후 max15초 actual receipts 관측·복사/hash, public managed/cookie 회수 성공, owner/marker/journal/receipts 제거·재사용. DB Interrupted recovery와 secret/source/privatepath 부재, source/tool/CAP/foreign hashes 보존. 실제 native 실행 이전 완료 주장 금지.

## 검증과 handoff

root GO 후 신규 source create-only 게시·동결 SHA256 `2704c8e0b41186fbbc9942f4cb67798ad6435e411e3d0245dfcb418fc7926f5d`. observer namespace `output/dlpad-paired-parent-crash-<nonce>/owned-fixture`와 CLI `paired-parent <ownedroot>`/`paired-inspect <ownedroot>`를 metrics_recovery와 합의했다. 원 root는 producer만 생성(또는 empty 확인)하며 모든 ancestor의 reparse attribute를 거절한다. 신규 writes는 create-only/16KiB, fixture reads는8MiB streaming bound, 원 executable hash는256MiB bound다.

parent는 원 source/privatecopy/tool/CAP/sentinel SHA·원 parent creation을 original-pins.json에 기록하고 공개 Repository의 managedSome+cookiesSome을 사용한다. 도구는 version/no cookie, metadata/free secret, download/실제 private cookie CAP검증 후 fsync ready→30초 대기한다. readiness thread는20초 이내 도구 준비 신호를 실제 관측한 뒤만 schema1/phase paired-tool-started/parent_creation을 stdout flush한다. 준비 실패는 취소 후 실패로 남는다. parent는 helper/도구 PID를 조회하거나 종료하지 않는다.

inspect는15초 이내 actual managed3/cookie2 receipt256을 기다려 원사본/SHA를 crash-receipt-evidence에 보존한다. 이것만으로 validity를 선언하지 않고 `ToolManager::recover_execution` recovered&&!pending 및 `cookies::recover_sessions` recovered1/deferred0/rejected0을 실제 성공해야 both_receipts_valid를 출력한다. 원 private bytes/source/tool/CAP/sentinel SHA를 회수 전에 대조하고 관리9files+marker/cookie owner 실제 제거·재사용, DB recover1/Interrupted/artifact0 및 DB/WAL/SHM secret/source/privatepath 부재를 요구한다. evidence JSON은 schema1/phase paired-recovery-evidence와 합의한4true필드를 갖는다. 원 immutable fixture binary도 freshinspect 실행 파일과 SHA/path 일치를 확인한다.

staging rustfmt 실제 exit0. Cargo/native 실행 및 다른 source 변경 없음. root build/observer actual native 인수·Job Active0·Terminated73 여부는 아직 미검증이다. startup 자동 GUI 회수는 이번 작업 밖이며 여전히 비활성이다.

주 담당 실제 인수: `windows-managed-startup-recovery-workspace-first` Windows GNU CPU fmt/clippy/test/release PASS, 신규 보존 회귀 8개 실제 통과. `windows-paired-parent-crash-sentinel-retry`는 공개 Repository 실행 중 원 부모 HANDLE TerminateProcess73, 실제 두 도메인 receipt·원 관리/쿠키 복구·재사용·DB Interrupted·원 Job 자연 Active0·private desktop 종료/input desktop 불변을 통과했다. root-review.json에서 원 receipt 5개 SHA/256바이트, 원 cookie/tool/sentinel 보존, DB Interrupted/artifact0·인증 비밀 부재, 소유 journal/pending/owner 실제 부재 및 당시 source SHA를 독립 대조했다. 초기 fixture의 foreign sentinel rejected0 기대 오류는 기존 API 계약의 scanned2/recovered1/rejected1로 고쳤으며 두 원 실패를 보존했다. 관측기의 빠른 종료 뒤 미수신 pipe 출력을 먼저 읽도록 bounded drain을 보강했다. 실제 GUI 전체 crash·sleep/reboot와 전체 P0 인수는 별도다.
