#!/usr/bin/env python3
"""Collect current default CPU synthetic background evidence in a private desktop."""
import argparse
import importlib.util
import json
import hashlib
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
# Published script location defines the project; nested helper imports use its scripts.
sys.path.insert(0, str(ROOT / 'scripts'))

def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value

def plain(path):
    if not path.is_absolute() or '..' in path.parts or any(p.is_symlink() for p in (path, *path.parents)):
        raise ValueError('absolute no-symlink input required')
    return path

def read_bounded(path,maximum):
    plain(path)
    with path.open('rb') as stream:
        raw=stream.read(maximum+1)
    if len(raw)>maximum:raise ValueError('raw bound')
    return raw

def read_json(path, maximum=8*1024*1024):
    with path.open('rb') as stream:
        data=stream.read(maximum+1)
    if len(data)>maximum:
        raise ValueError('JSON input bound')
    return json.loads(data.decode('utf-8-sig'))

def copy_new(source, destination, maximum):
    with source.open('rb') as src, destination.open('xb') as dst:
        total=0
        while True:
            data=src.read(min(65536,maximum-total+1))
            if not data:break
            total+=len(data)
            if total>maximum:raise ValueError('immutable copy bound')
            dst.write(data)

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--sha',required=True)
    parser.add_argument('--checks-proof',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    a=parser.parse_args()
    n=module('dlpad_native_background',ROOT/'scripts/windows_current_workspace_probe.py')
    whole_guard=module('dlpad_msvc_background_proof',ROOT/'scripts/windows_msvc_workspace_probe.py')
    terminal_guard=module('dlpad_background_terminal_guard',ROOT/'scripts/windows_selected_gui_collect.py')
    if n.ROOT.resolve()!=ROOT.resolve():raise ValueError('fixed project mismatch')
    binary=plain(a.binary);proof=plain(a.checks_proof);out=plain(a.output)
    if not binary.is_relative_to(ROOT) or not proof.is_relative_to(ROOT):raise ValueError('project immutable inputs only')
    if out.exists() or not out.is_relative_to(ROOT/'docs/validation/evidence'):raise ValueError('fresh project evidence only')
    baseline=n.snapshot()
    proof_raw=read_bounded(proof,8*1024*1024)
    checked=json.loads(proof_raw.decode('utf-8-sig'))
    whole_guard.require_current_whole(checked,baseline)
    for name,operation in checked['operations'].items():
        terminal_guard.guard(operation,read_bounded(proof.parent/(name+'.log'),8*1024*1024))
    artifact=checked['artifacts']['dlpad-cpu.exe']
    if artifact['sha256']!=a.sha or n.sha(binary)!=a.sha or binary.stat().st_size!=artifact['bytes']:raise ValueError('exact current default artifact mismatch')
    if n.snapshot()!=baseline:raise ValueError('source changed after proof validation')
    originals=[ROOT/'scripts/windows_cpu_background_native_probe.cs',ROOT/'scripts/windows_cpu_background_worker.cs',Path(__file__).resolve(),ROOT/'scripts/windows_current_workspace_probe.py',ROOT/'scripts/windows_msvc_workspace_probe.py',ROOT/'scripts/windows_selected_gui_collect.py',proof,binary]
    pins={str(p):n.sha(plain(p)) for p in originals}
    out.mkdir(mode=0o700);inputs=out/'inputs';inputs.mkdir(mode=0o700)
    copy_new(proof,inputs/'whole-proof.json',8*1024*1024)
    if n.sha(inputs/'whole-proof.json')!=hashlib.sha256(proof_raw).hexdigest() or n.sha(proof)!=hashlib.sha256(proof_raw).hexdigest():raise ValueError('decoded proof copy pin')
    copy_new(Path(__file__).resolve(),inputs/'collector.py',1048576)
    pins[str(inputs/'whole-proof.json')]=n.sha(inputs/'whole-proof.json')
    pins[str(inputs/'collector.py')]=n.sha(inputs/'collector.py')
    cpu=inputs/'dlpad-cpu.exe';observer_source=inputs/'observer.cs';worker_source=inputs/'worker.cs'
    for src,dst,limit in [(binary,cpu,268435456),(originals[0],observer_source,1048576),(originals[1],worker_source,1048576)]:
        copy_new(src,dst,limit)
        if n.sha(src)!=n.sha(dst):raise ValueError('input copy differs')
    records=[];native={};error=None;observer=out/'observer.exe';worker=out/'worker.exe'
    _,env=n.configure();host=str(n.TOOLS/'windows-cli-host.exe');fw=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319'
    try:
        common=[host,'45',fw+'\\csc.exe','/nologo']
        records.append(n.run('compile-worker',common+['/out:'+n.win(worker),n.win(worker_source)],env,out,45))
        refs=['/reference:System.Drawing.dll','/reference:System.Web.Extensions.dll']+['/reference:'+fw+'\\WPF\\'+x+'.dll' for x in ['UIAutomationClient','UIAutomationTypes','WindowsBase']]
        records.append(n.run('compile-observer',common+refs+['/out:'+n.win(observer),n.win(observer_source)],env,out,45))
        for operation in records:
            terminal_guard.guard(operation,read_bounded(out/(operation['operation']+'.log'),8*1024*1024))
        if not all(r['passed'] for r in records):raise ValueError('native compilation failed')
        compiled={str(p):n.sha(p) for p in (cpu,observer,worker,observer_source,worker_source)}
        (out/'compiled-pins.json').write_text(json.dumps(compiled,indent=2)+'\n')
        gui=out/'gui'
        records.append(n.run('gui',[host,'180',n.win(observer),n.win(cpu),a.sha,n.win(gui),n.win(worker),compiled[str(worker)]],env,out,180))
        native=read_json(gui/'native-result.json',262144)
        expected={'schema':1,'mode':'active-background','status':'PASS','exit_code':0,'wm_close_exited':True,'owned_job_naturally_empty':True,'owned_job_empty_after_cleanup':True,'private_desktop_closed':True,'input_desktop_unchanged':True,'mta_desktop_restored':True,'worker_in_owned_job':True,'owned_process_exited_after_job_close':True,'owned_job_active_before_cleanup':0,'original_window_hidden':True,'original_window_restored':True,'original_worker_naturally_exited':True,'hidden_continued_same_worker_restored':True,'secondary_exit_code':0,'sentinel_preserved':True,'binary_sha256_before':a.sha,'binary_sha256_after':a.sha,'worker_sha256_before':compiled[str(worker)],'worker_sha256_after':compiled[str(worker)]}
        for key,value in expected.items():
            if type(native.get(key)) is not type(value) or native[key]!=value:raise ValueError('exact native acceptance field '+key)
        for key in ['pid','start_filetime','worker_pid','worker_creation','secondary_pid','secondary_creation']:
            if type(native.get(key)) is not int or native[key]<=0:raise ValueError('original identity evidence missing')
        if type(native.get('heartbeat_before')) is not int or type(native.get('heartbeat_after')) is not int or native['heartbeat_after']<=native['heartbeat_before']:raise ValueError('hidden heartbeat failed')
        if native.get('owned_job_terminated') is True or native.get('timeout_owned_job_terminated') is True or native.get('handles_retained_until_host_exit') is True:raise ValueError('forced cleanup cannot pass')
        if not all(n.sha(Path(p))==sha for p,sha in compiled.items()):raise ValueError('compiled/input pins changed')
        for operation in records:
            terminal_guard.guard(operation,read_bounded(out/(operation['operation']+'.log'),8*1024*1024))
        if not all(r['passed'] for r in records):raise ValueError('host operation failed')
    except Exception as exc:
        error=type(exc).__name__+': '+str(exc)
    unchanged=all(n.sha(Path(p))==sha for p,sha in pins.items()) and n.snapshot()==baseline
    collected=error is None and unchanged
    result={'schema':1,'status':'REVIEW_REQUIRED' if collected else 'FAIL','native_checks_passed':collected,'acceptance_complete':False,'pixel_review_complete':False,'job_attempt_db_preservation_verified':False,'original_pins':pins,'source_before':baseline,'source_after':n.snapshot(),'source_inputs_unchanged':unchanged,'operations':records,'target':'x86_64-pc-windows-msvc','whole_proof_sha256':hashlib.sha256(proof_raw).hexdigest(),'cpu_artifact':artifact,'hidden_close_ack_verified':False,'metrics_stale_recovery_verified':False,'native':native,'error':error,'limits':['실제 PNG 내용 및 same job/attempt DB 논리 비교는 root 추가 검토 필요','native 수명 검사만 전체 파일/DB 보존 PASS로 확대하지 않음']}
    (out/'runner-result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'status':result['status'],'evidence':str(out)}))
    return 0 if collected else 2

if __name__=='__main__':raise SystemExit(main())
