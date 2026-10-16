# Windows private desktop GUI validation

**Current result (2026-10-02 14:16 UTC): active synthetic download hide/continue/restore/explicit stop PASS on native Windows, without fixture/trace flags.** Earlier failure sections below are historical diagnosis; they are superseded for this bounded R08 lifecycle. Real IME, actual media, user session logout/sleep and full P0 release remain separate gates.

2026-10-02 UTC, native Windows application SHA256 `8e895467d77bc8798be90110cda1bdd321c0436829f95eb2c54f24e075556743` (6,812,160 bytes).

Implemented `scripts/windows_background_probe.ps1`. Each invocation requires a fresh output root, creates it with current SID owner and protected user-only inheritable ACL, assigns owned app to kill-on-close Job before resuming its thread, and uses a newly created inactive desktop in STARTUPINFO. Window enumeration is limited to that desktop and app PID. WM_CLOSE targets only its verified window. Input desktop identity is checked before/after. No SwitchDesktop, SendInput, SetForegroundWindow, clipboard operation or visible-desktop fallback exists.

Actual command (PowerShell):

```powershell
powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File D:\Project\dlpad\scripts\windows_background_probe.ps1 -Binary D:\Project\dlpad\.local-tools\windows\target\release\dlpad.exe -OutputDirectory D:\Project\dlpad\.local-tools\windows-background-probe\stable-20261002
```

Exit 0: PID 11856, private desktop `dlpad-gui-9cd8d6dc3ec8463c9e4eb4e375e73e60`, native app window created, IsWindowVisible true after two-second startup stabilization, WM_CLOSE exited within five seconds, input desktop unchanged. Evidence: `evidence/windows-background-gui/private-desktop-startup.json`.

Initial trial found HWND before the app showed it (IsWindowVisible false); stabilized observation corrected that race without changing product code. PowerShell parameter syntax was corrected before the first runnable trial.

Scope is native GUI creation and idle close only. Active synthetic download, hide/background/secondary-instance restore, screenshots/rendering and real Korean IME remain unverified. This baseline is not acceptance evidence for R08 active-job background lifecycle. No product source or Cargo change was made. Existing Git metadata was preserved; worktree separation unavailable in the existing non-Git workspace.

## Active synthetic interaction experiment

`-Active` creates a bounded C# development fixture `worker.exe` inside the fresh protected output root. It implements `--version`, metadata JSON, 120-second progress/heartbeat overwrite, and a bounded 64 KiB synthetic artifact; no network or actual media. Keyboard events are HWND-scoped synthetic WM_KEYDOWN/WM_CHAR/WM_KEYUP, not IME. Mouse coordinates are scaled from GetClientRect; all input is scoped to the verified app PID/window. PrintWindow captures only this owned private HWND.

Actual active command uses the same baseline arguments, adds `-Active`, and uses fresh output `D:\Project\dlpad\.local-tools\windows-background-probe\active-20261002-4`. Exit **1**, correctly indicating acceptance failure. Trials 1–4 all found a visible native window and safely closed it with input desktop unchanged, but `worker_started=false`: URL/scroll remained unchanged in owned-window images. Scoped WM_SETFOCUS, 150 ms mouse-message spacing and scoped WM_PAINT did not change that result. All trial app Jobs were closed; no foreground, desktop switching or global input fallback was attempted.

Fixture independently responds through the existing inactive-desktop CLI host:

```text
windows-cli-host.exe 15 D:\Project\dlpad\.local-tools\windows-background-probe\active-20261002-4\worker.exe --version
2026.10.02
input-desktop-unchanged=True
exit 0
```

Evidence: `evidence/windows-background-gui/active-input-failure.json`, `active-typed.png`, `active-scrolled.png`. These show unmodified URL placeholder and scroll state; they do not establish whether loss occurs in winit message handling, Iced event scheduling or application interaction. Active download, hidden continuation and secondary-instance restoration remain **unverified**, not product failures proved by this harness. Proposed next bounded experiment: root-owned temporary fixture-only event tracing across winit/Iced/app, then remove trace before product validation. No product instrumentation has been added by this agent.

## Latest snapshot and leave-tracking experiment

2026-10-02, latest Windows native SHA `85cdad770ad3c1db25c25f3c50d6193636cc8b15b7d705f253b314ef48ffc59b`. `active-latest-20261002-1` reproduced the same input failure (PID12844, exit1), with visible private HWND, successful WM_CLOSE and unchanged input desktop. Evidence `latest-input-failure.json`.

