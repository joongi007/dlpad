#!/usr/bin/env python3
"""Observe owned cgroup FD lifetime after normal short user-service exit."""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import select
import stat
import subprocess
import tempfile
import time
import uuid
import systemd_user_guardian_probe as base

PROJECT = Path(__file__).resolve().parent.parent

class Bus:
    def __init__(self):
        self.lib = ctypes.CDLL('libsystemd.so.0')
        p = ctypes.c_void_p
        signatures = {
            'sd_bus_new': ([ctypes.POINTER(p)], ctypes.c_int),
            'sd_bus_set_address': ([p, ctypes.c_char_p], ctypes.c_int),
            'sd_bus_set_bus_client': ([p, ctypes.c_int], ctypes.c_int),
            'sd_bus_negotiate_fds': ([p, ctypes.c_int], ctypes.c_int),
            'sd_bus_start': ([p], ctypes.c_int),
            'sd_bus_message_new_method_call': ([p, ctypes.POINTER(p), ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p], ctypes.c_int),
            'sd_bus_message_set_allow_interactive_authorization': ([p, ctypes.c_int], ctypes.c_int),
            'sd_bus_message_open_container': ([p, ctypes.c_char, ctypes.c_char_p], ctypes.c_int),
            'sd_bus_message_close_container': ([p], ctypes.c_int),
            'sd_bus_message_append_basic': ([p, ctypes.c_char, p], ctypes.c_int),
            'sd_bus_message_read_basic': ([p, ctypes.c_char, p], ctypes.c_int),
            'sd_bus_call': ([p, p, ctypes.c_uint64, ctypes.POINTER(base.BusError), ctypes.POINTER(p)], ctypes.c_int),
            'sd_bus_message_unref': ([p], p), 'sd_bus_unref': ([p], p),
            'sd_bus_error_free': ([ctypes.POINTER(base.BusError)], None)}
        for name, (args, result) in signatures.items():
            function = getattr(self.lib, name); function.argtypes = args; function.restype = result
        self.bus = p()
        self.check(self.lib.sd_bus_new(ctypes.byref(self.bus)))
        self.check(self.lib.sd_bus_set_address(self.bus, f'unix:path=/run/user/{os.getuid()}/bus'.encode()))
        self.check(self.lib.sd_bus_set_bus_client(self.bus, 1))
        self.check(self.lib.sd_bus_negotiate_fds(self.bus, 1))
        self.check(self.lib.sd_bus_start(self.bus))
    def check(self, code):
        if code < 0: raise RuntimeError(f'sd-bus fixed failure errno={-code}')
        return code
    def call(self, path, interface, method, build, return_object=False):
        p = ctypes.c_void_p
        message = p(); reply = p(); error = base.BusError()
        self.check(self.lib.sd_bus_message_new_method_call(self.bus, ctypes.byref(message), b'org.freedesktop.systemd1', path.encode(), interface.encode(), method.encode()))
        def basic(kind, value):
            item = ctypes.c_char_p(value.encode()) if kind == 's' else (ctypes.c_uint64(value) if kind == 't' else ctypes.c_int(value))
            ptr = ctypes.cast(item, p) if kind == 's' else ctypes.cast(ctypes.byref(item), p)
            self.check(self.lib.sd_bus_message_append_basic(message, kind.encode(), ptr))
        def opened(kind, signature): self.check(self.lib.sd_bus_message_open_container(message, kind.encode(), signature.encode()))
        def closed(): self.check(self.lib.sd_bus_message_close_container(message))
        try:
            self.check(self.lib.sd_bus_message_set_allow_interactive_authorization(message, 0))
            build(basic, opened, closed)
            self.check(self.lib.sd_bus_call(self.bus, message, 5_000_000, ctypes.byref(error), ctypes.byref(reply)))
            if return_object:
                value = ctypes.c_char_p()
                self.check(self.lib.sd_bus_message_read_basic(reply, b'o', ctypes.byref(value)))
                return value.value.decode()
        finally:
            self.lib.sd_bus_error_free(ctypes.byref(error))
            if reply: self.lib.sd_bus_message_unref(reply)
            self.lib.sd_bus_message_unref(message)
    def start(self, unit, null_fd, remain_after_exit=False, slice_unit=None):
        argv = ['/usr/bin/python3', '-c', 'import time;time.sleep(2)']
        def build(basic, opened, closed):
            basic('s', unit); basic('s', 'fail'); opened('a', '(sv)')
            def prop(name, kind, value):
                opened('r', 'sv'); basic('s', name); opened('v', kind); basic(kind, value); closed(); closed()
            for name, value in [('Type', 'exec'), ('KillMode', 'control-group'), ('Restart', 'no'), ('CollectMode', 'inactive-or-failed')]: prop(name, 's', value)
            if slice_unit is not None:
                prop('Slice', 's', slice_unit)
                prop('Delegate', 'b', 0)
            for name, value in [('AddRef', 1), ('SendSIGKILL', 1), ('RemainAfterExit', int(remain_after_exit))]: prop(name, 'b', value)
            for name, value in [('RuntimeMaxUSec', 3_000_000), ('TimeoutStartUSec', 3_000_000), ('TimeoutStopUSec', 2_000_000)]: prop(name, 't', value)
            for name in ['StandardInputFileDescriptor', 'StandardOutputFileDescriptor', 'StandardErrorFileDescriptor']: prop(name, 'h', null_fd)
            opened('r', 'sv'); basic('s', 'ExecStartEx'); opened('v', 'a(sasas)'); opened('a', '(sasas)'); opened('r', 'sasas')
            basic('s', argv[0]); opened('a', 's')
            for value in argv: basic('s', value)
            closed(); opened('a', 's'); basic('s', 'no-env-expand'); closed(); closed(); closed(); closed(); closed()
            closed(); opened('a', '(sa(sv))'); closed()
        return self.call('/org/freedesktop/systemd1', 'org.freedesktop.systemd1.Manager', 'StartTransientUnit', build, True)
    def get(self, unit): return self.call('/org/freedesktop/systemd1', 'org.freedesktop.systemd1.Manager', 'GetUnit', lambda basic, opened, closed: basic('s', unit), True)
    def unref(self, path): self.call(path, 'org.freedesktop.systemd1.Unit', 'Unref', lambda *args: None)
    def stop(self, unit):
        def build(basic, opened, closed): basic('s', unit); basic('s', 'replace')
        self.call('/org/freedesktop/systemd1', 'org.freedesktop.systemd1.Manager', 'StopUnit', build, True)
    def close(self):
        if self.bus: self.lib.sd_bus_unref(self.bus); self.bus = None

