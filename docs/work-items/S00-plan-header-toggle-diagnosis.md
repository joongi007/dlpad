# S00: 실행 계획 헤더 native 토글 진단

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R07, R08, R11
- 담당/수정 범위: metrics_recovery — 신규 scripts/cpu_plan_toggle_probe.py 및 전용 docs/evidence
- 선행 작업: frozen CPU166946…; diagnose/setup 기존 local tracker 계약

## 사용자 결과

실행 계획을 pointer·native Action·Enter/Space로 실제 열고 닫을 수 있는지 분리한다.

## 계약과 범위

compiled source/Cargo/sharedhelpers 변경 없음. privateXvfb/DBus/임시data/합성도구/미디어미사용. 전용시험 action목록/role/state/내용tree/PNG만 기록한다.

## 인수 조건

pointer open/close 기준을 actual preview tree와 PNG로 재현. native index별 GetName/GetDescription/GetKeyBinding 대조. action반환만으로 성공하지 않고 실제 토글 확인. repaint지연 및 focus키보드 경로 비교; 원인이 확인되면 root 최소공유fix계약 전달.

## 검증과 handoff

가설순위1: 잘못된 actionkind/index.2: eguiheader input 소비누락.3: callback후 repaint wake지연. 각 예측은 실제 actionmetadata/토글대조/시간+pointermotion 경계로 시험한다. 초기 관측은 R07raw에서만 있으며 이 최소loop 완료 뒤 원인 판정한다.


## 실제 결과

footerfix 및 repeat 최소loop 각각 exit0/PASS. pointer/nativeindex0click/GrabFocusEnter/Space actualpreview출현·소멸을 확인했다. 원인은 docs/validation/cpu-plan-toggle-diagnosis.md를 따른다. 제품토글버그 재현안됨; 전용harness의고정footer좌표/cap/전체bounds 수집조건을교정했다. sharedsource/Cargo무변경. header expanded state 메타데이터는 여전히 미제공이며 전체 reader/다른OS는 별도 게이트다. ownedcleanup와binarySHA 보존을 기록했다.
