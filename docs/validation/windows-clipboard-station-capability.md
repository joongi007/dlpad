# Windows owned clipboard station capability: actual failure evidence

2026-10-03. The investigation and work-item contracts were read before this independent capability experiment. No CPU product process or UIA collector was launched.

Actual command:

```powershell
powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File D:\Project\dlpad\scripts\windows_clipboard_station_probe.ps1 -OutputDirectory D:\Project\dlpad\.local-tools\windows-clipboard-station\capability-20261003
```

A newly compiled console helper process had a60second deadline and a fresh currentSID-only protected output directory. Create-time SECURITY_ATTRIBUTES contained explicit owner/currentSID and protected single-ACE DACL; requested station rights0x2002f omit WRITE_DAC/WRITE_OWNER/EXITWINDOWS. The actual call `CreateWindowStationW(NULL, CWF_CREATE_ONLY=1, rights, attributes)` returned NULL with **ERROR_ALREADY_EXISTS183**. Helper exited1 immediately, before station name/flags/ACL verification, desktop creation, window creation, or any clipboard API. Its raw result records stage create station and code183.

The generated NULL-name station therefore already exists for this logon context, as opposed to proof that a fresh isolated station can be created. Its existing handle/name/ACL/contents were never opened or inspected. No station was reused, deleted or reconfigured. No explicit named station, administrator elevation, new user/logon/service or user window station workaround was attempted. The requested current-user ACL describes the failed creation request; it is not reported as an ACL verified on a created object.

Initial process-station/thread-desktop borrowed handles were restored within this owned helper only, both API returns true, and were not closed. No newly owned station/Desktop existed to close. Helper exited within a second. WinSta0 clipboard content, formats and sequence were never read or written. The prospective synthetic clipboard code is guarded after successful nonWinSta0/nonvisible/currentSID ACL and owned process-station evidence, but none of it ran in this result. Thus **no clipboard roundtrip is claimed**, and product Windows copy remains unverified.

`evidence/windows-clipboard-station/` preserves result.json, complete actual helper.cs/helper.exe, host Get-FileHash hashes and independent manifest with per-file bytes/SHA256. Source/Cargo/shared harness unchanged. Existing8hour owned runs were left running. This completed capability experiment is a failed access-path result, not a successful Windows S10 feature test. Any alternate isolation path needs a separate reviewed contract; this experiment does not authorize access to user WinSta0 clipboard or reuse of its generated station.
