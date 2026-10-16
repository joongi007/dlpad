#!/usr/bin/env python3
"""Windows paired public failure/cancel evidence on an owned private desktop."""
import argparse
import json
from pathlib import Path
import windows_current_workspace_probe as native

SCENARIOS = ('metadata-failure', 'download-failure', 'cancel')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    out = args.output
    if (not out.is_absolute() or '..' in out.parts or out.exists()
            or not out.is_relative_to(native.ROOT / 'docs/validation/evidence')
            or any(p.is_symlink() for p in (out, *out.parents))):
        parser.error('fresh project evidence directory required')
    collector = Path(__file__).resolve()
    collector_sha = native.sha(collector)
    baseline = native.snapshot()
    out.mkdir(mode=0o700)
    _, env = native.configure()
    host = native.TOOLS / 'windows-cli-host.exe'
    cargo = native.TOOLS / 'rust/bin/cargo.exe'
    fixture = native.TOOLS / 'target/debug/examples/windows_paired_download_fixture.exe'
    operations = {}
    acceptance = {}
    operations['build'] = native.run('build', [str(host), '600', native.win(cargo),
        'build', '-p', 'dlpad-engine', '--example', 'windows_paired_download_fixture',
        '--locked'], env, out, 600)
    binary_sha = native.sha(fixture) if operations['build']['passed'] else None
    if operations['build']['passed']:
        for scenario in SCENARIOS:
            evidence = out / (scenario + '-fixture.json')
            operations[scenario] = native.run(scenario, [str(host), '90',
                native.win(fixture), '--fixture-run', native.win(evidence),
                '--scenario', scenario], env, out, 90)
            if not operations[scenario]['passed']:
                break
            if not evidence.is_file() or evidence.stat().st_size > 16384:
                acceptance[scenario] = {'passed': False, 'reason': 'missing/oversized evidence'}
                break
            data = json.loads(evidence.read_text(encoding='utf-8'))
            required = ('finish_and_reuse', 'source_cookie_preserved',
                'cache_sentinel_preserved', 'db_authentication_absent',
                'authenticated_output_omitted', 'cookie_owner_removed')
            expected_state = 'Cancelled' if scenario == 'cancel' else 'Failed'
            acceptance[scenario] = {'passed': data.get('passed') is True
                and data.get('scenario') == scenario
                and data.get('job_state') == expected_state
                and data.get('artifact_count') == 0
                and all(data.get(key) is True for key in required), 'fixture': data}
            if not acceptance[scenario]['passed']:
                break
    final = native.snapshot()
    binary_unchanged = binary_sha is not None and native.sha(fixture) == binary_sha
    passed = (baseline == final and collector_sha == native.sha(collector)
        and binary_unchanged and len(acceptance) == len(SCENARIOS)
        and all(r['passed'] for r in operations.values())
        and all(r['passed'] for r in acceptance.values()))
    result = {'passed': passed, 'operations': operations, 'acceptance': acceptance,
        'source_baseline': baseline, 'source_final': final,
        'source_unchanged': baseline == final, 'binary_sha256': binary_sha,
        'binary_unchanged': binary_unchanged, 'collector_sha256': collector_sha,
        'collector_unchanged': collector_sha == native.sha(collector),
        'scope': 'Windows GNU synthetic paired public failures/cancel; parent crash and startup recovery excluded'}
    with (out / 'result.json').open('x', encoding='utf-8') as stream:
        json.dump(result, stream, indent=2)
        stream.write('\n')
    print(json.dumps({'passed': passed, 'completed': list(acceptance)}))
    return 0 if passed else 2


if __name__ == '__main__':
    raise SystemExit(main())
