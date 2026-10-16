# S10 CPU diagnostics: actual Linux native GUI

Actual final command:

```sh
python3 scripts/cpu_diagnostics_probe.py --binary target/release/dlpad-cpu --output docs/validation/evidence/cpu-diagnostics/full-safe-words
```

Exit0 on production CPU SHA `8fb4a67c27ff89fd883ff6fddaa9136a5fc36c62d74c049665543a513ab06986`,10,437,144 bytes. Source frozen throughout final trials; full-1/source-before.sha256 and full-safe-words/source-after.sha256 match: accessible.rs `cea30b6cf742dfa24b938d4c9a6ab16d4edc9227e1de7acc55bd23f3fe21a429`, accessible_diagnostics.rs `1c31efce92af339bf8460d593b390bc925c62376b725df4fcbc15a665fc29365`. Earlier initial inspect preceded root's cfg(test)-only fixture naming fix; that source distinction is preserved, while executable remained unchanged.

## Actual acceptance

Owned synthetic failure created Failed job `job-1790958133290345524`, one attempt, no artifacts; SQLite integrity ok independently after exit. AT-SPI queried actual owned widget bounds and Xvfb wheel scrolling reached the300pixel controls viewport. Expanded report and both Markdown/JSON were copied through only the new private X11 clipboard and preserved as raw files.

Three plain-word fixture logs were retained. Editing log0 to a synthetic Authorization/URL secret and applying re-sanitization reduced logs3→2, with both synthetic secrets absent from JSON. Individual omit reduced2→1; omit all reduced1→0. The raw editor still shows user-entered synthetic text: it is the editable input, not exported sanitized report content. The report JSON provides the redaction evidence.

A local MPD containing a synthetic sensitive BaseURL and byte ranges attached one structure-only representation; synthetic secret absent, range values retained. Actual individual MPD omit produced0 attachments, reattach restored1, and omit all produced0. DTD attachment recorded UnsafeXml;1MiB+1-byte fixture recorded TooLarge while existing good structure stayed intact. All three source fixture hashes stayed identical. These are deliberate boundary bytes, not actual media or user files. Fresh JSON export succeeded; second export to the same path preserved identical original bytes. Export/report data is only local fixture evidence.

WM_DELETE_WINDOW exercised the app's diagnostic-worker stop acknowledgement/CloseWait and exited0. AppPID173563/start1317755 and private busPID173535/start1317649 were independently absent afterward, log drain threads reaped. Xvfb was owned and closed. No user display, user clipboard/bus, credentials, media, network transmission or visible fallback was used.

## Preserved failures and scope

inspect-1 reached actual Failed UI, then a harness expansion label mismatch failed; capture after display teardown caused Xlib exit. Product failure was not claimed. The leftover private bus group139197/start1255851 was matched by its unique temp path/group and terminated; its descendants were verified absent. Cleanup order was corrected to drain/close owned connections before stopping Xvfb.

full-2 completed attachment/export/omission/CloseWait, but its initial single secret log was rightly omitted, so it did not validate log editing. full-safe-logs added ERROR: prefix lines; the colon fail-closed policy correctly omitted them too, and a new logs-count assertion caught the incomplete fixture. These raw failures remain. finalfull-safe-words used three plain-word lines and completed all log-count assertions. No unsuccessful test is counted as successful.

Evidence under `evidence/cpu-diagnostics/` was1,081,689 bytes before documentation metadata; each report/SQLite≤2MiB, bounded process logs≤64KiB, fixture MPD read boundary1MiB+1. Raw final JSON/Markdown/export/SQLite/PNG and intermediate trial JSON/PNG retained. Screenshots expanded, attached, oversize and omitted were directly viewed. This proves the tested small report and MPD size boundary, not a2MiB report-limit GUI stress test. Clipboard reading has a256KiB cap, sufficient for these small reports. Native Windows/macOS S10 GUI, real authentication/media and external sharing remain unverified. Editor input budgeting and full malformed-XML coverage additionally depend on existing automated tests; this GUI experiment does not replace them.
