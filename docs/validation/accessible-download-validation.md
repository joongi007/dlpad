# Accessible native candidate download acceptance

Status: completed for this bounded Linux candidate vertical slice (four GUI cases passed); complete P0 and stack adoption remain root decisions. Owned Xvfb only, temporary synthetic media/tool/data, no user desktop/clipboard/network/actual media. Candidate invocation is ordinary native executable with DLPAD_TEST_DATA/LIBRARY_ROOT/YTDLP and font `/mnt/c/Windows/Fonts/malgun.ttf`; no product test hook.

Initial ready snapshot SHA `0006783276df0485f25bfeec0af805389cb81ac0e35d9b25116243cf9c823dd1`; accessible.rs SHA `2469d3908c6f03e22fb6d5b753da58c63de60940dd5520497083b37a4e82b84c`. Actual commands:

```text
python3 scripts/accessible_download_probe.py target/release/dlpad-accessible /tmp/dlpad-accessible-20261002-inspect.json --case inspect --font /mnt/c/Windows/Fonts/malgun.ttf
python3 scripts/accessible_download_probe.py target/release/dlpad-accessible /tmp/dlpad-accessible-20261002-first.json --case all --font /mnt/c/Windows/Fonts/malgun.ttf
python3 scripts/accessible_download_probe.py target/release/dlpad-accessible /tmp/dlpad-accessible-20261002-pause2.json --case pause --font /mnt/c/Windows/Fonts/malgun.ttf
python3 scripts/accessible_download_probe.py target/release/dlpad-accessible /tmp/dlpad-accessible-20261002-close2.json --case close --font /mnt/c/Windows/Fonts/malgun.ttf
```

Initial inspection exit0 (inspection, not acceptance). First all exit1: success and failure/report-copy PASS, pause and close timeout. Actual screenshot determined StopExit dialog button center `(65,146)`; corrected close2 PASS. Pause2 used actual button center `(112,300)` and 0.5-second ready stabilization but again persisted Cancelled. Read-only source confirmed pause branch set pause=true then called request_stop(false), which overwrote pause intent. Product error proposal sent to root; root owns source correction/rebuild. This was not dismissed as input failure. Revised product requires a fresh full trial before acceptance.

Historical evidence copied under `evidence/accessible-download-candidate/` (451,670 bytes initially): `dlpad-accessible-20261002-first-success.json/-final.png`, `first-failure.json/-final.png`, `pause2-pause.json/-pause-ready.png/-error.png`, `close2-close.json/-prompt.png`. Failure clipboard verification read only owned Xvfb selection and checked diagnostic header/redacted source/signature; 1,423 UTF-8 bytes. Historical temporary SQLite files were cleaned with fixture lifetime, so those trials preserve queried JSON rather than raw database.

Current harness adds bounded read-only SQLite evidence backup (2 MiB, three-second deadline), synthetic artifact evidence (64 KiB cap), and actual redacted report preview from owned Xvfb clipboard (256 KiB cap). No user's clipboard is accessed. GUI output retained at64 KiB while extra stream bytes are drained. Worker PID/start ticks/token and script argv are verified before pidfd pinning; teardown uses only owned process session and pinned worker. It preserves same Job/new Attempt/partial bytes/original URL re-extraction for resume. No actual IME/screenreader, Windows/macOS candidate behavior or full P0 completion is claimed.


## Corrected snapshot final acceptance

Root corrected only Pause to `worker.request_stop(true)` and provided new ready binary SHA `55ecdbc89c4eb42c4e898e939e7ca164e94ed5359848a903c7d58d77a452401f`. Actual final command:

```text
python3 scripts/accessible_download_probe.py target/release/dlpad-accessible /tmp/dlpad-accessible-20261002-final.json --case all --font /mnt/c/Windows/Fonts/malgun.ttf
```

Exit **0, all four cases PASS**. Success: GUI URL input launched pinned owned synthetic tool and created one final artifact containing `synthetic candidate artifact`. Failure: exit7 resulted in persistent Failed and redacted diagnostic preview/copy. Pause: actual Pause button led to Paused with unchanged64KiB partial; Resume retained original Job ID, appended distinct second Attempt, re-extracted original URL twice and completed. Close: native WM_DELETE_WINDOW prompted StopExit; CancelWait persisted Cancelled, app exited0 and pinned worker disappeared. All four workers stopped after teardown, no pidfd kill fallback recorded.

Raw databases were copied with bounded read-only SQLite backup before temporary fixture cleanup. Independent opening of the permanent evidence returned integrity_check `ok` for all four:

| Case | Persisted final state | Attempts | Artifacts |
|---|---|---:|---:|
| success | Downloaded | 1 | 1 |
| failure | Failed | 1 | 0 |
| pause | Downloaded | 2 | 1 |
| close | Cancelled | 1 | 0 |

`evidence/accessible-download-candidate/dlpad-accessible-20261002-final.json` is the aggregate result. Per-case `...-final-{case}.json` records original binary/source/fixture hashes and owned app/worker PID/start ticks; `.sqlite` is the permanent database copy, and `-initial.png/-final.png/-pause-ready.png/-prompt.png` are actual owned-window screenshots where applicable. `...-final-failure-report.md` contains the **actual copied owned-Xvfb clipboard preview**,1,423UTF8 bytes. Independent preview inspection confirms no media.invalid, signature=fixture or fixture-secret. Success/pause `-artifact.bin` preserve only28 bytes of owned synthetic output. `final-evidence-manifest.json` records per-file SHA and sizes; historical plus final evidence totals873,013 bytes (16MiB cap, individual2MiB cap).

Some JSON file paths retain original `/tmp` fixture/evidence names for provenance; corresponding permanent copies are present by basename under the evidence directory. Temporary owned working fixtures were cleaned after the snapshot and GUI/worker shutdown. Original failed pause/close trials remain preserved and are not retrospectively labeled success. The product Pause correction belongs to root; this agent changed only its assigned harness/workitem/validation. Actual IME/screenreader/accessibility, full UI coverage, Windows/macOS candidate behavior, real media and eight-hour full GUI remain separate validation.

## Historical capture/isolation correction

The original Glow `-final.png` naming denotes capture after database observation, not proof of UI completion paint: the pause final PNG still showed running buttons before its completion event was consumed. Original evidence is preserved unchanged. That harness also did not replace inherited session/accessibility bus variables, so bus/reader isolation was not demonstrated for historical Glow runs. [CPU acceptance](accessible-cpu-download-validation.md) uses private buses/0700 XDG, adds0.6second paint stabilization and independently confirms actual Paused/final-confirmed UI. Database/worker/isolated-Xvfb clipboard evidence of the historical trials is unchanged.
