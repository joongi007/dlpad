#!/usr/bin/env python3
"""Read-only Linux/WSL observations; create one fresh project evidence directory.

No signals, settings changes, environment dump, command-line dump or restart.
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import tempfile

PROJECT = Path(__file__).resolve().parent.parent
LIMIT = 65536


def valid_boot(value: object) -> bool:
    return isinstance(value, str) and bool(re.fullmatch(
        r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}", value))


def observation_scope() -> dict:
    namespaces = {}
    for name in ("self", "1"):
        try:
            meta = os.stat(f"/proc/{name}/ns/pid")
            namespaces[name] = {"device": meta.st_dev, "inode": meta.st_ino}
        except OSError as error:
            namespaces[name] = {"read_error": type(error).__name__}
    return {"pid_namespaces": namespaces,
            "host_visibility_verified": False,
            "absence_is_not_wsl_shutdown_proof": True,
            "note": "Compare authorized host observations before declaring an owned process absent."}


def plain(path: Path) -> Path:
    if not path.is_absolute() or ".." in path.parts:
        raise ValueError("absolute path required")
    for part in [path, *path.parents]:
        try:
            if stat.S_ISLNK(part.lstat().st_mode):
                raise ValueError("symlink rejected")
        except FileNotFoundError:
            pass
    return path


def read(path: Path, limit: int = LIMIT) -> str:
    plain(path)
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    try:
        content = os.read(fd, limit + 1)
        if len(content) > limit:
            raise ValueError("input bound exceeded")
        return content.decode("utf-8", errors="strict")
    finally:
        os.close(fd)


def command(argv: list[str]) -> dict:
    # Kernel/journal output is never copied in full. Temporary files bound the
    # retained read; commands below have an explicit small journal record bound.
    try:
        with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
            result = subprocess.run(argv, stdin=subprocess.DEVNULL, stdout=stdout,
                                    stderr=stderr, timeout=5, check=False,
                                    env={"PATH": "/usr/bin:/bin", "LC_ALL": "C"})
            stdout.seek(0)
            data = stdout.read(LIMIT + 1)
            if len(data) > LIMIT:
                return {"available": False, "reason": "output-bound"}
            return {"available": result.returncode == 0,
                    "exit_code": result.returncode, "text": data.decode("utf-8", "replace")}
    except (OSError, subprocess.TimeoutExpired) as error:
        return {"available": False, "reason": type(error).__name__}


def process(identity: dict, expected_boot: str, current_boot: str) -> dict:
    if not valid_boot(expected_boot) or not valid_boot(current_boot):
        raise ValueError("invalid boot identity")
    if expected_boot.lower() != current_boot.lower():
        return {"status": "different-boot-not-inspected"}
    pid, start = identity.get("pid"), identity.get("start_ticks")
    if not isinstance(pid, int) or isinstance(pid, bool) or pid < 1 or not isinstance(start, int) or isinstance(start, bool) or start < 1:
        raise ValueError("invalid explicit process identity")
    try:
        fields = read(Path(f"/proc/{pid}/stat"), 4096).rsplit(")", 1)[1].split()
        actual = int(fields[19])
        return {"pid": pid, "expected_start_ticks": start, "observed_start_ticks": actual,
                "status": "reused-pid" if actual != start else "zombie" if fields[0] == "Z" else "identity-matches-live"}
    except FileNotFoundError:
        return {"pid": pid, "expected_start_ticks": start, "status": "absent"}
    except (OSError, ValueError, IndexError) as error:
        return {"pid": pid, "expected_start_ticks": start,
                "status": "observation-unavailable", "read_error": type(error).__name__}


def snapshot(launch: Path | None) -> dict:
    boot = read(Path("/proc/sys/kernel/random/boot_id"), 128).strip()
    if not valid_boot(boot):
        raise ValueError("invalid kernel boot ID")
    now = dt.datetime.now(dt.timezone.utc)
    uptime = float(read(Path("/proc/uptime"), 256).split()[0])
    result = {"observed_utc": now.isoformat(), "boot_id": boot, "uptime_seconds": uptime,
              "estimated_boot_utc": (now - dt.timedelta(seconds=uptime)).isoformat(),
              "shutdown_requester": "unconfirmed", "host_watchdog_installed": False,
              "wsl_exit_monitoring": "not possible inside a terminated WSL VM",
              "observation_scope": observation_scope(),
              "document_pointers": ["docs/validation/wsl-interruption-recovery.md",
                                    "docs/validation/evidence/wsl-interruption-20261003/observation.json"]}
    boots = command(["/usr/bin/journalctl", "--list-boots", "--no-pager", "-n", "3"])
    result["journal_boots"] = boots
    previous = command(["/usr/bin/journalctl", "-k", "-b", "-1", "--no-pager", "-n", "1", "-o", "json"])
    text = previous.pop("text", "")
    previous["record_present"] = bool(text.strip())
    # Drop message, hostname, account and arbitrary properties.
    if text.strip():
        try:
            record = json.loads(text.splitlines()[-1])
            previous["record_realtime_us"] = record.get("__REALTIME_TIMESTAMP")
            previous["record_boot_id"] = record.get("_BOOT_ID")
        except (ValueError, TypeError):
            previous["parse_status"] = "unrecognized"
    result["previous_kernel_journal_access"] = previous
    interop = Path("/proc/sys/fs/binfmt_misc/WSLInterop")
    try:
        lines = read(interop, 4096).splitlines()
        result["wsl_interop"] = {"registered": True, "enabled": "enabled" in lines,
                                 "interpreter_is_init": "interpreter /init" in lines}
    except OSError as error:
        result["wsl_interop"] = {"registered": False, "read_error": type(error).__name__}
    windows = Path("/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe")
    result["windows_hidden_eventlog_collector"] = {"known_executable_present": windows.is_file(),
                                                   "eventlog_access": "not tested by this Linux snapshot"}
    if launch:
        launch = plain(launch)
        launch.relative_to(PROJECT / "docs/validation/evidence")
        data = json.loads(read(launch))
        expected = data.get("boot_id")
        if not valid_boot(expected):
            raise ValueError("launch boot ID required")
        result["launch_evidence"] = str(launch.relative_to(PROJECT))
        result["launch_boot_id"] = expected
        result["owned_processes"] = {role: process(data[role], expected, boot)
                                      for role in ("download_runner", "guardian_runner", "archive_owner") if role in data}
        result["launch_state_is_not_current_liveness"] = data.get("state")
        result["boot_comparison"] = "same-boot" if expected.lower() == boot.lower() else "different-boot"
        result["process_observations_are_not_terminal_proofs"] = True
        result["interruption_assessment"] = (
            "kernel-boot-changed; review preserved terminal evidence before classifying old work"
            if expected.lower() != boot.lower() else
            "same-boot; absent/zombie/reused PID does not distinguish completion from interruption")
    else:
        result["process_tracking"] = "not requested; no developer or runner liveness claim"
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--launch", type=Path)
    args = parser.parse_args()
    output = plain(args.output)
    output.relative_to(PROJECT / "docs/validation/evidence")
    if output == PROJECT / "docs/validation/evidence":
        raise ValueError("fresh child evidence directory required")
    result = snapshot(args.launch)
    encoded = (json.dumps(result, indent=2, ensure_ascii=True) + "\n").encode()
    if len(encoded) > LIMIT:
        raise ValueError("result bound exceeded")
    output.mkdir(mode=0o700, parents=False, exist_ok=False)
    fd = os.open(output / "snapshot.json", os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "wb") as stream:
        stream.write(encoded)
        stream.flush()
        os.fsync(stream.fileno())
    print(json.dumps({"created": str(output / "snapshot.json"), "boot_id": result["boot_id"]}))


if __name__ == "__main__":
    main()
