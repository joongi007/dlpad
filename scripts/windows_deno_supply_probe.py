"""Collect pinned official Windows Deno bytes; never execute the runtime."""
import argparse
import datetime
import hashlib
import json
import pathlib
import stat
import tarfile
import time
import urllib.parse
import urllib.request
import zipfile

TAG = 'v2.9.7'
NAME = 'deno-x86_64-pc-windows-msvc.zip'
SIZE = 42630221
SHA = 'a0c3101b4158d1dfb7d6a78a7bf0f3de80c96bb423c152beec8beb22786f2238'
HOSTS = {'github.com', 'api.github.com', 'raw.githubusercontent.com',
         'release-assets.githubusercontent.com', 'codeload.github.com'}


class Redirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        checked_url(newurl)
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def checked_url(url):
    value = urllib.parse.urlsplit(url)
    if value.scheme != 'https' or value.hostname not in HOSTS or value.username or value.password:
        raise ValueError('official HTTPS origin required')


def fetch(url, destination, maximum):
    checked_url(url)
    digest = hashlib.sha256()
    count = 0
    start = time.monotonic()
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), Redirect())
    request = urllib.request.Request(url, headers={'User-Agent': 'dlpad-pinned-supply-probe'})
    with opener.open(request, timeout=20) as response, destination.open('xb') as output:
        checked_url(response.url)
        while True:
            if time.monotonic() - start > 180:
                raise TimeoutError('bounded download deadline')
            chunk = response.read(min(65536, maximum - count + 1))
            if not chunk:
                break
            count += len(chunk)
            if count > maximum:
                raise ValueError('download size bound')
            output.write(chunk)
            digest.update(chunk)
    return {'url': url, 'bytes': count, 'sha256': digest.hexdigest(),
            'seconds': time.monotonic() - start, 'file': destination.name}


