#!/usr/bin/env python3
"""Synthetic session cookies through private native AT-SPI; immutable app only."""
import argparse
import ast
import json
import hashlib
import os
from pathlib import Path
import re
import select
import signal
import subprocess
import tempfile
import threading
import time
import traceback

from accessible_download_probe import bounded_log, file_hash, pin_worker, state, wait, snapshot_database
from background_ui_probe import OwnedDisplay, reap
from cpu_library_probe import wheel
from cpu_options_probe import lookup
from egui_ime_probe import accessibility, call
from ime_probe import focus_owned_window
from native_probe import probe_display, screenshot
from subtitle_ui_probe import key_input, job
from x11_diagnostics import click_at
import systemd_user_guardian_probe as service

COOKIE_SOURCE = '이번 작업의 Netscape 쿠키 파일 (선택)'
IMPORT_ERROR = '쿠키 파일을 안전하게 가져오지 못했습니다. 파일 형식과 앱 전용 폴더 권한을 확인하세요.'
RECOVERY_ERROR = '이전 쿠키 복사본을 안전하게 확인하지 못했습니다. 앱 전용 폴더의 권한을 확인하고 다시 시도하세요.'
RECOVERY_PENDING = '이전 쿠키 복사본의 종료·소유권 확인이 남아 안전하게 보존했습니다. 새 작업은 새 쿠키 선택으로 실행합니다.'
SECRET = 'synthetic-native-session-only'
COOKIES = '# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tSESSION\t' + SECRET + '\n'
TOOL = r'''#!/usr/bin/python3
import json,os,pathlib,sys,time
root=pathlib.Path(__file__).parent
args=sys.argv[1:]
stage='version' if '--version' in args else 'metadata' if '--dump-single-json' in args else 'download'
copy=args[args.index('--cookies')+1] if '--cookies' in args else None
literal=pathlib.Path(copy).read_text() if copy else None
with (root/'calls.jsonl').open('a') as f:f.write(json.dumps({'stage':stage,'copy':copy,'literal':literal})+'\n')
if stage=='version':assert copy is None;print('2026.10.04');sys.exit(0)
assert copy and literal==(root/'source.cookies').read_text()
print('synthetic-native-session-only '+copy,file=sys.stderr,flush=True)
if stage=='metadata':print(json.dumps({'id':'synthetic','title':'synthetic-native-session-only','duration':120,'formats':[{'url':copy}]}));sys.exit(0)
cfg=json.loads((root/'control.json').read_text())
st=pathlib.Path('/proc/self/stat').read_text().rsplit(')',1)[1].split()
(root/'worker.json').write_text(json.dumps({'pid':os.getpid(),'start_ticks':int(st[19]),'token':'owned-cookie-gui'}))
for i in range(int(cfg['seconds']*5)):
 print('synthetic-native-session-only '+copy,flush=True)
 print('DLPAD_PROGRESS:'+json.dumps({'speed':1024,'eta':cfg['seconds']-i/5}),flush=True);time.sleep(.2)
p=pathlib.Path(args[args.index('--paths')+1])/args[args.index('--output')+1].replace('%(ext)s','mp4')
p.write_bytes(b'synthetic authenticated native artifact')
print('DLPAD_ARTIFACT:'+json.dumps(str(p)),flush=True)
'''


def no_leak(data, needles):
    return all(needle.encode() not in data for needle in needles)