def pin(path):
    text = str(path)
    if not text.startswith('/sys/fs/cgroup/') or len(text) > 1024 or any(x in ('', '.', '..') for x in text[1:].split('/')): raise RuntimeError('cgroup path rejected')
    fd = os.open('/', os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC)
    try:
        for part in text[1:].split('/'):
            child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW, dir_fd=fd)
            os.close(fd); fd = child
        lib = ctypes.CDLL(None, use_errno=True)
        buf = ctypes.create_string_buffer(256)
        if lib.fstatfs(fd, buf) != 0 or ctypes.c_long.from_buffer(buf).value != 0x63677270: raise RuntimeError('foreign filesystem rejected')
        return fd
    except BaseException: os.close(fd); raise

def fd_identity(fd):
    info = os.fstat(fd)
    fdinfo = Path(f'/proc/self/fdinfo/{fd}').read_text()
    if len(fdinfo) > 4096: raise RuntimeError('fdinfo bound')
    mount = next(int(line.split(':', 1)[1]) for line in fdinfo.splitlines() if line.startswith('mnt_id:'))
    return {'device': info.st_dev, 'inode': info.st_ino, 'mount_id': mount}

def read_fd(fd, limit):
    try:
        os.lseek(fd, 0, os.SEEK_SET)
        data = bytearray()
        while len(data) <= limit:
            chunk = os.read(fd, min(4096, limit + 1 - len(data)))
            if not chunk: break
            data.extend(chunk)
        if len(data) > limit: return {'readable': False, 'oversized': True}
        return {'readable': True, 'bytes': len(data), 'text': data.decode('ascii', 'strict')}
    except OSError as error: return {'readable': False, 'errno': error.errno}

