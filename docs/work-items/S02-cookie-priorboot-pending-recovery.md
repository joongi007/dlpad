# S02: 이전 부팅의 미확정 쿠키 reader 회수

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R02, R08, R10
- 담당 파일: root — crates/engine/src/cookies.rs, 이 문서; metrics_recovery — crates/engine/src/cookies/reader_journal.rs
- 의존성: S02-cookie-priorboot-pending-audit, 기존 recovery marker 및 journal native anchor

## 사용자 결과와 계약

실제 kernel 부팅 ID가 달라진 뒤 원래 보호된 쿠키 cache의 Intent/Bound 기록을 회수한다. 같은 부팅의 미확정 reader는 계속 보존한다. 원본 cookie, 외부 파일, 변조 항목은 삭제하지 않는다. 별도의 PreviousBootAuthority를 두 삭제 gate에 전달하며 receipt나 Resolved를 위조하지 않는다. 새 의존성/schema 변경은 없다.

## 인수 조건

원본 record/marker/native anchor/allowlist와 모든 slot의 원래 boot 및 marker digest를 검증한다. 생산 authority는 bounded kernel read에서만 현재 boot를 가져오며 각 삭제 전 재검증한다. 테스트용 boot 주입은 cfg(test)에만 존재한다. Intent/Bound 각각 두 slot, sameboot 보존, Busy/hardlink/digest 불일치 및 기존 legacy 회수를 검증한다. 실제 WSL 재부팅은 수행하지 않고 실제 reboot 인수는 별도로 미검증으로 남긴다.

## 검증과 handoff

구현 중. Rust 소스 동결 후 root가 통합 Cargo 및 native gate 시험을 실행한다. 과거 empty journal 회수 시험을 pending 회수의 증거로 대신하지 않는다.

첫 통합 all-target check는 exit0이다. engine lib는 249 PASS/2 FAIL/2 ignored였다. 새 slot1 fixture가 선행 slot0 Resolved 조건을 빠뜨려 begin에서 거절된 것과 기존 output-limit 시험의 TerminalUnconfirmed를 구분했다. 첫 원시 로그는 cookie-priorboot-dependency-first에 보존했다. slot1 준비에 실제 합성 metadata receipt 쓰기/sync/resolve를 추가하여 원 상태 전이를 지켰고, `cargo test -p dlpad-engine --lib --locked prior_boot_recovers_original_intent_and_bound_both_slots -- --nocapture` 실제 1 PASS를 확인했다. 테스트 내부에서는 Intent/Bound 각각 slot0/1 네 조합과 sameboot 보존을 검증한다. 제품 schema/정책을 fixture에 맞춰 완화하지 않았다. 두 번째 통합 실행 결과는 종료 뒤 별도로 기록한다.

두 번째 engine library는 251 PASS/2 ignored다. 최종 소스의 필수 전체 `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo build --workspace --release --locked`는 모두 exit0이며 source-before/after가 일치한다. 원시와 명령은 cookie-priorboot-dependency-workspace-seventh/result.json에 있다. 새로운 journal kernel parser/권한 생성 및 boot·digest·native 교체 거절 회귀도 이 범위에 포함된다. 실제 kernel reboot 인수와 다른 OS는 미검증으로 남겨 작업 상태를 전체 완료로 올리지 않는다.
