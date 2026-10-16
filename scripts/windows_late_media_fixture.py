"""Root-only owned Windows fixture experiment; records actual generation cost separately."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile
import time

PROJECT = Path('/mnt/d/project/dlpad')
sys.path.insert(0, str(PROJECT / 'scripts'))
import windows_msvc_run_selected_cli_collect as h
n = h.n
CAP = 256 * 1024 * 1024


def streamed_sha(path):
    path = h.plain(path)
    with path.open('rb') as file:
        initial = os.fstat(file.fileno())
        if not 0 < initial.st_size <= CAP:
            raise ValueError('bounded nonempty original file required')
        digest = hashlib.sha256()
        total = 0
        while True:
            block = file.read(min(1024 * 1024, initial.st_size - total + 1))
            if not block:
                break
            total += len(block)
            if total > initial.st_size:
                raise ValueError('source growth rejected')
            digest.update(block)
        final = os.fstat(file.fileno())
        current = path.stat()
        key = lambda info: (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns)
        if total != initial.st_size or key(initial) != key(final) or key(final) != key(current):
            raise ValueError('original file changed while hashing')
        return {'sha256': digest.hexdigest(), 'bytes': total, 'read_buffer_bytes': 1024 * 1024}


def metadata_from_log(raw):
    text = raw.decode('utf-8', 'strict')
    if len(raw) > 1024 * 1024:
        raise ValueError('metadata log cap')
    decoder = json.JSONDecoder()
    matches = []
    for pos, value in enumerate(text):
        if value != '{':
            continue
        try:
            row, _ = decoder.raw_decode(text[pos:])
        except ValueError:
            continue
        if isinstance(row, dict) and isinstance(row.get('format'), dict):
            matches.append(row)
    if len(matches) != 1:
        raise ValueError('exactly one native ffprobe metadata object required')
    return matches[0]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--ffmpeg', type=Path, required=True)
    parser.add_argument('--ffmpeg-sha256', required=True)
    parser.add_argument('--ffprobe-sha256', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    before = n.snapshot()
    ffmpeg = h.plain(args.ffmpeg)
    ffprobe = h.plain(ffmpeg.parent / 'ffprobe.exe')
    host = h.plain(n.TOOLS / 'windows-cli-host.exe')
    original_tools = {str(path): streamed_sha(path) for path in (ffmpeg, ffprobe, host)}
    for path, expected in ((ffmpeg, args.ffmpeg_sha256), (ffprobe, args.ffprobe_sha256)):
        if len(expected) != 64 or original_tools[str(path)]['sha256'] != expected.lower():
            raise ValueError('exact pinned original tool SHA required')
    output = h.plain(args.output)
    if output.exists() or not output.is_relative_to(n.ROOT / 'docs/validation/evidence'):
        raise ValueError('fresh project evidence required')
    temporary_parent = h.plain(n.TOOLS / 'tmp')
    if not temporary_parent.is_dir():
        raise ValueError('existing original project TEMP required')
    output.mkdir(mode=0o700)
    owned = h.plain(Path(tempfile.mkdtemp(prefix='dlpad-late8h-owned-', dir=temporary_parent)))
    fixture = h.plain(owned / 'red-blue-8h.mp4')
    if fixture.exists():
        raise ValueError('create-only fixture')
    configuration, env = n.configure()
    operations = []
    result = {'status': 'FAIL', 'source_before': before, 'original_tools': original_tools,
              'fixture_root': str(owned), 'fixture': str(fixture), 'operations': operations,
              'generation_timeout_seconds': 900, 'configured_output_limit_bytes': CAP,
              'max_log_bytes': 8 * 1024 * 1024, 'generation_cost_included': True,
              'actual_frames_planned': 57600, 'whole_source_loaded_into_memory': False,
              'root_review_pending': True, 'late_export_executed': False,
              'strict_disk_hard_cap_proven': False}
    try:
        start = time.monotonic()
        operation = n.run('generate', [str(host), '900', n.win(ffmpeg),
            '-nostdin', '-hide_banner', '-loglevel', 'error', '-max_alloc', '33554432',
            '-filter_threads', '1', '-filter_complex_threads', '1',
            '-f', 'lavfi', '-i', 'color=c=red:s=32x32:r=2:d=28798',
            '-f', 'lavfi', '-i', 'color=c=blue:s=32x32:r=2:d=2',
            '-filter_complex', '[0:v][1:v]concat=n=2:v=1:a=0[v]', '-map', '[v]',
            '-an', '-c:v', 'mpeg4', '-q:v', '3', '-threads', '1', '-pix_fmt', 'yuv420p',
            '-t', '28800', '-fs', str(CAP), '-f', 'mp4', '-n', n.win(fixture)], env, output, 900)
        operations.append(operation)
        result['generation_elapsed_seconds'] = time.monotonic() - start
        h.guard(operation, h.bounded(output / 'generate.log'))
        result['generated_source'] = streamed_sha(fixture)
        operation = n.run('probe', [str(host), '30', n.win(ffprobe), '-v', 'error',
            '-protocol_whitelist', 'file', '-show_entries',
            'format=duration,size:stream=codec_type,codec_name,width,height,r_frame_rate,nb_frames',
            '-of', 'json', '-i', n.win(fixture)], env, output, 30)
        operations.append(operation)
        raw = h.bounded(output / 'probe.log', 1024 * 1024)
        h.guard(operation, raw)
        metadata = metadata_from_log(raw)
        duration_ms = round(float(metadata['format']['duration']) * 1000)
        if abs(duration_ms - 28_800_000) > 500:
            raise ValueError('actual 8h duration required')
        streams = metadata.get('streams', [])
        if len(streams) != 1 or streams[0].get('codec_type') != 'video' or streams[0].get('codec_name') != 'mpeg4' or streams[0].get('width') != 32 or streams[0].get('height') != 32 or streams[0].get('r_frame_rate') != '2/1' or int(streams[0].get('nb_frames', 0)) != 57600:
            raise ValueError('actual encoded low-rate 8h video required')
        if streamed_sha(fixture) != result['generated_source']:
            raise ValueError('probe changed original fixture bytes')
        if n.snapshot() != before or any(streamed_sha(Path(path)) != proof for path, proof in original_tools.items()):
            raise ValueError('source/tool identity changed')
        result.update(status='REVIEW_REQUIRED', metadata=metadata, source_after=n.snapshot(),
                      source_unchanged=True, fixture_duration_ms=duration_ms,
                      fixture_windows_path=n.win(fixture), ffmpeg_windows_path=n.win(ffmpeg))
    except BaseException as error:
        result['failure_kind'] = type(error).__name__
        raise
    finally:
        # Keep every partial fixture and original tool; no pathname cleanup.
        if fixture.is_file():
            result['observed_fixture_bytes_on_exit'] = fixture.stat().st_size
        h.write(output / 'result.json', result)
    print(json.dumps({'status': result['status'], 'fixture': str(fixture),
                      'generation_elapsed_seconds': result['generation_elapsed_seconds'],
                      'source': result['generated_source'], 'late_export_executed': False}))


if __name__ == '__main__':
    main()
