#!/usr/bin/env python3
"""Bounded private Windows settings GUI evidence; no PowerShell or clipboard access."""
import argparse
import json
from pathlib import Path

import windows_current_workspace_probe as n


def copy_new_bounded(original, destination, maximum):
    with original.open('rb') as stream, destination.open('xb') as target:
        total = 0
        while True:
            data = stream.read(min(65536, maximum - total + 1))
            if not data:
                break
            total += len(data)
            if total > maximum:
                raise ValueError('immutable input copy bound')
            target.write(data)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--seed', required=True, choices=['System','Light','Dark'])
    parser.add_argument('--select', required=True, choices=['System','Light','Dark'])
    parser.add_argument('--collector', required=True, type=Path)
    args = parser.parse_args()
    out = args.output
    if (not out.is_absolute() or '..' in out.parts or out.exists()
            or not out.is_relative_to(n.ROOT / 'docs/validation/evidence')
            or any(p.is_symlink() for p in [out, *out.parents])):
        parser.error('fresh project evidence directory required')
    candidate = args.collector
    if (not candidate.is_absolute() or '..' in candidate.parts
            or any(p.is_symlink() for p in [candidate, *candidate.parents])):
        parser.error('absolute no-symlink project collector required')
    source = candidate.resolve()
    if (not source.is_relative_to(n.ROOT) or source.suffix != '.cs'
            or not source.is_file() or not 0 < source.stat().st_size <= 1024 * 1024):
        parser.error('root-published project collector required')
    collector_sha = n.sha(source)
    runner_sha = n.sha(Path(__file__).resolve())
    out.mkdir(mode=0o700)
    (out / 'inputs').mkdir(mode=0o700)
    pinned_source = out / 'inputs/collector.cs'
    copy_new_bounded(source, pinned_source, 1024 * 1024)
    if n.sha(pinned_source) != collector_sha or n.sha(source) != collector_sha:
        raise ValueError('collector changed during pin copy')
    _, env = n.configure()
    host = str(n.TOOLS / 'windows-cli-host.exe')
    framework = r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319'
    observer = out / 'observer.exe'
    operations = []
    argv = [host, '45', framework + '\\csc.exe', '/nologo',
            '/reference:System.Drawing.dll', '/reference:System.Web.Extensions.dll',
            *['/reference:' + framework + '\\WPF\\' + name + '.dll'
              for name in ['UIAutomationClient', 'UIAutomationTypes', 'WindowsBase']],
            '/out:' + n.win(observer), n.win(pinned_source)]
    operations.append(n.run('compile', argv, env, out, 45))
    operations[-1]['collector_source_unchanged'] = (
        n.sha(source) == collector_sha and n.sha(pinned_source) == collector_sha)
    operations[-1]['passed'] = (operations[-1]['passed']
        and operations[-1]['collector_source_unchanged'])
    if operations[-1]['passed']:
        binary = out / 'inputs/dlpad-cpu.exe'
        
        for original, copied in [(n.TOOLS / 'target/release/dlpad-cpu.exe', binary)]:
            if original.is_symlink() or not 0 < original.stat().st_size <= 256 * 1024 * 1024:
                raise ValueError('original binary bound/type')
            before = n.sha(original)
            copy_new_bounded(original, copied, 256 * 1024 * 1024)
            if n.sha(copied) != before or n.sha(original) != before:
                raise ValueError('original binary changed during copy')
        inputs = {'binary_sha': n.sha(binary), 
                  'observer_sha': n.sha(observer), 'collector_source_sha': collector_sha,
                  'runner_sha': runner_sha, 'source': n.snapshot()}
        (out / 'inputs.json').write_text(json.dumps(inputs, indent=2) + '\n')
        argv = [host, '230', n.win(observer), n.win(binary), inputs['binary_sha'],
                n.win(out / 'gui-settings'), args.seed, args.select]
        record = n.run('gui-settings', argv, env, out, 230)
        record['immutable_inputs_unchanged'] = (n.sha(binary) == inputs['binary_sha']
            and n.sha(observer) == inputs['observer_sha'] and n.sha(source) == collector_sha
            and n.sha(Path(__file__).resolve()) == runner_sha and n.sha(pinned_source) == collector_sha)
        record['passed'] = record['passed'] and record['immutable_inputs_unchanged']
        operations.append(record)
    result = {'passed': len(operations) == 2 and all(r['passed'] for r in operations),
              'operations': operations, 'seed': args.seed, 'select': args.select,
              'scope': 'private settings GUI; requires root SQLite/UIA/pixel review; first presentation and actual OS change untested'}
    (out / 'collector-result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'passed': result['passed'], 'evidence': str(out)}), flush=True)
    return 0 if result['passed'] else 2


if __name__ == '__main__':
    raise SystemExit(main())
