#!/usr/bin/env python3
"""Owned Windows managed execution crash fixture on existing private CLI desktop."""
import argparse, json
from pathlib import Path
import windows_current_workspace_probe as native

def main():
    p=argparse.ArgumentParser();p.add_argument('output',type=Path);a=p.parse_args()
    out=a.output
    if not out.is_absolute() or not out.is_relative_to(native.ROOT/'docs/validation/evidence') or out.exists():p.error('fresh project evidence required')
    source_paths=[native.ROOT/'scripts/windows_managed_execution_crash_observer.cs',Path(__file__).resolve()]
    source_before={str(q.relative_to(native.ROOT)):native.sha(q) for q in source_paths}
    out.mkdir(mode=0o700);values,env=native.configure();host=native.TOOLS/'windows-cli-host.exe'
    plans=[('build',[str(host),'600',native.win(native.TOOLS/'rust/bin/cargo.exe'),'build','-p','dlpad-engine','--example','managed_execution_crash_fixture','--locked'],600),('compile-observer',[str(host),'30',r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe','/nologo','/reference:System.Web.Extensions.dll','/out:'+native.win(out/'observer.exe'),native.win(native.ROOT/'scripts/windows_managed_execution_crash_observer.cs')],30)]
    results={}
    for name,argv,seconds in plans:
        results[name]=native.run(name,argv,env,out,seconds)
        if not results[name]['passed']:break
    if all(r['passed'] for r in results.values()):
        fixture=native.TOOLS/'target/debug/examples/managed_execution_crash_fixture.exe'
        results['native']=native.run('native',[str(host),'60',native.win(out/'observer.exe'),native.win(fixture),native.win(out)],env,out,60)
    passed=len(results)==3 and all(r['passed'] for r in results.values())
    if passed:
        actual=json.loads((out/'native-result.json').read_text());passed=actual['status']=='PASS' and len(actual['cases'])==2
    source_after={str(q.relative_to(native.ROOT)):native.sha(q) for q in source_paths}
    passed=passed and source_before==source_after
    result={'collector_source_before':source_before,'collector_source_after':source_after,'collector_source_unchanged':source_before==source_after,'passed':passed,'operations':results,'observer_source_sha256':native.sha(native.ROOT/'scripts/windows_managed_execution_crash_observer.cs'),'scope':'Windows original process only; no guardian handoff or automatic pending recovery; synthetic roots retained as evidence'}
    (out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':passed}));return 0 if passed else 2
if __name__=='__main__':raise SystemExit(main())
