#!/usr/bin/env python3
"""Root-reviewed private native experiment; dry plan unless --execute is explicit."""
import argparse
import hashlib
import json
import os
import re
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path('/mnt/d/project/dlpad')
TEMP = ROOT / '.local-tools/windows/tmp'
LIMIT = 268435456
CASES = {
    'fine-timebase': ['-enc_time_base:v', '1:1000', '-fps_mode:v', 'passthrough'],
    'fps30': ['-vf', 'fps=30:start_time=0'],
    'setpts': ['-vf', 'setpts=PTS-STARTPTS'],
}

def sha(path):
    initial = path.stat()
    if not path.is_file() or initial.st_size > LIMIT:
        raise ValueError('hash byte bound')
    signature = lambda value: (value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns)
    digest = hashlib.sha256()
    total = 0
    with path.open('rb') as handle:
        if signature(os.fstat(handle.fileno())) != signature(initial):
            raise ValueError('hash original identity changed')
        for block in iter(lambda: handle.read(65536), b''):
            total += len(block)
            if total > initial.st_size:
                raise ValueError('hash grew')
            digest.update(block)
        if signature(os.fstat(handle.fileno())) != signature(initial):
            raise ValueError('hash original changed')
    if total != initial.st_size or signature(path.stat()) != signature(initial):
        raise ValueError('hash postcheck changed')
    return digest.hexdigest()

def plain(path):
    path = Path(path).absolute()
    if '..' in path.parts:
        raise ValueError('parent traversal')
    for node in (path, *path.parents):
        if node.is_symlink():
            raise ValueError('symlink path')
    path.relative_to(ROOT)
    return path

def pinned(path, expected):
    path = plain(path)
    if not re.fullmatch('[0-9a-f]{64}', expected) or not path.is_file():
        raise ValueError('original lowercase SHA/file required')
    if sha(path) != expected:
        raise ValueError('original pin mismatch: ' + str(path))
    return path

def arguments(source, output, extra):
    return ['-nostdin', '-hide_banner', '-loglevel', 'error', '-threads', '1',
            '-filter_threads', '1', '-max_alloc', '33554432', '-protocol_whitelist', 'file',
            '-ss', '28798.250', '-accurate_seek', '-i', source, '-t', '1.500',
            '-map', '0:v:0', '-map', '0:a:0?', '-sn', '-dn', *extra,
            '-c:v', 'mpeg4', '-threads', '1', '-q:v', '3', '-c:a', 'aac',
            '-movflags', '+faststart', '-fs', str(LIMIT), '-f', 'mp4', '-n', output]

def metadata(raw):
    text = raw.decode('utf-8', 'strict')
    start = text.find('{')
    if start < 0:
        raise ValueError('probe JSON absent')
    value, _ = json.JSONDecoder().raw_decode(text[start:])
    if not isinstance(value, dict) or 'streams' not in value or 'format' not in value:
        raise ValueError('probe metadata absent')
    return value

