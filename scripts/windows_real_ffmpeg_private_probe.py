#!/usr/bin/env python3
"""Reproducible owned Windows real-media fixture; root must serialize Cargo/native runs."""
import argparse
import hashlib
import json
import ntpath
import os
from pathlib import Path
import re
import sys

KEYS = {'schema', 'yt_dlp', 'ffmpeg', 'ffprobe', 'license'}
MAX_TOOL = 256 * 1024 * 1024
PINS = {'ffmpeg': '0e482b853f2fd4821055d20cdce510f0eea6027037c04b994f454a2136e8868e',
        'ffprobe': 'b9627185ffbad1d4fc6ef953868ca9ad8df75fc8dd035fcab58aca78caa2d23e'}


def duplicate_rejected(items):
    result = {}
    for key, value in items:
        if key in result:
            raise ValueError('duplicate manifest key')
        result[key] = value
    return result


def manifest_shape(value):
    if not isinstance(value, dict) or set(value) != KEYS or type(value['schema']) is not int or value['schema'] != 1:
        raise ValueError('exact manifest schema required')
    for name in KEYS - {'schema'}:
        item = value[name]
        if not isinstance(item, dict) or set(item) != {'path', 'bytes', 'sha256'}:
            raise ValueError('exact tool fields required')
        if type(item['bytes']) is not int or not 0 < item['bytes'] <= MAX_TOOL:
            raise ValueError('tool bytes bound')
        if not isinstance(item['sha256'], str) or not re.fullmatch('[0-9a-f]{64}', item['sha256']):
            raise ValueError('exact lowercase SHA required')
        raw = item['path']
        if not isinstance(raw, str) or not raw or len(raw) > 32766 or '\0' in raw or '"' in raw:
            raise ValueError('tool path bound')
        canonical = raw[4:] if raw.startswith('\\\\?\\') else raw
        drive, tail = ntpath.splitdrive(canonical)
        if not re.fullmatch('[A-Za-z]:', drive) or not tail.startswith('\\') or '/' in tail or any(part in {'.', '..'} for part in tail.split('\\')):
            raise ValueError('absolute native drive path required')
        if name in PINS and item['sha256'] != PINS[name]:
            raise ValueError('fixture supports current fixed FFmpeg pins only')
    return value


def checked_parts(path):
    if '..' in path.parts:
        raise ValueError('parent traversal rejected')
    return path.absolute()


def regular(path, limit):
    path = checked_parts(path)
    if any(p.is_symlink() for p in [path, *path.parents]) or not path.is_file() or not 0 < path.stat().st_size <= limit:
        raise ValueError('bounded regular file without symlink ancestors required')
    return path


def hash_stream(file, limit):
    h = hashlib.sha256()
    count = 0
    while block := file.read(min(65536, limit - count + 1)):
        count += len(block)
        if count > limit:
            raise ValueError('actual hashing read bound exceeded')
        h.update(block)
    return h.hexdigest()


def sha(path, limit=MAX_TOOL):
    path = regular(path, limit)
    with path.open('rb') as file:
        return hash_stream(file, limit)


def native_file(raw):
    raw = raw[4:] if raw.startswith('\\\\?\\') else raw
    drive, tail = ntpath.splitdrive(raw)
    if not re.fullmatch('[A-Za-z]:', drive) or not tail.startswith('\\') or '/' in tail or any(part in {'.', '..'} for part in tail.split('\\')):
        raise ValueError('converted native path traversal or authority rejected')
    return checked_parts(Path('/mnt') / drive[0].lower() / tail.lstrip('\\').replace('\\', '/'))


