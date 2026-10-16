# Windows CPU Library snapshot actual GUI regression

Snapshot: executable SHA `38e7d11ad92b77011a562c679cf08a42a2c7d4fbab8e69931768da5baa7086c0`, 5,438,464 bytes. Product source frozen during trials. Previous5cb1 native GUI evidence remains untouched.

## Commands and scope

```powershell
powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File D:\Project\dlpad\scripts\windows_cpu_probe.ps1 -Binary D:\Project\dlpad\.local-tools\windows\target\release\dlpad-cpu.exe -OutputDirectory D:\Project\dlpad\.local-tools\windows-cpu-probe\library-cancel-20261003 -Mode cancel
```

The same command ran modes inspect, stop-exit and library, with respective fresh suffixes `library-inspect-20261003-2`, `library-stop-20261003`, `library-stop-20261003-2`, `library-tab-20261003`. Dedicated harness changed only fixed expected binary SHA and an inactive library mode: click owned HWND client149,38, wait1.2seconds, PrintWindow. Its pre-edit SHA `ef4011ee841fb6900bd772d8c5cec5531a22af14aee28f5fdeee851685d18469` is in before.sha256. Original windows_background_probe.ps1 unchanged. First sandbox Windows interop returned vsock binding error; authorized host execution proceeded. First host invocation rejected the old expected SHA before launching GUI; fresh directory was used after updating the explicit snapshot guard.

## Actual results

Inspect exit0, PID2564: visible only on owned inactive desktop, Korean download fields and new 파일 목록 tab directly viewed.10second working set69,750,784→69,767,168 bytes.

Cancel exit0, appPID20032/worker8676: real owned synthetic worker started; Job membership and creation FILETIME134354301454387929 match; cancellation returned idle, worker exited, primary exit0, input desktop unchanged.10second idle working set69,689,344→69,705,728 bytes. Independent read-only SQLite after app exit: integrity ok, job Cancelled, one attempt, no artifacts.

Library exit0, PID22092: actual owned tab click showed 파일 목록, 저장 폴더, 저장 위치 적용, 기본 문서 폴더 사용, 목록 새로고침 and disabled 다음128개. Screenshot directly viewed. Fixture folder was empty; no rows visible. The Documents button was not invoked. This verifies empty tab rendering only, not scan cancellation, selection/deletion, pagination bounds under load, or actual file-row behavior.

The original StopExit harness attempts were **unverified**: two real trials exit1 because worker never started. typed.png shows the expected ASCII URL; after-start.png shows an empty URL with HTTP/HTTPS URL-required message. The second trial ran alone and reproduced this. No job DB was created; each primary still exited0 and input desktop stayed unchanged. The cause is not established as a product bug. The harness synthesizes WM_KEYDOWN(A)/WM_CHAR/WM_KEYUP per character, so modifier/shortcut/timing dependence may affect it; no user keyboard state or foreground focus was altered to bypass the failure. Earlier5cb1 StopExit success is preserved and is not promoted to this executable.

## Evidence and limits

`evidence/windows-cpu-library/` stores before/after hashes, all JSON/PNG including both failures, Cancel raw SQLite plus sqlite-proof.json. Product before/after3source hashes and executable match; only dedicated harness changed. Source hashes: accessible.rs `a609bcc46f12d05a2b919a6ba9a6ad25de271ae7ae0b462793db762015bcccf2`, software_host.rs `b6507a9e5d7ededc3831092ea564f3f92ae004636d7a1e4bcfbfea402e64d5d3`, CPUmain `9a1ffab2e01b5455ff290a3e0a9a7e0f2652a552b03b0d4f6db94c4584758645`. All GUI ended before root freeze was released. No user desktop fallback, global input, clipboard, real IME/reader/media/auth was used. CPU background restoration remains unsupported/unverified.10second RSS is idle sampling, not long-term peak proof.


## Input boundary follow-up: StopExit now actually verified

Read-only source analysis: accessible.rs options/start never mutates self.url; the TextEdit field is its sole runtime mutator. winit0.30.13 Windows keyboard.rs168 ignores standalone WM_CHAR when no pending event_info exists; keyboard.rs495 onward builds logical and physical key information from declared VK/scancode and GetKeyboardState; get_async_kbd_state uses GetAsyncKeyState and only reads GetKeyState toggles. egui0.33.3 text_selection/cursor_range.rs122 interprets command+A as select-all. These facts make sending a declared A for every URL character a poor keyboard-event fixture. They do not prove that a specific modifier caused the earlier URL loss.

One owned CHAR-only control (`library-charonly-20261003`, -Mode stop-exit -CharOnly) exited1, typed.png stayed empty, and no worker started. This is consistent with the documented pending-event requirement, not a product StopExit failure. One approved matched-key control (`library-matchedkeys-20261003`, -Mode stop-exit -MatchedKeys) used VkKeyScanW low-byte VK and MapVirtualKeyW scan in key lParam, preserving original WM_CHAR text. Shift requirement high bits were not enacted, keyboard state/foreground were not changed. Finite29 ASCII characters only. Declared physical keys now match each character's base key mapping; punctuation shifted text remains explicitly supplied by WM_CHAR, so this is synthetic input rather than real keyboard/IME proof.

Matched control exit0: app11608, actual worker18312, Job membership true, creation FILETIME134354305603574948 matches, worker exited on cleanup, stop-dialog directly viewed, primary exit0, input desktop unchanged. Independent SQLite after exit: integrity ok and Cancelled job with one attempt/no artifacts.10second idle RSS69,775,360→69,791,744 bytes. Product binary and three source hashes stayed identical to the initial38e7 snapshot through this trial. All GUI ended and root freeze was released. A bounded product trace was unnecessary; exact inner event causality remains unproven. The dedicated CPU harness now defaults matched-key mode; explicit `-MatchedKeys:$false` retains the historical declared-A fixture for reproduction. The normal Iced harness is untouched.

Both original failures and CHAR-only control remain raw evidence. New successful JSON/PNGs/SQLite are under `evidence/windows-cpu-library/library-matchedkeys-20261003/`. This closes actual38e7 startup/render/Cancel/StopExit/emptyLibrary acceptance; loaded-row bounds, scanCancel, user IME/reader and CPU background restoration remain outside this experiment.
