"""Root-run local export private GUI collection; no acceptance without root review."""
import argparse
import hashlib
import json
import os
import stat
import sys
from pathlib import Path

PROJECT = Path('/mnt/d/project/dlpad')
sys.path.insert(0, str(PROJECT / 'scripts'))
import windows_msvc_run_selected_cli_collect as h
n = h.n


STREAM_LIMIT = 256 * 1024 * 1024


def stream_sha(path):
    path = h.plain(path)
    initial = path.stat()
    if not stat.S_ISREG(initial.st_mode) or initial.st_size > STREAM_LIMIT:
        raise ValueError('stream original file/size bound')
    identity = lambda value: (value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns)
    digest, total = hashlib.sha256(), 0
    with path.open('rb') as source:
        if identity(os.fstat(source.fileno())) != identity(initial):
            raise ValueError('stream original open identity')
        while block := source.read(65536):
            total += len(block)
            if total > initial.st_size or total > STREAM_LIMIT:
                raise ValueError('stream original grew')
            digest.update(block)
        if identity(os.fstat(source.fileno())) != identity(initial):
            raise ValueError('stream original handle changed')
    if total != initial.st_size or identity(path.stat()) != identity(initial):
        raise ValueError('stream original path changed')
    return digest.hexdigest()


def stream_copy(source, destination):
    source, destination = h.plain(source), h.plain(destination)
    expected = stream_sha(source)
    initial = source.stat()
    identity = lambda value: (value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns)
    total = 0
    with source.open('rb') as reader, destination.open('xb') as writer:
        if identity(os.fstat(reader.fileno())) != identity(initial):
            raise ValueError('copy original open identity')
        while block := reader.read(65536):
            total += len(block)
            if total > initial.st_size or total > STREAM_LIMIT:
                raise ValueError('copy original grew')
            writer.write(block)
        writer.flush()
        os.fsync(writer.fileno())
        if identity(os.fstat(reader.fileno())) != identity(initial):
            raise ValueError('copy original handle changed')
    if total != initial.st_size or identity(source.stat()) != identity(initial) or stream_sha(source) != expected or stream_sha(destination) != expected:
        raise ValueError('copy original/postcopy pin')


def validate_native_receipt(receipt, scenario):
    if scenario not in {'success', 'cancel', 'close', 'close-stop'}:
        raise ValueError('exact supported local scenario')
    if receipt.get('status') != 'REVIEW_REQUIRED' or receipt.get('scenario') != scenario:
        raise ValueError('actual local export collection failed')
    native = receipt['native']
    for key in ['wm_close_exited', 'original_handles_close_safe', 'owned_job_naturally_empty',
                'owned_job_empty_after_cleanup', 'private_desktop_closed', 'input_desktop_unchanged', 'mta_desktop_restored']:
        if native.get(key) is not True:
            raise ValueError('original private natural lifecycle ' + key)
    if native.get('natural_exit_code') != 0 or native.get('owned_job_active_before_cleanup') != 0 or native.get('owned_job_terminated', False):
        raise ValueError('original natural zero/Job0 only')
    if receipt.get('global_input') is not False or receipt.get('clipboard_invoked') is not False:
        raise ValueError('private local input scope')
    if native.get('actual_range_field_readback_verified') is not True or native.get('requested_start_ms') != 1100 or native.get('requested_end_ms') != 1800:
        raise ValueError('actual 1100..1800ms local range')
    if scenario == 'success' and (native.get('final_success_label_verified') is not True or native.get('same_file_stage_final_verified') is not True):
        raise ValueError('final/stage actual success')
    if scenario == 'cancel' and native.get('cancel_invoked_while_actual_running') is not True:
        raise ValueError('actual running cancel absent')
    if scenario == 'close' and native.get('close_prompt_visible_and_return_verified') is not True:
        raise ValueError('actual running close prompt absent')
    if scenario == 'close-stop':
        for key in ['export_running_observed', 'close_stop_prompt_title_verified',
                    'stop_exit_invoked_while_actual_running', 'close_stop_exit_observed',
                    'closewait_label_observed', 'closewait_inflight_verified',
                    'worker_ack_inferred_from_production_natural_exit']:
            if native.get(key) is not True:
                raise ValueError('actual close-stop required observation ' + key)
        if native.get('direct_worker_ack_observed') is not False or native.get('publication_review_pending_after_close') is not True:
            raise ValueError('close-stop direct ACK/publication review scope')
    return native


