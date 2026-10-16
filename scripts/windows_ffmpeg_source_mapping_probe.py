#!/usr/bin/env python3
"""Read-only pinned Windows supply mapping; never execute extracted source."""
import argparse
import hashlib
import io
import json
import re
import ssl
import tarfile
import time
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BUILD = '6c9aec5fc9a72ec3abedd1fa84db141fa18cf52b'
FFMPEG = '2a571b606854520cf89804d8030c8b328e621689'
PINNED = {
    '.local-tools/ffmpeg-win64-inventory-f5uj2z2m/ffmpeg-n9.0.2-17-g2a571b6068-win64-lgpl-9.0.zip': ('6b264b9e6019103f601d98c292bd332fd87acf1c5e941ddff4fb71760fe63432', 171535354),
    '.local-tools/ffmpeg-correspondence-9je4wjt6/ffmpeg-source.tar.gz': ('f02fc0809a13f37e3d4e1f74254d07e035a5ef01a6d91e727b915b1764d97540', 17336083),
    '.local-tools/ffmpeg-correspondence-9je4wjt6/btbn-build-source.tar.gz': ('81da4f7c878dc0f60e6b8d0049a112e32178e20f025d468fe4e99e77b2dfe0ec', 107893),
}

def digest(path):
    if path.is_symlink() or not path.is_file():
        raise ValueError('original regular file required')
    before = path.stat()
    sha = hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda: source.read(65536), b''):
            sha.update(block)
    after = path.stat()
    if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns):
        raise ValueError('original changed while hashing')
    return {'bytes': before.st_size, 'sha256': sha.hexdigest()}

def write(path, value):
    data = json.dumps(value, ensure_ascii=False, indent=2).encode()
    if len(data) > 4 * 1024 * 1024:
        raise ValueError('evidence JSON bound')
    with path.open('xb') as out:
        out.write(data)

def fetch(output, name, url, cap, deadline):
    row = {'name': name, 'url': url, 'limit': cap, 'status': 'FAIL'}
    path = output / name
    try:
        if time.monotonic() >= deadline:
            raise TimeoutError('collector deadline')
        request = urllib.request.Request(url, headers={'User-Agent': 'dlpad-public-source-evidence/1'})
        with urllib.request.urlopen(request, timeout=min(20, deadline-time.monotonic()), context=ssl.create_default_context()) as response:
            final = urllib.parse.urlparse(response.url)
            if final.scheme != 'https' or final.hostname not in {'raw.githubusercontent.com', 'codeload.github.com', 'ffmpeg.org'}:
                raise ValueError('unexpected official redirect')
            if 'text/html' in response.headers.get('Content-Type', '') and name.endswith('.tar.gz'):
                raise ValueError('HTML is not source archive')
            sha = hashlib.sha256(); size = 0
            with path.open('xb') as dest:
                while True:
                    if time.monotonic() >= deadline:
                        raise TimeoutError('collector deadline')
                    block = response.read(min(65536, cap-size+1))
                    if not block:
                        break
                    size += len(block)
                    if size > cap:
                        raise ValueError('response size bound')
                    dest.write(block); sha.update(block)
            row.update(status='PASS', bytes=size, sha256=sha.hexdigest(), final_url=response.url)
    except Exception as error:
        row.update(error_type=type(error).__name__, error=str(error)[:512], partial_bytes=path.stat().st_size if path.exists() else 0)
    return row

def archive_recipes():
    result = {}
    archive = ROOT / '.local-tools/ffmpeg-correspondence-9je4wjt6/btbn-build-source.tar.gz'
    with tarfile.open(archive, 'r:gz') as source:
        count = 0; total = 0
        for member in source:
            count += 1
            if count > 4096:
                raise ValueError('recipe archive entry bound')
            if member.isfile() and ('/scripts.d/' in member.name or member.name.endswith('/README.md') or '/images/base-win64/' in member.name or '/variants/' in member.name):
                if member.size > 128 * 1024:
                    raise ValueError('recipe member bound')
                total += member.size
                if total > 4 * 1024 * 1024:
                    raise ValueError('recipe total bound')
                data = source.extractfile(member).read(member.size+1)
                if len(data) != member.size:
                    raise ValueError('recipe length mismatch')
                result[member.name.split('/', 1)[1]] = data.decode('utf-8')
    return result

