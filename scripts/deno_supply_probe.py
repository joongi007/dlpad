"""Probe a pinned Deno ZIP with offline synthetic JavaScript and private cache."""
import argparse
import hashlib
import json
import os
import pathlib
import resource
import signal
import stat
import subprocess
import tempfile
import time
import zipfile
from ffmpeg_supply_probe import fetch

VERSION = 'v2.9.7'
NAME = 'deno-x86_64-unknown-linux-gnu.zip'
SIZE = 41596794
SHA = 'c6527f24f4b16031d3ae4fa9f658d5f11534c8d84ce7dc8502420280919c3490'
ORIGIN = 'https://github.com/denoland/deno/releases/download/' + VERSION + '/'
MAX_UNPACKED = 256 * 1024 * 1024
MAX_CACHE = 32 * 1024 * 1024


def extract(archive, destination):
    total = 0
    seen = set()
    selected = []
    with zipfile.ZipFile(archive) as bundle:
        members = bundle.infolist()
        if len(members) > 32:
            raise ValueError('ZIP entry bound')
        for item in members:
            path = pathlib.PurePosixPath(item.filename)
            if path.is_absolute() or '..' in path.parts or '\\' in item.filename or ':' in item.filename:
                raise ValueError('unsafe ZIP path')
            if str(path) in seen:
                raise ValueError('duplicate ZIP entry')
            seen.add(str(path))
            mode = item.external_attr >> 16
            if stat.S_ISLNK(mode) or (stat.S_IFMT(mode) and not stat.S_ISREG(mode)):
                raise ValueError('ZIP nonregular member')
            total += item.file_size
            if total > MAX_UNPACKED or item.file_size > MAX_UNPACKED or item.flag_bits & 1:
                raise ValueError('ZIP size/encryption bound')
            if item.filename not in {'deno', 'LICENSE', 'LICENSE.md', 'NOTICE', 'NOTICE.md', 'README.md'}:
                continue
            target = destination / item.filename
            digest = hashlib.sha256()
            count = 0
            with bundle.open(item) as reader, target.open('xb') as output:
                while True:
                    block = reader.read(1024 * 1024)
                    if not block:
                        break
                    count += len(block)
                    if count > item.file_size:
                        raise ValueError('ZIP actual size overflow')
                    digest.update(block)
                    output.write(block)
            if count != item.file_size:
                raise ValueError('ZIP short member')
            target.chmod(0o700 if item.filename == 'deno' else 0o600)
            selected.append({'name': item.filename, 'bytes': count, 'sha256': digest.hexdigest()})
    if 'deno' not in {item['name'] for item in selected}:
        raise ValueError('missing deno executable')
    return {'archive_unpacked_bytes': total, 'selected': selected}


def cache_size(cache):
    total = 0
    count = 0
    for path in cache.rglob('*'):
        count += 1
        if count > 512 or path.is_symlink():
            raise ValueError('cache entry/link bound')
        if path.is_file():
            total += path.stat().st_size
    if total > MAX_CACHE:
        raise ValueError('cache byte budget')
    return total


