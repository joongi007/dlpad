#!/usr/bin/env python3
"""Public startup timeout/cancel proof with owned copied driver and adjacent fake helper."""
import argparse, hashlib, json, os, resource, select, shutil, signal, subprocess, sys, tempfile, time
from pathlib import Path
import systemd_user_guardian_probe as base

CASES = ('noR_timeout', 'noR_cancel', 'partial_cancel', 'metadata_noR_timeout')
LIMIT = 128 * 1024
HELPER = r'''#!/usr/bin/python3
import json,os,socket,sys,time
from pathlib import Path
root=Path(os.environ['DLPAD_STARTUP_INTERRUPT_ROOT']);case=os.environ['DLPAD_STARTUP_INTERRUPT_CASE']
def record(name,value):
 data=json.dumps(value).encode();assert len(data)<=4096
 pending=root/(name+'.owned.tmp');pending.write_bytes(data);pending.replace(root/name)
fields=Path('/proc/self/stat').read_text().rsplit(')',1)[1].split()
started=time.monotonic()
control=socket.socket(fileno=int(sys.argv[2]));os.set_inheritable(control.fileno(),False);control.settimeout(3)
record('launcher.json',{'pid':os.getpid(),'start_ticks':int(fields[19]),'parent_pid':os.getppid(),'started_monotonic':started})
os.write(1,b'owned bounded stdout marker\n');os.write(2,b'owned bounded stderr marker\n')
if case=='partial_cancel':control.sendall(b'E')
data=control.recv(16)
record('eof.json',{'eof':data==b'','observed_monotonic':time.monotonic(),'partial_E_sent':case=='partial_cancel'})
time.sleep(30)
'''

def sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as file:
        for block in iter(lambda: file.read(65536), b''):
            digest.update(block)
    return digest.hexdigest()

def read_json(path, limit=LIMIT):
    with path.open('rb') as file:
        data = file.read(limit + 1)
    if len(data) > limit:
        raise RuntimeError('owned evidence byte bound')
    return json.loads(data)

def limits():
    resource.setrlimit(resource.RLIMIT_FSIZE, (LIMIT, LIMIT))

def execute(name, folder, out, source):
    result = {'case': name, 'actions': [], 'scope': 'public direct development driver; no service/cgroup/tool/media/network'}
    driver = folder / 'copied-driver'; shutil.copyfile(source, driver); driver.chmod(0o500)
    helper = folder / 'dlpad-process-guard'; helper.write_text(HELPER); helper.chmod(0o500)
    (folder / 'unexpected-tool.py').write_text("import sys\nfrom pathlib import Path\nPath(sys.argv[1],'TOOL_STARTED').write_text('unexpected')\n")
    result.update(driver_sha256=sha(driver), helper_sha256=sha(helper))
    env = dict(os.environ)
    for key in ('DLPAD_TEST_SYSTEMD_BUS_PATH', 'DLPAD_TEST_SYSTEMD_STARTUP_FAULT', 'DLPAD_SERVICE_TEST_ROOT', 'DLPAD_SERVICE_TEST_CASE'):
        env.pop(key, None)
    env.update(DLPAD_GUARDIAN_BACKEND='direct', DLPAD_STARTUP_INTERRUPT_ROOT=str(folder), DLPAD_STARTUP_INTERRUPT_CASE=name)
    child = None; identities = {}; pidfds = {}
    try:
        if sha(source) != result['driver_sha256']:
            raise RuntimeError('copied immutable driver mismatch')
        started = time.monotonic()
        with (out / (name + '.driver.log')).open('xb') as log:
            child = subprocess.Popen([str(driver), '--ignored', '--exact', 'controlled_startup_driver', '--nocapture'], env=env,
                                     stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT, close_fds=True, preexec_fn=limits)
            identities['driver'] = base.identity(child.pid); pidfds['driver'] = os.pidfd_open(child.pid)
            deadline = time.monotonic() + 2
            while not (folder / 'launcher.json').exists():
                if child.poll() is not None or time.monotonic() > deadline:
                    raise RuntimeError('owned launcher observation unavailable')
                time.sleep(.005)
            identities['helper'] = read_json(folder / 'launcher.json', 4096)
            launcher = identities['helper']
            if launcher['parent_pid'] != child.pid or not base.same_live(launcher):
                raise RuntimeError('owned launcher parent/start mismatch')
            pidfds['helper'] = os.pidfd_open(launcher['pid'])
            result['driver_exit_code'] = child.wait(timeout=12)
        result['elapsed_seconds'] = time.monotonic() - started
        execution = read_json(folder / 'result.json'); eof = read_json(folder / 'eof.json', 4096)
        result.update(execution=execution, eof=eof, owned_identities=identities,
                      owned_identities_gone={role: not base.same_identity(value) for role, value in identities.items()},
                      pidfds_ready={role: bool(select.select([fd], [], [], 0)[0]) for role, fd in pidfds.items()})
        current = execution['execution']; expected_kind = 'TimedOut' if name.endswith('timeout') else 'Interrupted'
        result['typed_startup_unconfirmed'] = (current.get('ok') is False and current.get('cleanup_unconfirmed') is True
                                               and current.get('ownership_marker') == 'StartupUnconfirmed' and current.get('error_kind') == expected_kind)
        eof_delay = eof['observed_monotonic'] - launcher['started_monotonic']
        result['startup_eof_delay_seconds'] = eof_delay
        result['short_startup_interrupt_observed'] = eof['eof'] and 0.05 <= eof_delay < 1.0
        result['natural_grace_observed'] = time.monotonic() - eof['observed_monotonic'] >= 4.5
        result['finite_engine_return'] = 4500 <= execution['elapsed_millis'] < 8000
        result['fd_stable'] = execution['fd_before'] == execution['fd_after']
        result['tool_not_started'] = not (folder / 'TOOL_STARTED').exists()
        result['log_bounded'] = (out / (name + '.driver.log')).stat().st_size <= LIMIT
        result['copy_inputs_unchanged'] = sha(driver) == result['driver_sha256'] and sha(helper) == result['helper_sha256']
        if 'cancel' in name:
            cancellation = execution['cancellation']
            delta = cancellation['cancel_elapsed_millis'] - cancellation['launcher_seen_elapsed_millis']
            result['cancel_after_observed_launcher'] = cancellation['launcher_observed'] and 90 <= delta < 1000
        result['partial_frame_contract'] = eof['partial_E_sent'] == (name == 'partial_cancel')
        checks = ('typed_startup_unconfirmed', 'short_startup_interrupt_observed', 'natural_grace_observed', 'finite_engine_return',
                  'fd_stable', 'tool_not_started', 'log_bounded', 'copy_inputs_unchanged', 'partial_frame_contract')
        result['passed'] = child.returncode == 0 and all(result[key] for key in checks) and all(result['owned_identities_gone'].values()) and all(result['pidfds_ready'].values()) and result.get('cancel_after_observed_launcher', True)
    except Exception as error:
        result.update(passed=False, error=str(error))
    finally:
        for role, fd in pidfds.items():
            if base.same_live(identities[role]):
                signal.pidfd_send_signal(fd, signal.SIGKILL); result['actions'].append('terminate exact owned ' + role)
            os.close(fd)
        if child is not None:
            if child.poll() is None:
                child.kill()
            child.wait(timeout=5)
    base.write_json(out / (name + '.json'), result)
    return result

