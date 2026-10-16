# S00: 구간 선택 탐색과 범위 조절 UX

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R02, R08, R11
- 담당/수정 범위: windows_auto — media_timeline.rs, media_workspace.rs 후보; root — catalog/통합
- 선행 작업: v20 사용자 UI 검수

## 사용자 결과

기존 시간 직접 입력 기능을 유지한다. 중복 전체 영상 표시와 선택 범위 표현을 정리하고 타임라인 클릭 탐색/범위 핸들 드래그/hover커서를 실제 조작별로 설명·구별한다. 합성media fixture/가상 화면을 사용한다.

## 계약과 범위

Rust native UI와 기존 소유권/취소/CloseWait를 유지한다. 각 후보는 별도 namespace만 수정하고 제품 소스 채택은 root가 조정한다. 공유 catalog는 root만 수정한다.

## 인수 조건

기존 시간 직접 입력 기능을 유지한다. 중복 전체 영상 표시와 선택 범위 표현을 정리하고 타임라인 클릭 탐색/범위 핸들 드래그/hover커서를 실제 조작별로 설명·구별한다. 합성media fixture/가상 화면을 사용한다.

## 검증과 handoff

구현·실제 시험 및 시각 검수 결과를 기록한다. 미실행은 통과로 보고하지 않는다.

## 2026-10-14 통합 checkpoint

타임라인6/0와 reset identity2/0, Main/옵션 회귀73/0(3 ignored)는 격리 실행 범위다. 전체축 중복 overview 제거, seek와 범위 handle의32px hit area/서로 다른 커서, 시간 입력을 timeline 옆에 배치한 후보를 v21 개발 소스에 통합했다. 실제 새 native200%/IME 입력 인수는 별도 남는다.