def run(argv, work, cache, label, deadline=10, cancel_after=None):
    log = work / (label + '.txt')
    environment = os.environ.copy()
    environment.update({'DENO_DIR': str(cache), 'DENO_NO_UPDATE_CHECK': '1',
                        'DENO_NO_PROMPT': '1', 'NO_COLOR': '1'})
    def limits():
        resource.setrlimit(resource.RLIMIT_FSIZE, (4 * 1024 * 1024, 4 * 1024 * 1024))
    started = time.monotonic()
    with log.open('xb') as stream:
        process = subprocess.Popen([str(item) for item in argv], cwd=work, env=environment,
                                   stdin=subprocess.DEVNULL, stdout=stream, stderr=subprocess.STDOUT,
                                   start_new_session=True, preexec_fn=limits)
    reason = 'Exited'
    rss = 0
    try:
        while True:
            pid, status, usage = os.wait4(process.pid, os.WNOHANG)
            if pid:
                break
            cache_size(cache)
            elapsed = time.monotonic() - started
            if elapsed >= deadline or (cancel_after is not None and elapsed >= cancel_after):
                reason = 'Cancelled' if cancel_after is not None else 'Timeout'
                os.killpg(process.pid, signal.SIGKILL)
                _, status, usage = os.wait4(process.pid, 0)
                break
            if log.stat().st_size > 64 * 1024:
                raise ValueError('log byte bound')
            try:
                if pathlib.Path('/proc', str(process.pid), 'exe').resolve() == pathlib.Path(argv[0]).resolve():
                    text = pathlib.Path('/proc', str(process.pid), 'status').read_text()
                    rss = max(rss, next((int(line.split()[1]) for line in text.splitlines() if line.startswith('VmRSS:')), 0))
            except (FileNotFoundError, ProcessLookupError):
                pass
            time.sleep(0.01)
    except BaseException:
        os.killpg(process.pid, signal.SIGKILL)
        _, status, usage = os.wait4(process.pid, 0)
        process.returncode = os.waitstatus_to_exitcode(status)
        raise
    process.returncode = os.waitstatus_to_exitcode(status)
    if log.stat().st_size > 64 * 1024:
        raise ValueError('final log byte bound')
    return {'argv': [str(item) for item in argv], 'outcome': reason, 'exit_code': process.returncode,
            'seconds': time.monotonic() - started, 'cpu_seconds': usage.ru_utime + usage.ru_stime,
            'launch_peak_rss_kib': usage.ru_maxrss, 'sampled_post_exec_rss_kib': rss,
            'cache_bytes': cache_size(cache), 'log': log.name}


