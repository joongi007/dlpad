# S00: 검색 결과·도움말·입력 문맥 통합

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R03, R07, R08, R10
- 담당/수정 범위: Root 통합, crates/app/src/{accessible.rs,ui_components.rs,list_search.rs,accessible_library.rs,accessible_history.rs}, vendor/egui/DLPAD_PATCH.md와 관련 검증 문서
- 선행 작업: v27 전체 검사·설치 검수, 검색·도움말·입력 문맥 개별 work-item의 실제 RED/GREEN

## 사용자 결과

빈 파일 검색 안내와 명시 이력 검색 결과를 한 번 드러낸다. Escape로 설명을 닫으면 같은 focus/hover에 다시 뜨지 않는다. 도구 입력의 라벨·도움말·입력·찾아보기를 가능한 경우 한 번에 드러내며 큰 그룹에서는 포커스된 조작을 우선한다.

## 계약과 범위

ADR0001의 네이티브 UI/renderer 한계 및 ADR0011의 Proposed 검색 계약을 유지한다. 검색 worker/DB/schema/도구 실행 권한·기존 ID·labelled_by를 바꾸지 않는다. 일시 UI scroll 의도와 도움말 억제 상태만 추가한다. 같은 파일 동시 수정 없이 Root가 검증된 scratch를 합친다. 새 dependency/lock/schema/thread/timer를 추가하지 않는다. 자동 GUI는 private desktop/owned Job만 사용한다.

## 인수 조건

검색 latest success·draft/old reply/failure/Stop·빈 결과·한 번만 공개, 실제 focus/clip/wheel no-snap을 검증한다. Help Escape 본문/제목 없음·owner focus·새 hover/focus/click 재개와 기존3회귀를 검증한다. 입력 문맥 100/200% 실제 label/help/input/browse clip·크기 초과·disabled·다른 입력 focus/wheel no-snap을 확인한다. 전체 fmt/strict clippy/test/release와 새 PE에 바인딩한 실제 창 검증·최신 패키지/설치 검수를 구분한다.

## 검증과 handoff

- 원 실패는 모두 보존한다. 마지막 선택 검사에서 field2/Help4, 검색 관련7개가 실제 Windows에서 통과했다. Root가 raw logs·owned Job/private desktop·Live/S 불변을 직접 대조했다.
- S의 strict workspace/all-targets clippy는 실제0이다. 마지막 rustfmt 전후는 정확한 formatter 출력으로 대조하며 실패를 통과로 덮지 않는다.
- 새 전체 검사·실제 창·패키지는 아직 후속이다. 이전 v27 ZIP은 사용자 확인 가능한 후보로 유지한다.
- 비용: 검색은 패널별 Option<u64> 하나, Help는 고정 logical ID별 일시 두 bool/frame, focus 전이에서 작은 rect union/fit 비교다. 이력은 기존 최대128행 렌더를 유지하고 중첩 scroll을 제거한다. 실제 binary/runtime 비용은 새 release/측정 뒤 기록한다.
- Git 조회128을 보존했으며 이력 재구성·외부 게시·서명·릴리스는 하지 않았다. 물리 IME·reader·OS theme/DPI·clean OS와 전체 P0 완료는 주장하지 않는다.

Root 제품5파일 통합 뒤 whole28 actual fmt/clippy/test/release 각각0, source674/1607 PASS/0 FAIL/42 ignored/43harness를 원로그·3PE·lock으로 독립 검토했습니다. CPU10315776B SHA039f2856b7bbe4aa7f93e375ebe12c2cb688011141551cb88c9e9aac6358a19b로 v27 대비9216B 증가, 다른2PE/lock은 같습니다. 실제 Search3ops/7PNG/131pins/sourceunchanged를 독립 검토했으며 Root3장 직접 확인: Library empty 안내는 읽히고 History 첫ID는 보이나 긴 취소 상태 본문은 아래로 잘려 추가 스크롤이 필요합니다. Help-v1은 Escape6표본 제목/본문0·ownerFocus를 확인했으나 환경 UIA ElementNotAvailable로 전체FAIL이며 원Job 종료정리를 확인했습니다. 실제 환경 전체문맥은 후속 v2 실행 중입니다.


## 후속 원 증거 보완

Help-v4는 actual2ops0/8PNG/41pins·source674 불변·앱 자연0을 독립 대조했다. Root가 5장을 직접 보고 도구3행 문맥·Escape 닫힘을 확인했으며 고정 설명의 아래 상태/버튼 가림은 별도 개선 중이다. 원 v2 미확인 및 v3 repeat1 실패는 보존한다.

D 여유가 949,907,456B로 줄어 소유 마커와 원 실험 증거를 확인한 뒤 24시간 이상 지난 재생성 가능한 dlpad_accessible debug incremental 폴더 6개만 정리했다. 소스·증거·사용자 데이터·registry·release 실행 파일은 보존했다. `.local-tools/session-recovery-20261013/owned-old-incremental-cleanup-20261015-v1/result.json`에서 논리 2,388,641,502B와 여유 2,906,386,432B 및 원3PE SHA 불변을 확인했다.
