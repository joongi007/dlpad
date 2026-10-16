# S00: 파일 목록의 선택 정보 간격과 안정적인 행 위치

- 상태: done / 구현·회귀·private 선택 기하
- 분류: ui-polish
- 우선순위: P0
- 요구사항: R03, R08, R11
- 담당/수정 범위: advanced_units — 분리된 accessible_library.rs 후보; root — catalog/interface·통합·전체 검증·문서
- 선행 작업: v22 실제 10,000개 목록의 여섯 PNG 및 제목·복사 공개 API 실행 검증

## 사용자 결과

선택한 파일의 이름·크기·수정 시각을 한 줄의 선택 정보 영역에서 보여준다. 선택 전 빈 메타데이터 두 줄을 위해 남겨 둔 큰 간격을 줄이고, 첫 선택 뒤에도 행 위치가 바뀌지 않아 더블클릭 대상이 유지되게 한다.

## 계약과 범위

실제 목록 PNG와 현재 코드의 빈 label 두 개를 근거로 검토한다. 고정 높이의 한 선택 정보 영역을 사용하며 긴 제목은 truncate하고 전체 경로/정보는 접근성 설명과 기존 상세 동작에서 확인한다. 기존 worker·DB·선택·double-click/Enter·복사·OS 열기·취소·catalog를 유지한다. 신규 dependency/schema/Cargo 변경은 없다. 후보는 별도 소스 복사본에서만 작성하고 root가 현재 설치본과 구분하여 통합한다.

## 인수 조건

미선택/선택에서 정보 영역과 첫 행의 y 위치가 같다. 긴 한글·emoji 이름과 800/960폭·100/150/200%에서 행 및 footer가 겹치지 않는다. 실제 Context로 첫 클릭→선택→두번째 클릭의 동일 대상 미리보기 요청을 확인한다. 새 전체 fmt/clippy/test/release와 private 목록 캡처로 인수하며 v22 검증을 새 binary에 소급하지 않는다.

## 검증과 handoff

현재는 실제 10,000개 목록에서 toolbar 아래 약 두 줄 이상의 빈 메타데이터 공간을 확인하고 독립 후보를 준비하는 단계다. 기존 두 blank label은 선택 전후 행 이동을 막기 위한 것이므로 단순 삭제로 double-click 좌표를 바꾸지 않는다. 새 후보 구현·시험·통합·새 설치본은 아직 미완료다.

Root가 frozen candidate SHA 전수·원Live/S 파일 동일성을 확인한 뒤 S 한 파일만 overlay했다. 실제 Windows 명령: `.local-tools/session-recovery-20261013/root-summary-phase-integration-v1/run_isolated.py summary-only-green-v1 20261014`. 원 Library module 결과 **48 PASS / 1 FAIL / 0 ignored**, 새 actual_selection_summary 시험의 consistency_row label lookup에서 None을 관측했다. strict clippy는 선행 실패로 실행하지 않았다. [원자료](../validation/evidence/managed-tool-isolated-summary-only-green-v1-20261014/result.json). 기존 v1 후보/원실패를 보존하고 새 namespace에서 scene·label·viewport 원인을 진단한다. Live/현재 v22 설치본은 변경하지 않았다.

원 진단 v3 실행은 첫800px/100%/Light의 LIBRARY_SELECT_HELP 검색에서 실패했다. 실제 row318..350은 bodyclip87..563 내에 있어 당시 실패가 200% offviewport 때문이라는 가설을 기각했다. Root가 원 egui response.rs884–887의 Role::Label→value 계약을 직접 확인했다. 새 greenv4는 readonly Label만 role+value exact 조회하며 Button label·기하·배율·sameclick/Enter/gate 인수를 유지한다. 제품 렌더러 prefix는 v1과 byte 동일하다. 현재 greenv4 실제 Windows 전체 Library 회귀를 실행 중이다. 원 실패를 삭제하거나 성공으로 승격하지 않는다.

### 최종 구현 인수

고정32px·안정적인ID와행위치는v23에통합했고 왼쪽정렬/metadata최대45% 파일명우선은v24에서수정했다. 실제Library50회귀와전체Windows1538/0/42 통과. v24 private 여섯장면은 UIA.TogglePattern선택Off→On,전후원행bounds/runtimeID동일,두열32px/nooverlap,원8MiB합성파일SHA/nativeID보존과자연app0/Job0를 Root독립대조했다. Root는여섯selected 원픽셀을직접확인했으며전체24PNG팀검수는후속보고서로연결한다. 실제물리더블클릭/IME/reader는미검증이고 Context의sameclick/Enter회귀와구분한다. [최신기하](../validation/evidence/current-library-selection-summary-native-v4-20261014/root-review.json), [v24설치후속](S00-windows-v24-integration-20261014.md).
