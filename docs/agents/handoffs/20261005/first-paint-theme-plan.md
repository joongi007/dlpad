# Windows S00/R06 first-visible-client theme audit

Scope: read-only audit; standalone collector proposal only. No product changes,
OS preference mutation, user desktop/clipboard/global input, native execution.
Read contract: README → PRODUCT → CONTEXT → R06 (REQUIREMENTS line12) →
ADR0001 native UI; current CPU software host/settings and Windows theme evidence.

## Current facts and exact gap

- software_host::run loads Candidate settings and applies ctx theme BEFORE event
  loop/window creation (lines485–488). Thus saved preference is not delayed until
  an asynchronous database load.
- AppearanceSettings::load_initial uses theme.sqlite readonly "theme" first,
  bounded legacy theme.json second; missing defaults System. Legacy seed is a
  JSON string "Light"/"Dark"/"System", not an object. An existing SQLite value
  overrides legacy. Each case needs fresh owned data, never user settings.
- resumed creates hidden window, surface and AccessKit, passes window.theme()
  into egui_winit::State, THEN set_visible(true) and request_redraw (349–382).
- pixels are rendered/presented only later in Host::paint (buffer.present line334).
  Therefore a visible-but-not-yet-presented client interval is possible. Current
  source does NOT prove a wrong theme flash; only a safe reproducible hypothesis.
- egui_winit handles OS ThemeChanged via input bridge; actual OS preference-change
  acceptance remains unmeasured. No synthetic callback is actual OS-event proof.
- windows-settings-{system-light,light-dark,dark-system}-20261004-resume
  root reviews validate choice persistence/natural exits/restart pixels; explicitly
  first_presentation_verified=false and actual_os_theme_change_verified=false.
  Existing ThemeObserve captures AFTER UIA/action/pump, so cannot close this gap.

## Smallest safe collector proposal

Reuse original owned private desktop, explicit STARTUPINFO.lpDesktop, Job and
original process HANDLE+creation FILETIME. Before ResumeThread, start observer
thread bound to that private desktop; record monotonic launch timestamp.
Enumerate only bounded desktop windows (maximum128/scan; deadline5s), requiring
exact PID, original retained process still matching creation and root top-level
HWND. Never select a guessed desktop/global foreground window. Once IsWindowVisible
and positive bounded client geometry, acquire owned HWND client DC and copy pixels
WITHOUT UIA, SetFocus, WM_PAINT, SendMessage, UpdateWindow, Pump or input injection.
Do not force app painting. Sample at most32 times, e.g. 5ms spacing, max160ms after
first visibility; retain first8 small lossless client tiles (96x96 max) plus up to
32 timestamp/color/glyph summaries. If sampling is missed, report INCOMPLETE,
never reconstruct a first frame from later pixels. Detect HWND/PID/creation each
sample, restore only observer's original thread desktop and original handles.

Seed three separate fresh fixtures via data/theme.json. Snapshot seed/data/source
hashes; no old DB. System record actual observed window/OS theme readonly and mark
unknown if API returnsNone, rather than guess Dark/Light. Sample a fixed interior
background tile away from border/titlebar; record raw BGRA histogram, varying
pixels and bounds. Constant untouched/black tiles are `prepaint_or_unknown`, NOT
proof of wrong-theme render. Compare first glyph-bearing tile and later settled
reference of SAME run; desktop/window background vs app paint is a distinct result.
Only after early sample sequence completes may UIA verify selected preference and
later full PNG; all phases keep original owned process and natural-exit cleanup.

## Acceptance and limits

For every theme: first visible client sample timestamps/raw pixels preserved,
first observed glyph-bearing client equals its settled theme, no opposite-theme
paint observed among32samples. Full screenshot verifies appearance. Any opposite
actual content paint is FAIL with exact sample; prepaint blank remains separate.
This is first OBSERVED visible client evidence, not guaranteed first presentation:
5ms sampling can miss a shorter flash. To prove every first presentation requires
an explicit perHost bounded paint receipt (original HWND/frame seq/theme/present
completion) or renderer instrumentation coordinated by root. No fixture hook yet.
Do not change visibility order until this causal evidence exists. A later root
fix could keep window hidden through first successful buffer.present, but must
preserve AccessKit initialization, repaint, resize and native HWND semantics.

Before implementation root should confirm private desktop GetDC/BitBlt captures
not showing placeholder under current compositor; unsupported capture stays
fail-closed, no visible fallback. Resource caps: <=32 samples/8PNG/128windows,
output <=1MiB, worker5s, original Job cleanup plus original input desktop unchanged.
