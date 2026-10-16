"""Owned MSVC fresh-bundle CLI/private GUI acceptance; root reviews evidence."""
import argparse,hashlib,json,sys,tempfile
from pathlib import Path
# Project-relative import; this collector is published under scripts by root.
PROJECT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(PROJECT/'scripts'))
import windows_msvc_run_selected_cli_collect as h
n=h.n

def main():
 p=argparse.ArgumentParser();p.add_argument('--proof',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 before=n.snapshot();proof=h.plain(a.proof);rawproof=h.bounded(proof);whole=json.loads(rawproof);h.require_current_whole(whole,before)
 root=n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release'
 cpu=root/'dlpad-cpu.exe';cli=root/'dlpad-local-install.exe'
 originals={str(f):h.sha(f) for f in [cpu,cli,n.ROOT/'Cargo.lock',n.TOOLS/'target/release/dlpad-cpu.exe',n.TOOLS/'target/release/dlpad-local-install.exe']}
 for f,key in [(cpu,'dlpad-cpu.exe'),(cli,'dlpad-local-install.exe')]:
  if whole['artifacts'][key]!={'sha256':h.sha(f),'bytes':f.stat().st_size}:raise ValueError('current whole artifact mismatch')
 out=h.plain(a.output)
 if out.exists()or not out.is_relative_to(n.ROOT/'docs/validation/evidence'):raise ValueError('fresh project evidence required')
 out.mkdir(mode=0o700);inputs=out/'inputs';inputs.mkdir(mode=0o700)
 observer=n.ROOT/'scripts/windows_run_selected_cli_probe.cs';native=n.ROOT/'scripts/windows_active_selection_native_ids.cs';host=n.TOOLS/'windows-cli-host.exe'
 for src,leaf in [(proof,'whole-proof.json'),(cpu,'cpu.exe'),(cli,'cli.exe'),(observer,'observer.cs'),(native,'native-ids.cs'),(Path(__file__).absolute(),'collector.py')]:h.copy(src,inputs/leaf)
 pins={str(f):h.sha(f)for f in [proof,cpu,cli,observer,native,host,*inputs.iterdir()]}
 if h.sha(inputs/'whole-proof.json')!=hashlib.sha256(rawproof).hexdigest() or h.sha(proof)!=hashlib.sha256(rawproof).hexdigest():raise ValueError('decoded whole proof changed before copy')
 owned=h.plain(Path(tempfile.mkdtemp(prefix='dlpad-user-bundle-cli-owned-',dir=n.TOOLS/'tmp')))
 source=owned/'source';source.mkdir(mode=0o700);h.copy(inputs/'cpu.exe',source/'dlpad.exe')
 (source/'notice.txt').write_bytes(b'Owned synthetic development fixture notice; not a license grant.\n')
 manifest={'schema':1,'version':'0.0.1','target':'x86_64-pc-windows-msvc','entries':[{'name':f.name,'bytes':f.stat().st_size,'sha256':h.sha(f)}for f in [source/'dlpad.exe',source/'notice.txt']],'development_only':True}
 h.write(owned/'manifest.json',manifest);(owned/'foreign-sentinel').write_bytes(b'owned bundle acceptance sentinel')
 sourcepins={str(f):h.sha(f)for f in [source/'dlpad.exe',source/'notice.txt',owned/'manifest.json',owned/'foreign-sentinel']}
 bundle=owned/'bundle';reg=bundle/'registered';receipt=bundle/'receipt';active=bundle/'active'
 data=h.plain(Path(tempfile.mkdtemp(prefix='dlpad-launch-cli-owned-',dir=n.TOOLS/'tmp')))
 for leaf in ['data','media','tmp']:(data/leaf).mkdir(mode=0o700)
 (data/'data/theme.json').write_bytes(b'"System"');(data/'foreign-sentinel').write_bytes(b'owned GUI data sentinel');sourcepins[str(data/'foreign-sentinel')]=h.sha(data/'foreign-sentinel')
 ops=[];rows=[];result={'schema':1,'status':'FAIL','source_before':before,'operations':ops,'rows':rows,'pins':pins,'owned_fixture_root':str(owned),'owned_data_root':str(data),'root_review_pending':True,'pixels_verified':False,'sqlite_verified':False,'installer_complete':False,'journal_recovery_verified':False};_,env=n.configure()
 try:
  def run(name,argv,seconds=100,code=0):
   op=n.run(name,[str(host),str(seconds),*argv],env,out,seconds);ops.append(op);raw=h.bounded(out/(name+'.log'));h.terminal(op,raw,code);return raw
  def command(target=bundle,m=owned/'manifest.json'):
   return ['prepare-user-bundle','--manifest',n.win(m),'--source-dir',n.win(source),'--bundle-root',n.win(target)]
  def clirow(name,args,code=0):
   value=h.jsonrow(run(name,[n.win(inputs/'cli.exe'),*args],code=code),'operation',args[0]);rows.append({'case':name,'receipt':value})
   if any(value.get(k)is not False for k in ['trusted_provenance','publisher_trust','installer_complete','upgrade_verified','deletion_authority','capabilities_retained_after_exit']):raise ValueError('scope receipt')
   return value
  value=clirow('prepare',command())
  if value['status']!='PREPARED_USER_BUNDLE_DEVELOPMENT_ONLY'or value.get('phase')!='complete'or value.get('outer_capability_acquired')is not True or value.get('retained_capabilities_present_before_drop')is not True or any(value.get(k)is not True for k in ['registration_verified','receipt_verified','pair_verified','active_selection_verified']) or value['mutation_may_have_occurred']is not True or value.get('journal_recovery_verified')is not False or value.get('actual_launch')is not False or value['retained_file_caps']is not None or value['retained_file_cap_count_reported']is not False:raise ValueError('bundle success receipt')
  h.names(bundle,['registered','receipt','active']);h.names(reg,['dlpad.exe','notice.txt','install-ready.json','install-registration.json','install-registration-seal.json']);h.names(receipt,['registration-receipt.json']);h.names(active,['active-selection.json'])
  created={str(f):h.sha(f)for directory in [reg,receipt,active]for f in directory.iterdir()}
  value=clirow('inspect-active',['inspect-active','--target',n.win(reg),'--receipt-target',n.win(receipt),'--active-target',n.win(active)])
  if value['status']!='INSPECTED_ACTIVE_DEVELOPMENT_ONLY':raise ValueError('cold active inspection')
  # Refusal cases use distinct untouched targets, never alter the successful bundle.
  foreign=owned/'foreign-bundle';foreign.mkdir();(foreign/'foreign.txt').write_bytes(b'preserve');sourcepins[str(foreign/'foreign.txt')]=h.sha(foreign/'foreign.txt')
  value=clirow('existing-foreign',command(foreign),1)
  if value['status']!='FAIL' or value.get('phase')!='user-bundle' or value.get('bundle_stage')!='prepare' or value.get('kind')!='AlreadyExists' or value.get('outer_capability_acquired')is not False or value.get('retained_capabilities_present_before_drop')is not False:raise ValueError('existing root unexpectedly adopted/retained')
  h.names(foreign,['foreign.txt'])
  bad=json.loads(json.dumps(manifest));bad['entries'][0]['sha256']='0'*64;h.write(owned/'bad-manifest.json',bad);sourcepins[str(owned/'bad-manifest.json')]=h.sha(owned/'bad-manifest.json');badtarget=owned/'bad-sha'
  value=clirow('bad-sha',command(badtarget,owned/'bad-manifest.json'),1)
  if value['status']!='FAIL' or value.get('phase')!='preflight' or value.get('kind')!='InvalidData' or value.get('mutation_may_have_occurred')is not False or type(value.get('retained_file_caps'))is not int or value['retained_file_caps']!=0 or any(k in value for k in ['outer_capability_acquired','retained_capabilities_present_before_drop','retained_file_cap_count_reported','bundle_stage']) or badtarget.exists():raise ValueError('bad source preflight receipt/target changed')
  for name,extra in [('startup-extra',['--startup-timeout-seconds','2']),('legacy-extra',['--target',n.win(bundle)]),('duplicate',['--bundle-root',n.win(bundle)]),('timeout-zero',['--timeout-seconds','0'])]:
   target=owned/('parser-'+name);value=clirow(name,command(target)+extra,1)
   if value['kind']!='InvalidInput'or value['mutation_may_have_occurred']is not False or target.exists():raise ValueError('parser mutation/refusal')
  fw=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319'
  for name,refs in [('observer',['System.Drawing.dll','System.Web.Extensions.dll',fw+'\\WPF\\UIAutomationClient.dll',fw+'\\WPF\\UIAutomationTypes.dll',fw+'\\WPF\\WindowsBase.dll']),('native-ids',['System.Web.Extensions.dll'])]:
   exe=out/(name+'.exe');run('compile-'+name,[fw+'\\csc.exe','/nologo',*['/reference:'+r for r in refs],'/out:'+n.win(exe),n.win(inputs/(name+'.cs'))],45);h.copy(exe,inputs/(name+'.exe'));pins[str(exe)]=h.sha(exe);pins[str(inputs/(name+'.exe'))]=h.sha(inputs/(name+'.exe'))
  h.write(out/'compiled-pins.json',pins)
  run('gui',[n.win(inputs/'observer.exe'),n.win(inputs/'cli.exe'),h.sha(inputs/'cli.exe'),n.win(reg/'dlpad.exe'),h.sha(source/'dlpad.exe'),n.win(reg),n.win(receipt),n.win(active),n.win(data),n.win(out/'gui')],600)
  summary=json.loads(h.bounded(out/'gui/run-selected-cli-result.json',1048576));result['observer_summary']=summary
  if summary['status']!='REVIEW_REQUIRED':raise ValueError('GUI summary')
  for case,reason,code in [('first','Natural',0),('restart','Natural',0),('cancel','Cancelled',1)]:
   e=json.loads(h.bounded(out/'gui'/case/'result.json',262144));v=json.loads(h.bounded(out/'gui'/case/'cli-receipt.json',4096));rows.append({'case':case,'observer':e,'receipt':v})
   if v.get('reason')!=reason or v.get('phase')!='run-wait'or v.get('job_terminal_confirmed')is not True or v.get('runtime_deadline_seconds','absent')is not None or v.get('startup_timeout_seconds')!=2 or e.get('producer_natural_exit_code')!=code or e.get('runtime_observed_elapsed_ms',0)<4000:raise ValueError('typed runtime')
   for key in ['resume_ack_observed_by_original_hwnd','child_actual_image_id_sha_matched','child_parent_job_verified','parent_job_naturally_empty','parent_job_empty_after_cleanup','worker_drained','stdout_reader_drained','mta_desktop_restored','private_desktop_closed','input_desktop_unchanged']:
    if e.get(key)is not True:raise ValueError('GUI original lifecycle')
   if e.get('forced_cleanup')is not False:raise ValueError('forced success')
  observations={label:h.jsonrow(run('native-'+label,[n.win(inputs/'native-ids.exe'),n.win(path),label],45),'status','OBSERVED_NATIVE_IDS')for label,path in [('registered',reg),('receipt',receipt),('active',active)]}
  rr=h.canonical(h.bounded(reg/'install-registration.json',65536));seal=h.canonical(h.bounded(reg/'install-registration-seal.json',4096));rc=h.canonical(h.bounded(receipt/'registration-receipt.json',65536));ar=h.canonical(h.bounded(active/'active-selection.json',65536))
  ready=h.canonical(h.bounded(reg/'install-ready.json',65536))
  if ready!=rr['prepared']:raise ValueError('original ready canonical prepared binding')
  enc=lambda x:json.dumps(x,ensure_ascii=False,separators=(',',':')).encode()
  if rr['domain']!='dlpad-app-registration-v1'or seal['domain']!='dlpad-app-registration-seal-v1'or rc['domain']!='dlpad-app-registration-receipt-v1':raise ValueError('domain')
  digest=hashlib.sha256(b'dlpad-app-registration-record-v1\0'+enc(rr)).hexdigest()
  if rr['prepared_sha256']!=hashlib.sha256(enc(rr['prepared'])).hexdigest() or seal['record_sha256']!=digest or rc['registration']!=rr or rc['seal']!=seal:raise ValueError('canonical nested binding')
  for key in ['nonce','root_identity','record_identity','seal_identity']:
   if seal[key]!=rr[key]:raise ValueError('seal original identity binding')
  if rr['prepared']['domain']!='dlpad-prepared-app-v1' or [{k:e[k] for k in ['name','bytes','sha256']}for e in rr['prepared']['entries']]!=manifest['entries']:raise ValueError('prepared source descriptor binding')
  # Engine cold inspection above validates active digest/domain; independently verify literal SHA bindings.
  if ar['domain']!='dlpad-explicit-active-selection-v1' or rr['prepared']['target']!='x86_64-pc-windows-msvc' or ar['registration_sha256']!=digest or ar['receipt_sha256']!=hashlib.sha256(enc(rc)).hexdigest():raise ValueError('active canonical digest')
  bindings=[(observations['active']['root'],ar['active_root_identity']),(observations['active']['entries']['active-selection.json'],ar['marker_identity']),(observations['registered']['root'],rr['root_identity']),(observations['receipt']['root'],rc['receipt_root_identity']),(observations['receipt']['entries']['registration-receipt.json'],rc['receipt_identity'])]
  for leaf,key in [('install-ready.json','ready_identity'),('install-registration.json','record_identity'),('install-registration-seal.json','seal_identity')]:bindings.append((observations['registered']['entries'][leaf],rr[key]))
  for entry in rr['prepared']['entries']:bindings.append((observations['registered']['entries'][entry['name']],entry['identity']))
  if len(bindings)!=10 or not all(h.identity(o,e)for o,e in bindings):raise ValueError('original native IDs')
  h.names(bundle,['registered','receipt','active']);h.names(reg,['dlpad.exe','notice.txt','install-ready.json','install-registration.json','install-registration-seal.json']);h.names(receipt,['registration-receipt.json']);h.names(active,['active-selection.json'])
  if any(h.sha(Path(f))!=s for f,s in {**sourcepins,**created,**pins,**originals}.items())or n.snapshot()!=before:raise ValueError('preservation pins')
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),source_unchanged=True,native_observations=observations,original_preservation_pins=sourcepins,created_pins=created)
 except BaseException as error:result['failure_kind']=type(error).__name__;raise
 finally:h.write(out/'result.json',result)
 print(json.dumps({'status':result['status'],'root_review_pending':True}))
if __name__=='__main__':main()
