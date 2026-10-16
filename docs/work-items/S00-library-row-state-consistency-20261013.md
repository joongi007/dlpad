# S00: 파일 목록 선택 행의 hover·누름 일관성

- 상태: in-progress
- 분류: UX
- 우선순위: P1
- 요구사항: R01, R08, R13
- 담당/수정 범위: windows_auto — /tmp/dlpad-library-row-consistency-candidate-20261013 후보만; root — crates/app/src/accessible_library.rs 채택·통합·검증 및 본 문서
- 선행 작업: 현재 frozen source의 실제GUI·기본18구간 비용 검증 종료

## 사용자 결과

선택한 파일 행에서도 마우스 hover와 누름을 구별하며 선택 표시·키보드 포커스를 유지한다.

## 계약과 범위

기존 공통 ui_components::choice를 재사용하는 단일 화면 후보다. 전체폭·말줄임·32px 가상행 stride·경로별 stable ID·native selectable semantics와 click/Enter 정책을 유지한다. core/schema/Cargo/공유 컴포넌트 변경은 하지 않는다. 같은 파일은 root만 최종 채택한다.

## 인수 조건

실제 LibraryPanel render에서 선택 idle→hover→pressed 상태색 구별, 키보드 포커스/선택 상태, disabled 조작 거부, 긴 경로와 가상화 계약 보존을 확인한다. 실제 Windows 화면·provider/물리 입력 검증은 별도로 기록한다.

## 검증과 handoff

원본 raw Button::selectable의 interact_selectable은 선택색을 hover/pressed에서도 같은 값으로 대체한다. 현재는 source audit와 /tmp 후보 준비 단계이며 구현·실행 완료가 아니다. 최신 빌드 비용 검증 후 후보 채택 여부를 결정한다.

## 2026-10-13 통합 검증

root가 후보를 실제 accessible_library.rs에 통합했다. 실제 LibraryPanel Context 회귀가 선택 idle/hover/press/focus/disabled 색과 조작 거부·경로 ID·긴 행 geometry를 확인한다. Windows 기본 workspace fmt/clippy/test/release 및 root 원로그/655 source/artifact/lock 대조 PASS, 전체 1278/0/30이다. 해당 회귀는 CPU/GPU 두 harness에서 실제 성공했다. native 화면·물리 누름은 아직 이 새 snapshot에서 미검증이며 전체 UI 인수로 확대하지 않는다. [원검토](../validation/evidence/windows-msvc-ui-feedback-library-worker-whole-20261013/root-review.json).

현재compact0093 기본CPU에서one3bytes file row nativeProvider Off→On/sameID/path exactreadback/modalclose→sameRowOn을실행하고root4PNG/원로그·pins독립검토PASS. 파란선택fill/포커스edge/선택복귀를직접보았다. 해당결과는supportedUIA Toggle/Invoke한정이며OSWMpressed 및물리마우스·reader·Dark/확대/10k/바쁜행을대체하지않는다. [국소인수](S00-library-provider-row-private-20261013.md).
