# S04: 공개 다운로드 첫 도구 이전 관리 환경 실패 인수

- 상태: implementation-ready (회귀 실행 대기)
- 분류: test
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; 본 문서 및 신규 `/tmp` 회귀 초안만
- 선행 작업: root 관리 environment full-package 및 root binding 수정

## 사용자 결과

잘못된 관리 환경은 첫 도구 version 실행 이전 고정 Policy 오류로 거절하고 원 관리 journal을 pristine으로 보존한다.

## 계약과 범위

기존 classification fixture(root)의 관리 package를 이용해 공개 DownloadRepository create/execute_with_session을 호출한다. A policy+B scope, 다른 Deno 선택, factory 이후 notice 변조를 검사한다. missing owned tool을 사용하고 정확 Policy variant와 scope.finish pristine을 모두 요구하여 도구 실행 실패가 정책 검증 PASS로 오인되지 않게 한다. 프로젝트 Rust/Cargo/native 실행은 root 담당이다.

## 인수 조건

3 native 회귀는 첫 도구 이전 Policy 및 pristine finish·재사용을 확인한다. 실제 실행 전 회귀 source 준비를 native PASS로 표현하지 않는다.

## 검증과 handoff

임시 소스 rustfmt/SHA/API를 root에 전달한다.

`/tmp/dlpad-managed-download-preflight-regression.rs` SHA `f8d2f4f5ed5bc8d7e83b600830e020f51d2c7c9aca7ccdb885c41caeee82af3a`를 동결했다. `rustfmt --edition 2024 /tmp/dlpad-managed-download-preflight-regression.rs` exit0. 기존 classification test module의 합성 fixture를 재사용하며 공개 Repository open/create_job/execute_with_session, factory/begin/finish API만 호출한다. 별도 private adapter 변경은 필요하지 않는다.

세 Windows 회귀는 A 환경+B scope, 존재하는 외부 합성 Deno 선택, factory 이후 notice 변조를 다룬다. owned missing 도구 경로를 넘겨 정확 `DownloadError::Policy("managed runtime original proof changed")`를 요구하고, Failed/빈 artifact, pristine finish·재사용, 원 main/외부 byte 보존을 확인한다. 첫 도구 이전 정책 거절이 목적이므로 helper 실행 선행 조건은 없고 Windows cfg 기본 테스트로 작성했다. Cargo/native 실행 및 프로젝트 Rust 게시는 하지 않았다. root 실제 실행 결과가 남았다.

## root 통합

회귀 원SHA f8d2f4f5ed5bc8d7e83b600830e020f51d2c7c9aca7ccdb885c41caeee82af3a를 대조하여 tools/managed_deno_classification.rs 기존 tests에 게시했다. root가 제품원scope/preflight수정 및 실제 실행·전체검증을 담당한다. 공식Denoexample를 포함한 직전 workspace4명령은 통과했으며 현재 새회귀 source의 실제 인수는 이후 확인한다.

## 실제 Windows 인수

[공개 preflight3개](../validation/evidence/windows-managed-download-preflight-20261004T123432Z/result.json)는 Windows native exit0/3PASS다. 존재하지 않는 yt-dlp를 열기 전에 exact Policy 오류로 거절하며 DB Failed/empty artifacts와 원 journal pristine finish/reuse 및 외부·원 bytes 보존을 확인했다. root-review.json에서 현재 source SHA를 대조했다. 전체 workspace/public 성공 회귀는 별도이며 이 범위를 GUI/actual site 인수로 확대하지 않는다.
