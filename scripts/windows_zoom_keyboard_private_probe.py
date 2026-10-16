#!/usr/bin/env python3
"""Private Windows app session zoom/keyboard observer. No desktop fallback."""
import argparse,hashlib,json,sys
from pathlib import Path
# Published runner imports its sibling; /tmp draft must be explicitly pointed at project scripts.
scripts_dir=Path(__file__).resolve().parent
if not (scripts_dir/'windows_current_workspace_probe.py').is_file():scripts_dir=Path.cwd()/'scripts'
sys.path.insert(0,str(scripts_dir))
import windows_current_workspace_probe as n

def copy_new(original,destination,limit):
    if original.is_symlink() or not original.is_file() or not 0<original.stat().st_size<=limit:
        raise ValueError('bounded original input rejected')
    with original.open('rb') as src,destination.open('xb') as dst:
        total=0
        while True:
            data=src.read(min(65536,limit-total+1))
            if not data:break
            total+=len(data)
            if total>limit:raise ValueError('stream input bound')
            dst.write(data)

def plain(path):
    if not path.is_absolute() or '..'in path.parts or any(p.is_symlink()for p in [path,*path.parents]):raise ValueError('absolute plain path required')

def load(path,limit):
    plain(path)
    if not path.is_file() or path.stat().st_size>limit:raise ValueError('bounded evidence rejected')
    return json.loads(path.read_text(encoding='utf-8-sig'))

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output',type=Path)
    parser.add_argument('--collector',required=True,type=Path)
    parser.add_argument('--binary',required=True,type=Path)
    parser.add_argument('--binary-sha',required=True)
    args=parser.parse_args()
    for p in [args.output,args.collector,args.binary]:plain(p)
    if args.output.exists() or not args.output.is_relative_to(n.ROOT/'docs/validation/evidence'):parser.error('fresh project evidence required')
    if not args.collector.is_relative_to(n.ROOT) or args.collector.suffix!='.cs':parser.error('root published C# collector required')
    if not args.binary.is_relative_to(n.ROOT) or len(args.binary_sha)!=64 or any(c not in '0123456789abcdef'for c in args.binary_sha):parser.error('project immutable binary and exact SHA required')
    for original,limit in [(args.collector,1024*1024),(args.binary,256*1024*1024)]:
        if not original.is_file() or not 0<original.stat().st_size<=limit:raise ValueError('original input size/type bound before hash')
    collector_sha=n.sha(args.collector);runner=Path(__file__).resolve();runner_sha=n.sha(runner)
    if n.sha(args.binary)!=args.binary_sha:raise ValueError('binary SHA mismatch')
    out=args.output;out.mkdir(mode=0o700);(out/'inputs').mkdir(mode=0o700)
    source=out/'inputs/collector.cs';binary=out/'inputs/dlpad-cpu.exe'
    copy_new(args.collector,source,1024*1024);copy_new(args.binary,binary,256*1024*1024)
    if n.sha(source)!=collector_sha or n.sha(binary)!=args.binary_sha:raise ValueError('input copied hash mismatch')
    _,env=n.configure();host=str(n.TOOLS/'windows-cli-host.exe');fw=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319';observer=out/'observer.exe'
    inputs={'binary_sha':args.binary_sha,'collector_sha':collector_sha,'runner_sha':runner_sha,'source':n.snapshot()}
    (out/'inputs.json').write_text(json.dumps(inputs,indent=2)+'\n')
    argv=[host,'45',fw+'\\csc.exe','/nologo','/reference:System.Drawing.dll','/reference:System.Web.Extensions.dll',*['/reference:'+fw+'\\WPF\\'+x+'.dll'for x in ['UIAutomationClient','UIAutomationTypes','WindowsBase']],'/out:'+n.win(observer),n.win(source)]
    ops=[n.run('compile',argv,env,out,45)]
    if ops[-1]['passed']:
        observer_sha=n.sha(observer)
        operation=n.run('gui',[host,'120',n.win(observer),n.win(binary),args.binary_sha,n.win(out/'gui')],env,out,120)
        operation['immutable_inputs_unchanged']=n.sha(binary)==args.binary_sha and n.sha(args.binary)==args.binary_sha and n.sha(source)==collector_sha and n.sha(args.collector)==collector_sha and n.sha(runner)==runner_sha and n.sha(observer)==observer_sha
        operation['passed']=operation['passed']and operation['immutable_inputs_unchanged'];ops.append(operation)
    accepted=False
    native=None
    if (out/'gui/native-result.json').exists():
        native=load(out/'gui/native-result.json',262144)
        accepted=native.get('schema')==1 and native.get('mode')=='session-zoom-keyboard' and native.get('status')=='PASS' and native.get('actual_session_zoom_geometry_verified')is True and native.get('keyboard_phase_initial_focus_count')==1 and native.get('keyboard_phase_pointer_actions')==0 and native.get('keyboard_phase_uia_invoke_actions')==0 and 11<=native.get('keyboard_steps',0)<=64 and native.get('exit_code')==0 and all(native.get(k)is True for k in ['owned_job_naturally_empty','owned_job_empty_after_cleanup','private_desktop_closed','input_desktop_unchanged','sentinel_preserved'])
        pngs=list((out/'gui').glob('keyboard-*.png'))
        if len(pngs)>66 or (accepted and not 11<=len(pngs)<=66):raise ValueError('bounded keyboard PNG inventory')
        for p in pngs:
            plain(p)
            if p.stat().st_size>16*1024*1024 or p.read_bytes()[:8]!=b'\x89PNG\r\n\x1a\n':raise ValueError('original keyboard PNG invalid')
    result={'schema':1,'passed':len(ops)==2 and all(x['passed']for x in ops)and accepted,'operations':ops,'native_acceptance':accepted,'scope':'private synthetic original-HWND keyboard at app zoom200%; not physical keyboard/OS192DPI/screenreader','requires_root_png_focus_review':True}
    (out/'collector-result.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'passed':result['passed'],'evidence':str(out)}),flush=True)
    return 0 if result['passed']else 2
if __name__=='__main__':raise SystemExit(main())
