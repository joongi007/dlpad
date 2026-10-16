# CPU native candidate download GUI acceptance

2026-10-03 KST. **Four actual Linux GUI cases PASS** on CPU native candidate SHA `28b412773a5fc1434f86d396667a30e7b6d671a8e30affbe1a41cb1ecea7d658`. Each case used its own Xvfb, private `dbus-daemon --session --nofork`,0700 temporary XDG config/cache/data/runtime and GSETTINGS_BACKEND=memory. Inherited DBUS_SESSION_BUS_ADDRESS/AT_SPI_BUS_ADDRESS/Wayland were removed. No user display, clipboard, bus or reader was used; copied report came only from the owned Xvfb selection.

Actual commands after root READY:

```text
python3 scripts/accessible_download_probe.py target/release/dlpad-cpu /tmp/dlpad-cpu-download-20261003-inspect.json --case inspect --font /mnt/c/Windows/Fonts/malgun.ttf
python3 scripts/accessible_download_probe.py target/release/dlpad-cpu /tmp/dlpad-cpu-download-20261003-final.json --case all --font /mnt/c/Windows/Fonts/malgun.ttf
python3 -m py_compile scripts/accessible_download_probe.py
```

Inspect exit0 (layout observation only). All exit0: success Downloaded/1Attempt/1artifact, failure Failed/1Attempt/noartifact+actual sanitized report copy, pause Paused→sameJob/new distinct Attempt/original source re-extraction/partial64KiB unchanged→Downloaded/2Attempts/1artifact, close CancelWait→Cancelled/1Attempt/noartifact/app exit0/worker reaped. Synthetic tool caps duration120seconds/partial64KiB, final bytes `synthetic candidate artifact` (28bytes); no actual network/media/yt-dlp/FFmpeg.

Source/fixture hashes and app/bus/worker PID/start ticks are in each result. Worker token and executable argv were verified before pidfd pinning. Start/end candidate hashes match in all four cases. GUI and bus logs are each bounded64KiB; SQLite backups are bounded2MiB/three seconds; report read256KiB maximum. Every bus/drainer was reaped, every worker stopped with no fallback. Independent host read after cleanup found all four recorded bus PIDs missing and owned process groups absent (`bus-cleanup-independent.json`). No global process scan/termination.

## UI completion versus SQLite completion

The earlier Glow snapshot captured immediately after database Downloaded; its PNG could still show the worker/running state before the UI consumed its completion. That was an observation race, not proof of failed completion. Historical PNGs remain unchanged.

The harness now waits0.6seconds after Downloaded/Paused before screenshot. CPU images were actually viewed: `final-pause-paused.png` shows **일시 중단됨** and Resume enabled; `final-pause-final.png` and `final-success-final.png` show **최종 파일 확인 완료** with the synthetic final path and no running Cancel/Pause buttons. These are completion UI evidence as well as database evidence.

## Permanent evidence and independent verification

All files are under `evidence/accessible-cpu-download/`:

- `dlpad-cpu-download-20261003-final.json`: aggregate four-case acceptance.
- `...-final-{case}.json/.sqlite`: raw result and read-only SQLite backup.
- `...-final-{case}-initial.png/-final.png`, pause `-pause-ready.png/-paused.png`, close `-prompt.png`: actual owned-window captures where applicable.
- `...-final-failure-report.md`: actual copied redacted preview (1,423UTF8bytes), independently checked for absence of media.invalid/signature=fixture/fixture-secret.
- success/pause `-artifact.bin`:28bytes synthetic output.
- `evidence-manifest.json`: per-file sizes and SHA,440,447bytes total,2MiB individual/16MiB aggregate caps.

Independent read-only open of all four permanent databases returned integrity_check `ok`, expected final states and Attempt counts1/1/2/1. Some JSON paths intentionally preserve original temporary provenance; corresponding files are permanently available by basename here. Temporary working fixtures and all GUI/bus processes were cleaned after collection.

Actual top-level minimization and zero-sized native surface are **not verified by these cases**. Current Xvfb has no window manager; an iconify request does not establish an actual minimized OS state. A separate owned window-manager experiment and native surface-state observation would be required. No synthetic zero-size event or window unmapping is presented as an actual minimize test. Actual IME/screenreader, Windows/macOS CPU candidate, full library UI and eight-hour GUI are separate root-owned gates.

Existing two eight-hour runs were maintained: last actual host observation engine/DB8149.57seconds, guardian8112.96seconds, all nine PID/startticks matched. Archive remained watching at2026-10-02T15:10:53.982492UTC with PID6202/start48872 matching. These are intermediate observations, not eight-hour completion.
