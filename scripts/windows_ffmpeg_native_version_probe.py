#!/usr/bin/env python3
"""Owned private Windows FFmpeg version-only evidence; no desktop fallback."""
import argparse
import datetime
import json
from pathlib import Path
import re
import windows_current_workspace_probe as native

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / '.local-tools/ffmpeg-win64-inventory-f5uj2z2m'
EXPECTED = {
    'ffmpeg.exe': (134497280, '0e482b853f2fd4821055d20cdce510f0eea6027037c04b994f454a2136e8868e'),
    'ffprobe.exe': (134296064, 'b9627185ffbad1d4fc6ef953868ca9ad8df75fc8dd035fcab58aca78caa2d23e'),
}

def pins():
    result = {}
    for name, (size, digest) in EXPECTED.items():
        path = SOURCE / name
        if any(p.is_symlink() for p in [path, *path.parents]):
            raise ValueError('symlink source forbidden')
        actual = {'bytes': path.stat().st_size, 'sha256': native.sha(path)}
        if actual != {'bytes': size, 'sha256': digest}:
            raise ValueError('source component mismatch')
        result[name] = actual
    return result

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    out = args.output
    if not out.is_absolute() or not out.is_relative_to(ROOT / 'docs/validation/evidence') or out.exists() or '..' in out.parts or any(p.is_symlink() for p in [out, *out.parents]):
        parser.error('fresh project evidence required')
    before = pins()
    out.mkdir(mode=0o700)
    _, env = native.configure()
    records = {}
    for name in EXPECTED:
        operation = name.removesuffix('.exe')
        r = native.run(operation, [str(native.TOOLS / 'windows-cli-host.exe'), '20', native.win(SOURCE / name), '-version'], env, out, 20)
        text = (out / (operation + '.log')).read_text(errors='replace')
        versions = re.findall(r'^' + operation + r' version ([^\s]+)', text, re.M)
        config = re.findall(r'^configuration: (.*)$', text, re.M)
        r['actual_versions'] = versions
        r['configuration'] = config
        r['enabled_features'] = sorted(set(re.findall(r'--enable-([\w-]+)', '\n'.join(config))))
        r['disabled_features'] = sorted(set(re.findall(r'--disable-([\w-]+)', '\n'.join(config))))
        r['runtime_version_verified'] = versions == ['n9.0.2-17-g2a571b6068-20260930'] and len(config) == 1
        r['native_binary_operation_passed'] = r['exit_code'] == 0 and r['collector_failure'] is None and r['private_desktop_reported'] and r['input_desktop_unchanged'] and r['runtime_version_verified']
        records[operation] = r
    after = pins()
    result = {'recorded_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'binary_before': before, 'binary_after': after, 'binary_unchanged': before == after, 'operations': records, 'native_binary_operations_passed': before == after and all(r['native_binary_operation_passed'] for r in records.values()), 'workspace_snapshot_passed': all(r['source_unchanged'] for r in records.values()), 'runtime_rss_measured': False, 'supply_adopted': False}
    with (out / 'result.json').open('x') as f:
        json.dump(result, f, indent=2)
        f.write('\n')
    print(json.dumps({k: result[k] for k in ('native_binary_operations_passed', 'workspace_snapshot_passed', 'binary_unchanged')}))
    return 0 if result['native_binary_operations_passed'] else 2

if __name__ == '__main__':
    raise SystemExit(main())
