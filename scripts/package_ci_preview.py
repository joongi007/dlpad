"""Package an already-built, same-run Windows CI artifact into unsigned preview ZIPs."""
import argparse
import hashlib
import re
from pathlib import Path
import stat
import zipfile

import ci_notice_graph
import ci_registry_source_binding as registry
import ci_whole_orchestrate
import ci_whole_receipt as receipt
import collect_ci_notices as notice
from windows_candidate_package import pe

TARGET = notice.TARGET
SYSTEM_DLLS = set('advapi32 bcrypt bcryptprimitives cfgmgr32 clbcatq combase comctl32 comdlg32 crypt32 cryptbase cryptnet cryptsp d2d1 d3d9 d3d11 d3d12 dcomp ddraw dbghelp dnsapi dsound dwmapi dwrite dxgi gdi32 gdi32full hid imagehlp imm32 iphlpapi kernel32 kernelbase mpr msacm32 msimg32 msvcrt ncrypt netapi32 normaliz ntdll ole32 oleacc oleaut32 opengl32 powrprof propsys psapi rpcrt4 secur32 setupapi shell32 shcore shlwapi sspicli user32 userenv usp10 ucrtbase uiautomationcore uxtheme version winhttp wininet winmm winspool wintrust wldap32 ws2_32 wtsapi32'.split())
MAX_ZIP = 256 * 1024 * 1024


def require(ok, message):
    notice.require(ok, message)


def pinned_file(path, expected, limit=128*1024*1024):
    require(receipt.pin(path, limit) == expected, 'original file pin mismatch: ' + str(path.name))


def identity(proof, commit, run_id):
    value = proof.get('identity')
    ci_whole_orchestrate.validate_identity(value)
    require(value['commit'] == commit and value['run_id'] == run_id and value['repository'] == 'joongi007/dlpad', 'same repository/SHA/run required')
    require(proof.get('target') == TARGET and proof.get('development_only') is True and proof.get('release_ready') is False and proof.get('signed_distribution') is False, 'unsigned CI identity required')
    return value


def runtime(path):
    result = pe(path)
    unknown = []
    for name in result['import_dlls']:
        lowered = name.lower()
        if not lowered.endswith('.dll') or (lowered[:-4] not in SYSTEM_DLLS and not lowered.startswith(('api-ms-win-', 'ext-ms-win-'))):
            unknown.append(name)
    require(not unknown, 'unbundled runtime dependency: ' + ', '.join(unknown))
    return result


