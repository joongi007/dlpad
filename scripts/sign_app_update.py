"""Sign preview updater metadata with an existing local Ed25519 key on Linux.

This development signing tool does not generate operating keys, upload secrets,
install software, or confer Windows Authenticode/publisher trust.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import stat
import subprocess
import sys
import tempfile
import threading
import time
import zipfile

MAX_KEY = 16 * 1024
MAX_PACKAGE = 256 * 1024 * 1024
MAX_METADATA = 64 * 1024
MAX_VALIDITY = 31 * 24 * 60 * 60
U64_MAX = (1 << 64) - 1
PUBLIC_PREFIX = bytes.fromhex('302a300506032b6570032100')
ZIP_FILES = {'dlpad-install.exe', 'payload/dlpad.exe', 'payload/LICENSE',
             'payload/THIRD-PARTY-NOTICES.txt', 'payload/README.txt',
             'payload/binding-record.txt', 'payload/manifest.json'}


def require(ok, message):
    if not ok:
        raise ValueError(message)


def plain(path):
    path = Path(path).absolute()
    require('..' not in path.parts, 'normalized path required')
    for part in [path, *path.parents]:
        try:
            info = part.lstat()
        except FileNotFoundError:
            continue
        require(not stat.S_ISLNK(info.st_mode) and not getattr(info, 'st_file_attributes', 0) & 0x400,
                'symlink/reparse path rejected')
    return path


def state(info):
    return (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns)


def hash_file(path, limit):
    path = plain(path)
    before = path.stat()
    require(stat.S_ISREG(before.st_mode) and 0 < before.st_size <= limit, 'regular bounded input required')
    digest = hashlib.sha256()
    count = 0
    with path.open('rb') as stream:
        require(state(os.fstat(stream.fileno())) == state(before), 'input identity changed')
        while chunk := stream.read(65536):
            count += len(chunk)
            require(count <= limit, 'input stream byte bound')
            digest.update(chunk)
        require(state(os.fstat(stream.fileno())) == state(before), 'input changed while reading')
    require(count == before.st_size and state(plain(path).stat()) == state(before), 'input changed after reading')
    return {'bytes': count, 'sha256': digest.hexdigest()}, state(before)


def key_pin(path):
    path = plain(path)
    info = path.stat()
    require(stat.S_ISREG(info.st_mode) and 0 < info.st_size <= MAX_KEY, 'private key size/type rejected')
    if os.name == 'posix':
        require(info.st_mode & 0o077 == 0, 'private key group/world permissions rejected')
        require(info.st_uid == os.getuid(), 'private key owner rejected')
    return hash_file(path, MAX_KEY)


def require_supported():
    require(sys.platform.startswith('linux') and os.name == 'posix'
            and all(hasattr(os, name) for name in ('waitid', 'WNOWAIT', 'WEXITED', 'WNOHANG', 'killpg'))
            and signal.getsignal(signal.SIGCHLD) == signal.SIG_DFL,
            'Unsupported platform: signing requires Linux owned process groups and waitid(WNOWAIT)')


def run_openssl(args, *, timeout=10, output_limit=4096):
    """Bounded Linux CLI with original unreaped leader anchoring its private PGID.

    OpenSSL is trusted not to detach children into another session. This narrow
    development-tool contract is not a general sandbox or product guardian.
    """
    require_supported()
    environment = dict(os.environ, OPENSSL_CONF=os.devnull)
    process = subprocess.Popen(['openssl', *args], stdin=subprocess.DEVNULL,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               shell=False, env=environment, start_new_session=True)
    buffers = [bytearray(), bytearray()]
    failed = threading.Event()
    group_closed = False

    def close_group():
        nonlocal group_closed
        if group_closed:
            return
        # Neither poll nor wait has reaped our leader. Its PID/PGID cannot have
        # been recycled, even after natural exit; WNOWAIT retains the zombie.
        # Refuse a numeric group operation if an outside reaper invalidated
        # the standalone tool's ownership contract. Never guess from a PID.
        os.waitid(os.P_PID, process.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT)
        while True:
            try:
                os.killpg(process.pid, signal.SIGKILL)
                break
            except InterruptedError:
                continue
            except ProcessLookupError:
                break  # No group remains, while the original leader is retained.
        group_closed = True

    def pump(stream, dest):
        try:
            while chunk := stream.read(1024):
                if len(dest) + len(chunk) > output_limit:
                    failed.set()
                    break
                dest.extend(chunk)
        except OSError:
            failed.set()
        finally:
            stream.close()

    readers = [threading.Thread(target=pump, args=(stream, dest), daemon=True)
               for stream, dest in zip((process.stdout, process.stderr), buffers)]
    try:
        for reader in readers:
            reader.start()
        deadline = time.monotonic() + timeout
        terminal = None
        while terminal is None:
            require(not failed.is_set(), 'OpenSSL output bound exceeded')
            require(time.monotonic() < deadline, 'OpenSSL operation timed out')
            terminal = os.waitid(os.P_PID, process.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT)
            if terminal is None:
                time.sleep(0.01)
        # End any descendants before draining. A successful leader may leave a
        # pipe-inheriting child; never reap first then address a reusable PGID.
        close_group()
        for reader in readers:
            reader.join(timeout=1)
        require(not failed.is_set() and all(not reader.is_alive() for reader in readers), 'OpenSSL reader/output failure')
        require(terminal.si_code == os.CLD_EXITED and terminal.si_status == 0, 'OpenSSL operation failed')
        return bytes(buffers[0])
    finally:
        # Also runs on timeout, output overflow, reader start failure, or error.
        # The only leader reap happens after the original group is terminated.
        close_group()
        process.wait(timeout=3)
        for reader in readers:
            if reader.ident is not None:
                reader.join(timeout=1)


def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON field')
        result[key] = value
    return result


def encoded(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True) + '\n').encode('ascii')


def preview_package(path, version):
    require(path.name == f'dlpad-{version}-windows-x86_64-installer.zip', 'canonical installer filename required')
    proof, identity = hash_file(path, MAX_PACKAGE)
    with zipfile.ZipFile(path) as archive:
        infos = archive.infolist()
        require(len(infos) == len(ZIP_FILES) and {i.filename for i in infos} == ZIP_FILES, 'exact installer ZIP namespace required')
        for info in infos:
            require(info.orig_filename == info.filename and not info.is_dir() and not info.flag_bits & 1
                    and stat.S_IFMT(info.external_attr >> 16) in (0, stat.S_IFREG), 'regular unencrypted ZIP member required')
            require(0 <= info.file_size <= (128*1024*1024 if info.filename.endswith('.exe') else 16*1024*1024), 'ZIP member size bound')
        require(sum(i.file_size for i in infos) <= MAX_PACKAGE, 'ZIP expanded byte bound')

        def document(name):
            require(archive.getinfo(name).file_size <= MAX_METADATA, 'metadata document bound')
            with archive.open(name) as stream:
                data = stream.read(MAX_METADATA + 1)
            require(len(data) <= MAX_METADATA, 'metadata document stream bound')
            return json.loads(data, object_pairs_hook=unique)
        manifest = document('payload/manifest.json')
        require(isinstance(manifest, dict) and set(manifest) == {'schema', 'version', 'target', 'entries', 'development_only'}, 'exact bundle manifest fields required')
        require(type(manifest.get('schema')) is int and manifest['schema'] == 1 and manifest.get('version') == version
                and manifest.get('target') == 'x86_64-pc-windows-msvc'
                and manifest.get('development_only') is True, 'preview bundle identity required')
        binding = document('payload/binding-record.txt')
        require(isinstance(binding, dict), 'preview binding object required')
        require(type(binding.get('schema')) is int and binding['schema'] == 1 and binding.get('version') == version and binding.get('development_only') is True
                and binding.get('release_ready') is False and binding.get('signed_distribution') is False
                and binding.get('complete_legal_coverage') is False, 'unsigned preview binding required')
        entries = manifest.get('entries')
        require(isinstance(entries, list) and len(entries) == 5, 'five manifest entries required')
        expected = {}
        for entry in entries:
            require(isinstance(entry, dict) and set(entry) == {'name', 'bytes', 'sha256'}, 'manifest entry fields')
            name = 'payload/' + entry['name'] if isinstance(entry.get('name'), str) else ''
            require(name in ZIP_FILES-{'dlpad-install.exe', 'payload/manifest.json'} and name not in expected
                    and type(entry['bytes']) is int and 0 < entry['bytes'] <= 128*1024*1024
                    and isinstance(entry['sha256'], str) and re.fullmatch('[0-9a-f]{64}', entry['sha256']), 'manifest entry pin domain')
            expected[name] = {key: entry[key] for key in ('bytes', 'sha256')}
        binaries = binding.get('binaries', {})
        require(isinstance(binaries, dict) and binaries.get('dlpad-cpu.exe') == expected['payload/dlpad.exe'], 'CPU binding mismatch')
        installer = binaries.get('dlpad-install.exe')
        require(isinstance(installer, dict) and set(installer) == {'bytes', 'sha256'}
                and type(installer['bytes']) is int and 0 < installer['bytes'] <= 128*1024*1024
                and isinstance(installer['sha256'], str) and re.fullmatch('[0-9a-f]{64}', installer['sha256']), 'installer binding pin domain')
        expected['dlpad-install.exe'] = installer
        for name, expected_pin in expected.items():
            digest = hashlib.sha256()
            count = 0
            with archive.open(name) as stream:
                while chunk := stream.read(65536):
                    count += len(chunk)
                    require(count <= expected_pin['bytes'], 'ZIP original member grew')
                    digest.update(chunk)
            require({'bytes': count, 'sha256': digest.hexdigest()} == expected_pin, 'ZIP original member pin mismatch')
    require(hash_file(path, MAX_PACKAGE) == (proof, identity), 'package changed during validation')
    return proof, identity


def metadata(package, version, sequence, channel, key_id, issued_at, expires_at, now=None):
    require(isinstance(version, str) and len(version) <= 32 and re.fullmatch(r'(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})', version), 'invalid numeric version')
    require(type(sequence) is int and 1 <= sequence <= U64_MAX, 'sequence must be a positive u64')
    require(channel == 'preview', 'only preview signing is enabled')
    require(isinstance(key_id, str) and re.fullmatch(r'[A-Za-z0-9_-]{1,64}', key_id), 'invalid key ID')
    require(type(issued_at) is int and type(expires_at) is int and 0 <= issued_at < expires_at <= U64_MAX
            and expires_at-issued_at <= MAX_VALIDITY, 'invalid validity interval')
    now = int(time.time()) if now is None else now
    require(issued_at <= now < expires_at, 'metadata must be currently valid')
    proof, identity = preview_package(package, version)
    asset = f'dlpad-{version}-windows-x86_64-installer.zip'
    result = {'schema': 1, 'domain': 'dlpad.app-update.v1', 'key_id': key_id,
              'sequence': sequence, 'version': version, 'channel': channel,
              'target': 'x86_64-pc-windows-msvc', 'repository': 'joongi007/dlpad',
              'asset_name': asset,
              'asset_url': f'https://github.com/joongi007/dlpad/releases/download/v{version}/{asset}',
              'asset_bytes': proof['bytes'], 'asset_sha256': proof['sha256'],
              'issued_at_unix': issued_at, 'expires_at_unix': expires_at}
    payload = encoded(result)
    require(0 < len(payload) <= MAX_METADATA, 'metadata byte bound')
    return payload, proof, identity


def sign(package, version, sequence, channel, key_id, private_key, output, issued_at, expires_at):
    require_supported()
    package, private_key, output = [plain(path) for path in (package, private_key, output)]
    require(not output.exists() and output.parent.is_dir(), 'fresh output directory required')
    original_key = key_pin(private_key)
    payload, proof, package_identity = metadata(package, version, sequence, channel, key_id, issued_at, expires_at)
    # Only public metadata and public key/signature are copied into this scratch directory.
    with tempfile.TemporaryDirectory(prefix='dlpad-update-sign-') as scratch:
        directory = Path(scratch)
        if os.name == 'posix':
            directory.chmod(0o700)
        message = directory/'metadata.json'
        message.write_bytes(payload)
        public_der = run_openssl(['pkey', '-in', str(private_key), '-passin', 'pass:', '-pubout', '-outform', 'DER'])
        require(len(public_der) == 44 and public_der.startswith(PUBLIC_PREFIX), 'an Ed25519 private key is required')
        require(key_pin(private_key) == original_key, 'private key changed during public key extraction')
        signature = run_openssl(['pkeyutl', '-sign', '-rawin', '-inkey', str(private_key), '-passin', 'pass:', '-in', str(message)])
        require(len(signature) == 64, 'exact Ed25519 signature required')
        public_file = directory/'public.der'
        signature_file = directory/'signature.bin'
        public_file.write_bytes(public_der)
        signature_file.write_bytes(signature)
        run_openssl(['pkeyutl', '-verify', '-pubin', '-keyform', 'DER', '-inkey', str(public_file), '-rawin', '-in', str(message), '-sigfile', str(signature_file)])
        require(message.read_bytes() == payload and key_pin(private_key) == original_key, 'signing inputs changed')
    require(hash_file(package, MAX_PACKAGE) == (proof, package_identity), 'original package changed before publication')
    public = {'schema': 1, 'algorithm': 'Ed25519', 'key_id': key_id, 'public_key_hex': public_der[len(PUBLIC_PREFIX):].hex()}
    # Public key is an out-of-band bootstrap aid, never candidate-provided trust.
    output.mkdir()
    for name, data in [('update-windows-x86_64.json', payload),
                       ('update-windows-x86_64.json.sig', signature.hex().encode('ascii')),
                       ('update-public-key.json', encoded(public))]:
        with (output/name).open('xb') as stream:
            stream.write(data)
    return public


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('package', 'private-key', 'output'):
        parser.add_argument('--'+name, type=Path, required=True)
    for name in ('version', 'key-id'):
        parser.add_argument('--'+name, required=True)
    parser.add_argument('--channel', choices=['preview'], default='preview')
    for name in ('sequence', 'issued-at', 'expires-at'):
        parser.add_argument('--'+name, type=int, required=True)
    result = sign(**vars(parser.parse_args()))
    print(encoded(result).decode('ascii'), end='')


if __name__ == '__main__':
    main()