def run_case(args, name, host_env):
    out = args.output / name
    out.mkdir(mode=0o700)
    result = {'case': name, 'passed': False, 'user_display_used': False, 'fields': []}
    app = bus = owner = context = None
    threads = []
    worker_fd = guardian_fd = None
    worker_info = guardian = None
    private_copy = None
    deferred_files = {}
    observation = None
    with tempfile.TemporaryDirectory(prefix='dlpad-cookie-gui-owned-') as tmp:
        root = Path(tmp)
        for child in ('data', 'media', 'config', 'cache', 'runtime', 'xdg-data'):
            (root / child).mkdir(mode=0o700)
        source = root / 'source.cookies'
        source.write_text(COOKIES if name != 'malformed' else 'invalid-cookie-' + SECRET)
        source.chmod(0o600)
        source_before = (source.stat().st_dev, source.stat().st_ino, file_hash(source))
        result['source_identity_before'] = {'device': source_before[0], 'inode': source_before[1], 'bytes': source.stat().st_size, 'sha256': source_before[2]}
        cache = root / 'data/session-cookies'
        if name == 'unsafe-cache':
            foreign_cache = root/'foreign-cache'
            foreign_cache.mkdir(mode=0o700)
            cache.symlink_to(foreign_cache, target_is_directory=True)
        else:
            cache.mkdir(mode=0o700)
        sentinel = cache / 'unrelated-sentinel'
        sentinel.write_bytes(b'owned unrelated sentinel')
        sentinel_before = (sentinel.stat().st_dev, sentinel.stat().st_ino, file_hash(sentinel))
        result['sentinel_identity_before'] = {'device': sentinel_before[0], 'inode': sentinel_before[1], 'bytes': sentinel.stat().st_size, 'sha256': sentinel_before[2]}
        tool = root / 'synthetic-tool'
        tool.write_text(TOOL)
        tool.chmod(0o700)
        result['synthetic_tool_sha256'] = file_hash(tool)
        result['synthetic_cookie_fixture_sha256'] = hashlib.sha256(COOKIES.encode()).hexdigest()
        if name == 'same-boot':
            if not args.fixture_binary or file_hash(args.fixture_binary) != args.fixture_sha:
                raise RuntimeError('explicit frozen fixture driver required')
            fixture_env = dict(os.environ)
            fixture_env['DLPAD_OWNED_COOKIE_FIXTURE_ROOT'] = str(root)
            child = subprocess.run([str(args.fixture_binary), '--ignored', '--exact', 'create_deferred_session_cookie_fixture', '--nocapture'], env=fixture_env, capture_output=True, timeout=15)
            if child.returncode or len(child.stdout) + len(child.stderr) > 16384:
                raise RuntimeError('synthetic deferred fixture generation failed')
            result['fixture_driver_sha256'] = file_hash(args.fixture_binary)
            result['fixture_driver_exit'] = child.returncode
            directories = [item for item in cache.iterdir() if item.is_dir() and not item.is_symlink()]
            if len(directories) != 1: raise RuntimeError('expected one owned deferred session')
            for item in directories[0].iterdir():
                if item.is_symlink() or not item.is_file() or item.stat().st_size > 8192: raise RuntimeError('deferred record fixture bound exceeded')
                deferred_files[item] = {'bytes': item.stat().st_size, 'sha256': file_hash(item), 'device': item.stat().st_dev, 'inode': item.stat().st_ino}
                if item.name in ('recovery.json', 'owner.marker'):
                    with (out/('deferred-before-'+item.name)).open('xb') as record_evidence: record_evidence.write(item.read_bytes())
            result['deferred_fixture_before'] = {str(item): metadata for item, metadata in deferred_files.items()}
        (root / 'control.json').write_text(json.dumps({'seconds': 120 if name == 'cancel' else 4}))
        env = dict(os.environ)
        for key in ('WAYLAND_DISPLAY', 'DBUS_SESSION_BUS_ADDRESS', 'AT_SPI_BUS_ADDRESS', 'XMODIFIERS', 'NO_AT_BRIDGE', 'DLPAD_GUARDIAN_BACKEND', 'DLPAD_TEST_SYSTEMD_BUS_PATH', 'DLPAD_TEST_SYSTEMD_STARTUP_FAULT'):
            env.pop(key, None)
        env.update(DLPAD_TEST_DATA=str(root/'data'), DLPAD_LIBRARY_ROOT=str(root/'media'), DLPAD_YTDLP=str(tool), DLPAD_TEST_FONT=str(args.font), XDG_CONFIG_HOME=str(root/'config'), XDG_DATA_HOME=str(root/'xdg-data'), XDG_CACHE_HOME=str(root/'cache'), XDG_RUNTIME_DIR=str(root/'runtime'), GSETTINGS_BACKEND='memory', WINIT_X11_SCALE_FACTOR='1')
        try:
            context = probe_display(out/'isolated', False)
            display, kind = context.__enter__()
            if kind != 'Xvfb isolated':
                raise RuntimeError('private Xvfb required')
            env['DISPLAY'] = display
            bus = subprocess.Popen(['dbus-daemon', '--session', '--nofork', '--print-address=1', '--address=unix:path='+str(root/'runtime/bus')], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
            thread = threading.Thread(target=bounded_log, args=(bus.stderr, out/'bus.log'), daemon=True)
            thread.start(); threads.append(thread)
            if not select.select([bus.stdout], [], [], 5)[0]:
                raise RuntimeError('private bus not ready')
            env['DBUS_SESSION_BUS_ADDRESS'] = bus.stdout.readline(4096).decode().strip()
            address = re.search(r"'([^']+)'", accessibility(env)['bus_address_reply']['stdout']).group(1)
            env['AT_SPI_BUS_ADDRESS'] = address
            app = subprocess.Popen([str(args.binary), '--fixture'], env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, start_new_session=True)
            result['app_identity'] = service.identity(app.pid)
            thread = threading.Thread(target=bounded_log, args=(app.stdout, out/'app.log'), daemon=True)
            thread.start(); threads.append(thread)
            time.sleep(2)
            owner = OwnedDisplay(display)
            geometry = screenshot(out/'initial.png', 'dlpad', display)
            focus_owned_window(display)

            def widget(label, role=None):
                for turn in range(20):
                    try:
                        point, node = lookup(env, address, label, role)
                        result['fields'].append({'label': label, 'bounds': node['bounds'], 'role': node['role']})
                        return point, node
                    except RuntimeError:
                        wheel(owner, 3 if turn < 10 else -3)
                raise RuntimeError('bounded AT-SPI lookup unavailable: '+label)

            def click(label):
                point, node = widget(label, 'action')
                if label == '작업 취소':
                    reply = call(env, node['destination'], node['path'], 'org.a11y.atspi.Action.DoAction', '0', address=address)
                    result['cancel_native_action'] = reply
                    if reply['exit'] or 'true' not in reply['stdout']: raise RuntimeError('native cancel action refused')
                else:
                    click_at(display, geometry, *point)
                time.sleep(.2)

            def text(label):
                pairs = lambda value: re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)", value)
                queue = pairs(call(env, 'org.a11y.atspi.Registry', '/org/a11y/atspi/accessible/root', 'org.a11y.atspi.Accessible.GetChildren', address=address)['stdout'])
                seen = set()
                while queue and len(seen) < 256:
                    destination, path = queue.pop(0)
                    if (destination, path) in seen: continue
                    seen.add((destination, path))
                    properties = call(env, destination, path, 'org.freedesktop.DBus.Properties.GetAll', 'org.a11y.atspi.Accessible', address=address)
                    if "Name': <'"+label+"'>" in properties['stdout']:
                        role = call(env, destination, path, 'org.a11y.atspi.Accessible.GetRole', address=address)
                        if not re.search(r'uint32 (79|61),', role['stdout']):
                            queue.extend(pairs(call(env, destination, path, 'org.a11y.atspi.Accessible.GetChildren', address=address)['stdout']))
                            continue
                        reply = call(env, destination, path, 'org.a11y.atspi.Text.GetText', '0', '-1', address=address)
                        if reply['exit'] == 0: return ast.literal_eval(reply['stdout'])[0]
                    queue.extend(pairs(call(env, destination, path, 'org.a11y.atspi.Accessible.GetChildren', address=address)['stdout']))
                raise RuntimeError('AT-SPI text unavailable')

            def label_present(label):
                pairs = lambda value: re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)", value)
                queue = pairs(call(env, 'org.a11y.atspi.Registry', '/org/a11y/atspi/accessible/root', 'org.a11y.atspi.Accessible.GetChildren', address=address)['stdout'])
                seen = set()
                while queue and len(seen) < 256:
                    destination, path = queue.pop(0)
                    if (destination, path) in seen: continue
                    seen.add((destination, path))
                    reply = call(env, destination, path, 'org.freedesktop.DBus.Properties.GetAll', 'org.a11y.atspi.Accessible', address=address)
                    if "Name': <'"+label+"'>" in reply['stdout']: return True
                    queue.extend(pairs(call(env, destination, path, 'org.a11y.atspi.Accessible.GetChildren', address=address)['stdout']))
                return False

            selections = [('영상 URL', 'https://fixture.invalid/watch')]
            if name != 'unsafe-cache': selections.append((COOKIE_SOURCE, str(source)))
            for label, value in selections:
                point, _ = widget(label, '79')
                key_input(display, geometry, point, value)
            if name == 'same-boot':
                wait(lambda: label_present(RECOVERY_PENDING), app)
                _, node = widget('다운로드 시작', 'action')
                reply = call(env, node['destination'], node['path'], 'org.a11y.atspi.Accessible.GetState', address=address)
                bits = int(re.search(r'(?:uint32 )?(\d+)', reply['stdout']).group(1))
                result['same_boot_pending_label_start_enabled'] = bool(bits & (1 << 8) and bits & (1 << 24))
                assert result['same_boot_pending_label_start_enabled']
            if name == 'unsafe-cache':
                wait(lambda: label_present(RECOVERY_ERROR), app)
                _, node = widget('다운로드 시작', 'action')
                reply = call(env, node['destination'], node['path'], 'org.a11y.atspi.Accessible.GetState', address=address)
                bits = int(re.search(r'(?:uint32 )?(\d+)', reply['stdout']).group(1))
                result['startup_fixed_label'] = True
                result['startup_start_disabled'] = not bits & (1 << 8) and not bits & (1 << 24)
                assert result['startup_start_disabled']
                result['disabled_start_action'] = call(env, node['destination'], node['path'], 'org.a11y.atspi.Action.DoAction', '0', address=address)
                time.sleep(.5)
                assert job(root/'data') is None and not (root/'calls.jsonl').exists()
                result['no_tool_or_job'] = True
                result['owned_foreign_cache_unchanged'] = cache.is_symlink() and cache.readlink() == foreign_cache and sorted(foreign_cache.iterdir()) == [foreign_cache/'unrelated-sentinel']
                assert result['owned_foreign_cache_unchanged']
            else:
                click('다운로드 시작')
            if name == 'malformed':
                time.sleep(1)
                assert job(root/'data') is None and not (root/'calls.jsonl').exists()
                result['no_tool_or_job'] = True
                click('실패 원인 보기')
                preview = text('진단 자료 미리보기')
                assert IMPORT_ERROR in preview and no_leak(preview.encode(), [SECRET, str(source)])
                result['fixed_error_preview_no_leak'] = True
            elif name != 'unsafe-cache':
                wait(lambda: (root/'worker.json').exists(), app)
                info, worker_fd = pin_worker(root, 'owned-cookie-gui')
                worker_info = info
                parent = int(Path(f"/proc/{info['pid']}/stat").read_text().rsplit(')', 1)[1].split()[1])
                guardian = service.identity(parent)
                cgroup = service.cgroup_of(parent)
                unit = cgroup.rsplit('/', 1)[-1]
                assert unit.startswith('dlpad-job-') and unit.endswith('.service')
                properties = service.props(unit, host_env)['properties']
                invocation = properties['InvocationID']
                service.verify_unit(unit, invocation, guardian, host_env)
                assert service.cgroup_of(info['pid']) == cgroup
                actual = Path(f'/proc/{parent}/exe').stat()
                expected = args.binary.stat()
                assert (actual.st_dev, actual.st_ino) == (expected.st_dev, expected.st_ino)
                assert file_hash(Path(f'/proc/{parent}/exe')) == args.expected_sha
                guardian_fd = os.pidfd_open(parent)
                assert service.same_live(guardian)
                observation = {'worker': info, 'guardian': guardian, 'unit': unit, 'invocation_id': invocation, 'control_group': cgroup, 'same_app_guardian': True}
                result['protection'] = observation
                calls = [json.loads(line) for line in (root/'calls.jsonl').read_text().splitlines()]
                assert [entry['stage'] for entry in calls] == ['version', 'metadata', 'download']
                assert calls[0]['copy'] is None and calls[0]['literal'] is None
                copy = calls[1]['copy']
                private_copy = copy
                if name == 'same-boot': assert Path(copy) not in deferred_files
                assert copy == calls[2]['copy'] and calls[1]['literal'] == calls[2]['literal'] == COOKIES
                assert Path(copy).is_relative_to(cache) and Path(copy).stat().st_mode & 0o777 == 0o600
                result['version_without_cookies_metadata_download_same_literal_copy'] = True
                if name == 'cancel':
                    click('작업 취소')
                final = wait(lambda: state(root/'data', 'Cancelled' if name == 'cancel' else 'Downloaded'), app, timeout=20)
                if len(json.dumps(final).encode()) > 262144: raise RuntimeError('final job evidence bound exceeded')
                service.write_json(out/'final-job.json', final)
                if name in ('success', 'same-boot'):
                    artifact = Path(final['job']['artifacts'][0]['path'])
                    assert artifact.is_relative_to(root/'media') and artifact.read_bytes() == b'synthetic authenticated native artifact'
                    result['artifact_verified'] = True
                    if artifact.stat().st_size > 1048576: raise RuntimeError('synthetic artifact bound exceeded')
                    result['artifact_evidence'] = {'path': str(artifact), 'bytes': artifact.stat().st_size, 'sha256': file_hash(artifact)}
                    with (out/'synthetic-artifact.bin').open('xb') as evidence_artifact:
                        with artifact.open('rb') as original_artifact:
                            for chunk in iter(lambda: original_artifact.read(65536), b''): evidence_artifact.write(chunk)
                wait(lambda: not Path(copy).exists(), app)
                result['copy_removed'] = True
                needles = [SECRET, str(source), copy]
                result['journal_no_secret_or_cookie_paths'] = all(no_leak(path.read_bytes(), needles) for path in (root/'data').glob('jobs.sqlite*'))
                assert result['journal_no_secret_or_cookie_paths']
                wait(lambda: not service.same_identity(guardian) and not service.same_identity({'pid': info['pid'], 'start_ticks': info['start_ticks']}), timeout=8)
                wait(lambda: service.props(unit, host_env)['properties'].get('LoadState') == 'not-found', timeout=5)
                assert not (Path('/sys/fs/cgroup')/cgroup.lstrip('/')).exists()
                result['owned_service_gc'] = True
            result['source_field_cleared'] = text(COOKIE_SOURCE) == ''
            assert result['source_field_cleared']
            screenshot(out/'final.png', 'dlpad', display)
            owner.delete_window(owner.window())
            wait(lambda: app.poll() is not None, timeout=10)
            assert app.returncode == 0
            result['passed'] = True
        except Exception as error:
            # Errors intentionally omit arbitrary text that might contain private paths.
            result['error_type'] = type(error).__name__
            result['error_frames'] = [{'file': Path(frame.filename).name, 'line': frame.lineno, 'function': frame.name} for frame in traceback.extract_tb(error.__traceback__)[-5:]]
            if isinstance(error, NameError): result['missing_identifier'] = error.name
            result['error_stage'] = 'native cookie fixture assertion or readiness'
            current = job(root/'data')
            result['observed_job_state_at_failure'] = current['job']['state'] if current else None
            result['worker_live_at_failure'] = bool(worker_info and service.same_live(worker_info))
            result['guardian_live_at_failure'] = bool(guardian and service.same_live(guardian))
            if owner: screenshot(out/'failure.png', 'dlpad', display)
        finally:
            if app: reap(app)
            for identity, fd in ((guardian, guardian_fd), (worker_info, worker_fd)):
                if fd is not None:
                    normalized = {'pid': identity['pid'], 'start_ticks': identity['start_ticks']}
                    if service.same_live(normalized): signal.pidfd_send_signal(fd, signal.SIGKILL)
                    os.close(fd)
            if owner: owner.close()
            if bus: reap(bus)
            for thread in threads: thread.join(3)
            if context: context.__exit__(None, None, None)
            calls_path = root/'calls.jsonl'
            if calls_path.exists():
                if calls_path.stat().st_size > 65536: raise RuntimeError('synthetic tool calls evidence bound exceeded')
                calls = [json.loads(line) for line in calls_path.read_text().splitlines()]
                if len(calls) > 16: raise RuntimeError('synthetic tool calls count exceeded')
                sanitized = [{'stage': item['stage'], 'synthetic_private_copy_path': item['copy'], 'literal_bytes': len(item['literal'].encode()) if item['literal'] is not None else None, 'literal_sha256': hashlib.sha256(item['literal'].encode()).hexdigest() if item['literal'] is not None else None} for item in calls]
                service.write_json(out/'synthetic-tool-calls.json', sanitized)
                result['synthetic_tool_call_count'] = len(sanitized)
            database = root/'data/jobs.sqlite'
            if database.exists():
                if database.stat().st_size > 2097152: raise RuntimeError('synthetic database evidence bound exceeded')
                snapshot_database(root/'data', out/'jobs.sqlite')
                result['sqlite_backup_bytes'] = (out/'jobs.sqlite').stat().st_size
                result['sqlite_backup_sha256'] = file_hash(out/'jobs.sqlite')
                if result['sqlite_backup_bytes'] > 2097152: raise RuntimeError('synthetic database backup bound exceeded')
            result['source_preserved'] = source.exists() and (source.stat().st_dev, source.stat().st_ino, file_hash(source)) == source_before
            result['sentinel_preserved'] = sentinel.exists() and (sentinel.stat().st_dev, sentinel.stat().st_ino, file_hash(sentinel)) == sentinel_before
            result['source_sha256_after'] = file_hash(source) if source.exists() else None
            result['sentinel_sha256_after'] = file_hash(sentinel) if sentinel.exists() else None
            if deferred_files:
                result['deferred_fixture_preserved_after_new_job'] = all(item.exists() and item.is_file() and not item.is_symlink() and metadata == {'bytes': item.stat().st_size, 'sha256': file_hash(item), 'device': item.stat().st_dev, 'inode': item.stat().st_ino} for item, metadata in deferred_files.items())
                for item in deferred_files:
                    if item.name in ('recovery.json', 'owner.marker') and item.is_file() and not item.is_symlink() and item.stat().st_size <= 8192:
                        with (out/('deferred-after-'+item.name)).open('xb') as record_evidence: record_evidence.write(item.read_bytes())
                result['passed'] &= result['deferred_fixture_preserved_after_new_job']
            needles = [SECRET, str(source)] + ([private_copy] if private_copy else [])
            result['app_log_no_secret_or_source'] = no_leak((out/'app.log').read_bytes() if (out/'app.log').exists() else b'', needles)
            result['journal_no_secret_or_cookie_paths'] = all(no_leak(path.read_bytes(), needles) for path in (root/'data').glob('jobs.sqlite*'))
            result['owned_cleanup'] = all(child is None or child.poll() is not None for child in (app, bus)) and all(not thread.is_alive() for thread in threads)
            result['passed'] &= all(result[key] for key in ('source_preserved', 'sentinel_preserved', 'app_log_no_secret_or_source', 'journal_no_secret_or_cookie_paths', 'owned_cleanup'))
    service.write_json(out/'result.json', result)
    print(name, result['passed'], flush=True)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--expected-sha', required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--font', type=Path, default=Path('/mnt/c/Windows/Fonts/malgun.ttf'))
    parser.add_argument('--fixture-binary', type=Path)
    parser.add_argument('--fixture-sha')
    parser.add_argument('--case', action='append', choices=('success', 'cancel', 'malformed', 'unsafe-cache', 'same-boot'))
    args = parser.parse_args()
    binary = args.binary
    evidence = Path(__file__).resolve().parent.parent/'docs/validation/evidence'
    if not binary.is_absolute() or not binary.is_relative_to(Path('/tmp')) or binary.is_symlink() or binary.name != 'dlpad-cpu' or any(parent.is_symlink() for parent in binary.parents) or not binary.is_file() or sorted(binary.parent.iterdir()) != [binary] or file_hash(binary) != args.expected_sha:
        parser.error('immutable explicit /tmp single-app snapshot required')
    if not args.output.is_absolute() or not args.output.is_relative_to(evidence) or args.output == evidence or '..' in args.output.parts or args.output.exists() or any(parent.is_symlink() for parent in [args.output, *args.output.parents]):
        parser.error('new explicit project evidence path required')
    if 'same-boot' in (args.case or ()):
        fixture = args.fixture_binary
        if not fixture or not args.fixture_sha or not fixture.is_absolute() or not fixture.is_relative_to(Path('/tmp')) or not fixture.is_file() or fixture.is_symlink() or any(parent.is_symlink() for parent in fixture.parents) or file_hash(fixture) != args.fixture_sha:
            parser.error('frozen explicit /tmp fixture driver SHA required')
    args.output.mkdir(mode=0o700)
    host_env = dict(os.environ)
    host_env.update(XDG_RUNTIME_DIR=f'/run/user/{os.getuid()}', DBUS_SESSION_BUS_ADDRESS=f'unix:path=/run/user/{os.getuid()}/bus')
    result = {'started_utc': service.utc_now(), 'boot_id': Path('/proc/sys/kernel/random/boot_id').read_text().strip(), 'binary_sha256_before': file_hash(binary), 'probe_sha256': file_hash(Path(__file__)), 'synthetic_credentials_only': True, 'desktop_fallback': False, 'cases': {}}
    for name in args.case or ('success', 'cancel', 'malformed'):
        result['cases'][name] = run_case(args, name, host_env)
    result['binary_sha256_after'] = file_hash(binary)
    if args.fixture_binary:
        result['fixture_binary_sha256_after'] = file_hash(args.fixture_binary)
    result['passed'] = all(case['passed'] for case in result['cases'].values()) and result['binary_sha256_after'] == args.expected_sha and sorted(binary.parent.iterdir()) == [binary]
    if args.fixture_binary: result['passed'] &= result['fixture_binary_sha256_after'] == args.fixture_sha
    result['ended_utc'] = service.utc_now()
    service.write_json(args.output/'result.json', result)
    return 0 if result['passed'] else 2


if __name__ == '__main__':
    raise SystemExit(main())