def load_manifest(path):
    path = regular(path, 65536)
    with path.open('rb') as file:
        raw = file.read(65537)
    if len(raw) > 65536:
        raise ValueError('manifest read bound')
    value = manifest_shape(json.loads(raw.decode('utf-8'), object_pairs_hook=duplicate_rejected))
    for name in KEYS - {'schema'}:
        item = value[name]
        original = regular(native_file(item['path']), MAX_TOOL)
        if original.stat().st_size != item['bytes'] or sha(original) != item['sha256']:
            raise ValueError('original actual tool pin mismatch')
    return path, raw, value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--tools-manifest', type=Path, required=True)
    parser.add_argument('--project', type=Path, default=Path.cwd())
    args = parser.parse_args()
    project = checked_parts(args.project)
    if not project.is_dir() or any(p.is_symlink() for p in [project, *project.parents]):
        parser.error('regular project root without symlink ancestors required')
    out = checked_parts(args.output)
    expected_evidence = project / 'docs/validation/evidence'
    if out.exists() or not out.is_relative_to(expected_evidence) or any(p.is_symlink() for p in [out, *out.parents]):
        parser.error('fresh project evidence required')
    checked_parts(args.tools_manifest)
    regular(project / 'scripts/windows_current_workspace_probe.py', 512 * 1024)
    sys.path.insert(0, str(project / 'scripts'))
    import windows_current_workspace_probe as n
    if project.resolve() != n.ROOT.resolve():
        parser.error('exact native helper project required')
    out = checked_parts(args.output)
    if out.exists() or not out.is_relative_to(n.ROOT / 'docs/validation/evidence') or any(p.is_symlink() for p in [out, *out.parents]):
        parser.error('fresh project evidence required')
    if not args.tools_manifest.is_absolute():
        parser.error('original absolute manifest required')
    original_manifest, raw, tools = load_manifest(args.tools_manifest)
    out.mkdir(mode=0o700)
    frozen_manifest = out / 'tools-manifest.json'
    with frozen_manifest.open('xb') as file:
        file.write(raw)
        file.flush()
        os.fsync(file.fileno())
    source = n.snapshot()
    manifest_sha = sha(original_manifest, 65536)
    _, env = n.configure()
    host = str(n.TOOLS / 'windows-cli-host.exe')
    build = n.run('build-fixture', [host, '600', n.win(n.TOOLS / 'rust/bin/cargo.exe'),
        'build', '-p', 'dlpad-engine', '--example', 'windows_real_ffmpeg_cut_fixture',
        '--release', '--locked'], env, out, 600)
    report = {'schema': 1, 'passed': False, 'build': build, 'source_before': source,
              'original_manifest': str(original_manifest), 'manifest_sha256': manifest_sha,
              'scope': 'actual synthetic loopback media, fixed external tools; no supply adoption'}
    if build['passed']:
        executable = regular(n.TOOLS / 'target/release/examples/windows_real_ffmpeg_cut_fixture.exe', MAX_TOOL)
        fixture_sha = sha(executable)
        report['fixture_sha256'] = fixture_sha
        report['fixture_bytes'] = executable.stat().st_size
        result = n.run('actual-media', [host, '240', n.win(executable), n.win(frozen_manifest),
            n.win(out / 'native-media-result.json')], env, out, 240)
        report['native'] = result
        report['fixture_sha_after'] = sha(executable)
        report['passed'] = result['passed'] and report['fixture_sha_after'] == fixture_sha
        result_path = out / 'native-media-result.json'
        if result_path.exists():
            regular(result_path, 2 * 1024 * 1024)
            value = json.loads(result_path.read_text(encoding='utf-8'))
            report['native_summary'] = {k: value.get(k) for k in ['status', 'current_phase', 'current_cut_id', 'error', 'owned_root', 'original_tool_hashes_preserved', 'foreign_sentinel_preserved']}
            report['passed'] = report['passed'] and value.get('status') == 'PASS'
        else:
            report['passed'] = False
    report['source_after'] = n.snapshot()
    try:
        report['pins_unchanged'] = source == report['source_after'] and sha(original_manifest, 65536) == manifest_sha and sha(frozen_manifest, 65536) == manifest_sha
        load_manifest(original_manifest)
    except (ValueError, OSError):
        report['pins_unchanged'] = False
        report['pin_check_error'] = 'original pin revalidation failed' 
    report['passed'] = report['passed'] and report['pins_unchanged']
    with (out / 'result.json').open('x', encoding='utf-8') as file:
        json.dump(report, file, indent=2)
    # Preserve original owned TEMP evidence; native fixture owns cleanup/terminal.
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
