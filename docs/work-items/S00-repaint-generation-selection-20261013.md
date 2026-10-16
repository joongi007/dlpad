# S00: 실제 egui pass generation에 맞춘 callback repaint 선택

- 상태: in-progress
- 분류: performance
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — software_host.rs·통합·측정; windows_auto — /tmp/dlpad-repaint-generation-candidate-20261013 후보 작성 완료
- 선행 작업: S00-current-idle-cost-diagnosis-20261013, 현재 UI native source고정검증 종료

## 사용자 결과

이미 두 pass 이상 진행한 UI의 오래된 repaint callback을 다시 실행하지 않으면서 정상 worker와 한 pass settling wake를 보존한다. 기존 CPU 예산 실패 해결은 별도 동일조건 실제 측정으로 판정한다.

## 계약과 범위

locked eframe0.33.3의 현재==request 또는 현재==request+1 계약을 참고한다. bounded 두 generation 슬롯에 각 earliest deadline을 유지해 오래된 immediate deadline을 새 지연 request로 재표시하지 않는다. callback에서 Context를 읽지 않고 소비자가 Context→queue 순서로 읽어 lock inversion을 피한다. 하나의 ROOT viewport에만 적용한다. Root output/metricsfreshness/worker/hidden/close/schema/Cargo.lock/threshold를 바꾸지 않는다. 현재는 /tmp 후보이며 실제 원 peak의 원인 확정이 아니다.

## 인수 조건

실제 Context.run/callback의 settling·두 pass 후 stale·worker immediate/delayed 경합·cachehit·다른 viewport·8generation bounded eviction을 회귀로 확인한다. fmt/clippy/workspace/test/release 실제 통합과 원피크실패 보존, 동일18구간 기본 빌드 비용 재측정, 기존 focus/pointer/hidden/close/producer recovery를 확인한다. 진단6구간만으로 전체 기본 예산 인수를 주장하지 않는다.

## 검증과 handoff

root는 전체 후보와 일곱 actualContext 회귀를 읽고 현재 baseSHA8cf96305…를 확인했다. native pinned UI 시험 종료까지 source는 유지한다. 채택·Cargo/native 회귀·compiled크기·실제 CPU 개선은 아직 검증 전이다. 두 Option 슬롯은 heap allocation이나 새thread를 만들지 않으며 실제 byte/runtime 비용은 빌드 후 기록한다.

root가 모든 원source SHA/bytes를 먼저 대조한 뒤 독점 파일 후보를 채택했다. Rust1.94 파일별 rustfmt를 실행했고 fresh Windows 전체검사 windows-msvc-ui-semantics-repaint-whole-20261013을 시작했다. 이전 성공 proof는 고정 snapshot으로 보존하며 새 소스의 검사 성공을 아직 주장하지 않는다.

## 후속 실제 통합 — 2026-10-13

root가 후보를 현재 소스에 채택하고 Windows `fmt`, `clippy --workspace --all-targets --locked -- -D warnings`, `test --workspace --locked`, `build --workspace --release --locked`를 실제 실행했다. whole-v3 **1272/0/30**,43 harness와 원본 로그·source/artifact/lock 해시 독립 대조PASS. 앞의 후보/미실행 문단은 이전 단계 기록이다. 현재 전체GUI/reader/물리 입력 인수는 미완료다.

실제 egui Context 기반7개 callback generation 회귀가 통과했다. 최신 기본18구간 CPU 재측정은 아직 실행 전이며 과거CPU2FAIL 원인 해결로 확대하지 않는다. CPU exe는8.5KiB 증가, 신규crate/version/lock 변경 없음.
