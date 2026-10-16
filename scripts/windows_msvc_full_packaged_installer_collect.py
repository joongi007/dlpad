"""Current-proof packaged bootstrap private GUI collector. Root reviews IDs, bytes and PNGs."""
import argparse,hashlib,json,sys
from pathlib import Path
sys.path.insert(0,'/mnt/d/project/dlpad/scripts')
sys.path.insert(0,str(Path(__file__).resolve().parent))
from full_payload_validation import validate
import windows_msvc_run_selected_cli_collect as h
n=h.n

def main():
 p=argparse.ArgumentParser();p.add_argument('--proof',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--generated',type=Path,required=True);p.add_argument('--manifest-sha256',required=True);p.add_argument('--binding-sha256',required=True);p.add_argument('--observer',type=Path,required=True);a=p.parse_args();before=n.snapshot();proof=h.plain(a.proof);proofraw=h.bounded(proof);whole=json.loads(proofraw);h.require_current_whole(whole,before)
 logs=[]
 for name,op in whole['operations'].items():
  f=h.plain(proof.parent/(name+'.log'));h.guard(op,h.bounded(f));logs.append(f)
 release=n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release';cpu=h.plain(release/'dlpad-cpu.exe');installer=h.plain(release/'dlpad-install.exe');cli=h.plain(release/'dlpad-local-install.exe')
 for f in [cpu,installer,cli]:
  if whole['artifacts'][f.name]!={'sha256':h.sha(f),'bytes':f.stat().st_size}:raise ValueError('whole immutable artifact mismatch')
 host=h.plain(n.TOOLS/'windows-cli-host.exe');observer=h.plain(a.observer);config=h.plain(proof.parent/'effective-configuration.json');configraw=h.bounded(config,65536)
 generated=h.plain(a.generated);generated_pins,entry_proofs=validate(h,generated,a.manifest_sha256,a.binding_sha256,whole)
 out=h.plain(a.output)
 if out.exists()or not out.is_relative_to(n.ROOT/'docs/validation/evidence'):raise ValueError('fresh project evidence')
 out.mkdir(mode=0o700);inputs=out/'inputs';inputs.mkdir(mode=0o700)
 originals=[proof,config,cpu,installer,cli,host,observer,Path(__file__).resolve(),Path(h.__file__).resolve(),Path(n.__file__).resolve(),n.ROOT/'scripts/windows_msvc_workspace_probe.py',n.ROOT/'Cargo.lock',n.TOOLS/'target/release/dlpad-cpu.exe',n.TOOLS/'target/release/dlpad-local-install.exe',*logs];pins={str(f):h.sha(f)for f in originals};pins.update(generated_pins);pins[str(Path(__file__).with_name('full_payload_validation.py'))]=h.sha(Path(__file__).with_name('full_payload_validation.py'))
 for f,name in [(proof,'whole-proof.json'),(config,'effective-configuration.json'),(cpu,'cpu.exe'),(installer,'installer.exe'),(observer,'observer.cs'),(Path(__file__).resolve(),'collector.py')]:h.copy(f,inputs/name)
 if h.sha(inputs/'whole-proof.json')!=hashlib.sha256(proofraw).hexdigest()or h.sha(inputs/'effective-configuration.json')!=hashlib.sha256(configraw).hexdigest():raise ValueError('decoded proof/config copy binding')
 pins.update({str(f):h.sha(f)for f in inputs.iterdir()});_,env=n.configure();ops=[];result={'schema':1,'status':'FAIL','source_before':before,'operations':ops,'pins':pins,'root_review_pending':True,'native_id_review_pending':True,'pixel_review_pending':True,'actual_ime':False,'physical_reader':False,'installer_complete':False,'crash_commit_proven':False,'historical_ownership':False,'deletion_authority':False,'actual_inflight_gui_cancel_verified':False,'actual_inflight_gui_close_ack_verified':False}
 try:
  fw=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319';exe=out/'observer.exe';op=n.run('compile',[str(host),'60',fw+'\\csc.exe','/nologo','/reference:System.Drawing.dll','/reference:System.Web.Extensions.dll','/out:'+n.win(exe),n.win(inputs/'observer.cs')],env,out,60);ops.append(op);h.guard(op,h.bounded(out/'compile.log'));pins[str(exe)]=h.sha(exe)
  op=n.run('native-gui',[str(host),'320',n.win(exe),n.win(inputs/'installer.exe'),whole['artifacts']['dlpad-install.exe']['sha256'],n.win(inputs/'cpu.exe'),whole['artifacts']['dlpad-cpu.exe']['sha256'],n.win(out/'gui'),n.win(generated),a.manifest_sha256],env,out,320);ops.append(op);raw=h.bounded(out/'native-gui.log');h.guard(op,raw);receipt=h.jsonrow(raw,'status','REVIEW_REQUIRED');result['fixture_receipt']=receipt
  if receipt.get('natural_exit_code')!=0 or any(receipt.get(k)is not True for k in ['natural_job_zero','install_verify_status_observed','thread_desktop_restored','private_desktop_closed','input_desktop_unchanged'])or receipt.get('forced_cleanup',False)is not False:raise ValueError('actual private GUI lifecycle')
  if any(receipt.get(k)is not False for k in ['actual_ime','physical_reader','installer_complete','global_input','clipboard_used']):raise ValueError('GUI scope')
  owned=h.winpath(receipt['owned_root'])
  if not owned.is_relative_to(n.TOOLS/'tmp')or not owned.name.startswith('dlpad-full-packaged-installer-owned-'):raise ValueError('own TEMP fixture')
  for key in ['full_generated_payload','original_generated_source_files_preserved','installed_five_entry_sha_verified','internal_source_controls_hidden','derived_journal_readonly','target_only_derived_journal','forged_journal_ignored_install_and_verify','journal_sha_preserved_by_readonly_verify','packaged_noargs','targets_initially_empty','source_display_tamper_worker_original_preserved']:
   if receipt.get(key)is not True:raise ValueError('full payload receipt '+key)
  if type(receipt.get('installed_entry_count'))is not int or receipt['installed_entry_count']!=5 or receipt.get('manifest_sha256')!=a.manifest_sha256 or receipt.get('original_binding_sha256')!=a.binding_sha256 or receipt.get('staged_installer_sha256')!=whole['artifacts']['dlpad-install.exe']['sha256'] or receipt.get('source_cpu_sha256')!=whole['artifacts']['dlpad-cpu.exe']['sha256']:raise ValueError('full receipt original binding')
  observed=receipt.get('original_generator_files_sha256')
  if not isinstance(observed,dict)or len(observed)!=7 or {str(h.winpath(k)):v for k,v in observed.items()}!=generated_pins:raise ValueError('exact seven originals receipt')
  current_pins,current_entries=validate(h,generated,a.manifest_sha256,a.binding_sha256,whole)
  if current_pins!=generated_pins or current_entries!=entry_proofs:raise ValueError('generated source namespace/bytes changed')
  registered=h.plain(owned/'bundle/registered')
  if {f.name for f in registered.iterdir()}!=set(entry_proofs)|{'install-ready.json','install-registration.json','install-registration-seal.json'}:raise ValueError('installed exact namespace')
  for name,proofentry in entry_proofs.items():
   file=h.plain(registered/name)
   if file.stat().st_size!=proofentry['bytes']or h.sha(file)!=proofentry['sha256']:raise ValueError('installed five actual SHA/bytes')
  journal=h.winpath(receipt['derived_journal_root']);expected_journal=owned/'.bundle-install-journal'
  if journal!=expected_journal:raise ValueError('derived journal identity path')
  journal_receipt=receipt.get('journal_records_sha256');leaves={'intent.json','prepared.json','registered.json','receipt.json','active.json'}
  if not isinstance(journal_receipt,dict)or set(journal_receipt)!=leaves or {f.name for f in journal.iterdir()}!=leaves:raise ValueError('journal namespace')
  if any(h.sha(h.plain(journal/name))!=digest for name,digest in journal_receipt.items()):raise ValueError('journal actual preserved SHA')
  if n.snapshot()!=before or any(h.sha(Path(f))!=d for f,d in pins.items()):raise ValueError('source/artifact changed')
  result['generated_original_pins']=generated_pins;result['manifest_sha256']=a.manifest_sha256;result['binding_sha256']=a.binding_sha256;result['full_payload_receipt_verified']=True
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),source_unchanged=True,owned_fixture_root=str(owned))
 except BaseException as error:result['failure_kind']=type(error).__name__;raise
 finally:h.write(out/'result.json',result)
 print(json.dumps({'status':result['status'],'root_review_pending':True}))
if __name__=='__main__':main()
