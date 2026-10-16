# S00 AT-SPI 비활성 상태 보정

- 상태: completed
- 분류: bug
- 우선순위: P0
- 요구사항: R08,R11
- 담당/수정 범위: guardian_fault; 이 작업문서, docs/validation/atspi-disabled-state.md, /tmp 단독 회귀 설계 및 소유 cpu_background_probe.py
- 선행 작업: root vendor/Cargo 수정 및 정확 CPU binary READY

## 사용자 결과

비활성 백그라운드 버튼을 보조 기술이 Enabled/Sensitive로 잘못 읽는 결함을 실제 소비자 tree와 격리 GUI로 검증한다.

## 계약과 범위

locked accesskit_atspi_common 0.14.2 node.rs348와 consumer0.31.0을 read-only 확인. vendor node/Cargo/lock은 root만 수정한다. 현재 GUI를 시작하지 않는다. 사용자 desktop/clipboard/실제 미디어 금지.

## 인수 조건

실제 consumer Tree의 Button/TextInput 각각 read_only/disabled 조합 4개로 Enabled/Sensitive/ReadOnly 회귀를 구분한다. disabled Button은 Enabled/Sensitive 없음. 기존 TextInput readonly 정책은 범위 밖 변경 없이 유지한다. READY 뒤 unavailable 실제 클릭과 AT-SPI 상태 일치, active URL 비활성 상태를 관측한다.

## 검증과 handoff

root 실제 consumerTree 8fixtures/1test 및 CPU59tests PASS. 최신 고정 binary의 unavailable/active-hidden 실제 Linux GUI 모두 PASS; 접근성 비활성 상태와 클릭 동작 일치, host 소유 PID 종료/SQLite 무결성/sourcehash 동일 확인. 자세한 명령과 원시 증거는 전용 validation에 기록한다.
