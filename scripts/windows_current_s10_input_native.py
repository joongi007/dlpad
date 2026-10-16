#!/usr/bin/env python3
"""Bounded private Windows S10 GUI evidence; no PowerShell or clipboard access."""
import argparse
import json
import shutil
from pathlib import Path

import windows_current_workspace_probe as n
from windows_msvc_workspace_probe import require_current_whole


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--scenario', required=True, choices=['save', 'save-collision', 'edit', 'omit-one', 'omit-all', 'mpd-attach', 'mpd-remove', 'mpd-reattach', 'mpd-all', 'mpd-unsafe', 'mpd-large'])
    parser.add_argument('--proof', type=Path, required=True)
    args = parser.parse_args()
    whole = json.loads(args.proof.read_text())
    before = n.snapshot()
    require_current_whole(whole, before)
    out = args.output
    if (not out.is_absolute() or '..' in out.parts or out.exists()
            or not out.is_relative_to(n.ROOT / 'docs/validation/evidence')
            or any(p.is_symlink() for p in [out, *out.parents])):
        parser.error('fresh project evidence directory required')
    source = n.ROOT / 'scripts/windows_current_s10_input_probe.cs'
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
        for original, copied in [(n.TOOLS / 'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release/dlpad-cpu.exe', binary),
                                 (n.TOOLS / 'target/debug/examples/session_cookie_fixture_tool.exe', fixture)]:
            if original.is_symlink() or not 0 < original.stat().st_size <= 256 * 1024 * 1024:
                raise ValueError('original binary bound/type')
            pin = n.sha(original)
            if copied == binary and whole['artifacts']['dlpad-cpu.exe'] != {'sha256': pin, 'bytes': original.stat().st_size}:
                raise ValueError('current CPU artifact mismatch')
            shutil.copyfile(original, copied)
            if n.sha(copied) != pin or n.sha(original) != pin:
                raise ValueError('original binary changed during copy')
        inputs = {'binary_sha': n.sha(binary), 'fixture_sha': n.sha(fixture),
                  'observer_sha': n.sha(observer), 'collector_source_sha': collector_sha,
                  'runner_sha': runner_sha, 'source': n.snapshot()}
        (out / 'inputs.json').write_text(json.dumps(inputs, indent=2) + '\n')
        for mode in [('all' if args.scenario == 'omit-all' else 'single')]:
            print('S10 standalone private GUI ' + mode, flush=True)
            argv = [host, '240', n.win(observer), n.win(binary), inputs['binary_sha'],
                    n.win(fixture), inputs['fixture_sha'], n.win(out / ('gui-' + mode)),
                    'metadata-failure', mode, args.scenario]
            record = n.run('gui-' + args.scenario, argv, env, out, 240)
            record['immutable_inputs_unchanged'] = (
                n.sha(binary) == inputs['binary_sha'] and n.sha(fixture) == inputs['fixture_sha']
                and n.sha(observer) == inputs['observer_sha'] and n.sha(source) == collector_sha
                and n.sha(Path(__file__).resolve()) == runner_sha)
            record['passed'] = record['passed'] and record['immutable_inputs_unchanged']
            operations.append(record)
            print(json.dumps({'operation': mode, 'passed': record['passed']}), flush=True)
            if not record['passed']:
                break
    result = {'passed': len(operations) == 2 and all(r['passed'] for r in operations),
              'operations': operations, 'scenario': args.scenario,
              'scope': 'current MSVC private S10 GUI; requires root DB/export/image review; clipboard untested',
              'whole_proof_sha256': n.sha(args.proof), 'source_before': before, 'source_after': n.snapshot()}
    result['passed'] = result['passed'] and result['source_after'] == before
    (out / 'collector-result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'passed': result['passed'], 'evidence': str(out)}), flush=True)
    return 0 if result['passed'] else 2


if __name__ == '__main__':
    raise SystemExit(main())
