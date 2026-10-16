"""Run installed native Windows GNU Rust CLI checks from WSL, without a GUI."""
import argparse
import hashlib
import json
import os
import pathlib
import subprocess
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('operation', choices=('versions', 'fmt', 'check', 'clippy', 'test', 'build'))
parser.add_argument('--seconds', type=int, default=1800)
parser.add_argument('--features', choices=('cpu', 'alternative'))
options = parser.parse_args()
if options.features and options.operation in ('fmt', 'versions'):
    parser.error('--features requires check/clippy/test/build')
root = pathlib.Path(__file__).resolve().parents[1]
tools = root / '.local-tools' / 'windows'


def windows(path):
    return subprocess.check_output(['wslpath', '-w', str(path)], text=True).strip()


def snapshot():
    paths = {root / 'Cargo.toml', root / 'Cargo.lock', root / 'rust-toolchain.toml'}
    paths.update((root / 'crates').rglob('*.rs'))
    paths.update((root / 'crates').rglob('Cargo.toml'))
    paths.update((root / 'vendor').rglob('*.rs'))
    paths.update((root / 'vendor').rglob('Cargo.toml'))
    return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(paths) if p.is_file()}


def run(executable, arguments, environment):
    started = time.monotonic()
    inputs_before = snapshot()
    log_dir = tools / 'logs'
    log_dir.mkdir(exist_ok=True)
    logfile = log_dir / (options.operation + '.log')
    # WSL Win32 interop inherits redirected pipes; no console or GUI is created.
    process = subprocess.Popen([str(tools / "windows-cli-host.exe"), str(options.seconds), windows(executable), *arguments], cwd=root,
                               env=environment, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.STDOUT)
    try:
        with logfile.open('wb') as output:
            for line in iter(process.stdout.readline, b''):
                output.write(line)
                output.flush()
                print(line.decode('utf-8', errors='replace').rstrip(), flush=True)
                # Native host enforces the deadline and terminates only its Job Object.
        result = process.wait(timeout=10)
    except BaseException:
        # Cancel through the owned native host, never guessed Windows PIDs.
        try:
            process.stdin.write(b'C')
            process.stdin.flush()
        except (BrokenPipeError, OSError):
            pass
        process.stdin.close()
        process.wait(timeout=8)
        raise
    finally:
        if process.stdin and not process.stdin.closed:
            process.stdin.close()
    inputs_after = snapshot()
    (log_dir / (options.operation + '-inputs.json')).write_text(json.dumps({
        'before': inputs_before, 'after': inputs_after,
        'unchanged': inputs_before == inputs_after, 'exit_code': result,
        'elapsed_seconds': time.monotonic() - started,
        'argv': [windows(executable), *arguments],
        'rustflags': configure['RUSTFLAGS'],
    }, indent=2) + '\n')
    print(f'native-windows operation={options.operation} exit={result} elapsed={time.monotonic()-started:.2f}s', flush=True)
    if result:
        raise SystemExit(result)
    if inputs_before != inputs_after:
        print("Source changed during this operation; repeat for a stable final snapshot", flush=True)
        raise SystemExit(3)


environment = os.environ.copy()
configure = {
    'DLPAD_NATIVE_CONTROL_STDIN': '1',
    'DLPAD_NATIVE_TOOL_PATH': windows(tools / 'rust' / 'bin') + ';' + windows(tools / 'llvm-mingw' / 'bin'),
    'CARGO': windows(tools / 'rust' / 'bin' / 'cargo.exe'),
    'RUSTC': windows(tools / 'rust' / 'bin' / 'rustc.exe'),
    'RUSTFMT': windows(tools / 'rust' / 'bin' / 'rustfmt.exe'),
    'RUSTDOC': windows(tools / 'rust' / 'bin' / 'rustdoc.exe'),
    'RUSTFLAGS': '-C link-self-contained=yes',
    'CARGO_HOME': windows(tools / 'cargo-home'),
    'CARGO_TARGET_DIR': windows(tools / 'target'),
    'CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER': windows(tools / 'llvm-mingw' / 'bin' / 'x86_64-w64-mingw32-gcc.exe'),
    'CC_x86_64_pc_windows_gnu': windows(tools / 'llvm-mingw' / 'bin' / 'x86_64-w64-mingw32-gcc.exe'),
    'AR_x86_64_pc_windows_gnu': windows(tools / 'llvm-mingw' / 'bin' / 'llvm-ar.exe'),
    'TMP': windows(tools / 'tmp'),
    'TEMP': windows(tools / 'tmp'),
}
(tools / 'tmp').mkdir(exist_ok=True)
environment.update(configure)
environment['WSLENV'] = environment.get('WSLENV', '') + ':' + ':'.join(configure)
bin_dir = tools / 'rust' / 'bin'
if options.operation == 'versions':
    run(bin_dir / 'rustc.exe', ['--version', '--verbose'], environment)
    run(bin_dir / 'cargo.exe', ['--version'], environment)
else:
    commands = {
        'fmt': (bin_dir / 'cargo-fmt.exe', ['fmt', '--all', '--', '--check']),
        'check': (bin_dir / 'cargo.exe', ['check', '--workspace', '--all-targets', '--locked']),
        'clippy': (bin_dir / 'cargo-clippy.exe', ['clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings']),
        'test': (bin_dir / 'cargo.exe', ['test', '--workspace', '--locked']),
        'build': (bin_dir / 'cargo.exe', ['build', '--workspace', '--release', '--locked']),
    }
    executable, arguments = commands[options.operation]
    if options.features:
        insertion = arguments.index("--") if "--" in arguments else len(arguments)
        arguments[insertion:insertion] = ["--features", options.features]
    run(executable, arguments, environment)
