# S03: Windows private desktop active download background lifecycle

- 상태: completed (bounded Windows private desktop synthetic lifecycle)
- 분류: verification
- 우선순위: P0
- 요구사항: R08, R10, R11
- 담당/수정 범위: guardian_fault; scripts/windows_background_probe.ps1, 전용 validation/evidence
- 선행 작업: S03-windows-background-gui baseline; root native Windows activation backend

## 사용자 결과

활성 합성 다운로드 중 Windows native 창을 숨겨도 소유 작업이 계속되며 동일 실행 파일 재실행으로 원래 창이 복귀하는지 실증한다.

## 계약과 범위

README/PRODUCT/CONTEXT와 background activation ADR 및 ARCHITECTURE 계약 유지. 제품 코드/Cargo는 root 소유로 수정하지 않는다. 새 currentSID-only protected ACL fixture와 C# 합성 도구만 생성한다. 새 inactive private desktop의 소유 HWND에만 PostMessage를 보내며 desktop 전환, foreground 전환, SendInput, clipboard, 사용자 데이터, 실제 미디어/네트워크를 사용하지 않는다. 기존 3개 soak/archive 세션을 유지한다. synthetic keyboard messages는 실제 IME 검증이 아니다.

## 인수 조건

현재 snapshot SHA 일치; private 창과 실제 합성 worker heartbeat 확인; WM_CLOSE 확인창의 Background 버튼으로 원래 창 숨김; heartbeat 계속 증가; 같은 snapshot의 두 번째 인스턴스 종료 및 원래 PID 창 표시 복귀; 소유 Job 종료와 fixture 정리; input desktop 변경 없음. 입력 실패는 원인/증거를 기록하고 사용자 desktop 우회 금지. 성공하지 않은 구간은 미검증으로 남긴다.

## 검증과 handoff

실제 실행 결과는 후속 전용 validation 및 JSON evidence로 기록한다.

2026-10-02 final: normal app flags(no fixture trace) active lifecycle PASS3회, both primary/secondary exit0, pinned ownedJob worker starttime 일치/숨김 heartbeat5→11/동일worker 복원/정리완료. [실제 검증과 한계](../validation/windows-background-gui.md). 실제IME/media/logout/sleep는별도미검증.