def self_test():
    results = {}
    with tempfile.TemporaryDirectory(prefix='dlpad-deno-zip-') as folder:
        root = pathlib.Path(folder)
        for label, names, symlink in [('absolute', ['/tmp/escape'], False), ('parent', ['../escape'], False),
              ('backslash', ['..\\escape'], False), ('symlink', ['deno'], True),
              ('duplicate', ['deno', 'deno'], False), ('valid', ['deno', 'unlisted'], False)]:
            archive = root / (label + '.zip')
            with zipfile.ZipFile(archive, 'w') as bundle:
                for name in names:
                    item = zipfile.ZipInfo(name)
                    item.external_attr = (stat.S_IFLNK | 0o777) << 16 if symlink else (stat.S_IFREG | 0o700) << 16
                    bundle.writestr(item, b'x')
            destination = root / label
            destination.mkdir()
            try:
                extract(archive, destination)
                outcome = 'accepted'
            except (ValueError, FileExistsError):
                outcome = 'rejected'
            if outcome != ('accepted' if label == 'valid' else 'rejected'):
                raise AssertionError(label)
            if (destination / 'unlisted').exists():
                raise AssertionError('allowlist bypass')
            results[label] = outcome
        oversized = root / 'oversized.zip'
        with zipfile.ZipFile(oversized, 'w') as bundle:
            bundle.writestr('deno', b'x')
        data = bytearray(oversized.read_bytes())
        central = data.index(b'PK\x01\x02')
        data[central + 24:central + 28] = (MAX_UNPACKED + 1).to_bytes(4, 'little')
        oversized.write_bytes(data)
        destination = root / 'oversized'
        destination.mkdir()
        try:
            extract(oversized, destination)
            raise AssertionError('oversized ZIP accepted')
        except ValueError:
            results['oversized'] = 'rejected'
        cache = root / 'cache'
        cache.mkdir()
        with (cache / 'synthetic-sparse').open('wb') as stream:
            stream.seek(MAX_CACHE)
            stream.write(b'x')
        try:
            cache_size(cache)
            raise AssertionError('cache budget exceeded')
        except ValueError:
            results['cache-budget'] = 'rejected'
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evidence', type=pathlib.Path)
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    if args.self_test:
        print(json.dumps(self_test(), indent=2))
        return
    if args.evidence is None:
        parser.error('--evidence required')
    project = pathlib.Path(__file__).resolve().parents[1]
    evidence = args.evidence.resolve()
    if evidence.exists() or not evidence.is_relative_to(project / 'docs/validation'):
        raise ValueError('new project validation path required')
    work = pathlib.Path(tempfile.mkdtemp(prefix='deno-supply-', dir=project / '.local-tools'))
    work.chmod(0o700)
    cache = work / 'cache'
    cache.mkdir(mode=0o700)
    report = {'version_pin': VERSION, 'work': str(work.relative_to(project)), 'adopted': False, 'runs': {}}
    def save():
        evidence.write_text(json.dumps(report, indent=2) + '\n')
    try:
        api = work / 'release.json'
        report['api'] = fetch('https://api.github.com/repos/denoland/deno/releases/tags/' + VERSION, api, 2 * 1024 * 1024)
        release = json.loads(api.read_text())
        asset = next(item for item in release['assets'] if item['name'] == NAME)
        if asset['size'] != SIZE or asset['digest'] != 'sha256:' + SHA:
            raise ValueError('Deno release pin mismatch')
        checksum = work / 'archive.sha256sum'
        report['checksum'] = fetch(ORIGIN + NAME + '.sha256sum', checksum, 4096)
        if checksum.read_text().split()[0] != SHA:
            raise ValueError('Deno checksum mismatch')
        report['license'] = fetch('https://raw.githubusercontent.com/denoland/deno/' + VERSION + '/LICENSE.md', work / 'LICENSE.md', 64 * 1024)
        archive = work / NAME
        report['archive'] = fetch(ORIGIN + NAME, archive, 48 * 1024 * 1024, SIZE, SHA)
        installation = work / 'selected'
        installation.mkdir(mode=0o700)
        report['extraction'] = extract(archive, installation)
        binary = installation / 'deno'
        report['runs']['version'] = run([binary, '--version'], work, cache, 'version')
        if report['runs']['version']['exit_code'] != 0 or 'deno 2.9.7' not in (work / 'version.txt').read_text():
            raise ValueError('Deno version mismatch')
        fixture = work / 'fixture.js'
        fixture.write_text('const a=JSON.parse(Deno.args[0]); console.log(JSON.stringify({sum:a.reduce((x,y)=>x+y,0),count:a.length,digest:Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256",new TextEncoder().encode("dlpad synthetic")))).map(x=>x.toString(16).padStart(2,"0")).join("")}));\n')
        infinite = work / 'infinite.js'
        infinite.write_text('console.log("ready"); while(true){}\n')
        base = [binary, 'run', '--no-remote', '--cached-only', '--no-check', '--no-config', '--no-lock', '--no-prompt', '--deny-net', '--deny-run', '--deny-read', '--deny-write', '--deny-env', '--deny-sys']
        report['runs']['fixture'] = run(base + [fixture, '[3,5,8]'], work, cache, 'fixture')
        if report['runs']['fixture']['exit_code'] != 0:
            raise RuntimeError((work / 'fixture.txt').read_text())
        value = json.loads((work / 'fixture.txt').read_text())
        if value != {'sum': 16, 'count': 3, 'digest': hashlib.sha256(b'dlpad synthetic').hexdigest()}:
            raise ValueError('synthetic fixture result mismatch')
        report['runs']['timeout'] = run(base + [infinite], work, cache, 'timeout', deadline=1)
        report['runs']['cancel'] = run(base + [infinite], work, cache, 'cancel', deadline=5, cancel_after=0.25)
        for label, outcome in [('timeout', 'Timeout'), ('cancel', 'Cancelled')]:
            result = report['runs'][label]
            if result['outcome'] != outcome or result['exit_code'] == 0 or 'ready' not in (work / result['log']).read_text():
                raise ValueError('actual ' + label + ' not verified')
        report['cache_bytes'] = cache_size(cache)
        report['self_test'] = self_test()
        report['status'] = 'candidate-probed-not-adopted'
        report['gates'] = ['third-party notices/source and supplier trust', 'bundled EJS exact-version integration', 'other OS and managed archive activation']
        save()
    except BaseException as error:
        report['status'], report['error'] = 'failed', str(error)
        save()
        raise
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
