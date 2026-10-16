# S00: 키보드 포커스 도움말 공개

- 상태: completed (scoped)
- 분류: UX
- 우선순위: P2
- 요구사항: R06, R08
- 담당/수정 범위: root — 공유 ui_components.rs/관련 호출 통합; advanced_units — /tmp 후보
- 선행 작업: 공통 P0 native입력·중앙UI catalog 구현, 현재목록identity batch source동결

## 사용자 결과

키보드·마우스로 동일한 도움말과 입력 문맥을 이해하고 편집 중 대상을 유지한다.

## 계약과 범위

기존 hover-only tooltip과 DESIGN keyboard focus 계약을 실제Context로 확인한다. 기존Response/ID/편집caret/값/disabled/hover동작을유지하고 필수오류를tooltip으로숨기지않는다. 후보준비와정적판정만으로버그/인수를선언하지않는다.

## 인수 조건

실제관찰/명령/소스pin·대상위젯별검증상태를기록하고 actualRED/GREEN·시각·물리입력을구분한다. 불필요한실험반복·사용자Defaultdesktop시험·새의존성·schema/Cargo변경을하지않는다. 제품수정채택은root가통합검증한다.

## 검증과 handoff

/tmp 병렬준비중이며 실제제품수정/인수는미완료이다. 현재시험의sourceguard가끝날때까지제품소스를바꾸지않는다.


실제original hover helper에서 TabfocusACK가있고pointerhover없는Lightpath의tooltipText0개로0/1 RED를확인했다. 현재제품caller/helper는변경하지않고test-only원source를증거에보존한뒤shared원제품을복원했다. 다른위젯hover와focusedhelp의arbitration을검증하기전원hover계약을깨지않도록후보채택을보류한다. /tmp변경중후보와roottestedcopy의동일성guard거부로실패한adoption과뒤oldalias재실행은별도setup기록이며GREEN으로취급하지않는다. [실제RED](../validation/evidence/windows-msvc-keyboard-context-help-baseline-20261013/root-failure-review.json), [보류](../validation/evidence/windows-msvc-keyboard-context-help-baseline-20261013/root-deferred-adoption.json).


source17동결을유지한rootowned별도Rust export에서제안된강제focus tooltip이다른nativehover도움말을가리는actualContext RED(0/1,101)를확인했다. focusedcase의16boundedhoverID를Context lock밖에서확인하고다른interactivehover를우선하는candidate는교차hover와Tab/caret/Enter/disabled2filter각1/0/0으로통과했다. 제품helper/caller채택은아직없으며독립namespace source와original17/3PE/lock/SQL pin보존을구분했다.


2026-10-13 root 통합: 원 source freeze 종료 후 제품 코드에 채택했다. 완료 인수는 새 전체 Windows 검증·설치본 검수 이후다. [격리 actual Context 2/0 독립 검토](../validation/evidence/keyboard-context-help-isolated-acceptance-20261013/green/root-review.json). 경로 입력 및 도구 찾기 caller에 연결했으며 필수 인라인 오류·응답 ID·커서 정책은 유지한다.


v18 통합 인수: 네 Windows 실제 명령 1327/0/33·43harness·source657, 실제 현재3PE·lock·원로그 독립 대조, 새 ZIP·정상 설치/앱 자연 종료·변조 거부·canonical27ID·앱63 및 설치8 PNG 직접 검수를 완료했다. 공유 Cargo/schema/의존성 변경은 없다. [현재 whole](../validation/evidence/windows-msvc-keyboard-diagnostic-help-whole-20261013/root-review.json). 추가 Settings distinct-commit semantic FAIL은 별도 후속 작업이며 이 scoped 성공으로 덮지 않는다. 전체 P0·물리 IME/reader/OS DPI·full UI 인수는 미완료다.
