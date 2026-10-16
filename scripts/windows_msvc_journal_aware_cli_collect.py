"""Owned journal-aware CLI acceptance; no GUI launch; root reviews native IDs."""
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
 host=n.TOOLS/'windows-cli-host.exe'
 config=h.plain(proof.parent/'effective-configuration.json');configraw=h.bounded(config,65536)
 rawlogs=[]
 for name,op in whole['operations'].items():
  log=h.plain(proof.parent/(name+'.log'));h.guard(op,h.bounded(log));rawlogs.append(log)
 for src,leaf in [(proof,'whole-proof.json'),(cpu,'cpu.exe'),(cli,'cli.exe'),(config,'effective-configuration.json'),(Path(__file__).absolute(),'collector.py')]:h.copy(src,inputs/leaf)
 pins={str(f):h.sha(f)for f in [proof,cpu,cli,config,host,Path(h.__file__),Path(n.__file__),*rawlogs,*inputs.iterdir()]}
 if h.sha(inputs/'effective-configuration.json')!=hashlib.sha256(configraw).hexdigest():raise ValueError('decoded configuration copy binding')
 if h.sha(inputs/'whole-proof.json')!=hashlib.sha256(rawproof).hexdigest() or h.sha(proof)!=hashlib.sha256(rawproof).hexdigest():raise ValueError('decoded whole proof changed before copy')
 owned=h.plain(Path(tempfile.mkdtemp(prefix='dlpad-journal-cli-owned-',dir=n.TOOLS/'tmp')))
 source=owned/'source';source.mkdir(mode=0o700);h.copy(inputs/'cpu.exe',source/'dlpad.exe')
 (source/'notice.txt').write_bytes(b'Owned synthetic development fixture notice; not a license grant.\n')
 manifest={'schema':1,'version':'0.0.1','target':'x86_64-pc-windows-msvc','entries':[{'name':f.name,'bytes':f.stat().st_size,'sha256':h.sha(f)}for f in [source/'dlpad.exe',source/'notice.txt']],'development_only':True}
 h.write(owned/'manifest.json',manifest);(owned/'foreign-sentinel').write_bytes(b'owned bundle acceptance sentinel')
 sourcepins={str(f):h.sha(f)for f in [source/'dlpad.exe',source/'notice.txt',owned/'manifest.json',owned/'foreign-sentinel']}
 bundle=owned/'bundle';journal=owned/'journal'
 ops=[];rows=[];created={};result={'schema':1,'status':'FAIL','source_before':before,'operations':ops,'rows':rows,'pins':pins,'original_pins':originals,'owned_fixture_root':str(owned),'root_review_pending':True,'native_ids_verified':False,'actual_launch':False,'installer_complete':False,'historical_ownership':False,'crash_commit_proven':False};_,env=n.configure()
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
  for which in ['bundle','journal']:
   b=owned/('existing-'+which+'-bundle');j=owned/('existing-'+which+'-journal');foreign=b if which=='bundle'else j
   foreign.mkdir();(foreign/'foreign').write_bytes(b'preserve owned refusal sentinel');sourcepins[str(foreign/'foreign')]=h.sha(foreign/'foreign')
   value=clirow('existing-'+which,command(b,j),1);refusal(value,'AlreadyExists','journal-user-bundle')
   if any(value.get(k)is not False for k in ['bundle_capabilities_present_before_drop','journal_capabilities_present_before_drop'])or (j if which=='bundle'else b).exists():raise ValueError('existing root adopted')
   h.names(foreign,['foreign'])
  bad=json.loads(json.dumps(manifest));bad['entries'][0]['sha256']='0'*64;h.write(owned/'bad-manifest.json',bad);sourcepins[str(owned/'bad-manifest.json')]=h.sha(owned/'bad-manifest.json')
  b=owned/'wrongsha-bundle';j=owned/'wrongsha-journal';refusal(clirow('wrongsha',command(b,j,owned/'bad-manifest.json'),1),'InvalidData','preflight')
  if b.exists()or j.exists():raise ValueError('wrong SHA mutation')
  # Existing source parent is deliberate; overlap rejection precedes creation.
  for name,b,j in [('same-target',owned/'same',owned/'same'),('journal-source',owned/'overlap-bundle',source),('bundle-source',source,owned/'overlap-journal')]:
   refusal(clirow(name,command(b,j),1),'InvalidInput','preflight')
   if (b!=source and b.exists()) or (j!=source and j.exists()):raise ValueError('overlap created output')
  for name,extra in [('duplicate',['--journal-root',n.win(journal)]),('startup-extra',['--startup-timeout-seconds','2']),('legacy-extra',['--target',n.win(bundle)]),('timeout-zero',['--timeout-seconds','0']),('timeout-large',['--timeout-seconds','601']),('unknown',['--foreign','x'])]:
   b=owned/('parser-'+name+'-bundle');j=owned/('parser-'+name+'-journal');refusal(clirow(name,command(b,j)+extra,1),'InvalidInput','preflight')
   if b.exists()or j.exists():raise ValueError('parser mutation')
  if any(h.sha(Path(f))!=digest for f,digest in {**sourcepins,**created,**pins,**originals}.items())or n.snapshot()!=before:raise ValueError('preservation pins')
  for directory,names in directories:h.names(directory,names)
  h.names(bundle,['registered','receipt','active'])
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),source_unchanged=True,source_preservation_pins=sourcepins,created_pins=created,journal_path=str(journal),bundle_path=str(bundle),native_identity_review_pending=True)
 except BaseException as error:result['failure_kind']=type(error).__name__;raise
 finally:h.write(out/'result.json',result)
 print(json.dumps({'status':result['status'],'root_review_pending':True}))
if __name__=='__main__':main()
