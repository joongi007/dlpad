#!/usr/bin/env python3
"""Unsigned development payload only. Never overwrites or removes an output."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import stat
import zipfile

MAX_ZIP = 256 * 1024 * 1024
MAX_DOC = 16 * 1024 * 1024
MAX_EXE = 128 * 1024 * 1024
TARGET = 'x86_64-pc-windows-msvc'
BINS = {'dlpad-cpu.exe', 'dlpad-install.exe', 'dlpad-local-install.exe'}

def require(value, message):
    if not value:
        raise ValueError(message)

def sha_stream(stream, limit):
    h = hashlib.sha256()
    total = 0
    while chunk := stream.read(65536):
        total += len(chunk)
        require(total <= limit, 'stream byte bound')
        h.update(chunk)
    return h.hexdigest(), total

def plain_file(path):
    path = Path(path)
    for part in [path, *path.parents]:
        require(not part.is_symlink(), 'symlink input/path')
    require(stat.S_ISREG(path.stat().st_mode), 'regular source required')
    return path

def file_sha(path, limit):
    with plain_file(path).open('rb') as stream:
        return sha_stream(stream, limit)

def read_small(z, name, limit=MAX_DOC):
    require(z.getinfo(name).file_size <= limit, 'document byte bound')
    with z.open(name) as stream:
        data = stream.read(limit + 1)
        require(len(data) <= limit and not stream.read(1), 'document stream byte bound')
    return data

def json_exact(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, 'duplicate JSON key')
            result[key] = value
        return result
    return json.loads(raw, object_pairs_hook=pairs)

def encoded(value):
    return (json.dumps(value, indent=2, sort_keys=True) + '\n').encode()

def validate_zip(z):
    infos = z.infolist()
    require(1 <= len(infos) <= 4096, 'archive member count')
    names = set()
    total = 0
    observed = {}
    for info in infos:
        name = info.filename
        require(info.orig_filename == name, 'unsafe original ZIP member path')
        parts = name.split('/')
        require(name and name.isascii() and not name.startswith('/') and '\\' not in name
                and ':' not in name and not any(ord(c) < 32 for c in name)
                and all(p and p not in {'.', '..'} and not p.endswith(('.', ' ')) for p in parts),
                'unsafe ZIP member path')
        require(name.casefold() not in names, 'duplicate ZIP member')
        names.add(name.casefold())
        mode = info.external_attr >> 16
        require(stat.S_IFMT(mode) in {0, stat.S_IFREG} and not info.is_dir(), 'ZIP non-regular member')
        require(not info.flag_bits & 1, 'encrypted ZIP member')
        limit = MAX_EXE if name in BINS else MAX_DOC
        require(0 <= info.file_size <= limit, 'member byte bound')
        total += info.file_size
        require(total <= MAX_ZIP, 'archive expanded byte bound')
        with z.open(info) as stream:
            digest, size = sha_stream(stream, limit)  # ZipExtFile checks original CRC on EOF.
        require(size == info.file_size, 'ZIP declaration byte mismatch')
        observed[name] = {'sha256': digest, 'bytes': size}
    return observed

def generate(candidate, output, candidate_sha256, license_path, license_sha256):
    require(re.fullmatch('[0-9a-f]{64}', candidate_sha256) and re.fullmatch('[0-9a-f]{64}', license_sha256), 'lowercase SHA256 pins required')
    candidate = plain_file(candidate)
    license_path = plain_file(license_path)
    output = Path(output)
    require(not output.exists() and not output.is_symlink(), 'fresh output required')
    for parent in output.parents:
        require(not parent.is_symlink(), 'symlink output parent')
    require(output.parent.is_dir(), 'existing output parent required')
    require(file_sha(candidate, MAX_ZIP)[0] == candidate_sha256, 'candidate ZIP pin mismatch')
    license_digest, license_size = file_sha(license_path, MAX_DOC)
    require(license_digest == license_sha256 and license_size > 0, 'project LICENSE pin mismatch')
    with license_path.open('rb') as stream:
        license_raw = stream.read(MAX_DOC + 1)
        require(len(license_raw) <= MAX_DOC and not stream.read(1), 'project LICENSE stream bound')
    require(hashlib.sha256(license_raw).hexdigest() == license_sha256 and len(license_raw) == license_size, 'project LICENSE changed')
    with candidate.open('rb') as source, zipfile.ZipFile(source) as z:
        observed = validate_zip(z)
        manifest = json_exact(read_small(z, 'candidate-manifest.json', 1024 * 1024))
        require(manifest.get('schema') == 1 and manifest.get('target') == TARGET
                and manifest.get('development_only') is True and manifest.get('development_candidate') is True
                and manifest.get('release_ready') is False and manifest.get('legal_coverage_complete') is False,
                'development candidate readiness required')
        require(manifest.get('entrypoint') == 'dlpad-cpu.exe', 'explicit CPU entrypoint required')
        binaries = manifest.get('binaries')
        require(isinstance(binaries, dict) and {'dlpad-cpu.exe', 'dlpad-install.exe'} <= set(binaries)
                and set(binaries) <= BINS, 'exact candidate binaries required')
        require({name for name in observed if name.lower().endswith('.exe')} == set(binaries), 'unbound executable member')
        for name, proof in binaries.items():
            require(isinstance(proof, dict) and type(proof.get('bytes')) is int
                    and 0 < proof['bytes'] <= MAX_EXE and isinstance(proof.get('sha256'), str)
                    and re.fullmatch('[0-9a-f]{64}', proof['sha256']), 'binary pin domain')
            require(proof == observed.get(name), 'executable bytes/SHA mismatch')
        native = manifest.get('native_installer', {})
        require(native.get('entry') == 'dlpad-install.exe' and native.get('development_only') is True
                and native.get('installer_complete') is False
                and native.get('sha256') == binaries['dlpad-install.exe']['sha256']
                and native.get('bytes') == binaries['dlpad-install.exe']['bytes'], 'installer candidate binding')
        for key, name in [('whole_proof_sha256', 'build-evidence/whole-proof-sha256.txt'), ('graph_proof_sha256', 'build-evidence/graph-proof-sha256.txt')]:
            require(re.fullmatch('[0-9a-f]{64}', manifest.get(key, '')) is not None, 'proof SHA pin missing')
            require(read_small(z, name, 256).decode().strip() == manifest[key], 'proof member pin mismatch')
        require(observed['build-evidence/rust-toolchain.toml']['sha256'] == manifest.get('toolchain_file_sha256'), 'toolchain pin mismatch')
        inventory = json_exact(read_small(z, 'license-inventory.json'))
        require(inventory.get('complete') is False and inventory.get('missing') == [], 'selected notice inventory required')
        notices = {}
        for record in inventory['records']:
            for notice in record['notices']:
                name = notice['entry']
                require(name.startswith('licenses/') and name in observed
                        and observed[name]['sha256'] == notice['sha256'], 'original notice pin mismatch')
                notices[name] = observed[name]
        require(notices and set(notices) == {n for n in observed if n.startswith('licenses/')}, 'exact inventory notice coverage')
        chunks = []
        notice_size = 0
        for name in sorted(notices):
            proof = notices[name]
            raw = read_small(z, name)
            header = f'\n===== original-entry: {name}; bytes: {proof["bytes"]}; sha256: {proof["sha256"]} =====\n'.encode()
            chunks.extend([header, raw, b'\n===== end-original-entry =====\n'])
            notice_size += len(header) + len(raw) + len(chunks[-1])
            require(notice_size <= MAX_DOC, 'concatenated notice byte bound')
        notices_raw = b''.join(chunks)
        binding = {'schema': 1, 'development_only': True, 'release_ready': False,
                   'complete_legal_coverage': False, 'signed_distribution': False,
                   'candidate_zip_sha256': candidate_sha256,
                   'candidate_manifest_sha256': observed['candidate-manifest.json']['sha256'],
                   'mapping': {'dlpad-cpu.exe': 'payload/dlpad.exe', 'dlpad-install.exe': 'dlpad-install.exe'},
                   'binaries': {name: binaries[name] for name in ['dlpad-cpu.exe', 'dlpad-install.exe']},
                   'project_license': {'source_name': license_path.name, 'sha256': license_sha256, 'bytes': license_size},
                   'notices': notices}
        readme = b'UNSIGNED DEVELOPMENT INSTALL PAYLOAD ONLY. Not a signed release or updater. complete_legal_coverage=false. CPU bytes are explicitly mapped from dlpad-cpu.exe to payload/dlpad.exe. Original notice bytes are preserved between origin/SHA headers; complete legal coverage is unverified. Extract the entire ZIP to a new folder and run dlpad-install.exe without arguments. Choose an existing parent folder or enter one new installation folder. The separate installation journal is derived automatically; do not enter a second path. Existing installation folders are never overwritten. Optional external tools are unchecked by default; their failure does not remove the app. Cancellation keeps partial files for inspection. External tools, data, media and credentials are not included. See binding-record.txt.\n'
        small = {'LICENSE': license_raw, 'THIRD-PARTY-NOTICES.txt': notices_raw, 'README.txt': readme, 'binding-record.txt': encoded(binding)}
        entries = [{'name': 'dlpad.exe', **binaries['dlpad-cpu.exe']}]
        entries += [{'name': name, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()} for name, raw in sorted(small.items())]
        bundle = {'schema': 1, 'version': '0.1.0', 'target': TARGET, 'entries': entries, 'development_only': True}
        bundle_raw = encoded(bundle)
        require(len(entries) <= 64 and len(bundle_raw) <= 65536 and sum(e['bytes'] for e in entries) <= MAX_ZIP, 'BundleManifest bound')
        require(file_sha(candidate, MAX_ZIP)[0] == candidate_sha256, 'candidate source changed before publication')
        output.mkdir()  # Atomic fresh name; failures preserve all partial output for inspection.
        payload = output / 'payload'
        payload.mkdir()
        for source_name, destination in [('dlpad-install.exe', output / 'dlpad-install.exe'), ('dlpad-cpu.exe', payload / 'dlpad.exe')]:
            h = hashlib.sha256()
            count = 0
            with z.open(source_name) as reader, destination.open('xb') as writer:
                while chunk := reader.read(65536):
                    count += len(chunk)
                    require(count <= MAX_EXE, 'binary stream bound')
                    h.update(chunk)
                    writer.write(chunk)
            require({'bytes': count, 'sha256': h.hexdigest()} == binaries[source_name], 'source binary changed during copy')
        for name, raw in small.items():
            with (payload / name).open('xb') as stream:
                stream.write(raw)
        require(file_sha(candidate, MAX_ZIP)[0] == candidate_sha256, 'candidate changed during publication')
        # Manifest last: partial output is never advertised as a complete payload.
        with (payload / 'manifest.json').open('xb') as stream:
            stream.write(bundle_raw)
    return bundle

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--candidate', type=Path, required=True)
    parser.add_argument('--candidate-sha256', required=True)
    parser.add_argument('--license', type=Path, required=True, dest='license_path')
    parser.add_argument('--license-sha256', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    generate(**vars(args))

if __name__ == '__main__':
    main()
