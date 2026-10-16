# S03: Windows private desktop synthetic input diagnosis

- 상태: completed (bounded Windows private desktop synthetic lifecycle)
- 분류: diagnosis
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: guardian_fault; scripts/windows_background_probe.ps1 및 전용 validation/evidence
- 선행 작업: private desktop baseline 및 active 입력 실패 evidence; 최신 native Windows snapshot

## 사용자 결과

사용자 desktop을 건드리지 않고 private GUI에서 synthetic 입력 누락 위치를 구분하여 active background 검증 가능성을 판단한다.

## 계약과 범위

제품/vendor/Cargo read-only. WM_KEYDOWN→WM_CHAR와 mouse event 경로를 source에서 분석하고 private HWND PostMessage만 수정한다. SendInput/SetForegroundWindow/clipboard/SwitchDesktop/visible fallback 금지. 최신 release SHA85cdad770ad3c1db25c25f3c50d6193636cc8b15b7d705f253b314ef48ffc59b 사용. root 임시 fixture trace 필요시 정확event/interface 제안 후 root 담당으로 유지. 기존 장시간 실행/아카이브 재시작·종료 금지.

## 인수 조건

source hypothesis와 실제 관측을 분리; 새snapshot GUI/입력 trial 및 bounded owned-window evidence; 사용자 input desktop 유지; 실패도 동일 cleanup; actual active lifecycle만 success로 표시. 정상앱에는trace를생성하지않는다. fixture-only 제품trace의유지/제거는root담당이며 비밀없는64행계약을유지한다.

## 검증과 handoff

초기 read-only hypothesis: winit mouse Enter가 TrackMouseEvent(TME_LEAVE)를 실행하고 user pointer가 private HWND 밖이면 synthetic CursorLeft가 마지막 state를 지워 Iced AboutToWait batching cursor가 unavailable일 수 있다. root에 최대64개 Iced Mouse/Keyboard event tracing 제안; 아직 actual 원인으로 확정하지 않음.

2026-10-02 final: normal app flags(no fixture trace) active lifecycle PASS3회, both primary/secondary exit0, pinned ownedJob worker starttime 일치/숨김 heartbeat5→11/동일worker 복원/정리완료. [실제 검증과 한계](../validation/windows-background-gui.md). 실제IME/media/logout/sleep는별도미검증.
