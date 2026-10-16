#!/usr/bin/env python3
"""Freeze native driver/helper inputs, then reuse the existing terminal regression."""
import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

PROJECT = Path(__file__).resolve().parent.parent
EXCLUDED = Path('crates/engine/src/cookies/reader_journal.rs')

def sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(65536), b''):
            digest.update(block)
    return digest.hexdigest()

def inputs():
    paths = set(PROJECT.glob('crates/**/src/**/*.rs'))
    paths.update(PROJECT.glob('crates/**/tests/**/*.rs'))
    paths.update(PROJECT.glob('crates/**/Cargo.toml'))
    paths.update(PROJECT.glob('scripts/*.py'))
    paths.update(PROJECT / name for name in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml'))
    paths.discard(PROJECT / EXCLUDED)
    return {str(path.relative_to(PROJECT)): sha(path) for path in sorted(paths) if path.is_file()}

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('driver', type=Path)
    parser.add_argument('helper', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    evidence = PROJECT / 'docs/validation/evidence'
    out = args.output
    if (not out.is_absolute() or '..' in out.parts or not out.is_relative_to(evidence)
            or out.exists() or any(p.is_symlink() for p in [out, *out.parents])):
        parser.error('new absolute project evidence path required')
    for binary in (args.driver, args.helper):
        if not binary.is_file() or binary.is_symlink():
            parser.error('regular explicit built driver and helper required')
    out.mkdir(mode=0o700)
    before = inputs()
    excluded_path = PROJECT / EXCLUDED
    record = {'source_before': before, 'excluded_uncompiled': {
        'path': str(EXCLUDED), 'reason': 'root contract: new module is not wired into compiled source',
        'sha256': sha(excluded_path) if excluded_path.is_file() else None},
        'driver_sha256': sha(args.driver), 'helper_sha256': sha(args.helper),
        'existing_units_targeted': False, 'native_executed': False}
    code = 2
    try:
        with tempfile.TemporaryDirectory(prefix='dlpad-reader-drain-binaries-', dir='/tmp') as folder:
            owned = Path(folder)
            driver = owned / 'terminal-driver'
            helper = owned / 'dlpad-process-guard'
            for original, copied in ((args.driver, driver), (args.helper, helper)):
                shutil.copyfile(original, copied)
                copied.chmod(0o500)
            if sha(driver) != record['driver_sha256'] or sha(helper) != record['helper_sha256']:
                raise RuntimeError('input changed during immutable copy')
            command = [sys.executable, str(PROJECT / 'scripts/terminal_user_service_probe.py'),
                       str(driver), str(out / 'native')]
            record['command'] = command
            record['native_executed'] = True
            with (out / 'native.log').open('xb') as log:
                completed = subprocess.run(command, stdin=subprocess.DEVNULL, stdout=log,
                                           stderr=subprocess.STDOUT, check=False)
            record['probe_exit_code'] = completed.returncode
            record['copied_binaries_unchanged'] = (
                sha(driver) == record['driver_sha256'] and sha(helper) == record['helper_sha256'])
            result_file = out / 'native/result.json'
            if result_file.is_file() and result_file.stat().st_size <= 2 * 1024 * 1024:
                native = json.loads(result_file.read_text())
                record['native_passed'] = native.get('passed') is True
                record['native_case_count'] = len(native.get('cases', {}))
            code = completed.returncode
        record['owned_binary_temp_removed'] = not owned.exists()
    except Exception as error:
        record['error'] = str(error)
    finally:
        record['source_after'] = inputs()
        record['source_unchanged'] = before == record['source_after']
        record['original_binaries_unchanged'] = (
            sha(args.driver) == record['driver_sha256'] and sha(args.helper) == record['helper_sha256'])
        record['passed'] = (code == 0 and record.get('native_passed') is True
                            and record.get('native_case_count') == 22
                            and record.get('copied_binaries_unchanged') is True
                            and record.get('owned_binary_temp_removed') is True
                            and record['source_unchanged'] and record['original_binaries_unchanged'])
        (out / 'wrapper.json').write_text(json.dumps(record, indent=2) + '\n')
    print(json.dumps({'passed': record['passed'], 'native_case_count': record.get('native_case_count')}))
    return 0 if record['passed'] else 2

if __name__ == '__main__':
    raise SystemExit(main())
