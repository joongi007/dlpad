# S00: Settings 한글 확정 입력과 선택 영역 검증

- 상태: in-progress
- 분류: validation
- 우선순위: P2
- 요구사항: R06, R08
- 담당/수정 범위: root — 소유 Rust export·vendor/egui/src/widgets/text_edit/builder.rs 통합·실제 Windows 검증·문서; windows_auto — /tmp accessible_settings.rs 및 vendor 후보; advanced_units — /tmp ime_cancel_regressions.rs 강화 후보
- 선행 작업: 현재18 전체 Windows 검증 통과와 전용 debug Cargo target 독점

## 사용자 결과

선택한 한글을 확정 입력으로 바꿀 때 문자열·커서가 보존되고 확대 화면에서도 입력과 커서가 잘리지 않는지 확인한다.

## 계약과 범위

실제 Candidate의 Settings page·input provider를 사용하고 소유 임시 설정만 사용한다. 100/200 Context zoom, Light/Dark, 두 directed selection으로 검사한다. 물리 OS IME·실제 Scale Combo 선택과 구분한다. 기존 composition cancellation 정책·제품 source·vendor·schema·Cargo를 바꾸지 않는다. source-only 가설을 제품 버그로 확정하지 않는다.

## 인수 조건

실제 input Focus/Expand/Text/preedit/nonemptyCommit ACK, 정확한 값·선택·ID·glyph/caret/body clip을 확인한다. setup/semantic/cleanup 실패를 구분하고 원 실패를 보존한다. 성공 시에도 물리 IME 완료로 보고하지 않는다. 소유 App close worker gates 확인 없이 fixture 데이터를 삭제하지 않는다.

## 검증과 handoff

후보 준비만 완료했다. 새 소유 export에서 root가 실행할 예정이며 아직 actual PASS/RED가 아니다. 현재18 제품 source와 release/설치 기록은 유지한다.


actual Settings distinct preview 조→commit 한국어는 격리 Windows 0/1·Rust101/private naturalJob0 실패였다. Focus/preedit와 paint setup 이후 actual node A조Z가 남았다. [원 실패](../validation/evidence/settings-ime-positive-commit-isolated-20261013/root-failure-review.json). 원 raw의 방향 정보가 없어 방향을 추정하지 않는다. 계측으로 좁히고 cancel/외부변경/size/undo/선택 이동 보호를 유지하며 최소 vendor 후보를 실제 시험 중이다. 제품 vendor/Settings는 아직 미채택이다.


계측 baseline에서 Light100 정방향 확정은 PASS, 역방향은 focus 유지 후 A조Z가 남는 semantic FAIL을 확인했다. 강화 기존9개는8/1로 masked commit을 분리했다. 첫 최소 preview-anchor 후보는100% 네 경우를 고쳤으나200% 첫 field paint setup0/1 및 Disabled 뒤 lateCommit 역방향 새 회귀8/1로 인수를 거부했다. movement guard는1/0이며 제품vendor를 채택하지 않았다. 성공 rollback 후 originalanchor를 함께 복원하는 새 후보와 실제 focus scroll의 bounded frame 안정화 fixture를 준비 중이다. [후보 인수 거부](../validation/evidence/settings-ime-anchor-green-v1-isolated-20261013/root-incomplete-classification.json).


두 번째 후보는 actual4필터12/0/0, positive8·moved-selection2, 기존강화9와공통취소1을 모두 통과했다.200%는원scroll animation17빈프레임에서original32px/fullbodyclip가안정화됐고최대32조건을유지했다. 새fixture는data/media를소유temp의형제로만들고plainpath설정을seed해실제Documents를사용하지않는다. root가candidate namespace·원18소스 보존과원로그private/Job0를검토한뒤Windowsrustfmt를완료하고vendor/Settingstest/강화9회귀를제품에채택했다. [actual GREEN](../validation/evidence/settings-ime-anchor-green-v2-isolated-20261013/root-review.json). 새전체/설치검수는진행중이다.

## 재시작 후 실제 검증 checkpoint (2026-10-13)

새 `windows-msvc-settings-ime-commit-whole-resumed-20261013`에서 fmt/clippy/test/release 모두 실제 통과했다. 기본 테스트는 1331 PASS, 0 FAIL, 33 ignored, 43 harness이며 source inventory는 657이다. 기존 중단 whole은 미완료 기록으로 보존한다. 현재 installer GDI ignored 3개도 별도 실제 통과했다.

현재 v19 ZIP 생성과 원본 7파일 hash/CRC 검토 완료. 가상 데스크톱의 정상 설치·실행 및 소유 EXE 1바이트 변조 거부 collector 종료 완료; 원시 host/source/원본 pin은 root가 독립 재검토했다. 앱63PNG 생성과 7PNG 직접 검토 완료, 나머지 시각 검토는 별도 담당 진행 중이다. 이 checkpoint는 전체 UI/UX·물리 IME·실제 OS DPI·정식 배포 완료를 뜻하지 않는다. 최신 canonical27/긴경로/CI/업데이트 흐름은 후속 인수가 남아 있다.
