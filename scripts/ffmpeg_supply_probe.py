"""Probe one pinned Linux LGPL FFmpeg archive in a fresh project-owned directory."""
import argparse
import hashlib
import io
import lzma
import json
import os
import pathlib
import resource
import signal
import subprocess
import tarfile
import tempfile
import time
import urllib.request

TAG = 'autobuild-2026-09-30-13-08'
NAME = 'ffmpeg-n9.0.2-17-g2a571b6068-linux64-lgpl-9.0.tar.xz'
ROOT_NAME = NAME[:-7]
SIZE = 137863452
DIGEST = '2d41cbea0ca1a15029b638330740f78d6f5aeb062355bf425433fc938705350a'
ORIGIN = 'https://github.com/BtbN/FFmpeg-Builds/releases/download/' + TAG + '/'
MAX_TOTAL = 768 * 1024 * 1024
DOCS = {'LICENSE', 'LICENSE.txt', 'LICENSE.md', 'COPYING', 'COPYING.LGPLv2.1',
        'COPYING.LGPLv3', 'README', 'README.txt', 'README.md', 'SOURCE', 'SOURCE.txt',
        'SOURCES', 'SOURCES.txt', 'NOTICE', 'NOTICE.txt', 'buildconf.txt'}


def fetch(url, destination, maximum, expected_size=None, expected_digest=None):
    start = time.monotonic()
    digest = hashlib.sha256()
    total = 0
    with urllib.request.urlopen(url, timeout=20) as response, destination.open('xb') as stream:
        if not response.url.startswith(('https://github.com/', 'https://release-assets.githubusercontent.com/', 'https://api.github.com/', 'https://raw.githubusercontent.com/')):
            raise ValueError('unexpected redirect origin')
        while True:
            if time.monotonic() - start > 300:
                raise TimeoutError('download wall deadline')
            block = response.read(min(1024 * 1024, maximum - total + 1))
            if not block:
                break
            total += len(block)
            if total > maximum:
                raise ValueError('download byte bound')
            stream.write(block)
            digest.update(block)
    value = digest.hexdigest()
    if expected_size is not None and total != expected_size:
        raise ValueError('archive size mismatch')
    if expected_digest is not None and value != expected_digest:
        raise ValueError('archive checksum mismatch')
    return {'url': url, 'bytes': total, 'sha256': value, 'seconds': time.monotonic() - start}


def extract(archive, destination):
    total = 0
    members = 0
    seen = set()
    selected = []
    inventory = []
    started = time.monotonic()
    with tarfile.open(archive, 'r|xz') as stream:
        for member in stream:
            members += 1
            if members > 4096 or time.monotonic() - started > 120:
                raise ValueError('archive member/time bound')
            path = pathlib.PurePosixPath(member.name)
            if path.is_absolute() or '..' in path.parts or '\\' in member.name or not path.parts or path.parts[0] != ROOT_NAME:
                raise ValueError('unsafe archive path')
            if member.name in seen:
                raise ValueError('duplicate archive member')
            seen.add(member.name)
            if not (member.isdir() or member.isreg()):
                raise ValueError('archive special/link member')
            if member.size < 0:
                raise ValueError('negative member size')
            total += member.size
            if total > MAX_TOTAL:
                raise ValueError('unpacked byte bound')
            inventory.append({'name': member.name, 'bytes': member.size, 'regular': member.isreg()})
            relative = path.parts[1:]
            binary = relative in [('bin', 'ffmpeg'), ('bin', 'ffprobe')]
            notice = len(relative) == 1 and relative[0] in DOCS
            if not member.isreg() or not (binary or notice):
                continue
            limit = 384 * 1024 * 1024 if binary else 2 * 1024 * 1024
            if member.size > limit:
                raise ValueError('selected member byte bound')
            target = destination.joinpath(*relative)
            target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
            digest = hashlib.sha256()
            copied = 0
            with stream.extractfile(member) as reader, target.open('xb') as writer:
                while True:
                    block = reader.read(min(1024 * 1024, member.size - copied + 1))
                    if not block:
                        break
                    copied += len(block)
                    if copied > member.size:
                        raise ValueError('member size mismatch')
                    writer.write(block)
                    digest.update(block)
            if copied != member.size:
                raise ValueError('short archive member')
            target.chmod(0o700 if binary else 0o600)
            selected.append({'name': member.name, 'relative': '/'.join(relative), 'bytes': copied, 'sha256': digest.hexdigest()})
    if not {'bin/ffmpeg', 'bin/ffprobe'} <= {item['relative'] for item in selected}:
        raise ValueError('missing executable set')
    return {'members': members, 'archive_unpacked_bytes': total,
            'selected_bytes': sum(item['bytes'] for item in selected),
            'seconds': time.monotonic() - started, 'selected': selected, 'inventory': inventory}


