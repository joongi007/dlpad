#!/usr/bin/env python3
"""Create-only isolated source commit/bundle. Never touches source .git or network."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

PROJECT = Path('/mnt/d/project/dlpad')
MAX_FILE = 256 * 1024 * 1024
MAX_RECEIPT = 16 * 1024 * 1024
MAX_FILES = 20000


def require(value, message):
    if not value:
        raise ValueError(message)


def plain(path):
    path = Path(path)
    require(path.is_absolute() and '..' not in path.parts, 'plain absolute path')
    require(not any(p.is_symlink() for p in (path, *path.parents)), 'symlink path')
    return path


def fresh_root(path, source):
    path, source = plain(path), plain(source)
    require(path.is_relative_to(Path('/tmp')) and path != Path('/tmp'), 'isolated /tmp leaf only')
    require(not path.exists(), 'fresh output required')
    require(path.parent.is_dir(), 'existing output parent')
    require(not path.is_relative_to(source) and not source.is_relative_to(path), 'source/output separation')
    return path


def relative(value):
    require(isinstance(value, str) and value and '\\' not in value and '\x00' not in value, 'relative path domain')
    p = Path(value)
    require(not p.is_absolute() and all(v not in ('', '.', '..', '.git') for v in value.split('/')), 'unsafe receipt path')
    require(p.as_posix() == value, 'normalized receipt path')
    return p


def hashes(path):
    path = plain(path)
    initial = path.stat()
    require(path.is_file() and initial.st_size <= MAX_FILE, 'bounded regular file')
    key = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns)
    sha256 = hashlib.sha256()
    blob = hashlib.sha1(b'blob ' + str(initial.st_size).encode('ascii') + b'\0')
    count = 0
    with path.open('rb') as handle:
        require(key(os.fstat(handle.fileno())) == key(initial), 'hash identity changed')
        for chunk in iter(lambda: handle.read(65536), b''):
            count += len(chunk)
            require(count <= initial.st_size, 'hash source grew')
            sha256.update(chunk)
            blob.update(chunk)
        require(key(os.fstat(handle.fileno())) == key(initial), 'hash descriptor changed')
    require(count == initial.st_size and key(path.stat()) == key(initial), 'hash postcheck changed')
    return {'sha256': sha256.hexdigest(), 'bytes': count, 'blob_sha1': blob.hexdigest()}


def exact_json(raw):
    def pairs(items):
        output = {}
        for key, value in items:
            require(key not in output, 'duplicate JSON field')
            output[key] = value
        return output
    return json.loads(raw, object_pairs_hook=pairs)


def inventory(root, include_git=False):
    output = set()
    for current, dirs, files in os.walk(root, followlinks=False):
        if include_git and Path(current) == root:
            dirs[:] = [v for v in dirs if v != '.git']
        for name in dirs:
            require(not (Path(current) / name).is_symlink(), 'inventory directory symlink')
        for name in files:
            path = plain(Path(current) / name)
            require(path.is_file(), 'inventory regular file')
            output.add(path.relative_to(root).as_posix())
    return output


def verify_export(root, source, include_git=False):
    root, source = plain(root), plain(source)
    receipt_path = root / 'SOURCE_EXPORT.json'
    require(receipt_path.stat().st_size <= MAX_RECEIPT, 'receipt byte bound')
    receipt = exact_json(receipt_path.read_bytes())
    require(receipt.get('schema') == 2 and receipt.get('release_ready') is False, 'source export schema/readiness')
    records = receipt.get('files')
    require(isinstance(records, list) and 0 < len(records) <= MAX_FILES, 'receipt file count')
    expected, seen = {}, set()
    for record in records:
        require(isinstance(record, dict), 'receipt record')
        name = record.get('path')
        relative(name)
        leaf=Path(name).name.lower()
        require(leaf not in {'.env','.env.local','.netrc','.npmrc','.pypirc','credentials','id_rsa','id_ed25519'} and not (leaf.startswith('.env.') and leaf!='.env.example') and Path(name).suffix.lower() not in {'.pem','.p12','.pfx','.key'}, 'credential-like export filename')
        require(name.casefold() not in seen and name != 'SOURCE_EXPORT.json', 'duplicate/reserved receipt path')
        seen.add(name.casefold())
        digest = record.get('sha256')
        require(isinstance(digest, str) and re.fullmatch('[0-9a-f]{64}', digest), 'receipt SHA domain')
        require(type(record.get('bytes')) is int and 0 <= record['bytes'] <= MAX_FILE, 'receipt size domain')
        proof = hashes(root / name)
        require(proof['sha256'] == digest and proof['bytes'] == record['bytes'], 'export bytes/SHA mismatch')
        if record.get('generated') is True:
            require(name == receipt.get('omission_note') == 'docs/EXPORT-EVIDENCE-OMITTED.md', 'only omission note generated')
        else:
            require(isinstance(record.get('original_sha256'), str), 'original SHA missing')
            require(hashes(source / name)['sha256'] == record['original_sha256'], 'original changed since export')
        expected[name] = proof
    require(inventory(root, include_git) == set(expected) | {'SOURCE_EXPORT.json'}, 'exact export namespace')
    expected['SOURCE_EXPORT.json'] = hashes(receipt_path)
    return expected


def clean_env(work):
    return {'PATH': '/usr/bin:/bin', 'LANG': 'C.UTF-8', 'LC_ALL': 'C.UTF-8',
            'TMPDIR': str(work), 'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': '/dev/null',
            'GIT_CONFIG_COUNT': '0', 'GIT_TERMINAL_PROMPT': '0', 'GIT_ALLOW_PROTOCOL': '',
            'GIT_PROTOCOL_FROM_USER': '0'}


def tree_entries(raw):
    require(len(raw) <= 4 * 1024 * 1024, 'tree byte bound')
    entries = {}
    for row in raw.split(b'\0'):
        if not row:
            continue
        head, name = row.split(b'\t', 1)
        mode, kind, oid = head.split(b' ')
        path = name.decode('utf-8', 'strict')
        relative(path)
        require(mode == b'100644' and kind == b'blob' and re.fullmatch(b'[0-9a-f]{40}', oid), 'regular SHA1 tree entry')
        require(path not in entries and len(entries) < MAX_FILES, 'duplicate/tree count')
        entries[path] = oid.decode('ascii')
    return entries


def verify_tree(raw, expected):
    tree = tree_entries(raw)
    require(set(tree) == set(expected), 'committed exact tree namespace')
    require(all(tree[name] == proof['blob_sha1'] for name, proof in expected.items()), 'committed blob bytes differ')


def git_runner(binary, work, template, operations):
    config = ['-c', 'core.hooksPath=' + str(template), '-c', 'core.fsmonitor=false',
              '-c', 'core.attributesFile=/dev/null', '-c', 'core.excludesFile=/dev/null',
              '-c', 'core.autocrlf=false', '-c', 'core.fileMode=false', '-c', 'protocol.allow=never',
              '-c', 'credential.helper=', '-c', 'commit.gpgSign=false', '-c', 'tag.gpgSign=false',
              '-c', 'gc.auto=0', '-c', 'maintenance.auto=false']
    def run(*argv):
        command = [str(binary), *config, *argv]
        # Local Git only; configs/hooks/network protocols disabled. No shell.
        output = work / ('git-%02d.stdout' % len(operations))
        error = work / ('git-%02d.stderr' % len(operations))
        with output.open('xb') as stdout, error.open('xb') as stderr:
            process = subprocess.run(command, env=clean_env(work), cwd=work, stdin=subprocess.DEVNULL,
                                     stdout=stdout, stderr=stderr, timeout=120, check=False)
        operations.append({'argv': command, 'returncode': process.returncode,
                           'stdout': output.name, 'stderr': error.name})
        require(output.stat().st_size <= 4 * 1024 * 1024 and error.stat().st_size <= 4 * 1024 * 1024,
                'Git evidence byte bound (postcheck, not disk hard cap)')
        require(process.returncode == 0, 'local Git operation failed; preserve output')
        return output.read_bytes()
    return run


def verify_committed_sha256(binary, repository, work, expected):
    # Read actual Git object bytes independently of SHA1 object-name comparison.
    # Owned anonymous sinks avoid whole-file RAM and preserve source/export files.
    for name, proof in expected.items():
        command = [str(binary), '-c', 'core.hooksPath=/dev/null', '-c', 'core.fsmonitor=false',
                   '-c', 'protocol.allow=never', '-C', str(repository), 'cat-file', 'blob', proof['blob_sha1']]
        with tempfile.TemporaryFile(dir=work) as output, tempfile.TemporaryFile(dir=work) as error:
            process = subprocess.run(command, env=clean_env(work), cwd=work, stdin=subprocess.DEVNULL,
                                     stdout=output, stderr=error, timeout=30, check=False)
            require(process.returncode == 0, 'committed object read failed')
            require(output.tell() == proof['bytes'] and error.tell() <= 65536, 'committed object byte bound')
            output.seek(0)
            digest = hashlib.sha256()
            for chunk in iter(lambda: output.read(65536), b''):
                digest.update(chunk)
            require(digest.hexdigest() == proof['sha256'], 'actual committed object SHA256 differs')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output-root', type=Path, required=True)
    p.add_argument('--git', type=Path, default=Path('/usr/bin/git'))
    p.add_argument('--git-sha256', required=True)
    p.add_argument('--execute', action='store_true')
    a = p.parse_args()
    destination = fresh_root(a.output_root, PROJECT)
    git = plain(a.git)
    require(git == Path('/usr/bin/git'), 'explicit local Linux Git only')
    require(re.fullmatch('[0-9a-f]{64}', a.git_sha256) and hashes(git)['sha256'] == a.git_sha256, 'original Git executable pin')
    if not a.execute:
        print(json.dumps({'status': 'PLAN_ONLY', 'source': str(PROJECT), 'output_root': str(destination),
                          'original_git_untouched': True, 'network': False, 'git_executed': False}))
        return
    destination.mkdir(mode=0o700)
    template = destination / 'empty-template'
    template.mkdir(mode=0o700)
    export_root = destination / 'source'
    operations = []
    result = {'schema': 1, 'status': 'FAILED_PRESERVED', 'source': str(PROJECT),
              'original_git_untouched': True, 'network': False, 'remote_configured': False,
              'external_ci_executed': False, 'release_ready': False, 'operations': operations}
    try:
        sys.dont_write_bytecode = True
        sys.path.insert(0, str(PROJECT / 'scripts'))
        from export_source import export
        export(PROJECT, export_root)
        expected = verify_export(export_root, PROJECT)
        result['export_file_count'] = len(expected)
        # check_docs is Python stdlib only; no product/native executable is launched.
        docs = subprocess.run([sys.executable, '-B', str(export_root / 'scripts/check_docs.py')],
                              env={'PATH':'/usr/bin:/bin','PYTHONDONTWRITEBYTECODE':'1'},cwd=export_root,
                              stdin=subprocess.DEVNULL,capture_output=True,timeout=120,check=False)
        require(len(docs.stdout) + len(docs.stderr) <= 1024 * 1024, 'docs evidence bound')
        result['docs'] = {'returncode':docs.returncode,'stdout':docs.stdout.decode('utf-8','replace'),
                          'stderr':docs.stderr.decode('utf-8','replace')}
        require(docs.returncode == 0, 'export docs check failed')
        require(verify_export(export_root, PROJECT) == expected, 'export changed after docs check')
        run = git_runner(git, destination, template, operations)
        run('init','--initial-branch=main','--object-format=sha1','--template='+str(template),str(export_root))
        require(not list(template.iterdir()), 'empty hook template preserved')
        run('-C',str(export_root),'add','--force','--all')
        run('-c','user.name=dlpad source export','-c','user.email=source-export@dlpad.invalid',
            '-C',str(export_root),'commit','-m','Import reviewed source export')
        require(run('-C',str(export_root),'status','--porcelain=v1','--untracked-files=all') == b'', 'commit clean state')
        verify_tree(run('-C',str(export_root),'ls-tree','-rz','--full-tree','HEAD'),expected)
        verify_committed_sha256(git, export_root, destination, expected)
        run('-C',str(export_root),'fsck','--full')
        require(run('-C',str(export_root),'remote') == b'', 'no remote')
        bundle = destination / 'source.bundle'
        require(not bundle.exists(), 'create-only bundle')
        run('-C',str(export_root),'bundle','create',str(bundle),'--all')
        run('-C',str(export_root),'bundle','verify',str(bundle))
        require(verify_export(export_root, PROJECT, include_git=True) == expected, 'post-Git export/original changed')
        require(hashes(git)['sha256'] == a.git_sha256, 'Git executable changed')
        result.update(status='REVIEW_REQUIRED',bundle=hashes(bundle),bundle_path=str(bundle),
                      commit=run('-C',str(export_root),'rev-parse','HEAD').decode('ascii').strip(),
                      exact_receipt_tree_sha_verified=True)
    except BaseException as error:
        result['failure'] = type(error).__name__ + ': ' + str(error)
        raise
    finally:
        with (destination / 'result.json').open('x',encoding='utf-8') as handle:
            json.dump(result,handle,indent=2);handle.write('\n')
    print(str(destination / 'result.json'))

if __name__ == '__main__':
    main()
