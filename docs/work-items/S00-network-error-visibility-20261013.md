# S00: 확대 화면에서 연결 입력 오류 공개

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R01, R11, R13
- 담당/수정 범위: installer_batch — /tmp/dlpad-network-error-visibility-candidate-20261013만; root — crates/app/src/accessible_metrics.rs 통합·실제검증 및 본 문서
- 선행 작업: 현재 소스 frozen CI/검증 종료

## 사용자 결과

200% 배율에서도 잘못된 연결 이름의 설명을 입력 가까이에서 볼 수 있다.

## 계약과 범위

실제 Dark200 오류 PNG에서 입력과 Apply는 보이지만 오류 설명이 접힌 영역 아래로 밀렸다. 기존 catalog 문구·syntax policy·stable ID·Apply refusal·worker failure/Auto/Off 정책을 유지하며 오류 설명을 input 직후로 옮기는 단일 화면 후보다. 공통 component/catalog/schema/Cargo/worker 변경은 root 협의 없이 하지 않는다.

## 인수 조건

실제 egui Context에서 오류 설명이 입력 직후·Apply 이전에 위치하고 수정/빈값에서 해제되는지 검증한다. 오류 발생 시 관련 입력+설명을 함께 공개하며 정상 입력 help와 focus/selected 의미를 보존한다. Windows Light100/Dark200 provider/PNG 검증을 별도로 실행한다.

## 검증과 handoff

근거는 current-app-semantics-native-network-syntax-dark200-800-v6-20261013/gui/stage-005-fill/input-network-setter.png와 같은 stage JSON의 actual258UTF16 value이다. 현재 syntax/거부/지우기/Off/Auto 동작 자체는 통과했지만 오류 설명의 시각적 공개는 미완료다. source에는 아직 적용하지 않았다.

## 2026-10-13 통합 검증

root가 오류 설명을 input 직후·Apply 이전으로 옮기고 입력+오류 영역을 함께 scroll 공개하도록 통합했다. 실제 Context 회귀는 leading 480px/400×360 viewport에서도 관련 영역이 공개되는지 확인하며 수정/빈값의 오류 해제·기존 semantics 회귀도 성공했다. Windows 전체 fmt/clippy/test/release 1278/0/30 및 root 원로그/655 source/artifact/lock 대조 PASS다. 이전 Dark200 PNG는 수정 전 증거로 보존한다. 새 Windows Light100/Dark200 실제 PNG 검수는 다음 단계이며 현재 완료로 쓰지 않는다. [원검토](../validation/evidence/windows-msvc-ui-feedback-library-worker-whole-20261013/root-review.json).
