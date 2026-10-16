#!/usr/bin/env python3
"""Bounded direct keeper cost only; not total systemd/service/application cost."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import tempfile
import time

PROJECT = Path(__file__).resolve().parent.parent
IDLE_SECONDS = 10
ROUNDS = 5
PROC_BOUND = 65536


def hash_file(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        while block := stream.read(65536):
            digest.update(block)
    return digest.hexdigest()


def bounded_read(path, limit=PROC_BOUND):
    with path.open('rb') as stream:
        data = stream.read(limit + 1)
    if len(data) > limit:
        raise RuntimeError('owned proc observation exceeded bound')
    return data


def identity(pid):
    text = bounded_read(Path(f'/proc/{pid}/stat'), 16384).decode('ascii')
    fields = text.rsplit(')', 1)[1].split()
    if fields[0] in ['Z', 'X']:
        raise RuntimeError('owned keeper not live')
    return {'pid': pid, 'start_ticks': int(fields[19]),
            'cpu_ticks': int(fields[11]) + int(fields[12]), 'state': fields[0]}


def require_identity(expected):
    now = identity(expected['pid'])
    if now['start_ticks'] != expected['start_ticks']:
        raise RuntimeError('owned keeper identity changed')
    return now


def memory_and_fds(expected):
    require_identity(expected)
    values = {}
    for line in bounded_read(Path(f"/proc/{expected['pid']}/smaps_rollup")).decode('ascii').splitlines():
        if ':' not in line:
            continue
        key, value = line.split(':', 1)
        if key in ['Rss', 'Pss']:
            parts = value.split()
            if len(parts) != 2 or parts[1] != 'kB':
                raise RuntimeError('owned smaps format unavailable')
            values[key.lower() + '_kib'] = int(parts[0])
    if set(values) != {'rss_kib', 'pss_kib'}:
        raise RuntimeError('owned RSS/PSS unavailable')
    count = 0
    for _ in Path(f"/proc/{expected['pid']}/fd").iterdir():
        count += 1
        if count > 4096:
            raise RuntimeError('owned FD count exceeded bound')
    require_identity(expected)
    return dict(values, fd_count=count)


def wait_owned(child, seconds):
    deadline = time.monotonic() + seconds
    while True:
        status = child.poll()
        if status is not None:
            return status
        if time.monotonic() >= deadline:
            raise TimeoutError('owned keeper exit deadline exceeded')
        time.sleep(.005)


def round_measure(helper, helper_identity, index):
    result = {'round': index, 'idle_target_seconds': IDLE_SECONDS}
    parent_fd_before = len(list(Path('/proc/self/fd').iterdir()))
    read_fd = write_fd = None
    child = None
    try:
        with tempfile.TemporaryDirectory(prefix='dlpad-keeper-cost-', dir='/tmp') as fixture:
            os.chmod(fixture, 0o700)
            read_fd, write_fd = os.pipe2(os.O_CLOEXEC)
            started = time.monotonic()
            child = subprocess.Popen([str(helper), '--dlpad-internal-cookie-keeper'],
                                     stdin=read_fd, stdout=subprocess.DEVNULL,
                                     stderr=subprocess.DEVNULL, close_fds=True, cwd=fixture)
            os.close(read_fd)
            read_fd = None
            expected = identity(child.pid)
            result['owned'] = {'pid': child.pid, 'start_ticks': expected['start_ticks']}
            ready_deadline = time.monotonic() + 2
            while True:
                require_identity(expected)
                actual_exe = Path(f'/proc/{child.pid}/exe').stat()
                if (actual_exe.st_dev, actual_exe.st_ino) != helper_identity:
                    raise RuntimeError('owned keeper executable identity changed')
                wchan = bounded_read(Path(f'/proc/{child.pid}/wchan'), 256).decode('ascii').strip()
                if 'pipe_read' in wchan:
                    break
                if time.monotonic() >= ready_deadline:
                    raise RuntimeError('owned keeper blocked read not observed')
                time.sleep(.01)
            cmdline = bounded_read(Path(f'/proc/{child.pid}/cmdline'), 4096).split(b'\0')
            if cmdline != [os.fsencode(helper), b'--dlpad-internal-cookie-keeper', b'']:
                raise RuntimeError('owned keeper argv identity mismatch')
            result['sample_before'] = memory_and_fds(expected)
            result['spawn_to_sample_seconds'] = time.monotonic() - started
            result['blocked_wchan'] = wchan
            before = require_identity(expected)
            idle_start = time.monotonic()
            # One idle interval, no periodic polling or watchdog sampling.
            time.sleep(IDLE_SECONDS)
            idle_end = time.monotonic()
            after = require_identity(expected)
            result['sample_after'] = memory_and_fds(expected)
            delta = after['cpu_ticks'] - before['cpu_ticks']
            if delta < 0:
                raise RuntimeError('owned CPU tick delta invalid')
            hz = os.sysconf('SC_CLK_TCK')
            result.update(idle_cpu_ticks_before=before['cpu_ticks'], idle_cpu_ticks_after=after['cpu_ticks'],
                          idle_cpu_ticks_delta=delta, clock_ticks_per_second=hz,
                          idle_monotonic_seconds=idle_end-idle_start,
                          idle_cpu_percent_single_core=100*delta/hz/(idle_end-idle_start))
            eof_start = time.monotonic()
            os.close(write_fd)
            write_fd = None
            status = wait_owned(child, 5)
            result.update(exit_code=status, eof_to_reap_seconds=time.monotonic()-eof_start)
            if status != 0:
                raise RuntimeError('owned keeper EOF exit unsuccessful')
            result['passed'] = True
    except Exception as error:
        result.update(passed=False, error_type=type(error).__name__, error=str(error))
    finally:
        for fd in [read_fd, write_fd]:
            if fd is not None:
                os.close(fd)
        if child is not None and child.poll() is None:
            try:
                result['cleanup_exit_code'] = wait_owned(child, 5)
            except TimeoutError:
                # Popen still owns this unreaped child; never signal a manager or foreign PID.
                child.kill()
                try:
                    result['cleanup_exit_code'] = wait_owned(child, 2)
                except TimeoutError:
                    result.update(passed=False, cleanup_unconfirmed=True)
        result['parent_fd_before'] = parent_fd_before
        result['parent_fd_after'] = len(list(Path('/proc/self/fd').iterdir()))
        if result['parent_fd_before'] != result['parent_fd_after']:
            result.update(passed=False, fd_cleanup_unconfirmed=True)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--helper', type=Path, default=PROJECT/'target/debug/dlpad-process-guard')
    args = parser.parse_args()
    output, helper = args.output, args.helper
    if not output.is_absolute() or '..' in output.parts or not output.is_relative_to(PROJECT/'docs/validation/evidence') or output.exists() or any(p.is_symlink() for p in [output, *output.parents]):
        parser.error('fresh absolute project evidence required')
    info = helper.lstat()
    if not helper.is_absolute() or not stat.S_ISREG(info.st_mode) or not os.access(helper, os.X_OK):
        parser.error('absolute regular executable helper required')
    snapshot = {'sha256': hash_file(helper), 'bytes': info.st_size, 'dev': info.st_dev, 'ino': info.st_ino}
    output.mkdir(mode=0o700)
    results = []
    for index in range(ROUNDS):
        result = round_measure(helper, (info.st_dev, info.st_ino), index)
        encoded = (json.dumps(result, indent=2)+'\n').encode()
        if len(encoded) > 65536:
            raise RuntimeError('round evidence exceeded bound')
        with (output/f'round-{index}.json').open('xb') as file:
            file.write(encoded)
        results.append(result)
        if not result['passed']:
            break
    final = helper.stat()
    unchanged = hash_file(helper) == snapshot['sha256'] and (final.st_dev, final.st_ino, final.st_size) == (info.st_dev, info.st_ino, info.st_size)
    summary = {'scope': 'direct keeper only; excludes systemd and application costs',
               'source_sha256': hash_file(Path(__file__)), 'source_bytes': Path(__file__).stat().st_size,
               'helper': snapshot, 'helper_unchanged': unchanged, 'rounds': len(results),
               'passed': unchanged and len(results) == ROUNDS and all(r['passed'] for r in results)}
    with (output/'result.json').open('x') as file:
        json.dump(summary, file, indent=2)
        file.write('\n')
    print(json.dumps(summary))
    return 0 if summary['passed'] else 2


if __name__ == '__main__':
    raise SystemExit(main())
