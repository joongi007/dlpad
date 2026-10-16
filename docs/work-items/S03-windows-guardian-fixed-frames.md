# S03: Windows guardian fixed frame

- 상태: implemented; root 통합 검증 대기
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R02
- 담당/수정 범위: soak_recovery_audit — crates/engine/src/process/windows_guardian_protocol.rs, 이 문서
- 선행 작업: root가 합의한 DLPWG001 128-byte wire 계약; parent module 연결은 root 담당

## 계약과 인수

Prepared/Ack/Begin/Cancel/TimedOut/Terminal의 canonical 필드와 exact-size/version/reserved 검증을 순수 모듈로 구현한다. Debug는 payload를 노출하지 않는다. malformed·partial·nonce/digest/slot/version/reserved/cross-kind 회귀가 필요하다. state-machine replay 판정, helper loop, config serde, 제품 wiring은 범위 밖이며 root가 조정한다. Cargo와 parent process.rs를 변경하거나 실행하지 않는다.

## 검증과 handoff

128-byte encode/decode와 pub(super) typed getters 및 redacted Debug를 구현했다. 네 회귀는 모든 role roundtrip/LE native creation, 모든 partial length·reserved byte·header 오류, role별 필드 canonical 오류, decoder가 replay/session 권한을 주장하지 않는 경계를 검사한다. nonce/digest 일치와 role 순서는 향후 state machine이 검증해야 한다.

담당 파일 rustfmt 실제 exit 0. 기본 sandbox의 포맷 쓰기는 EROFS였고 허가된 프로젝트 파일에 대한 승격 포맷만 수행했다. Cargo·Rust 테스트·native 실행은 하지 않았으며 parent module 연결과 컴파일/회귀는 root 담당이다.

Root 통합: process.rs에 cfg(any(windows,test)) 모듈 연결. Linux `cargo test -p dlpad-engine --lib --locked windows_guardian_protocol` actualexit0, 정확4test PASS. helper 상태 머신·replay·실제 native 제어채널 검증은 아직 미완료다.
