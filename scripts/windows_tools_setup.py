"""Install pinned native Windows GNU validation tools into project-only .local-tools/windows."""
import concurrent.futures
import hashlib
import json
import pathlib
import re
import shutil
import tarfile
import urllib.request
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1] / '.local-tools' / 'windows'
ROOT.mkdir(parents=True, exist_ok=True)
CACHE = ROOT / 'archives'
CACHE.mkdir(exist_ok=True)
MANIFEST_URL = 'https://static.rust-lang.org/dist/channel-rust-1.94.0.toml'
LLVM_URL = 'https://github.com/mstorsjo/llvm-mingw/releases/download/20260922/llvm-mingw-20260922-msvcrt-x86_64.zip'
LLVM_HASH = '1e936a4a694fc27f9625e3311f5ec5d6d99abfeaa514fd41c52a4f8347145d1a'
LLVM_SIZE = 173053724


def request(url):
    return urllib.request.urlopen(urllib.request.Request(url, headers={'User-Agent': 'dlpad-local-validation'}), timeout=60)


with request(MANIFEST_URL) as response:
    manifest = response.read(2 * 1024 * 1024)
with request(MANIFEST_URL + '.sha256') as response:
    expected_manifest = response.read(1024).decode().split()[0]
if hashlib.sha256(manifest).hexdigest() != expected_manifest:
    raise RuntimeError('Rust manifest hash mismatch')
(CACHE / 'channel-rust-1.94.0.toml').write_bytes(manifest)
artifacts = []
for package in ['rustc', 'cargo', 'rust-std', 'rustfmt-preview', 'clippy-preview', 'rust-mingw']:
    section = re.search(r'^\[pkg\.' + re.escape(package) + r'\.target\.x86_64-pc-windows-gnu\]\n(.*?)(?=^\[|\Z)', manifest.decode(), re.M | re.S).group(1)
    url = re.search(r'^xz_url = "([^"]+)"', section, re.M).group(1)
    digest = re.search(r'^xz_hash = "([^"]+)"', section, re.M).group(1)
    artifacts.append({'package': package, 'url': url, 'sha256': digest})
artifacts.append({'package': 'llvm-mingw', 'url': LLVM_URL, 'sha256': LLVM_HASH, 'expected_size': LLVM_SIZE})


def fetch(artifact):
    target = CACHE / artifact['url'].rsplit('/', 1)[-1]
    if not target.exists():
        partial = target.with_suffix(target.suffix + '.partial')
        digest = hashlib.sha256()
        length = 0
        with request(artifact['url']) as response, partial.open('wb') as output:
            while block := response.read(1024 * 1024):
                length += len(block)
                if length > 300 * 1024 * 1024:
                    raise RuntimeError('Tool archive size limit exceeded')
                digest.update(block)
                output.write(block)
        if digest.hexdigest() != artifact['sha256']:
            raise RuntimeError('Tool archive hash mismatch: ' + target.name)
        if artifact.get('expected_size', length) != length:
            raise RuntimeError('Tool archive size mismatch')
        partial.rename(target)
    digest = hashlib.sha256()
    with target.open('rb') as archived:
        while block := archived.read(1024 * 1024):
            digest.update(block)
    if digest.hexdigest() != artifact['sha256']:
        raise RuntimeError('Cached tool archive hash mismatch')
    artifact['size'] = target.stat().st_size
    print(json.dumps({'downloaded': target.name, 'bytes': artifact['size'], 'sha256': artifact['sha256']}), flush=True)
    return artifact, target


def safe_path(base, name):
    name = name.replace('\\', '/')
    if name.startswith('/') or ':' in name:
        raise RuntimeError('Unsafe tool archive path')
    destination = (base / name).resolve()
    if base.resolve() not in destination.parents and destination != base.resolve():
        raise RuntimeError('Escaping tool archive path')
    return destination


with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
    fetched = list(pool.map(fetch, artifacts))
previous = {}
if (ROOT / 'tool-evidence.json').exists():
    previous = {a['package']: a['sha256'] for a in json.loads((ROOT / 'tool-evidence.json').read_text())['artifacts']}
for artifact, archive in fetched:
    if previous.get(artifact['package']) == artifact['sha256']:
        continue
    if artifact['package'] == 'llvm-mingw':
        base = ROOT / 'llvm-mingw'
        base.mkdir(exist_ok=True)
        with zipfile.ZipFile(archive) as bundle:
            for member in bundle.infolist():
                # Strip the single published archive root; preserve license files.
                name = member.filename.split('/', 1)
                if len(name) != 2 or not name[1]:
                    continue
                destination = safe_path(base, name[1])
                if member.is_dir():
                    destination.mkdir(parents=True, exist_ok=True)
                else:
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    with bundle.open(member) as source, destination.open('wb') as output:
                        shutil.copyfileobj(source, output, 1024 * 1024)
    else:
        base = ROOT / 'rust'
        base.mkdir(exist_ok=True)
        with tarfile.open(archive, 'r:xz') as bundle:
            for member in bundle:
                parts = member.name.split('/', 2)
                if len(parts) < 3:
                    # Root package licenses and source version are preserved separately.
                    if member.isfile() and len(parts) == 2:
                        destination = safe_path(ROOT / 'licenses' / artifact['package'], parts[1])
                    else:
                        continue
                else:
                    if parts[1] not in ('rustc', 'cargo', 'rustfmt-preview', 'clippy-preview', 'rust-std-x86_64-pc-windows-gnu', 'rust-mingw'):
                        continue
                    destination = safe_path(base, parts[2])
                if member.isdir():
                    destination.mkdir(parents=True, exist_ok=True)
                elif member.isfile():
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    with bundle.extractfile(member) as source, destination.open('wb') as output:
                        shutil.copyfileobj(source, output, 1024 * 1024)
                else:
                    raise RuntimeError('Unsupported link in tool archive')
(ROOT / 'tool-evidence.json').write_text(json.dumps({'rust_manifest_url': MANIFEST_URL, 'rust_manifest_sha256': expected_manifest, 'llvm_release_api': 'https://api.github.com/repos/mstorsjo/llvm-mingw/releases/tags/20260922', 'artifacts': artifacts}, indent=2) + '\n')
print(json.dumps({'installed': str(ROOT), 'download_bytes': sum(a['size'] for a in artifacts), 'installed_bytes': sum(p.stat().st_size for p in ROOT.rglob('*') if p.is_file())}), flush=True)
