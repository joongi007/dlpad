"""Draft only: explicit CLI/CPU pins, private native host, copy-phase cancellation.
No installer/registration completion claim; root executes after source freeze coordination.
"""
import argparse, hashlib, importlib, json, os, re, sys
from pathlib import Path

def sha(p,cap=134217728):
    sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'scripts'))
    h=importlib.import_module('windows_msvc_run_selected_cli_collect')
    if p.stat().st_size>cap:raise ValueError('specific SHA cap')
    return h.sha(p)

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
    rawlogs=[]
    for name,op in whole['operations'].items():
        log=path(proof.parent/(name+'.log'));h.guard(op,h.bounded(log));rawlogs.append(log)
    config=path(proof.parent/'effective-configuration.json');h.bounded(config,65536)
    for f,key in [(cpu,'dlpad-cpu.exe'),(cli,'dlpad-local-install.exe')]:
        if whole['artifacts'][key]!={'sha256':sha(f),'bytes':f.stat().st_size}:raise ValueError('current whole artifact')
    out.mkdir();inputs=out/'inputs';inputs.mkdir()
    pins={str(proof):hashlib.sha256(proofraw).hexdigest(),str(cli):a.cli_sha,str(cpu):a.cpu_sha,str(observer):sha(observer),str(Path(__file__).absolute()):sha(Path(__file__).absolute())}
    for f in [config,*rawlogs]:pins[str(f)]=sha(f)
    for src,name in [(config,'effective-configuration.json'),(proof,'whole-proof.json'),(cli,'dlpad-local-install.exe'),(cpu,'dlpad-cpu.exe'),(observer,'observer.cs'),(Path(__file__).absolute(),'collector.py')]:
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
                records.append(n.run(mode,[str(host),'120',n.win(exe),n.win(inputs/'dlpad-local-install.exe'),a.cli_sha,n.win(out/('owned-'+mode)),mode,n.win(inputs/'dlpad-cpu.exe'),a.cpu_sha],env,out,120))
                if not records[-1]['passed']:break
                raw=h.bounded(out/(mode+'.log'));h.guard(records[-1],raw);row=h.jsonrow(raw,'case_name',mode)
                if row.get('status')!='PASS' or row.get('reader_drained')is not True or row.get('reader_failed')is not False or row.get('copy_observed')is not True or row.get('natural_exit_code')!=1 or row.get('forced_cleanup')is not False or row.get('cli_timeout_seconds')!=30:raise ValueError('control typed scope')
                receipt=json.loads(row['cli_receipt'])
                if receipt.get('operation')!='prepare-journal-user-bundle' or receipt.get('phase')!='journal-user-bundle' or receipt.get('kind')!=('Interrupted'if mode=='cancel'else'TimedOut'):raise ValueError('actual copy failure stage')
                if any(row.get(k)is not True for k in ['journal_exact_intent_prefix_preserved','current_cpu_preserved','synthetic_source_derived_from_current_cpu','source_is_padded_to_128MiB','natural_job_zero','input_desktop_unchanged','private_console_closed','copied_prefix_matches','source_preserved','foreign_preserved']) or row.get('current_cpu_sha256')!=a.cpu_sha:raise ValueError('journal/current-source control preservation')
                if any(receipt.get(k)is not True for k in ['mutation_may_have_occurred','bundle_capabilities_present_before_drop','journal_capabilities_present_before_drop']) or receipt.get('retained_file_caps','missing')is not None or receipt.get('retained_file_cap_count_reported')is not False:raise ValueError('actual partial capability facts')
                save(out/(mode+'-receipt.json'),row)
    except Exception as e:error=type(e).__name__
    source_after=n.snapshot();source_same=source_after==source
    try:pins_same=all(sha(Path(p))==h for p,h in pins.items())
    except Exception:pins_same=False
    passed=error is None and len(records)==3 and all(x['passed']for x in records) and source_same and pins_same
    save(out/'collector-result.json',{'passed':passed,'error_type':error,'operations':records,'source_before':source,'source_after':source_after,'source_unchanged':source_same,'input_pins_unchanged':pins_same,'copy_phase_only':True,'operation':'prepare-journal-user-bundle','deadline_seconds':30,'timeout_pause_is_scheduling_only':True,'registration_completion_verified':False,'status':'REVIEW_REQUIRED' if passed else 'FAIL','historical_ownership':False,'deletion_authority':False,'partial_journal_scope':'intent-only immutable prefix, no authority reconstruction','physical_stall_verified':False,'installer_complete':False,'root_review_pending':True})
    print(json.dumps({'passed':passed,'root_review_pending':True}));return 0 if passed else 2
if __name__=='__main__':raise SystemExit(main())
