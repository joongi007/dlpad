# S02: 쿠키 복구 변경의 CPU 후보 통합 검증

- 상태: completed (CPU feature 빌드·시험 범위)
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당 파일: root — 이 문서, 해당 새 evidence; 제품 파일 변경 없음
- 의존성: 이전 부팅 authority 및 service 의존성 변경의 기본 workspace/native 검증

## 범위와 인수 조건

동일 Rust/Cargo source manifest에서 dlpad/cpu feature 전체 Clippy·workspace 시험·release 빌드를 검증한다. 기본 workspace 통과를 CPU 후보 통과로 대체하지 않는다. ignored native 시험과 GUI/한글 입력/Windows/macOS의 실제 동작은 별도 범위다. 보이는 desktop을 실행하지 않는다.

## 검증과 handoff

기본 workspace 네 필수 명령과 최신 native3+4+2는 통과했다. CPU feature 명령은 실행 중이며 종료 및 source 일치 확인 후 결과를 기록한다.

실제 `cargo clippy --workspace --all-targets --features dlpad/cpu --locked -- -D warnings`, `cargo test --workspace --features dlpad/cpu --locked`, `cargo build --workspace --features dlpad/cpu --release --locked` 모두 exit0이다. `docs/validation/evidence/cookie-priorboot-cpu-current-first/result.json` 및 각 로그에 명령·결과가 있고 기본 workspace와 같은 source manifest가 유지됐다. GUI·입력·접근성·다른 OS 및 ignored native의 자동 실행을 의미하지 않는다. Git 상태는 여전히 fatal/not a git repository이며 기존 .git을 변경하지 않았다.
