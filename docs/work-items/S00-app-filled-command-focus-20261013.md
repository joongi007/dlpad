# S00: 앱 공통 filled 명령의 포커스 대비

- 상태: completed (scoped)
- 분류: UX
- 우선순위: P0
- 요구사항: R06, R08
- 담당/수정 범위: root — 공유 ui_components.rs 통합/실제검증/본 기록; advanced_units — /tmp 실제Context 회귀·최소 후보
- 선행 작업: v15 native설치/200%scroll source동결 검증 종료

## 사용자 결과

앱의 기본·삭제 명령도 키보드 포커스 선을 배경과 구분하며 설치기와 같은 filled 역할 규칙을 사용한다.

## 계약과 범위

root가 ui_components.rs121–137 공통command를 확인한 결과 Light primary 배경/안쪽focus선 모두ACCENT_SOLID이다. 실제Context로 이1:1패턴을 재현한다. filledPrimary/Danger만 기존WHITE로 선을 그리고 neutral/Ghost·disabled·선택·ID/Response·handler·폰트·치수·painter상한·Cargo/schema는 유지한다. source정적사실을 물리OS keyboardfocus 성공으로 주장하지 않는다.

## 인수 조건

실제primary/danger Context의 양테마·hover/press/idle focus 선/배경으로 독립 대비3:1 기준의 RED/GREEN을 확인한다. 실제focus없음/disabled 상태와handler/ID를보존한다. 단순return-equality 구현복제시험은추가하지않는다. 통합후Windows전체4검사/원로그/새후보actualGUI를 확인한다. physicalfocus/IME/reader는별도다.

## 검증과 handoff

v15의 설치기focusedfilled 역할은 별도GDI3개로 수정·검증됐지만 앱공통역할은 이새작업의 범위다. 제품common API/source는root만변경하며 현재nativecollector중에는변경하지않는다. 후보준비와정적사실만으로완료를기록하지않는다.


실제원Context은LightPrimaryidle outline과fill 모두#006FBEFF로대비1:1을0성공1실패(Rust101/자연Job0)로재현했다. 2줄filledrole WHITE 적용후 실제1회귀가통과했다. Light/Dark×Primary/Danger×idle/hover/pressed12paint case의독립대비≥3, 실제Tab focusACK·hover/pressACK·releaseclick·focusoff/disablednonactivation·원Response/node/geometry를확인했다. root source/원로그/privateJob0를대조했다. 전체Windows/release/새후보는진행중이다. [RED](../validation/evidence/windows-msvc-app-filled-focus-baseline-20261013/root-failure-review.json), [GREEN](../validation/evidence/windows-msvc-app-filled-focus-green-20261013/root-review.json).


v16같은source의원fmt/clippy/test3명령과새release복구를독립대조해전체4검사를확인했다(1,309/0/33·43harness/source656). 중단없는단일whole로주장하지않고원143을보존한다. 새63PNG팀직접검수·currentZIP·direct정상설치/Run outer0·변조거부outer0·독립27ID·Dark200하단5명령 actualscroll을root통합검증했다. 원앱focus 실제RED/GREEN과새artifact·원source/lock/GNU를연결하며전체물리UI/P0/정식배포는미완료다. [전체](../validation/evidence/windows-msvc-app-filled-focus-whole-recovered-20261013/root-review.json), [63화면](../validation/evidence/current-app-filled-focus-app63-20261013/root-team-visual-review.json), [설치](../validation/evidence/windows-msvc-ui-cross-surface-v16-20261013-command-happy/root-review.json).
