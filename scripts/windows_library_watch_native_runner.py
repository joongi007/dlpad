#!/usr/bin/env python3
"""Staged current default CPU watch-only GUI collection; root executes later."""
import argparse
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value

def plain(path):
    if not path.is_absolute() or '..' in path.parts or any(p.is_symlink() for p in (path, *path.parents)):
        raise ValueError('absolute no-symlink input required')
    return path

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
    n=module('dlpad_watch_native',ROOT/'scripts/windows_current_workspace_probe.py')
    package=module('dlpad_watch_proof',ROOT/'scripts/windows_candidate_package.py')
    if n.ROOT.resolve()!=ROOT.resolve():raise ValueError('fixed project mismatch')
    binary=plain(a.binary);proof=plain(a.checks_proof);out=plain(a.output)
    if not binary.is_relative_to(ROOT) or not proof.is_relative_to(ROOT):raise ValueError('project inputs required')
    if out.exists() or not out.is_relative_to(ROOT/'docs/validation/evidence'):raise ValueError('fresh evidence required')
    checked=package.checks(proof);artifact=checked['cpu_artifact']
    if a.sha!=artifact['sha256'] or n.sha(binary)!=a.sha or binary.stat().st_size!=artifact['bytes']:raise ValueError('current default CPU proof mismatch')
    source=ROOT/'scripts/windows_library_watch_probe.cs'
    originals=[source,binary,proof,Path(__file__).resolve(),ROOT/'scripts/windows_current_workspace_probe.py',ROOT/'scripts/windows_candidate_package.py']
    original_pins={str(p):n.sha(plain(p)) for p in originals};baseline=n.snapshot()
    out.mkdir(mode=0o700);inputs=out/'inputs';inputs.mkdir(mode=0o700)
    cpu=inputs/'dlpad-cpu.exe';cs=inputs/'collector.cs'
    copy_new(binary,cpu,268435456);copy_new(source,cs,1048576)
    if n.sha(cpu)!=a.sha or n.sha(cs)!=original_pins[str(source)]:raise ValueError('immutable copy mismatch')
    observer=out/'observer.exe';ops=[];native={};error=None;compiled={}
    _,env=n.configure();host=str(n.TOOLS/'windows-cli-host.exe');fw=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319'
    try:
        refs=['/reference:System.Drawing.dll','/reference:System.Web.Extensions.dll']+['/reference:'+fw+'\\WPF\\'+x+'.dll' for x in ['UIAutomationClient','UIAutomationTypes','WindowsBase']]
        ops.append(n.run('compile',[host,'45',fw+'\\csc.exe','/nologo']+refs+['/out:'+n.win(observer),n.win(cs)],env,out,45))
        if not ops[-1]['passed']:raise ValueError('compile failed')
        compiled={str(p):n.sha(p) for p in [cpu,cs,observer]}
        (out/'compiled-pins.json').write_text(json.dumps(compiled,indent=2)+'\n')
        gui=out/'gui'
        ops.append(n.run('gui',[host,'180',n.win(observer),n.win(cpu),a.sha,n.win(gui)],env,out,180))
        native=read_json(gui/'native-result.json',262144)
        expected={'schema':1,'mode':'watch-only-library','status':'PASS','exit_code':0,'wm_close_exited':True,'owned_job_naturally_empty':True,'owned_job_empty_after_cleanup':True,'private_desktop_closed':True,'input_desktop_unchanged':True,'mta_desktop_restored':True,'original_handles_close_safe':True,'initial_stable_samples':10,'initial_stable_interval_ms':100,'fixture_files_created':10000,'loaded_first_page_actual_row':True,'watch_only_refresh_verified':True,'manual_refresh_invoked':False,'library_reentry_invoked':False,'rename_native_file_id_preserved':True,'sentinel_preserved':True,'no_download_invoked':True,'binary_sha256_before':a.sha,'binary_sha256_after':a.sha}
        for key,value in expected.items():
            if type(native.get(key)) is not type(value) or native[key]!=value:raise ValueError('native field rejected '+key)
        if any(native.get(k) is True for k in ['owned_job_terminated','owned_job_terminated_for_worker_drain','original_handles_retained_until_process_exit']):raise ValueError('forced cleanup cannot pass')
        for key in ['pid','start_filetime','hwnd']:
            if type(native.get(key)) is not int or native[key]<=0:raise ValueError('original identity missing')
        if not all(op['passed'] for op in ops):raise ValueError('native host failure')
        if not all(n.sha(Path(p))==sha for p,sha in compiled.items()):raise ValueError('compiled pins changed')
    except Exception as exc:error=type(exc).__name__+': '+str(exc)
    unchanged=all(n.sha(Path(p))==sha for p,sha in original_pins.items()) and n.snapshot()==baseline
    collected=error is None and unchanged
    result={'schema':1,'status':'REVIEW_REQUIRED' if collected else 'FAIL','native_checks_passed':collected,'acceptance_complete':False,'root_recovery_verified':False,'pixel_review_complete':False,'full_db_inventory_and_payload_preservation_verified':False,'original_pins':original_pins,'compiled_pins':compiled,'source_before':baseline,'source_after':n.snapshot(),'source_unchanged':unchanged,'operations':ops,'native':native,'error':error,'scope':'default GUI watch-only add/delete/rename; root recovery and paced-scan preemption are separate','limitations':['collector subscription directly observed=false preserved','원 HWND PNG actual glyph 및 DB rename/10k 논리 rows는 root 추가 대조 필요','UIA 호출 자체 취소는 보장하지 않으며 bounded join 실패 시 원 HANDLE을 종료까지 보존한다']}
    (out/'runner-result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'status':result['status'],'evidence':str(out)}))
    return 0 if collected else 2

if __name__=='__main__':raise SystemExit(main())
