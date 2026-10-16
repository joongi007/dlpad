#!/usr/bin/env python3
"""Bounded private Windows S10 GUI evidence; no PowerShell or clipboard access."""
import argparse
import json
import shutil
from pathlib import Path

import windows_current_workspace_probe as n


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    # Fixed negative case, no source/scenario/command override.
    parser.set_defaults(scenario='large-json')
    args = parser.parse_args()
    out = args.output
    if (not out.is_absolute() or '..' in out.parts or out.exists()
            or not out.is_relative_to(n.ROOT / 'docs/validation/evidence')
            or any(p.is_symlink() for p in [out, *out.parents])):
        parser.error('fresh project evidence directory required')
    source = n.ROOT / 'scripts/windows_s10_job_peak_negative_probe.cs'
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
        for original, copied in [(n.TOOLS / 'target/release/dlpad-cpu.exe', binary),
                                 (n.TOOLS / 'target/debug/examples/session_cookie_fixture_tool.exe', fixture)]:
            if original.is_symlink() or not 0 < original.stat().st_size <= 256 * 1024 * 1024:
                raise ValueError('original binary bound/type')
            before = n.sha(original)
            shutil.copyfile(original, copied)
            if n.sha(copied) != before or n.sha(original) != before:
                raise ValueError('original binary changed during copy')
        inputs = {'binary_sha': n.sha(binary), 'fixture_sha': n.sha(fixture),
                  'observer_sha': n.sha(observer), 'collector_source_sha': collector_sha,
                  'runner_sha': runner_sha, 'source': n.snapshot()}
        (out / 'inputs.json').write_text(json.dumps(inputs, indent=2) + '\n')
        for mode in [('all' if args.scenario == 'omit-all' else 'single')]:
            print('S10 standalone private GUI ' + mode, flush=True)
            argv = [host, '150', n.win(observer), n.win(binary), inputs['binary_sha'],
                    n.win(fixture), inputs['fixture_sha'], n.win(out / ('gui-' + mode)),
                    'metadata-failure', mode, args.scenario]
            record = n.run('gui-' + args.scenario, argv, env, out, 150)
            record['immutable_inputs_unchanged'] = (
                n.sha(binary) == inputs['binary_sha'] and n.sha(fixture) == inputs['fixture_sha']
                and n.sha(observer) == inputs['observer_sha'] and n.sha(source) == collector_sha
                and n.sha(Path(__file__).resolve()) == runner_sha)
            record['original_native_operation_passed'] = record['passed']
            negative = json.loads((out / ('gui-' + mode) / 'result.json').read_text())
            measured = negative.get('stress_cost', {})
            raw = json.loads((out / ('gui-' + mode) / 'stress-cost-samples.json').read_text())
            record['expected_negative_accepted'] = (
                record.get('exit_code') == 2 and negative.get('status') == 'FAIL'
                and raw.get('observer_fault') is not None
                and 'Fixed negative info9 invalid-HANDLE' in raw['observer_fault']
                and raw.get('whole_job_committed_peak_measured') is False
                and measured.get('whole_job_committed_peak_measured') is False
                and (negative.get('negative_failure_natural_close') is True
                     or negative.get('wm_close_exited') is True)
                and negative.get('owned_job_active_before_cleanup') == 0
                and negative.get('owned_job_empty_after_cleanup') is True
                and not negative.get('owned_job_terminated', False)
                and negative.get('private_desktop_closed') is True
                and negative.get('mta_desktop_restored') is True
                and record.get('source_unchanged') is True
                and record.get('private_desktop_reported') is True
                and record.get('input_desktop_unchanged') is True
                and negative.get('input_desktop_unchanged') is True)
            record['passed'] = record['expected_negative_accepted'] and record['immutable_inputs_unchanged']
            operations.append(record)
            print(json.dumps({'operation': mode, 'passed': record['passed']}), flush=True)
            if not record['passed']:
                break
    result = {'passed': len(operations) == 2 and all(r['passed'] for r in operations),
              'operations': operations, 'scenario': args.scenario,
              'scope': 'private S10 app WS/private samples and OS whole-Job committed peaks; not whole-Job RSS; root DB/export/image/budget review required; clipboard untested'}
    (out / 'collector-result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'passed': result['passed'], 'evidence': str(out)}), flush=True)
    return 0 if result['passed'] else 2


if __name__ == '__main__':
    raise SystemExit(main())