def write(path, value):
    with path.open('x', encoding='utf-8') as handle:
        json.dump(value, handle, indent=2)
        handle.write('\n')

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('source', 'ffmpeg', 'ffprobe', 'host'):
        parser.add_argument('--' + name, type=Path, required=True)
        parser.add_argument('--' + name + '-sha256', required=True)
    parser.add_argument('--output', type=Path, required=True,
                        help='fresh directory under project docs/validation/evidence')
    parser.add_argument('--execute', action='store_true')
    args = parser.parse_args()
    files = {name: pinned(getattr(args, name), getattr(args, name + '_sha256'))
             for name in ('source', 'ffmpeg', 'ffprobe', 'host')}
    files['source'].relative_to(TEMP)
    if not files['source'].parent.name.startswith('dlpad-late8h-owned-'):
        raise ValueError('original owned late8h fixture required')
    if files['source'].stat().st_size > LIMIT:
        raise ValueError('source byte bound')
    output = plain(args.output)
    output.relative_to(ROOT / 'docs/validation/evidence')
    if output.exists() or not output.parent.is_dir():
        raise ValueError('fresh evidence leaf under existing parent required')
    plan = {'status': 'PLAN_ONLY', 'native_executed': False,
            'source_pins': {name: {'path': str(path), 'sha256': getattr(args, name + '_sha256')}
                            for name, path in files.items()},
            'cases': CASES, 'requested_start_ms': 28798250, 'requested_end_ms': 28799750,
            'encode_seconds_cap': 120, 'probe_seconds_cap': 30,
            'max_output_bytes': LIMIT, 'strict_disk_cap_proven': False,
            'fps30_frame_multiplier_for_2fps': 15,
            'policy_accepted': False, 'audio_sync_proven': False}
    if not args.execute:
        print(json.dumps(plan, indent=2))
        return
    sys.path.insert(0, str(ROOT / 'scripts'))
    import windows_current_workspace_probe as n
    import windows_msvc_run_selected_cli_collect as h
    if files['host'] != plain(n.TOOLS / 'windows-cli-host.exe'):
        raise ValueError('guarded original host location required')
    output.mkdir()
    work = Path(tempfile.mkdtemp(prefix='dlpad-export-timing-owned-', dir=TEMP))
    _, env = n.configure()
    results = dict(plan, status='REVIEW_REQUIRED', native_executed=True,
                   work_root=str(work), operations=[], source_unchanged=False)
    write(output / 'plan.json', plan)
    try:
        for name, extra in CASES.items():
            # Revalidate originals before every encode/probe; never replace or delete them.
            for key, path in files.items():
                pinned(path, getattr(args, key + '_sha256'))
            target = work / (name + '.mp4')
            argv = arguments(n.win(files['source']), n.win(target), extra)
            start = time.monotonic()
            op = n.run(name + '-encode', [str(files['host']), '120', n.win(files['ffmpeg']), *argv],
                       env, output, 120)
            results['operations'].append({'case': name, 'role': 'encode',
                                          'elapsed_including_collector_seconds': time.monotonic() - start,
                                          'native_elapsed_seconds': op['elapsed_seconds'],
                                          'argv': op['argv'], 'passed_process_guard': op['passed']})
            h.guard(op, h.bounded(output / (name + '-encode.log')))
            target = plain(target)
            if not target.is_file() or not 0 < target.stat().st_size <= LIMIT:
                raise ValueError('output missing or byte bound exceeded; preserve partial')
            probe_argv = ['-v', 'error', '-protocol_whitelist', 'file', '-show_entries',
                          'format=start_time,duration,size:stream=codec_type,start_time,duration,nb_frames,time_base,avg_frame_rate,r_frame_rate',
                          '-of', 'json', n.win(target)]
            op = n.run(name + '-probe', [str(files['host']), '30', n.win(files['ffprobe']), *probe_argv],
                       env, output, 30)
            raw = h.bounded(output / (name + '-probe.log'))
            h.guard(op, raw)
            results['operations'].append({'case': name, 'role': 'probe', 'argv': op['argv'],
                                          'native_elapsed_seconds': op['elapsed_seconds'],
                                          'metadata': metadata(raw), 'passed_process_guard': True,
                                          'output': str(target), 'sha256': sha(target),
                                          'bytes': target.stat().st_size})
    except BaseException as error:
        results['status'] = 'FAILED_PRESERVED'
        results['failure'] = type(error).__name__ + ': ' + str(error)
        raise
    finally:
        results['original_pins_on_exit'] = {key: sha(path) for key, path in files.items()}
        results['source_unchanged'] = all(results['original_pins_on_exit'][key] ==
                                          getattr(args, key + '_sha256') for key in files)
        if not results['source_unchanged']:
            results['status'] = 'FAILED_ORIGINAL_CHANGED'
        write(output / 'result.json', results)
    if not results['source_unchanged']:
        raise ValueError('original pin changed')
    print(str(output / 'result.json'))

if __name__ == '__main__':
    main()
