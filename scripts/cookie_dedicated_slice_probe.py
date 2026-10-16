#!/usr/bin/env python3
"""Owned random reader-container slice lifetime experiment; no cgroup writes or signals."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import select
import stat
import subprocess
import time
import uuid
import cookie_cgroup_lifetime_probe as lifetime
import systemd_user_guardian_probe as base

PROJECT = Path(__file__).resolve().parent.parent

def props(unit, env):
    keys = ['LoadState', 'ActiveState', 'SubState', 'InvocationID', 'MainPID', 'ExecMainPID', 'ExecMainCode', 'ExecMainStatus', 'ControlGroup', 'Slice', 'Delegate', 'StopWhenUnneeded', 'Result']
    result = subprocess.run(['/usr/bin/systemctl', '--user', 'show', unit, *['--property=' + k for k in keys]], env=env, stdin=subprocess.DEVNULL, capture_output=True, timeout=3)
    if len(result.stdout) > 65536 or len(result.stderr) > 65536: raise RuntimeError('property output bound exceeded')
    return {'exit_code': result.returncode, 'properties': dict(line.split('=', 1) for line in result.stdout.decode().splitlines() if '=' in line)}

def start_slice(bus, name):
    def build(basic, opened, closed):
        basic('s', name); basic('s', 'fail'); opened('a', '(sv)')
        for key, kind, value in [('AddRef', 'b', 1), ('StopWhenUnneeded', 'b', 0), ('CollectMode', 's', 'inactive-or-failed')]:
            opened('r', 'sv'); basic('s', key); opened('v', kind); basic(kind, value); closed(); closed()
        closed(); opened('a', '(sa(sv))'); closed()
    return bus.call('/org/freedesktop/systemd1', 'org.freedesktop.systemd1.Manager', 'StartTransientUnit', build, True)

def pin_group(group, fds):
    path = Path('/sys/fs/cgroup' + group)
    directory = lifetime.pin(path); fds.append(directory)
    files = []
    for name in ['cgroup.events', 'cgroup.procs']:
        fd = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK, dir_fd=directory); fds.append(fd); files.append(fd)
    return path, directory, *files

def snapshot(pinned): return lifetime.observe_fds(pinned[1], pinned[2], pinned[3], pinned[0])

def wait_props(unit, env, predicate, invocation=None):
    deadline = time.monotonic() + 5
    while True:
        observed = props(unit, env)['properties']
        if invocation is not None and observed.get('InvocationID') != invocation: raise RuntimeError('owned invocation changed')
        if predicate(observed): return observed
        if time.monotonic() >= deadline: raise RuntimeError('bounded property observation expired')
        time.sleep(.02)

def wait_gone(bus, unit):
    deadline = time.monotonic() + 5
    while True:
        try: bus.get(unit)
        except RuntimeError as error:
            if str(error) == 'sd-bus fixed failure errno=2':
                return {'method': 'GetUnit', 'errno': 2, 'object_absent': True, 'does_not_load_unit': True}
            raise
        if time.monotonic() >= deadline: raise RuntimeError('owned unit GetUnit GC not observed')
        time.sleep(.02)

def main():
    parser = argparse.ArgumentParser(description=__doc__); parser.add_argument('output', type=Path); args = parser.parse_args(); out = args.output
    if not out.is_absolute() or '..' in out.parts or not out.is_relative_to(PROJECT / 'docs/validation/evidence') or out.exists() or any(p.is_symlink() for p in [out, *out.parents]): parser.error('fresh project evidence required')
    runtime = Path('/run/user') / str(os.getuid())
    for p in [runtime / 'bus', runtime / 'systemd/private']:
        info = p.lstat()
        if not stat.S_ISSOCK(info.st_mode) or info.st_uid != os.getuid(): parser.error('owned manager required')
    env = dict(os.environ); env.update(XDG_RUNTIME_DIR=str(runtime), DBUS_SESSION_BUS_ADDRESS='unix:path=' + str(runtime / 'bus'))
    out.mkdir(mode=0o700); token = uuid.uuid4().hex
    slice_unit = 'dlpadcookie' + token + '.slice'; service = 'dlpad-cookie-container-' + token + '.service'
    r = {'slice': slice_unit, 'service': service, 'started_utc': base.utc_now(), 'worker_seconds': 2, 'runtime_max_seconds': 3, 'signals_sent': False, 'cgroup_writes': False, 'existing_units_targeted': False,
         'source_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), 'reused_source_sha256': hashlib.sha256(Path(lifetime.__file__).read_bytes()).hexdigest()}
    baseline = len(list(Path('/proc/self/fd').iterdir())); bus = None; fds = []; slice_inv = None; service_inv = None
    try:
        bus = lifetime.Bus(); r['slice_start_job'] = start_slice(bus, slice_unit); slice_path = bus.get(slice_unit)
        with open('/dev/null', 'rb') as null: r['service_start_job'] = bus.start(service, null.fileno(), slice_unit=slice_unit)
        service_path = bus.get(service)
        active = wait_props(service, env, lambda p: p.get('ActiveState') == 'active' and int(p.get('MainPID', '0')) > 1)
        parent = wait_props(slice_unit, env, lambda p: p.get('ActiveState') == 'active' and p.get('ControlGroup'))
        slice_inv = parent['InvocationID']; service_inv = active['InvocationID']
        for value in [slice_inv, service_inv]:
            if len(value) != 32 or any(c not in '0123456789abcdef' for c in value): raise RuntimeError('invocation framing rejected')
        parent_group = parent['ControlGroup']; child_group = active['ControlGroup']
        if not parent_group.startswith(f'/user.slice/user-{os.getuid()}.slice/user@{os.getuid()}.service/') or not parent_group.endswith('/' + slice_unit): raise RuntimeError('parent ownership rejected')
        if active.get('Slice') != slice_unit or active.get('Delegate') != 'no' or child_group != parent_group + '/' + service: raise RuntimeError('service dedicated membership rejected')
        pid = base.identity(int(active['MainPID']))
        if pid is None or not base.same_live(pid) or base.cgroup_of(pid['pid']) != child_group: raise RuntimeError('main native membership rejected')
        pidfd = os.pidfd_open(pid['pid']); fds.append(pidfd)
        parent_pin = pin_group(parent_group, fds); child_pin = pin_group(child_group, fds)
        if lifetime.fd_identity(parent_pin[1])['mount_id'] != lifetime.fd_identity(child_pin[1])['mount_id']: raise RuntimeError('container mount mismatch')
        r.update(owned_pid=pid, parent_active_properties=parent, service_active_properties=active, parent_active=snapshot(parent_pin), service_active=snapshot(child_pin))
        # Recheck the same native membership after both pins are established.
        if props(service, env)['properties'].get('InvocationID') != service_inv or props(slice_unit, env)['properties'].get('InvocationID') != slice_inv or not base.same_live(pid) or base.cgroup_of(pid['pid']) != child_group: raise RuntimeError('identity changed during pin')
        terminal = wait_props(service, env, lambda p: p.get('ActiveState') in ('inactive', 'failed'), service_inv)
        if terminal.get('ExecMainCode') != '1' or terminal.get('ExecMainStatus') != '0' or terminal.get('Result') != 'success': raise RuntimeError('worker did not exit normally')
        parent_terminal = props(slice_unit, env)['properties']
        if parent_terminal.get('InvocationID') != slice_inv or parent_terminal.get('ActiveState') != 'active' or parent_terminal.get('ControlGroup') != parent_group: raise RuntimeError('dedicated parent lifetime changed')
        r.update(service_terminal_properties=terminal, parent_terminal_properties=parent_terminal, parent_after_terminal=snapshot(parent_pin), service_after_terminal=snapshot(child_pin), owned_pid_gone=not base.same_identity(pid), pidfd_ready=bool(select.select([pidfd], [], [], 0)[0]))
        time.sleep(.1); r['parent_after_terminal_100ms'] = snapshot(parent_pin)
        if props(service, env)['properties'].get('InvocationID') != service_inv: raise RuntimeError('service release identity changed')
        bus.unref(service_path); r['service_unref'] = True
        r['service_final'] = wait_gone(bus, service)
        parent_before_stop = props(slice_unit, env)['properties']
        if parent_before_stop.get('InvocationID') != slice_inv or parent_before_stop.get('ControlGroup') != parent_group or not r['owned_pid_gone'] or not r['pidfd_ready']: raise RuntimeError('slice stop identity not confirmed')
        r['parent_before_stop'] = snapshot(parent_pin)
        bus.stop(slice_unit); r['slice_stop'] = True
        r['slice_stopped'] = wait_props(slice_unit, env, lambda p: p.get('ActiveState') in ('inactive', 'failed'), slice_inv)
        if props(slice_unit, env)['properties'].get('InvocationID') != slice_inv: raise RuntimeError('slice release identity changed')
        bus.unref(slice_path); r['slice_unref'] = True
        r['slice_final'] = wait_gone(bus, slice_unit)
        r['completed'] = True
    except Exception as error: r.update(completed=False, error_type=type(error).__name__, error=str(error))
    finally:
        for fd in reversed(fds): os.close(fd)
        if bus is not None: bus.close()
        r['fd_before'] = baseline; r['fd_after'] = len(list(Path('/proc/self/fd').iterdir())); r['ended_utc'] = base.utc_now()
        r['passed'] = r.get('completed') is True and r.get('owned_pid_gone') is True and r.get('pidfd_ready') is True and r['fd_before'] == r['fd_after']
        data = (json.dumps(r, indent=2) + '\n').encode()
        if len(data) > 262144: raise RuntimeError('evidence capacity exceeded')
        with (out / 'result.json').open('xb') as stream: stream.write(data)
    print(json.dumps({'passed': r['passed'], 'parent_held_empty': r.get('parent_after_terminal', {}).get('held_empty'), 'parent_new_openat_empty': r.get('parent_after_terminal', {}).get('new_openat_empty')}))
    return 0 if r['passed'] else 2

if __name__ == '__main__': raise SystemExit(main())
