# S11: CI notice 읽기 deadline 회귀의 단계 정합성

- 상태: done; Root Windows strict lint·모듈·전체 engine 인수 완료, hosted 전체 CI 후속
- 분류: bug
- 우선순위: P0
- 요구사항: R05, R08, R11
- 담당/수정 범위: updater_audit; crates/engine/src/tools/archive_install.rs와 본 문서만
- 선행 작업: ab4a7c1 hosted CI 원 로그·기존 notice transfer 실패 보존 계약

## 사용자 결과

Windows CI의 notice 읽기 timeout 회귀가 준비 I/O·스케줄링 비용 때문에 다른 단계에서 timeout되었을 때 거짓 실패하지 않게 한다. 실제 읽기 취소/시간 초과·원 입력/직전 active·foreign destination 보존 검증은 유지한다. 제품의 전체 absolute deadline을 늘리거나 reset하지 않는다.

## 계약과 범위

README→PRODUCT→관련 CONTEXT/R05/R08/R11→ADR0008와 INTEGRATIONS·ARCHITECTURE를 읽고 diagnose 순서로 재현/가설/최소 고정/회귀를 수행한다. 로컬 markdown tracker 설정을 읽었다. 별도 codex/notice-deadline-20261018 worktree를 ab4a7c1 기준으로 만들었으며 shared Cargo/lib/app_update·다른 agent 파일·main index·공개 원격은 수정하지 않는다. 네트워크/실사용 영상/계정/사용자 Documents/새 하위 agent/push0.

## 인수 조건

- 원 증상은 read-timeout의 result TimedOut이나 notice_reads==0인 단계 오인임을 정확히 재현한다.
- test fixture 시간경합과 production absolute deadline 정책을 구분한다. 단순 시간 증가·ignore·읽기 발생 단정 삭제는 허용하지 않는다.
- notice 실제 read 중 실패를 deterministic하게 주입·검증하고 원자료/직전 active/foreign 보존 회귀를 유지한다.
- Linux targeted/strict lint와 Root 실제 Windows targeted/full CI 검증을 구분해 보고한다.

## 검증과 handoff

원 hosted run39064076669의 tests.log:180에서 archive_install.rs:1214 "must fail during actual notice read: read-timeout" panic, engine618 PASS/1 FAIL/17 ignored, exit101을 확인했다. install 전체1초 deadline은 archive transfer·staging/journal 준비·notice 파일 생성보다 먼저 시작한다. notice fixture의1100ms sleep은 notice read에 도달한 경우에만 실행된다. 이 원 증상을 동일 workspace에서 재현한 뒤 테스트 fixture만 수정했다. 제품 absolute deadline 정책은 변경하지 않았다.


## 진단과 최소 변경

가설은 (1) archive/staging 준비 시간이 전체 deadline을 소비함, (2) notice URL/mode wiring 오류, (3) read counter publication 오류 순으로 세웠다. test-only transport의 archive 단계에 1100ms 준비 지연을 잠시 주입해 원 테스트를 실행하자 `notice_reads == 0` 상태로 같은 panic/exit101이 재현되었다(1.44초). 원래 notice body 안의 sleep까지 도달하지 못한 것이므로 (1)을 확인했다. 실험 지연과 DEBUG 표식은 최종 소스에서 제거했다.

`NoticeRead`의 read-timeout fixture는 첫 호출에 실제 1byte를 전달하고 두 번째 호출에 `io::ErrorKind::TimedOut`을 반환한다. 원 실패 보존 테스트는 최소 두 번의 notice Read와 실제 body I/O timeout 오류 종류를 검증한다. read-timeout에만 있던 1초 전역 guard는 다른 일반 실패 사례와 같은 기존 10초 guard로 합쳤다. 실패 발생은 sleep/경과시간이 아니라 해당 Read의 I/O 오류로 결정되므로 단순 timeout 연장이 아니다. zero deadline 사례는 여전히 `ToolError::TimedOut`, 실제 읽기 취소는 여전히 Cancelled를 검증한다.

