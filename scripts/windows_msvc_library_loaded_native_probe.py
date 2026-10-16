"""Staged exact Windows private loaded-library acceptance; native host owns cleanup."""
import argparse
import importlib.util
import json
import hashlib
from pathlib import Path
import sqlite3
import sys
import os
import time


def plain(path):
    if not path.is_absolute() or '..' in path.parts or any(p.is_symlink() for p in (path,*path.parents)):
        raise ValueError('absolute no-symlink path required')
    return path


def main():
    parser=argparse.ArgumentParser()
    for name in ('project','binary','output'):
        parser.add_argument('--'+name,type=Path,required=True)
    parser.add_argument('--sha',required=True)
    parser.add_argument('--checks-proof',type=Path,required=True)
    a=parser.parse_args()
    project=plain(a.project)
    module=project/'scripts/windows_current_workspace_probe.py'
    spec=importlib.util.spec_from_file_location('dlpad_native_loaded',module)
    native=importlib.util.module_from_spec(spec);spec.loader.exec_module(native)
    if native.ROOT.resolve()!=project.resolve():raise ValueError('native root mismatch')
    sys.path.insert(0,str(project/'scripts'))
    from windows_msvc_workspace_probe import require_current_whole
    from windows_selected_gui_collect import guard
    proof_path=plain(a.checks_proof)
    if not proof_path.is_relative_to(project):raise ValueError('project whole proof only')
    with proof_path.open('rb') as stream:proof_raw=stream.read(8*1024*1024+1)
    if len(proof_raw)>8*1024*1024:raise ValueError('whole proof bound')
    whole=json.loads(proof_raw)
    require_current_whole(whole,native.snapshot())
    for name,op in whole['operations'].items():
        with (proof_path.parent/(name+'.log')).open('rb') as stream:raw=stream.read(8*1024*1024+1)
        if len(raw)>8*1024*1024:raise ValueError('whole raw bound')
        guard(op,raw)
    out=plain(a.output);binary=plain(a.binary);collector=out/'observer.exe';source=plain(project/'scripts/windows_library_loaded_probe.cs')
    if out.exists() or not out.is_relative_to(project/'docs/validation/evidence'):raise ValueError('fresh project evidence required')
    if not binary.is_relative_to(project):raise ValueError('owned binary location required')
    if len(a.sha)!=64 or any(c not in '0123456789abcdef' for c in a.sha) or native.sha(binary)!=a.sha:raise ValueError('frozen binary mismatch')
    for p in (binary,source):
        if not p.is_file() or p.stat().st_size>268435456:raise ValueError('bounded original input required')
    if whole['artifacts']['dlpad-cpu.exe']!={'sha256':a.sha,'bytes':binary.stat().st_size}:raise ValueError('current MSVC CPU artifact mismatch')
    before=native.snapshot();
    if before!=whole['source_after']:raise ValueError('source changed since default proof')
    proof_sha=hashlib.sha256(proof_raw).hexdigest()
    observers={str(p):native.sha(p) for p in (source,Path(__file__),module,proof_path,project/'scripts/windows_msvc_workspace_probe.py',project/'scripts/windows_selected_gui_collect.py')}
    out.mkdir(mode=0o700)
    (out/'inputs-before.json').write_text(json.dumps({'source':before,'observers':observers,'binary_sha256':a.sha},indent=2)+'\n')
    with (out/'whole-proof.json').open('xb') as stream:stream.write(proof_raw)
    if native.sha(proof_path)!=proof_sha:raise ValueError('decoded proof changed')
    values,env=native.configure()
    inputs=out/'inputs';inputs.mkdir(mode=0o700)
    frozen_source=inputs/'observer.cs'
    with source.open('rb') as src,frozen_source.open('xb') as dst:
        count=0
        while True:
            chunk=src.read(min(65536,1048576-count+1))
            if not chunk:break
            count+=len(chunk)
            if count>1048576:raise ValueError('observer source copy bound')
            dst.write(chunk)
    if native.sha(frozen_source)!=observers[str(source)]:raise ValueError('observer source copy pin')
    framework=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319'
    refs=['/reference:System.Drawing.dll','/reference:System.Web.Extensions.dll']
    refs+=['/reference:'+framework+'\\WPF\\'+name+'.dll' for name in ['UIAutomationClient','UIAutomationTypes','WindowsBase']]
    compile_operation=native.run('compile-observer',[str(native.TOOLS/'windows-cli-host.exe'),'45',framework+'\\csc.exe','/nologo','/codepage:65001',*refs,'/out:'+native.win(collector),native.win(frozen_source)],env,out,45)
    try:
        with (out/'compile-observer.log').open('rb') as stream:compile_raw=stream.read(8*1024*1024+1)
        if len(compile_raw)>8*1024*1024:raise ValueError('compile raw bound')
        guard(compile_operation,compile_raw)
    except Exception as error:
        (out/'result.json').write_text(json.dumps({'status':'FAIL','failure_scope':'observer-compile','error':str(error),'operations':[compile_operation],'source_before':before,'source_after':native.snapshot()},indent=2)+'\n')
        return 2
    if not collector.is_file() or collector.stat().st_size>16*1024*1024:raise ValueError('compiled observer bound')
    compiled_pins={str(collector):native.sha(collector),str(frozen_source):native.sha(frozen_source)}
    observers.update(compiled_pins)
    (out/'compiled-pins.json').write_text(json.dumps(compiled_pins,indent=2)+'\n')
    argv=[str(native.TOOLS/'windows-cli-host.exe'),'180',native.win(collector),native.win(binary),a.sha,native.win(out/'gui')]
    operation=native.run('native',argv,env,out,180)
    native_path=out/'gui/native-result.json';proof={};review={};failure=None
    try:
        with (out/'native.log').open('rb') as stream:raw=stream.read(8*1024*1024+1)
        if len(raw)>8*1024*1024:raise ValueError('native raw bound')
        guard(operation,raw)
        if native_path.is_file() and native_path.stat().st_size<=262144:proof=json.loads(native_path.read_text(encoding='utf-8-sig'))
        if proof.get('owned_job_terminated') is True or proof.get('timeout_owned_job_terminated') is True:raise ValueError('forced cleanup')
        required={'schema':1,'mode':'loaded-library','status':'PASS','exit_code':0,'wm_close_exited':True,'owned_job_naturally_empty':True,'owned_job_empty_after_cleanup':True,'private_desktop_closed':True,'input_desktop_unchanged':True,'sentinel_preserved':True,'loaded_first_page_actual_row':True,'loaded_second_page_actual_row':True,'manual_add_rename_delete_actual_rows':True,'invalid_root_preserved_rows':True,'binary_sha256_before':a.sha,'binary_sha256_after':a.sha}
        if any(type(proof.get(k)) is not type(v) or proof.get(k)!=v for k,v in required.items()):raise ValueError('exact native evidence rejected')
        raw=proof.get('owned_fixture_root')
        if not isinstance(raw,str) or len(raw)>248:raise ValueError('owned fixture path bound')
        import subprocess
        owned=plain(Path(subprocess.check_output(['wslpath','-u',raw],text=True,timeout=3).strip()))
        if owned.parent.resolve()!=(native.TOOLS/'tmp').resolve() or not owned.name.startswith('dlpad-library-owned-'):raise ValueError('fixture confinement rejected')
        review_started=time.monotonic()
        media=plain(owned/'media')
        expected={f'media-{i:05d}.mp4' for i in range(10000)}-{'media-00000.mp4','media-00001.mp4'}|{'media-00000-renamed.mp4','media-00000-added.mp4'}
        hashes={}
        with os.scandir(media) as entries:
            for entry in entries:
                if time.monotonic()-review_started>=120:raise ValueError('fixture review deadline')
                if len(hashes)>=10000:raise ValueError('fixture entry count exceeds bound')
                if entry.is_symlink() or not entry.is_file(follow_symlinks=False) or entry.name not in expected:raise ValueError('unexpected fixture entry')
                fd=os.open(entry.path,os.O_RDONLY|os.O_CLOEXEC|os.O_NOFOLLOW)
                with os.fdopen(fd,'rb') as file:actual=file.read(4)
                payload=bytes([7,8,9]) if entry.name=='media-00000-added.mp4' else bytes([1,2,3])
                if actual!=payload:raise ValueError('fixture payload changed')
                hashes[entry.name]=hashlib.sha256(actual).hexdigest()
        if set(hashes)!=expected or (owned/'foreign-sentinel').read_bytes()!=b'dlpad-library-sentinel-v1':raise ValueError('fixture names/sentinel changed')
        (out/'fixture-hashes.json').write_text(json.dumps(hashes,indent=2)+'\n')
        database=plain(owned/'data/library.sqlite')
        if not database.is_file() or database.stat().st_size>64*1024*1024:raise ValueError('bounded original DB required')
        with sqlite3.connect(database.as_uri()+'?mode=ro',uri=True) as db:
            db.execute('PRAGMA query_only=ON')
            if db.execute('PRAGMA integrity_check').fetchall()!=[('ok',)]:raise ValueError('DB integrity check')
            def normalize(value):
                if not isinstance(value,str) or len(value)>32768:raise ValueError('DB root path rejected')
                if value.startswith('\\\\?\\'):value=value[4:]
                return value.replace('\\','/').rstrip('/').casefold()
            expected_root=normalize(native.win(media))
            roots=db.execute('SELECT DISTINCT root FROM library_entries LIMIT 2').fetchall()
            if len(roots)!=1 or normalize(roots[0][0])!=expected_root:raise ValueError('DB exact scoped root mismatch')
            original_root=roots[0][0]
            count=db.execute('SELECT COUNT(*) FROM library_entries WHERE root=? AND present=1',(original_root,)).fetchone()[0]
            rows=db.execute('SELECT path,size,file_identity FROM library_entries WHERE root=? AND present=1 ORDER BY path LIMIT 129',(original_root,)).fetchall()
        names=[str(row[0]).replace('\\','/').rsplit('/',1)[-1] for row in rows]
        if count!=10000 or not {'media-00000-added.mp4','media-00000-renamed.mp4'}.issubset(names) or 'media-00001.mp4' in names:raise ValueError('read-only DB actual change mismatch')
        review={'count':count,'first129':rows,'fixture_payloads_preserved':True,'sentinel_preserved':True,'db_sha256':native.sha(database)}
    except Exception as error:failure=type(error).__name__+': '+str(error)
    unchanged=native.snapshot()==before and all(native.sha(Path(p))==sha for p,sha in observers.items()) and native.sha(binary)==a.sha
    result={'schema':1,'status':'REVIEW_REQUIRED' if operation['passed'] and unchanged and failure is None else 'FAIL','operations':[compile_operation,operation],'operation':operation,'native':proof,'review':review,'review_error':failure,'source_unchanged':unchanged,'observers_before':observers,'fixed_environment':values,'target':'x86_64-pc-windows-msvc','whole_proof_sha256':proof_sha,'pixel_acceptance_complete':False,'hidden_close_ack_verified':False,'scope':'Windows synthetic loaded library; watcher-only/scan-closewait/native rename identity remain separate'}
    (out/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    return 0 if result['status']=='REVIEW_REQUIRED' else 2


if __name__=='__main__':sys.exit(main())
