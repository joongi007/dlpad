# S00: 짧은 작업 이력과 입력 보호 도움말 통합

- 상태: prepared, 최종 격리 clippy 진행 중
- 분류: integration
- 우선순위: P0
- 요구사항: R07/R08/R13 및 S00 실제 UI 검수
- 담당/수정 범위: Root가 accessible_history.rs/catalog.rs/list_search.rs/accessible.rs/ui_components.rs 5파일을 통합한다. 하위 에이전트는 각 scratch만 작성했다.
- 선행 작업: S03-history-status-compact, S00-help-placement-protected-input, 현재 Windows 전체28/공개 구간 읽기 검증 종료

## 사용자 결과

작업 이력은 짧은 상태와 필수 보존/재개 안내를 본문에 표시한다. 도구 경로 도움말은 가능한 공간에서 입력·찾아보기 영역 바깥에 표시하며, 공간 부족 시 긴 설명을 스크롤하여 읽는다.

## 계약과 범위

공유 Search 일회성 공개 bool과 help_button/show_help 내부 API는 Root가 검토한다. 원 Label/도움말/재개 ID와 실제 실행 gate를 유지한다. dependency/lock/schema/worker/timer 변경은 없다. 추가 viewport별 기하 계산과 설명 popup의 bounded ScrollArea만 허용한다. 배치 수정의 기존 기본 help(None) 범위는 유지하며 도구 일반 popup의 이웃 버튼 가림은 별도 후속이다.

## 인수 조건

실제 selected History12/Help11·경로2·찾기배치1 회귀와 수정 fixture Help2를 원로그로 확인한다. 전체 fmt/strict clippy/test/release를 새 소스에서 실행하고 원3PE·lock·소스를 독립 대조한다. source/PE가 연결된 private GUI에서 실제 상태/필수 안내·도구 경로 Help를 직접 캡처 검수한다. 전체 GUI/물리 입력·IME·reader/P0를 단위 검사로 완료 처리하지 않는다. 새 검수 후보 설치 파일은 이 검사 뒤 생성한다.

## 검증과 handoff

기존 RED/잘못된 fixture 전제 실패를 원namespace에 보존했다. 현재 수정된 Help 기하2와 History12 및 관련 회귀가 실제 통과했으며 최종 격리 clippy 검사를 기다린다. 원 source28·사용자 검수 ZIP28은 그대로다. Root adoption helper는 exact source28·선택 원로그·각 담당 파일 SHA·최종 candidate 전후 불변을 확인한 뒤 5파일만 채택하도록 준비했다. Windows 외 인수·공개 서명/배포·업데이트·clean OS는 완료가 아니다.


Root가 source28·원3PE·각 담당파일·실제 원로그·최종 S 불변을 확인해 5파일을 Live에 채택했다. receipt는 `.local-tools/session-recovery-20261013/root-ui-v29-integration-v1/receipt.json`이다. 새 whole29 fmt/clippy/test/release 실행 중이며 성공 결과를 아직 가정하지 않는다.

Root 새 whole29 완료: fmt/strict workspace all-targets clippy/test/release 실제0, 1,613 PASS/0 FAIL/42 ignored·43harness·source674이며 원로그/current3PE/lock을 독립 대조했다. CPU10,321,920B(+6,144B), CLI978,432B, installer4,432,896B다. native 화면/최신 ZIP은 후속 진행이며 전체P0로 확대하지 않는다.