Read-only source analysis: winit0.30.13 WM_MOUSEMOVE entering a window arms TrackMouseEvent(TME_LEAVE); WM_MOUSELEAVE emits CursorLeft. Iced0.14.1 clears cursor_position on CursorLeft and supplies the latest cursor to an entire event batch in AboutToWait. Synthetic pointer entry on an inactive desktop could therefore lose widget hit-testing before input is consumed. This remains a **hypothesis**, not proven event ordering.

The root authorized a target-HWND-only TrackMouseEvent cancellation experiment. `-CancelLeave` starts a finite owned-process-lifetime thread, checks HWND PID before each call, and invokes only TME_CANCEL|TME_LEAVE, never moves the system pointer or switches desktop. The thread sleeps2ms and joins before app cleanup. Trial `active-cancel-20261002-1`, PID23476, recorded1047 successful cancellations and0 API failures; exit1 because worker_started stayed false. WM_CLOSE exited and input desktop stayed unchanged. Evidence `track-cancel-input-failure.json`. Cancellation did not establish causality or resolve interaction; OS timer granularity and generated-message ordering remain uncertain.

Next proposed root-owned diagnostic: fixture-only, maximum64-row Mouse/Keyboard/Window event-kind and captured/ignored status trace; exclude actual key/text/URLs/secrets. Product/vendor source has not been modified by this task. Background active lifecycle remains unverified.


## Confirmed cause and active lifecycle acceptance

Fixture-only trace build SHA `98c48919e79f713c0ecca74882ad4f4d5ad34beec19b0a2778e9f839e3f15ff7` separated delivery from hit-testing. Delayed mouse messages produced sequence cursor-entered → cursor-moved → cursor-left → button-pressed; all ignored. Both baseline and TrackMouseEvent-cancel traces reproduced that ordering. PostMessage input reaches winit/Iced/app; private-desktop mouse leave clears the cursor before the widget can acquire focus.

Burst move→press produced button-pressed Captured **before** CursorLeft and subsequent key-pressed Captured; owned image confirmed the exact synthetic URL. Burst move→release and move→wheel likewise kept hit-testing available during each operation. This is a test harness timing change, with no fake cursor/focus/product input bypass. `-BurstClick` is required for active tests. Leave cancellation is optional diagnosis only and was unnecessary for successful lifecycle tests.

Successful fixture tracing: `events-burst-success.jsonl` and `events-burst-success-result.json` (64 rows, only sequence/kind/captured). Preserved failure comparisons: events-baseline, events-cancel, events-burst. A schema check verified maximum64 rows and those exact three fields for every trace; no key/text/coordinates/URL/PID/path appears in event rows.

Actual final command:

```powershell
powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File D:\Project\dlpad\scripts\windows_background_probe.ps1 -Binary D:\Project\dlpad\.local-tools\windows\target\release\dlpad.exe -OutputDirectory D:\Project\dlpad\.local-tools\windows-background-probe\active-normal-final-20261002-2 -Active -BurstClick
```

Exit0, appPID21540, secondaryPID17592; **both process exit codes0**. Owned workerPID14460/startFILETIME134354242039504248 was pinned by query/synchronize handle, proven a member of the owned Job and matched its exact creation time. Background hid the original window; heartbeat advanced **5→11** while it was hidden. Secondary same-binary instance restored the original window while the same pinned worker remained alive. Final StopExit/WM_CLOSE terminated the worker and app; worker handle signaled before cleanup finished. Input desktop identity stayed unchanged. `fixture_trace_exists=false` proves ordinary app invocation made no trace. Evidence `active-normal-final.json` and `active-normal-restored.png`. Two earlier independent ordinary invocations also passed (PID22416 and16696).

Failure-path trace ownership: `-FixtureEvents -ExistingTrace` precreates an owned sentinel using CreateNew. Root trace writer refuses to overwrite; GUI startup/WM_CLOSE still succeed, protected sentinel hash remains unchanged (`existing-trace-preserved.json`). No preexisting user file is used. Normal execution has no fixture trace.

All GUI fixtures used fresh protected currentSID-only ACL roots and newly created inactive desktops. No user desktop switching, system pointer movement, clipboard operation, SendInput or SetForegroundWindow. Keyboard messages are synthetic and **not Korean IME proof**. No actual network/media/yt-dlp/FFmpeg or logout/sleep scenario is covered. Product/Cargo/vendor modifications remain root-owned; this task changed only the harness/workitems/validation evidence.
