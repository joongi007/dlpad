# S02: receipt lock 시험의 subprocess 격리

- 상태: in-progress
- 분류: fix
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: metrics_recovery — process/reader_receipt.rs의 cfg(test) 및 이 문서
- 선행 작업: root 전체 workspace receipt lock 시험 실제 실패

## 현재 근거와 가설

[전체 workspace 첫 시험 원시 로그](../validation/evidence/cookie-product-workspace-first/test.log)에서 246PASS/1FAIL/2ignored이며 actual_256_sync_and_record_lock_live_until_handles_drop의 handles Drop 이후 즉시 flock이 -1이다. 당시 errno는 기록되지 않아 lock 경합과 FD 오류를 구분할 수 없다. 같은 harness의 다른 시험이 fork/pre_exec에서 CLOEXEC FD를 잠시 상속하면 마지막 원본 FD 닫기와 global OFD 해제가 일치하지 않을 수 있다. 이 가설은 아직 실제 원인 확정이 아니다.

## 수정 계약과 인수

해당 lock 시험을 fixture 생성 전에 같은 test executable과 현재 harness test thread name의 exact filter로 owned subprocess에 격리한다. child 내부의 원래 strict flock 즉시 해제 assert를 유지한다. child 실행 상한 10초, timeout/error에는 소유 child kill 및 추가 2초 reap 확인만 수행한다. stdout/stderr는 상속해 새 출력 버퍼를 만들지 않으며 stdin은 null이다. 제품 timeout·락 정책은 변경하지 않는다.

실패 진단은 syscall 직후 errno, 원래 file dev/ino/mode/nlink, FD 번호·flags 및 같은 inode를 가리키는 자체 FD 번호만 포함한다. 경로·cookie·payload·token을 출력하지 않는다. 기존 단위 시험을 전체 병렬 harness에서 실행하고 exact child 시험이 실제 수행되는지 확인하는 인수는 root 담당이다. 독립 재현이나 root 재시험 없이 성공을 주장하지 않는다.

## 검증과 handoff

자체 Cargo/native 실행 금지. 소스 정렬 및 동결 이후 root가 실제 검증한다.

## 구현 동결

cfg(test) helper에서 syscall 직후 errno를 수집하고 실패 시에만 numeric metadata 및 동일 inode의 자체 FD 목록을 출력한다. 원래 strict assert는 재시도나 flock 대기 없이 즉시 실행한다. fixture를 만들기 전 동일 test executable의 현재 test thread name을 exact filter로 실행하며, 같은 이름의 내부 marker를 받은 child만 fixture를 생성한다. 필터 이름을 lib 전용 문자열로 하드코딩하지 않아 integration harness에도 동일 경로를 쓴다.

하위 프로세스는 10초 try_wait 관찰, 실패 시 소유 child kill 후 2초 유한 reap 관찰을 한다. 제품 소스와 다른 시험의 병렬 조건을 변경하지 않았다. output은 상속하여 무제한 captured buffer를 추가하지 않는다.

`.local-tools/cargo/bin/rustfmt --edition 2024 crates/engine/src/process/reader_receipt.rs` 실제 exit 0. 소스 SHA256 `a2c6bda8989bd59196be3eb33522ee9817acdebd018ffe26c16eefa8338ae6a3`로 root에 동결 인계했다. 자체 Cargo 및 실제 격리 실행은 하지 않았다. 가설의 독립 재현과 수정 후 exact child/전체 병렬 workspace의 실제 결과는 root 검증에서 구분해야 한다.

## journal 전체 시험의 후속 격리

root의 두 번째 전체 실행은 engine 247PASS 뒤 cookie_reader_models에서 journal reopen strict 시험이 실패했다. fixed Debug만으로 Busy를 확정할 수 없으므로 cfg(test) reopen wrapper에 오류 variant의 고정 이름을 출력하도록 했다. 제품 Debug는 변경하지 않았다.