def validate(root, inputs, notices, commit, run_id, version):
    require(re.fullmatch('[0-9a-f]{40}', commit) and re.fullmatch('[1-9][0-9]{0,19}', run_id), 'commit/run domain')
    require(re.fullmatch(r'(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})', version), 'numeric version required')
    source = notice.raw(root/'Cargo.toml').decode().split('[workspace.package]', 1)
    require(len(source) == 2, 'workspace package version required')
    section = source[1].split('\n[', 1)[0]
    versions = re.findall(r'(?m)^version\s*=\s*"([0-9.]+)"\s*(?:#.*)?$', section)
    require(versions == [version], 'tag/workspace version mismatch')
    whole_path = inputs/'ci-captures/whole.json'
    proof = notice.obj(whole_path)
    ci_identity = identity(proof, commit, run_id)
    require(proof.get('source_before') == proof.get('source_after') and proof['source_before'], 'whole source mismatch')
    for name, pin in proof['source_before'].items():
        pinned_file(receipt.path(root, name), pin, 16*1024*1024)
    for file, key in [('Cargo.lock', 'lock'), ('rust-toolchain.toml', 'toolchain')]:
        pinned_file(inputs/file, proof[key], 4*1024*1024)
    require(set(proof.get('operations', {})) == set(receipt.REQUIRED) and set(proof.get('capture_lifecycle_bindings', {})) == set(receipt.REQUIRED), 'seven successful captures required')
    for name, argv in receipt.REQUIRED.items():
        operation = proof['operations'][name]
        require(operation.get('argv') == argv and type(operation.get('exit_code')) is int and operation['exit_code'] == 0, 'exact successful operation required')
        record_path = inputs/'ci-captures'/ (name+'.json')
        binding = proof['capture_lifecycle_bindings'][name]
        pinned_file(record_path, binding['record'], 65536)
        record = notice.obj(record_path, 65536)
        require(record['name'] == name and record['argv'] == argv and type(record['exit_code']) is int and record['exit_code'] == 0, 'operation record mismatch')
        lifecycle, logs = ci_whole_orchestrate.lifecycle(inputs, 'ci-captures', name, record)
        require(lifecycle == binding['lifecycle'] and logs == binding['raw'], 'original lifecycle binding mismatch')
        for stream in ('stdout', 'stderr'):
            require(operation['logs'][stream] == {'path': record[stream], **logs[stream]}, 'original raw log binding mismatch')
    require(set(proof.get('artifacts', {})) == set(receipt.BINS), 'four CI binary bindings required')
    for name in receipt.BINS:
        pinned_file(inputs/'target/release'/name, proof['artifacts'][name])
    graph = notice.obj(inputs/'ci-graph-format.json')
    classified = ci_notice_graph.classify(notice.raw(inputs/'ci-app-tree.stdout.txt'), notice.raw(inputs/'ci-engine-tree.stdout.txt'), notice.raw(inputs/'ci-metadata.stdout.json'), notice.raw(inputs/'Cargo.lock'))
    require(graph == classified, 'original graph input mismatch')
    inventory = notice.obj(notices/'inventory.json')
    require(inventory.get('schema') == 1 and inventory.get('identity') == ci_identity and inventory.get('target') == TARGET, 'notice identity mismatch')
    require(inventory.get('whole') == notice.pin(notice.raw(whole_path)) and inventory.get('graph') == notice.pin(notice.raw(inputs/'ci-graph-format.json')) and inventory.get('lock') == proof['lock'], 'notice source pins mismatch')
    for flag, value in [('development_only', True), ('release_ready', False), ('legal_coverage_complete', False)]:
        require(inventory.get(flag) is value, 'notice readiness claim')
    rows = inventory.get('records')
    expected = {(p['name'], p['version'], p['source']) for p in graph['resolved_packages']}
    require(isinstance(rows, list) and len(rows) == len(expected) and {(p['name'], p['version'], p['source']) for p in rows} == expected, 'notice package coverage mismatch')
    files = inventory.get('files')
    require(isinstance(files, dict) and 0 < len(files) <= 4096, 'notice file count bound')
    notices_raw = bytearray()
    for name, pin in sorted(files.items()):
        data = notice.raw(receipt.path(notices, name))
        require(notice.pin(data) == pin, 'notice bytes mismatch')
        header = ('\n===== original-entry: '+name+'; sha256: '+pin['sha256']+' =====\n').encode()
        require(len(notices_raw) + len(header) + len(data) + 1 <= notice.MAX_DOC, 'combined notice byte bound')
        notices_raw.extend(header + data + b'\n')
    for row in rows:
        require(row.get('notices'), 'empty package notices')
        for entry in row['notices']:
            require(files.get(entry['entry']) == {k: entry[k] for k in ('sha256', 'bytes')}, 'notice record pin mismatch')
    cpu, installer = inputs/'target/release/dlpad-cpu.exe', inputs/'target/release/dlpad-install.exe'
    imports = {name: runtime(path) for name, path in [('dlpad-cpu.exe', cpu), ('dlpad-install.exe', installer)]}
    binding = {'schema': 1, 'identity': ci_identity, 'version': version, 'target': TARGET, 'development_only': True, 'release_ready': False, 'signed_distribution': False, 'complete_legal_coverage': False, 'whole': notice.pin(notice.raw(whole_path)), 'notice_inventory': notice.pin(notice.raw(notices/'inventory.json')), 'binaries': proof['artifacts'], 'mapping': {'dlpad-cpu.exe': 'payload/dlpad.exe', 'dlpad-install.exe': 'dlpad-install.exe'}, 'pe': imports}
    return cpu, installer, bytes(notices_raw), binding