def main():
    parser = argparse.ArgumentParser(description=__doc__); parser.add_argument('test_binary', type=Path); parser.add_argument('output', type=Path)
    parser.add_argument('--case', choices=CASES, action='append'); args = parser.parse_args()
    source = args.test_binary; out = args.output; project = Path(__file__).resolve().parent.parent; evidence = project / 'docs/validation/evidence'
    if not source.is_absolute() or source.is_symlink() or not source.is_file():
        parser.error('explicit immutable native test binary required')
    if not out.is_absolute() or '..' in out.parts or not out.is_relative_to(evidence) or out == evidence or out.exists() or any(p.is_symlink() for p in [out, *out.parents]):
        parser.error('new project-owned evidence path required')
    out.mkdir(mode=0o700)
    result = {'scope': 'manager-independent startup public run/run_metadata; no cgroup claim', 'binary_sha256_before': sha(source),
              'probe_sha256': sha(Path(__file__)), 'base_probe_sha256': sha(Path(base.__file__)), 'cases': {}, 'existing_units_targeted': False}
    with tempfile.TemporaryDirectory(prefix='dlpad-startup-interrupt-owned-', dir='/tmp') as tmp:
        folder = Path(tmp)
        sentinel = subprocess.Popen([sys.executable, '-c', 'import time;time.sleep(300)'], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        sentinel_id = base.identity(sentinel.pid)
        try:
            for name in args.case or CASES:
                owned = folder / name; owned.mkdir(mode=0o700)
                result['cases'][name] = execute(name, owned, out, source)
            result['sentinel_preserved'] = base.same_live(sentinel_id); result['sentinel_identity'] = sentinel_id
        finally:
            sentinel.terminate(); sentinel.wait(timeout=5)
    result.update(owned_temp_removed=not folder.exists(), binary_sha256_after=sha(source))
    result['passed'] = all(value['passed'] for value in result['cases'].values()) and result['sentinel_preserved'] and result['owned_temp_removed'] and result['binary_sha256_before'] == result['binary_sha256_after']
    base.write_json(out / 'result.json', result)
    print(json.dumps({'passed': result['passed'], 'cases': {name: value['passed'] for name, value in result['cases'].items()}}))
    return 0 if result['passed'] else 2
if __name__ == '__main__':
    raise SystemExit(main())