공통 `run_isolated_lock_test()`를 reader_protocol.rs의 cfg(test) pub(crate) helper로 이동하고 receipt 시험 및 journal 모듈의 14개 시험이 fixture 전에 재사용하도록 했다. 모든 original strict proof/락 정책은 유지한다. Linux cookie recovery의 defer/drop 직후 recover도 같은 잠재적 상속 경계가 있어 root가 cookies.rs 시험 guard를 직접 조정한다. 이 담당은 cookies.rs를 수정하지 않았다. exact child가 0개 시험만 실행하고 성공 exit하는 거짓 통과를 막는 entry marker 보강은 root와 별도 합의한다.

root가Linux recovery_tests의8개fixture에도같은 exact child guard를추가했다. drop→recover의즉시flock해제 기대를다른test fork와분리하며원래 파일소유/proof/거절조건은유지한다. 두번째fullworkspace는engine247PASS였지만cookie_reader_models journal재open1case실패(39PASS/1ignored)였다. fixedvariant 진단+공통cfg(test)격리로 재검증한다. 제품동작/정리증거/deadline 변경은없다.

## 최종 ghost0 방지 및 source freeze

동일 위임 범위 내 root의 추가 승인으로 parent의 private TempDir에 create-new 진입 marker를 추가했다. exact child의 helper 분기만 고정 1바이트를 쓰며 parent는 child의 성공 exit와 최대 2바이트 marker read의 정확한 `1`을 함께 검증한다. 필터가 0개 시험만 실행하면 marker가 없어 실패한다. marker는 진입 증거이며 child 성공 exit와 결합해 strict body 완료를 요구한다. 제품 환경/프로토콜은 변경하지 않았다.

최종 SHA256:

- reader_protocol.rs: `8b3afca995af7bb74a298ea3a7cf8f521fbfe33f3d22f46ca80c0f82e929b72c`
- reader_receipt.rs: `d3cab81034838d84f7f039303236a74cd5c30bb7dc3961d022051fec614dcabe`
- reader_journal.rs: `9e20f962082eec5121ccfeacf9d23f3cdd88ef70d8c700ee26f22703f382d3b9`

세 파일의 개별 rustfmt 실제 exit 0. Cargo 및 native 재시험은 실행하지 않았고 root에게 최종 동결 인계했다. 기존 실패 증거는 보존하며 성공 결과는 root의 actual 검증 후 추가한다.

세번째workspace는fmt0/clippy101(items_after_test_module)로 test/release 미실행. root가공통cfg(test)helper를test모듈앞으로이동했다. CookieReaderModels의최상위durablegate→drop→reopen 시험도fixture전동일격리를적용하여실제proof/assert를유지했다. 원시third/clippy.log보존, 전체검증다시진행한다.

네 번째 검증은 fmt0, clippy101이었다. protocol 소스를 함께 가져오는 cookie_guardian_gate harness에는 잠금 시험이 없어 공통 cfg(test) helper의 dead_code 경고가 발생했다. 해당 helper에만 이유를 적은 allow(dead_code)를 추가했다. 제품 코드의 lint 정책과 잠금 증거는 변경하지 않았다. 다섯 번째 검증에서 fmt와 clippy는 실제 exit0이며, 전체 test/release 결과는 실행 종료 후 기록한다. 각 실패 원시 로그는 보존한다.

다섯 번째 전체 검증이 종료됐다. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo build --workspace --release --locked` 모두 실제 exit0이다. `docs/validation/evidence/cookie-product-workspace-fifth/result.json`과 각 원시 로그에 기록했고 Rust/Cargo/toolchain source-before/after SHA가 일치한다. 플랫폼 조건으로 0개가 실행된 시험과 ignored native 시험은 OS/native 인수 통과로 계산하지 않는다. 별도 native 및 다른 OS/부팅 복구 인수는 해당 작업에 남아 있다.
