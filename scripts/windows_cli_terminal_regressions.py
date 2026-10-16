"""Staged native host regressions; root schedules execution after source freeze."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('output',type=Path)
    p.add_argument('--host-source',type=Path,required=True)
    p.add_argument('--fixture-source',type=Path,required=True)
    a=p.parse_args()
    project=Path.cwd()
    sys.path.insert(0,str(project/'scripts'))
    import windows_current_workspace_probe as n
    out=a.output.absolute()
    if '..' in a.output.parts or out.exists() or not out.is_relative_to(n.ROOT/'docs/validation/evidence') or any(x.is_symlink() for x in [out,*out.parents]):p.error('fresh owned project evidence')
    out.mkdir(mode=0o700)
    _,env=n.configure()
    original_host_sha=n.sha(n.TOOLS/'windows-cli-host.exe')
    records={}
    for kind,source in [('new-host',a.host_source),('fixture',a.fixture_source)]:
        if '..' in source.parts or source.is_symlink() or source.stat().st_size>128*1024:raise ValueError('source bound')
        destination=out/(kind+'.cs')
        with source.open('rb') as src,destination.open('xb') as dst:dst.write(src.read(128*1024+1))
        executable=out/(kind+'.exe')
        records[kind]=n.run('compile-'+kind,[str(n.TOOLS/'windows-cli-host.exe'),'45',r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe','/nologo','/target:exe','/out:'+n.win(executable),n.win(destination)],env,out,45)
        if not records[kind]['passed']:raise RuntimeError('native compile failed; original raw retained')
    # Run NEW host directly, not nested in old host; n.run supplies original stdin
    # lifetime and bounded pipes/owned interop PID. Old host remains unchanged.
    host=out/'new-host.exe';fixture=out/'fixture.exe'
    for name,seconds,mode,expected in [('normal',5,'normal',0),('grandchild',5,'grandchild',125),('timeout',1,'linger',124)]:
        r=n.run(name,[str(host),str(seconds),n.win(fixture),mode],env,out,seconds+15)
        raw=(out/(name+'.log')).read_text(errors='replace')
        records[name]={'exit_code':r['exit_code'],'expected_exit':expected,'terminal_record':next((line for line in raw.splitlines() if line.startswith('dlpad-owned-job-terminal ')),None),'raw':r}
        assert r['exit_code']==expected
        assert 'final-zero=True' in raw and 'final-active=0' in raw and 'private-desktop-closed=True' in raw and 'dlpad-input-desktop-unchanged=True' in raw
        if name=='normal':assert 'natural-zero=True' in raw and 'forced=False' in raw
        if name=='grandchild':assert 'natural-zero=False' in raw and 'forced=True' in raw
    # ORIGINAL host owns the stdin-driver/new-host tree, so driver failure is
    # bounded by original Job kill-on-close even if the draft host misbehaves.
    r=n.run('stdin-cancel',[str(n.TOOLS/'windows-cli-host.exe'),'25',n.win(fixture),'stdin-driver',n.win(host)],env,out,25)
    text=(out/'stdin-cancel.log').read_text(errors='replace')
    assert r['passed'] and 'stdin-cancel=True' in text and 'final-zero=True' in text and 'private-desktop-closed=True' in text and 'dlpad-input-desktop-unchanged=True' in text
    records['stdin-cancel']=r
    records['original_host_sha_before']=original_host_sha
    records['original_host_unchanged']=original_host_sha==n.sha(n.TOOLS/'windows-cli-host.exe')
    with (out/'result.json').open('x') as f:json.dump(records,f,indent=2)
    return 0
if __name__=='__main__':raise SystemExit(main())
