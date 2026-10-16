"""Owned journal-aware CLI acceptance; no GUI launch; root reviews native IDs."""
import argparse,hashlib,json,sys,tempfile
from pathlib import Path
# Project-relative import; this collector is published under scripts by root.
PROJECT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(PROJECT/'scripts'))
import windows_msvc_run_selected_cli_collect as h
n=h.n

def main():
 p=argparse.ArgumentParser();p.add_argument('--proof',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--native-source',type=Path,required=True);a=p.parse_args()
 before=n.snapshot();proof=h.plain(a.proof);rawproof=h.bounded(proof);whole=json.loads(rawproof);h.require_current_whole(whole,before)
 root=n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release'
 cpu=root/'dlpad-cpu.exe';cli=root/'dlpad-local-install.exe'
 originals={str(f):h.sha(f) for f in [cpu,cli,n.ROOT/'Cargo.lock',n.TOOLS/'target/release/dlpad-cpu.exe',n.TOOLS/'target/release/dlpad-local-install.exe']}
 for f,key in [(cpu,'dlpad-cpu.exe'),(cli,'dlpad-local-install.exe')]:
  if whole['artifacts'][key]!={'sha256':h.sha(f),'bytes':f.stat().st_size}:raise ValueError('current whole artifact mismatch')
 for leaf,pin in whole['artifacts'].items():
  artifact=h.plain(root/leaf)
  if pin!={'sha256':h.sha(artifact),'bytes':artifact.stat().st_size}:raise ValueError('all three current artifact pins')
  originals[str(artifact)]=pin['sha256']
 if len(whole['artifacts'])!=3:raise ValueError('current three artifact proof required')
 out=h.plain(a.output)
 if out.exists()or not out.is_relative_to(n.ROOT/'docs/validation/evidence'):raise ValueError('fresh project evidence required')
 out.mkdir(mode=0o700);inputs=out/'inputs';inputs.mkdir(mode=0o700)
 host=n.TOOLS/'windows-cli-host.exe'
 config=h.plain(proof.parent/'effective-configuration.json');configraw=h.bounded(config,65536)
 rawlogs=[]
 for name,op in whole['operations'].items():
  log=h.plain(proof.parent/(name+'.log'));h.guard(op,h.bounded(log));rawlogs.append(log)
 for src,leaf in [(proof,'whole-proof.json'),(cpu,'cpu.exe'),(cli,'cli.exe'),(config,'effective-configuration.json'),(Path(__file__).absolute(),'collector.py')]:h.copy(src,inputs/leaf)
 pins={str(f):h.sha(f)for f in [proof,cpu,cli,config,host,Path(__file__).resolve(),PROJECT/"scripts/windows_msvc_workspace_probe.py",Path(h.__file__),Path(n.__file__),*rawlogs,*inputs.iterdir()]}
 if h.sha(inputs/'effective-configuration.json')!=hashlib.sha256(configraw).hexdigest():raise ValueError('decoded configuration copy binding')
 if h.sha(inputs/'whole-proof.json')!=hashlib.sha256(rawproof).hexdigest() or h.sha(proof)!=hashlib.sha256(rawproof).hexdigest():raise ValueError('decoded whole proof changed before copy')
 ops=[];rows=[];result={'schema':1,'status':'FAIL','source_before':before,'operations':ops,'rows':rows,'root_review_pending':True,'native_ids_verified':False,'actual_launch':False,'installer_complete':False};_,env=n.configure()
 seed=PROJECT/'scripts/windows_bundle_removal_cli_seed.cs'
 if not seed.exists():raise ValueError('publish staged seed helper first')
 h.copy(seed,inputs/'seed.cs');pins[str(seed)]=h.sha(seed);pins[str(inputs/'seed.cs')]=h.sha(inputs/'seed.cs')
 def run(name,args,seconds=160,code=0):
  try:
   op=n.run(name,[str(host),str(seconds),*args],env,out,seconds);ops.append(op);raw=h.bounded(out/(name+'.log'));h.terminal(op,raw,code);return raw
  except BaseException as error:
   result['failure_kind']=type(error).__name__;result['source_after']=n.snapshot();h.write(out/'result.json',result);raise
 run('compile-seed',[r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe','/nologo','/reference:System.Web.Extensions.dll','/out:'+n.win(out/'seed.exe'),n.win(inputs/'seed.cs')],45)
 native_source=h.plain(a.native_source);native_original=h.plain(PROJECT/'scripts/windows_active_selection_native_ids.cs')
 h.copy(native_source,inputs/'native.cs');h.copy(native_original,inputs/'native-original.cs');pins[str(native_source)]=h.sha(native_source);pins[str(native_original)]=h.sha(native_original);pins[str(inputs/'native.cs')]=h.sha(inputs/'native.cs');pins[str(inputs/'native-original.cs')]=h.sha(inputs/'native-original.cs')
 run('compile-native',[r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe','/nologo','/reference:System.Web.Extensions.dll','/out:'+n.win(out/'native.exe'),n.win(inputs/'native.cs')],45)
 h.copy(out/'native.exe',inputs/'native.exe');pins[str(out/'native.exe')]=h.sha(out/'native.exe');pins[str(inputs/'native.exe')]=h.sha(inputs/'native.exe')
 h.copy(out/'seed.exe',inputs/'seed.exe');pins[str(out/'seed.exe')]=h.sha(out/'seed.exe');pins[str(inputs/'seed.exe')]=h.sha(inputs/'seed.exe')
 seeded=h.jsonrow(run('seed',[n.win(inputs/'seed.exe'),n.win(n.TOOLS/'tmp')],45),'status','OWNED_PRIVATE_SEEDED')
 if seeded.get('current_sid_protected_acl_verified')is not True:raise ValueError('private fixture seed')
 owned=h.winpath(seeded['owned_root'])
 if not owned.is_relative_to(n.TOOLS/'tmp')or not owned.name.startswith('dlpad-bundle-remove-cli-owned-'):raise ValueError('owned namespace')
 source=owned/'source';h.copy(inputs/'cpu.exe',source/'dlpad.exe')
 (source/'notice.txt').write_bytes(b'Owned synthetic development fixture notice; not a license grant.\n')
 manifest={'schema':1,'version':'0.0.1','target':'x86_64-pc-windows-msvc','entries':[{'name':f.name,'bytes':f.stat().st_size,'sha256':h.sha(f)}for f in [source/'dlpad.exe',source/'notice.txt']],'development_only':True}
 h.write(owned/'manifest.json',manifest);(owned/'foreign-sentinel').write_bytes(b'owned bundle acceptance sentinel')
 sourcepins={str(f):h.sha(f)for f in [source/'dlpad.exe',source/'notice.txt',owned/'manifest.json',owned/'foreign-sentinel']}
 bundle=owned/'bundle';journal=owned/'journal'
 created={};result.update(pins=pins,original_pins=originals,owned_fixture_root=str(owned),historical_ownership=False,crash_commit_proven=False)
 def command(b=bundle,j=journal,m=owned/'manifest.json'):
  return ['prepare-journal-user-bundle','--manifest',n.win(m),'--source-dir',n.win(source),'--bundle-root',n.win(b),'--journal-root',n.win(j)]
 def clirow(name,args,code=0):
  op=n.run(name,[str(host),'160',n.win(inputs/'cli.exe'),*args],env,out,160);ops.append(op)
  raw=h.bounded(out/(name+'.log'))
  result.setdefault('raw_log_pins',{})[name]=hashlib.sha256(raw).hexdigest()
  h.terminal(op,raw,code)
  lines=[line for line in raw.splitlines()if line.startswith(b'{')]
  if len(lines)!=1 or len(lines[0])>4096:raise ValueError('one bounded CLI JSON receipt')
  value=json.loads(lines[0]);rows.append({'case':name,'receipt':value})
  if value.get('schema')!=1 or value.get('operation')!=args[0] or any(value.get(k)is not False for k in ['trusted_provenance','publisher_trust','installer_complete','upgrade_verified','deletion_authority','capabilities_retained_after_exit']):raise ValueError('CLI receipt scope')
  return value
 def refusal(value,kind,phase):
  if value.get('status')!='FAIL' or value.get('kind')!=kind or value.get('phase')!=phase or value.get('mutation_may_have_occurred')is not False:raise ValueError('typed no mutation refusal')
 try:
  value=clirow('prepare-journal',command())
  if value.get('status')!='PREPARED_JOURNAL_USER_BUNDLE_DEVELOPMENT_ONLY' or value.get('phase')!='complete' or any(value.get(k)is not True for k in ['mutation_may_have_occurred','bundle_capabilities_present_before_drop','journal_capabilities_present_before_drop','current_consistency_verified','four_verified_stage_records_observed']) or value.get('manual_backfill_used')is not False:raise ValueError('actual stage success')
  for key in ['retained_file_caps','new_registration_file_caps','new_receipt_file_caps','new_active_file_caps','registration_mutation_may_have_occurred','receipt_mutation_may_have_occurred']:
   if key not in value or value[key]is not None:raise ValueError('unknown counts/mutation are not zero')
  directories=[(journal,['intent.json','prepared.json','registered.json','receipt.json','active.json']),(bundle/'registered',['dlpad.exe','notice.txt','install-ready.json','install-registration.json','install-registration-seal.json']),(bundle/'receipt',['registration-receipt.json']),(bundle/'active',['active-selection.json'])]
  h.names(bundle,['registered','receipt','active'])
  for directory,names in directories:
   h.names(directory,names)
   for name in names:
    f=h.plain(directory/name)
    if directory==journal and f.stat().st_size>65536:raise ValueError('journal record cap')
    created[str(f)]=h.sha(f)
  value=clirow('inspect-journal',['inspect-install-journal','--journal-root',n.win(journal),'--bundle-root',n.win(bundle)])
  if value.get('status')!='INSPECTED_INSTALL_JOURNAL_DEVELOPMENT_ONLY' or value.get('current_consistency_verified')is not True:raise ValueError('current journal compatibility')
  value=clirow('inspect-bundle',['inspect-user-bundle','--bundle-root',n.win(bundle)])
  if value.get('status')!='INSPECTED_USER_BUNDLE_DEVELOPMENT_ONLY':raise ValueError('bundle compatibility')
  for name,args,status in [('reg',['inspect-registered','--target',n.win(bundle/'registered')],'INSPECTED_REGISTERED_DEVELOPMENT_ONLY'),('pair',['inspect-pair','--target',n.win(bundle/'registered'),'--receipt-target',n.win(bundle/'receipt')],'INSPECTED_PAIR_DEVELOPMENT_ONLY'),('active',['inspect-active','--target',n.win(bundle/'registered'),'--receipt-target',n.win(bundle/'receipt'),'--active-target',n.win(bundle/'active')],'INSPECTED_ACTIVE_DEVELOPMENT_ONLY')]:
   if clirow('compat-'+name,args).get('status')!=status:raise ValueError('readonly legacy compatibility')
  journalpins={str(journal/name):created[str(journal/name)]for name in ['intent.json','prepared.json','registered.json','receipt.json','active.json']}
  if any(h.sha(Path(f))!=digest for f,digest in {**created,**sourcepins,**pins,**originals}.items())or n.snapshot()!=before:raise ValueError('all original12 files/source/currentpins before removal')
  observations={}
  for mode,path in [('outer',bundle),('registered',bundle/'registered'),('receipt',bundle/'receipt'),('active',bundle/'active'),('journal',journal),('source',source)]:
   raw=run('before-remove-native-'+mode,[n.win(inputs/'native.exe'),n.win(path),mode],45)
   observations[mode]=h.jsonrow(raw,'status','OBSERVED_NATIVE_IDS')
  result['before_removal_native_id_snapshots']=observations
  if any(h.sha(Path(f))!=digest for f,digest in {**created,**sourcepins}.items()):raise ValueError('native observation preserved all original file SHA')
  removal=clirow('remove',['remove-user-bundle','--bundle-root',n.win(bundle),'--journal-root',n.win(journal)])
  if removal.get('status')!='REMOVED_USER_BUNDLE_DEVELOPMENT_ONLY'or removal.get('phase')!='bundle-remove-complete'or removal.get('accepted_file_dispositions')!=7 or removal.get('accepted_root_dispositions')!=4 or removal.get('outer_namespace_absence_confirmed')is not True or removal.get('bundle_root_disposition_accepted')is not True or bundle.exists():raise ValueError('exact removed bundle')
  if removal.get('retained_file_caps','missing')is not None or removal.get('retained_file_cap_count_reported')is not False:raise ValueError('unknown generic capcount')
  preserved=dict(journalpins);negative_namespaces=[]
  other_bundle=owned/'other-bundle';other_journal=owned/'other-journal'
  if clirow('prepare-other-valid',command(other_bundle,other_journal)).get('status')!='PREPARED_JOURNAL_USER_BUNDLE_DEVELOPMENT_ONLY':raise ValueError('other independently valid current bundle')
  other_directories=[(other_journal,directories[0][1]),(other_bundle/'registered',directories[1][1]),(other_bundle/'receipt',directories[2][1]),(other_bundle/'active',directories[3][1])]
  h.names(other_bundle,['registered','receipt','active'])
  for directory,names in other_directories:
   h.names(directory,names)
   for name in names:preserved[str(directory/name)]=h.sha(directory/name)
  other_checked=clirow('inspect-other-valid',['inspect-install-journal','--bundle-root',n.win(other_bundle),'--journal-root',n.win(other_journal)])
  if other_checked.get('status')!='INSPECTED_INSTALL_JOURNAL_DEVELOPMENT_ONLY' or other_checked.get('current_consistency_verified')is not True or any(h.sha(Path(f))!=digest for f,digest in preserved.items()):raise ValueError('other cold current valid bundle/journal and preserved original bytes')
  for mode in ['foreign','torn','mixed']:
   b=owned/(mode+'-bundle');j=owned/(mode+'-journal');v=clirow('prepare-'+mode,command(b,j))
   if v.get('status')!='PREPARED_JOURNAL_USER_BUNDLE_DEVELOPMENT_ONLY':raise ValueError('fresh negative preparation')
   if mode=='foreign':
    with (b/'foreign').open('xb')as f:f.write(b'owned foreign preserve')
   elif mode=='torn':
    # Alter only this disposable negative fixture; preserve archived logs and prior accepted fixtures.
    with (j/'active.json').open('r+b')as f:f.truncate(0);f.write(b'{torn');f.flush();__import__('os').fsync(f.fileno())
   targetjournal=other_journal if mode=='mixed'else j
   known=[(b/'registered',directories[1][1]),(b/'receipt',directories[2][1]),(b/'active',directories[3][1]),(j,directories[0][1])]
   for directory,names in known:
    h.names(directory,names)
    for name in names:preserved[str(directory/name)]=h.sha(directory/name)
   h.names(b,['registered','receipt','active']+(['foreign']if mode=='foreign'else []));negative_namespaces.append((b,['registered','receipt','active']+(['foreign']if mode=='foreign'else [])))
   if mode=='foreign':preserved[str(b/'foreign')]=h.sha(b/'foreign')
   value=clirow('refuse-'+mode,['remove-user-bundle','--bundle-root',n.win(b),'--journal-root',n.win(targetjournal)],1);refusal(value,'InvalidData','bundle-remove-authorize')
   if value.get('accepted_file_dispositions')!=0 or value.get('accepted_root_dispositions')!=0:raise ValueError('refusal disposed files')
  # Parser-only legacy failures need no historical fixture mutation or app execution.
  commands=['prepare','inspect','prepare-register','inspect-registered','prepare-register-receipt','inspect-pair','remove-registered','select-active','inspect-active','launch-selected','run-selected','prepare-user-bundle','inspect-user-bundle','inspect-install-journal','prepare-journal-user-bundle']
  for index,name in enumerate(commands):refusal(clirow('legacy-parser-'+str(index),[name,'--unknown','x'],1),'InvalidInput','preflight')
  for name,extra in [('duplicate',['--journal-root',n.win(journal)]),('startup',['--startup-timeout-seconds','2']),('manifest',['--manifest',n.win(owned/'manifest.json')]),('timeout0',['--timeout-seconds','0']),('timeout601',['--timeout-seconds','601']),('unknown',['--unknown','x'])]:
   missing=owned/('parser-'+name);refusal(clirow('parser-'+name,['remove-user-bundle','--bundle-root',n.win(missing),'--journal-root',n.win(journal),*extra],1),'InvalidInput','preflight')
   if missing.exists():raise ValueError('parser createdroot')
  if any(h.sha(Path(f))!=digest for f,digest in {**sourcepins,**preserved,**pins,**originals}.items())or n.snapshot()!=before:raise ValueError('preservation pins')
  for directory,names in [*negative_namespaces,*other_directories]:h.names(directory,names)
  h.names(other_bundle,['registered','receipt','active'])
  h.names(journal,directories[0][1])
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),source_unchanged=True,source_preservation_pins=sourcepins,preserved_negative_and_journal_pins=preserved,removed_original_pins=created,journal_path=str(journal),bundle_path=str(bundle),actual_bundle_namespace_absent=True,native_identity_review_pending=True)
 except BaseException as error:result['failure_kind']=type(error).__name__;raise
 finally:
  h.write(out/('result.json'if not(out/'result.json').exists()else'final-failure-state.json'),result)
 print(json.dumps({'status':result['status'],'root_review_pending':True}))
if __name__=='__main__':main()
