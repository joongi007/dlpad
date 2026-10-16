# S03: 이력 정책의 OS 공통 회귀 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R07, R08, R10, R11
- 담당/수정 범위: root; crates/engine/src/download.rs의 시험만, 본 문서 및 전용 검증 증거
- 선행 작업: S03-history-bounded-api

## 사용자 결과

이력 페이지 상한과 재개 실패 정책을 Linux 외 Windows native에서도 같은 DB fixture로 검증한다.

## 계약과 범위

ADR-0002의 같은 Job ID/검증된 typed 옵션 계약을 유지한다. Unix 실행 파일 생성 fixture에 묶였던 순수 DB 시험 세 개를 공통 cfg(test) module로 옮긴다. 제품 API·schema·Cargo·실행 정책 변경은 없다. 합성 URL와 임시 폴더만 사용한다.

## 인수 조건

손상/초과 job JSON·terminal state·변경 옵션은 새 Attempt 없이 거절한다. 잘못된 ID·버전 token은 노출하지 않는다. 128행 keyset page는 반복하지 않는다. Linux 해당 세 시험, fmt/clippy 및 Windows native 해당 시험 실제 결과를 대조한다.

## 검증과 handoff

구현 전 기록. Windows native 수행 전에는 OS 공통 검증 완료를 주장하지 않는다. macOS 실제 실행은 별도 게이트다.

실제 결과: `cargo fmt --all -- --check` exit0, `cargo test -p dlpad-engine --locked portable_history_tests` 3PASS/0failed, `cargo clippy --workspace --all-targets --features cpu --locked -- -D warnings` exit0. `python3 scripts/windows_native_probe.py test --features cpu` exit0, 253PASS/1ignored, portable_history_tests 3PASS를 raw log에서 확인했다. 입력 258개 before/after/current SHA가 동일하다. [지속 증거](../validation/evidence/portable-history-policy/result.json). macOS 실제 시험은 미검증이며 제품 정책·schema·dependency 변경은 없다.
