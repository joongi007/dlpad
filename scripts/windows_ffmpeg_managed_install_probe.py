#!/usr/bin/env python3
"""Official pinned Windows ZIP install in temporary synthetic manager roots."""
import argparse,json
from pathlib import Path
import windows_current_workspace_probe as native

def main():
    parser=argparse.ArgumentParser();parser.add_argument('output',type=Path);args=parser.parse_args();out=args.output
    if not out.is_absolute() or not out.is_relative_to(native.ROOT/'docs/validation/evidence') or out.exists() or any(p.is_symlink() for p in [out,*out.parents]):parser.error('fresh project evidence required')
    out.mkdir(mode=0o700)
    supply=native.ROOT/'.local-tools/ffmpeg-win64-inventory-f5uj2z2m'
    inventory=json.loads((supply/'manifest.json').read_text());archive=supply/inventory['asset']
    if native.sha(archive)!=inventory['sha256'] or archive.stat().st_size!=inventory['bytes']:raise ValueError('original archive mismatch')
    components=[];mappings=[]
    for c in inventory['components']:
        name=c['file'];role='Executable' if name=='ffmpeg.exe' else 'Auxiliary' if name=='ffprobe.exe' else 'Notice'
        components.append(dict(name=name,bytes=c['bytes'],sha256=c['sha256'],role=role));mappings.append(dict(member=c['member'],component=name))
    package=dict(version=2,tool='ffmpeg',tool_version='n9.0.2-17-g2a571b6068-20260930',target='x86_64-pc-windows-gnu',source_url=inventory['url'],license='LGPL-3.0-or-later; supply adoption pending',ownership='Managed',archive_bytes=inventory['bytes'],archive_sha256=inventory['sha256'],components=components,layout={'Zip':dict(root=inventory['asset'][:-4],mappings=mappings)})
    manifest=out/'package.json';manifest.write_text(json.dumps(package,indent=2)+'\n')
    _,env=native.configure();host=native.TOOLS/'windows-cli-host.exe';exe=native.TOOLS/'target/debug/examples/install_managed_archive.exe'
    plans=[('build',[str(host),'600',native.win(native.TOOLS/'rust/bin/cargo.exe'),'build','-p','dlpad-engine','--example','install_managed_archive','--locked'],600)]
    for mode in ['local','local-verified']:
        plans.append((mode,[str(host),'240',native.win(exe),native.win(manifest),mode,native.win(archive)],240))
    operations={}
    for name,argv,seconds in plans:
        operations[name]=native.run(name,argv,env,out,seconds)
        if not operations[name]['passed']:break
    result=dict(passed=len(operations)==3 and all(r['passed'] for r in operations.values()),operations=operations,archive_sha256_after=native.sha(archive),scope='Windows pinned ZIP actual inactive install/removal and verified selection; supply adoption/GUI/crash recovery unverified')
    result['passed']=result['passed'] and result['archive_sha256_after']==inventory['sha256']
    (out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':result['passed']}));return 0 if result['passed'] else 2
if __name__=='__main__':raise SystemExit(main())
