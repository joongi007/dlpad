# S00: 파일 목록 동작 안내의 행 정렬

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root — crates/app/src/accessible_library.rs, 이 기록과 실제 화면 검증
- 선행 작업: 전체 경로 창 키보드 복귀 캡처

## 사용자 결과

선택한 영상 확인과 경로 확인 버튼 아래에서 안내 문장을 한 흐름으로 읽는다.

## 계약과 범위

현재 horizontal_wrapped 안의 안내가 남은 좁은 폭에서 파일을만 첫 행에 표시하고 나머지를 다음 행에 넘기는 실제 Dark800200 캡처를 확인했다. 버튼은 wrapped 행을 유지하고 안내는 별도 행에 배치한다. 선택·가상화·ID·preview 정책은 유지한다.

## 인수 조건

800200 Light/Dark에서 버튼·안내·목록·footer가 겹치지 않으며 문장 첫 단어가 버튼 옆에 고립되지 않는다. 기존 실제 Panel keyboard 복귀 회귀를 보존한다.

## 검증과 handoff

source 및 실제 restartDark restored PNG에서 문제를 확인했다. 아직 수정하지 않았으며 현재 whole 독립 검토 종료 뒤 통합한다.

- emoji whole57080 및 독립11889 exit0 후 버튼 행 밖으로 안내를 옮겼다. 새 의존성/선택/DB 변경 없음. 실제 캡처와 기존 Panel 회귀 검증은 이어서 수행한다.

- 전체1251 및 독립77085 exit0: 1203성공/0실패/30ignored. CPU9,780,224B/SHA `13c79ec63557183ab4a11e325a01f96a3a2d680e589eff32e88afec67d7f2242`, emoji whole 대비 크기 변화 없음. 실제 native 배치·glyph·긴경로 스크롤 시험은 이어서 수행한다.
