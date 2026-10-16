"""Private native frame-phase diagnostic build; default artifacts remain pinned."""
import argparse
import hashlib
import json
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import windows_current_workspace_probe as n
from windows_selected_gui_collect import bounded, copy, guard, plain, sha, write
from windows_msvc_workspace_probe import require_current_whole


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--default-proof', type=Path, required=True)
    parser.add_argument('--configuration', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--reuse-owned-target', action='store_true')
    args = parser.parse_args()
    before = n.snapshot()
    default_proof = plain(args.default_proof)
    proof_raw = bounded(default_proof)
    whole = json.loads(proof_raw)
    require_current_whole(whole, before)
    default = n.TOOLS / 'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release'
    for leaf in ['dlpad-cpu.exe', 'dlpad-local-install.exe']:
        artifact = plain(default / leaf)
        if whole['artifacts'][leaf] != {'sha256': sha(artifact), 'bytes': artifact.stat().st_size}:
            raise ValueError('completed default proof current artifact required')
    configuration = plain(args.configuration)
    raw = bounded(configuration, 65536)
    config = json.loads(raw)
    commands = config.get('commands')
    if not isinstance(commands, list) or [item[0] for item in commands] != ['fmt', 'clippy', 'test', 'release']:
        raise ValueError('exact four ordered Cargo commands required')
    for name, command in commands:
        expected = {'fmt': 'fmt', 'clippy': 'clippy', 'test': 'test', 'release': 'build'}[name]
        if not isinstance(command, list) or not command or command[0] != expected:
            raise ValueError('actual Cargo subcommand required')
        if name == 'fmt':
            if command != ['fmt', '--all', '--', '--check']:
                raise ValueError('whole format check required')
        else:
            if '--workspace' not in command or '--locked' not in command or command.count('--features') != 1 or command[command.index('--features') + 1:command.index('--features') + 2] != ['cpu'] or command.count('--target') != 1 or command[command.index('--target') + 1:command.index('--target') + 2] != ['x86_64-pc-windows-msvc']:
                raise ValueError('exact locked CPU workspace MSVC target required')
            if name == 'clippy' and ('--all-targets' not in command or command[-3:] != ['--', '-D', 'warnings']):
                raise ValueError('strict all-target clippy required')
            if name == 'release' and '--release' not in command:
                raise ValueError('actual release required')
    out = plain(args.output)
    if out.exists() or not out.is_relative_to(n.ROOT / 'docs/validation/evidence'):
        raise ValueError('fresh project evidence required')
    target = n.TOOLS / 'msvc-supply-20261006/target-frame-phase-20261006'
    reuse = target.exists()
    marker = target / 'dlpad-experiment-owner.json'
    if reuse:
        if not args.reuse_owned_target or target.is_symlink():
            raise ValueError('explicit owned diagnostic target reuse required')
        owner = json.loads(bounded(plain(marker), 4096))
        prior = plain(Path(owner['evidence']))
        if owner.get('development_only') is not True or owner.get('frame_phase_diagnostic') is not True or not prior.is_relative_to(n.ROOT / 'docs/validation/evidence'):
            raise ValueError('original diagnostic ownership marker required')
        previous = json.loads(bounded(prior / 'result.json'))
        if previous.get('passed') is not True or previous.get('features') != 'cpu,first-paint-fixture' or previous.get('target') != 'x86_64-pc-windows-msvc' or previous.get('diagnostic_only') is not True:
            raise ValueError('completed original diagnostic build required')
        if previous.get('effective_values', {}).get('CARGO_TARGET_DIR') != n.win(target):
            raise ValueError('original diagnostic proof exact target required')
        frozen = plain(Path(previous['immutable_binary']))
        if frozen != prior / 'inputs/cpu.exe' or {'sha256': sha(frozen), 'bytes': frozen.stat().st_size} != previous.get('artifact'):
            raise ValueError('original immutable diagnostic artifact required')
    elif args.reuse_owned_target:
        raise ValueError('owned diagnostic target missing')
    for path in config['required_layout']:
        layout = plain(Path(path))
        if not layout.exists() or not layout.is_relative_to(n.TOOLS):
            raise ValueError('owned supply layout required')
    if n.snapshot() != before:
        raise ValueError("source changed during preflight")
    default = n.TOOLS / 'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release'
    originals = {str(path): {'sha256': sha(path), 'bytes': path.stat().st_size}
                 for path in [default_proof, configuration, n.ROOT / 'Cargo.lock',
                              default / 'dlpad-cpu.exe', default / 'dlpad-local-install.exe',
                              n.TOOLS / 'target/release/dlpad-cpu.exe',
                              n.TOOLS / 'target/release/dlpad-local-install.exe']}
    for leaf in ['dlpad-cpu.exe', 'dlpad-local-install.exe']:
        if originals[str(default / leaf)] != whole['artifacts'][leaf]:
            raise ValueError('captured default preservation pin differs from proof')
    out.mkdir()
    inputs = out / 'inputs'
    inputs.mkdir()
    copy(default_proof, inputs / 'default-proof.json')
    if sha(default_proof) != hashlib.sha256(proof_raw).hexdigest() or sha(inputs / 'default-proof.json') != hashlib.sha256(proof_raw).hexdigest():
        raise ValueError('decoded/default proof copy pin mismatch')
    copy(configuration, inputs / 'configuration.json')
    if sha(inputs / 'configuration.json') != hashlib.sha256(raw).hexdigest():
        raise ValueError('decoded configuration copy pin mismatch')
    copy(Path(__file__).resolve(), inputs / 'collector.py')
    for frozen in [inputs / 'default-proof.json', inputs / 'configuration.json', inputs / 'collector.py']:
        originals[str(frozen)] = {'sha256': sha(frozen), 'bytes': frozen.stat().st_size}
    if not reuse:
        target.mkdir()
        write(marker, {'development_only': True, 'evidence': str(out), 'frame_phase_diagnostic': True})
    originals[str(marker)] = {'sha256': sha(marker), 'bytes': marker.stat().st_size}
    _, env = n.configure()
    removed = set(config['remove_inherited'])
    for key in removed:
        env.pop(key, None)
    env['WSLENV'] = ':'.join(s for s in env.get('WSLENV', '').split(':')
                            if s.split('/')[0] not in removed)
    values = dict(config['values'])
    values.update(CARGO_TARGET_DIR=n.win(target), CARGO_BUILD_JOBS='2')
    env.update(values)
    forwarding = [s for s in env.get('WSLENV', '').split(':') if s]
    for key in values:
        if key not in {s.split('/')[0] for s in forwarding}:
            forwarding.append(key)
    env['WSLENV'] = ':'.join(forwarding)
    result = {'schema': 1, 'passed': False, 'status': 'FAIL', 'source_before': before,
              'target': 'x86_64-pc-windows-msvc', 'features': 'cpu,first-paint-fixture',
              'diagnostic_only': True, 'default_budget_acceptance': False,
              'default_proof_sha256': hashlib.sha256(proof_raw).hexdigest(),
              'default_proof_requires_completed_current_source': True,
              'originals': originals, 'effective_values': values, 'operations': {},
              'gui_verified': False, 'release_ready': False}
    try:
        for name, original_args in config['commands']:
            command = list(original_args)
            if '--features' in command:
                command[command.index('--features') + 1] = 'cpu,first-paint-fixture'
            op = n.run(name, [str(n.TOOLS / 'windows-cli-host.exe'), '1800',
                             n.win(n.TOOLS / 'rust/bin/cargo.exe'), *command],
                       env, out, 1800)
            result['operations'][name] = op
            guard(op, bounded(out / (name + '.log')))
        after = n.snapshot()
        if after != before or any({'sha256': sha(Path(path)), 'bytes': Path(path).stat().st_size} != pin
                                 for path, pin in originals.items()):
            raise ValueError('source/default/GNU/lock/config preservation failed')
        binary = plain(target / 'x86_64-pc-windows-msvc/release/dlpad-cpu.exe')
        copy(binary, inputs / 'cpu.exe')
        pin = {'sha256': sha(binary), 'bytes': binary.stat().st_size}
        if {'sha256': sha(inputs / 'cpu.exe'), 'bytes': (inputs / 'cpu.exe').stat().st_size} != pin:
            raise ValueError('immutable diagnostic binary mismatch')
        result.update(passed=True, status='REVIEW_REQUIRED', source_after=after,
                      artifact=pin, immutable_binary=str(inputs / 'cpu.exe'))
    except BaseException as error:
        result.update(failure_kind=type(error).__name__, failure=str(error))
        raise
    finally:
        write(out / 'result.json', result)


if __name__ == '__main__':
    main()
