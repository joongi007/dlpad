# CPU typed options: actual Linux GUI

Frozen executable SHA `2459ba126c6bcc13470b470c8a384d9d8ecef779893fd832411db0a78932f559`,10,508,440 bytes. Product before/after identical: accessible.rs `c04febee10154285b3d12fea72b48a8800f929dda5de1400d4efcb4671ac4063`, accessible_options.rs `cbbdfb5a7dd5b3957a71a8df629e749184e57618fd5657e73e9cb73fa0452dd8`. Product/Cargo/shared helpers were not changed by this harness work.

Actual commands:

```sh
python3 scripts/cpu_options_probe.py --binary target/release/dlpad-cpu --output docs/validation/evidence/cpu-options/inspect-1 --case inspect
python3 scripts/cpu_options_probe.py --binary target/release/dlpad-cpu --output docs/validation/evidence/cpu-options/all-1 --case all
python3 scripts/cpu_options_probe.py --binary target/release/dlpad-cpu --output docs/validation/evidence/cpu-options/subtitles-clip-fixed --case subtitles
python3 scripts/cpu_options_probe.py --binary target/release/dlpad-cpu --output docs/validation/evidence/cpu-options/conflict-clip-fixed --case conflict
```

Inspect exit0. Firstall exit1:9cases passed and2cases failed because the harness clicked clipped content underneath fixed actions. Actual subtitle extents y588(height32) overlapped Start at y585; screenshot still showed no subtitle selected. Its default download had started accidentally. The conflict failure had the same boundary. These raw failures and SQLite are preserved. The corrected harness limits content centers below575 while allowing actual fixed-action extents to630; widget coordinates always come from owned AT-SPI, not fixed field coordinates. Only those two original missing cases were re-tested, each exit0. No unbounded retry or product workaround.

## Actual acceptance

- Best, Height720 and Audio each reached Downloaded with matching typed SQLite and --format argv. Audio selects bestaudio/best; no format conversion or absence-of-video guarantee.
- Fast/Accurate range1.001→10.123 seconds reached Downloaded and exact --download-sections; force-keyframes-at-cuts only on accurate. This verifies typed argv and synthetic publication, not actual media cutting/FFmpeg output accuracy.
- KoreanAndEnglish plus auto=true emitted ko,en/write-auto-subs and published two16byte synthetic VTT sidecars with the28byte synthetic media artifact. Other language/auto=false combinations and None re-selection reset rely on existing module tests, not this GUI experiment.
- Advanced1048576B/s/retries0/fragments16 appeared in actual argv; retries appeared once, matching DB options. Height143 and retries21 started no worker and created no job/Attempt. Range+subtitle conflict likewise created no worker/job after corrected input. Other upper bounds/overflow cases are unit-tested, not all GUI-tested.
- Active URL field AT-SPI state lacked Enabled and Sensitive, and active PNG showed disabled controls. This is a representative active-field probe, not an independent state query for every widget. Actual Pause reached Paused; changing visible quality to Audio then Resume preserved the original Height720 snapshot, same job/new second Attempt and matching both worker argvs. Persisted resume policy is independent of edits to the current form.
- All successful cases exited0 after owned WM_DELETE_WINDOW; all case GUI/bus/drainer cleanup flags true. Exact app/bus/worker PID+start identities were independently absent afterward. Saved SQLite integrity checks all ok. All four exec sessions are terminal: inspect34064 exit0, firstall31705 exit1, subtitle22596 exit0, conflict86206 exit0.

AT-SPI role79 text fields and role43 buttons were directly inspected; automatic subtitle checkbox used actual role7. Query traversal caps256nodes and scroll attempts16. TextEdit extents32logicalpx observed. Owned Xvfb, private D-Bus/AT-SPI,0700task XDG/data and bounded120second/64KiB fixture inputs were used. Clipboard, actual media, accounts, network and user display were unused. Final configured/rejected/paused screenshots were directly viewed. Scope is Linux CPU candidate only; Windows/macOS typed options GUI remains unverified. Reports/options are snapshots of synthetic tools, not proof of real site availability.

`evidence/cpu-options/` preserves raw case JSON, actualargv, SQLite, PNG, source-before/after and summary.json. Artifact bytes themselves were not separately copied after temp cleanup; typed state/metadata and tool fixture recipe are retained. All remaining work is deferred under the user's pause instruction. Existing8hour runs were left unchanged and remain a separate pending gate.
