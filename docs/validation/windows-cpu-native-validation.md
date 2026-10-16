# Windows CPU native GUI validation

Status: completed for native startup/render/ASCII synthetic worker/Cancel/StopExit and 10-second idle RSS. Initial actual startup failure is preserved below.

Initial native CPU executable SHA `82602ea20f183bdf44021e60ad26416703c60dd5f9c6e89ec7bb805aee1f8704`. Dedicated `scripts/windows_cpu_probe.ps1` is a read-only clone/adaptation of existing private desktop helper; `windows_background_probe.ps1` was not changed. Fresh fixture uses currentSID-only protected ACL, new inactive Desktop, kill-on-close Job assignment before ResumeThread, HWND/PID verification, PrintWindow only on owned window, finite10second working-set samples and ASCII burst messages. CPU candidate does not expose background activation; hidden/restore is outside acceptance.

Actual failed inspect command:

```powershell
powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File D:\Project\dlpad\scripts\windows_cpu_probe.ps1 -Binary D:\Project\dlpad\.local-tools\windows\target\release\dlpad-cpu.exe -OutputDirectory D:\Project\dlpad\.local-tools\windows-cpu-probe\inspect-20261003 -Mode inspect
```

Exit1: processPID5892 exited before any dlpad HWND. Window/render/RSS were not measured; input desktop unchanged. Original JSON `evidence/windows-cpu-native/startup-failure.json` preserves the first result (its primary_exit_code259 is an initial harness missing-observation, not the actual exit code; harness now queries exit code even before a window exists).

The existing native CLI runner independently reproduced startup on another owned inactive desktop using the same fixture environment and argv `--downloads`: `windows-cli-host.exe 15 D:\Project\dlpad\.local-tools\windows\target\release\dlpad-cpu.exe --downloads`. It exited**101**, with AccessKit winit0.29.2 panic: adapter must be created before the window is shown. User input desktop remained unchanged. Evidence `startup-panic.log`; full source input manifest `startup-source-manifest.json` has software_host.rs SHA `336a2ddc19723fa36ccbf6e771eb72faa767a30ed44f772ad10f6b6e25a70fcc`.

Source read confirms Window default visible creation preceded input.init_accesskit. Root/library owner was notified to create with_visible(false), initialize adapter, then set_visible(true). This agent did not change product or Cargo; new READY is required before retry. The first CLI diagnostic accidentally omitted the runner's required argv and returned usage2; it was corrected and is not product evidence.

No user desktop/SendInput/foreground switching/clipboard/actual IME/auth/media was used. Original raw failures are kept. Initial failure was subsequently re-tested against the corrected snapshot below.


## Corrected native snapshot: actual GUI results

SHA `5cb1a363223ae5a958737c537f6b50466659e44909419b0ba9242159623b37f1`, 5,297,664 bytes. Product owner changed hidden window creation before AccessKit initialization. The harness was adapted to Windows CPU layout and atomic owned-HWND mouse-message bursts. Earlier delayed clicks did not start a worker; those harness failures remain preserved, and are not product download failures.

Using the command above with `-Mode inspect`, `cancel`, and `stop-exit`, and fresh output suffixes `inspect-fixed-20261003-2`, `cancel-fixed-20261003-4`, `stop-fixed-20261003`, all three actual runs exited0. Inspect PID7148 rendered Korean text and exited on WM_CLOSE. Cancel PID7820 started synthetic worker27788, verified Job membership and creation FILETIME134354293255415103, returned to idle, and exited0. Read-only SQLite independent inspection after exit: integrity ok, Cancelled job, one attempt, zero artifacts. StopExit PID13740 started worker19628 (creation FILETIME134354294145606778), showed the actual stop/return dialog, selected stop, and exited0. Both pinned workers exited on owned cleanup. Each run reported unchanged input desktop. Screenshots `render.png`, `cancelled.png`, and `close-prompt.png` were directly inspected.

Ten-second idle working-set samples (not long-run peak memory): inspect69,578,752→69,591,040 bytes; Cancel69,677,056→69,697,536; StopExit69,787,648→69,804,032. Before/after binary SHA and source hashes stayed identical: accessible.rs `cdcc03c0147e7f273171ae7d2c8f24cf96fd58ffd5765d458440f057cc2c6c45`, software_host.rs `b6507a9e5d7ededc3831092ea564f3f92ae004636d7a1e4bcfbfea402e64d5d3`, accessible_cpu_main.rs `087b169e17c1fad77651dbd3d0e3dc2528bfebb64b5f5f11afe70d921432d4cc`. Root received GUI completion/freeze release before further product edits.

Evidence directory `evidence/windows-cpu-native/` retains initial panic/source manifest, intermediate input failures, three successful JSON/PNGs, raw owned SQLite and independent sqlite-proof.json; total826,214 bytes at preservation time, per-file cap2MiB. Original Iced background harness is unchanged. Synthetic ASCII messages are not IME or accessibility proof. CPU background hide/restore, screen reader, real downloads, and sustained-memory behavior are not validated here. No user clipboard, visible fallback, foreground activation, or global input was used.
