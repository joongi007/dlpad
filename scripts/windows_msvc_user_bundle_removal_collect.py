"""Native explicit bundle-removal fixture collector. Actual execution requires a current whole proof and root-exclusive target use."""
import argparse,hashlib,json,sys
from pathlib import Path

def main():
 p=argparse.ArgumentParser();p.add_argument('--project',type=Path,default=Path.cwd());p.add_argument('--proof',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 project=a.project.absolute()
 if '..' in project.parts:raise ValueError('project traversal')
 sys.dont_write_bytecode=True;sys.path.insert(0,str(project/'scripts'))
 import windows_msvc_run_selected_cli_collect as h
 n=h.n
 if project!=n.ROOT:raise ValueError('exact original project required')
 before=n.snapshot();proof=h.plain(a.proof);raw=h.bounded(proof);whole=json.loads(raw);h.require_current_whole(whole,before)
 logs=[]
 for name,op in whole['operations'].items():
  log=h.plain(proof.parent/(name+'.log'));h.guard(op,h.bounded(log));logs.append(log)
 config=h.plain(proof.parent/'effective-configuration.json');configraw=h.bounded(config,65536);configuration=json.loads(configraw)
 if not isinstance(configuration.get('values'),dict):raise ValueError('effective configuration values')
 # Mirror the root workspace collector's mandatory override after decoding its config.
 values=dict(configuration['values']);values['CARGO_BUILD_JOBS']='2'
 target=n.TOOLS/'msvc-supply-20261006/target-msvc'
 if values.get('CARGO_TARGET_DIR','').replace('\\','/').lower()!=n.win(target).replace('\\','/').lower():raise ValueError('root-coordinated original target required')
 _,env=n.configure();removed=set(configuration['remove_inherited'])
 for key in removed:env.pop(key,None)
 env['WSLENV']=':'.join(s for s in env.get('WSLENV','').split(':') if s.split('/')[0] not in removed);env.update(values)
 forward=[s for s in env.get('WSLENV','').split(':') if s]
 for key in values:
  if key not in {s.split('/')[0] for s in forward}:forward.append(key)
 env['WSLENV']=':'.join(forward)
 base=target/'x86_64-pc-windows-msvc/release';originals={str(base/leaf):whole['artifacts'][leaf] for leaf in ['dlpad-cpu.exe','dlpad-local-install.exe','dlpad-install.exe']}
 for name,pin in whole.get('gnu_originals',{}).items():originals[name]=pin
 pinfiles=[proof,config,*logs,n.TOOLS/'windows-cli-host.exe',n.ROOT/'Cargo.lock',Path(__file__).resolve(),Path(h.__file__).resolve(),Path(n.__file__).resolve(),n.ROOT/'scripts/windows_msvc_workspace_probe.py',n.ROOT/'scripts/windows_selected_gui_collect.py']
 for file in pinfiles:originals[str(file)]={'sha256':h.sha(file),'bytes':file.stat().st_size}
 for path,pin in originals.items():
  f=h.plain(Path(path))
  if {'sha256':h.sha(f),'bytes':f.stat().st_size}!=pin:raise ValueError('original pin changed')
 out=h.plain(a.output)
 if out.exists() or not out.is_relative_to(n.ROOT/'docs/validation/evidence'):raise ValueError('fresh project evidence required')
 out.mkdir();inputs=out/'inputs';inputs.mkdir()
 for source,leaf in [(proof,'whole-proof.json'),(config,'effective-configuration.json'),(Path(__file__),'collector.py')]:h.copy(source,inputs/leaf)
 if h.sha(inputs/'whole-proof.json')!=hashlib.sha256(raw).hexdigest() or h.sha(inputs/'effective-configuration.json')!=hashlib.sha256(configraw).hexdigest():raise ValueError('decoded/copied proof mismatch')
 result={'schema':1,'status':'FAIL','source_before':before,'originals':originals,'operations':[],'effective_configuration_with_root_jobs2_override':True,'target_reuse_requires_root_exclusive_coordination':True,'actual_launch':False,'historical_ownership':False,'installer_complete':False,'root_review_pending':True,'native_ID_review_pending':True}
 try:
  argv=['build','-p','dlpad-engine','--example','windows_user_bundle_removal_fixture','--locked','--offline','--release','--target','x86_64-pc-windows-msvc']
  op=n.run('example-build',[str(n.TOOLS/'windows-cli-host.exe'),'1800',n.win(n.TOOLS/'rust/bin/cargo.exe'),*argv],env,out,1800);result['operations'].append(op);h.guard(op,h.bounded(out/'example-build.log'))
  binary=h.plain(base/'examples/windows_user_bundle_removal_fixture.exe');initial={'sha256':h.sha(binary),'bytes':binary.stat().st_size};h.copy(binary,inputs/'example.exe')
  if {'sha256':h.sha(inputs/'example.exe'),'bytes':(inputs/'example.exe').stat().st_size}!=initial:raise ValueError('immutable fixture mismatch')
  result['artifact']=initial;result['immutable_binary']=str(inputs/'example.exe')
  op=n.run('removal-fixture',[str(n.TOOLS/'windows-cli-host.exe'),'120',n.win(inputs/'example.exe'),'--fixed-explicit-bundle-removal'],env,out,120);result['operations'].append(op);runraw=h.bounded(out/'removal-fixture.log');h.terminal(op,runraw,0)
  lines=[line for line in runraw.splitlines() if line.startswith(b'{')]
  if len(lines)!=1 or len(lines[0])+1>16384 or (lines[0]+b'\r\n' not in runraw and lines[0]+b'\n' not in runraw):raise ValueError('single bounded newline fixture receipt')
  receipt=h.jsonrow(runraw,'phase','explicit-user-bundle-removal');result['fixture_receipt']=receipt
  if receipt.get('schema')!=1 or receipt.get('status')!='PASS' or receipt.get('source_bytes')!=70000 or receipt.get('synthetic_PE_not_currentCPU')is not True or receipt.get('source_cursor17_preserved')is not True:raise ValueError('synthetic fixture receipt')
  for key in ['historical_ownership','journal_deleted','actual_launch','installer_complete','partial_postdisposition_cancel_verified']:
   if receipt.get(key)is not False:raise ValueError('false scope field')
  cases=receipt.get('cases')
  if not isinstance(cases,list) or [c.get('case')for c in cases]!=['normal','cancel-retry','foreign','deadline']:raise ValueError('four exact cases')
  for c in cases:
   if c.get('journal_preserved')is not True:raise ValueError('journal preservation')
   if c['case']in ['normal','cancel-retry']:
    if c.get('accepted_files')!=6 or c.get('accepted_roots')!=4 or c.get('bundle_absent')is not True:raise ValueError('exact disposition/absence')
   elif c.get('accepted_files')!=0 or c.get('accepted_roots')!=0 or c.get('kind')!=('InvalidData' if c['case']=='foreign' else 'TimedOut'):raise ValueError('typed preservation failure')
  owned=h.winpath(receipt['owned_root'])
  if not owned.is_relative_to(n.TOOLS/'tmp') or not owned.name.startswith('dlpad-bundle-removal-owned-'):raise ValueError('owned fixture namespace')
  if before!=n.snapshot():raise ValueError('source changed')
  for path,pin in originals.items():
   f=Path(path)
   if {'sha256':h.sha(f),'bytes':f.stat().st_size}!=pin:raise ValueError('original postpin mismatch')
  if h.sha(binary)!=initial['sha256'] or h.sha(inputs/'example.exe')!=initial['sha256']:raise ValueError('fixture postpin')
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),source_unchanged=True,owned_fixture_root=str(owned),archived_pins={str(f):h.sha(f)for f in inputs.iterdir()})
 except BaseException as error:result['failure_kind']=type(error).__name__;raise
 finally:h.write(out/'result.json',result)
if __name__=='__main__':main()
