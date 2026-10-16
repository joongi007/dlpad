"""Root-run developer-only MSVC reconcile phase experiment."""
import argparse,json,os,sys,hashlib
from pathlib import Path
ROOT=Path('/mnt/d/project/dlpad');sys.path.insert(0,str(ROOT/'scripts'))
import windows_current_workspace_probe as n
from windows_selected_gui_collect import plain,sha,bounded,copy,guard,write
from windows_msvc_workspace_probe import require_current_whole

def main():
 p=argparse.ArgumentParser();p.add_argument('--whole-proof',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 proof=plain(a.whole_proof);whole=json.loads(bounded(proof));before=n.snapshot();configpath=plain(proof.parent/'effective-configuration.json');configraw=bounded(configpath,65536);config=json.loads(configraw)
 require_current_whole(whole,before)
 target=n.TOOLS/'msvc-supply-20261006/target-msvc';marker=plain(target/'dlpad-experiment-owner.json');owner=json.loads(bounded(marker,4096))
 if owner.get('development_only')is not True or not Path(owner['evidence']).is_relative_to(ROOT/'docs/validation/evidence'):raise ValueError('original target reuse owner')
 for path in config['required_layout']:
  path=plain(Path(path))
  if not path.exists()or not path.is_relative_to(n.TOOLS):raise ValueError('supply layout missing')
 out=plain(a.output)
 if out.exists()or not out.is_relative_to(ROOT/'docs/validation/evidence'):raise ValueError('fresh output')
 out.mkdir();inputs=out/'inputs';inputs.mkdir();copy(proof,inputs/'whole-proof.json');copy(configpath,inputs/'effective-configuration.json')
 source=ROOT/'crates/engine/examples/windows_library_reconcile_cost.rs';copy(source,inputs/'experiment.rs');copy(Path(__file__).absolute(),inputs/'collector.py')
 originals={str(f):{'sha256':sha(f),'bytes':f.stat().st_size}for f in [ROOT/'Cargo.lock',n.TOOLS/'target/release/dlpad-cpu.exe',n.TOOLS/'target/release/dlpad-local-install.exe',marker,proof,configpath,source]}
 for name,pin in whole['artifacts'].items():
  f=plain(target/'x86_64-pc-windows-msvc/release'/name)
  if {'sha256':sha(f),'bytes':f.stat().st_size}!=pin:raise ValueError('MSVC whole artifact pin')
  originals[str(f)]=pin
 _,env=n.configure();removed=set(config['remove_inherited'])
 for key in removed:env.pop(key,None)
 env['WSLENV']=':'.join(s for s in env.get('WSLENV','').split(':')if s.split('/')[0]not in removed)
 values=dict(config['values']);values['CARGO_BUILD_JOBS']='2';env.update(values)
 forwarding=[s for s in env.get('WSLENV','').split(':')if s]
 for key in values:
  if key not in {s.split('/')[0]for s in forwarding}:forwarding.append(key)
 env['WSLENV']=':'.join(forwarding)
 # No engine cpu feature exists. Reuse exact target env, rather than workspace feature names.
 args=['build','-p','dlpad-engine','--example','windows_library_reconcile_cost','--target','x86_64-pc-windows-msvc','--release','--locked','--features','library-scan-profile-fixture']
 releaseargs=dict(config['commands'])['release']
 if '--offline'in releaseargs:args.append('--offline')
 result={'schema':1,'passed':False,'target':'x86_64-pc-windows-msvc','source_before':before,'configuration_sha256':hashlib.sha256(configraw).hexdigest(),'originals':originals,'gui_verified':False,'release_ready':False,'effective_values':values}
 try:
  op=n.run('build-producer',[str(n.TOOLS/'windows-cli-host.exe'),'300',n.win(n.TOOLS/'rust/bin/cargo.exe'),*args],env,out,300);result['operation']=op;raw=bounded(out/'build-producer.log');guard(op,raw)
  sourceexe=plain(target/'x86_64-pc-windows-msvc/release/examples/windows_library_reconcile_cost.exe');pin={'sha256':sha(sourceexe),'bytes':sourceexe.stat().st_size};copy(sourceexe,inputs/'experiment.exe')
  pins_before_run={str(sourceexe):sha(sourceexe),str(inputs/'experiment.exe'):sha(inputs/'experiment.exe')}
  for key in ['TEMP','TMP']:env[key]=n.win(n.TOOLS/'tmp')
  env['WSLENV']=env.get('WSLENV','')+':TEMP:TMP'
  n.MAX_LOG=1024*1024
  run=n.run('profile-experiment',[str(n.TOOLS/'windows-cli-host.exe'),'180',n.win(inputs/'experiment.exe'),'--fixed-cost-experiment'],env,out,180);result['experiment_operation']=run
  raw=bounded(out/'profile-experiment.log',1024*1024);guard(run,raw)
  rows=[json.loads(line)for line in raw.splitlines()if line.startswith(b'{')and len(line)<=1024*1024]
  final=[v for v in rows if v.get('status')=='PASS']
  if len(final)!=1:raise ValueError('exact synthetic experiment JSON')
  observation=final[0]
  if observation.get('present_rows')!=10000 or observation.get('database_reopen_verified')is not True or len(observation.get('rounds',[]))!=3:raise ValueError('actual reopened count/three rounds')
  rootwin=observation['owned_fixture_root'];expected=n.win(n.TOOLS/'tmp')+'\\dlpad-reconcile-cost-owned-'
  if not isinstance(rootwin,str)or not rootwin.lower().startswith(expected.lower()):raise ValueError('owned synthetic TEMP root')
  for i,row in enumerate(observation['rounds'],1):
   if row['round']!=i or row['files']!=10000 or row['page_native_identity_preserved']is not True or row['process_cpu_end_100ns']<row['process_cpu_begin_100ns']:raise ValueError('original CPU/identity round')
  profiles=[line.decode('ascii')for line in raw.splitlines()if line.startswith(b'DLPAD_SCAN_PROFILE ')]
  if len(profiles)!=3 or any(len(line)>4096 for line in profiles):raise ValueError('exact bounded three scan profile records')
  import re
  for line in profiles:
   tail=line.split('phases_preflight_setup_traversal_insert_candidates_update_upsert_delete_clear_commit=',1)
   if len(tail)!=2 or len(re.findall(r'\(\d+, \d+, \d+\)',tail[1]))!=9:raise ValueError('nine phase tuples')
  if any(sha(Path(f))!=h for f,h in pins_before_run.items()):raise ValueError('original experiment changed')
  result.update(experiment=observation,profile_records=profiles,profile_log_sha256=sha(out/'profile-experiment.log'),gui_idle_verified=False)
  if n.snapshot()!=before or any({'sha256':sha(Path(f)),'bytes':Path(f).stat().st_size}!=pin for f,pin in originals.items()):raise ValueError('original source/lock/CPU/CLI/config changed')
  result.update(passed=True,status='PASS_SYNTHETIC_RECONCILE_PROFILE_ONLY',source_after=n.snapshot(),fixture_exe=str(sourceexe),fixture_artifact=pin,immutable_fixture_exe=str(inputs/'experiment.exe'),immutable_fixture_artifact={'sha256':sha(inputs/'experiment.exe'),'bytes':(inputs/'experiment.exe').stat().st_size})
 except BaseException as e:result.update(status='FAIL',failure_kind=type(e).__name__);raise
 finally:write(out/'result.json',result)
if __name__=='__main__':main()