def collect(work):
    report = {'version': 1, 'observed_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'tag': TAG, 'native_executed': False, 'adopted': False, 'materials': [], 'missing': []}
    try:
        api = 'https://api.github.com/repos/denoland/deno/'
        report['release'] = fetch(api + 'releases/tags/' + TAG, work / 'release.json', 2 * 1024 * 1024)
        release = json.loads((work / 'release.json').read_text())
        assets = [a for a in release['assets'] if a['name'] == NAME]
        if len(assets) != 1 or assets[0]['size'] != SIZE or assets[0].get('digest') != 'sha256:' + SHA:
            raise ValueError('official release metadata pin mismatch')
        origin = 'https://github.com/denoland/deno/releases/download/' + TAG + '/'
        report['checksum'] = fetch(origin + NAME + '.sha256sum', work / 'archive.sha256sum', 4096)
        checksum_text = (work / 'archive.sha256sum').read_text().strip()
        checksum = checksum_text.split()
        gnu_ok = len(checksum) == 2 and checksum[0].lower() == SHA and checksum[1].lstrip('*') == NAME
        rows = [line.strip().split(':', 1) for line in checksum_text.splitlines() if line.strip()]
        windows_ok = len(rows) == 3 and all(len(row) == 2 for row in rows)
        if windows_ok:
            fields = {key.strip(): value.strip() for key, value in rows}
            windows_ok = (set(fields) == {'Algorithm', 'Hash', 'Path'} and fields['Algorithm'] == 'SHA256'
                          and fields['Hash'].lower() == SHA and fields['Path'].split('\\')[-1] == NAME)
        if not (gnu_ok or windows_ok):
            raise ValueError('official checksum mismatch')
        report['checksum_format'] = 'PowerShell Get-FileHash' if windows_ok else 'sha256sum'
        report['archive'] = fetch(origin + NAME, work / NAME, SIZE)
        if report['archive']['bytes'] != SIZE or report['archive']['sha256'] != SHA:
            raise ValueError('actual archive pin mismatch')
        with zipfile.ZipFile(work / NAME) as archive:
            entries = archive.infolist()
            if len(entries) != 1 or entries[0].filename != 'deno.exe':
                raise ValueError('unexpected package inventory')
            entry = entries[0]
            mode = entry.external_attr >> 16
            if entry.flag_bits & 1 or stat.S_ISLNK(mode) or (stat.S_IFMT(mode) and not stat.S_ISREG(mode)) or entry.file_size > 256 * 1024 * 1024:
                raise ValueError('unsafe executable entry')
            count = 0
            digest = hashlib.sha256()
            with archive.open(entry) as source, (work / 'deno.exe').open('xb') as output:
                while chunk := source.read(65536):
                    count += len(chunk)
                    if count > entry.file_size:
                        raise ValueError('executable size overflow')
                    output.write(chunk)
                    digest.update(chunk)
            if count != entry.file_size:
                raise ValueError('executable short read')
            report['executable'] = {'bytes': count, 'sha256': digest.hexdigest(), 'crc32': entry.CRC,
                                    'file': 'deno.exe', 'native_executed': False}
        report['commit_metadata'] = fetch(api + 'commits/' + TAG, work / 'commit.json', 2 * 1024 * 1024)
        commit = json.loads((work / 'commit.json').read_text())
        revision = commit['sha']
        if len(revision) != 40 or any(c not in '0123456789abcdef' for c in revision):
            raise ValueError('invalid full revision')
        report['source_revision'] = revision
        report['tree'] = fetch(api + 'git/trees/' + commit['commit']['tree']['sha'], work / 'tree.json', 1024 * 1024)
        tree = json.loads((work / 'tree.json').read_text())
        if tree.get('truncated'):
            raise ValueError('incomplete official source tree')
        third = [i for i in tree['tree'] if i['path'] == 'third_party' and i['type'] == 'tree']
        paths = []
        if third:
            report['third_party_tree'] = fetch(api + 'git/trees/' + third[0]['sha'] + '?recursive=1',
                                               work / 'third-party-tree.json', 4 * 1024 * 1024)
            subtree = json.loads((work / 'third-party-tree.json').read_text())
            if subtree.get('truncated'):
                raise ValueError('incomplete third-party tree')
            paths = ['third_party/' + i['path'] for i in subtree['tree'] if i['type'] == 'blob']
        selected = ['LICENSE.md', 'Cargo.lock', 'Cargo.toml']
        selected += [p for p in paths if any(x in pathlib.PurePosixPath(p).name.lower() for x in ('license', 'notice', 'copying'))]
        if len(selected) > 32:
            raise ValueError('notice entry bound')
        report['third_party_inventory'] = paths[:256]
        for index, path in enumerate(selected):
            url = 'https://raw.githubusercontent.com/denoland/deno/' + revision + '/' + path
            item = fetch(url, work / ('material-%02d.txt' % index), 4 * 1024 * 1024)
            item['source_path'] = path
            report['materials'].append(item)
        report['source_archive'] = fetch('https://codeload.github.com/denoland/deno/tar.gz/' + revision,
                                         work / 'deno-source.tar.gz', 64 * 1024 * 1024)
        prefix = 'deno-' + revision + '/'
        bundled = {'cli/tsc/dts/node/LICENSE', 'cli/tsc/dts/node/undici/LICENSE',
                   'ext/webgpu/LICENSE.md', '.gitmodules', 'cli/tsc/99_main_compiler.js'}
        report['bundled_materials'] = []
        total = 0
        entries = 0
        with tarfile.open(work / 'deno-source.tar.gz', 'r|gz') as source:
            for member in source:
                entries += 1
                total += member.size
                if entries > 30000 or total > 512 * 1024 * 1024:
                    raise ValueError('source inventory bound')
                if member.isdir() and member.name.rstrip('/') == prefix.rstrip('/'):
                    continue
                if not member.name.startswith(prefix):
                    raise ValueError('source revision root mismatch')
                relative = member.name[len(prefix):]
                if relative not in bundled:
                    continue
                if not member.isfile() or member.size > 1024 * 1024:
                    raise ValueError('bundled material bound/type')
                stream = source.extractfile(member)
                if stream is None:
                    raise ValueError('bundled material missing data')
                payload = stream.read(1024 * 1024 + 1)
                if len(payload) != member.size:
                    raise ValueError('bundled material size mismatch')
                target = 'bundled-%02d.txt' % len(report['bundled_materials'])
                with (work / target).open('xb') as output:
                    output.write(payload)
                report['bundled_materials'].append({'source_path': relative, 'file': target,
                    'bytes': len(payload), 'sha256': hashlib.sha256(payload).hexdigest(),
                    'source_revision': revision, 'source_archive_sha256': report['source_archive']['sha256']})
        report['source_inventory_entries'] = entries
        report['source_declared_bytes'] = total
        report['missing'] = ['transitive Cargo/V8 embedded notice and exact source completeness audit',
                             'binary attestation/signature verification', 'Windows native version/activation/cache/RSS timing']
        report['status'] = 'materials-collected-not-adopted'
    except Exception as error:
        report['status'] = 'failed'
        report['error_type'] = type(error).__name__
        report['error'] = str(error)[:512]
    payload = json.dumps(report, ensure_ascii=False, indent=2).encode()
    if len(payload) > 65536:
        raise ValueError('result bound')
    with (work / 'result.json').open('xb') as output:
        output.write(payload)
    return report['status'] != 'failed'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=pathlib.Path)
    args = parser.parse_args()
    work = args.output.absolute()
    work.mkdir(parents=True, exist_ok=False)
    return 0 if collect(work) else 1


if __name__ == '__main__':
    raise SystemExit(main())
