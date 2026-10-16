"""Create-only MSVC development ZIP; source-tree reuse, no native execution."""
import argparse,hashlib,json,sys,os,zipfile,re
from pathlib import Path
ROOT=Path('/mnt/d/project/dlpad');sys.path.insert(0,str(ROOT/'scripts'))
import windows_candidate_package as base
import windows_current_workspace_probe as n
from windows_msvc_workspace_probe import require_current_whole

def plain(p):
 if not p.is_absolute()or '..'in p.parts:raise ValueError('absolute no-parent path')
 return base.plain(p)
def validate_proofs(whole,graph,snapshot,whole_sha,lock_packages,observed):
 if whole.get('passed')is not True or whole.get('target')!='x86_64-pc-windows-msvc' or whole.get('source_before')!=snapshot or whole.get('source_after')!=snapshot:raise ValueError('current completed MSVC proof')
 if set(whole.get('operations',{}))!={'fmt','clippy','test','release'}or any(x.get('passed')is not True or x.get('exit_code')!=0 or x.get('source_unchanged')is not True for x in whole['operations'].values()):raise ValueError('four terminal operations')
 if graph.get('status')!='REVIEW_REQUIRED'or graph.get('source_before')!=snapshot or graph.get('source_after')!=snapshot:raise ValueError('current graph source')
 if whole_sha not in graph.get('pins',{}).values():raise ValueError('graph original whole proof SHA')
 if whole.get('artifacts')!=observed or sorted(graph.get('artifacts',{}).values(),key=lambda x:x['sha256'])!=sorted(observed.values(),key=lambda x:x['sha256']):raise ValueError('exact binaries whole/graph')
 packages=graph.get('resolved_packages',[]);keys={p['name']+'-'+p['version']for p in packages}
 if len(packages)!=258 or len(keys)!=258 or graph.get('union_count')!=258:raise ValueError('exact graph258')
 for package in packages:
  key=package['name']+'-'+package['version']
  if key not in lock_packages or package.get('lock_checksum')!=lock_packages[key].get('checksum') or package.get('source')!=lock_packages[key].get('source'):raise ValueError('graph original lock binding')
 return keys

def select_notices(entries,inventory,keys):
 records=[r for r in inventory['records']if r['name']+'-'+r['version']in keys]
 if len(records)!=len(keys)or {r['name']+'-'+r['version']for r in records}!=keys:raise ValueError('selected inventory exactset')
 selected=base.BudgetEntries()
 for r in records:
  if not r.get('notices'):raise ValueError('selected notice missing')
  for notice in r['notices']:
   leaf=notice['entry'];content=entries.get(leaf)
   if content is None or hashlib.sha256(content).hexdigest()!=notice['sha256']:raise ValueError('selected notice original SHA')
   if leaf not in selected:selected.add_bytes(leaf,content)
 return selected,records

