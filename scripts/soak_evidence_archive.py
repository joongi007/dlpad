#!/usr/bin/env python3
"""Bounded evidence for explicitly owned synthetic Linux soaks; never signals processes."""
import argparse
import datetime
import hashlib
import json
import math
import os
from pathlib import Path
import stat
import time
import uuid

FILE_LIMIT = 1024 * 1024
TOTAL_LIMIT = 8 * FILE_LIMIT
HISTORY_LIMIT = 512
PROCESS_ROLES = {
    "download_runner", "download_owner", "download_guardian", "download_worker",
    "guardian_owner", "guardian_guardian", "guardian_root", "guardian_middle",
    "guardian_leaf", "guardian_detached",
}
JSON_LIMIT = 65536
ALLOWLIST = {
    "download": {"status.json": "download-status.json",
                 "result.json": "download-result.json",
                 "long-job/worker-status.json": "download-worker-status.json",
                 "long-job/samples.jsonl": "download-samples.jsonl"},
    "runner": {"status.json": "download-runner-status.json",
               "result.json": "runner-result.json", "test.log": "download-test.log"},
    "guardian": {"status.json": "guardian-status.json", "result.json": "guardian-result.json",
                 "samples.jsonl": "guardian-samples.jsonl", "ownership.json": "guardian-ownership.json",
                 "guardian.log": "guardian.log"},
}


class EvidenceError(Exception):
    pass


def utc_now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def check_path(path, directory=True, owned=True):
    path = Path(path)
    if not path.is_absolute() or ".." in path.parts:
        raise EvidenceError("absolute normalized path required")
    for ancestor in [*reversed(path.parents), path]:
        info = ancestor.lstat()
        if stat.S_ISLNK(info.st_mode):
            raise EvidenceError("symlink path rejected")
    info = path.lstat()
    if directory and not stat.S_ISDIR(info.st_mode):
        raise EvidenceError("directory required")
    if owned and info.st_uid != os.getuid():
        raise EvidenceError("ownership mismatch")
    return path


def open_dir(path):
    check_path(path)
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    info = os.fstat(fd)
    if info.st_uid != os.getuid():
        os.close(fd)
        raise EvidenceError("ownership mismatch")
    return fd


def source_fd(root_fd, relative):
    pieces = relative.split("/")
    parent = os.dup(root_fd)
    try:
        for component in pieces[:-1]:
            child = os.open(component, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=parent)
            os.close(parent)
            parent = child
            if os.fstat(parent).st_uid != os.getuid():
                raise EvidenceError("source directory ownership mismatch")
        fd = os.open(pieces[-1], os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW, dir_fd=parent)
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid():
            os.close(fd)
            raise EvidenceError("owned regular source required")
        return fd
    finally:
        os.close(parent)


def read_json(root_fd, relative, missing=None, expected_sha256=None):
    try:
        fd = source_fd(root_fd, relative)
    except FileNotFoundError:
        return missing
    with os.fdopen(fd, "rb") as stream:
        data = stream.read(JSON_LIMIT + 1)
    if len(data) > JSON_LIMIT:
        raise EvidenceError("JSON exceeds bound")
    if expected_sha256 is not None and hashlib.sha256(data).hexdigest() != expected_sha256:
        raise EvidenceError("copied terminal snapshot changed")
    try:
        value = json.loads(data)
    except (ValueError, UnicodeError) as error:
        raise EvidenceError("invalid JSON") from error
    if not isinstance(value, dict):
        raise EvidenceError("JSON object required")
    return value


