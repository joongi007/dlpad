#!/usr/bin/env python3
"""Run every bounded private S10 scenario with independent fresh fixtures."""
import argparse
import datetime
import json
import sys
from pathlib import Path

import windows_current_workspace_probe as n
import windows_s10_report_save_native_probe as scenario

CASES = (
    "save", "save-collision", "edit", "omit-one", "omit-all", "mpd-attach",
    "mpd-remove", "mpd-reattach", "mpd-all", "mpd-unsafe", "mpd-large",
)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    out = args.output
    if (not out.is_absolute() or ".." in out.parts or out.exists()
            or not out.is_relative_to(n.ROOT / "docs/validation/evidence")
            or any(p.is_symlink() for p in [out, *out.parents])):
        parser.error("fresh project evidence directory required")
    baseline = n.snapshot()
    pinned = {Path(p): n.sha(Path(p)) for p in (
        scenario.__file__, n.ROOT / "scripts/windows_s10_report_save_probe.cs",
        n.TOOLS / "target/release/dlpad-cpu.exe",
        n.TOOLS / "target/debug/examples/session_cookie_fixture_tool.exe",
        Path(__file__),
    )}
    out.mkdir(mode=0o700)
    results = []
    saved_argv = sys.argv
    try:
        for case in CASES:
            if baseline != n.snapshot() or any(n.sha(p) != h for p, h in pinned.items()):
                raise RuntimeError("suite original inputs changed")
            print("S10 suite case=" + case, flush=True)
            sys.argv = [scenario.__file__, str(out / case), "--scenario", case]
            code = scenario.main()
            raw_path = out / case / ("gui-all" if case == "omit-all" else "gui-single") / "result.json"
            collector = json.loads((out / case / "collector-result.json").read_text())
            gui_ran = any(r.get("operation", "").startswith("gui-")
                          for r in collector["operations"])
            cleanup_verified = not gui_ran
            if raw_path.exists():
                raw = json.loads(raw_path.read_text(encoding="utf-8-sig"))
                cleanup_verified = all(raw.get(k) is True for k in (
                    "owned_job_empty_after_cleanup", "owned_process_exited_after_job_close",
                    "input_desktop_unchanged",
                ))
            results.append({"scenario": case, "exit_code": code,
                            "passed": code == 0, "owned_cleanup_verified": cleanup_verified})
            (out / "progress.json").write_text(json.dumps({
                "recorded_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
                "results": results, "root_db_export_image_review_required": True,
            }, indent=2) + "\n")
            # A failing case has its own terminal cleanup; independent cases still run.
            if not cleanup_verified:
                break
    finally:
        sys.argv = saved_argv
    unchanged = baseline == n.snapshot() and all(n.sha(p) == h for p, h in pinned.items())
    passed = unchanged and len(results) == len(CASES) and all(r["passed"] for r in results)
    (out / "result.json").write_text(json.dumps({
        "passed": passed, "source_inputs_unchanged": unchanged, "results": results,
        "scope": "private S10 scenarios; root DB/export/image review required",
        "clipboard_verified": False, "closewait_verified_by_this_suite": False,
    }, indent=2) + "\n")
    return 0 if passed else 2


if __name__ == "__main__":
    raise SystemExit(main())
