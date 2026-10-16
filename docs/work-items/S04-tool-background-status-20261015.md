# S04: 도구 확인과 설치의 진행 상태 구분

- 상태: in_progress / 원인 확인·scratch 구현 중
- 분류: bug
- 우선순위: P0
- 요구사항: R05/R08, 사용자 설정 도구 진행 표시 피드백
- 담당/수정 범위: p0_audit scratch accessible_tools.rs; Root accessible.rs/catalog.rs 통합·검증
- 선행 작업: source30 native/package 종료 후 source 통합

## 사용자 결과

설치된 도구를 조회할 때 하단에 확인 중이라고 표시하고 버전 확인·설치·검증·삭제와 구별한다. 조회를 다운로드나 설치 작업으로 오인하지 않게 한다.

## 계약과 범위

원인은 accessible.rs status가 tools.busy()에 공통 TOOL_BACKGROUND_WORK를 표시하는 것이다. Panel::background_status()로 실제 pending 작업에 맞는 중앙 catalog 문구를 제공한다. 기존 busy/cache/명령 gate·취소·worker 수명은 유지한다. 같은 파일의 동시 수정은 하지 않는다.

## 인수 조건

조회/새 버전/설치/검증/삭제/선택별 진행 표시, queue 실패·완료·오류·stale receipt 뒤 상태 복구, idle시 표시 제거를 확인한다. cached 목록은 조회 중 유지한다. 실제 다운로드나 다른 작업의 상태를 잘못 덮지 않는지 Root 통합 검증한다.

## 검증과 handoff

현재 코드 경로에서 원인을 확인했다. 새 API 구현·실행 결과는 아직 없다. 원 busy gate 제거로 해결하지 않는다.

## 실제 격리 검증

`root-ui-next-stage-v1/run.py ui-next-selected-v1 20261015`에서 actual_background_4개가실제통과했다. stale receipt·실패/정상복귀·queue거절·기존cache유지를확인했다. 후보본문과하단이같은작업라벨을쓰도록Root가accessible.rs/catalog를S에통합했고strictclippy실행중이다. Live/사용자ZIP에는아직미포함이다. 원 busy/종료gate는유지한다.

첫strictclippy는새테스트의불필요한동일타입`.into()`2개로실패했다. Root가S에서해당2개변환만제거했고새전체strict검사로확인한다. 원실패`managed-tool-isolated-ui-next-clippy-v2-20261016`는보존했다.

Root가실제S검사·원소스674/현재12파일·lock불변을대조해source31에채택했다. source675의새Windows전체검사실행중이며현재사용자v30 ZIP과구별한다. 격리최종strictworkspace/all-targetsclippy는0이다.
