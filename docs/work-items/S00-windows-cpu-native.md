# S00: Windows CPU candidate native GUI

- 상태: completed
- 분류: verification
- 우선순위: P0
- 요구사항: R01, R08, R11
- 담당/수정 범위: guardian_fault; 신규 scripts/windows_cpu_probe.ps1 및 전용 validation/evidence
- 선행 작업: root WindowsCPU READY SHA82602ea20f183bdf44021e60ad26416703c60dd5f9c6e89ec7bb805aee1f8704

## 사용자 결과

WindowsCPU native 후보를 비활성privateDesktop에서실행해폰트/렌더/ASCII URL입력/소유합성다운로드/취소·StopExit정리/10초RSS를실증한다.

## 계약과 범위

기존windows_background_probe.ps1 read-only clone으로helper재사용,원본default동작보존. 신규CPUscript독점. 제품/Cargo/build수정금지. currentSID-onlyprotected ACL freshfixture/newinactiveDesktop/ownedkill-on-closeJob/suspendedassignbeforeResume. PID+HWND/GetProcessTimes/pinnedworkerJobmembership확인. 사용자desktop/SwitchDesktop/SendInput/foreground/clipboard/실제IME/auth/실제미디어금지. candidateActivationServer없으므로backgroundrestorePASS로확대금지. 기존8h/archive유지.

## 인수 조건

startup/renderPNG직접view;10초finiteRSSsamples;ASCIIURL와actualsyntheticworker;Cancel·StopExit분리시험,pinnedworker종료/primaryexit0/CancelledSQLite독립대조;inputdesktopunchanged. 실패rawJSON/PNG보존,visiblefallback금지. 미실증OS/IME/reader/전체UI/hiddenrestore미검증유지.

## 검증과 handoff

2026-10-03: 초기 AccessKit startup panic은 제품 담당의 hidden-init 수정 후 실제 재검증했다. 새 SHA5cb1a363223ae5a958737c537f6b50466659e44909419b0ba9242159623b37f1에서 inspect/Cancel/StopExit 모두 exit0. Cancelled SQLite 1 attempt, artifact 없음, integrity ok; 소유 worker Job/생성시각 일치 및 종료 확인. before/after 3 source와 binary SHA 동일 뒤 root freeze 해제 통보. 전용 validation에 실패·성공과 제한 기록.

신규harness준비뒤currentbinaryhash검증/actualprivateDesktoptrial,프로젝트boundedrawevidence및전용doc갱신.
