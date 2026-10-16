# S02: 쿠키 복구의 잠금 전 소유 기록 검증

- 상태: done
- 분류: fix
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — crates/engine/src/cookies.rs, 이 문서
- 선행 작업: journal anchor 연결, 전체 engine 시험의 hardlink 회귀 실패

## 사용자 결과

변조된 복구 기록은 reader journal이 사용 중이어도 거절하고 파일을 보존한다. 정상 잠금은 계속 정리 보류다.

## 계약과 범위

잠금 획득 전에 record/marker native identity·단일 링크·권한·digest·boot·cookie binding을 검증한다. 기존 journal 잠금과 receipt 증명, 같은 boot 보존 정책을 완화하지 않는다. 의존성·schema 변경 없음.

## 인수 조건

실제 잠금을 유지한 상태에서 hardlink 변조를 거절하고 source/copy/link를 보존한다. 정상 busy는 deferred. 기존 쿠키 회귀와 전체 engine 시험 통과.

## 검증과 handoff

원시 실패 /tmp/dlpad-engine-resume-trace-authorized.log: library229 pass/1 fail/2 ignored. fork~exec 사이 CLOEXEC FD 상속으로 busy 가능하지만 당시 원인은 미확정. 검증 순서 결함은 코드로 확인했다.

수정 후 실제 focused regression1 PASS, 전체 `cargo test -p dlpad-engine --locked` exit0/library231 PASS/2ignored와 guardian9 등 integration 모두 통과. 정상 busy 회귀는 deferred를 유지했다. 원시 전후 로그와 source SHA는 docs/validation/evidence/cookie-recovery-before-busy-first/에 보존했다. 전체 workspace와 OS 검증은 별도 게이트다. `cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings` exit0.
