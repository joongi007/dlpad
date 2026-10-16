"""Draft only: explicit CLI/CPU pins, private native host, copy-phase cancellation.
No installer/registration completion claim; root executes after source freeze coordination.
"""
import argparse, hashlib, importlib, json, os, re, sys
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
    for a in ['project','output','cli','cli-sha','cpu','cpu-sha','observer','proof']:ap.add_argument('--'+a,required=True)
    a=ap.parse_args(); project=path(a.project);out=path(a.output);cli=path(a.cli);cpu=path(a.cpu);observer=path(a.observer)
    if not out.is_relative_to(project/'docs/validation/evidence') or out.exists():raise ValueError('fresh evidence required')
    if not all(re.fullmatch('[0-9a-f]{64}',s)for s in [a.cli_sha,a.cpu_sha]):raise ValueError('SHA syntax')
    if sha(cli)!=a.cli_sha or sha(cpu)!=a.cpu_sha:raise ValueError('input pin mismatch')
    sys.path.insert(0,str(project/'scripts'));n=importlib.import_module('windows_current_workspace_probe')
    if n.ROOT.resolve()!=project.resolve():raise ValueError('helper root mismatch')
    h=importlib.import_module('windows_msvc_run_selected_cli_collect')
    proof=path(a.proof);proofraw=h.bounded(proof);whole=json.loads(proofraw);source=n.snapshot();h.require_current_whole(whole,source)
    for f,key in [(cpu,'dlpad-cpu.exe'),(cli,'dlpad-local-install.exe')]:
        if whole['artifacts'][key]!={'sha256':sha(f),'bytes':f.stat().st_size}:raise ValueError('current whole artifact')
    out.mkdir();inputs=out/'inputs';inputs.mkdir()
    pins={str(proof):hashlib.sha256(proofraw).hexdigest(),str(cli):a.cli_sha,str(cpu):a.cpu_sha,str(observer):sha(observer),str(Path(__file__).absolute()):sha(Path(__file__).absolute())}
    for src,name in [(proof,'whole-proof.json'),(cli,'dlpad-local-install.exe'),(cpu,'dlpad-cpu.exe'),(observer,'observer.cs'),(Path(__file__).absolute(),'collector.py')]:
        initial=src.stat().st_size;count=0
        with src.open('rb') as f, (inputs/name).open('xb') as g:
            while b:=f.read(65536):
                count+=len(b)
                if count>initial:raise ValueError('copy growth')
                g.write(b)
            g.flush();os.fsync(g.fileno())
        if count!=initial:raise ValueError('copy truncation')
        if sha(inputs/name)!=pins[str(src)]:raise ValueError('copy pin mismatch')
        pins[str(inputs/name)]=sha(inputs/name)
    if sha(proof)!=hashlib.sha256(proofraw).hexdigest() or sha(inputs/'whole-proof.json')!=hashlib.sha256(proofraw).hexdigest():raise ValueError('decoded/copied proof changed')
    for f in [project/'Cargo.lock',n.TOOLS/'target/release/dlpad-cpu.exe',n.TOOLS/'target/release/dlpad-local-install.exe']:pins[str(f)]=sha(f)
    _,env=n.configure();host=n.TOOLS/'windows-cli-host.exe'
    pins[str(host)]=sha(host)
    for helper in ['windows_current_workspace_probe.py','windows_cli_host.cs','windows_msvc_run_selected_cli_collect.py','windows_msvc_workspace_probe.py']:
        p=project/'scripts'/helper;pins[str(p)]=sha(p)
        initial=p.stat().st_size;count=0
        with p.open('rb')as f,(inputs/helper).open('xb')as g:
            while b:=f.read(65536):
                count+=len(b)
                if count>initial:raise ValueError('helper copy growth')
                g.write(b)
            g.flush();os.fsync(g.fileno())
        if count!=initial or sha(inputs/helper)!=pins[str(p)]:raise ValueError('helper archive pin')
        pins[str(inputs/helper)]=sha(inputs/helper)
    save(out/'inputs.json',{'source':source,'pins':pins,'cli_sha':a.cli_sha,'cpu_sha':a.cpu_sha,'cpu_executed':False})
    records=[];error=None
    try:
        exe=out/'observer.exe'
        records.append(n.run('compile',[str(host),'45',r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe','/nologo','/reference:System.Web.Extensions.dll','/out:'+n.win(exe),n.win(inputs/'observer.cs')],env,out,45))
        h.guard(records[-1],h.bounded(out/'compile.log'))
        if records[-1]['passed']:
            pins[str(exe)]=sha(exe);save(out/'compiled-pins.json',pins)
            for mode in ['cancel','timeout']:
                if n.snapshot()!=source or any(sha(Path(p))!=h for p,h in pins.items()):raise ValueError('pre-run pins changed')
                records.append(n.run(mode,[str(host),'120',n.win(exe),n.win(inputs/'dlpad-local-install.exe'),a.cli_sha,n.win(out/('owned-'+mode)),mode],env,out,120))
                if not records[-1]['passed']:break
                raw=h.bounded(out/(mode+'.log'));h.guard(records[-1],raw);row=h.jsonrow(raw,'case_name',mode)
                if row.get('status')!='PASS' or row.get('reader_drained')is not True or row.get('reader_failed')is not False or row.get('copy_observed')is not True or row.get('natural_exit_code')!=1 or row.get('forced_cleanup')is not False or row.get('cli_timeout_seconds')!=30:raise ValueError('control typed scope')
                receipt=json.loads(row['cli_receipt'])
                if receipt.get('operation')!='prepare-user-bundle' or receipt.get('phase')!='user-bundle' or receipt.get('bundle_stage')!='prepare' or receipt.get('kind')!=('Interrupted'if mode=='cancel'else'TimedOut'):raise ValueError('actual copy failure stage')
                save(out/(mode+'-receipt.json'),row)
    except Exception as e:error=type(e).__name__
    source_after=n.snapshot();source_same=source_after==source
    try:pins_same=all(sha(Path(p))==h for p,h in pins.items())
    except Exception:pins_same=False
    passed=error is None and len(records)==3 and all(x['passed']for x in records) and source_same and pins_same
    save(out/'collector-result.json',{'passed':passed,'error_type':error,'operations':records,'source_before':source,'source_after':source_after,'source_unchanged':source_same,'input_pins_unchanged':pins_same,'copy_phase_only':True,'operation':'prepare-user-bundle','deadline_seconds':30,'timeout_pause_is_scheduling_only':True,'registration_completion_verified':False,'physical_stall_verified':False,'installer_complete':False,'root_review_pending':True})
    print(json.dumps({'passed':passed,'root_review_pending':True}));return 0 if passed else 2
if __name__=='__main__':raise SystemExit(main())