def main():
    parser = argparse.ArgumentParser()
    for name in ['proof', 'output', 'observer-source', 'ffmpeg', 'ffprobe', 'video']:
        parser.add_argument('--' + name, type=Path, required=True)
    for name in ['ffmpeg-sha256', 'ffprobe-sha256', 'video-sha256']:
        parser.add_argument('--' + name, required=True)
    parser.add_argument('--scenario', choices=['success', 'cancel', 'close', 'close-stop'], required=True)
    args = parser.parse_args()
    before = n.snapshot()
    proof = h.plain(args.proof)
    proofraw = h.bounded(proof)
    whole = json.loads(proofraw)
    h.require_current_whole(whole, before)
    logs = []
    for name, operation in whole['operations'].items():
        log = h.plain(proof.parent / (name + '.log'))
        h.guard(operation, h.bounded(log))
        logs.append(log)
    release = n.TOOLS / 'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release'
    binaries = [h.plain(release / name) for name in ['dlpad-cpu.exe', 'dlpad-local-install.exe', 'dlpad-install.exe']]
    for binary in binaries:
        if whole['artifacts'][binary.name] != {'sha256': stream_sha(binary), 'bytes': binary.stat().st_size}:
            raise ValueError('whole original artifact mismatch')
    cpu = binaries[0]
    ffmpeg, ffprobe, video = [h.plain(path) for path in [args.ffmpeg, args.ffprobe, args.video]]
    for original, expected in [(ffmpeg, args.ffmpeg_sha256), (ffprobe, args.ffprobe_sha256), (video, args.video_sha256)]:
        if len(expected) != 64 or any(c not in '0123456789abcdef' for c in expected) or stream_sha(original) != expected:
            raise ValueError('explicit original tool/video SHA pin mismatch')
    if ffprobe != ffmpeg.parent / 'ffprobe.exe' or not 0 < video.stat().st_size <= 1048576:
        raise ValueError('explicit sibling FFprobe/bounded synthetic video')
    observer = h.plain(args.observer_source)
    host = h.plain(n.TOOLS / 'windows-cli-host.exe')
    config = h.plain(proof.parent / 'effective-configuration.json')
    configraw = h.bounded(config, 65536)
    out = h.plain(args.output)
    if out.exists() or not out.is_relative_to(n.ROOT / 'docs/validation/evidence') or not out.parent.is_dir():
        raise ValueError('fresh project evidence required')
    out.mkdir(mode=0o700)
    inputs = out / 'inputs'
    inputs.mkdir(mode=0o700)
    originals = [proof, config, *binaries, ffmpeg, ffprobe, video, host, observer,
                 Path(__file__).resolve(), Path(h.__file__).resolve(), Path(n.__file__).resolve(),
                 n.ROOT / 'scripts/windows_msvc_workspace_probe.py', n.ROOT / 'Cargo.lock',
                 n.TOOLS / 'target/release/dlpad-cpu.exe', n.TOOLS / 'target/release/dlpad-local-install.exe', *logs]
    pins = {str(path): stream_sha(path) for path in originals}
    for original, name in [(proof, 'whole-proof.json'), (config, 'effective-configuration.json'),
                           (cpu, 'cpu.exe'), (ffmpeg, 'ffmpeg.exe'), (ffprobe, 'ffprobe.exe'),
                           (video, 'synthetic-red-blue.mkv'), (observer, 'observer.cs'),
                           (Path(__file__).resolve(), 'collector.py')]:
        stream_copy(original, inputs / name)
    if stream_sha(inputs / 'whole-proof.json') != hashlib.sha256(proofraw).hexdigest() or stream_sha(inputs / 'effective-configuration.json') != hashlib.sha256(configraw).hexdigest():
        raise ValueError('decoded proof/config copy binding')
    pins.update({str(path): stream_sha(path) for path in inputs.iterdir()})
    _, env = n.configure()
    # Explicit Windows-owned TEMP is supplied by n.configure/WSLENV. No /tmp or
    # ambient external TEMP fallback; C# confines all fixture roots below this root.
    expected_temp = n.win(h.plain(n.TOOLS / 'tmp'))
    if env.get('TEMP') != expected_temp or env.get('TMP') != expected_temp:
        raise ValueError('own Windows TEMP configuration')
    operations = []
    result = {'schema': 1, 'status': 'FAIL', 'source_before': before, 'pins': pins,
              'operations': operations, 'scenario': args.scenario, 'root_review_pending': True,
              'native_id_review_pending': True, 'pixel_review_pending': True,
              'independent_duration_blue_decode_pending': True,
              'physical_input': False, 'actual_ime': False, 'physical_reader': False,
              'release_ready': False, 'actual_inflight_gui_cancel_verified': False,
              'actual_inflight_gui_close_ack_verified': False,
              'close_stop_production_ack_inference_review_pending': args.scenario == 'close-stop'}
    try:
        framework = r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319'
        executable = out / 'observer.exe'
        operation = n.run('compile', [str(host), '60', framework + '\\csc.exe', '/nologo',
                           '/reference:System.Drawing.dll', '/reference:System.Web.Extensions.dll',
                           *['/reference:' + framework + '\\WPF\\' + name + '.dll' for name in ['UIAutomationClient', 'UIAutomationTypes', 'WindowsBase']],
                           '/out:' + n.win(executable), n.win(inputs / 'observer.cs')], env, out, 60)
        operations.append(operation)
        h.guard(operation, h.bounded(out / 'compile.log'))
        pins[str(executable)] = stream_sha(executable)
        operation = n.run('native-gui', [str(host), '320', n.win(executable),
                          n.win(inputs / 'cpu.exe'), whole['artifacts']['dlpad-cpu.exe']['sha256'],
                          n.win(ffmpeg), args.ffmpeg_sha256,
                          n.win(ffprobe), args.ffprobe_sha256,
                          n.win(video), args.video_sha256,
                          n.win(out / 'gui'), args.scenario], env, out, 320)
        operations.append(operation)
        raw = h.bounded(out / 'native-gui.log')
        h.guard(operation, raw)
        # C# writes its bounded receipt create-only. Parse it from owned evidence,
        # because its Main doesn't print a JSON receipt to stdout.
        receipt_path = h.plain(out / 'gui/local-export-result.json')
        receipt = json.loads(h.bounded(receipt_path, 65536))
        native = validate_native_receipt(receipt, args.scenario)
        owned = h.winpath(receipt['owned_root'])
        if not owned.is_relative_to(n.TOOLS / 'tmp') or not owned.name.startswith('dlpad-local-export-owned-'):
            raise ValueError('owned TEMP fixture root confinement')
        copy_source = h.plain(owned / 'media/synthetic-red-blue.mkv')
        if stream_sha(copy_source) != args.video_sha256 or receipt.get('source_sha256') != args.video_sha256 or receipt.get('CPU_sha256') != whole['artifacts']['dlpad-cpu.exe']['sha256'] or receipt.get('ffmpeg_sha256') != args.ffmpeg_sha256 or receipt.get('ffprobe_sha256') != args.ffprobe_sha256:
            raise ValueError('observed original CPU/tools/source pin binding')
        rows = native['export_files']
        if not isinstance(rows, list) or len(rows) > 2:
            raise ValueError('bounded local export output receipt')
        expected_names = {'clip.mp4', 'stage.mp4'}
        for row in rows:
            path = h.winpath(row['path'])
            if not path.is_relative_to(owned / 'media') or path.name not in expected_names or path.parent.parent != owned / 'media':
                raise ValueError('exact fresh local output confinement')
            if not isinstance(row['bytes'], int) or not 0 <= row['bytes'] <= 256 * 1024 * 1024:
                raise ValueError('collector bounded synthetic export bytes')
            if stream_sha(path) != row['sha256'] or path.stat().st_size != row['bytes']:
                raise ValueError('original exported bytes pin changed')
            pins[str(path)] = row['sha256']
        if native.get('final_success_label_verified') is True:
            if len(rows) != 2 or {h.winpath(row['path']).name for row in rows} != expected_names or any(rows[0][key] != rows[1][key] for key in ['native_id', 'sha256', 'bytes']):
                raise ValueError('original same-file stage/final facts')
            parent = h.winpath(rows[0]['path']).parent
            h.names(parent, expected_names)
        pins[str(copy_source)] = args.video_sha256
        pins[str(receipt_path)] = stream_sha(receipt_path)
        if n.snapshot() != before or any(stream_sha(Path(path)) != expected for path, expected in pins.items()):
            raise ValueError('whole source or original immutable artifact changed')
        result.update(status='REVIEW_REQUIRED', source_after=n.snapshot(), source_unchanged=True,
                      owned_fixture_root=str(owned), fixture_receipt=receipt)
    except BaseException as error:
        result['failure_kind'] = type(error).__name__
        raise
    finally:
        h.write(out / 'result.json', result)
    print(json.dumps({'status': result['status'], 'root_review_pending': True,
                      'independent_duration_blue_decode_pending': True}))

if __name__ == '__main__':
    main()
