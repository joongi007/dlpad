#!/usr/bin/env python3
"""Owned Linux headless guardian soak; never substitutes for media/DB/UI testing."""
import argparse
import ctypes
from datetime import datetime, timezone
import json
import hashlib
import os
from pathlib import Path
import signal
import socket
import subprocess
import sys
import tempfile
import time

MAX_SECONDS = 8 * 60 * 60
MAX_SAMPLES_BYTES = 512 * 1024
PARTIAL_BYTES = 64 * 1024
FIXTURE = r'''
import json, os, pathlib, subprocess, sys, time
folder=pathlib.Path(sys.argv[1]); role=sys.argv[2]
duration=float(sys.argv[3]); tick=float(sys.argv[4])
started=time.monotonic(); children=[]
if role=='root':
    middle=subprocess.Popen([sys.executable,__file__,str(folder),'middle',str(duration+60),str(tick)])
    detached=subprocess.Popen([sys.executable,__file__,str(folder),'detached',str(duration+60),str(tick)],start_new_session=True)
    children=[middle.pid,detached.pid]
elif role=='middle':
    leaf=subprocess.Popen([sys.executable,__file__,str(folder),'leaf',str(duration+60),str(tick)])
    children=[leaf.pid]
(folder/(role+'.json')).write_text(json.dumps({'pid':os.getpid(),'children':children}))
count=0; chunk=b'DLPAD_SYNTHETIC_PARTIAL\n'.ljust(65536,b'0')
while time.monotonic()-started < duration:
    count+=1
    value={'pid':os.getpid(),'count':count,'elapsed_seconds':time.monotonic()-started}
    pending=folder/(role+'.heartbeat.pending')
    pending.write_text(json.dumps(value)); pending.replace(folder/(role+'.heartbeat.json'))
    if role=='root':
        with (folder/'synthetic.part').open('wb') as stream: stream.write(chunk)
        if count==1 or count%12==0:
            print('DLPAD_SYNTHETIC_SOAK '+json.dumps(value),flush=True)
    time.sleep(tick)
'''


def utc_now():
    return datetime.now(timezone.utc).isoformat()


def atomic_json(path, value):
    pending = path.with_suffix('.pending')
    pending.write_text(json.dumps(value, indent=2) + '\n')
    pending.replace(path)


def read_fixture(path, deadline):
    while time.monotonic() < deadline:
        try:
            return json.loads(path.read_text())
        except (FileNotFoundError, json.JSONDecodeError):
            time.sleep(0.02)
    raise RuntimeError('Synthetic fixture startup timed out')


def process_sample(pid):
    fields = Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()
    status = Path(f'/proc/{pid}/status').read_text()
    rss = next((int(line.split()[1]) for line in status.splitlines() if line.startswith('VmRSS:')), 0)
    return {'pid': pid, 'start_ticks': int(fields[19]),
            'cpu_ticks': int(fields[11]) + int(fields[12]), 'rss_kib': rss}


