"""Root-run typed refusal supplement; no resume, deletion, or fresh payload copy."""
import argparse,hashlib,json,os,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
import windows_msvc_run_selected_cli_collect as h
n=h.n

def tree(root):
 values={};total=0
 def walk(path):
  nonlocal total
  h.plain(path)
  if len(values)>=64:raise ValueError('fixture namespace cap')
  stat=path.stat();directory=path.is_dir()
  value={'directory':directory,'device':stat.st_dev,'inode':stat.st_ino,'bytes':0 if directory else stat.st_size}
  if not directory:
   total+=stat.st_size
   if total>256*1024*1024:raise ValueError('aggregate256MiB bound')
   value['sha256']=h.sha(path)
  values[str(path)]=value
  if directory:
   count=0
   for child in path.iterdir():
    count+=1
    if count>64:raise ValueError('directory cap')
    walk(child)
 walk(root);return values

def main():
 p=argparse.ArgumentParser()
 for name in ['proof','cold-result','output']:p.add_argument('--'+name,type=Path,required=True)
 p.add_argument('--cold-sha256',required=True);a=p.parse_args();before=n.snapshot();proof=h.plain(a.proof);pr=h.bounded(proof);whole=json.loads(pr);h.require_current_whole(whole,before)
 for name,op in whole['operations'].items():h.guard(op,h.bounded(proof.parent/(name+'.log')))
 cold=h.plain(a.cold_result);cr=h.bounded(cold,1024*1024);previous=json.loads(cr)
 if hashlib.sha256(cr).hexdigest()!=a.cold_sha256 or previous.get('status')!='REVIEW_REQUIRED' or previous.get('source_before')!=before or previous.get('source_after')!=before or previous.get('source_unchanged')is not True:raise ValueError('exact current cold result')
 for op in previous['operations']:h.guard(op,h.bounded(cold.parent/(op['operation']+'.log')))
 pins=dict(previous['pins']);pins.update({str(proof):h.sha(proof),str(cold):a.cold_sha256,str(Path(__file__).resolve()):h.sha(Path(__file__).resolve()),str(Path(h.__file__).resolve()):h.sha(Path(h.__file__).resolve()),str(Path(n.__file__).resolve()):h.sha(Path(n.__file__).resolve())})
 def unchanged_pins():
  if n.snapshot()!=before:raise ValueError('frozen source changed')
  for name,digest in pins.items():
   if h.sha(h.plain(Path(name)))!=digest:raise ValueError('original pin changed')
 unchanged_pins();owned=h.plain(Path(previous['owned_fixture_root']))
 if not owned.is_relative_to(n.TOOLS/'tmp')or not owned.name.startswith('dlpad-native-installer-owned-'):raise ValueError('owned fixture TEMP only')
 originals=tree(owned)
 for item in previous['fixture_receipt']['originals']:
  path=h.winpath(item['path']);actual=originals.get(str(path))
  if actual is None or (item['sha256'] is not None and (actual.get('sha256')!=item['sha256']or actual['bytes']!=item['bytes'])):raise ValueError('cold preserved original bytes binding')
 bundle=owned/'bundle';journal=owned/'journal';source=owned/'source';manifest=owned/'manifest.json'
 if (bundle/'registered/install-ready.json').exists()or(bundle/'receipt').exists()or(bundle/'active').exists():raise ValueError('incomplete fixture unexpectedly completed')
 h.names(journal,['intent.json'])
 release=n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release';cli=h.plain(release/'dlpad-local-install.exe');host=h.plain(n.TOOLS/'windows-cli-host.exe')
 if whole['artifacts']['dlpad-local-install.exe']!={'sha256':h.sha(cli),'bytes':cli.stat().st_size}:raise ValueError('current CLI artifact')
 pins.update({str(cli):h.sha(cli),str(host):h.sha(host)});out=h.plain(a.output)
 if out.exists()or not out.is_relative_to(n.ROOT/'docs/validation/evidence')or not out.parent.is_dir():raise ValueError('fresh evidence output')
 out.mkdir(mode=0o700);_,env=n.configure();ops=[];rows=[];result={'schema':1,'status':'FAIL','source_before':before,'pins':pins,'operations':ops,'cold_result_sha256':a.cold_sha256,'originals_before':originals,'historical_ownership':False,'resume_proven':False,'upgrade_proven':False,'native_id_supplement_proven':False,'root_review_pending':True}
 try:
  cases=[('partial-inspect',['inspect-user-bundle','--bundle-root',n.win(bundle)],'InvalidData'),('same-root-prepare',['prepare-journal-user-bundle','--manifest',n.win(manifest),'--source-dir',n.win(source),'--bundle-root',n.win(bundle),'--journal-root',n.win(journal)],'AlreadyExists')]
  for name,argv,kind in cases:
   unchanged_pins()
   if tree(owned)!=originals:raise ValueError('fixture changed before operation')
   op=n.run(name,[str(host),'180',n.win(cli),*argv,'--timeout-seconds','150'],env,out,180);ops.append(op);raw=h.bounded(out/(name+'.log'),1024*1024);h.terminal(op,raw,1);row=h.jsonrow(raw,'operation',argv[0]);rows.append(row)
   if row.get('schema')!=1 or row.get('status')!='FAIL'or row.get('kind')!=kind:raise ValueError('exact typed refusal')
   if name=='same-root-prepare'and(row.get('phase')!='journal-user-bundle'or row.get('mutation_may_have_occurred')is not False):raise ValueError('existing-root refusal must precede mutation')
   if any(row.get(k)is not False for k in ['trusted_provenance','publisher_trust','installer_complete','upgrade_verified','deletion_authority','capabilities_retained_after_exit']):raise ValueError('scope and capability flags')
   if tree(owned)!=originals:raise ValueError('fixture namespace/identity/bytes changed')
   unchanged_pins()
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),source_unchanged=True,originals_after=tree(owned),typed_rejections=rows,partial_inspect_invalid_data=True,same_root_prepare_already_exists=True,original_namespace_and_bytes_unchanged=True)
 finally:h.write(out/'result.json',result)
 print(json.dumps({'status':result['status'],'root_review_pending':True}))
if __name__=='__main__':main()
