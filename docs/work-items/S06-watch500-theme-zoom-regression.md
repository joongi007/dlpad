# S06: 현재 기본 제품의 테마·200% 세션 확대 회귀

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당/수정 범위: root — scripts/windows_settings_private_probe.cs, scripts/windows_zoom_keyboard_native_focus_probe.cs 및 새 evidence·본 문서; metrics_recovery — /tmp 수명 보강 초안만
- 선행 작업: 현재 기본 CPU source543 전체 검증·기본 SHA 복원·CPU 측정 종료

## 사용자 결과

기본 제품에서 Light/Dark/System 선택과 저장·재시작, 200% 세션 확대 후 한글 UI·원 창 입력을 확인한다.

## 계약과 범위

fresh private data와 private desktop, 기본 CPU SHA59448518d46ae8f3da4bf18540ae8ed031497584f13799e231966dd28f754848·원 native HWND/creation/Job를 고정한다. 기존 관측 도구 timeout 수명을 cooperative stop/Job 종료/join5초/safeClose-or-retain로 보강한다. 사용자 desktop/입력/clipboard/실제 IME 및 physical DPI 설정은 바꾸지 않는다. 제품/Cargo/schema 변경은 없다.

## 인수 조건

세 테마 설정 실제 UIA·DB·재시작 및 PNG, 200% session scale 실제 geometry·한글 field 원 owner 입력과 PNG, 자연 exit0/Job0/private/input/source pins를 root가 대조한다. viewport scale과 physical DPI·실제 composition을 구별하고 강제 종료/retained 결과는 FAIL이다.

## 검증과 handoff

하위가 legacy helper 수명 보강 초안을 준비했으며 root가 통합 검토한다. native 실행 전이다.

Light/Dark/System 실제 UIA 선택·재시작 값·저장된 SQLite theme·두 단계 자연 exit0/Job0/private/input/safe close와 PNG6개를 확인했다. 200% 시험은 geometry32→64까지 확인했으나 target GUI thread GetThreadDesktop가 오류5로 거절돼 keyboard 인수 FAIL이다. 원 실패를 보존하며 해당 조회의 성공을 주장하지 않는다. 원 CreateDesktop HANDLE을 통한 [EnumDesktopWindows](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdesktopwindows) exact HWND/PID/TID/creation membership은 이미 소유한 capability에 대한 독립 증명이다. root는 이 증명을 검토 중이며 target 조회 재시도·권한 변경·global desktop fallback 없이 원 [AttachThreadInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-attachthreadinput) 실패/원 focus readback/반드시 detach 계약을 유지한다. 이 API도 거절되면 그 동작은 중단한다.

추가 검증: 원 private desktop HANDLE의 exact HWND/PID/TID/creation membership을 attach 전·detach 후 확인한 새 시험은 200% geometry와 합성 키보드47단계를 통과했다. 별도 한글 텍스트 전송 단계는 길이9이나 실제 값 `\\? 끜% 123`으로 실패했다. 실패 evidence는 보존한다. collector의 CharSet 생략 PostMessage 호출을 명시 PostMessageW로 교정하여 fresh `windows-watch500-session-zoom-hangul-unicode-20261006-continue`에서 재시험 중이다. 제품 Rust 변경은 없으며 실제 IME 조합 입력으로 집계하지 않는다.

최종 scoped 결과: 명시적 PostMessageW로 교정한 새 Unicode 시험이 통과했다. root가 입력 PNG와47개 focus/geometry, 원 membership 전후·detach, 합성 한글 expected/actual 일치, CAP/sentinel·immutable 입력 핀, 자연 exit0/Job0/private/input을 대조했다. [root 검토](../validation/evidence/windows-watch500-session-zoom-hangul-unicode-20261006-continue/root-review.json). 실제 IME·reader·물리 keyboard/DPI 인수는 미완료다. 이후 설치 기반 소스 변경은 당시543개 snapshot의 시험 결과에 합산하지 않는다.