def atomic_bytes(dest_fd, name, data):
    if len(data) > FILE_LIMIT:
        raise EvidenceError("publication exceeds file bound")
    temporary = ".archive-" + uuid.uuid4().hex + ".tmp"
    try:
        fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=dest_fd)
        with os.fdopen(fd, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.rename(temporary, name, src_dir_fd=dest_fd, dst_dir_fd=dest_fd)
    finally:
        try:
            os.unlink(temporary, dir_fd=dest_fd)
        except FileNotFoundError:
            pass


def encoded(value):
    return (json.dumps(value, ensure_ascii=True, indent=2) + "\n").encode()


def process_observation(pid, expected):
    try:
        fields = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()
        actual = int(fields[19])
        zombie = fields[0] == "Z"
    except (OSError, ValueError, IndexError):
        return {"pid": pid, "expected_start_ticks": expected, "actual_start_ticks": None, "matches": False}
    return {"pid": pid, "expected_start_ticks": expected, "actual_start_ticks": actual,
            "matches": actual == expected and not zombie}


def validate_manifest(value, roots):
    if set(value) != {"boot_id", "roots", "processes"}:
        raise EvidenceError("ownership manifest keys invalid")
    boot = value["boot_id"]
    if not isinstance(boot, str) or len(boot) != 36:
        raise EvidenceError("boot ID required")
    try:
        if str(uuid.UUID(boot)) != boot:
            raise ValueError("noncanonical boot ID")
    except ValueError as error:
        raise EvidenceError("canonical boot UUID required") from error
    if value["roots"] != {name: str(path) for name, path in roots.items()}:
        raise EvidenceError("explicit roots disagree with manifest")
    processes = value["processes"]
    if not isinstance(processes, dict) or not 1 <= len(processes) <= 32:
        raise EvidenceError("bounded process ownership required")
    for name, item in processes.items():
        if name not in PROCESS_ROLES:
            raise EvidenceError("known scoped process role required")
        if not isinstance(item, dict) or set(item) != {"pid", "start_ticks"}:
            raise EvidenceError("process identity required")
        if any(type(item[key]) is not int or item[key] <= 0 for key in item):
            raise EvidenceError("positive integer PID/start required")


def finite_eight_hours(value):
    elapsed = value.get("elapsed_seconds")
    return type(elapsed) in (int, float) and math.isfinite(elapsed) and elapsed >= 28800


def scope_terminal_states(download, runner, guardian):
    # Terminal proof belongs to its process group. Groups can finish in either order.
    download_state = None
    for result in (download, runner):
        if result is not None and result.get("state") != "completed":
            download_state = "failed"
    if download is not None and not (download.get("eight_hours_verified") is True and finite_eight_hours(download)):
        download_state = "failed"
    if runner is not None and not (type(runner.get("exit_code")) is int and runner["exit_code"] == 0):
        download_state = "failed"
    if download_state is None and download is not None and runner is not None:
        download_state = "completed"
    guardian_state = None
    if guardian is not None:
        valid = (guardian.get("state") == "completed" and guardian.get("eight_hours_elapsed") is True
                 and guardian.get("descendants_reaped") is True and finite_eight_hours(guardian))
        guardian_state = "completed" if valid else "failed"
    return {"download": download_state, "guardian": guardian_state}


def terminal_state(download, runner, guardian):
    scopes = scope_terminal_states(download, runner, guardian)
    if "failed" in scopes.values():
        return "failed"
    return "completed" if all(state == "completed" for state in scopes.values()) else None


class Archive:
    def __init__(self, manifest, roots, destination):
        validate_manifest(manifest, roots)
        self.manifest = manifest
        self.roots = roots
        self.source_dirs = {}
        self.destination = Path(destination)
        project = Path(__file__).absolute().parent.parent
        evidence = project / "docs/validation/evidence"
        if not self.destination.is_relative_to(evidence) or self.destination == evidence:
            raise EvidenceError("destination must be a new project evidence directory")
        check_path(self.destination.parent)
        if self.destination.exists() or self.destination.is_symlink():
            raise EvidenceError("destination already exists")
        for path in roots.values():
            if not path.is_absolute() or path.parent != Path("/tmp") or ".." in path.parts:
                raise EvidenceError("source roots must be direct /tmp paths")
            try:
                check_path(path)
            except FileNotFoundError:
                pass
        self.destination.mkdir(mode=0o700)
        self.dest_fd = open_dir(self.destination)
        self.history = []
        self.files = {}
        try:
            atomic_bytes(self.dest_fd, "ownership-manifest.json", encoded(manifest))
        except Exception:
            os.close(self.dest_fd)
            raise

    def close(self):
        for fd in self.source_dirs.values():
            os.close(fd)
        self.source_dirs.clear()
        os.close(self.dest_fd)

    def connect_sources(self):
        for name, path in self.roots.items():
            if name in self.source_dirs:
                try:
                    info = path.lstat()
                except FileNotFoundError:
                    os.close(self.source_dirs.pop(name))
                    continue
                old = os.fstat(self.source_dirs[name])
                if stat.S_ISLNK(info.st_mode) or (info.st_dev, info.st_ino) != (old.st_dev, old.st_ino):
                    raise EvidenceError("owned root replaced")
                continue
            try:
                self.source_dirs[name] = open_dir(path)
            except FileNotFoundError:
                continue

    def check_destination(self):
        info = self.destination.lstat()
        old = os.fstat(self.dest_fd)
        if stat.S_ISLNK(info.st_mode) or (info.st_dev, info.st_ino) != (old.st_dev, old.st_ino):
            raise EvidenceError("destination root replaced")
        allowed = {target for files in ALLOWLIST.values() for target in files.values()}
        allowed.update({"ownership-manifest.json", "progress-history.jsonl", "archive-status.json", "archive-result.json"})
        for name in os.listdir(self.dest_fd):
            info = os.stat(name, dir_fd=self.dest_fd, follow_symlinks=False)
            if name not in allowed or not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_nlink != 1:
                raise EvidenceError("unexpected or unsafe destination entry")

    def copy_file(self, source, relative, target):
        try:
            source_fd_value = source_fd(source, relative)
        except FileNotFoundError:
            return
        temporary = ".archive-" + uuid.uuid4().hex + ".tmp"
        try:
            with os.fdopen(source_fd_value, "rb") as stream:
                initial = os.fstat(stream.fileno())
                if initial.st_size > FILE_LIMIT:
                    self.files[target] = {**self.files.get(target, {}), "omitted": "file limit", "source_bytes": initial.st_size}
                    return
                total = sum(item.stat().st_size for item in self.destination.iterdir())
                old_size = (self.destination / target).stat().st_size if (self.destination / target).exists() else 0
                if total - old_size + initial.st_size > TOTAL_LIMIT - 2 * FILE_LIMIT:
                    self.files[target] = {**self.files.get(target, {}), "omitted": "aggregate reserve"}
                    return
                fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=self.dest_fd)
                digest = hashlib.sha256()
                count = 0
                with os.fdopen(fd, "wb") as output:
                    while True:
                        chunk = stream.read(min(65536, FILE_LIMIT + 1 - count))
                        if not chunk:
                            break
                        count += len(chunk)
                        if count > FILE_LIMIT:
                            self.files[target] = {**self.files.get(target, {}), "omitted": "file grew past limit"}
                            return
                        digest.update(chunk)
                        output.write(chunk)
                    output.flush()
                    os.fsync(output.fileno())
                os.rename(temporary, target, src_dir_fd=self.dest_fd, dst_dir_fd=self.dest_fd)
                self.files[target] = {"bytes": count, "sha256": digest.hexdigest()}
                return True
        finally:
            try:
                os.unlink(temporary, dir_fd=self.dest_fd)
            except FileNotFoundError:
                pass

    def step(self):
        current_boot = Path("/proc/sys/kernel/random/boot_id").read_text().strip()
        boot_matches = current_boot == self.manifest["boot_id"]
        self.check_destination()
        copied = set()
        if boot_matches:
            self.connect_sources()
            for name, source in self.source_dirs.items():
                # Copy only explicitly owned allowlist entries, before deriving
                # terminal state. A source result can appear while copying.
                for relative, target in ALLOWLIST[name].items():
                    if self.copy_file(source, relative, target):
                        copied.add(target)
        terminal_files = ("download-result.json", "runner-result.json", "guardian-result.json")
        results = [read_json(self.dest_fd, target, expected_sha256=self.files[target]["sha256"])
                   if target in copied else None for target in terminal_files]
        # Observe identities after the potentially long copy. A live observation
        # before copying must not excuse an unexpected death during copying.
        current_boot = Path("/proc/sys/kernel/random/boot_id").read_text().strip()
        boot_matches = current_boot == self.manifest["boot_id"]
        if not boot_matches:
            results = [None, None, None]
        processes = {name: process_observation(item["pid"], item["start_ticks"])
                     for name, item in self.manifest["processes"].items()}
        for item in processes.values():
            item["matches"] = boot_matches and item["matches"]
        scopes = scope_terminal_states(*results)
        terminal = terminal_state(*results)
        for role, item in processes.items():
            scope = role.split("_", 1)[0]
            item["expected_terminal_exit"] = boot_matches and scopes[scope] == "completed" and not item["matches"]
        state = "interrupted" if not boot_matches else terminal
        if state is None:
            expected = all(item["matches"] or item["expected_terminal_exit"] for item in processes.values())
            state = "watching" if len(self.source_dirs) == 3 and expected else "interrupted"
        value = {"observed_utc": utc_now(), "state": state, "boot_id": current_boot,
                 "expected_boot_id": self.manifest["boot_id"], "processes": processes,
                 "scope_terminal_proofs": scopes, "files": self.files, "source_roots_present": sorted(self.source_dirs),
                 "terminal_snapshot_files": {target: self.files[target] for target in terminal_files if target in copied},
                 "copy_limit_per_file": FILE_LIMIT, "copy_total_limit": TOTAL_LIMIT,
                 "history_limit": HISTORY_LIMIT, "process_actions": []}
        self.history.append({key: value[key] for key in ("observed_utc", "state", "boot_id", "processes")})
        self.history = self.history[-HISTORY_LIMIT:]
        history_data = b"".join(json.dumps(item, separators=(",", ":")).encode() + b"\n" for item in self.history)
        # Byte bound is stronger than row bound when many explicit identities are supplied.
        while len(history_data) > FILE_LIMIT:
            self.history.pop(0)
            history_data = b"".join(json.dumps(item, separators=(",", ":")).encode() + b"\n" for item in self.history)
        atomic_bytes(self.dest_fd, "progress-history.jsonl", history_data)
        atomic_bytes(self.dest_fd, "archive-status.json", encoded(value))
        if state != "watching":
            atomic_bytes(self.dest_fd, "archive-result.json", encoded(value))
        total = sum(item.stat().st_size for item in self.destination.iterdir())
        if total > TOTAL_LIMIT:
            raise EvidenceError("aggregate bound violated")
        return value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ownership", type=Path, required=True)
    parser.add_argument("--download-root", type=Path, required=True)
    parser.add_argument("--guardian-root", type=Path, required=True)
    parser.add_argument("--destination", type=Path, required=True)
    parser.add_argument("--interval", type=float, default=60)
    parser.add_argument("--once", action="store_true")
    args = parser.parse_args()
    if not 1 <= args.interval <= 60:
        parser.error("interval must be 1..60 seconds")
    roots = {"download": args.download_root,
             "runner": args.download_root.with_name(args.download_root.name + "-runner"),
             "guardian": args.guardian_root}
    for path in roots.values():
        if not path.is_absolute() or path.parent != Path("/tmp") or ".." in path.parts:
            parser.error("explicit source roots must be direct /tmp paths")
    archive = None
    try:
        check_path(args.ownership, directory=False)
        parent_fd = open_dir(args.ownership.parent)
        try:
            manifest = read_json(parent_fd, args.ownership.name)
        finally:
            os.close(parent_fd)
        archive = Archive(manifest, roots, args.destination)
        while True:
            value = archive.step()
            print(json.dumps({key: value[key] for key in ("observed_utc", "state", "boot_id")}), flush=True)
            if value["state"] != "watching" or args.once:
                return 0 if value["state"] in ("watching", "completed") else 1
            time.sleep(args.interval)
    except (EvidenceError, OSError) as error:
        print(json.dumps({"state": "failed", "error": str(error), "process_actions": []}), flush=True)
        return 1
    finally:
        if archive is not None:
            archive.close()


if __name__ == "__main__":
    raise SystemExit(main())
