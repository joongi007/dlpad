"""Preserve fixed-source Windows GNU CLI evidence, including every vendor file."""
import datetime
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys

root = pathlib.Path(__file__).resolve().parents[1]
out = root / 'docs' / 'validation'
logs = root / '.local-tools' / 'windows' / 'logs'


def snapshot():
    paths = {root / name for name in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml',
             'scripts/windows_native_probe.py', 'scripts/windows_cli_host.cs',
             'scripts/windows_winit_revalidate.py', '.local-tools/windows/windows-cli-host.exe')}
    paths.update(root.glob('crates/**/*.rs'))
    paths.update(root.glob('crates/**/Cargo.toml'))
    paths.update(path for path in (root / 'vendor').rglob('*') if path.is_file())
    return {str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(paths)}


baseline = snapshot()
manifest = {'recorded_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'scope': 'native Windows GNU CLI; GUI/IME/MSVC/macOS not verified',
            'baseline': baseline, 'operations': {}}
for operation in ('fmt', 'clippy', 'test', 'build'):
    print('Windows fixed snapshot operation=' + operation, flush=True)
    before = snapshot()
    result = subprocess.run([sys.executable, str(root / 'scripts/windows_native_probe.py'), operation], cwd=root)
    after = snapshot()
    for suffix in ('.log', '-inputs.json'):
        shutil.copyfile(logs / (operation + suffix), out / ('windows-winit-' + operation + suffix))
    inputs = json.loads((logs / (operation + '-inputs.json')).read_text())
    manifest['operations'][operation] = {'exit_code': result.returncode,
        'elapsed_seconds': inputs['elapsed_seconds'], 'argv': inputs['argv'],
        'same_baseline': baseline == before == after,
        'runner_unchanged': inputs['unchanged'],
        'log': 'windows-winit-' + operation + '.log',
        'inputs': 'windows-winit-' + operation + '-inputs.json'}
    manifest['final'] = after
    (out / 'windows-winit-final.json').write_text(json.dumps(manifest, indent=2) + '\n')
    if result.returncode or not (baseline == before == after) or not inputs['unchanged']:
        raise SystemExit(result.returncode or 3)
exe = root / '.local-tools/windows/target/release/dlpad.exe'
manifest['release_artifact'] = {'bytes': exe.stat().st_size,
                               'sha256': hashlib.sha256(exe.read_bytes()).hexdigest()}
(out / 'windows-winit-final.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(json.dumps({'same_inputs': True, 'input_count': len(baseline),
                 'operations': manifest['operations'], 'artifact': manifest['release_artifact']}, indent=2))
