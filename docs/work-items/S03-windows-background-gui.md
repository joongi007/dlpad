# S03: Windows 비활성 private desktop GUI background 실증

- 상태: completed (bounded Windows private desktop synthetic lifecycle)
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R10, R11
- 담당/수정 범위: guardian_fault; 신규 scripts/windows_background_probe.ps1, 전용 validation/evidence만
- 선행 작업: Windows activation native backend, latest native release, private CLI host

## 사용자 결과

사용자 화면을 전환하거나 입력을 보내지 않고 새 private desktop에서 실제 Windows 앱 창 생성·작업 중 숨김·명시적 재실행 복귀를 검증한다.

## 계약과 범위

AGENTS/README 문서 순서 및 R08/ADR-0002를 따른다. product/Cargo/activation/app 수정 금지. library_identity가 release현재snapshot을확인: dlpad.exe6812160B SHA8e895467d77bc8798be90110cda1bdd321c0436829f95eb2c54f24e075556743, source53입력stable. 기존 windows-cli-host.exe는 읽기전용 재사용한다. 새data/fixture directory만currentSID user-only protectedACL로생성한다. 기존사용자data ACL변경금지. Win32 CreateDesktop/CreateProcess STARTUPINFO.lpDesktop로새privateDesktop만 사용한다. SwitchDesktop/SendInput/userdesktop/fallback금지. 시험입력은소유privatewindow HWND/PID확인후PostMessage와WM_CLOSE만허용,실제IME검증아님. 기존3soaksessions/archives유지.

## 인수 조건

- 새 private desktop의 제품window 생성 및 HWND/PID ownership을관측하고 inputdesktop전후불변을확인한다.
- 합성tool/owneddata에대해 가능하면 실제worker→WM_CLOSE→background숨김→sameapp재실행복귀를검증한다.
- GUI/renderer/privateDesktop실패는visiblefallback하지않고API오류/실제로그를기록한다.
- ownedJob/processhandle로cleanup하고desktopclose,syntheticfixture외자료삭제금지.
- screenshot있어도postmessage는syntheticinput이며IME/a11y/실제userdesktop검증으로확대하지않는다.

## 검증과 handoff

현재harness구현착수,제품GUI성공주장없음. exactbinary/source준비는담당자와합의완료.

2026-10-02 final: normal app flags(no fixture trace) active lifecycle PASS3회, both primary/secondary exit0, pinned ownedJob worker starttime 일치/숨김 heartbeat5→11/동일worker 복원/정리완료. [실제 검증과 한계](../validation/windows-background-gui.md). 실제IME/media/logout/sleep는별도미검증.
