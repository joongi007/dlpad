# S04: 복구 재확인 버튼의 실행 가능 상태 통일

- 상태: implemented; Windows workspace verified; live GUI validation pending
- 분류: fix
- 우선순위: P0
- 요구사항: R03, R08, R11
- 담당: root
- 수정 파일: crates/app/src/accessible.rs, 본 문서
- 의존성: 기존 startup 복구 worker 및 관리 pending UI; 진행 중 Deno native collector 종료 후 source 변경

## 범위

도구 화면의 이전 작업 다시 확인 버튼이 다른 panel worker 실행 중에도 활성화되지만 실제 recover handler는 요청을 거부하는 상태를 고친다. 다운로드 화면과 도구 화면 및 handler가 같은 실행 가능 조건을 사용하여 사용자가 실행되지 않을 버튼을 누르는 동작을 제거한다. 새 worker/API/schema/의존성 없이 기존 close_wait와 download/tools/history/library/diagnostics busy 조건을 유지한다.

## 인수 조건

busy 또는 CloseWait 중 재확인 비활성/새 worker 생성 금지, idle 상태에서 기존 복구 허용. 기존 Windows startup 실패/보존 회귀 및 fmt/clippy/test/release를 실행한다. 실제 live Busy/CloseWait GUI 인수는 별도 담당 fixture/observer 통합 후 수행하며 단위시험을 GUI 완료로 표현하지 않는다.

## 검증

현재 코드와 UI 활성 조건 차이를 읽기 확인했다. 구현 및 새 시험은 아직 수행하지 않았다.


## 구현과 root 검증

`Candidate::can_recover`가 기존 close_wait/download/tools/history/library/diagnostics busy 조건을 단일화하고, recover와 도구 화면 버튼이 이를 함께 사용한다. 다운로드 화면은 기존 동일 busy 조건의 enabled scope를 유지한다. 기존 CloseWait 회귀가 idle enabled→CloseWait disabled와 새 worker 미생성을 함께 확인한다. 새 dependency/schema/worker/thread 비용 없음.

첫 Windows fmt check는 줄 정렬로 exit1; 원 실패 [기록](../validation/evidence/windows-recovery-retry-availability/result.json)을 보존했다. 정렬 후 fresh Windows GNU CPU `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked --features cpu -- -D warnings`, `cargo test --workspace --locked --features cpu`, `cargo build --workspace --release --locked --features cpu` 모두 exit0. [전체 결과](../validation/evidence/windows-recovery-retry-availability-formatted/result.json)와 [현재 source 독립 대조](../validation/evidence/windows-recovery-retry-availability-formatted/root-review.json). 실제 live creator Busy GUI observer/fixture는 별도 작업 중이며 해당 인수와 실제 CloseWait GUI는 아직 완료하지 않았다.
