#!/usr/bin/env python3
"""Original-capability Windows scan CloseWait evidence; root native execution only."""
import argparse
import hashlib
import json
import ntpath
import os
from pathlib import Path
import stat
import subprocess
import windows_current_workspace_probe as n


def copy_new(source, destination, cap):
    if source.is_symlink() or not source.is_file():
        raise ValueError('regular immutable source required')
    before = source.stat()
    if not 0 <= before.st_size <= cap:
        raise ValueError('input byte bound')
    fd = os.open(source, os.O_RDONLY | os.O_NOFOLLOW)
    with os.fdopen(fd, 'rb') as stream, destination.open('xb') as output:
        actual = os.fstat(stream.fileno())
        if (actual.st_dev, actual.st_ino) != (before.st_dev, before.st_ino):
            raise ValueError('original input changed')
        total = 0
        while True:
            block = stream.read(min(65536, cap - total + 1))
            if not block:
                break
            total += len(block)
            if total > cap:
                raise ValueError('stream byte bound')
            output.write(block)
    if n.sha(source) != n.sha(destination):
        raise ValueError('immutable copy mismatch')


def regular_windows(value):
    if not isinstance(value, str) or len(value) > 32766:
        raise ValueError('native path bound')
    if value.startswith('\\\\?\\'):
        value = value[4:]
    drive, _ = ntpath.splitdrive(value)
    if len(drive) != 2 or drive[1] != ':' or not ntpath.isabs(value):
        raise ValueError('local drive absolute path required')
    if any(part in ('.', '..') for part in value.replace('/', '\\').split('\\')):
        raise ValueError('native path traversal')
    return ntpath.normpath(value)


def seed_result(log):
    if log.stat().st_size > 128 * 1024:
        raise ValueError('seed log bound')
    candidates = []
    for line in log.read_text(encoding='utf-8').splitlines():
        if line.startswith('{'):
            item = json.loads(line)
            if isinstance(item, dict) and item.get('phase') == 'scan-closewait-seeded':
                candidates.append(item)
    if len(candidates) != 1 or candidates[0].get('passed') is not True:
        raise ValueError('exact successful seed summary absent')
    return candidates[0]


