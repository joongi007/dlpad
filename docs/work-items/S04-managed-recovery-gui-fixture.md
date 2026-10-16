# S04: Windows managed startup GUI 복구 fixture 준비

- 상태: implemented; native-gui-validation-pending
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; windows_paired_parent_crash_fixture.rs와 본 문서만
- 선행 작업: 기존 actual parent kill/paired receipt fixture, root private CPU startup GUI observer

## 사용자 결과

실제 receipt를 보존한 startup 복구 대상과 pristine pending/손상 marker 대상을 합성 fixture로 준비한다.

## 계약과 범위

paired-preserve는 actual5 receipt/원 pins 확인·사본 보존까지만 하며 공개 recover 호출금지. prepare-pending/prepare-malformed는 fresh namespace/CAP/sentinel/data/managed execution을 만들고 drop하여 pending 보존; malformed는 그 원소유 marker bounded partial bytes만 쓴다. 기존 parent/inspect 동작 유지, 기존 공유 API/Cargo/다른 파일 변경 제외.

## 인수 조건

prepare/preserve 완료와 실제 GUI 복구 완료를 구분한다. original source/foreign/tool/hash 검사, 실제 owner/marker/journal/receipts 존재, no helper PID/debug hook, bounded/create-only I/O. root sourcefreeze 후 native build/private GUI가 검증하며 visible fallback 금지.

## 검증과 handoff

source freeze SHA256 `353e34ce61926244e36d981fc25225d34542b2a43690583740e8be62680ffe0d`. CLI는 기존 namespace `output/dlpad-paired-parent-crash-<nonce>/owned-fixture`에서 `paired-preserve`, `prepare-pending`, `prepare-malformed`를 받는다. 기존 parent/inspect mode 및 실제 cookie scan sentinel rejected1 수정은 유지했다.

paired-preserve는 inspector의15초 actual5receipt readiness→원256사본/SHA→원 source/private/tool/CAP/sentinel/immutable original pins 확인을 공유한다. 관리 marker/journal/8receipts 및 cookie owner/privatecopy 존재를 요구하고 **ToolManager::recover_execution/cookies::recover_sessions 호출 전에** schema1/phase paired-preserved/both_receipts_ready/source_foreign_preserved true/recovery_called false를 기록·flush한다. 사본 관측을 native receipt validity/GUI 회수 성공으로 표현하지 않는다.

prepare-pending/malformed는 fresh empty 또는 create-only namespace/CAP/foreign/data/ToolManager를 만들고 begin_execution을 drop하여 원 pristine reader pending을 남긴다. malformed는 no-reparse/16KiB 이내인 그 원소유 marker만 partial `{"version":`(11 bytes)로 truncate/write/fsync한다. phase pending-prepared/malformed-prepared 및 원 marker/CAP/sentinel SHA/bytes를 create-only prepared-evidence.json에 기록한다. 이 두 모드에는 실제 cookie source/도구 reader가 없으며 pristine reader는 해당 생성 API 실행 사실로만 기술한다.

staging rustfmt 실제 exit0. source 한 파일 및 본 작업 문서 외 변경 없음. Cargo/native GUI는 아직 미실행이며 root build/private observer+CPU startup 인수 대기. visible desktop/clipboard/전원/WSL 변경 없음. 이 fixture만으로 startup 제품 동작이 검증되었다고 주장하지 않는다.

주 담당 실제 Windows 인수: 첫 준비 수집에서 fixture·두 C# observer 컴파일 및 원 부모 종료 후 receipt 보존 PASS. 실제 GUI pending/malformed의 재확인·원 보호 SHA·malformed 시작 차단·원 Job 자연 종료/private desktop 종료/input desktop 유지 PASS, pending-malformed-root-review.json으로 독립 대조했다. completed 첫 수집은 .NET4 경로 길이 제한으로 실패하여 원 실패를 보존했다. 짧은 fresh `mg-gui2` 합성 경로에서 새 부모 종료 기록으로 completed GUI 회수/실제 비정상 중단 이력·원 보호 SHA·소유 기록 부재·DB Interrupted를 통과했고 root-review.json으로 독립 대조했다. 당시 고정 앱 SHA709de3이며 이후 pending 안내를 항상 보이는 status에 보강했다. 최신 Windows 전체 fmt/clippy/test/release PASS, 최신 pending 화면 재검증은 진행 중이다. 실제사이트/IME/절전/전체 P0 완료로 확대하지 않는다.
