"""Staged current-proof native removal GUI collector; one fresh case per invocation. Root reviews IDs, bytes and PNGs."""
import argparse,hashlib,json,sys
from pathlib import Path
PROJECT=Path('/mnt/d/project/dlpad') if Path('/mnt/d/project/dlpad/scripts').is_dir() else Path(__file__).resolve().parents[1]
sys.path.insert(0,str(PROJECT/'scripts'))
import windows_msvc_run_selected_cli_collect as h
n=h.n

def main():
 p=argparse.ArgumentParser();p.add_argument('--proof',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--observer-source',type=Path,required=True);p.add_argument('--mode',choices=['restart-back','restart-remove'],required=True);a=p.parse_args();before=n.snapshot();proof=h.plain(a.proof);proofraw=h.bounded(proof);whole=json.loads(proofraw);h.require_current_whole(whole,before)
 logs=[]
 for name,op in whole['operations'].items():
  f=h.plain(proof.parent/(name+'.log'));h.guard(op,h.bounded(f));logs.append(f)
 release=n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release';cpu=h.plain(release/'dlpad-cpu.exe');installer=h.plain(release/'dlpad-install.exe');cli=h.plain(release/'dlpad-local-install.exe')
 for f in [cpu,installer,cli]:
  if whole['artifacts'][f.name]!={'sha256':h.sha(f),'bytes':f.stat().st_size}:raise ValueError('whole immutable artifact mismatch')
 host=h.plain(n.TOOLS/'windows-cli-host.exe');observer=h.plain(a.observer_source);config=h.plain(proof.parent/'effective-configuration.json');configraw=h.bounded(config,65536)
 out=h.plain(a.output)
 if out.exists()or not out.is_relative_to(n.ROOT/'docs/validation/evidence'):raise ValueError('fresh project evidence')
 out.mkdir(mode=0o700);inputs=out/'inputs';inputs.mkdir(mode=0o700)
 originals=[proof,config,cpu,installer,cli,host,observer,Path(__file__).resolve(),Path(h.__file__).resolve(),Path(n.__file__).resolve(),n.ROOT/'scripts/windows_msvc_workspace_probe.py',n.ROOT/'Cargo.lock',n.TOOLS/'target/release/dlpad-cpu.exe',n.TOOLS/'target/release/dlpad-local-install.exe',*logs];pins={str(f):h.sha(f)for f in originals}
 for f,name in [(proof,'whole-proof.json'),(config,'effective-configuration.json'),(cpu,'cpu.exe'),(installer,'installer.exe'),(observer,'observer.cs'),(Path(__file__).resolve(),'collector.py')]:h.copy(f,inputs/name)
 if h.sha(inputs/'whole-proof.json')!=hashlib.sha256(proofraw).hexdigest()or h.sha(inputs/'effective-configuration.json')!=hashlib.sha256(configraw).hexdigest():raise ValueError('decoded proof/config copy binding')
 pins.update({str(f):h.sha(f)for f in inputs.iterdir()});_,env=n.configure();ops=[];result={'schema':1,'status':'FAIL','source_before':before,'operations':ops,'pins':pins,'root_review_pending':True,'native_id_review_pending':True,'pixel_review_pending':True,'actual_ime':False,'physical_reader':False,'installer_complete':False,'crash_commit_proven':False,'historical_ownership':False,'deletion_authority':False,'actual_inflight_gui_cancel_verified':False,'actual_inflight_gui_close_ack_verified':False,'mode':a.mode,'accepted_disposition_counts_observed':False}
 try:
  fw=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319';exe=out/'observer.exe';op=n.run('compile',[str(host),'60',fw+'\\csc.exe','/nologo','/reference:System.Drawing.dll','/reference:System.Web.Extensions.dll','/out:'+n.win(exe),n.win(inputs/'observer.cs')],env,out,60);ops.append(op);h.guard(op,h.bounded(out/'compile.log'));pins[str(exe)]=h.sha(exe)
  op=n.run('native-gui',[str(host),'500',n.win(exe),n.win(inputs/'installer.exe'),whole['artifacts']['dlpad-install.exe']['sha256'],n.win(inputs/'cpu.exe'),whole['artifacts']['dlpad-cpu.exe']['sha256'],n.win(out/'gui'),a.mode],env,out,500);ops.append(op);raw=h.bounded(out/'native-gui.log');h.guard(op,raw);receipt=h.jsonrow(raw,'status','REVIEW_REQUIRED');result['fixture_receipt']=receipt
  if receipt.get('natural_exit_code')!=0 or any(receipt.get(k)is not True for k in ['natural_job_zero','install_verify_status_observed','thread_desktop_restored','private_desktop_closed','input_desktop_unchanged'])or receipt.get('forced_cleanup',False)is not False:raise ValueError('actual private GUI lifecycle')
  if any(receipt.get(k)is not False for k in ['actual_ime','physical_reader','installer_complete','global_input','clipboard_used']):raise ValueError('GUI scope')
  if receipt.get('control_mode')!=a.mode:raise ValueError('exact mode receipt')
  required={'restart-back':'confirmation_cancel_no_deletion','restart-remove':'verified_confirmed_removal_observed'}[a.mode]
  if receipt.get(required)is not True:raise ValueError('mode-specific actual acceptance')
  if len(receipt.get('original_file_sha256',{}))!=12 or len(receipt.get('preserved_journal_sha256',{}))!=5:raise ValueError('fixed original/journal snapshot counts')
  result['actual_inflight_gui_cancel_verified']=a.mode=='cancel'
  result['actual_inflight_gui_close_ack_verified']=a.mode=='close'
  if receipt.get('preserved_native_identities_verified')is not True or len(receipt.get('removal_native_before',{}))!=11:raise ValueError('original native identities')
  if receipt.get('first_natural_exit_code')!=0 or any(receipt.get(k)is not True for k in ['first_natural_job_zero','separate_restart_two_root_verified','restart_original_bundle_identities_preserved']):raise ValueError('separate restart actual facts')
  owned=h.winpath(receipt['owned_root'])
  if not owned.is_relative_to(n.TOOLS/'tmp')or not owned.name.startswith('dlpad-native-installer-owned-'):raise ValueError('own TEMP fixture')
  if n.snapshot()!=before or any(h.sha(Path(f))!=d for f,d in pins.items()):raise ValueError('source/artifact changed')
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),source_unchanged=True,owned_fixture_root=str(owned))
 except BaseException as error:result['failure_kind']=type(error).__name__;raise
 finally:
  rawfile=out/'gui/result.json'
  if rawfile.is_file():
   try:result['raw_gui_summary']=json.loads(h.bounded(rawfile,32768))
   except Exception as error:result['raw_gui_summary_read_error']=type(error).__name__
  h.write(out/'result.json',result)
 print(json.dumps({'status':result['status'],'root_review_pending':True}))
if __name__=='__main__':main()
