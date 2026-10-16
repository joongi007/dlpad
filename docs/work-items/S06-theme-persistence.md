# S06: 테마 저장과 작업 상태 분리

- 상태: done
- 분류: fix
- 우선순위: P0
- 요구사항: R06, R08
- 담당/수정 범위: 주 담당; crates/app/src/main.rs, crates/engine/src/store.rs, .gitignore, docs/DIAGNOSTICS.md
- 선행 작업: S00 개발 UI

## 사용자 결과

빠른 테마 변경도 마지막 선택을 저장하며 테마 저장이 다른 작업의 busy 상태를 해제하지 않는다.

## 계약과 범위

테마 쓰기 한 번만 실행, 대기 선택 최신 하나로 병합. 기존 Store SQLite 사용, legacy JSON 읽기 fallback. 추가 dependency/schema 없음.

## 인수 조건

연속 변경과 실패 상태 회귀 테스트, 통합 fmt/clippy/test/release 및 문서 검사. IME/OS theme는 별도 게이트.

## 검증과 handoff

cargo fmt --all -- --check, cargo clippy --workspace --all-targets --locked -- -D warnings, cargo test --workspace --locked: 성공(61 tests). cargo build --workspace --release --locked 성공. 연속 테마 변경·다른 busy 상태 보존·DB 쓰기 실패 후 이전 선택 보존 검증 포함. Git 저장소 없음. 개발 도구를 프로젝트 전용 .local-tools에 복구했다. OS System theme 실시간 변경과 IME는 미검증.
