# S00: AT-SPI 펼침 상태 bridge

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R07, R08, R11
- 담당/수정 범위: metrics_recovery — vendor/accesskit_atspi_common/src/node.rs, DLPAD_PATCH.md, 전용 validation
- 선행 작업: root UI helper AccessKit expanded property 제공; locked accesskit0.21.1/common0.14.2

## 사용자 결과

명시적인 펼침 상태를 native AT-SPI Expandable/Expanded로 관측한다.

## 계약과 범위

root가 shared UI/GUI/Cargo 빌드를 조정한다. vendor state mapper만 None→둘다없음/Some(false)→Expandable/Some(true)→Expandable+Expanded, disabled/readOnly 기존policy 보존. 새의존성/schema/Cargo/sharedhelper 변경없음. licenses/provenance 유지.

## 인수 조건

actual consumer Tree 및 실제 Adapter update closed→open→closed 상태와 StateChanged events, None에서없음. 단위시험을 OS GUI/reader로확대하지않고 root가private 실제GUI통합인수한다.

## 검증과 handoff

setter/getter 및 기존 generic state XOR notification 경로 확인. 실제 adapter 회귀 작성 후root직렬 Cargo에서 red/green과 GUI를 검증한다. 후속결과 전완료를주장하지않는다.


## Red와 좁은 수정

root가 actual_adapter_tree_updates_expand_states_and_events를 실제 실행했고 exit101/closed Expandable 기대true·actualfalse를 재현했다. 이후 owner가 NodeWrapper state mapping에 Some(false/true)분기만 추가했다. existing generic XOR notification은 변경하지 않았다. source freeze를 root에 전달했으며 post-fix Cargo/GUI는 root 직렬 검증을 기다린다.


## 최초 green 컴파일 실패와 수정

owner의 consumer getter forwarding 보고가 잘못됐다. 실제 consumerNode에는 is_expanded 직접method가 없으며 data().is_expanded()를 사용해야 한다. root green은 E0599/exit실패를 보존했고 owner가 해당호출 한 줄을 정확히 수정했다. setter는 accesskitNode.set_expanded(bool) 그대로다. 이후 green/GUI 결과 전 통과를 주장하지 않는다. cpu_plan_toggle_probe.py 소유권은 root에 양도했다.

## 통합 인수 결과

[실제 통합](../validation/cpu-expanded-state-validation.md): egui actualtree 및adapterstate/event 회귀PASS. Linux 실행계획 pointer/AT-SPI action/Enter/Space와진단·MPD header closed/open/closed/reopen 실제GetState를 확인했다. default200/CPU339/Windows255PASS, ignored5/5/1별도, fmt·clippy·release PASS. root보존DB·원Failedjob·소유PID부재·PNG독립확인. 다른OS reader/전배율/전체P0·스택완료와구분한다.