def run(argv, work, label, expected=0):
    logfile = work / (label + '.txt')
    started = time.monotonic()
    def child_limits():
        # Applies only to our child; bounds logs and generated fixture files.
        resource.setrlimit(resource.RLIMIT_FSIZE, (4 * 1024 * 1024, 4 * 1024 * 1024))
    process = subprocess.Popen([str(arg) for arg in argv], stdin=subprocess.DEVNULL,
                               stdout=logfile.open('xb'), stderr=subprocess.STDOUT,
                               start_new_session=True, cwd=work, preexec_fn=child_limits)
    status = None
    usage = None
    live_rss = 0
    try:
        while status is None:
            pid, status_value, used = os.wait4(process.pid, os.WNOHANG)
            if pid:
                status, usage = status_value, used
                break
            try:
                if pathlib.Path('/proc', str(process.pid), 'exe').resolve() == pathlib.Path(argv[0]).resolve():
                    text = pathlib.Path('/proc', str(process.pid), 'status').read_text()
                    live_rss = max(live_rss, next((int(line.split()[1]) for line in text.splitlines() if line.startswith('VmRSS:')), 0))
            except (FileNotFoundError, PermissionError, ProcessLookupError):
                pass
            if time.monotonic() - started > 30 or logfile.stat().st_size > 256 * 1024:
                raise TimeoutError('owned tool deadline/log bound')
            time.sleep(0.01)
    except BaseException:
        os.killpg(process.pid, signal.SIGKILL)
        _, status, usage = os.wait4(process.pid, 0)
        process.returncode = os.waitstatus_to_exitcode(status)
        raise
    process.returncode = os.waitstatus_to_exitcode(status)
    if logfile.stat().st_size > 256 * 1024:
        raise ValueError('completed tool log bound')
    record = {'argv': [str(arg) for arg in argv], 'exit_code': process.returncode,
              'seconds': time.monotonic() - started, 'cpu_seconds': usage.ru_utime + usage.ru_stime,
              'peak_rss_kib': usage.ru_maxrss, 'sampled_post_exec_rss_kib': live_rss,
              'rss_note': 'wait4 includes inherited pre-exec launcher RSS; post-exec sample every 10ms may miss peaks', 'log': logfile.name}
    if expected is not None and process.returncode != expected:
        raise RuntimeError(label + ': ' + logfile.read_text(errors='replace')[-8192:])
    return record


