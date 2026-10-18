"""Sign release metadata locally using the Windows-user protected operational key.

Run in WSL/Linux. The private key crosses only a local stdin/stdout pipe, is held
temporarily in a 0700 /tmp directory (0600 file), and is never uploaded. Only the
three public metadata files are output. Hosted signing remains separately gated.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile

import sign_app_update as signer


def windows_path(path):
    path = Path(path).absolute()
    parts = path.parts
    if len(parts) < 4 or parts[:2] != ('/', 'mnt') or parts[2] not in 'abcdefghijklmnopqrstuvwxyz':
        raise ValueError('a local WSL Windows-drive path is required')
    return parts[2].upper() + ':\\' + '\\'.join(parts[3:])


def sign(protected_key, windows_python, python_sha256, **options):
    signer.require_supported()
    executable = signer.plain(windows_python)
    proof, _ = signer.hash_file(executable, 128 * 1024 * 1024)
    signer.require(proof['sha256'] == python_sha256, 'Windows Python pin mismatch')
    encrypted = signer.plain(protected_key)
    signer.hash_file(encrypted, 65536)
    bridge = Path(__file__).absolute().with_name('windows_update_key.py')
    trusted = json.loads((Path(__file__).absolute().parent.parent /
                         'crates/engine/src/app_update/release-public-key.json').read_text())
    signer.require(options['key_id'] == trusted['key_id'], 'compiled release key ID mismatch')
    process = subprocess.run([str(executable), windows_path(bridge), 'unprotect',
                              windows_path(encrypted)], stdin=subprocess.DEVNULL,
                             capture_output=True, timeout=30, check=False)
    signer.require(process.returncode == 0 and 0 < len(process.stdout) <= signer.MAX_KEY,
                   'local key unprotection failed; output omitted')
    # Only the locally pinned stdlib bridge runs above. It creates no child processes.
    with tempfile.TemporaryDirectory(prefix='dlpad-release-key-', dir='/tmp') as directory:
        os.chmod(directory, 0o700)
        key = Path(directory) / 'private.pem'
        with os.fdopen(os.open(key, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600), 'wb') as stream:
            stream.write(process.stdout)
        del process
        public = signer.run_openssl(['pkey', '-in', str(key), '-passin', 'pass:', '-pubout', '-outform', 'DER'])
        signer.require(len(public) == 44 and public.startswith(signer.PUBLIC_PREFIX)
                       and public[12:].hex() == trusted['public_key_hex'],
                       'local private key does not match the compiled trust root')
        return signer.sign(private_key=key, **options)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('protected-key', 'windows-python', 'package', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    for name in ('python-sha256', 'version', 'key-id'):
        parser.add_argument('--' + name, required=True)
    parser.add_argument('--channel', choices=['preview'], default='preview')
    for name in ('sequence', 'issued-at', 'expires-at'):
        parser.add_argument('--' + name, required=True, type=int)
    public = sign(**vars(parser.parse_args()))
    print(json.dumps(public, sort_keys=True))


if __name__ == '__main__':
    main()
