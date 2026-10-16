"""Isolated native Windows GNU-host/MSVC-target Cargo experiment."""
import argparse,json,os,sys,hashlib
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
import windows_current_workspace_probe as n

def require_current_whole(result,source):
 if result.get('passed')is not True or result.get('target')!='x86_64-pc-windows-msvc' or result.get('source_before')!=source or result.get('source_after')!=source:
  raise ValueError('current MSVC whole source required')
 operations=result.get('operations',{})
 if set(operations)!={'fmt','clippy','test','release'}:raise ValueError('four actual Cargo checks required')
 for name,op in operations.items():
  if op.get('passed')is not True or op.get('exit_code')!=0 or op.get('collector_failure')is not None or op.get('source_unchanged')is not True or op.get('source_before')!=source or op.get('source_after')!=source or op.get('private_desktop_reported')is not True or op.get('input_desktop_unchanged')is not True:
   raise ValueError('actual Cargo source/exit/private guard required')
  args=op.get('argv',[])
  command={'fmt':'fmt','clippy':'clippy','test':'test','release':'build'}[name]
  if len(args)<4 or args[3]!=command:raise ValueError('actual Cargo subcommand required')
  if name=='fmt':
   if args[4:]!=['--all','--','--check']:raise ValueError('whole format check required')
  else:
   if '--workspace'not in args or args.count('--features')!=1 or args[args.index('--features')+1:args.index('--features')+2]!=['cpu']or args.count('--target')!=1:
    raise ValueError('whole CPU feature target required')
   if name=='clippy'and('--all-targets'not in args or args[-3:]!=['--','-D','warnings']):raise ValueError('strict all-targets clippy required')
   if name=='release'and'--release'not in args:raise ValueError('release build required')
  if name!='fmt' and ('--target'not in args or args[args.index('--target')+1:args.index('--target')+2]!=['x86_64-pc-windows-msvc']or'--locked'not in args):
   raise ValueError('exact locked MSVC Cargo target required')

def sha(p):
 h=hashlib.sha256()
 with p.open('rb')as f:
  while b:=f.read(65536):h.update(b)
 return h.hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--configuration',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--reuse-owned-target',action='store_true');p.add_argument('--allow-download',action='store_true');a=p.parse_args();root=n.ROOT;sup=n.TOOLS/'msvc-supply-20261006';target=sup/'target-msvc';out=a.output
 if not out.is_absolute()or '..'in out.parts or not out.is_relative_to(root/'docs/validation/evidence')or out.exists()or any(x.is_symlink()for x in [out,*out.parents]):raise ValueError('fresh evidence')
 raw=a.configuration.read_bytes()
 if len(raw)>65536:raise ValueError('config bound')
 config=json.loads(raw)
 if target.exists():
  marker=target/'dlpad-experiment-owner.json'
  if not a.reuse_owned_target or target.is_symlink()or not marker.is_file()or marker.is_symlink()or marker.stat().st_size>4096:raise ValueError('owned target required')
  owner=json.loads(marker.read_text());prior=Path(owner['evidence'])
  if owner.get('development_only')is not True or not prior.is_relative_to(root/'docs/validation/evidence')or not (prior/'result.json').is_file():raise ValueError('original experiment marker required')
 elif a.reuse_owned_target:raise ValueError('owned target missing')
 if a.allow_download:
  config['commands']=[[name,[v for v in args if v!='--offline']]for name,args in config['commands']]
 for name in config['required_layout']:
  f=Path(name)
  if not f.exists()or f.is_symlink()or not f.is_relative_to(n.TOOLS):raise ValueError('required project layout')
 before=n.snapshot();out.mkdir();(out/'configuration.json').write_bytes(raw);(out/'collector.py').write_bytes(Path(__file__).read_bytes());(out/'effective-configuration.json').write_text(json.dumps(config,indent=2))
 if not target.exists():
  target.mkdir();(target/'dlpad-experiment-owner.json').write_text(json.dumps({'development_only':True,'evidence':str(out),'source_entries':len(before)}))
 _,env=n.configure();removed=set(config['remove_inherited'])
 for key in removed:env.pop(key,None)
 env['WSLENV']=':'.join(s for s in env.get('WSLENV','').split(':')if s.split('/')[0]not in removed)
 values=dict(config['values']);values['CARGO_BUILD_JOBS']='2';env.update(values)
 forwarding=[s for s in env.get('WSLENV','').split(':')if s]
 for key in values:
  if key not in {s.split('/')[0]for s in forwarding}:forwarding.append(key)
 env['WSLENV']=':'.join(forwarding)
 originals={str(f):{'sha256':sha(f),'bytes':f.stat().st_size}for f in [n.TOOLS/'target/release/dlpad-cpu.exe',n.TOOLS/'target/release/dlpad-local-install.exe',root/'Cargo.lock']}
 result={'passed':False,'status':'FAIL','target':'x86_64-pc-windows-msvc','source_before':before,'operations':{},'gnu_originals':originals,'clean_os_verified':False,'release_ready':False,'gui_verified':False,'windows_ambient_environment_cleared_verified':False}
 try:
  for name,args in config['commands']:
   op=n.run(name,[str(n.TOOLS/'windows-cli-host.exe'),'1800',n.win(n.TOOLS/'rust/bin/cargo.exe'),*args],env,out,1800);result['operations'][name]=op
   if not op['passed']:raise ValueError('actual Cargo '+name+' failed')
  result['source_after']=n.snapshot()
  if result['source_after']!=before:raise ValueError('source changed')
  for path,pin in originals.items():
   f=Path(path)
   if sha(f)!=pin['sha256']or f.stat().st_size!=pin['bytes']:raise ValueError('GNU source artifact/lock changed')
  result['artifacts']={}
  for name in ['dlpad-cpu.exe','dlpad-local-install.exe','dlpad-install.exe']:
   f=target/'x86_64-pc-windows-msvc/release'/name;result['artifacts'][name]={'sha256':sha(f),'bytes':f.stat().st_size}
  result.update(passed=True,status='REVIEW_REQUIRED')
 except BaseException as error:result['failure_kind']=type(error).__name__;result['failure']=str(error);raise
 finally:(out/'result.json').write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps({'passed':result['passed'],'status':result['status']}))
if __name__=='__main__':main()
