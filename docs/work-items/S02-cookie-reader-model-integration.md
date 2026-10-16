# S02: reader record와 protocol Cargo 검증

- 상태: done
- 분류: test
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: root — 신규 crates/engine/tests/cookie_reader_models.rs, reader_protocol 내부 visibility, 이 문서
- 선행 작업: reader protocol/record source 동결

## 사용자 결과

동일 boot 복구를 위한 두 reader의 상태·identity hash·receipt 검증을 실제 Cargo 시험에 포함한다. 제품 durable 파일 연결과 crash recovery는 후속이며 이 시험으로 완료를 주장하지 않는다.

## 계약과 범위

기존 dependency만 사용하고 내부 protocol visibility를 crate 범위로 제한한다. 원본 모듈을 path로 참조해 Cargo integration 문맥에서 시험한다. 제품 process/cookie 실행 경로는 아직 모듈을 사용하지 않는다.

## 인수 조건

부분packet/상한/nonce/index/identity digest/receipt mismatch와 version1 보존, 고정두slot 전이 시험을 실행한다. serialize record4096 상한 유지.

## 검증과 handoff

root Cargo 검증 전 in-progress. 파일 durability/native crash/제품 P0 복구는 미완료다.

root 실제 `cargo test -p dlpad-engine --test cookie_reader_models --locked` exit0, 8 tests PASS. 증거: docs/validation/evidence/cookie-reader-model-integration/. native durable 기록과 실제 crash recovery는 아직 미완료다.
