#!/usr/bin/env python3
"""Create-only acceptance manifest for project-owned Windows recovery fixtures."""
import argparse
import json
import re
from pathlib import Path
import windows_current_workspace_probe as native


def write_manifest(root, case):
    if case not in ('completed', 'pending', 'malformed'):
        raise ValueError('unknown recovery case')
    root = Path(root)
    if (not root.is_absolute() or '..' in root.parts
            or not root.is_relative_to(native.ROOT / 'docs/validation/evidence')
            or root.name != 'owned-fixture'
            or not root.parent.name.startswith('dlpad-paired-parent-crash-')
            or any(p.is_symlink() for p in (root, *root.parents))):
        raise ValueError('owned project fixture namespace required')
    if (root / 'fixture.cap').read_bytes() != b'dlpad-paired-parent-crash-synthetic-v1':
        raise ValueError('original synthetic fixture capability required')
    managed = root / 'data/managed-tools'
    names = ['.execution-pending', '.execution-readers.json'] + [
        f'.execution-reader-{i}.receipt' for i in range(8)]
    records = [managed / name for name in names]
    protected = [root / 'fixture.cap', root / 'foreign-sentinel']
    protected.extend(p for p in (root / 'source.cookies',) if p.is_file())
    removed = []
    if case == 'completed':
        if not all(p.is_file() for p in records):
            raise ValueError('original managed records missing before GUI')
        tool = managed / 'yt-dlp/2026.09.30/x86_64-pc-windows-msvc/yt-dlp.exe'
        protected.append(tool)
        cache = root / 'data/session-cookies'
        owners = [p for p in cache.iterdir() if re.fullmatch(r'cookie-session-[0-9a-f]{64}', p.name)]
        if len(owners) != 1 or not owners[0].is_dir() or owners[0].is_symlink():
            raise ValueError('one original cookie owner required')
        protected.append(cache / 'foreign-sentinel')
        removed = records + owners
    else:
        protected.extend(records)
    for path in protected + removed:
        if any(p.is_symlink() for p in (path, *path.parents)) or not path.is_relative_to(root):
            raise ValueError('fixture namespace changed')
    if any(not p.is_file() or p.stat().st_size > 256 * 1024 * 1024 for p in protected):
        raise ValueError('protected file bound exceeded')
    manifest = {'schema': 1, 'case': case,
        'protected_files': [native.win(p) for p in protected],
        'removed_files': [native.win(p) for p in removed],
        'protected_sha256': {native.win(p): native.sha(p) for p in protected}}
    destination = root / 'gui-recovery-case.json'
    with destination.open('x', encoding='utf-8') as stream:
        json.dump(manifest, stream, indent=2)
        stream.write('\n')
    return destination


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root', type=Path)
    parser.add_argument('--case', choices=('completed', 'pending', 'malformed'), required=True)
    args = parser.parse_args()
    print(write_manifest(args.root, args.case))


if __name__ == '__main__':
    main()