def observe_fds(directory, events, procs, path):
    value = {'directory_identity': fd_identity(directory), 'path_present': path.exists(), 'held_events': read_fd(events, 4096), 'held_procs': read_fd(procs, 65536)}
    reopened = {}
    for name, limit in [('cgroup.events', 4096), ('cgroup.procs', 65536)]:
        try:
            fd = os.open(name, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=directory)
            try: reopened[name] = read_fd(fd, limit)
            finally: os.close(fd)
        except OSError as error: reopened[name] = {'readable': False, 'open_errno': error.errno}
    value['new_openat'] = reopened
    def empty(events_value, procs_value):
        return events_value.get('readable') is True and 'populated 0\n' in events_value.get('text', '') and procs_value.get('readable') is True and procs_value.get('bytes') == 0
    value['held_empty'] = empty(value['held_events'], value['held_procs'])
    value['new_openat_empty'] = empty(reopened['cgroup.events'], reopened['cgroup.procs'])
    return value

def main():
    parser = argparse.ArgumentParser(description=__doc__); parser.add_argument('output', type=Path); parser.add_argument('--remain-after-exit', action='store_true'); args = parser.parse_args()
    out = args.output
    if not out.is_absolute() or '..' in out.parts or not out.is_relative_to(PROJECT / 'docs/validation/evidence') or out.exists() or any(p.is_symlink() for p in [out, *out.parents]): parser.error('new project evidence required')
    runtime = Path('/run/user') / str(os.getuid())
    for path in (runtime / 'bus', runtime / 'systemd/private'):
        info = path.lstat()
        if not stat.S_ISSOCK(info.st_mode) or info.st_uid != os.getuid(): parser.error('owned user-manager sockets required')
    env = dict(os.environ); env.update(XDG_RUNTIME_DIR=str(runtime), DBUS_SESSION_BUS_ADDRESS='unix:path=' + str(runtime / 'bus'))
    out.mkdir(mode=0o700)
    unit = 'dlpad-cgroup-lifetime-' + uuid.uuid4().hex + '.service'
    result = {'unit': unit, 'source_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), 'started_utc': base.utc_now(), 'fixed_worker_seconds': 2, 'runtime_max_seconds': 3, 'add_ref': True, 'remain_after_exit': args.remain_after_exit, 'other_units_targeted': False, 'signals_sent': False}
    baseline = len(list(Path('/proc/self/fd').iterdir()))
    bus = None; fds = []; invocation = None; pid = None; unitpath = None
    try:
        bus = Bus()
        with open('/dev/null', 'rb') as null:
            result['start_job'] = bus.start(unit, null.fileno(), args.remain_after_exit)
        unitpath = bus.get(unit)
        deadline = time.monotonic() + 1
        while True:
            props = base.props(unit, env)['properties']
            mainpid = int(props.get('MainPID', '0'))
            if props.get('ActiveState') == 'active' and mainpid > 1: break
            if time.monotonic() >= deadline: raise RuntimeError('active identity not captured')
            time.sleep(.02)
        invocation = props['InvocationID']
        if len(invocation) != 32 or any(c not in '0123456789abcdef' for c in invocation): raise RuntimeError('invocation rejected')
        group = props['ControlGroup']
        if not group.startswith(f'/user.slice/user-{os.getuid()}.slice/user@{os.getuid()}.service/') or not group.endswith('/' + unit): raise RuntimeError('group ownership rejected')
        pid = base.identity(mainpid)
        if pid is None or not base.same_live(pid): raise RuntimeError('main identity rejected')
        if base.cgroup_of(mainpid) != group: raise RuntimeError('membership mismatch')
        pidfd = os.pidfd_open(mainpid); fds.append(pidfd)
        path = Path('/sys/fs/cgroup' + group)
        directory = pin(path); fds.append(directory)
        events = os.open('cgroup.events', os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK, dir_fd=directory); fds.append(events)
        procs = os.open('cgroup.procs', os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK, dir_fd=directory); fds.append(procs)
        result.update(initial_properties=props, owned_pid=pid, cgroup=group, active_observation=observe_fds(directory, events, procs, path))
        again = base.props(unit, env)['properties']
        if again.get('InvocationID') != invocation or again.get('MainPID') != str(mainpid) or not base.same_live(pid): raise RuntimeError('active identity changed while pinning')
        deadline = time.monotonic() + 5
        while True:
            props = base.props(unit, env)['properties']
            if props.get('InvocationID') != invocation: raise RuntimeError('terminal invocation changed')
            if args.remain_after_exit:
                if props.get('ActiveState') == 'active' and props.get('SubState') == 'exited' and not base.same_identity(pid) and select.select([pidfd], [], [], 0)[0]: break
            elif props.get('ActiveState') in ('inactive', 'failed'): break
            if time.monotonic() >= deadline: raise RuntimeError('normal terminal not observed')
            time.sleep(.02)
        result['terminal_properties'] = props
        result['after_terminal'] = observe_fds(directory, events, procs, path)
        time.sleep(.1)
        result['after_terminal_100ms'] = observe_fds(directory, events, procs, path)
        result['owned_pid_gone'] = not base.same_identity(pid)
        result['pidfd_ready'] = bool(select.select([pidfd], [], [], 0)[0])
        # Same retained bus owns AddRef; release only after exact invocation recheck.
        if base.props(unit, env)['properties'].get('InvocationID') != invocation: raise RuntimeError('release invocation changed')
        if args.remain_after_exit:
            if not result['owned_pid_gone'] or not result['pidfd_ready']: raise RuntimeError('owned worker terminal not confirmed')
            bus.stop(unit); result['owned_stop_completed'] = True
            deadline = time.monotonic() + 3
            while True:
                stopped = base.props(unit, env)['properties']
                if stopped.get('InvocationID') != invocation: raise RuntimeError('stop invocation changed')
                if stopped.get('ActiveState') in ('inactive', 'failed'): break
                if time.monotonic() >= deadline: raise RuntimeError('owned stop terminal not observed')
                time.sleep(.02)
            result['stopped_properties'] = stopped
        if base.props(unit, env)['properties'].get('InvocationID') != invocation: raise RuntimeError('unref invocation changed')
        bus.unref(unitpath); result['unref_completed'] = True
        deadline = time.monotonic() + 3
        while True:
            final = base.props(unit, env)
            if final['properties'].get('LoadState') == 'not-found': break
            if time.monotonic() >= deadline: raise RuntimeError('owned unit GC not observed')
            time.sleep(.02)
        result['final_unit'] = final
        result['completed'] = True
    except Exception as error: result.update(completed=False, error_type=type(error).__name__, error=str(error))
    finally:
        for fd in reversed(fds): os.close(fd)
        if bus is not None: bus.close()
        result['fd_before'] = baseline; result['fd_after'] = len(list(Path('/proc/self/fd').iterdir()))
        result['ended_utc'] = base.utc_now()
        result['passed'] = result.get('completed') is True and result.get('owned_pid_gone') is True and result.get('pidfd_ready') is True and result['fd_before'] == result['fd_after']
        encoded = (json.dumps(result, indent=2) + '\n').encode()
        if len(encoded) > 262144: raise RuntimeError('result bound exceeded')
        with (out / 'result.json').open('xb') as stream: stream.write(encoded)
    print(json.dumps({'passed': result['passed'], 'after_terminal_held_empty': result.get('after_terminal', {}).get('held_empty'), 'after_terminal_new_openat_empty': result.get('after_terminal', {}).get('new_openat_empty')}))
    return 0 if result['passed'] else 2

if __name__ == '__main__': raise SystemExit(main())
