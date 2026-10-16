"""Stage-only readonly journal CLI replay; retained fixture evidence is never mutated."""
import argparse,hashlib,json,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'scripts'))
import windows_msvc_run_selected_cli_collect as h
n=h.n

def main():
 p=argparse.ArgumentParser()
 for key in ('proof','fixture-proof','output'):p.add_argument('--'+key,type=Path,required=True)
 a=p.parse_args();before=n.snapshot();proof=h.plain(a.proof);raw=h.bounded(proof);whole=json.loads(raw);h.require_current_whole(whole,before)
 for name,op in whole['operations'].items():h.guard(op,h.bounded(proof.parent/(name+'.log')))
 fixture=h.plain(a.fixture_proof);fixture_raw=h.bounded(fixture);prior=json.loads(fixture_raw)
 if prior.get('status')!='REVIEW_REQUIRED' or prior.get('source_before')!=before or prior.get('source_after')!=before or prior.get('source_unchanged')is not True:raise ValueError('current fixture proof')
 receipt=prior['fixture_receipt'];owned=h.winpath(receipt['owned_root'])
 if not owned.is_relative_to(n.TOOLS/'tmp') or not owned.name.startswith('dlpad-install-journal-owned-') or receipt.get('status')!='PASS':raise ValueError('owned fixture')
 if len(prior.get('operations',[]))!=1:raise ValueError('single actual fixture op')
 fixture_log=h.plain(fixture.parent/'journal-fixture.log');fixture_run_raw=h.bounded(fixture_log)
 h.guard(prior['operations'][0],fixture_run_raw)
 if h.jsonrow(fixture_run_raw,'phase','install-journal-evidence')!=receipt:raise ValueError('raw fixture receipt mismatch')
 priorpins={}
 for group in ['original_pins','archived_pins']:
  values=prior.get(group)
  if not isinstance(values,dict) or not 1<=len(values)<=64:raise ValueError('prior pin map bound')
  for path,digest in values.items():
   f=h.plain(Path(path))
   if not f.is_relative_to(n.ROOT) or not isinstance(digest,str) or len(digest)!=64 or h.sha(f)!=digest:raise ValueError('prior pin changed')
   priorpins[str(f)]=digest
 decoded_whole_sha=hashlib.sha256(raw).hexdigest()
 if prior['original_pins'].get(str(proof))!=decoded_whole_sha or prior['archived_pins'].get(str(fixture.parent/'inputs/whole-proof.json'))!=decoded_whole_sha:raise ValueError('prior decoded whole binding')
 release=n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release';cli=h.plain(release/'dlpad-local-install.exe');cpu=h.plain(release/'dlpad-cpu.exe');host=n.TOOLS/'windows-cli-host.exe'
 for f,key in [(cli,'dlpad-local-install.exe'),(cpu,'dlpad-cpu.exe')]:
  if whole['artifacts'][key]!={'sha256':h.sha(f),'bytes':f.stat().st_size}:raise ValueError('current artifact')
 def decode_leaf(value):
  if not isinstance(value,dict) or set(value)!={'Windows'}:raise ValueError('OsString Windows domain')
  units=value['Windows']
  if not isinstance(units,list) or not 1<=len(units)<=64 or any(type(u)is not int or not 0<=u<=65535 for u in units):raise ValueError('UTF16 name bound')
  decoded=b''.join(u.to_bytes(2,'little')for u in units).decode('utf-16-le','strict')
  if any(ord(c)<32 or ord(c)==127 or c in '/\\' for c in decoded) or decoded not in ['intent.json','prepared.json','registered.json','receipt.json','active.json','foreign.txt']:raise ValueError('fixed journal leaf')
  return decoded
 created={}
 for leaf,key,count in [('prefix','incomplete_prefix',2),('complete','unknown_preserved',6),('torn','torn_preserved',2),('cli-current','pristine_current',5)]:
  data=receipt[key]
  if len(data['entries'])!=count:raise ValueError('journal count')
  h.names(owned/leaf,[decode_leaf(e['name']) for e in data['entries']])
  for entry in data['entries']:
   f=h.plain(owned/leaf/decode_leaf(entry['name']))
   if f.stat().st_size!=entry['bytes'] or h.sha(f)!=entry['sha256']:raise ValueError('retained journal hash')
   created[str(f)]=entry['sha256']
 # Retain exact current bundle files and outside fixtures without following stored metadata paths.
 h.names(owned/'bundle',['registered','receipt','active'])
 for folder,names in [('registered',['dlpad.exe','notice.txt','install-ready.json','install-registration.json','install-registration-seal.json']),('receipt',['registration-receipt.json']),('active',['active-selection.json'])]:
  directory=h.plain(owned/'bundle'/folder);h.names(directory,names)
  for name in names:
   f=h.plain(directory/name);created[str(f)]=h.sha(f)
 for leaf in ['notice.txt','foreign-sentinel']:created[str(owned/leaf)]=h.sha(owned/leaf)
 pins={str(f):h.sha(f) for f in [proof,fixture,fixture.parent/'journal-fixture.log',cli,cpu,host,Path(__file__),Path(h.__file__),Path(n.__file__),ROOT/'scripts/windows_msvc_workspace_probe.py']}
 pins.update(priorpins)
 out=h.plain(a.output)
 if out.exists() or not out.is_relative_to(ROOT/'docs/validation/evidence'):raise ValueError('fresh output')
 out.mkdir();inputs=out/'inputs';inputs.mkdir()
 for src,name in [(proof,'whole-proof.json'),(fixture,'fixture-proof.json'),(cli,'cli.exe'),(Path(__file__),'collector.py')]:h.copy(src,inputs/name)
 if h.sha(inputs/'whole-proof.json')!=hashlib.sha256(raw).hexdigest() or h.sha(inputs/'fixture-proof.json')!=hashlib.sha256(fixture_raw).hexdigest():raise ValueError('decoded copy proof pin')
 archived={str(f):h.sha(f) for f in inputs.iterdir()};ops=[];rows=[];result={'schema':1,'status':'FAIL','source_before':before,'operations':ops,'rows':rows,'original_pins':pins,'archived_pins':archived,'retained_fixture_pins':created,'actual_launch':False,'root_review_pending':True,'native_ids_independently_verified':False}
 _,env=n.configure()
 try:
  def run(name,args,code):
   op=n.run(name,[str(host),'100',n.win(inputs/'cli.exe'),*args],env,out,100);ops.append(op);raw=h.bounded(out/(name+'.log'));h.terminal(op,raw,code);lines=[line for line in raw.splitlines()if line.startswith(b'{')]
   if len(lines)!=1 or len(lines[0])>4096:raise ValueError('exact4K receipt')
   value=h.jsonrow(raw,'operation',args[0]);rows.append({'case':name,'receipt':value});return value
  def args(journal):return ['inspect-install-journal','--journal-root',n.win(journal),'--bundle-root',n.win(owned/'bundle')]
  def scope(v):
   if any(v.get(k)is not False for k in ['crash_commit_proven','historical_ownership','deletion_authority','mutation_may_have_occurred','actual_launch','trusted_provenance','publisher_trust','installer_complete'])or v.get('retained_file_caps')is not None or v.get('cap_count_reported')is not False:raise ValueError('scope fields')
  good=run('current',args(owned/'cli-current'),0);scope(good)
  if good.get('status')!='INSPECTED_INSTALL_JOURNAL_DEVELOPMENT_ONLY'or good.get('current_consistency_verified')is not True:raise ValueError('current binding')
  for leaf in ['prefix','complete','torn']:
   value=run(leaf,args(owned/leaf),1);scope(value)
   if value.get('status')!='FAIL'or value.get('current_consistency_verified')is not False:raise ValueError('refusal')
  old=run('bundle-compatibility',['inspect-user-bundle','--bundle-root',n.win(owned/'bundle')],0)
  if old.get('status')!='INSPECTED_USER_BUNDLE_DEVELOPMENT_ONLY' or old.get('mutation_may_have_occurred')is not False:raise ValueError('previous bundle command regression')
  old=run('foreign-journal-flag',['inspect-user-bundle','--bundle-root',n.win(owned/'bundle'),'--journal-root',n.win(owned/'cli-current')],1)
  if old.get('phase')!='preflight' or old.get('kind')!='InvalidInput':raise ValueError('foreign command flag accepted')
  base=args(owned/'cli-current')
  for name,argv in [('missing-option',['inspect-install-journal']),('duplicate',base+['--journal-root',n.win(owned/'cli-current')]),('unknown',base+['--unknown','x']),('timeout0',base+['--timeout-seconds','0']),('timeout601',base+['--timeout-seconds','601'])]:
   value=run(name,argv,1);scope(value)
   if value.get('phase')!='preflight'or value.get('kind')!='InvalidInput':raise ValueError('parser failure')
  fresh=h.plain(Path(tempfile.mkdtemp(prefix='dlpad-journal-cli-missing-',dir=n.TOOLS/'tmp')));missing=fresh/'absent'
  value=run('missing-root',args(missing),1);scope(value)
  if value.get('kind')!='NotFound'or missing.exists():raise ValueError('missing root created')
  if n.snapshot()!=before or any(h.sha(Path(f))!=digest for f,digest in {**pins,**archived,**created}.items()):raise ValueError('preservation')
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),source_unchanged=True,owned_fixture_root=str(owned),missing_fixture_parent=str(fresh))
 except BaseException as e:result['failure_kind']=type(e).__name__;raise
 finally:h.write(out/'result.json',result)
if __name__=='__main__':main()
