"""Private original-guard option/help/plan evidence. Root review remains required."""
import argparse
import json
import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
import windows_msvc_run_selected_cli_collect as h
n = h.n

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--proof', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    before = n.snapshot(); proof = h.plain(args.proof)
    whole = json.loads(h.bounded(proof)); h.require_current_whole(whole, before)
    logs = []
    for name, operation in whole['operations'].items():
        path = proof.parent / (name + '.log'); h.guard(operation, h.bounded(path)); logs.append(path)
    cpu = n.TOOLS / 'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release/dlpad-cpu.exe'
    if whole['artifacts']['dlpad-cpu.exe'] != {'sha256':h.sha(cpu),'bytes':cpu.stat().st_size}: raise ValueError('original current CPU')
    out = h.plain(args.output)
    if out.exists() or not out.is_relative_to(n.ROOT / 'docs/validation/evidence') or not out.parent.is_dir(): raise ValueError('fresh project evidence')
    source = Path(__file__).with_name('windows_custom_tool_path_probe.cs')
    host = n.TOOLS / 'windows-cli-host.exe'; config = proof.parent / 'effective-configuration.json'
    pins = {str(p):h.sha(p) for p in [cpu, proof, source, Path(__file__), host, config, Path(h.__file__), Path(n.__file__), *logs]}
    out.mkdir(); inputs=out/'inputs'; inputs.mkdir()
    for original, name in [(cpu,'cpu.exe'),(source,'observer.cs'),(proof,'whole-proof.json'),(config,'effective-configuration.json'),(Path(__file__),'collector.py')]: h.copy(original, inputs/name)
    pins.update({str(p):h.sha(p) for p in inputs.iterdir()})
    _, env=n.configure()
    expected=n.win(n.TOOLS/'tmp')
    if env.get('TEMP')!=expected or env.get('TMP')!=expected: raise ValueError('own Windows TEMP')
    result={'status':'FAIL','operations':[],'pins':pins,'source_before':before,'root_review_pending':True,'actual_download_verified':False,'physical_pointer_verified':False}
    def run(name, argv, seconds):
        operation=n.run(name,[str(host),str(seconds),*argv],env,out,seconds);result['operations'].append(operation);h.guard(operation,h.bounded(out/(name+'.log')))
    try:
        fw=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319'; executable=out/'observer.exe'
        run('compile',[fw+'\\csc.exe','/nologo','/reference:System.Drawing.dll','/reference:System.Web.Extensions.dll',*['/reference:'+fw+'\\WPF\\'+name+'.dll' for name in ['UIAutomationClient','UIAutomationTypes','WindowsBase']],'/out:'+n.win(executable),n.win(inputs/'observer.cs')],60)
        pins[str(executable)]=h.sha(executable)
        run('native-gui',[n.win(executable),n.win(inputs/'cpu.exe'),h.sha(cpu),n.win(out/'native')],720)
        receipt=json.loads(h.bounded(out/'native/custom-tool-path-result.json',65536)); native=receipt['native']
        if receipt['status']!='REVIEW_REQUIRED' or native['status']!='REVIEW_REQUIRED': raise ValueError('actual option evidence')
        for key in ['wm_close_exited','original_handles_close_safe','owned_job_naturally_empty','owned_job_empty_after_cleanup','private_desktop_closed','input_desktop_unchanged','mta_desktop_restored','session_navigation_readback_verified','invalid_relative_plan_rejected_three','custom_literal_argv_verified']:
            if native.get(key) is not True: raise ValueError('exact native guard '+key)
        if native.get('natural_exit_code')!=0 or native.get('owned_job_active_before_cleanup')!=0 or native.get('owned_job_terminated',False): raise ValueError('original natural zero Job0')
        for key in ['actual_download_verified','cookies_used','network_used']:
            if native.get(key) is not False: raise ValueError('bounded synthetic scope')
        rows=native['custom_path_fields']
        labels=['기존 yt-dlp 실행 파일','기존 FFmpeg 실행 파일 (선택)','Deno 실행 파일 (선택 사항)']
        paths=receipt['synthetic_fixture_paths']
        if len(paths)!=3 or len(rows)!=9:raise ValueError('exact3 tool fields/9 edit receipts')
        for index,label in enumerate(labels):
            expected=[(label,paths[index],False),(label,'relative-도구.exe',True),(label,paths[index],False)]
            if [(r['label'],r['value'],r['relative']) for r in rows[index*3:index*3+3]]!=expected:raise ValueError('exact Unicode/long/invalid readback')
        restart=receipt['restart_native']
        for key in ['wm_close_exited','original_handles_close_safe','owned_job_naturally_empty','owned_job_empty_after_cleanup','private_desktop_closed','input_desktop_unchanged','mta_desktop_restored','restart_initial_environment_readback_verified']:
            if restart.get(key) is not True:raise ValueError('restart original natural lifecycle '+key)
        if restart.get('natural_exit_code')!=0 or restart.get('owned_job_active_before_cleanup')!=0 or restart.get('owned_job_terminated',False):raise ValueError('restart original natural zero Job0')
        if restart.get('custom_path_persisted') is not False or receipt.get('custom_persistence_contract_review_pending') is not True:raise ValueError('custom persistence observation scope')
        owned=h.winpath(receipt['owned_root'])
        if not owned.is_relative_to(n.TOOLS/'tmp') or not owned.name.startswith('dlpad-custom-path-owned-'): raise ValueError('own TEMP fixture')
        if h.sha(owned/'sentinel')!=receipt['sentinel_sha256']: raise ValueError('sentinel immutable')
        for raw in paths:
            path=h.winpath(raw)
            if not path.is_relative_to(owned) or h.sha(path)!=receipt['synthetic_fixture_sha256']:raise ValueError('owned synthetic original pin')

        if n.snapshot()!=before or any(h.sha(Path(p))!=sha for p,sha in pins.items()): raise ValueError('original source/pins changed')
        result.update(status='REVIEW_REQUIRED',source_unchanged=True,receipt=receipt)
    finally: h.write(out/'result.json',result)
    print(result['status'])
if __name__=='__main__': main()
