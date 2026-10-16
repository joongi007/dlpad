# S01: 변경 없는 스캔의 임시 identity 인덱스 비용 제거

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root crates/engine/src/library.rs·원회귀/비용/evidence·CONTEXT/ADR0002; metrics_recovery /tmp 읽기검토
- 선행 작업: quiet worker 실측 스캔2회 각CPU0.34375초·traversal/insert가 지배

## 사용자 결과

동일한 파일 목록을 확인할 때 rename 처리를 위한 불필요한 임시 인덱스 유지 비용을 줄인다.

## 계약과 범위

원 full fresh metadata·path PK/중복검사·exact whole-set 비교·취소/실패 rollback·변경 시 rename identity 처리와 저장 schema를 유지한다. temp.library_observation_identity만 setup에서 제거하고 publication_needed일 때 생성한다. 사용자 DB의 persistent 인덱스·파일을 삭제하지 않는다. Cargo/lock/의존성 변경 없음.

## 인수 조건

같은 Store changed→unchanged→rename 및 취소 rollback/alias freshness/Windows exact 비교 회귀 유지. 같은 quietworker source로 전후 CPU를 측정하고 최신 전체/GUI 인수 범위를 명시한다. 성능 개선/예산 통과를 실행 전에 주장하지 않는다.

## 검증과 handoff

원 baseline 원로그를 먼저 고정하고 root만 코드 수정한다. 임시 인덱스는 구현 내부의 재생성 가능한 상태이며 main.library_identity를 변경하지 않는다.

### 2026-10-06 실제 통합 결과

동일 quietworker 평균은 baseline1.32685%→lazy index1.24799%로 한 쌍 실측했으며 미달이다. 실제 Windows library17회귀와 이후 최신 전체4검증을 통과했다. persistent schema/Cargo/lock 변경은 없다. 최신 GUI 성능 인수는 미완료이므로 scoped 상태는 유지한다. [전체 증거](../validation/evidence/windows-library-observation-cost-workspace-20261006-continue/result.json).
