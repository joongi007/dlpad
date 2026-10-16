"""Staged first-observed private client sampler; execution requires root scheduling."""
import argparse
import json
import ntpath
import re
import sys
import os
import hashlib
from pathlib import Path
# Standalone staging uses explicit project helper; no production script changes.
# Run from the project root or supply an explicit existing project root.
project = Path(os.environ.get('DLPAD_PROBE_PROJECT', os.getcwd())).absolute()
if not (project / 'scripts/windows_current_workspace_probe.py').is_file():
    raise RuntimeError('project-relative native helper unavailable')
sys.path.insert(0, str(project / 'scripts'))
import windows_current_workspace_probe as n


def pinned_file(path, limit):
    path = path.absolute()
    if any(p.is_symlink() for p in [path, *path.parents]) or not path.is_file():
        raise ValueError('regular original path and no symlink ancestors required')
    if path.stat().st_size <= 0 or path.stat().st_size > limit:
        raise ValueError('original file size bound')
    return path


def copy_pinned(source, destination, expected, limit):
    h = hashlib.sha256()
    count = 0
    with source.open('rb') as src, destination.open('xb') as dst:
        while block := src.read(65536):
            count += len(block)
            if count > limit:
                raise ValueError('immutable copy bound')
            h.update(block)
            dst.write(block)
        dst.flush()
        os.fsync(dst.fileno())
    if not count or h.hexdigest() != expected:
        raise ValueError('immutable copy digest mismatch')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--sha256', required=True)
    parser.add_argument('--theme', choices=['Light', 'Dark', 'System'], required=True)
    parser.add_argument('--observer', type=Path, default=Path(__file__).with_suffix('.cs'))
    args = parser.parse_args()
    out = args.output.absolute()
    if not re.fullmatch('[0-9a-f]{64}', args.sha256):
        parser.error('exact frozen executable SHA required')
    if out.exists() or not out.is_relative_to(n.ROOT / 'docs/validation/evidence') or any(p.is_symlink() for p in [out, *out.parents]):
        parser.error('fresh owned evidence directory required')
    args.binary = pinned_file(args.binary, 256 * 1024 * 1024)
    args.observer = pinned_file(args.observer, 128 * 1024)
    if n.sha(args.binary) != args.sha256:
        parser.error('original executable pin required')
    out.mkdir(mode=0o700)
    frozen_binary = out / 'dlpad-frozen.exe'
    copy_pinned(args.binary, frozen_binary, args.sha256, 256 * 1024 * 1024)
    source = out / 'observer.cs'
    with args.observer.open('rb') as src, source.open('xb') as dst:
        raw = src.read(128 * 1024 + 1)
        if len(raw) > 128 * 1024:
            raise ValueError('observer source bound')
        dst.write(raw)
    before = n.snapshot()
    observer_sha = n.sha(source)
    values, env = n.configure()
    exe = out / 'observer.exe'
    csc = r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe'
    compiled = n.run('compile', [str(n.TOOLS / 'windows-cli-host.exe'), '45', csc,
        '/nologo', '/target:exe', '/out:' + n.win(exe), '/r:System.Drawing.dll',
        '/r:System.Web.Extensions.dll', n.win(source)], env, out, 45)
    report = {'version': 1, 'theme': args.theme, 'binary_sha256': args.sha256,
              'observer_sha256': observer_sha, 'first_presentation_verified': False,
              'source_before': before, 'compiled': compiled, 'passed': False, 'theme_pass_verified': False,
              'classification_requires_pixel_review': True}
    if compiled['passed']:
        import uuid
        root = ntpath.join(values['TEMP'], 'dlpad-first-theme-' + uuid.uuid4().hex)
        report['owned_native_root'] = root
        result = n.run('observe', [str(n.TOOLS / 'windows-cli-host.exe'), '45',
            n.win(exe), n.win(frozen_binary), args.sha256, root, args.theme], env, out, 45)
        report['native'] = result
        report['passed'] = result['passed']
    report['source_after'] = n.snapshot()
    report['pins_unchanged'] = (before == report['source_after'] and
        n.sha(source) == observer_sha and n.sha(args.binary) == args.sha256 and
        n.sha(frozen_binary) == args.sha256 and n.sha(args.observer) == observer_sha)
    report['passed'] = report['passed'] and report['pins_unchanged']
    # Native-owned root intentionally preserved. Inspect its original result/PNGs
    # after Job-empty confirmation; no path-based deletion/archive adoption here.
    with (out / 'result.json').open('x', encoding='utf-8') as f:
        json.dump(report, f, indent=2)
    return 0 if report['passed'] else 1

if __name__ == '__main__':
    raise SystemExit(main())
