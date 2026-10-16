# S03: 닫기·background 복귀 native GUI 검증 준비

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: diagnostic_cache agent; 신규 scripts/background_ui_probe.py, 이 계약, docs/validation/background-ui-validation.md 및 전용 evidence만
- 선행 작업: 주 담당의 background/restore 인터페이스 합의와 새 release binary

## 사용자 결과

작업 중 X에서 돌아가기/중단 후 종료/백그라운드 계속을 실제 native GUI로 구별하고 창이 숨겨져도 같은 소유 프로세스의 작업이 유지되며 명시적 복귀가 가능한지 검증한다.

## 계약과 범위

README→PRODUCT→CONTEXT→R08/R11→ADR0002/UX/ARCHITECTURE를 읽고 실제 app/instance 및 pinned Iced0.14 소스를 read-only 조사한다. 현재 app Close는 active에서 StopExit/Return만 표시하며 background/restore는 아직 구현되지 않았다. 문서 목표를 구현 완료로 표현하지 않는다. 기존 app/process/engine/Cargo/probes 수정·build·새 subagent 금지. Xvfb WM 없는 환경만 사용하고 실패 시 desktop fallback하지 않는다. 임시 sibling data/media와 합성 owned tool만 사용한다.

## 인수 조건

주 담당과 interface 합의 후 WM_DELETE_WINDOW ClientMessage로 정확 owned 창 close를 보내고 XGetWindowAttributes.map_state로 숨김/복귀를 관측한다. hidden 동안 같은 app PID+startticks·소유 tool progress와 DB 작업 유지, secondary restore가 worker/journal 중복을 만들지 않음, Return은 visible 유지, StopExit는 Cancelled/소유 tool 종료 뒤 app exit를 검증한다. native screenshot·binary SHA·미검증 OS/WM/트레이/자원 비용을 구분한다.

## 검증과 handoff

read-only audit→Linux per-instance nonce Restore API를 주 담당과 합의했다. 신규background_ui_probe.py 구현/AST9arg 및 Xvfb의owned100x100selfcheckwindow에서WM_DELETE_WINDOW/map_stateABI통과. 실제 background binary 제공 전 GUI 완료를 주장하지 않는다. [조사/검증 기록](../validation/background-ui-validation.md). IPC는별도작업계약과10개 engine시험으로 검증했다. 새 product binary signal/close 선택 screenshot 좌표와 실제 hidden/restore/StopExit/Return 실증은 남는다.

2026-10-02 실제 Linux Xvfb Return/StopExit/background 복귀/endpoint unavailable 완료. 인자10건, 실측 버튼 좌표, same-process worker/journal 유지, Cancelled/exit0와 screenshot 수동 검토를 전용 validation에 기록했다. 최초 긴 socket path 실패 증거도 보존했다. docs checker115개 통과. 다른 OS/실제 WM/tr레이/hidden 완료 정책은 미검증이며 최종 binary 변경 후 재시험은 통합 담당 gate다.

최종 release SHA256 e9fa3ce36b3e4a1285ef0ee5930179ee4fa126451730fd0cbdcb2bae020bcc7f의 background8.182초/processing37.192초 재시험과 화면검토 완료. [최종 증거](../validation/background-ui-validation.md). Linux Xvfb 범위로 done이며 다른 OS/실제 WM/외 UID/악의적 same-user path race gate는 문서에 유지한다.