def self_test():
    results = {}
    with tempfile.TemporaryDirectory(prefix='dlpad-supply-security-') as folder:
        root = pathlib.Path(folder)
        def member(name, size=0, kind=tarfile.REGTYPE):
            item = tarfile.TarInfo(name)
            item.size, item.type, item.linkname = size, kind, '/tmp/foreign'
            return item
        def check(name, entries, expected):
            raw = io.BytesIO()
            for item, payload in entries:
                raw.write(item.tobuf())
                raw.write(payload)
                raw.write(b'\0' * ((-len(payload)) % 512))
            raw.write(b'\0' * 1024)
            archive = root / (name + '.tar.xz')
            archive.write_bytes(lzma.compress(raw.getvalue()))
            destination = root / name
            destination.mkdir()
            try:
                extract(archive, destination)
                result = 'accepted'
            except (ValueError, FileExistsError):
                result = 'rejected'
            if result != expected:
                raise AssertionError((name, result))
            results[name] = result
        prefix = ROOT_NAME + '/'
        for name, path in [('absolute-path', '/tmp/escape'), ('parent-path', prefix + '../escape')]:
            check(name, [(member(path), b'')], 'rejected')
        for name, kind in [('symlink', tarfile.SYMTYPE), ('hardlink', tarfile.LNKTYPE)]:
            check(name, [(member(prefix + 'bin/ffmpeg', kind=kind), b'')], 'rejected')
        check('duplicate', [(member(prefix + 'LICENSE.txt'), b'')] * 2, 'rejected')
        check('huge-size', [(member(prefix + 'bin/ffmpeg', MAX_TOTAL + 1), b'')], 'rejected')
        check('allowlist-control', [(member(prefix + 'bin/ffmpeg', 1), b'f'),
                                   (member(prefix + 'bin/ffprobe', 1), b'p'),
                                   (member(prefix + 'unlisted.txt', 1), b'u')], 'accepted')
        if (root / 'allowlist-control/unlisted.txt').exists():
            raise AssertionError('unlisted file extracted')
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
        parser.error('--evidence is required for the network probe')
    project = pathlib.Path(__file__).resolve().parents[1]
    evidence = args.evidence.resolve()
    if not evidence.is_relative_to(project / 'docs' / 'validation') or evidence.exists():
        raise ValueError('new project validation evidence path required')
    work = pathlib.Path(tempfile.mkdtemp(prefix='ffmpeg-supply-', dir=project / '.local-tools'))
    work.chmod(0o700)
    report = {'tag': TAG, 'work': str(work.relative_to(project)), 'adopted': False, 'runs': {}}
    def save():
        evidence.write_text(json.dumps(report, indent=2) + '\n')
    try:
        api = work / 'release.json'
        report['api'] = fetch('https://api.github.com/repos/BtbN/FFmpeg-Builds/releases/tags/' + TAG, api, 2 * 1024 * 1024)
        release = json.loads(api.read_text())
        asset = next(item for item in release['assets'] if item['name'] == NAME)
        if asset['size'] != SIZE or asset['digest'] != 'sha256:' + DIGEST or asset['browser_download_url'] != ORIGIN + NAME:
            raise ValueError('official asset pin mismatch')
        report['published_at'] = release['published_at']
        checksum = work / 'checksums.sha256'
        report['checksums'] = fetch(ORIGIN + 'checksums.sha256', checksum, 64 * 1024)
        if not any(line.split() == [DIGEST, NAME] for line in checksum.read_text().splitlines()):
            raise ValueError('official checksum row mismatch')
        archive = work / NAME
        report['archive'] = fetch(ORIGIN + NAME, archive, 140 * 1024 * 1024, SIZE, DIGEST)
        save()
        installation = work / 'selected'
        installation.mkdir(mode=0o700)
        report['extraction'] = extract(archive, installation)
        ffmpeg, ffprobe = installation / 'bin/ffmpeg', installation / 'bin/ffprobe'
        for binary, label in [(ffmpeg, 'ffmpeg'), (ffprobe, 'ffprobe')]:
            for flag, detail in [('-version', 'version'), ('-buildconf', 'buildconf'), ('-L', 'license')]:
                report['runs'][label + '-' + detail] = run([binary, flag], work, label + '-' + detail)
        buildconf = (work / 'ffmpeg-buildconf.txt').read_text()
        if '--enable-gpl' in buildconf or '--enable-nonfree' in buildconf:
            raise ValueError('GPL/nonfree build outside LGPL candidate')
        media = work / 'synthetic.mp4'
        prefix = [ffmpeg, '-nostdin', '-hide_banner', '-loglevel', 'error']
        report['runs']['encode'] = run(prefix + ['-f', 'lavfi', '-i', 'testsrc2=size=160x90:rate=10', '-f', 'lavfi', '-i', 'sine=frequency=440:sample_rate=48000', '-t', '2', '-c:v', 'mpeg4', '-c:a', 'aac', media], work, 'encode')
        report['runs']['probe'] = run([ffprobe, '-v', 'error', '-show_streams', '-show_format', '-of', 'json', media], work, 'probe')
        probe = json.loads((work / 'probe.txt').read_text())
        if {stream['codec_name'] for stream in probe['streams']} != {'mpeg4', 'aac'}:
            raise ValueError('unexpected synthetic codecs')
        report['runs']['decode'] = run(prefix + ['-i', media, '-f', 'null', '-'], work, 'decode')
        cut = work / 'default-accurate-cut.mp4'
        report['runs']['default-cut'] = run(prefix + ['-ss', '0.2', '-i', media, '-t', '0.8', cut], work, 'default-cut', expected=None)
        if report['runs']['default-cut']['exit_code'] == 0:
            report['runs']['cut-decode'] = run(prefix + ['-i', cut, '-f', 'null', '-'], work, 'cut-decode')
        report['runs']['encoders'] = run([ffmpeg, '-hide_banner', '-encoders'], work, 'encoders')
        report['fixture_bytes'] = media.stat().st_size
        report['status'] = 'candidate-probed-not-adopted'
        report['gates'] = ['exact corresponding source/dependency notices/build recipe', 'supplier signature/attestation trust', 'Windows/macOS and minimum OS', 'managed multi-component archive activation/rollback']
        save()
    except BaseException as error:
        report['status'] = 'failed'
        report['error'] = str(error)
        save()
        raise
    print(json.dumps({'evidence': str(evidence), 'work': str(work), 'status': report['status'], 'archive': report['archive'], 'extraction': {k:v for k,v in report['extraction'].items() if k!='inventory'}, 'runs': report['runs']}, indent=2))


if __name__ == '__main__':
    main()
