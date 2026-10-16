"""Collect official pinned bundled EJS and upstream local challenge evidence."""
import argparse
import json
import hashlib
import os
import tarfile
from pathlib import Path
import zipfile
from windows_deno_supply_probe import fetch
import windows_current_workspace_probe as n
from windows_deno_node_shim_optout_probe import CS, SOURCE, SHA
EJS_REV = "4fb477f4af56880cfd324c48bd4294a2d2294e50"
ZIP_SHA = "1fa6733c37ea6fb51c99ad8fe785e7b7e5f3246c9b980230329d4fb72ed8d4d6"
TAG = '2026.08.19'


def execute(out, report, scripts):
    if SOURCE.is_symlink() or n.sha(SOURCE) != SHA:
        raise ValueError('original Deno pin mismatch')
    for name in ['cache', 'tmp', 'runtime']:
        (out / name).mkdir(mode=0o700)
    exe = out / 'runtime/deno.exe'
    with SOURCE.open('rb') as source, exe.open('xb') as target:
        total = 0
        while block := source.read(65536):
            total += len(block)
            if total > 97462048:
                raise ValueError('Deno copy bound')
            target.write(block)
    if n.sha(exe) != SHA:
        raise ValueError('Deno copy mismatch')
    values = {'DLPAD_NATIVE_CONTROL_STDIN': '1', 'DENO_DIR': n.win(out / 'cache'),
              'DENO_DISABLE_NODE_SHIM': '1', 'DENO_NO_UPDATE_CHECK': '1', 'DENO_NO_PROMPT': '1',
              'NO_COLOR': '1', 'TMP': n.win(out / 'tmp'), 'TEMP': n.win(out / 'tmp')}
    env = os.environ.copy()
    env.update(values)
    env['WSLENV'] = env.get('WSLENV', '') + ':' + ':'.join(values)
    report['operations'] = {}
    def run(name, binary, arguments):
        r = n.run(name, [str(n.TOOLS / 'windows-cli-host.exe'), '20', binary, *arguments], env, out, 20)
        report['operations'][name] = {k: v for k, v in r.items() if k not in {'source_before', 'source_after'}}
        if not r['passed']:
            raise ValueError('native operation failed: ' + name)
        return (out / (name + '.log')).read_text(errors='replace')
    report['native_executed'] = True
    with (out / 'observer.cs').open('x') as output:
        output.write(CS)
    run('compile-observer', r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe',
        ['/nologo', '/reference:System.Web.Extensions.dll', '/out:' + n.win(out / 'observer.exe'), n.win(out / 'observer.cs')])
    def observe(label):
        text = run(label, n.win(out / 'observer.exe'), [n.win(exe)])
        rows = [line.split(':', 1)[1] for line in text.splitlines() if line.startswith('DLPAD_NATIVE_FILE:')]
        if len(rows) != 1:
            raise ValueError('native Deno observation missing')
        value = json.loads(rows[0])
        if value['links'] != 1 or value['bytes'] != 97462048:
            raise ValueError('native Deno link/size failed')
        return value
    report['native_before'] = observe('native-before')
    input_data = {'type': 'preprocessed', 'preprocessed_player': '_result.n=(s)=>s.split("").reverse().join(""); _result.sig=(s)=>s.slice(1);',
                  'requests': [{'type': 'n', 'challenges': ['dlpad-synthetic']}, {'type': 'sig', 'challenges': ['fixture-signature']}, {'type': 'invalid', 'challenges': ['x']}]}
    suffix = 'const input=' + json.dumps(input_data) + ';const actual=jsc(input);'
    suffix += 'const missing=jsc({type:"preprocessed",preprocessed_player:"",requests:[{type:"sig",challenges:["x"]}]});'
    suffix += 'console.log("DLPAD_EJS:"+JSON.stringify({actual,missing}));'
    suffix += 'if(actual.type!=="result"||actual.responses[0].data["dlpad-synthetic"]!=="citehtnys-dapld"||actual.responses[1].data["fixture-signature"]!=="ixture-signature"||actual.responses[2].type!=="error"||missing.responses[0].type!=="error")Deno.exit(7);'
    script = out / 'runtime/solver.mjs'
    with script.open('x', encoding='utf-8') as output:
        output.write(scripts[0] + '\nObject.assign(globalThis,lib);\n' + scripts[1] + '\n' + suffix)
    report['fixture'] = {'origin': 'selfauthored local preprocessed API input; not upstream player fixture', 'sha256': hashlib.sha256(json.dumps(input_data).encode()).hexdigest()}
    report['combined_js_sha256'] = n.sha(script)
    text = run('bundled-ejs', n.win(exe), ['run', '--no-config', '--no-lock', '--no-prompt', '--deny-read', '--deny-net', '--deny-env', n.win(script)])
    rows = [line.split(':', 1)[1] for line in text.splitlines() if line.startswith('DLPAD_EJS:')]
    if len(rows) != 1:
        raise ValueError('EJS response missing')
    report['ejs_response'] = json.loads(rows[0])
    report['native_after'] = observe('native-after')
    if report['native_before'] != report['native_after']:
        raise ValueError('native identity changed')
    count = size = 0
    for path in (out / 'cache').rglob('*'):
        count += 1
        if count > 512 or path.is_symlink():
            raise ValueError('cache entry bound')
        if path.is_file():
            size += path.stat().st_size
    report['cache_logical_bytes'] = size
    if size > 32 * 1024 * 1024 or (out / 'cache/node_compat_bin/node.exe').exists():
        raise ValueError('cache/node shim bound')
    report['bundled_sha256_after'] = {item['name']: n.sha(out / item['name']) for item in report['bundled']}
    if any(report['bundled_sha256_after'][item['name']] != item['sha256'] for item in report['bundled']):
        raise ValueError('bundled original changed')
    report['copy_sha256_after'] = n.sha(exe)
    report['source_sha256_after'] = n.sha(SOURCE)
    if report['copy_sha256_after'] != SHA or report['source_sha256_after'] != SHA or n.sha(script) != report['combined_js_sha256']:
        raise ValueError('original/combined bytes changed')
    report['native_executed'] = True
    report['passed'] = True
    report['status'] = 'bundled-preprocessed-api-pass-not-player-extraction'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--execute', action='store_true')
    args = parser.parse_args()
    out = args.output.absolute()
    if not out.is_relative_to(n.ROOT / 'docs/validation/evidence') or out.exists() or '..' in out.parts or any(p.is_symlink() for p in [out, *out.parents]):
        parser.error('fresh owned evidence required')
    out.mkdir(parents=True, exist_ok=False)
    report = {'version': 1, 'tag': TAG, 'native_executed': False, 'passed': False}
    try:
        report['release'] = fetch('https://api.github.com/repos/yt-dlp/yt-dlp/releases/tags/' + TAG, out / 'release.json', 2 * 1024 * 1024)
        release = json.loads((out / 'release.json').read_text())
        assets = [a for a in release['assets'] if a['name'] == 'yt-dlp']
        if len(assets) != 1 or assets[0]['size'] > 8 * 1024 * 1024:
            raise ValueError('official zipapp inventory bound')
        a = assets[0]
        report['zipapp'] = fetch(a['browser_download_url'], out / 'yt-dlp.zipapp', 8 * 1024 * 1024)
        if report['zipapp']['sha256'] != ZIP_SHA or report['zipapp']['bytes'] != a['size'] or 'sha256:' + report['zipapp']['sha256'] != a['digest']:
            raise ValueError('official zipapp pin mismatch')
        with zipfile.ZipFile(out / 'yt-dlp.zipapp') as z:
            entries = z.infolist()
            if len(entries) > 5000:
                raise ValueError('zipapp entry bound')
            report['ejs_entries'] = [{'name': m.filename, 'bytes': m.file_size} for m in entries if 'ejs' in m.filename.lower()]
        report['ejs_revision'] = EJS_REV
        report['ejs_commit'] = fetch('https://api.github.com/repos/yt-dlp/ejs/commits/0.8.0', out / 'ejs-commit.json', 1024 * 1024)
        if json.loads((out / 'ejs-commit.json').read_text())['sha'] != EJS_REV:
            raise ValueError('upstream EJS tag revision mismatch')
        report['ejs_source'] = fetch('https://codeload.github.com/yt-dlp/ejs/tar.gz/' + EJS_REV, out / 'ejs-source.tar.gz', 8 * 1024 * 1024)
        report['upstream_fixture_gap'] = 'solver tests require separately downloaded real YouTube player JS; not fetched'
        report['source_members'] = []
        with tarfile.open(out / 'ejs-source.tar.gz', 'r|gz') as source:
            count = 0
            for member in source:
                count += 1
                if count > 4096:
                    raise ValueError('source entry bound')
                if member.isfile() and member.name.endswith(('/solver/main.ts', '/solver/solvers.ts', '/solver/solvers.test.ts', '/solver/test/tests.ts', '/solver/test/download.ts')):
                    if member.size > 65536:
                        raise ValueError('source member bound')
                    data = source.extractfile(member).read(65537)
                    filename = 'source-%d.txt' % len(report['source_members'])
                    with (out / filename).open('xb') as output:
                        output.write(data)
                    report['source_members'].append({'origin': member.name, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'file': filename})
        report['bundled'] = []
        scripts = []
        with zipfile.ZipFile(out / 'yt-dlp.zipapp') as bundle:
            version = bundle.read('yt_dlp_ejs/_version.py').decode()
            if "version = '0.8.0'" not in version:
                raise ValueError('bundled EJS version mismatch')
            for name in ['lib.min.js', 'core.min.js']:
                data = bundle.read('yt_dlp_ejs/yt/solver/' + name)
                if len(data) > 256 * 1024:
                    raise ValueError('bundled JS size bound')
                with (out / name).open('xb') as output:
                    output.write(data)
                scripts.append(data.decode())
                report['bundled'].append({'name': name, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()})
        report['status'] = 'prepared-only'
        if args.execute:
            execute(out, report, scripts)

    except Exception as e:
        report['status'] = 'failed'
        report['error_type'] = type(e).__name__
        report['error'] = str(e)[:512]
    payload = json.dumps(report, indent=2).encode()
    if len(payload) > 65536:
        raise ValueError('summary bound')
    with (out / 'result.json').open('xb') as f:
        f.write(payload)
    return 1 if report['status'] == 'failed' else 0


if __name__ == '__main__':
    raise SystemExit(main())
