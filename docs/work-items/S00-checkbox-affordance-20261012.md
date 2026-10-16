# S00: 체크박스의 시각적 구별

- 상태: implementing
- 분류: bug
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: 주 담당; crates/app/src/ui_components.rs, crates/app/src/accessible.rs, 이 문서. diagnostic_inputs는 /tmp 후보 담당.
- 선행 작업: current-ui-tools-action-first-20261012 다운로드 캡처 직접 검토

## 사용자 결과

미리보기 생략 여부를 선택하는 체크박스를 라디오 버튼과 구별되는 모서리가 있는 사각형으로 표시한다.

## 계약과 범위

실제 14px indicator에 공통 radius6이 적용되는 원 painter를 확인했다. native Checkbox를 유지하고 이 컴포넌트만 radius3으로 제한한다. 기존 frame간 ID·클릭·disabled·Tab focus·AccessKit role/toggled와 sibling button 스타일이 유지돼야 한다. ui.scope 추가로 과거 autoID와 완전 동일하다고 주장하지 않는다. 의존성·schema·worker 변경 없음.

## 인수 조건

Light/Dark 실제 paint에 사각형 indicator를 표시하고 실제 클릭으로 토글된다. disabled는 반응하지 않고 키보드 포커스와 접근성 Checkbox 의미가 유지된다. sibling 버튼 radius6은 바뀌지 않는다. 실제 가상 화면 배율별 캡처와 전체 검사를 수행한다.

## 검증과 handoff

실제 다운로드 캡처에서 원형처럼 보이는 표시를 주 담당이 확인했다. 먼저 기존 native Checkbox painter로 같은 회귀를 실행한다. 후보 rustfmt는 실행했으나 실제 Windows RED/GREEN은 아직이다.

실제 Windows RED: 회귀1개 FAIL/exit101, 원14px indicator의 CornerRadius6이 예상 사각형radius3과 다름을 actual painter로 검출했다. 자연 Job 종료·원 사용자 화면 보존 확인. 주 담당은 기존 Checkbox에 한정한 style scope와 production call을 통합했다. GREEN 및 native 재캡처는 아직이다.

실제 Windows GREEN1PASS/0FAIL/exit0: Light/Dark 원 painter radius3·sibling radius6, 실제 pointer click toggle, disabled 무반응, 실제 Tab focus·AccessKit CheckBox/toggled·frame간 ID 안정성을 확인했다. 합성 Context 입력이며 물리 마우스/IME 및 native PNG 인수는 아니다. 전체 workspace 검사와 새 캡처를 이어간다.
