"""Owned journal evidence collector; root reviews retained TEMP and native identities."""
import argparse,hashlib,json,sys
from pathlib import Path
PROJECT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(PROJECT/'scripts'))
import windows_msvc_run_selected_cli_collect as h
n=h.n

def main():
 p=argparse.ArgumentParser()
 for name in ('proof','example-proof','example','output'):p.add_argument('--'+name,type=Path,required=True)
 a=p.parse_args();before=n.snapshot();proof=h.plain(a.proof);raw=h.bounded(proof);whole=json.loads(raw);h.require_current_whole(whole,before)
 proof_sha=hashlib.sha256(raw).hexdigest();logs=[]
 for name,op in whole['operations'].items():
  log=h.plain(proof.parent/(name+'.log'));h.guard(op,h.bounded(log));logs.append(log)
 producer_path=h.plain(a.example_proof);producer_raw=h.bounded(producer_path);producer=json.loads(producer_raw);example=h.plain(a.example)
 if producer.get('status')!='REVIEW_REQUIRED' or producer.get('target')!='x86_64-pc-windows-msvc' or producer.get('source_before')!=before or producer.get('source_after')!=before or producer.get('whole_proof_sha256')!=proof_sha:raise ValueError('current producer proof')
 if producer.get('binary')!=str(example) or producer.get('artifact')!={'sha256':h.sha(example),'bytes':example.stat().st_size}:raise ValueError('example artifact binding')
 op=producer.get('operation',{});argv=op.get('argv',[])
 args=['build','-p','dlpad-engine','--example','windows_install_journal_crash_fixture','--locked','--offline','--release','--target','x86_64-pc-windows-msvc']
 if len(argv)!=13 or argv[0]!=str(n.TOOLS/'windows-cli-host.exe') or argv[2]!=n.win(n.TOOLS/'rust/bin/cargo.exe') or argv[3:]!=args or op.get('source_before')!=before or op.get('source_after')!=before:raise ValueError('exact producer build operation')
 if op.get('operation')!='example-build':raise ValueError('fixed producer raw log')
 build_log=h.plain(producer_path.parent/'example-build.log');h.terminal(op,h.bounded(build_log),0)
 release=n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release';cpu=h.plain(release/'dlpad-cpu.exe');cli=h.plain(release/'dlpad-local-install.exe')
 for file,leaf in [(cpu,'dlpad-cpu.exe'),(cli,'dlpad-local-install.exe')]:
  if whole['artifacts'][leaf]!={'sha256':h.sha(file),'bytes':file.stat().st_size}:raise ValueError('default artifact current pin')
 originals=producer.get('originals',{})
 if not isinstance(originals,dict) or not 1<=len(originals)<=32:raise ValueError('producer original pin bound')
 for name,pin in originals.items():
  f=h.plain(Path(name))
  if not f.is_relative_to(n.ROOT) or pin!={'sha256':h.sha(f),'bytes':f.stat().st_size}:raise ValueError('producer original pin changed')
 config=h.plain(proof.parent/'effective-configuration.json');configraw=h.bounded(config,65536)
 if originals.get(str(config))!={'sha256':hashlib.sha256(configraw).hexdigest(),'bytes':len(configraw)}:raise ValueError('producer effective config pin')
 host=n.TOOLS/'windows-cli-host.exe'
 pinfiles=[proof,producer_path,build_log,example,cpu,cli,config,host,n.ROOT/'Cargo.lock',Path(__file__).resolve(),Path(h.__file__).resolve(),Path(n.__file__).resolve(),n.ROOT/'scripts/windows_msvc_workspace_probe.py',*logs]
 pins={str(f):h.sha(f) for f in pinfiles}
 pins.update({path:pin["sha256"] for path,pin in originals.items()})
 out=h.plain(a.output)
 if out.exists() or not out.is_relative_to(n.ROOT/'docs/validation/evidence'):raise ValueError('fresh evidence')
 out.mkdir(mode=0o700);inputs=out/'inputs';inputs.mkdir(mode=0o700)
 for source,leaf in [(proof,'whole-proof.json'),(producer_path,'example-proof.json'),(build_log,'example-build.log'),(example,'example.exe'),(cpu,'cpu.exe'),(cli,'cli.exe'),(config,'effective-configuration.json'),(Path(__file__),'collector.py')]:h.copy(source,inputs/leaf)
 if h.sha(inputs/'whole-proof.json')!=proof_sha or h.sha(proof)!=proof_sha or h.sha(inputs/'example-proof.json')!=hashlib.sha256(producer_raw).hexdigest() or h.sha(inputs/'effective-configuration.json')!=hashlib.sha256(configraw).hexdigest():raise ValueError('decoded immutable proof binding')
 archived={str(f):h.sha(f) for f in inputs.iterdir()};ops=[]
 result={'schema':1,'status':'FAIL','source_before':before,'operations':ops,'original_pins':pins,'archived_pins':archived,'actual_launch':False,'gui_verified':False,'resume_authority':False,'internally_owned_child_explicit_exit':True,'crash_commit_proven':False,'deletion_authority':False,'installer_complete':False,'root_review_pending':True}
 _,env=n.configure()
 try:
  operation=n.run('journal-crash-fixture',[str(host),'45',n.win(inputs/'example.exe'),'--fixed-journal-explicit-exit'],env,out,45);ops.append(operation)
  rawrun=h.bounded(out/'journal-crash-fixture.log');h.terminal(operation,rawrun,0)
  lines=[line for line in rawrun.splitlines() if line.startswith(b'{')]
  if len(lines)!=1 or len(lines[0])+1>4096:raise ValueError('exact4096B newline record')
  receipt=h.jsonrow(rawrun,'phase','journal-explicit-process-exit-evidence');result['fixture_receipt']=receipt
  if receipt.get('schema')!=1 or receipt.get('child_exit')!=73 or type(receipt.get('child_exit'))is not int or receipt.get('prefix_preserved')is not True:raise ValueError('explicit child exit/prefix receipt')
  if any(receipt.get(key)is not False for key in ['crash_commit_proven','resume_authority','deletion_authority','actual_launch']):raise ValueError('false scope facts')
  owned=h.winpath(receipt['owned_root'])
  if not owned.is_relative_to(n.TOOLS/'tmp') or not owned.name.startswith('dlpad-journal-explicit-exit-owned-'):raise ValueError('owned TEMP confinement')
  snapshot=receipt.get('snapshot')
  if not isinstance(snapshot,dict) or set(snapshot)!={'root_identity','files'}:raise ValueError('snapshot shape')
  files=snapshot['files']
  if not isinstance(files,list) or len(files)!=2 or [f.get('name')for f in files]!=['intent.json','prepared.json']:raise ValueError('exact string prefix names')
  def identity(value):
   if not isinstance(value,dict) or set(value)!={'Windows'}:raise ValueError('Windows identity shape')
   w=value['Windows']
   if not isinstance(w,dict) or set(w)!={'volume_serial','file_id'} or type(w['volume_serial'])is not int or not 0<w['volume_serial']<=2**64-1:raise ValueError('native volume')
   fid=w['file_id']
   if not isinstance(fid,list) or len(fid)!=16 or any(type(v)is not int or not 0<=v<=255 for v in fid) or not any(fid):raise ValueError('native128ID')
  identity(snapshot['root_identity'])
  ids=[snapshot['root_identity']]
  for f in files:
   if set(f)!={'name','identity','bytes','sha256'} or type(f['bytes'])is not int or not 0<f['bytes']<=65536 or not isinstance(f['sha256'],str) or len(f['sha256'])!=64 or any(v not in '0123456789abcdef'for v in f['sha256']):raise ValueError('bounded file snapshot')
   identity(f['identity'])
   if f['identity']in ids:raise ValueError('distinct original identities')
   ids.append(f['identity'])
  if n.snapshot()!=before or any(h.sha(Path(f))!=digest for f,digest in {**pins,**archived}.items()):raise ValueError('source/input changed')
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),source_unchanged=True,owned_fixture_root=str(owned),independent_native_ids_verified=False,retained_bytes_reviewed=False)
 except BaseException as e:result['failure_kind']=type(e).__name__;raise
 finally:
  log=out/'journal-crash-fixture.log'
  if 'fixture_receipt'not in result and log.exists():
   try:result['failed_fixture_receipt']=h.jsonrow(h.bounded(log),'phase','journal-explicit-process-exit-evidence')
   except (ValueError,KeyError,json.JSONDecodeError):pass
  h.write(out/'result.json',result)
if __name__=='__main__':main()
