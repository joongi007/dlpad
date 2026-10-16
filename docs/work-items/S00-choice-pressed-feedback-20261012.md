# S00: 탐색 버튼 눌림 피드백

- 상태: implementing
- 분류: bug
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: 주 담당 통합; crates/app/src/ui_components.rs, 이 문서. 하위 에이전트는 /tmp 후보만 준비.
- 선행 작업: pointer-state-synthetic-egui-20261012 실제 실패 확인

## 사용자 결과

탐색 버튼을 누르고 있는 동안 호버와 구별되는 배경을 보여준다. 선택 및 키보드 포커스 표시는 유지한다.

## 계약과 범위

Rust native UI 유지. 선택되지 않은 버튼의 hover와 pointer-down에 동일 neutral fill을 사용한 것을 원본 소스에서 확인했다. Light/Dark 색을 좁은 범위로 수정하며 Cargo·schema·의존성 변경 없음.

## 인수 조건

실제 egui pointer move/press/release에서 idle·hover·pressed·selected가 구별되고 disabled는 반응하지 않는다. 가상 desktop 실제 캡처의 hover와 pressed 변화가 모두 관측된다. 합성 egui 입력을 물리 OS pointer 검증으로 표현하지 않는다.

## 검증과 handoff

원 시험 exit1: hover_changed_pixels2252, pressed_changed_pixels0, settings selection ACK true. 원본 Job 자연 종료/final active0/private desktop closed/input desktop unchanged. 주 담당은 tab-pressed.png 직접 확인. 검수용 feature whole는1149/0/30, 일반 whole1129/0/30은 이 수정 이전 범위다. package62316 종료0 확인 후 source freeze 해제. 후보 검토·회귀·전체 검사·재캡처는 아직이다.

수정 전 실제 Windows 회귀 RED 확인:2 tests 중 disabled1PASS/hover-press1FAIL, exit101. 원본 동일fill을 검출했다. 검토한 production 분기만 적용: neutral pressed Light212/Dark80, selected pressed는 기존 selected idle palette 사용. 선택/포커스/disabled·의존성 유지. GREEN 및 native 재캡처는 아직.

첫 GREEN 실행은 hover/press 차이·release click을 통과한 뒤 release.has_focus 가정에서 FAIL했다. 실제 키보드 포커스 요구를 마우스 release와 혼동한 시험을 수정해 실제 Tab 이벤트로 focus 및 painter outline을 검증한다. 생산 UI 포커스 강제 요청을 추가하지 않았고 pressed assertion은 유지했다.

실제 Windows corrected GREEN PASS:2 passed/0 failed/0 ignored, exit0. Light/Dark·selected/unselected pointer hover/press/release painterfill 차이·click·identity·hover복원·PointingHand·실제Tabfocus outline·disabled무반응 확인. native screenshot 재검증 및 전체4검사는 아직이다.

## 수정 후 전체 검사와 실제 캡처

windows-msvc-choice-pressed-feedback-whole-20261012에서 fmt·clippy·test·release 모두 자연 종료0, 1,133 passed/0 failed/30 ignored를 독립 대조했다. pointer-state-synthetic-egui-pressed-feedback-20261012에서 hover2252px·pressed2266px 변화와 Settings 선택 ACK를 확인하고 주 담당이 hover·pressed·selected PNG를 직접 검토했다. 합성 egui 입력의 시각 상태 인수이며 물리 OS pointer·caret·전체 UI/UX 인수는 아니다.
