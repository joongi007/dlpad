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
 args=['build','-p','dlpad-engine','--example','windows_install_journal_fixture','--locked','--offline','--release','--target','x86_64-pc-windows-msvc']
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
 out=h.plain(a.output)
 if out.exists() or not out.is_relative_to(n.ROOT/'docs/validation/evidence'):raise ValueError('fresh evidence')
 out.mkdir(mode=0o700);inputs=out/'inputs';inputs.mkdir(mode=0o700)
 for source,leaf in [(proof,'whole-proof.json'),(producer_path,'example-proof.json'),(build_log,'example-build.log'),(example,'example.exe'),(cpu,'cpu.exe'),(config,'effective-configuration.json'),(Path(__file__),'collector.py')]:h.copy(source,inputs/leaf)
 if h.sha(inputs/'whole-proof.json')!=proof_sha or h.sha(proof)!=proof_sha or h.sha(inputs/'example-proof.json')!=hashlib.sha256(producer_raw).hexdigest() or h.sha(inputs/'effective-configuration.json')!=hashlib.sha256(configraw).hexdigest():raise ValueError('decoded immutable proof binding')
 archived={str(f):h.sha(f) for f in inputs.iterdir()};ops=[]
 result={'schema':1,'status':'FAIL','source_before':before,'operations':ops,'original_pins':pins,'archived_pins':archived,'actual_launch':False,'gui_verified':False,'crash_commit_proven':False,'deletion_authority':False,'installer_complete':False,'root_review_pending':True}
 _,env=n.configure()
 try:
  operation=n.run('journal-fixture',[str(host),'160',n.win(inputs/'example.exe'),'--fixed-install-journal',n.win(inputs/'cpu.exe'),whole['artifacts']['dlpad-cpu.exe']['sha256']],env,out,160);ops.append(operation)
  rawrun=h.bounded(out/'journal-fixture.log');h.terminal(operation,rawrun,0)
  lines=[line for line in rawrun.splitlines() if line.startswith(b'{')]
  if len(lines)!=1 or len(lines[0])>16384:raise ValueError('exact16KiB producer record')
  receipt=h.jsonrow(rawrun,'phase','install-journal-evidence');result['fixture_receipt']=receipt
  if receipt.get('status')!='PASS' or receipt.get('source_sha256')!=whole['artifacts']['dlpad-cpu.exe']['sha256'] or receipt.get('source_bytes')!=whole['artifacts']['dlpad-cpu.exe']['bytes']:raise ValueError('CPU source receipt')
  if any(receipt.get(key)is not True for key in ['source_cursor17_preserved_by_operations','current_bundle_binding_verified','four_typed_failures_preserved','journal_size_rejected_before_read','torn_protected_identity_preserved']) or any(receipt.get(key)is not False for key in ['crash_commit_proven','prior_phase_authority_verified_by_journal','deletion_authority','actual_launch','installer_complete']):raise ValueError('scope receipt facts')
  owned=h.winpath(receipt['owned_root'])
  if not owned.is_relative_to(n.TOOLS/'tmp') or not owned.name.startswith('dlpad-install-journal-owned-'):raise ValueError('owned TEMP result confinement')
  for key,count in [('incomplete_prefix',2),('complete_before_unknown',5),('unknown_preserved',6),('torn_preserved',2)]:
   snapshot=receipt[key]
   if not isinstance(snapshot,dict) or len(snapshot.get('entries',[]))!=count:raise ValueError('bounded phase snapshot count')
  if n.snapshot()!=before or any(h.sha(Path(f))!=digest for f,digest in {**pins,**archived}.items()):raise ValueError('source/input changed')
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),source_unchanged=True,owned_fixture_root=str(owned),independent_native_ids_verified=False,retained_bytes_reviewed=False)
 except BaseException as e:result['failure_kind']=type(e).__name__;raise
 finally:
  log=out/'journal-fixture.log'
  if 'fixture_receipt'not in result and log.exists():
   try:result['failed_fixture_receipt']=h.jsonrow(h.bounded(log),'phase','install-journal-evidence')
   except (ValueError,KeyError,json.JSONDecodeError):pass
  h.write(out/'result.json',result)
if __name__=='__main__':main()