def source_archive_notices(path):
    rows = []; count = 0; total = 0
    with tarfile.open(path, 'r:gz') as archive:
        for entry in archive:
            count += 1
            if count > 30000:
                raise ValueError('source entry bound')
            total += entry.size
            if total > 256 * 1024 * 1024:
                raise ValueError('source expanded bound')
            basename = entry.name.rsplit('/', 1)[-1].lower()
            if entry.isfile() and (basename.startswith(('copying', 'license', 'licence', 'copyright')) or basename in {'notice', 'notice.txt'}):
                if entry.size > 1024 * 1024 or len(rows) >= 512:
                    raise ValueError('notice bound')
                data = archive.extractfile(entry).read(entry.size+1)
                if len(data) != entry.size:
                    raise ValueError('notice length mismatch')
                rows.append({'member': entry.name, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()})
    return {'entries': count, 'declared_expanded_bytes': total, 'notices': rows, 'filesystem_extraction': False}

def existing_notice_links(mapped):
    commits = {}
    for item in mapped:
        for recipe in item['recipes']:
            for field, value in recipe['declarations'].items():
                token = value.strip('\"\' ')
                if 'COMMIT' in field and re.fullmatch('[a-f0-9]{40}', token):
                    commits.setdefault(token, []).append(item['feature'])
    rows = []; seen = set()
    folders = ['ffmpeg-enabled-notices-8ehqjqje', 'ffmpeg-remaining-notices-lgsdyldh']
    def visit(value, directory, manifest):
        if isinstance(value, dict):
            if all(key in value for key in ('file', 'bytes', 'sha256', 'url')):
                path = directory / value['file']
                url = value['url']
                matching = [commit for commit in commits if commit in url]
                if matching and path.is_file() and path not in seen:
                    observed = digest(path)
                    if observed != {'bytes': value['bytes'], 'sha256': value['sha256']}:
                        raise ValueError('previous exact notice changed')
                    seen.add(path)
                    rows.append({'file': str(path.relative_to(ROOT)), 'manifest': str(manifest.relative_to(ROOT)), 'url': url, **observed, 'matching_revisions': matching, 'windows_features': sorted(set(feature for commit in matching for feature in commits[commit])), 'meaning': 'same recipe revision notice only; actual payload and full source unverified'})
            for nested in value.values():
                visit(nested, directory, manifest)
        elif isinstance(value, list):
            for nested in value:
                visit(nested, directory, manifest)
    for folder in folders:
        directory = ROOT / '.local-tools' / folder
        for manifest in sorted(directory.glob('*.json')):
            if manifest.stat().st_size > 4 * 1024 * 1024:
                raise ValueError('previous manifest bound')
            visit(json.loads(manifest.read_text()), directory, manifest)
    return rows

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('output', type=Path)
    parser.add_argument('--collect', action='store_true')
    args = parser.parse_args()
    output = args.output.absolute()
    if output.exists():
        raise ValueError('fresh evidence required')
    output.mkdir(parents=True, exist_ok=False)
    started = time.monotonic(); deadline = started + 240
    before = {name: digest(ROOT / name) for name in PINNED}
    for name, (sha, size) in PINNED.items():
        if before[name] != {'bytes': size, 'sha256': sha}:
            raise ValueError('pinned source mismatch')
    raw = (ROOT / 'docs/validation/evidence/windows-ffmpeg-native-version-frozen-retry/ffmpeg.log').read_bytes()
    if len(raw) > 65536:
        raise ValueError('native output bound')
    lines = raw.decode('utf-8').splitlines()
    configuration = [line for line in lines if line.startswith('configuration: ')]
    if len(configuration) != 1 or not any('n9.0.2-17-g2a571b6068-20260930' in line for line in lines):
        raise ValueError('exact Windows native version required')
    enabled = re.findall(r'(?<!\S)--enable-([a-zA-Z0-9-]+)(?=\s|$)', configuration[0])
    disabled = re.findall(r'(?<!\S)--disable-([a-zA-Z0-9-]+)(?=\s|$)', configuration[0])
    recipes = archive_recipes(); mapped = []
    for feature in enabled:
        matches = []
        for name, text in recipes.items():
            if not name.startswith('scripts.d/'):
                continue
            if re.search(r'--enable-' + re.escape(feature) + r'(?![a-zA-Z0-9-])', text):
                fields = dict(re.findall(r'^((?:SCRIPT|GNULIB)_[A-Z0-9_]+)=[\"\']?([^\n]+)', text, re.M))
                matches.append({'recipe': name, 'sha256': hashlib.sha256(text.encode()).hexdigest(), 'declarations': fields, 'target_condition_not_evaluated': True})
        mapped.append({'feature': feature, 'recipes': matches, 'actual_payload_provenance_verified': False})
    write(output / 'mapping.json', {'windows_version_log_sha256': hashlib.sha256(raw).hexdigest(), 'configuration': configuration[0], 'enabled': mapped, 'disabled': disabled, 'recipe_commit': BUILD, 'ffmpeg_commit': FFMPEG, 'method': 'literal configure flag, scripts never executed', 'all_source_complete': False})
    links = existing_notice_links(mapped)
    write(output / 'prior-exact-notice-links.json', links)
    selected = ['README.md', 'images/base-win64/Dockerfile', 'images/base-win64/ct-ng-config', 'scripts.d/10-mingw.sh', 'scripts.d/10-mingw-std-threads.sh', 'scripts.d/50-schannel.sh']
    for name in selected:
        with (output / ('recipe-' + name.replace('/', '_'))).open('x') as file:
            file.write(recipes[name])
    fetches = []
    if args.collect:
        requests = [
            ('official-build-readme.txt', f'https://raw.githubusercontent.com/BtbN/FFmpeg-Builds/{BUILD}/README.md', 128*1024),
            ('official-ffmpeg-legal.html', 'https://ffmpeg.org/legal.html', 128*1024),
            ('mingw-std-threads-source.tar.gz', 'https://codeload.github.com/meganz/mingw-std-threads/tar.gz/c931bac289dd431f1dd30fc4a5d1a7be36668073', 4*1024*1024),
            ('mingw-w64-source.tar.gz', 'https://codeload.github.com/mingw-w64/mingw-w64/tar.gz/57b595039040eaa15bece85b7cc71d952281b269', 32*1024*1024),
        ]
        for name, url, cap in requests:
            row = fetch(output, name, url, cap, deadline)
            if row['status'] == 'PASS' and name.endswith('.tar.gz'):
                try:
                    row['source_inventory'] = source_archive_notices(output / name)
                except Exception as error:
                    row['inventory_error'] = type(error).__name__ + ': ' + str(error)[:512]
            fetches.append(row)
    after = {name: digest(ROOT / name) for name in PINNED}
    result = {'scope': 'Windows pinned configure/recipe/source evidence only, no execution or adoption', 'original_before': before, 'original_after': after, 'originals_unchanged': before == after, 'enabled_count': len(enabled), 'disabled_count': len(disabled), 'mapped_literal_count': sum(bool(row['recipes']) for row in mapped), 'fetches': fetches, 'prior_exact_notice_files_verified': len(links), 'all_source_complete': False, 'actual_binary_dependency_provenance_verified': False, 'elapsed_seconds': time.monotonic()-started}
    write(output / 'result.json', result)
    print(json.dumps({key: value for key, value in result.items() if key not in {'original_before', 'original_after', 'fetches'}}))
    return 0 if before == after else 2

if __name__ == '__main__':
    raise SystemExit(main())
