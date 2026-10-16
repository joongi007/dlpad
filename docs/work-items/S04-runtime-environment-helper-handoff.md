# S04: Windows runtime environment helper wire handoff

- 상태: implementation-ready (임시 사본; 제품 게시·인수 대기)
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; 신규 wire 모듈 및 3 guardian config/3 capture의 /tmp 수정 사본과 본 문서만
- 선행 작업: root WindowsManagedDenoEnvironment/ProcessSpec policy, native Windows environment encoder

## 사용자 결과

선택된 runtime 환경 policy가 쿠키/managed/paired helper 경계를 통과하고 잘못된 version/path는 도구 실행 전에 거절된다.

## 계약과 범위

platform-independent strict serde wire version1/UTF16 cache vector, validate_cache_units 재사용. optional field default/skipNone로 기존 None wire 유지. capture는 native policy를 검증한 뒤 wire복사, helper process_spec은 다시 검증/복원. cache path를 소유권 증명으로 삼지 않는다. root temporary fail-before-mutation guard는 모든 handoff source 검증 후 root가 제거 여부 결정한다. Cargo/dependency/native windows.rs/shared policy/프로젝트 Rust 게시 제외.

## 인수 조건

None wire unchanged/Some roundtrip/nested unknown version·field·malformed UTF16/NUL/relative path 실패 회귀. 실제3 helper 흐름 Job/stdio/gate/receipt 유지. actual native 생성·Some environment propagation 검증 전 완료 주장 금지.

## 검증과 handoff

먼저 /tmp staging; root 게시/모듈 선언과 compile/native 검증 담당. dependency0.

2026-10-04: `/tmp/dlpad-runtime-environment-handoff/`에 신규 모듈과 6 수정 사본, 원 source SHA baseline, 전체 staged SHA manifest 및 6 diff를 동결했다. 기존 프로젝트 Rust 6파일 SHA는 baseline과 모두 같으며 게시하지 않았다. `rustfmt --edition 2024 /tmp/dlpad-runtime-environment-handoff/*.rs` exit0. Cargo/순수 회귀/native 시험은 실행하지 않았다.

새 strict wire는 nested Deserialize에서도 검증하며, 3 config는 None field를 생략하고 Some 검증·복원을 수행한다. 테스트 소스는 unknown version/field, malformed UTF16/NUL/경로, None 생략·재인코딩, Some roundtrip 및 Windows native 복원을 다룬다. 세 capture의 기존 임시 `runtime environment handoff unavailable` guard는 그대로 보존했다. 따라서 현 사본을 실제 환경 전달 성공으로 보고하지 않는다.

통합 담당 잔여: 새 모듈을 `cfg(any(windows,test))`로 선언하고 `windows_helper_launch.rs`의 5 기존 config literal에 None을 추가한다. 새 모듈 SHA는 `bef3783ae4e60a0b73f0502e156fb9ed9da59a93a0779a36f8a70311824e1215`; 전체 7 SHA는 임시 `handoff-manifest.json`에 기록했다. 프로젝트 게시/회귀/Windows 실제 helper 전달 및 기존 Job·stdio·gate·receipt 인수를 통과한 뒤에만 root가 guard 제거를 결정한다.


Root는 staged7files/원6source SHA를 검토해 게시하고 module 및 root-owned GuardianConfig None literal5개를 연결했다. Windows 전체 fmt/clippy/test/release exit0와 모든 세 carrier None/Some typed wire·native복원 회귀를 확인했다. [root 근거](../validation/evidence/windows-runtime-environment-pseudo-fixed/root-review.json). 실제 helper Some 실행 인수 전까지 capture unavailable guard를 원 journal 변경 전에 유지한다. 따라서 생산 다운로드에 이 정책이 적용됐다고 주장하지 않는다.
