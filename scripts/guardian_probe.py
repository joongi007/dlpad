#!/usr/bin/env python3
"""Exercise the release app's headless guardian without any desktop or user data."""
import argparse
import ctypes
import json
import os
from pathlib import Path
import signal
import select
import socket
import subprocess
import sys
import tempfile
import time


def read_fixture(path):
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        try:
            return json.loads(path.read_text())
        except (FileNotFoundError, json.JSONDecodeError):
            time.sleep(0.02)
    raise RuntimeError('Synthetic guardian fixture startup timed out')


def owner(binary, folder):
    app, guard = socket.socketpair()
    env = dict(os.environ)
    env.pop('DISPLAY', None)
    env.pop('WAYLAND_DISPLAY', None)
    with (folder / 'guardian.log').open('wb') as log:
        child = subprocess.Popen([str(binary), '--dlpad-internal-guardian', str(guard.fileno()), '--', sys.executable, str(folder / 'root.py'), str(folder)],
                                 pass_fds=(guard.fileno(),), stdin=subprocess.DEVNULL,
                                 stdout=log, stderr=subprocess.STDOUT, env=env)
        guard.close()
        try:
            app.settimeout(10)
            if app.recv(1) != b'R':
                raise RuntimeError('Release guardian did not acknowledge readiness')
            (folder / 'owner.json').write_text(json.dumps({'guardian': child.pid}))
            while True:
                time.sleep(1)
        finally:
            app.close()
            child.wait(timeout=5)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('binary', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--owner-child', type=Path)
    parser.add_argument('--fault', choices=('owner-sigkill', 'guardian-sigterm', 'guardian-sigkill'), default='owner-sigkill')
    args = parser.parse_args()
    if args.owner_child is not None:
        return owner(args.binary.resolve(), args.owner_child)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    libc = ctypes.CDLL(None, use_errno=True)
    if libc.prctl(36, 1, 0, 0, 0) != 0:  # PR_SET_CHILD_SUBREAPER, only this fixture driver.
        raise OSError(ctypes.get_errno(), 'Cannot adopt fixture guardian')
    result = {'binary': str(args.binary), 'bytes': args.binary.stat().st_size,
              'platform': os.uname().release, 'gui_started': False,
              'release_headless_entry': False, 'fault': args.fault}
    identities = []
    child = None
    guardian_pid = None
    pids = []
    with tempfile.TemporaryDirectory(prefix='dlpad-guardian-') as temporary:
        folder = Path(temporary)
        (folder / 'root.py').write_text('''import json,os,pathlib,subprocess,sys,time
folder=pathlib.Path(sys.argv[1])
nested=subprocess.Popen([sys.executable,'-c','import time;time.sleep(30)'])
detached=subprocess.Popen([sys.executable,'-c','import time;time.sleep(30)'],start_new_session=True)
(folder/'tree.json').write_text(json.dumps([os.getpid(),nested.pid,detached.pid]))
time.sleep(30)
''')
        try:
            child = subprocess.Popen([sys.executable, str(Path(__file__).resolve()), str(args.binary.resolve()), str(args.output.resolve()), '--owner-child', str(folder)],
                                     stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            guardian_pid = read_fixture(folder / 'owner.json')['guardian']
            pids = [guardian_pid, *read_fixture(folder / 'tree.json')]
            for pid in pids:
                identities.append(os.pidfd_open(pid))
            result['release_headless_entry'] = True
            status = Path(f'/proc/{guardian_pid}/status').read_text()
            result['guardian_rss_kib'] = int(next(line.split()[1] for line in status.splitlines() if line.startswith('VmRSS:')))
            stat = Path(f'/proc/{guardian_pid}/stat').read_text().rsplit(')', 1)[1].split()
            before = int(stat[11]) + int(stat[12])
            start = time.monotonic()
            time.sleep(3)
            stat = Path(f'/proc/{guardian_pid}/stat').read_text().rsplit(')', 1)[1].split()
            elapsed = time.monotonic() - start
            result['active_guardian_cpu_percent'] = (int(stat[11]) + int(stat[12]) - before) / os.sysconf('SC_CLK_TCK') / elapsed * 100
            result['sample_seconds'] = elapsed
            if args.fault != 'owner-sigkill':
                fault_signal = signal.SIGTERM if args.fault == 'guardian-sigterm' else signal.SIGKILL
                signal.pidfd_send_signal(identities[0], fault_signal)
                observer = select.poll()
                observer.register(identities[0], select.POLLIN)
                if not observer.poll(5000):
                    raise RuntimeError('Guardian did not exit on fixture signal')
                if args.fault == 'guardian-sigkill':
                    leader = select.poll()
                    leader.register(identities[1], select.POLLIN)
                    result['direct_leader_parent_death_exit'] = bool(leader.poll(3000))
                    descendants = select.poll()
                    for descriptor in identities[2:]:
                        descendants.register(descriptor, select.POLLIN)
                    result['surviving_descendants'] = len(identities[2:]) - len(descendants.poll(250))
                    result['limitation_reproduced'] = result['direct_leader_parent_death_exit'] and result['surviving_descendants'] > 0
            child.kill()
            child.wait(timeout=5)
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                pid, _ = os.waitpid(guardian_pid, os.WNOHANG)
                if pid:
                    break
                time.sleep(0.02)
            else:
                raise RuntimeError('Guardian did not exit after owner SIGKILL')
            result['owner_fixture_sigkill'] = args.fault == 'owner-sigkill'
            result['descendants_reaped'] = all(not Path(f'/proc/{pid}').exists() for pid in pids)
            result['protection_passed'] = result['descendants_reaped']
            if not result['descendants_reaped'] and args.fault != 'guardian-sigkill':
                raise RuntimeError('Release guardian left fixture descendants')
        except Exception as error:
            result['error'] = str(error)
        finally:
            if child is not None and child.poll() is None:
                child.kill()
                child.wait(timeout=5)
            for descriptor in identities:
                try:
                    signal.pidfd_send_signal(descriptor, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                finally:
                    os.close(descriptor)
            # The driver is a subreaper; after the fixture owner/guardian exits,
            # reap only named, pidfd-pinned synthetic descendants we just killed.
            for pid in pids[1:]:
                try:
                    os.waitpid(pid, 0)
                except ChildProcessError:
                    pass
            if guardian_pid is not None:
                try:
                    os.waitpid(guardian_pid, os.WNOHANG)
                except ChildProcessError:
                    pass
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))
    return 1 if result.get('error') else (2 if result.get('protection_passed') is False else 0)


if __name__ == '__main__':
    raise SystemExit(main())
