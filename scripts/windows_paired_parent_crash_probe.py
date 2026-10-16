#!/usr/bin/env python3
"""Synthetic parent-only Windows crash and independent receipt recovery evidence."""
import argparse
import json
from pathlib import Path
import windows_current_workspace_probe as native


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    out = args.output
    if (not out.is_absolute() or '..' in out.parts or out.exists()
            or not out.is_relative_to(native.ROOT / 'docs/validation/evidence')
            or any(p.is_symlink() for p in (out, *out.parents))):
        parser.error('fresh project evidence directory required')
    sources = (Path(__file__).resolve(), native.ROOT / 'scripts/windows_paired_parent_crash_observer.cs')
    collector_before = {str(p.relative_to(native.ROOT)): native.sha(p) for p in sources}
    baseline = native.snapshot()
    out.mkdir(mode=0o700)
    _, env = native.configure()
    host = native.TOOLS / 'windows-cli-host.exe'
    fixture = native.TOOLS / 'target/debug/examples/windows_paired_parent_crash_fixture.exe'
    plans = (
        ('build', [str(host), '600', native.win(native.TOOLS / 'rust/bin/cargo.exe'),
            'build', '-p', 'dlpad-engine', '--example', 'windows_paired_parent_crash_fixture', '--locked'], 600),
        ('compile-observer', [str(host), '30', r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe',
            '/nologo', '/reference:System.Web.Extensions.dll', '/out:' + native.win(out / 'observer.exe'),
            native.win(sources[1])], 30),
        ('native', [str(host), '90', native.win(out / 'observer.exe'), native.win(fixture),
            native.win(out / 'native')], 90),
    )
    operations = {}
    binary_sha = None
    for name, argv, seconds in plans:
        operations[name] = native.run(name, argv, env, out, seconds)
        if not operations[name]['passed']:
            break
        if name == 'build':
            binary_sha = native.sha(fixture)
    proof = {}
    proof_path = out / 'native/result.json'
    if proof_path.is_file() and proof_path.stat().st_size <= 65536:
        proof = json.loads(proof_path.read_text(encoding='utf-8-sig'))
    inspected = proof.get('inspection', {})
    valid = (proof.get('status') == 'PASS' and proof.get('parent_exit') == 73
        and proof.get('owned_job_empty') is True and proof.get('private_desktop_closed') is True
        and proof.get('input_desktop_unchanged') is True
        and all(inspected.get(key) is True for key in (
            'both_receipts_valid', 'source_foreign_preserved', 'managed_recovered', 'cookie_recovered')))
    final = native.snapshot()
    collector_after = {str(p.relative_to(native.ROOT)): native.sha(p) for p in sources}
    unchanged = binary_sha is not None and native.sha(fixture) == binary_sha
    passed = (len(operations) == 3 and all(r['passed'] for r in operations.values())
        and valid and baseline == final and collector_before == collector_after and unchanged)
    result = {'passed': passed, 'operations': operations, 'native_proof': proof,
        'source_baseline': baseline, 'source_final': final, 'source_unchanged': baseline == final,
        'collector_source_before': collector_before, 'collector_source_after': collector_after,
        'binary_sha256': binary_sha, 'binary_unchanged': unchanged,
        'scope': 'Windows GNU public paired parent-only kill, actual receipts and independent recovery; no GUI crash claim'}
    with (out / 'result.json').open('x', encoding='utf-8') as stream:
        json.dump(result, stream, indent=2)
        stream.write('\n')
    print(json.dumps({'passed': passed, 'operations': list(operations)}))
    return 0 if passed else 2


if __name__ == '__main__':
    raise SystemExit(main())
