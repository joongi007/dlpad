#!/usr/bin/env python3
"""Repeat the unchanged native moved-leader regression with frozen adjacent binaries."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time

PROJECT = Path(__file__).resolve().parent.parent
NAME = 'leader_that_changes_group_is_still_owned_and_reaped'

def sha(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(65536), b''): value.update(block)
    return value.hexdigest()

def identity(pid):
    try:
        text = Path(f'/proc/{pid}/stat').read_text()
        if len(text) > 4096: return None
        fields = text.rsplit(')', 1)[1].split()
        return {'pid': pid, 'start_ticks': int(fields[19]), 'parent_pid': int(fields[1]), 'state': fields[0]}
    except (OSError, ValueError, IndexError): return None

def same(value):
    current = identity(value['pid'])
    return current is not None and current['start_ticks'] == value['start_ticks']

def observe(root, known):
    pending = [root]
    while pending:
        item = pending.pop()
        if not same(item): continue
        raw_parts = []
        try:
            with os.scandir(f"/proc/{item['pid']}/task") as tasks:
                count = 0
                for task in tasks:
                    count += 1
                    if count > 64: raise RuntimeError('owned task capacity exceeded')
                    if not task.name.isdigit(): continue
                    try: raw = Path(task.path, 'children').read_text()
                    except OSError: continue
                    if len(raw) > 4096: raise RuntimeError('owned inventory read exceeded bound')
                    raw_parts.append(raw)
        except FileNotFoundError: continue
        raw = ' '.join(raw_parts)
        for word in raw.split():
            child = identity(int(word))
            if child is None or child['parent_pid'] != item['pid']: continue
            key = (child['pid'], child['start_ticks'])
            if key not in known:
                if len(known) >= 64: raise RuntimeError('owned inventory capacity exceeded')
                known[key] = child
            pending.append(child)

def run(driver, folder, number, suite=False, cleanup_stage_trace=False):
    start = time.monotonic()
    command = [str(driver), '--nocapture'] if suite else [str(driver), '--exact', NAME, '--nocapture']
    env = dict(os.environ)
    env.pop('DLPAD_TEST_CLEANUP_STAGE_TRACE', None)
    if cleanup_stage_trace: env['DLPAD_TEST_CLEANUP_STAGE_TRACE'] = '1'
    child = subprocess.Popen(command, env=env, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    pin = os.pidfd_open(child.pid)
    root = identity(child.pid)
    known = {}
    output = bytearray()
    oversized = False
    timed_out = False
    inventory_error = None
    os.set_blocking(child.stdout.fileno(), False)
    try:
        while child.poll() is None:
            if root is not None and inventory_error is None:
                try: observe(root, known)
                except RuntimeError as error: inventory_error = str(error)
            try: chunk = os.read(child.stdout.fileno(), 65536)
            except BlockingIOError: chunk = b''
            oversized |= len(output) + len(chunk) > 131072
            output.extend(chunk[:max(0, 131072 - len(output))])
            if time.monotonic() - start >= 10:
                timed_out = True
                signal.pidfd_send_signal(pin, signal.SIGKILL)
                break
            time.sleep(.005)
        child.wait(timeout=3)
        while True:
            try: chunk = os.read(child.stdout.fileno(), 65536)
            except BlockingIOError: break
            if not chunk: break
            oversized |= len(output) + len(chunk) > 131072
            output.extend(chunk[:max(0, 131072 - len(output))])
    finally:
        child.stdout.close()
        os.close(pin)
    (folder / f'{number:02d}.log').write_bytes(output)
    text = output.decode('utf8', 'replace')
    survivors = [value for value in known.values() if same(value)]
    result = {'iteration': number, 'exit_code': child.returncode, 'elapsed_seconds': time.monotonic() - start,
              'timeout': timed_out, 'output_oversized': oversized, 'driver_identity': root,
              'observed_owned_descendants': list(known.values()), 'unknown_surviving_identities': survivors,
              'terminal_unconfirmed': 'TerminalUnconfirmed' in text,
              'suite': suite, 'command': command,
              'inventory_verified': inventory_error is None and root is not None,
              'inventory_error': inventory_error,
              'expected_test_count': 9 if suite else 1,
              'native_tests_passed': child.returncode == 0 and ('9 passed; 0 failed' if suite else '1 passed; 0 failed') in text,
              'passed': child.returncode == 0 and ('9 passed; 0 failed' if suite else '1 passed; 0 failed') in text and not timed_out and not oversized and not survivors and inventory_error is None and root is not None}
    (folder / f'{number:02d}.json').write_text(json.dumps(result, indent=2) + '\n')
    return result

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('driver', type=Path)
    parser.add_argument('helper', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--suite', action='store_true', help='run the unchanged full nine-test parallel suite')
    parser.add_argument('--cleanup-stage-trace', action='store_true', help='pass the fixed debug trace flag only to this driver and descendants')
    args = parser.parse_args()
    out = args.output
    if not out.is_absolute() or '..' in out.parts or not out.is_relative_to(PROJECT / 'docs/validation/evidence') or out.exists() or any(p.is_symlink() for p in [out, *out.parents]):
        parser.error('new absolute project evidence path required')
    if any(not p.is_file() or p.is_symlink() for p in (args.driver, args.helper)):
        parser.error('explicit regular binaries required')
    out.mkdir(mode=0o700)
    source_paths = ['crates/engine/src/process.rs', 'crates/engine/src/process/guardian.rs', 'crates/engine/src/process/terminal.rs', 'crates/engine/tests/guardian.rs']
    before = {name: sha(PROJECT / name) for name in source_paths}
    record = {'driver_sha256': sha(args.driver), 'helper_sha256': sha(args.helper), 'source_before': before, 'test_name': None if args.suite else NAME, 'suite': args.suite, 'cleanup_stage_trace': args.cleanup_stage_trace, 'original_driver_path': str(args.driver.absolute()), 'original_helper_path': str(args.helper.absolute()), 'unchanged_timeout_ms': 300, 'iterations': []}
    with tempfile.TemporaryDirectory(prefix='dlpad-moved-leader-frozen-') as tmp:
        owned = Path(tmp)
        driver = owned / 'guardian-test'
        helper = owned / 'dlpad-process-guard'
        for original, copied in ((args.driver, driver), (args.helper, helper)):
            shutil.copyfile(original, copied); copied.chmod(0o500)
        if sha(driver) != record['driver_sha256'] or sha(helper) != record['helper_sha256']: raise RuntimeError('binary changed during copy')
        for number in range(1, 21):
            value = run(driver, out, number, args.suite, args.cleanup_stage_trace)
            value['driver_sha256'] = sha(driver); value['helper_sha256'] = sha(helper)
            (out / f'{number:02d}.json').write_text(json.dumps(value, indent=2) + '\n')
            record['iterations'].append(value)
            (out / 'progress.json').write_text(json.dumps(record, indent=2) + '\n')
        record['immutable_binaries_unchanged'] = sha(driver) == record['driver_sha256'] and sha(helper) == record['helper_sha256']
    record['owned_binary_temp_removed'] = not owned.exists()
    record['original_binaries_unchanged'] = sha(args.driver) == record['driver_sha256'] and sha(args.helper) == record['helper_sha256']
    record['source_after'] = {name: sha(PROJECT / name) for name in source_paths}
    record['source_unchanged'] = record['source_before'] == record['source_after']
    record['passed'] = all(value['passed'] for value in record['iterations']) and record['immutable_binaries_unchanged'] and record['source_unchanged'] and record['owned_binary_temp_removed'] and record['original_binaries_unchanged']
    (out / 'result.json').write_text(json.dumps(record, indent=2) + '\n')
    print(json.dumps({'passed': record['passed'], 'passes': sum(value['passed'] for value in record['iterations']), 'terminal_unconfirmed': sum(value['terminal_unconfirmed'] for value in record['iterations'])}))
    return 0 if record['passed'] else 2

if __name__ == '__main__': raise SystemExit(main())
