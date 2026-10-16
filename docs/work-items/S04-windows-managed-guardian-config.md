# S04: Windows managed guardian bounded config

- 상태: implemented; integration-validation-pending
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; 신규 process/windows_managed_guardian_config.rs와 본 문서만
- 선행 작업: Windows managed reader record typed-stage/8-slot 계약, 기존 cookie config

## 사용자 결과

관리 도구 helper handoff를 bounded/redacted 구성으로 분리하여 잘못된 Intent·nonce·경로·timeout 입력을 도구 실행 전에 거절한다.

## 계약과 범위

256KiB JSON/version1/deny unknown/UTF16 경로·argv와 timeout/log 상한. ManagedReaderRecord의 이미 등록된 index Intent를 clone.bind(index,1,1)로 확인하고 nonce를 대조한다. root_path는 native 소유권 증명이 아니며 실제 helper ID/receipt/native 권한은 별도 runtime에서 검증한다. parent declaration/helper 연결은 root 담당. Cargo/sharedschema/기존 source 변경 제외.

## 인수 조건

정상 roundtrip/redactedDebug, 잘못된 index/nonce/Intent 상태/unknown field/JSON상한/UTF16 NUL/timeout/log상한 실패 회귀. 실제 launcher 사용이나 Windows runtime 완료로 보고하지 않는다.

## 검증과 handoff

신규 모듈 source freeze SHA256 `a795b706ad27234358ed93b241976bd72cf57cbbe7b09f0ea886ac8dfbcb8889`. `ManagedGuardianConfig::{validate,encode,decode,timeout}` 및 Windows `process_spec/root_path` 구현. Debug/error는 고정 redacted 문자열이고 경로·argv·nonce를 출력하지 않는다. native ID 1/1은 clone의 Intent 상태 검사에만 사용하며 원 record를 변경하거나 native 증명/receipt를 생성하지 않는다.

6개 의미 있는 회귀를 작성했다: roundtrip/redaction; scalar/24h/subsecond/log bounds; executable/root/args NUL·unit/count/sum bounds; pristine/잘못된 nonce/없는 index Intent/이미 Bound·Resolved 거절; 모든8 index의 순차 typed Intent→Bound→Resolved 상태검증; unknown/malformed/oversized JSON와 encode 상한. 실제 tests는 아직 실행하지 않았다. Cargo/rustfmt/native 실행 및 parent module 수정은 하지 않았고 root의 module 연결/컴파일/회귀 검증을 기다린다. **실제 launcher는 이 config를 아직 사용하지 않는다.**

### root 첫 통합 검증

rootmodule연결후6시험이fixture생성에서전부실패했다. 원record의nativeanchoralias거절정책에반해fixture가12개파일에같은ID를사용했다. root는서로다른12ID로fixture를수정하고재실행한다. 제품정책을완화하지않으며첫원결과0PASS/6FAIL을보존한다.

주 담당 fixture의 12개 원 파일 ID를 서로 다르게 수정한 후 설정 테스트 6개가 모두 통과했다. 최초 실행의 cargo PATH 실패와 쓰기 제한 실패는 코드 테스트 결과가 아니며, 프로젝트 쓰기 권한으로 재실행한 실제 cargo test의 결과는 6 passed / 0 failed다. Windows 전체 native 인수는 별도다.

주 담당 실제 Windows GNU CPU 통합 검증: `python3 scripts/windows_current_workspace_probe.py .../windows-managed-journal-import-retry --execute --seconds 1800`의 fmt·전체 Clippy(-D warnings)·workspace test·release 모두 통과했다. 주 담당이 result.json과 원 로그, 최종 소스 SHA를 독립 대조했다. native journal 4개 실패 보존/잠금/원 receipt 시험도 통과했다. 자동 회수·실제 reader guardian 연결은 아직 미완료이며 합성 receipt 시험을 실제 Job 종료 증거로 사용하지 않는다.
