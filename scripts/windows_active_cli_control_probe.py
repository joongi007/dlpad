"""Draft only: explicit CLI/CPU pins, private native host, ready-event precreation cancellation/deadline.
No installer/registration completion claim; root executes after source freeze coordination.
"""
import argparse, hashlib, importlib, json, os, re, shutil, sys
from pathlib import Path

def sha(path, cap=134217728):
    before=path.stat(); total=0; h=hashlib.sha256()
    if not path.is_file() or path.is_symlink() or before.st_size>cap: raise ValueError('pin bounds')
    with path.open('rb') as f:
        while True:
            b=f.read(min(65536, cap-total+1))
            if not b: break
            total+=len(b)
            if total>cap or total>before.st_size: raise ValueError('growing pin')
            h.update(b)
    after=path.stat()
    if total!=before.st_size or (before.st_dev,before.st_ino,before.st_size)!=(after.st_dev,after.st_ino,after.st_size):raise ValueError('pin changed')
    return h.hexdigest()

def path(raw):
    p=Path(raw)
    if not p.is_absolute() or '..' in p.parts: raise ValueError('absolute nonparent required')
    for a in [p,*p.parents]:
        if a.is_symlink():raise ValueError('symlink')
    return p

def save(p,value):
    data=(json.dumps(value,indent=2)+'\n').encode()
    if len(data)>1048576:raise ValueError('receipt bound')
    with p.open('xb') as f:f.write(data)

def main():
    ap=argparse.ArgumentParser()
    for a in ['project','output','cli','cli-sha','cpu','cpu-sha','observer','registered','receipt']:ap.add_argument('--'+a,required=True)
    a=ap.parse_args(); registered=path(a.registered);receipt=path(a.receipt); project=path(a.project);out=path(a.output);cli=path(a.cli);cpu=path(a.cpu);observer=path(a.observer)
    if not out.is_relative_to(project/'docs/validation/evidence') or out.exists():raise ValueError('fresh evidence required')
    if not all(re.fullmatch('[0-9a-f]{64}',s)for s in [a.cli_sha,a.cpu_sha]):raise ValueError('SHA syntax')
    if sha(cli)!=a.cli_sha or sha(cpu)!=a.cpu_sha:raise ValueError('input pin mismatch')
    sys.path.insert(0,str(project/'scripts'));n=importlib.import_module('windows_current_workspace_probe')
    if n.ROOT.resolve()!=project.resolve():raise ValueError('helper root mismatch')
    out.mkdir();inputs=out/'inputs';inputs.mkdir()
    source=n.snapshot();pins={str(cli):a.cli_sha,str(cpu):a.cpu_sha,str(observer):sha(observer),str(Path(__file__).absolute()):sha(Path(__file__).absolute())}
    for src,name in [(cli,'dlpad-local-install.exe'),(cpu,'dlpad-cpu.exe'),(observer,'observer.cs'),(Path(__file__).absolute(),'collector.py')]:
        initial=src.stat().st_size;total=0
        if initial>134217728:raise ValueError('copy initial bound')
        with src.open('rb') as f, (inputs/name).open('xb') as g:
            while chunk:=f.read(65536):
                total+=len(chunk)
                if total>initial:raise ValueError('copy grew before write')
                g.write(chunk)
            if total!=initial or src.stat().st_size!=initial:raise ValueError('copy short/changed size')
            g.flush();os.fsync(g.fileno())
        if sha(inputs/name)!=pins[str(src)]:raise ValueError('copy pin mismatch')
        pins[str(inputs/name)]=sha(inputs/name)
    _,env=n.configure();host=n.TOOLS/'windows-cli-host.exe'
    pins[str(host)]=sha(host)
    for helper in ['windows_current_workspace_probe.py','windows_cli_host.cs']:
        p=project/'scripts'/helper;pins[str(p)]=sha(p)
    save(out/'inputs.json',{'source':source,'pins':pins,'cli_sha':a.cli_sha,'cpu_sha':a.cpu_sha,'cpu_executed':False})
    records=[];error=None
    try:
        exe=out/'observer.exe'
        records.append(n.run('compile',[str(host),'45',r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe','/nologo','/reference:System.Web.Extensions.dll','/out:'+n.win(exe),n.win(inputs/'observer.cs')],env,out,45))
        if records[-1]['passed']:
            pins[str(exe)]=sha(exe);save(out/'compiled-pins.json',pins)
            for mode in ['cancel','timeout']:
                if n.snapshot()!=source or any(sha(Path(p))!=h for p,h in pins.items()):raise ValueError('pre-run pins changed')
                records.append(n.run(mode,[str(host),'120',n.win(exe),n.win(inputs/'dlpad-local-install.exe'),a.cli_sha,n.win(out/('owned-'+mode)),mode,n.win(registered),n.win(receipt)],env,out,120))
                if not records[-1]['passed']:break
    except Exception as e:error=type(e).__name__
    source_same=n.snapshot()==source
    try:pins_same=all(sha(Path(p))==h for p,h in pins.items())
    except Exception:pins_same=False
    passed=error is None and len(records)==3 and all(x['passed']for x in records) and source_same and pins_same
    save(out/'collector-result.json',{'passed':passed,'error_type':error,'operations':records,'source_unchanged':source_same,'input_pins_unchanged':pins_same,'ready_ack_precreation':True,'release_event_signalled':False,'active_selection_completion_verified':False,'physical_stall_verified':False,'installer_complete':False,'root_review_pending':True})
    print(json.dumps({'passed':passed,'root_review_pending':True}));return 0 if passed else 2
if __name__=='__main__':raise SystemExit(main())
