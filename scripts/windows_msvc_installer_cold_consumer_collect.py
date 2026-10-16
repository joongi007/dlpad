"""Current-proof native bootstrap inflight control collector. Root reviews IDs, bytes and PNGs."""
import argparse,hashlib,json,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
import windows_msvc_run_selected_cli_collect as h
n=h.n

def main():
 p=argparse.ArgumentParser();p.add_argument('--proof',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--prior-result',type=Path,required=True);p.add_argument('--prior-sha256',required=True);p.add_argument('--observer',type=Path,required=True);a=p.parse_args();before=n.snapshot();proof=h.plain(a.proof);proofraw=h.bounded(proof);whole=json.loads(proofraw);h.require_current_whole(whole,before)
 logs=[]
 for name,op in whole['operations'].items():
  f=h.plain(proof.parent/(name+'.log'));h.guard(op,h.bounded(f));logs.append(f)
 release=n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release';cpu=h.plain(release/'dlpad-cpu.exe');installer=h.plain(release/'dlpad-install.exe');cli=h.plain(release/'dlpad-local-install.exe')
 for f in [cpu,installer,cli]:
  if whole['artifacts'][f.name]!={'sha256':h.sha(f),'bytes':f.stat().st_size}:raise ValueError('whole immutable artifact mismatch')
 host=h.plain(n.TOOLS/'windows-cli-host.exe');observer=h.plain(a.observer);config=h.plain(proof.parent/'effective-configuration.json');configraw=h.bounded(config,65536)
 prior=h.plain(a.prior_result);priorraw=h.bounded(prior,1024*1024);previous=json.loads(priorraw)
 if hashlib.sha256(priorraw).hexdigest()!=a.prior_sha256 or previous.get('status')!='REVIEW_REQUIRED' or previous.get('source_before')!=before or previous.get('source_after')!=before or previous.get('source_unchanged')is not True:raise ValueError('exact current prior receipt')
 for op in previous['operations']:h.guard(op,h.bounded(prior.parent/(op['operation']+'.log')))
 for path,d in previous['pins'].items():
  if h.sha(h.plain(Path(path)))!=d:raise ValueError('prior original pin changed')
 owned=h.winpath(previous['fixture_receipt']['owned_root'])
 if not owned.is_relative_to(n.TOOLS/'tmp')or not owned.name.startswith('dlpad-native-installer-owned-'):raise ValueError('owned fixture TEMP')
 out=h.plain(a.output)
 if out.exists()or not out.is_relative_to(n.ROOT/'docs/validation/evidence'):raise ValueError('fresh project evidence')
 out.mkdir(mode=0o700);inputs=out/'inputs';inputs.mkdir(mode=0o700)
 originals=[prior,proof,config,cpu,installer,cli,host,observer,Path(__file__).resolve(),Path(h.__file__).resolve(),Path(n.__file__).resolve(),n.ROOT/'scripts/windows_msvc_workspace_probe.py',n.ROOT/'Cargo.lock',n.TOOLS/'target/release/dlpad-cpu.exe',n.TOOLS/'target/release/dlpad-local-install.exe',*logs];pins={str(f):h.sha(f)for f in originals}
 for f,name in [(proof,'whole-proof.json'),(config,'effective-configuration.json'),(cpu,'cpu.exe'),(cli,'cli.exe'),(prior,'prior-result.json'),(installer,'installer.exe'),(observer,'observer.cs'),(Path(__file__).resolve(),'collector.py')]:h.copy(f,inputs/name)
 if h.sha(inputs/'whole-proof.json')!=hashlib.sha256(proofraw).hexdigest()or h.sha(inputs/'effective-configuration.json')!=hashlib.sha256(configraw).hexdigest():raise ValueError('decoded proof/config copy binding')
 pins.update({str(f):h.sha(f)for f in inputs.iterdir()});_,env=n.configure();ops=[];result={'schema':1,'status':'FAIL','source_before':before,'operations':ops,'pins':pins,'root_review_pending':True,'native_id_review_pending':True,'pixel_review_pending':True,'actual_ime':False,'physical_reader':False,'installer_complete':False,'crash_commit_proven':False,'historical_ownership':False,'deletion_authority':False,'actual_inflight_gui_cancel_verified':False,'actual_inflight_gui_close_ack_verified':False}
 try:
  fw=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319';exe=out/'observer.exe';op=n.run('compile',[str(host),'60',fw+'\\csc.exe','/nologo','/reference:System.Drawing.dll','/reference:System.Web.Extensions.dll','/out:'+n.win(exe),n.win(inputs/'observer.cs')],env,out,60);ops.append(op);h.guard(op,h.bounded(out/'compile.log'));pins[str(exe)]=h.sha(exe)
  gui=out/'cold';op=n.run('cold-consumer',[str(host),'760',n.win(exe),n.win(inputs/'cli.exe'),whole['artifacts']['dlpad-local-install.exe']['sha256'],n.win(inputs/'prior-result.json'),a.prior_sha256,n.win(gui),n.win(gui/'selected-new-bundle'),n.win(gui/'selected-new-journal')],env,out,760);ops.append(op);raw=h.bounded(out/'cold-consumer.log');h.guard(op,raw);receipt=h.jsonrow(raw,'status','REVIEW_REQUIRED');result['fixture_receipt']=receipt
  if any(receipt.get(k)is not True for k in ['readonly_partial_verify_failed_unchanged','same_roots_prepare_refused_unchanged','fresh_explicit_install_reverified','private_desktop_closed','input_desktop_unchanged']):raise ValueError('cold consumer acceptance')
  steps=receipt['operations']
  if len(steps)!=4 or any(s.get('natural_job_zero')is not True or s.get('forced')is not False for s in steps)or not all(s['exit_code']!=0 for s in steps[:2])or not all(s['exit_code']==0 for s in steps[2:]):raise ValueError('original child process outcomes')
  if n.snapshot()!=before or h.bounded(prior,1024*1024)!=priorraw or any(h.sha(Path(f))!=d for f,d in pins.items()):raise ValueError('source/artifact/prior changed')
  for path,d in previous['pins'].items():
   if h.sha(h.plain(Path(path)))!=d:raise ValueError('prior original postpin changed')
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),source_unchanged=True,owned_fixture_root=str(owned),prior_result_sha256=a.prior_sha256)
 except BaseException as error:result['failure_kind']=type(error).__name__;raise
 finally:h.write(out/'result.json',result)
 print(json.dumps({'status':result['status'],'root_review_pending':True}))
if __name__=='__main__':main()
