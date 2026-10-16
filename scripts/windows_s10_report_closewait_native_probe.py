#!/usr/bin/env python3
"""Bounded private Windows S10 GUI evidence; no PowerShell or clipboard access."""
import argparse
import json
import shutil
from pathlib import Path

import sys
# Staging invocation requires the project cwd; publication in scripts also works.
sys.path.insert(0, str(Path.cwd() / "scripts"))
import windows_current_workspace_probe as n


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--binary-sha', required=True)
    parser.add_argument('--fixture', type=Path, required=True)
    parser.add_argument('--fixture-sha', required=True)
    parser.add_argument('--observer-source', type=Path, required=True)
    parser.add_argument('--scenario', choices=['release', 'timeout', 'both'], default='both')
    args = parser.parse_args()
    out = args.output
    if (not out.is_absolute() or '..' in out.parts or out.exists()
            or not out.is_relative_to(n.ROOT / 'docs/validation/evidence')
            or any(p.is_symlink() for p in [out, *out.parents])):
        parser.error('fresh project evidence directory required')
    original_source = args.observer_source
    if (not original_source.is_absolute() or '..' in original_source.parts
            or any(p.is_symlink() for p in [original_source, *original_source.parents])
            or not (original_source.is_relative_to(n.ROOT) or original_source.is_relative_to(Path('/tmp')))
            or not original_source.is_file() or original_source.stat().st_size > 256*1024):
        raise ValueError('bounded plain ROOT/tmp observer source required')
    source = original_source.resolve()
    collector_sha = n.sha(source)
    runner_sha = n.sha(Path(__file__).resolve())
    out.mkdir(mode=0o700)
    (out / 'inputs').mkdir(mode=0o700)
    _, env = n.configure()
    host = str(n.TOOLS / 'windows-cli-host.exe')
    framework = r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319'
    observer = out / 'observer.exe'
    operations = []
    argv = [host, '45', framework + '\\csc.exe', '/nologo',
            '/reference:System.Drawing.dll', '/reference:System.Web.Extensions.dll',
            *['/reference:' + framework + '\\WPF\\' + name + '.dll'
              for name in ['UIAutomationClient', 'UIAutomationTypes', 'WindowsBase']],
            '/out:' + n.win(observer), n.win(source)]
    operations.append(n.run('compile', argv, env, out, 45))
    if operations[-1]['passed']:
        binary = out / 'inputs/dlpad-cpu.exe'
        fixture = out / 'inputs/session_cookie_fixture_tool.exe'
        for original, copied in [(args.binary, binary), (args.fixture, fixture)]:
            if (not original.is_absolute() or '..' in original.parts
                    or any(p.is_symlink() for p in [original, *original.parents])
                    or not original.is_file() or not 0 < original.stat().st_size <= 256 * 1024 * 1024):
                raise ValueError('original binary bound/type')
            before = n.sha(original)
            expected = args.binary_sha if copied == binary else args.fixture_sha
            if before != expected:
                raise ValueError("explicit immutable SHA mismatch")
            shutil.copyfile(original, copied)
            if n.sha(copied) != before or n.sha(original) != before:
                raise ValueError('original binary changed during copy')
        inputs = {'binary_sha': n.sha(binary), 'fixture_sha': n.sha(fixture),
                  'observer_sha': n.sha(observer), 'collector_source_sha': collector_sha,
                  'runner_sha': runner_sha, 'source': n.snapshot()}
        (out / 'inputs.json').write_text(json.dumps(inputs, indent=2) + '\n')
        scenarios = ['release', 'timeout'] if args.scenario == 'both' else [args.scenario]
        for mode in scenarios:
            print('S10 standalone private GUI ' + mode, flush=True)
            argv = [host, '150', n.win(observer), n.win(binary), inputs['binary_sha'],
                    n.win(fixture), inputs['fixture_sha'], n.win(out / ('gui-' + mode)),
                    'metadata-failure', mode]
            record = n.run('gui-' + mode, argv, env, out, 150)
            record['immutable_inputs_unchanged'] = (
                n.sha(binary) == inputs['binary_sha'] and n.sha(fixture) == inputs['fixture_sha']
                and n.sha(observer) == inputs['observer_sha'] and n.sha(source) == collector_sha
                and n.sha(Path(__file__).resolve()) == runner_sha)
            record['passed'] = record['passed'] and record['immutable_inputs_unchanged']
            raw_path = out / ('gui-' + mode) / 'result.json'
            if record['passed']:
                raw = json.loads(raw_path.read_text(encoding='utf-8-sig'))
                require = ['started_original_file_opened', 'pre_release_closewait_verified',
                           'worker_stop_ack_verified', 'owned_job_naturally_empty',
                           'owned_job_empty_after_cleanup', 'input_desktop_unchanged']
                record['native_scope_verified'] = (
                    raw.get('status') == 'REVIEW_REQUIRED'
                    and all(raw.get(key) is True for key in require)
                    and raw.get('natural_exitcode') == 0
                    and raw.get('scenario') == mode
                    and raw.get('closewait_inflight_verified') is (mode == 'release')
                    and raw.get('expected_timeout_failure_verified') is (mode == 'timeout'))
                record['passed'] = record['passed'] and record['native_scope_verified']
            operations.append(record)
            print(json.dumps({'operation': mode, 'passed': record['passed']}), flush=True)
            if not record['passed']:
                break
    result = {'passed': len(operations) == (3 if args.scenario == 'both' else 2) and all(r['passed'] for r in operations),
              'operations': operations, 'scenario': args.scenario,
              'scope': 'private fixture-feature CloseWait release and expected timeout failure; root DB/export/image review required; no production claim'}
    (out / 'collector-result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'passed': result['passed'], 'evidence': str(out)}), flush=True)
    return 0 if result['passed'] else 2


if __name__ == '__main__':
    raise SystemExit(main())