def write_zip(path, entries):
    require(len(entries) <= 64, 'package member count')
    total = 0
    pins = {}
    normalized = set()
    for name in entries:
        receipt.relative_parts(name)
        require(name.casefold() not in normalized, 'duplicate/case-colliding package member')
        normalized.add(name.casefold())
    with path.open('xb') as stream, zipfile.ZipFile(stream, 'w', compression=zipfile.ZIP_STORED, allowZip64=False) as archive:
        for name, source in sorted(entries.items()):
            receipt.relative_parts(name)
            info = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = (stat.S_IFREG | (0o755 if name.endswith('.exe') else 0o644)) << 16
            digest = hashlib.sha256()
            count = 0
            with archive.open(info, 'w') as writer:
                if isinstance(source, bytes):
                    require(len(source) <= notice.MAX_DOC, 'document bound')
                    writer.write(source)
                    digest.update(source)
                    count = len(source)
                else:
                    receipt.no_reparse(source)
                    with source.open('rb') as reader:
                        while block := reader.read(65536):
                            count += len(block)
                            require(count <= 128*1024*1024, 'executable byte bound')
                            digest.update(block)
                            writer.write(block)
                total += count
                require(total <= MAX_ZIP, 'package expanded byte bound')
            pins[name] = {'sha256': digest.hexdigest(), 'bytes': count}
    with zipfile.ZipFile(path) as archive:
        require(set(archive.namelist()) == set(entries) and len(archive.infolist()) == len(entries), 'archive namespace mismatch')
        for name, expected in pins.items():
            with archive.open(name) as reader:
                count, digest = registry.hashed(reader, 128*1024*1024)
            require(expected == {'bytes': count, 'sha256': digest}, 'archive CRC/hash mismatch')
    return pins


def package(root, inputs, notices, commit, run_id, version, output):
    root, inputs, notices, output = [registry.plain(Path(p).absolute()) for p in (root, inputs, notices, output)]
    require(not output.exists() and output.parent.is_dir() and not output.is_relative_to(inputs), 'fresh output outside inputs required')
    cpu, installer, third_party, binding = validate(root, inputs, notices, commit, run_id, version)
    readme = b'dlpad unsigned Windows preview. Extract the complete ZIP. Installer: run dlpad-install.exe; portable: run dlpad.exe. App data and downloaded media are separate and are not removed by this package. External tools are optional and are not bundled. This is not an Authenticode-signed stable release. From v0.1.1, use Settings > App update for user-requested signed updates; v0.1.0 needs one manual install. No unattended update installation is enabled. Windows clean-machine/runtime and legal completeness are not claimed. See binding-record.txt.\n'
    small = {'LICENSE': notice.raw(root/'LICENSE'), 'THIRD-PARTY-NOTICES.txt': third_party, 'README.txt': readme, 'binding-record.txt': notice.encoded(binding)}
    entries = [{'name': 'dlpad.exe', **receipt.pin(cpu, 128*1024*1024)}] + [{'name': name, **notice.pin(data)} for name, data in sorted(small.items())]
    manifest = notice.encoded({'schema': 1, 'version': version, 'target': TARGET, 'entries': entries, 'development_only': True})
    require(len(manifest) <= 65536, 'bundle manifest byte bound')
    payload = {'dlpad.exe': cpu, **small, 'manifest.json': manifest}
    output.mkdir()
    prefix = 'dlpad-'+version+'-windows-x86_64-'
    packages = {}
    for kind, members in [('installer', {'dlpad-install.exe': installer, **{'payload/'+name: value for name, value in payload.items()}}), ('portable', payload)]:
        destination = output/(prefix+kind+'.zip')
        files = write_zip(destination, members)
        require(files['payload/dlpad.exe' if kind == 'installer' else 'dlpad.exe'] == binding['binaries']['dlpad-cpu.exe'], 'packaged CPU bytes changed')
        if kind == 'installer':
            require(files['dlpad-install.exe'] == binding['binaries']['dlpad-install.exe'], 'packaged installer bytes changed')
        packages[destination.name] = receipt.pin(destination, MAX_ZIP+1024*1024)
    # Recheck all original source/artifact/notice pins after writing both ZIPs.
    require(validate(root, inputs, notices, commit, run_id, version)[3] == binding, 'original package inputs changed')
    checksums = ''.join(pin['sha256']+'  '+name+'\n' for name, pin in sorted(packages.items())).encode()
    for name, data in [('SHA256SUMS', checksums), ('release-binding.json', notice.encoded({**binding, 'packages': packages})), ('RELEASE-NOTES.md', ('# dlpad '+version+' Windows preview\n\nUnsigned development preview. Includes installer and portable ZIPs, original dependency notices, checksums, and same-run CI bindings. Not an Authenticode-signed stable release. App updates are supported from v0.1.1 when signed release metadata is supplied, through Settings > App update. v0.1.0 requires one manual install; unattended update installation is not enabled.\n').encode())]:
        with (output/name).open('xb') as stream:
            stream.write(data)
    return packages


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('root', 'inputs', 'notices', 'output'):
        parser.add_argument('--'+name, type=Path, required=True)
    for name in ('commit', 'run-id', 'version'):
        parser.add_argument('--'+name, required=True)
    package(**vars(parser.parse_args()))


if __name__ == '__main__':
    main()