def main():
 p=argparse.ArgumentParser();p.add_argument('--whole-proof',type=Path,required=True);p.add_argument('--graph-proof',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--native-installer-exe',type=Path);p.add_argument('--native-installer-sha256');a=p.parse_args()
 if (a.native_installer_exe is None)!=(a.native_installer_sha256 is None):raise ValueError('native installer path/SHA must be paired')
 if a.native_installer_sha256 is not None and not re.fullmatch('[0-9a-f]{64}',a.native_installer_sha256):raise ValueError('native installer SHA lowercase64')
 wp=plain(a.whole_proof);gp=plain(a.graph_proof);out=plain(a.output)
 if out.exists()or not out.is_relative_to(ROOT/'docs/validation/evidence')or out.suffix!='.zip'or not out.parent.is_dir():raise ValueError('fresh project evidence ZIP')
 wr=base.bounded(wp);gr=base.bounded(gp);whole=json.loads(wr);graph=json.loads(gr);snapshot=n.snapshot();require_current_whole(whole,snapshot)
 if graph.get('pins',{}).get(str(wp))!=hashlib.sha256(wr).hexdigest():raise ValueError('exact graph whole proof path binding')
 lockraw=base.bounded(ROOT/'Cargo.lock');lock=base.toml_inventory(lockraw.decode(),True);packages={p['name']+'-'+p['version']:p for p in lock['package']}
 directory=n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release';bins={name:plain(directory/name)for name in ['dlpad-cpu.exe','dlpad-local-install.exe']}
 if a.native_installer_exe is not None:bins['dlpad-install.exe']=plain(a.native_installer_exe)
 observed={name:{'sha256':base.digest(path),'bytes':path.stat().st_size}for name,path in bins.items()}
 if a.native_installer_exe is not None and observed['dlpad-install.exe']['sha256']!=a.native_installer_sha256:raise ValueError('native installer original SHA')
 keys=validate_proofs(whole,graph,snapshot,hashlib.sha256(wr).hexdigest(),packages,observed)
 # Read original raw logs; passed booleans alone do not prove owned terminal.
 for op,parent in [(op,wp.parent)for op in whole['operations'].values()]+[(op,gp.parent)for op in graph['operations']]:
  if op.get('passed')is not True or op.get('exit_code')!=0 or op.get('source_unchanged')is not True:raise ValueError('terminal proof refused')
  raw=base.bounded(parent/(op['operation']+'.log'));terms=[r for r in raw.splitlines()if r.startswith(b'dlpad-owned-job-terminal ')]
  if len(terms)!=1 or not all(k in terms[0]for k in [b'natural-zero=True',b'final-zero=True',b'final-active=0',b'forced=False',b'job-closed=True',b'private-desktop-closed=True'])or b'dlpad-input-desktop-unchanged=True'not in raw:raise ValueError('raw owned terminal')
 for path,expected in graph['pins'].items():
  if base.digest(plain(Path(path)))!=expected:raise ValueError('original graph pin')
 entries,inventory=base.licenses();entries,records=select_notices(entries,inventory,keys)
 manifest={'schema':1,'target':'x86_64-pc-windows-msvc','development_candidate':True,'development_only':True,'release_ready':False,'legal_coverage_complete':False,'publication_provenance_verified':False,'independent_source_rebuild_verified':False,'stack_adopted':False,'static_crt_build_requested':True,'static_crt_runtime_completeness_verified':False,'entrypoint':'dlpad-cpu.exe','features':['cpu'],'binaries':observed,'whole_proof_sha256':hashlib.sha256(wr).hexdigest(),'graph_proof_sha256':hashlib.sha256(gr).hexdigest(),'cargo_lock_sha256':hashlib.sha256(lockraw).hexdigest(),'toolchain_file_sha256':base.digest(ROOT/'rust-toolchain.toml'),'selected_graph_count':258,'selected_notice_missing_count':0,'scope':'normal/build/proc-macro graph superset; exact linked code/embedded/legal coverage unproven','embedded_fonts':{'notice_sources_present':True,'this_package_linked_payload_verified':False}}
 manifest['pe']={name:base.pe(path,3 if name=='dlpad-local-install.exe'else 2)for name,path in bins.items()}
 if 'dlpad-install.exe' in bins:manifest['native_installer']={'entry':'dlpad-install.exe','role':'development-native-installer','sha256':observed['dlpad-install.exe']['sha256'],'bytes':observed['dlpad-install.exe']['bytes'],'pe':manifest['pe']['dlpad-install.exe'],'development_only':True,'installer_complete':False}
 entries.add_json('candidate-manifest.json',manifest);entries.add_json('license-inventory.json',{'scope':manifest['scope'],'records':records,'missing':[],'complete':False});entries.add_json('whole-lock-appendix.json',{'scope':'whole Cargo.lock superset, separate from selected MSVC graph','missing':inventory['missing'],'missing_count':len(inventory['missing']),'complete':False})
 entries.add_file('build-evidence/rust-toolchain.toml',ROOT/'rust-toolchain.toml');entries.add_bytes('build-evidence/whole-proof-sha256.txt',(manifest['whole_proof_sha256']+'\n').encode());entries.add_bytes('build-evidence/graph-proof-sha256.txt',(manifest['graph_proof_sha256']+'\n').encode());entries.add_bytes('README.txt',b'MSVC DEVELOPMENT CANDIDATE ONLY. Run dlpad-cpu.exe. Companion dlpad-local-install.exe provides explicit development prepare/register/selection/supervised launch/removal only. Not a signed installer, release, updater or publisher trust. External tools, app data, credentials and media are not bundled.\n')
 if set(entries)&set(bins):raise ValueError('binary/notice entry collision')
 if 'dlpad-install.exe' in bins:entries.add_bytes('NATIVE-INSTALLER.txt',b'Optional dlpad-install.exe is an unsigned development native interface for explicit fresh install, existing verification and confirmed removal. No updater, signing, publisher trust, clean-OS or legal completeness claim. No automatic install or launch.\n')
 if len(entries)+len(bins)>4096 or entries.used>16*1024*1024 or entries.used+sum(x['bytes']for x in observed.values())>base.LIMIT:raise ValueError('archive budget')
 with out.open('xb')as stream:
  with zipfile.ZipFile(stream,'w',compression=zipfile.ZIP_STORED,allowZip64=False)as archive:
   for name in sorted([*entries,*bins]):
    info=zipfile.ZipInfo(name,(1980,1,1,0,0,0));info.create_system=3;info.external_attr=(0o100755 if name in bins else 0o100644)<<16
    with archive.open(info,'w')as sink:
     if name in bins:
      pin=observed[name];count=0;h=hashlib.sha256()
      with bins[name].open('rb')as src:
       while b:=src.read(65536):
        count+=len(b)
        if count>pin['bytes']:raise ValueError('binary grew')
        h.update(b);sink.write(b)
      if count!=pin['bytes']or h.hexdigest()!=pin['sha256']:raise ValueError('binary stream pin')
     else:sink.write(entries[name])
  stream.flush();os.fsync(stream.fileno())
 # Failed or partial output is preserved. A pathname is not deletion authority.
 if n.snapshot()!=snapshot or base.bounded(wp)!=wr or base.bounded(gp)!=gr or base.bounded(ROOT/'Cargo.lock')!=lockraw or any(base.digest(bins[k])!=v['sha256']or bins[k].stat().st_size!=v['bytes']for k,v in observed.items()):raise ValueError('postpackage original pins')
 print(json.dumps({'status':'DEVELOPMENT_ZIP_CREATED','sha256':base.digest(out),'bytes':out.stat().st_size,'release_ready':False,'selected_graph_count':258,'whole_lock_missing_count':len(inventory['missing'])}))
if __name__=='__main__':main()
