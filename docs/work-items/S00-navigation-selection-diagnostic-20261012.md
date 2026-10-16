# S00: 탐색 선택 표시의 상태 일치 진단

- 상태: in-progress
- 분류: quality
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: 주 담당 accessible.rs 테스트와 이 문서
- 선행 작업: 실제 탐색 선택 상태의 첫 프레임 캡처 검토

## 사용자 결과

선택한 화면과 선택 표시의 지속적 불일치를 검출한다.

## 계약과 범위

실제 Candidate에 접근성 Click을 전달하고 현재 화면·paint·선택 상태를 비교한다. 부분 AccessKit 업데이트는 누적하고 제거된 노드는 제외한다. 클릭 직후 일시적 차이를 강제로 실패시키지 않는다. production 정책 변경 없음.

## 인수 조건

후속 두 실제 프레임에서 화면과 선택 표시가 일치하며 부분 업데이트 누락/제거 경계가 검사된다. 네이티브 화면과 물리 입력은 별도 검증한다.

## 검증과 handoff

주 담당 통합 후 검증 진행 중. UI 전체 인수는 미완료다.

첫 통합whole는fmt통과 뒤clippy E0433/E0425로 실패했다. 주 담당이진단을Candidate tests대신bounded_font_tests에붙여Fixture/wait_finished범위를잘못사용했다. 원본실패를보존하고기존Candidate tests안으로옮겨새검증한다. 설치문구lookup2곳누락도동시에고쳤다.