def cleanup(guardian, control, identities, descendant_pids):
    control.close()  # Guardian treats owned socket EOF as mandatory cleanup.
    try:
        guardian.wait(timeout=5)
    except subprocess.TimeoutExpired:
        signal.pidfd_send_signal(identities[0], signal.SIGKILL)
        guardian.wait(timeout=5)
    for descriptor in identities[1:]:
        try:
            signal.pidfd_send_signal(descriptor, signal.SIGKILL)
        except ProcessLookupError:
            pass
    # Subreaper adopts only named fixture descendants; no arbitrary wait/kill.
    for pid in descendant_pids:
        try:
            os.waitpid(pid, 0)
        except ChildProcessError:
            pass
    for descriptor in identities:
        os.close(descriptor)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('binary', type=Path)
    parser.add_argument('output', type=Path, help='A new directory inside /tmp')
    parser.add_argument('--seconds', type=float, default=MAX_SECONDS)
    parser.add_argument('--interval', type=float, default=60)
    args = parser.parse_args()
    if sys.platform != 'linux':
        parser.error('Linux-only headless guardian soak')
    if not 1 <= args.seconds <= MAX_SECONDS:
        parser.error('Duration must be 1..28800 seconds')
    if not 0.25 <= args.interval <= 60 or (args.seconds > 60 and args.interval != 60):
        parser.error('Long soak uses exactly 60 second samples; short fixture interval is 0.25..60')
    output = args.output.resolve()
    if not output.is_relative_to(Path('/tmp')):
        parser.error('Output must be a fresh /tmp directory')
    binary = args.binary.resolve(strict=True)
    with binary.open('rb') as executable:
        binary_stat = os.fstat(executable.fileno())
        digest = hashlib.sha256()
        while chunk := executable.read(64 * 1024):
            digest.update(chunk)
        binary_sha256 = digest.hexdigest()
    output.mkdir(exist_ok=False)
    libc = ctypes.CDLL(None, use_errno=True)
    if libc.prctl(36, 1, 0, 0, 0) != 0:
        raise OSError(ctypes.get_errno(), 'Cannot adopt owned fixture descendants')
    result = {'scope': 'Linux release headless guardian synthetic process longevity',
              'media_download_db_ui_verified': False, 'state': 'starting',
              'started_utc': utc_now(), 'planned_seconds': args.seconds,
              'binary': str(binary), 'binary_bytes': binary_stat.st_size, 'binary_sha256': binary_sha256,
              'platform': os.uname().release, 'gui_started': False,
              'samples': 0, 'log_limit_bytes': 256 * 1024,
              'partial_limit_bytes': PARTIAL_BYTES}
    atomic_json(output / 'status.json', result)
    guardian = None
    control = None
    identities = []
    descendant_pids = []
    started = time.monotonic()
    try:
        with tempfile.TemporaryDirectory(prefix='owned-fixture-', dir=output) as temporary:
            folder = Path(temporary)
            script = folder / 'fixture.py'
            script.write_text(FIXTURE)
            control, guard_socket = socket.socketpair()
            env = dict(os.environ)
            env.pop('DISPLAY', None)
            env.pop('WAYLAND_DISPLAY', None)
            tick = 5 if args.seconds > 60 else 0.05
            with (output / 'guardian.log').open('xb') as log:
                guardian = subprocess.Popen([str(binary), '--dlpad-internal-guardian', str(guard_socket.fileno()), '--',
                                             sys.executable, str(script), str(folder), 'root', str(args.seconds), str(tick)],
                                            pass_fds=(guard_socket.fileno(),), stdin=subprocess.DEVNULL,
                                            stdout=log, stderr=subprocess.STDOUT, env=env)
                guard_socket.close()
                identities.append(os.pidfd_open(guardian.pid))
                running_binary = Path(f'/proc/{guardian.pid}/exe').stat()
                if (running_binary.st_dev, running_binary.st_ino) != (binary_stat.st_dev, binary_stat.st_ino):
                    raise RuntimeError('Release binary changed between hash and launch')
                control.settimeout(10)
                if control.recv(1) != b'R':
                    raise RuntimeError('Guardian readiness not acknowledged')
                deadline = time.monotonic() + 10
                root = read_fixture(folder / 'root.json', deadline)
                middle = read_fixture(folder / 'middle.json', deadline)
                detached = read_fixture(folder / 'detached.json', deadline)
                leaf = read_fixture(folder / 'leaf.json', deadline)
                descendant_pids = [root['pid'], middle['pid'], leaf['pid'], detached['pid']]
                for pid in descendant_pids:
                    identities.append(os.pidfd_open(pid))
                processes = {'owner': os.getpid(), 'guardian': guardian.pid,
                             'root': root['pid'], 'middle': middle['pid'],
                             'leaf': leaf['pid'], 'detached': detached['pid']}
                result.update(state='running', processes=processes, fixture_folder=str(folder), readiness=True)
                atomic_json(output / 'ownership.json', {'created_utc': utc_now(), 'processes': processes})
                previous = None
                previous_at = None
                previous_heartbeats = None
                next_sample = time.monotonic()
                with (output / 'samples.jsonl').open('xb') as samples:
                    while guardian.poll() is None:
                        now = time.monotonic()
                        if (output / 'stop.request').exists():
                            result['state'] = 'stopped'
                            break
                        if now - started > args.seconds + 15:
                            raise RuntimeError('Soak exceeded duration plus cleanup grace')
                        if now >= next_sample:
                            observed = {name: process_sample(pid) for name, pid in processes.items()}
                            if previous is not None:
                                for name, value in observed.items():
                                    if value['start_ticks'] != previous[name]['start_ticks']:
                                        raise RuntimeError('Owned process identity changed')
                                    value['cpu_percent'] = round((value['cpu_ticks'] - previous[name]['cpu_ticks']) / os.sysconf('SC_CLK_TCK') / (now - previous_at) * 100, 4)
                            heartbeats = {role: read_fixture(folder / (role + '.heartbeat.json'), time.monotonic() + 1)
                                          for role in ['root', 'middle', 'leaf', 'detached']}
                            for role, heartbeat in heartbeats.items():
                                heartbeat['elapsed_seconds'] = round(heartbeat['elapsed_seconds'], 3)
                                if previous_heartbeats is not None and heartbeat['count'] <= previous_heartbeats[role]['count']:
                                    raise RuntimeError('Synthetic heartbeat stopped advancing')
                            partial_bytes = (folder / 'synthetic.part').stat().st_size
                            if partial_bytes != PARTIAL_BYTES or (output / 'guardian.log').stat().st_size > 256 * 1024:
                                raise RuntimeError('Synthetic partial/log bound violated')
                            row = {'utc': utc_now(), 'elapsed_seconds': round(now - started, 3),
                                   'processes': observed, 'heartbeats': heartbeats,
                                   'partial_bytes': partial_bytes}
                            encoded = (json.dumps(row, separators=(',', ':')) + '\n').encode()
                            if samples.tell() + len(encoded) > MAX_SAMPLES_BYTES or result['samples'] >= 481:
                                raise RuntimeError('Soak sample storage bound exceeded')
                            samples.write(encoded)
                            samples.flush()
                            result.update(samples=result['samples'] + 1, elapsed_seconds=now - started,
                                          latest_utc=row['utc'], latest=row)
                            atomic_json(output / 'status.json', result)
                            print(json.dumps({'state': result['state'], 'elapsed_seconds': round(now - started, 3),
                                              'samples': result['samples'], 'output': str(output)}), flush=True)
                            previous, previous_at = observed, now
                            previous_heartbeats = heartbeats
                            next_sample = now + args.interval
                        time.sleep(min(0.2, args.interval))
                status = guardian.poll()
                if result['state'] != 'stopped':
                    if status != 0:
                        raise RuntimeError(f'Guardian unexpectedly exited: {status}')
                    result['state'] = 'completed'
                cleanup(guardian, control, identities, descendant_pids)
                identities = []
                result['descendants_reaped'] = all(not Path(f'/proc/{pid}').exists() for pid in [guardian.pid, *descendant_pids])
                if not result['descendants_reaped']:
                    raise RuntimeError('Owned fixture process remains after cleanup')
    except (Exception, KeyboardInterrupt) as error:
        result.update(state='failed', error=str(error) or 'Interrupted')
    finally:
        if guardian is not None and identities:
            cleanup(guardian, control, identities, descendant_pids)
        elif control is not None:
            control.close()
        if guardian is not None and descendant_pids:
            result['descendants_reaped'] = all(not Path(f'/proc/{pid}').exists() for pid in [guardian.pid, *descendant_pids])
            if not result['descendants_reaped']:
                result.update(state='failed', error='Owned fixture remains after cleanup')
        result.update(ended_utc=utc_now(), elapsed_seconds=time.monotonic() - started)
        result['eight_hours_elapsed'] = result['state'] == 'completed' and result['elapsed_seconds'] >= MAX_SECONDS
        atomic_json(output / 'status.json', result)
        atomic_json(output / 'result.json', result)
        print(json.dumps({key: result.get(key) for key in ['state', 'elapsed_seconds', 'samples', 'descendants_reaped', 'eight_hours_elapsed', 'error']}), flush=True)
    return 0 if result['state'] == 'completed' else 1


if __name__ == '__main__':
    raise SystemExit(main())