원 입력 package/notice bytes, prior active package, foreign destination/sentinel 보존 단정은 유지했다. 새 직접 fixture 회귀는 첫 byte 내용·두 번째 TimedOut·정확히 두 Read·cancel 미설정을 검증한다. 수정은 `#[cfg(test)]` 내부뿐이며 제품 동작/전체 deadline/종속성/산출물 크기 비용은 변경하지 않는다.

## 실제 실행 결과

공통 Linux 환경은 Rust 1.94.0, `CARGO_HOME=/mnt/d/project/dlpad/.local-tools/cargo`, 독립 `CARGO_TARGET_DIR=/tmp/dlpad-notice-deadline-target-20261018`, DEV/TEST DEBUG=0, INCREMENTAL=0이다.

- RED: 임시 준비 지연을 넣고 `cargo test -p dlpad-engine --locked --offline tools::archive_install::tests::notice_transfer_failures_preserve_original_inputs_prior_active_and_foreign_destination -- --exact --nocapture`: 원 panic 동일, 0 pass/1 fail, exit101. 이후 실험 지연 제거.
- GREEN: `cargo test -p dlpad-engine --lib --locked --offline tools::archive_install::tests::notice_ -- --nocapture`: 3 pass/0 fail, 0.66초.
- 확장 회귀: `cargo test -p dlpad-engine --lib --locked --offline tools::archive_install::tests:: -- --nocapture`: 14 pass/0 fail, 2.49초.
- 소유 파일 Rust 1.94 rustfmt 실행 성공. 최종 debug 표식 검색 결과0.

Linux 테스트에서는 수정하지 않은 `media_preview.rs:591`의 cfg별 `unused_mut` 경고가 보인다. 이 파일은 담당 범위 밖이므로 수정하지 않았다. Windows native/전체 workspace/호스팅 CI의 최종 성공 여부는 Root 검증 결과로 구분한다. 권장 Windows 재검증 필터는 `tools::archive_install::tests::notice_` 및 `tools::archive_install::tests::`이다.

- `python3 scripts/check_docs.py --source-only`: PASS; 1127 documents, local-only evidence omitted, runtime evidence 미검증.
- `rustfmt --edition 2024 --config skip_children=true --check crates/engine/src/tools/archive_install.rs`: exit0.
- `cargo clippy -p dlpad-engine --lib --tests --locked --offline -- -D warnings`: exit101; 수정 범위 밖 기존 `media_preview.rs:591` `unused_mut` 한 건. Windows strict lint/실행 결과는 Root가 별도로 검증한다.

## 인계

동결 소스 SHA256: `846b42673280b05b5cbfdd88e06ae9ec4b90bae68bba4b8a0ea89ca72aaa2230`. Root가 해당 파일을 통합하며 Windows·후속 CI를 판정한다. 본 작업은 원격 push·공개 릴리스·키 생성·공유 파일 수정 없이 완료했다.


## Root 실제 Windows 통합 검증

Root가 원49c/ab4 CI 실패 로그와 패치 diff·소스 SHA를 직접 확인했다. 실제 원 Job/private desktop에서 `cargo clippy -p dlpad-engine --all-targets --locked --offline -- -D warnings` exit0, `cargo test -p dlpad-engine --lib --locked --offline tools::archive_install -- --test-threads=1` 13PASS/0FAIL을 확인했다. 원 로그는 `.local-tools/update-integration-20261018/native-engine-final-v1/native-tests.log`다. 이어진 전체 engine 시험은 Root가 정리된 debug cache의 필수 guardian 선행 빌드를 누락해8FAIL을 냈다. 원 실패는 보존했다. product/test 단정을 바꾸지 않고 `cargo build -p dlpad-engine --bin dlpad-process-guard --locked --offline` 뒤 전체 `cargo test -p dlpad-engine --lib --locked --offline`를 재실행해 주 harness635PASS/0FAIL/18ignored, 별도 filtered child1PASS를 확인했다. 같은 notice 실패 보존 시험도 실제 성공했다. 원 결과는 native-engine-final-v2/native-tests.log이며 경과43.41초, 두 명령 exit0, natural-zero/final-zero=True, forced=False, final-active0, private-desktop-closed/input-desktop-unchanged/source-unchanged=True다. 전체 Windows workspace/GPU/autoZIP hosted CI는 이 engine 검증과 별도다.
