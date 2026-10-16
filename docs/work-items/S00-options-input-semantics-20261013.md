# S00: 속도 단위와 구간 입력의 목적·오류 연결

- 상태: in-progress
- 분류: UX
- 우선순위: P1
- 요구사항: R01, R06, R07, R13
- 담당/수정 범위: windows_auto — /tmp/dlpad-options-semantics-candidate-20261013만; root — accessible_options.rs/catalog.rs 채택·통합·검증
- 선행 작업: S00-ui-cross-surface-audit-20261013의 source고정 실제검증 종료

## 사용자 결과

속도 단위 선택의 목적과 잘못된 구간 입력의 이유를 입력 위치에서 이해한다.

## 계약과 범위

기존 stable ID/native Combo/변환·worker·입력값·gating을 유지한다. 중앙 catalog 목적 문자열과 기존 Invalid/description 패턴을 사용한다. 공유 catalog 채택은 root가 조정한다. 현재 단계는 /tmp 후보뿐이며 프로젝트 구현 완료가 아니다.

## 인수 조건

실제 목적 Name/Value/RuntimeID, 유효→잘못된 구간→수정 상태와 관련 입력에만 오류 연결, focus/scroll 공개, 기존 단위 변환·빈 구간 기본값 보존을 확인한다. 단위테스트만으로 reader/전체GUI 인수를 대신하지 않는다.

## 검증과 handoff

현재 소스의 rate unit 목적 label 누락과 range 그룹 오류만 표시되는 경로를 read-only audit에서 확인했다. source freeze 종료 뒤 root가 후보와 meaningful 회귀를 검토하고 새 통합 proof와 실제 UIA/캡처에 연결한다. native/Cargo/변경동작 검증은 아직 실행하지 않았다.

root가 모든 원source SHA/bytes를 먼저 대조한 뒤 독점 파일 후보를 채택했다. Rust1.94 파일별 rustfmt를 실행했고 fresh Windows 전체검사 windows-msvc-ui-semantics-repaint-whole-20261013을 시작했다. 이전 성공 proof는 고정 snapshot으로 보존하며 새 소스의 검사 성공을 아직 주장하지 않는다.

## 후속 실제 통합 — 2026-10-13

root가 후보를 현재 소스에 채택하고 Windows `fmt`, `clippy --workspace --all-targets --locked -- -D warnings`, `test --workspace --locked`, `build --workspace --release --locked`를 실제 실행했다. whole-v3 **1272/0/30**,43 harness와 원본 로그·source/artifact/lock 해시 독립 대조PASS. 앞의 후보/미실행 문단은 이전 단계 기록이다. 현재 전체GUI/reader/물리 입력 인수는 미완료다.

Light100 actual rate/range v3는 목적Name·Value·stableID·단위변환·오류→수정→복귀와 자연종료/Job0을 root 검토했다. UIA validity/description 미지원은 reader 검증 성공으로 쓰지 않는다. Dark200 scoped 검증은 진행 중이다.
