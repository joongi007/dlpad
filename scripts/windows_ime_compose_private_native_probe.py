#!/usr/bin/env python3
"""Bounded private native IME capability observation, without input mutation."""
import argparse
import json
from pathlib import Path
import shutil
import windows_current_workspace_probe as n


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--case', required=True, choices=['cancel', 'commit'])
    args = parser.parse_args()
    out = args.output
    if (not out.is_absolute() or '..' in out.parts or out.exists()
            or not out.is_relative_to(n.ROOT / 'docs/validation/evidence')
            or any(p.is_symlink() for p in [out, *out.parents])):
        parser.error('fresh project evidence directory required')
    source = n.ROOT / 'scripts/windows_ime_compose_private_probe.cs'
    original = n.TOOLS / 'target/release/dlpad-cpu.exe'
    source_sha, runner_sha, binary_sha = n.sha(source), n.sha(Path(__file__)), n.sha(original)
    if original.is_symlink() or not 0 < original.stat().st_size <= 256 * 1024 * 1024:
        parser.error('bounded original binary required')
    out.mkdir(mode=0o700)
    (out / 'inputs').mkdir(mode=0o700)
    binary = out / 'inputs/dlpad-cpu.exe'
    shutil.copyfile(original, binary)
    if n.sha(binary) != binary_sha or n.sha(original) != binary_sha:
        raise ValueError('binary changed while copying')
    _, env = n.configure()
    host = str(n.TOOLS / 'windows-cli-host.exe')
    framework = r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319'
    observer = out / 'observer.exe'
    operations = []
    inputs = {'source_sha': source_sha, 'runner_sha': runner_sha, 'binary_sha': binary_sha,
              'source': n.snapshot()}
    (out / 'inputs.json').write_text(json.dumps(inputs, indent=2) + '\n')
    operations.append(n.run('compile', [host, '45', framework + '\\csc.exe', '/nologo',
        '/reference:System.Drawing.dll', '/reference:System.Web.Extensions.dll',
        *['/reference:' + framework + '\\WPF\\' + name + '.dll'
          for name in ['UIAutomationClient', 'UIAutomationTypes', 'WindowsBase']],
        '/out:' + n.win(observer), n.win(source)], env, out, 45))
    if operations[-1]['passed']:
        inputs['observer_sha'] = n.sha(observer)
        (out / 'inputs.json').write_text(json.dumps(inputs, indent=2) + '\n')
        record = n.run('gui', [host, '150', n.win(observer), n.win(binary), binary_sha,
                              n.win(out / 'gui'), args.case], env, out, 150)
        record['immutable_inputs_unchanged'] = (
            n.sha(source) == source_sha and n.sha(Path(__file__)) == runner_sha
            and n.sha(binary) == binary_sha and n.sha(original) == binary_sha
            and n.sha(observer) == inputs['observer_sha'])
        record['passed'] = record['passed'] and record['immutable_inputs_unchanged']
        operations.append(record)
    result = {'passed': len(operations) == 2 and all(r['passed'] for r in operations),
              'operations': operations,
              'scope': 'synthetic native IMM transport; physical Korean keys and whole P0 unverified'}
    (out / 'collector-result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'passed': result['passed'], 'evidence': str(out)}), flush=True)
    return 0 if result['passed'] else 2


if __name__ == '__main__':
    raise SystemExit(main())
