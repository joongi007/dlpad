# S00: 네트워크 첫 관측을 실패와 구별

- 상태: in-progress
- 분류: UX
- 우선순위: P1
- 요구사항: R01, R12, R13
- 담당/수정 범위: installer_batch — /tmp/dlpad-network-priming-status-candidate-20261013만; root — accessible_metrics.rs/catalog.rs 통합·검증
- 선행 작업: Windows 실제 NIC 관측 연결 구현, 현재 source freeze 종료

## 사용자 결과

첫 수신·송신 표본을 기다리는 동안 실패 문구 대신 관측 준비 상태를 본다.

## 계약과 범위

현재 footer는 fresh Observing 상태에서 rate가 아직 없을 때도 NETWORK_UNKNOWN을 표시한다. 기존 mode/worker/sample/revision/3초 freshness/실패/Off·NoRoute 정책을 바꾸지 않고 표시 문구만 구분한다. catalog 추가는 root가 조정한다. stale/중단/실패를 준비 중으로 숨기지 않는다.

## 인수 조건

실제 Context status render에서 pending 또는 fresh Observing/no-rate는 준비 중, 실제 실패/NoRoute/Off와 stale·중단은 기존 문구, 유효한 rate는 기존 숫자·단위를 표시한다. worker 조회와 구조 변경 없이 실패 경로를 검증한다. 실제 rate의 native 확인과 본 표시 검증을 구분한다.

## 검증과 handoff

현재 native Auto 복귀 직후 화면에서 수신·송신 측정 불가 문구를 관찰했고 source status 분기를 대조했다. 현재는 후보 준비 단계이며 기능 완료가 아니다.

## 2026-10-13 통합 검증

root가 accessible_metrics.rs/catalog.rs에 관측 준비 문구를 통합했다. fresh Observing/no-rate 및 첫 pending에는 준비 상태를 표시하고 실패·오래된 표본·종료·꺼짐·경로 없음·유효 rate는 기존 정책을 유지한다. worker-free 실제 Context 상태 회귀와 Windows 전체 fmt/clippy/test/release 1278/0/30 및 root 원로그/655 source/artifact/lock 대조 PASS다. 첫 요청이 dispatch된 뒤 표본이 없는 별도 구간은 기존 UNKNOWN을 유지한다. 실제 native 관측의 준비 상태 캡처는 미검증이다. 새 timer/의존성/schema 추가 없음. [원검토](../validation/evidence/windows-msvc-ui-feedback-library-worker-whole-20261013/root-review.json).

후속 실제 발견: 현재8ab7의before-uia.png는처음querydispatch후표본도착전수신·송신측정불가를보인다. 기존후보의pending-only구간제한이원인으로source상확인되어, no completed sample/Auto·Manual/실패·stop없음은dispatch이후에도관측준비로표시하는좁은후보를준비했다. cached stale/잘못된revision·실패·Off/NoRoute·stop정책은그대로이며원시험을보존한다. 기존Context회귀에서queued와dispatched-first-awaiting을각각확인하도록수정했다. root의metrics단일파일label/error후보와같은base SHA에합성했으며아직live source에는적용하지않았다.

root가labelcontext후보와함께live MetricsPanel에first-dispatch waiting수정을통합했다. 새whole이진행중이며현재v8 exe/ZIP에는이추가수정이없다. 진짜 실패·stale/revision·Off·stop/NoRoute는기존대로다.

### 최초 요청 dispatch 후 대기 공백 수정

기존 pending=false·latest=None 상태가 첫 관측 전에도 측정 불가로 표시되는 실제 초기 캡처를 확인했다. 아직 완료된 샘플이 없고 오류/중지/Off 상태가 아니면 dispatch 여부와 무관하게 준비 중으로 표시한다. 기존 실패·Off·정상 rate·stale/wrong revision 의미는 유지한다. 기존 실제 Context test와 narrow footer 기대문구를 갱신했다. 최초 whole는 테스트의 이전 UNKNOWN 문구 기대만 실패했고 원 실패를 보존했다. 후속 whole-v2는1,284/0/30, fmt/clippy/release 성공이다.

현재 default f15 빌드의 current-default-titlebar-theme-private-20261013 before-uia.png/startup-full.png를 주 담당이 직접 확인: UIA 조작 전 footer에 수신·송신 관측 준비 중이 보인다. 이후 설정에서 실제 수신·송신 수치가 표시된다. 최초 presentation 타이밍·물리 desktop·모든 startup race까지 검증한 것은 아니다. [실제 GUI 검토](../validation/evidence/current-default-titlebar-theme-private-20261013/root-review.json).
