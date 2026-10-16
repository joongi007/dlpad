#!/usr/bin/env python3
"""Fresh bounded Windows GNU CPU CLI evidence; no visible desktop fallback."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import subprocess
import time
ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / '.local-tools/windows'
MAX_LOG = 8 * 1024 * 1024

def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(65536), b''):h.update(block)
    return h.hexdigest()

def win(path):
    return subprocess.check_output(['wslpath','-w',str(path)],text=True,timeout=3).strip()

def snapshot():
    paths={ROOT/name for name in ['Cargo.toml','Cargo.lock','rust-toolchain.toml','crates/engine/src/tools/staging_schema.sql','scripts/windows_cli_host.cs','scripts/windows_current_workspace_probe.py','.local-tools/windows/windows-cli-host.exe']}
    paths.update((ROOT/'crates').rglob('*.rs'));paths.update((ROOT/'crates').rglob('Cargo.toml'))
    paths.update(p for p in (ROOT/'crates/app/assets').rglob('*') if p.is_file())
    paths.update(p for p in (ROOT/'vendor').rglob('*') if p.is_file())
    return {str(p.relative_to(ROOT)):sha(p) for p in sorted(paths)}

def tools_manifest():
    paths=[TOOLS/'windows-cli-host.exe',TOOLS/'rust/bin/rustc.exe',TOOLS/'rust/bin/cargo.exe',TOOLS/'rust/bin/cargo-fmt.exe',TOOLS/'rust/bin/cargo-clippy.exe',TOOLS/'rust/bin/rustfmt.exe',TOOLS/'llvm-mingw/bin/x86_64-w64-mingw32-gcc.exe',TOOLS/'llvm-mingw/bin/llvm-ar.exe']
    return {str(p.relative_to(ROOT)):{'sha256':sha(p),'bytes':p.stat().st_size} for p in paths}

def configure():
    values={'DLPAD_NATIVE_CONTROL_STDIN':'1','DLPAD_NATIVE_TOOL_PATH':win(TOOLS/'rust/bin')+';'+win(TOOLS/'llvm-mingw/bin'),'CARGO':win(TOOLS/'rust/bin/cargo.exe'),'RUSTC':win(TOOLS/'rust/bin/rustc.exe'),'RUSTFMT':win(TOOLS/'rust/bin/rustfmt.exe'),'RUSTDOC':win(TOOLS/'rust/bin/rustdoc.exe'),'RUSTFLAGS':'-C link-self-contained=yes','CARGO_HOME':win(TOOLS/'cargo-home'),'CARGO_TARGET_DIR':win(TOOLS/'target'),'CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER':win(TOOLS/'llvm-mingw/bin/x86_64-w64-mingw32-gcc.exe'),'CC_x86_64_pc_windows_gnu':win(TOOLS/'llvm-mingw/bin/x86_64-w64-mingw32-gcc.exe'),'AR_x86_64_pc_windows_gnu':win(TOOLS/'llvm-mingw/bin/llvm-ar.exe'),'TMP':win(TOOLS/'tmp'),'TEMP':win(TOOLS/'tmp')}
    env=os.environ.copy();env.update(values)
    env['WSLENV']=env.get('WSLENV','')+':'+':'.join(values)
    return values,env

def own_identity(pid):
    raw=Path(f'/proc/{pid}/stat').read_text()
    return {'pid':pid,'start_ticks':int(raw.rsplit(')',1)[1].split()[19])}

def run(name,argv,env,out,seconds):
    before=snapshot();start=time.monotonic();records={'operation':name,'argv':argv,'timeout_seconds':seconds,'max_log_bytes':MAX_LOG,'source_before':before}
    child=subprocess.Popen(argv,cwd=ROOT,env=env,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    records['owned_interop_host']=own_identity(child.pid)
    selector=selectors.DefaultSelector();selector.register(child.stdout,selectors.EVENT_READ);total=0;tail=bytearray();header_prefix=bytearray();failure=None
    try:
        with (out/(name+'.log')).open('xb') as log:
            while selector.get_map():
                if time.monotonic()-start>seconds+15:raise TimeoutError('owned host outer deadline')
                for key,_ in selector.select(.25):
                    data=os.read(key.fd,65536)
                    if not data:selector.unregister(key.fileobj);continue
                    if total+len(data)>MAX_LOG:raise RuntimeError('native output bound exceeded')
                    log.write(data);total+=len(data);tail.extend(data)
                    # Host emits its ownership header before child output. Long test
                    # output can evict that header from the bounded terminal tail.
                    if len(header_prefix)<4096:header_prefix.extend(data[:4096-len(header_prefix)])
                    if len(tail)>65536:del tail[:-65536]
        code=child.wait(timeout=8)
    except BaseException as error:
        failure=type(error).__name__
        # Native host owns a pinned Job handle; do not kill guessed Windows PIDs.
        try:child.stdin.write(b'C');child.stdin.flush()
        except (BrokenPipeError,OSError):pass
        child.stdin.close()
        try:code=child.wait(timeout=8)
        except subprocess.TimeoutExpired:code=None
    finally:
        selector.close();child.stdout.close()
        if child.stdin and not child.stdin.closed:child.stdin.close()
    after=snapshot();text=tail.decode('utf-8','replace');pids=[int(v) for v in re.findall(r'^dlpad-owned-cli pid=(\d+) private-desktop=true\r?$',header_prefix.decode('utf-8','replace'),re.MULTILINE)]
    records.update(exit_code=code,collector_failure=failure,elapsed_seconds=time.monotonic()-start,log_bytes=total,source_after=after,source_unchanged=before==after,native_child_pids_reported_by_owned_host=pids,private_desktop_reported=bool(pids),input_desktop_unchanged='dlpad-input-desktop-unchanged=True' in text)
    records['passed']=code==0 and failure is None and before==after and bool(pids) and records['input_desktop_unchanged']
    with (out/(name+'.json')).open('x') as f:json.dump(records,f,indent=2);f.write('\n')
    return records

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=Path);p.add_argument('--capability',action='store_true');p.add_argument('--execute',action='store_true',help='root-authorized actual Cargo checks');p.add_argument('--seconds',type=int,default=1800);a=p.parse_args();out=a.output
    if not 1<=a.seconds<=7200:p.error('host timeout out of range')
    if not out.is_absolute() or '..' in out.parts or not out.is_relative_to(ROOT/'docs/validation/evidence') or out.exists() or any(v.is_symlink() for v in [out,*out.parents]):p.error('fresh project evidence required')
    if not (TOOLS/'tmp').is_dir() or not (TOOLS/'cargo-home').is_dir():p.error('existing project-local tool directories required')
    manifest=tools_manifest();values,env=configure();base=snapshot();out.mkdir(mode=0o700)
    bin=TOOLS/'rust/bin';host=TOOLS/'windows-cli-host.exe'
    plans={
        'fmt':[str(host),str(a.seconds),win(bin/'cargo-fmt.exe'),'fmt','--all','--','--check'],
        'clippy':[str(host),str(a.seconds),win(bin/'cargo-clippy.exe'),'clippy','--workspace','--all-targets','--locked','--features','cpu','--','-D','warnings'],
        'test':[str(host),str(a.seconds),win(bin/'cargo.exe'),'test','--workspace','--locked','--features','cpu'],
        'release':[str(host),str(a.seconds),win(bin/'cargo.exe'),'build','--workspace','--release','--locked','--features','cpu'],
    }
    result={'recorded_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'Windows GNU CPU CLI; GUI/IME/MSVC/macOS/Linux keeper runtime unverified','source_baseline':base,'tools':manifest,'fixed_environment':values,'plans':plans,'actual_operations':{}}
    with (out/'plan.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
    if a.capability:
        argv=[str(host),'10',win(bin/'rustc.exe'),'--version','--verbose']
        result['actual_operations']['capability']=run('capability',argv,env,out,10)
    if a.execute and all(v['passed'] for v in result['actual_operations'].values()):
        for name,argv in plans.items():
            print('Windows owned CLI operation='+name,flush=True)
            r=run(name,argv,env,out,a.seconds);result['actual_operations'][name]=r
            if not r['passed']:break
    result['source_final']=snapshot();result['source_unchanged']=base==result['source_final'];result['passed']=result['source_unchanged'] and all(v['passed'] for v in result['actual_operations'].values())
    result['cargo_checks_complete']=a.execute and all(name in result['actual_operations'] and result['actual_operations'][name]['passed'] for name in plans)
    artifact=TOOLS/'target/release/dlpad-cpu.exe'
    if result['cargo_checks_complete'] and artifact.is_file():result['cpu_artifact']={'sha256':sha(artifact),'bytes':artifact.stat().st_size}
    local_install=TOOLS/'target/release/dlpad-local-install.exe'
    if result['cargo_checks_complete'] and local_install.is_file():result['local_install_artifact']={'sha256':sha(local_install),'bytes':local_install.stat().st_size}
    with (out/'result.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
    print(json.dumps({'passed':result['passed'],'cargo_checks_complete':result['cargo_checks_complete'],'actual_operations':list(result['actual_operations'])}));return 0 if result['passed'] else 2
if __name__=='__main__':raise SystemExit(main())
