"""Fresh Windows Deno copy/node-shim opt-out; original native file identity evidence."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import windows_current_workspace_probe as n
ROOT = n.ROOT
SOURCE = ROOT / 'docs/validation/evidence/windows-deno-supply-root-header-fix/deno.exe'
SHA = 'e020f3e232bd16e33768dee528e5983349c962952051ced0a5d58ad42f5d9b33'
CS = r'''using System;using System.IO;using System.Runtime.InteropServices;using System.Web.Script.Serialization;
class Observe {
[StructLayout(LayoutKind.Sequential)]struct Standard{public long AllocationSize;public long EndOfFile;public uint NumberOfLinks;public byte DeletePending;public byte Directory;}
[StructLayout(LayoutKind.Sequential)]struct Identity{public ulong VolumeSerialNumber;[MarshalAs(UnmanagedType.ByValArray,SizeConst=16)]public byte[] FileId;}
[DllImport("kernel32.dll",SetLastError=true)]static extern bool GetFileInformationByHandleEx(IntPtr h,int c,out Standard s,uint n);
[DllImport("kernel32.dll",SetLastError=true)]static extern bool GetFileInformationByHandleEx(IntPtr h,int c,out Identity s,uint n);
static int Main(string[] args){try{if(args.Length!=1)return 2;using(var f=new FileStream(args[0],FileMode.Open,FileAccess.Read,FileShare.ReadWrite|FileShare.Delete)){
Standard s;Identity i;if(!GetFileInformationByHandleEx(f.SafeFileHandle.DangerousGetHandle(),1,out s,(uint)Marshal.SizeOf(typeof(Standard)))||!GetFileInformationByHandleEx(f.SafeFileHandle.DangerousGetHandle(),18,out i,(uint)Marshal.SizeOf(typeof(Identity))))return 3;
if(s.Directory!=0||s.DeletePending!=0)return 4;Console.WriteLine("DLPAD_NATIVE_FILE:"+new JavaScriptSerializer().Serialize(new{bytes=s.EndOfFile,links=s.NumberOfLinks,volume=i.VolumeSerialNumber,file_id=BitConverter.ToString(i.FileId).Replace("-","")}));return 0;}}catch{return 5;}}
}'''
JS = '''const path=Deno.args[0];const queries={};
for(const item of [{name:"read",path},{name:"net",host:"127.0.0.1:9"},{name:"env",variable:"DLPAD_SYNTHETIC_NEVER_READ"}])queries[item.name]=(await Deno.permissions.query(item)).state;
let denied=false;try{await Deno.readTextFile(path)}catch(e){denied=e.name==="NotCapable"||e.name==="PermissionDenied"}
let sum=0;for(let i=1;i<=100000;i++)sum+=i;
console.log("DLPAD_OFFLINE:"+JSON.stringify({sum,queries,owned_read_denied:denied}));
if(sum!==5000050000||!denied||Object.values(queries).some(s=>s!=="denied"))Deno.exit(7);
'''


def text_create(path, text):
    with path.open('x', encoding='utf-8') as f:
        f.write(text)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    out = parser.parse_args().output.absolute()
    if not out.is_relative_to(ROOT / 'docs/validation/evidence') or out.exists() or '..' in out.parts or any(p.is_symlink() for p in [out, *out.parents]):
        parser.error('fresh owned evidence required')
    if SOURCE.is_symlink() or SOURCE.stat().st_size != 97462048 or n.sha(SOURCE) != SHA:
        raise ValueError('original executable pin mismatch')
    out.mkdir(mode=0o700)
    for child in ['managed-copy', 'cache', 'tmp', 'runtime']:
        (out / child).mkdir(mode=0o700)
    exe = out / 'managed-copy/deno.exe'
    h = hashlib.sha256()
    size = 0
    with SOURCE.open('rb') as source, exe.open('xb') as dest:
        while block := source.read(65536):
            size += len(block)
            if size > 97462048:
                raise ValueError('copy bound')
            h.update(block)
            dest.write(block)
    if size != 97462048 or h.hexdigest() != SHA:
        raise ValueError('copy hash mismatch')
    values = {'DLPAD_NATIVE_CONTROL_STDIN': '1', 'DENO_DIR': n.win(out / 'cache'),
              'DENO_DISABLE_NODE_SHIM': '1', 'DENO_NO_UPDATE_CHECK': '1', 'DENO_NO_PROMPT': '1',
              'NO_COLOR': '1', 'TMP': n.win(out / 'tmp'), 'TEMP': n.win(out / 'tmp')}
    env = os.environ.copy()
    env.update(values)
    env['WSLENV'] = env.get('WSLENV', '') + ':' + ':'.join(values)
    report = {'version': 1, 'exe_sha256': SHA, 'source_revision': '0c071246a412575e07423263404a5d13e7ed6aa2',
              'explicit_node_shim_optout': True, 'operations': {}, 'passed': False,
              'ejs_verified': False, 'product_guardian_environment_verified': False, 'peak_rss_bytes': None}
    def run(name, binary, arguments):
        r = n.run(name, [str(n.TOOLS / 'windows-cli-host.exe'), '20', binary, *arguments], env, out, 20)
        report['operations'][name] = {k: v for k, v in r.items() if k not in {'source_before', 'source_after'}}
        if not r['passed']:
            raise RuntimeError('owned native operation failed: ' + name)
        return (out / (name + '.log')).read_text(errors='replace')
    def observe(name):
        text = run(name, n.win(out / 'observer.exe'), [n.win(exe)])
        rows = [line.split(':', 1)[1] for line in text.splitlines() if line.startswith('DLPAD_NATIVE_FILE:')]
        if len(rows) != 1:
            raise ValueError('native file observation missing')
        value = json.loads(rows[0])
        if value['links'] != 1 or value['bytes'] != 97462048:
            raise ValueError('native link/size policy failed')
        return value
    try:
        text_create(out / 'observer.cs', CS)
        run('compile-observer', r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe',
            ['/nologo', '/reference:System.Web.Extensions.dll', '/out:' + n.win(out / 'observer.exe'), n.win(out / 'observer.cs')])
        report['native_before'] = observe('native-before')
        version = run('version', n.win(exe), ['--version'])
        if 'deno 2.9.7 (stable, release, x86_64-pc-windows-msvc)' not in version:
            raise ValueError('version mismatch')
        sentinel = out / 'runtime/sentinel.txt'
        with sentinel.open('xb') as f:
            f.write(b'owned synthetic read sentinel')
        script = out / 'runtime/pure.mjs'
        text_create(script, JS)
        text = run('pure-offline', n.win(exe), ['run', '--no-config', '--no-lock', '--no-prompt',
            '--deny-read', '--deny-net', '--deny-env', n.win(script), n.win(sentinel)])
        rows = [line.split(':', 1)[1] for line in text.splitlines() if line.startswith('DLPAD_OFFLINE:')]
        if len(rows) != 1:
            raise ValueError('offline result missing')
        report['offline'] = json.loads(rows[0])
        report['native_after'] = observe('native-after')
        if report['native_before'] != report['native_after']:
            raise ValueError('native original identity changed')
        count = total = 0
        for p in (out / 'cache').rglob('*'):
            count += 1
            if count > 512 or p.is_symlink():
                raise ValueError('cache entry/link bound')
            if p.is_file():
                total += p.stat().st_size
        report['cache'] = {'entries': count, 'logical_bytes': total, 'budget_bytes': 32 * 1024 * 1024}
        if total > 32 * 1024 * 1024 or (out / 'cache/node_compat_bin/node.exe').exists():
            raise ValueError('node shim/cache budget failed')
        if sentinel.read_bytes() != b'owned synthetic read sentinel':
            raise ValueError('owned sentinel changed')
        report['passed'] = True
    except Exception as error:
        report['error_type'] = type(error).__name__
        report['error'] = str(error)[:512]
    report['source_sha256_after'] = n.sha(SOURCE)
    report['copy_sha256_after'] = n.sha(exe)
    if report['source_sha256_after'] != SHA or report['copy_sha256_after'] != SHA:
        report['passed'] = False
        report['error'] = 'original bytes changed'
    payload = json.dumps(report, indent=2).encode()
    if len(payload) > 65536:
        raise ValueError('summary bound')
    with (out / 'result.json').open('xb') as f:
        f.write(payload)
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
