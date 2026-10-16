#!/usr/bin/env python3
"""Fixed official Deno notice transaction in an owned Windows private desktop."""
import argparse
import json
from pathlib import Path
import windows_current_workspace_probe as n

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('output',type=Path)
    a=ap.parse_args()
    out=a.output
    if not out.is_absolute() or not out.is_relative_to(n.ROOT/'docs/validation/evidence') or out.exists() or '..' in out.parts or any(p.is_symlink() for p in [out,*out.parents]):
        ap.error('fresh project evidence directory required')
    out.mkdir(mode=0o700)
    base=n.snapshot(); tools=n.tools_manifest(); _,env=n.configure()
    host=str(n.TOOLS/'windows-cli-host.exe'); cargo=n.win(n.TOOLS/'rust/bin/cargo.exe')
    records=[n.run('build',[host,'600',cargo,'build','-p','dlpad-engine','--release','--locked','--example','windows_official_deno_notice_fixture'],env,out,600)]
    receipt=None
    if records[0]['passed']:
        exe=n.TOOLS/'target/release/examples/windows_official_deno_notice_fixture.exe'
        pin={'sha256':n.sha(exe),'bytes':exe.stat().st_size}
        records.append(n.run('official-https',[host,'200',n.win(exe),'--fixed-notice-experiment'],env,out,200))
        raw=(out/'official-https.log').read_text(encoding='utf-8',errors='replace')
        values=[json.loads(line) for line in raw.splitlines() if line.startswith('{')]
        if len(values)==1: receipt=values[0]
        if n.sha(exe)!=pin['sha256']:raise ValueError('original experiment executable changed')
    else:pin=None
    final=n.snapshot()
    flags=['fresh_fullset_verified','tampered_notice_rejected','package_removed','sentinel_preserved']
    passed=base==final and tools==n.tools_manifest() and len(records)==2 and all(r['passed'] for r in records) and isinstance(receipt,dict) and receipt.get('status')=='PASS' and all(receipt.get(k) is True for k in flags) and all(receipt.get(k) is False for k in ['native_tool_executed','activated','supply_adopted'])
    result={'status':'PASS' if passed else 'FAIL','scope':'official fixed HTTPS ZIP plus root LICENSE transaction; no runtime activation/trust/transitive notice completeness','records':records,'receipt':receipt,'source_baseline':base,'source_final':final,'source_unchanged':base==final,'tools':tools,'experiment_executable':pin}
    (out/'result.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'status':result['status'],'operations':[r['operation'] for r in records],'receipt':receipt}))
    return 0 if passed else 2
if __name__=='__main__':raise SystemExit(main())
