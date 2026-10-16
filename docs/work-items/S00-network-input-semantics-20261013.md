# S00: 수동 네트워크 입력의 문법 오류 연결

- 상태: in-progress
- 분류: UX
- 우선순위: P1
- 요구사항: R01, R07, R11, R13
- 담당/수정 범위: advanced_units — /tmp/dlpad-network-input-semantics-candidate-20261013만; root — accessible_metrics.rs 채택·통합·검증
- 선행 작업: S00-ui-cross-surface-audit-20261013 source고정검증종료

## 사용자 결과

잘못된 연결 이름 문법의 이유를 해당 입력에서 확인한다. 존재하지 않는 유효 이름의 관측 실패와 문법 오류를 구별한다.

## 계약과 범위

기존 input/Apply stable ID, Auto선택·worker·실제 측정·privacy를 유지한다. 기존 Invalid/description 입력 패턴을 따르고 문법 수정 시 오류 상태를 해제한다. 공유 API/schema 변경은 root 조정이며 현재 /tmp 후보만 작성한다.

## 인수 조건

실제 오류→수정/빈값→Auto 상태, 원 입력 값·ID·선택 보존, 유효하지만 없는 인터페이스의 의미 구별 및 focus/scroll 공개를 확인한다. 단위 테스트가 실제UIA/reader/픽셀 인수를 대신하지 않는다.

## 검증과 handoff

현재 Apply 후 오류 안내는 있으나 Edit에 오류가 연결되지 않은 source 경로를 read-only audit에서 확인했다. 후보 검토·채택 및 actual 회귀/whole/native는 아직 미실행이다.

root가 모든 원source SHA/bytes를 먼저 대조한 뒤 독점 파일 후보를 채택했다. Rust1.94 파일별 rustfmt를 실행했고 fresh Windows 전체검사 windows-msvc-ui-semantics-repaint-whole-20261013을 시작했다. 이전 성공 proof는 고정 snapshot으로 보존하며 새 소스의 검사 성공을 아직 주장하지 않는다.

whole-v2에서새MissingAdapter회귀가실패했다. root가vendor/egui response.rs를직접확인해Role::Label의본문이label이아닌value에들어가는실제계약을확인했다. 테스트는정확한Role::Label+NETWORK_MANUAL_FAILED value를요구하도록수정했으며workerfailure/Invalid분리인수는유지한다. production오류정책을바꾸거나검사를제거하지않았다. freshwhole-v3를실행중이다.

## 후속 실제 통합 — 2026-10-13

root가 후보를 현재 소스에 채택하고 Windows `fmt`, `clippy --workspace --all-targets --locked -- -D warnings`, `test --workspace --locked`, `build --workspace --release --locked`를 실제 실행했다. whole-v3 **1272/0/30**,43 harness와 원본 로그·source/artifact/lock 해시 독립 대조PASS. 앞의 후보/미실행 문단은 이전 단계 기록이다. 현재 전체GUI/reader/물리 입력 인수는 미완료다.

실제 Context 회귀는 syntaxInvalid→Apply거부→수정과 유효하지만 없는 adapter의 workerFailed를 구분해 통과했다. native v3 Unicode 시험은 입력 전달 readback0으로 실패; 원본 실패를 보존하고 surrogate 전달 수집기를 수정 중이다.
