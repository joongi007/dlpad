# S00: v26 private UI 검수

- 상태: proposed
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R05, R08, R10
- 담당/수정 범위: advanced_units의 B/ui-feedback-v26-private-v1 helper와 이 문서만
- 선행 작업: 새 v26 whole 및 Root 독립 검토; 검색 UI 최종 source 통합

## 사용자 결과

새 도구 상태 표시와 캐시 탭 복귀가 작은 창과200%에서도 읽히는지 실제 Windows private 화면으로 확인한다. 검색과 전송 설명은 실제 상태가 있는 별도 검수 흐름으로 확인한다.

## 계약과 범위

현재 helper는960 Light/Dark100 및800 Light/Dark200 네 owned scope/12PNG를 준비한다. 실제 UIA Toggle/Selection/Invoke 지원과 selected ACK, RuntimeID, 설치상태 준비를 관찰한다. 기존 private desktop/소유 Job/원 input desktop/자연 종료/source/현재3artifact/원proof pins를 유지한다. UI thread I/O·외부 다운로드·실제 사용자파일·클립보드·global input·visible desktop fallback은 없다.

## 인수 조건

새 exact whole/root-review/current source만 허용한다. 준비 관측은20초 상한이며 unknown 상태를 ready로 보지 않는다. 캐시 표시와 zero worker I/O를 혼동하지 않는다. 실제 지원되지 않은 패턴 또는 중복 selector는 실패 원자료를 보존한다. 검색/도움말/진행률은 아직 이 도구 slice에 없으므로 미검증이다. 최종 검색 source와 합성 진행 fixture가 준비된 뒤 후속 helper에 연결한다.

## 검증과 handoff

Python AST만 통과했다. C# compile/native/직접 PNG 검수는 미실행이다. exact argv와 원본/후보 SHA, 차이는 B/ui-feedback-v26-private-v1에 보존한다. 제품/S/Cargo/기존 evidence는 수정하지 않았다. 실행은 Root whole 승인 뒤 Root가 담당한다.