def archive_owned(root, output):
    # WSL O_NOFOLLOW/ID checks are additional guards. Original Windows native
    # ACL/reparse proofs belong to the collector; do not claim this is an NT API.
    queue = [root]
    rows = []
    count = total = 0
    output.mkdir(mode=0o700)
    while queue:
        parent = queue.pop()
        for entry in parent.iterdir():
            count += 1
            if count > 64:
                raise ValueError('owned archive entry bound')
            info = entry.lstat()
            if stat.S_ISLNK(info.st_mode):
                raise ValueError('owned archive symlink rejected')
            relative = entry.relative_to(root)
            destination = output / relative
            if stat.S_ISDIR(info.st_mode):
                destination.mkdir(mode=0o700)
                if len(queue) >= 16:
                    raise ValueError('owned archive queue bound')
                queue.append(entry)
            elif stat.S_ISREG(info.st_mode):
                if info.st_size > 2 * 1024 * 1024:
                    raise ValueError('owned archive file bound')
                total += info.st_size
                if total > 8 * 1024 * 1024:
                    raise ValueError('owned archive total bound')
                # Empty SQLite sidecars retain the same no-follow source open.
                copy_new(entry, destination, 2 * 1024 * 1024)
                rows.append({'path': str(relative), 'bytes': info.st_size, 'sha256': n.sha(destination)})
            else:
                raise ValueError('owned archive special entry rejected')
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--case', required=True, choices=['release', 'timeout'])
    parser.add_argument('--expected-binary-sha', required=True)
    args = parser.parse_args()
    out = args.output
    if (not out.is_absolute() or '..' in out.parts or out.exists()
            or not out.is_relative_to(n.ROOT / 'docs/validation/evidence')
            or any(p.is_symlink() for p in [out, *out.parents])):
        parser.error('fresh owned project evidence required')
    if len(args.expected_binary_sha) != 64 or any(c not in '0123456789abcdef' for c in args.expected_binary_sha):
        parser.error('exact lowercase binary SHA required')
    source = n.ROOT / 'scripts/windows_library_scan_closewait_probe.cs'
    binary_source = n.TOOLS / 'target/release/dlpad-cpu.exe'
    source_sha, binary_sha, runner_sha = n.sha(source), n.sha(binary_source), n.sha(Path(__file__))
    if binary_sha != args.expected_binary_sha:
        parser.error('feature binary SHA differs from root READY')
    out.mkdir(mode=0o700)
    inputs = out / 'inputs'
    inputs.mkdir(mode=0o700)
    collector = inputs / 'collector.cs'
    binary = inputs / 'dlpad-cpu.exe'
    copy_new(source, collector, 1024 * 1024)
    copy_new(binary_source, binary, 256 * 1024 * 1024)
    _, env = n.configure()
    host = str(n.TOOLS / 'windows-cli-host.exe')
    framework = r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319'
    observer = out / 'observer.exe'
    operations = []
    result = {'passed': False, 'case': args.case, 'source_sha': source_sha, 'binary_sha': binary_sha,
              'runner_sha': runner_sha, 'fixture_deleted': False, 'native_acl_proof_by_python': False}
    try:
        argv = [host, '45', framework + '\\csc.exe', '/nologo',
                '/reference:System.Drawing.dll', '/reference:System.Web.Extensions.dll',
                *['/reference:' + framework + '\\WPF\\' + name + '.dll'
                  for name in ['UIAutomationClient', 'UIAutomationTypes', 'WindowsBase']],
                '/out:' + n.win(observer), n.win(collector)]
        operations.append(n.run('compile', argv, env, out, 45))
        if not operations[-1]['passed']:
            raise ValueError('native compile failed')
        observer_sha = n.sha(observer)
        log_cap = n.MAX_LOG
        try:
            n.MAX_LOG = 128 * 1024
            argv = [host, '1800', env['CARGO'], 'run', '-p', 'dlpad-engine', '--example',
                    'windows_library_scan_closewait_seed', '--locked']
            operations.append(n.run('seed', argv, env, out, 1800))
        finally:
            n.MAX_LOG = log_cap
        if not operations[-1]['passed'] or n.sha(binary_source) != binary_sha:
            raise ValueError('seed failed or feature binary changed')
        seed = seed_result(out / 'seed.log')
        root_native = regular_windows(seed['owned_root'])
        temp_native = regular_windows(env['TEMP'])
        if (ntpath.dirname(root_native).casefold() != temp_native.casefold()
                or not ntpath.basename(root_native).startswith('dlpad-library-scan-closewait-owned-')):
            raise ValueError('original seed namespace rejected')
        root = Path(subprocess.check_output(['wslpath', '-u', root_native], text=True, timeout=3).strip())
        if not root.is_absolute() or any(p.is_symlink() for p in [root, *root.parents]):
            raise ValueError('owned WSL root alias rejected')
        cap = root / 'fixture.cap'
        if cap.stat().st_size != seed['cap_bytes'] or cap.stat().st_size > 4096 or n.sha(cap) != seed['cap_sha256']:
            raise ValueError('original fixture CAP differs')
        (out / 'seed-summary.json').write_text(json.dumps(seed, indent=2) + '\n')
        argv = [host, '150', n.win(observer), n.win(binary), binary_sha, root_native,
                seed['cap_sha256'], n.win(out / 'gui'), args.case]
        operations.append(n.run('gui', argv, env, out, 150))
        native = json.loads((out / 'gui/result.json').read_text())
        required = ['actual_first_observation_started', 'pre_release_closewait_verified',
                    'closewait_inflight_verified', 'owned_job_naturally_empty',
                    'owned_job_empty_after_cleanup', 'owned_process_exited_after_job_close',
                    'private_desktop_closed', 'input_desktop_unchanged']
        if (native.get('status') != 'REVIEW_REQUIRED' or native.get('natural_exitcode') != 0
                or native.get('worker_stop_ack_verified') is not True
                or native.get('owned_job_active_before_cleanup') != 0
                or not all(native.get(k) is True for k in required)):
            raise ValueError('original native Job/process/desktop cleanup unconfirmed')
        rows = archive_owned(root, out / 'owned-files')
        (out / 'owned-files.json').write_text(json.dumps(rows, indent=2) + '\n')
        result['immutable_inputs_unchanged'] = (n.sha(source) == source_sha and n.sha(collector) == source_sha
            and n.sha(binary_source) == binary_sha and n.sha(binary) == binary_sha
            and n.sha(observer) == observer_sha and n.sha(Path(__file__)) == runner_sha)
        result['passed'] = result['immutable_inputs_unchanged'] and all(r['passed'] for r in operations)
        result['scope'] = 'original scan CloseWait; root must review rollback SQLite/IDs/files'
    except Exception as error:
        result['error'] = type(error).__name__ + ': ' + str(error)
    result['operations'] = operations
    (out / 'collector-result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'passed': result['passed'], 'evidence': str(out)}), flush=True)
    return 0 if result['passed'] else 2


if __name__ == '__main__':
    raise SystemExit(main())
