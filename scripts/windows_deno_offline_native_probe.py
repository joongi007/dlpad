"""Pinned Windows Deno offline native checks through the original private CLI Job host."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import windows_current_workspace_probe as n
ROOT = n.ROOT
SUPPLY = ROOT / 'docs/validation/evidence/windows-deno-supply-root-header-fix'
ARCHIVE_SHA = 'a0c3101b4158d1dfb7d6a78a7bf0f3de80c96bb423c152beec8beb22786f2238'
EXE_SHA = 'e020f3e232bd16e33768dee528e5983349c962952051ced0a5d58ad42f5d9b33'


def sha(path):
    if path.stat().st_size > 256 * 1024 * 1024 or path.is_symlink():
        raise ValueError('source bound/type')
    return n.sha(path)


def inventory(root):
    total = 0
    count = 0
    for path in root.rglob('*'):
        count += 1
        if count > 512 or path.is_symlink():
            raise ValueError('cache inventory bound/link')
        if path.is_file():
            total += path.stat().st_size
        if total > 512 * 1024 * 1024:
            raise ValueError('cache observation hard bound')
    return {'entries': count, 'bytes': total, 'within_32mib_budget': total <= 32 * 1024 * 1024}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    out = args.output.absolute()
    if not out.is_relative_to(ROOT / 'docs/validation/evidence') or out.exists() or '..' in out.parts or any(p.is_symlink() for p in [out, *out.parents]):
        parser.error('fresh project evidence required')
    supply = json.loads((SUPPLY / 'result.json').read_text())
    exe = SUPPLY / 'deno.exe'
    archive = SUPPLY / 'deno-x86_64-pc-windows-msvc.zip'
    if (supply['archive']['sha256'] != ARCHIVE_SHA or sha(archive) != ARCHIVE_SHA or archive.stat().st_size != 42630221
            or supply['executable']['sha256'] != EXE_SHA or sha(exe) != EXE_SHA or exe.stat().st_size != 97462048):
        raise ValueError('original supply pins mismatch')
    out.mkdir(mode=0o700)
    for child in ['cache', 'tmp', 'runtime']:
        (out / child).mkdir(mode=0o700)
    values = {'DLPAD_NATIVE_CONTROL_STDIN': '1', 'DENO_DIR': n.win(out / 'cache'),
              'DENO_NO_UPDATE_CHECK': '1', 'DENO_NO_PROMPT': '1', 'NO_COLOR': '1',
              'TMP': n.win(out / 'tmp'), 'TEMP': n.win(out / 'tmp')}
    env = os.environ.copy()
    env.update(values)
    env['WSLENV'] = env.get('WSLENV', '') + ':' + ':'.join(values)
    result = {'version': 1, 'supply_result_sha256': sha(SUPPLY / 'result.json'),
              'archive_sha256': ARCHIVE_SHA, 'exe_sha256_before': sha(exe), 'exe_bytes': exe.stat().st_size,
              'operations': {}, 'peak_rss_bytes': None, 'peak_rss_reason': 'original CLI host does not report pinned child peak RSS',
              'adopted': False, 'passed': False}
    def run(name, arguments):
        r = n.run(name, [str(n.TOOLS / 'windows-cli-host.exe'), '15', n.win(exe), *arguments], env, out, 15)
        result['operations'][name] = {key: value for key, value in r.items() if key not in {'source_before', 'source_after'}}
        if not r['passed']:
            raise RuntimeError('native operation failed: ' + name)
        text = (out / (name + '.log')).read_text(errors='replace')
        return text
    try:
        version = run('version', ['--version'])
        if 'deno 2.9.7' not in version or 'x86_64-pc-windows-msvc' not in version:
            raise ValueError('native version/target mismatch')
        help_text = run('help', ['--help'])
        result['help_license_mentions'] = [line[:256] for line in help_text.splitlines() if 'license' in line.lower()][:16]
        run_help = run('run-help', ['run', '--help'])
        for flag in ['--no-config', '--no-lock', '--no-prompt', '--deny-read', '--deny-net', '--deny-env']:
            if flag not in run_help:
                raise ValueError('native help missing flag: ' + flag)
        sentinel = out / 'runtime/sentinel.txt'
        sentinel.write_bytes(b'owned synthetic read sentinel')
        script = out / 'runtime/pure.mjs'
        script.write_text('''const path = Deno.args[0];
const queries = {};
for (const item of [{name:"read",path},{name:"net",host:"127.0.0.1:9"},{name:"env",variable:"DLPAD_SYNTHETIC_NEVER_READ"}]) {
 queries[item.name]=(await Deno.permissions.query(item)).state;
}
let denied = false;
try { await Deno.readTextFile(path); } catch(e) { denied = e.name === "NotCapable" || e.name === "PermissionDenied"; }
let sum=0; for(let i=1;i<=100000;i++) sum+=i;
const result={sum,queries,owned_read_denied:denied};
console.log("DLPAD_OFFLINE:"+JSON.stringify(result));
if(sum!==5000050000 || !denied || Object.values(queries).some(s=>s!=="denied")) Deno.exit(7);
''', encoding='utf-8')
        text = run('pure-offline', ['run', '--no-config', '--no-lock', '--no-prompt', '--deny-read', '--deny-net', '--deny-env', n.win(script), n.win(sentinel)])
        lines = [line for line in text.splitlines() if line.startswith('DLPAD_OFFLINE:')]
        if len(lines) != 1:
            raise ValueError('missing bounded offline result')
        result['offline'] = json.loads(lines[0].split(':', 1)[1])
        if sentinel.read_bytes() != b'owned synthetic read sentinel':
            raise ValueError('owned sentinel changed')
        result['cache'] = inventory(out / 'cache')
        if not result['cache']['within_32mib_budget']:
            raise ValueError('cache 32MiB budget exceeded')
        result['temporary'] = inventory(out / 'tmp')
        result['exe_sha256_after'] = sha(exe)
        if result['exe_sha256_after'] != EXE_SHA or sha(archive) != ARCHIVE_SHA:
            raise ValueError('original source changed')
        result['passed'] = True
    except Exception as error:
        result['error_type'] = type(error).__name__
        result['error'] = str(error)[:512]
    result['exe_sha256_after'] = sha(exe)
    result['archive_sha256_after'] = sha(archive)
    result['original_bytes_unchanged'] = result['exe_sha256_after'] == EXE_SHA and result['archive_sha256_after'] == ARCHIVE_SHA
    result['exe_links_observed_after'] = exe.stat().st_nlink
    payload = json.dumps(result, ensure_ascii=False, indent=2).encode()
    if len(payload) > 65536:
        raise ValueError('bounded summary exceeded')
    with (out / 'result.json').open('xb') as stream:
        stream.write(payload)
    return 0 if result['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
