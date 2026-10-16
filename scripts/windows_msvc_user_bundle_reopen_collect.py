"""Owned current-source readonly bundle inspection and synthetic failure evidence."""
import argparse,hashlib,json,sys
from pathlib import Path
PROJECT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(PROJECT/'scripts'))
import windows_msvc_run_selected_cli_collect as h
n=h.n

def main():
 p=argparse.ArgumentParser()
 for key in ('proof','output','bundle','example','example-proof'):p.add_argument('--'+key,type=Path,required=True)
 p.add_argument('--example-sha256',required=True);a=p.parse_args()
 before=n.snapshot();proof=h.plain(a.proof);raw=h.bounded(proof);whole=json.loads(raw);h.require_current_whole(whole,before)
 for name,op in whole['operations'].items():h.guard(op,h.bounded(proof.parent/(name+'.log')))
 release=n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release'
 cli=h.plain(release/'dlpad-local-install.exe');cpu=h.plain(release/'dlpad-cpu.exe');example=h.plain(a.example);bundle=h.plain(a.bundle)
 example_proof=h.plain(a.example_proof);example_raw=h.bounded(example_proof);producer=json.loads(example_raw)
 if producer.get('status')!='REVIEW_REQUIRED' or producer.get('target')!='x86_64-pc-windows-msvc' or producer.get('source_before')!=before or producer.get('source_after')!=before or producer.get('whole_proof_sha256')!=hashlib.sha256(raw).hexdigest():raise ValueError('actual producer proof source/whole binding')
 if producer.get('binary')!=str(example) or producer.get('artifact')!={'sha256':h.sha(example),'bytes':example.stat().st_size}:raise ValueError('producer binary artifact binding')
 operation=producer.get('operation',{});argv=operation.get('argv',[])
 suffix=['build','-p','dlpad-engine','--example','windows_user_bundle_reopen_fixture','--locked','--offline','--release','--target','x86_64-pc-windows-msvc']
 if len(argv)!=13 or argv[3:]!=suffix or argv[0]!=str(n.TOOLS/'windows-cli-host.exe') or argv[2]!=n.win(n.TOOLS/'rust/bin/cargo.exe') or operation.get('source_before')!=before or operation.get('source_after')!=before:raise ValueError('exact actual producer Cargo argv/source')
 op_name=operation.get('operation')
 if not isinstance(op_name,str) or not op_name or any(c not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_' for c in op_name):raise ValueError('producer raw log name')
 producer_log=h.plain(example_proof.parent/(op_name+'.log'));h.terminal(operation,h.bounded(producer_log),0)
 if not bundle.is_relative_to(n.TOOLS/'tmp') or len(a.example_sha256)!=64 or any(c not in '0123456789abcdef' for c in a.example_sha256) or h.sha(example)!=a.example_sha256:raise ValueError('owned input/example pin')
 for f,key in [(cpu,'dlpad-cpu.exe'),(cli,'dlpad-local-install.exe')]:
  if whole['artifacts'][key]!={'sha256':h.sha(f),'bytes':f.stat().st_size}:raise ValueError('whole artifact pin')
 out=h.plain(a.output)
 if out.exists() or not out.is_relative_to(n.ROOT/'docs/validation/evidence'):raise ValueError('fresh evidence required')
 out.mkdir(mode=0o700);inputs=out/'inputs';inputs.mkdir(mode=0o700)
 native=n.ROOT/'scripts/windows_active_selection_native_ids.cs';host=n.TOOLS/'windows-cli-host.exe'
 original={str(f):h.sha(f) for f in [proof,example_proof,producer_log,cli,cpu,example,native,host,n.ROOT/'Cargo.lock',Path(__file__).resolve(),Path(h.__file__).resolve(),Path(n.__file__).resolve(),PROJECT/'scripts/windows_msvc_workspace_probe.py']}
 for f,name in [(proof,'whole-proof.json'),(example_proof,'example-proof.json'),(producer_log,'example-build.log'),(cli,'cli.exe'),(example,'example.exe'),(native,'native.cs'),(Path(__file__),'collector.py')]:h.copy(f,inputs/name)
 if h.sha(inputs/'whole-proof.json')!=hashlib.sha256(raw).hexdigest() or h.sha(proof)!=hashlib.sha256(raw).hexdigest() or h.sha(inputs/'example-proof.json')!=hashlib.sha256(example_raw).hexdigest():raise ValueError('decoded proof replaced before immutable archive')
 # Extend only archived helper: root mode queries the original outer directory, no files.
 cs=(inputs/'native.cs').read_text();needle='args[1]=="active"?'
 if cs.count(needle)!=1:raise ValueError('native helper exact transform')
 (inputs/'native.cs').write_text(cs.replace(needle,'args[1]=="outer"?new string[0]:args[1]=="active"?'))
 pins={str(f):h.sha(f) for f in inputs.iterdir()};ops=[];rows=[]
 result={'schema':1,'status':'FAIL','source_before':before,'operations':ops,'rows':rows,'input_pins':original,'archived_pins':pins,'root_review_pending':True,'actual_launch':False,'installer_complete':False,'canonical_record_identity_independently_compared':False}
 _,env=n.configure()
 try:
  def run(name,args,seconds=100,code=0):
   op=n.run(name,[str(host),str(seconds),*args],env,out,seconds);ops.append(op);raw=h.bounded(out/(name+'.log'));h.terminal(op,raw,code);return raw
  def command(name,args,code=0):
   v=h.jsonrow(run(name,[n.win(inputs/'cli.exe'),*args],code=code),'operation',args[0]);rows.append({'case':name,'receipt':v});return v
  def observe(tag):
   data={}
   for mode,path in [('outer',bundle),('registered',bundle/'registered'),('receipt',bundle/'receipt'),('active',bundle/'active')]:
    raw=run(tag+'-'+mode,[n.win(inputs/'native.exe'),n.win(path),mode]);v=h.jsonrow(raw,'status','OBSERVED_NATIVE_IDS');data[mode]=v
   return data
  fw=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319'
  run('compile-native',[fw+'\\csc.exe','/nologo','/reference:System.Web.Extensions.dll','/out:'+n.win(inputs/'native.exe'),n.win(inputs/'native.cs')],45)
  pins[str(inputs/'native.exe')]=h.sha(inputs/'native.exe')
  original_ids=observe('before')
  producer_raw=run('example',[n.win(inputs/'example.exe'),'--fixed-user-bundle-reopen',n.win(bundle)],100)
  producer_lines=[line for line in producer_raw.splitlines() if line.startswith(b'{')]
  if len(producer_lines)!=1 or len(producer_lines[0])>16384:raise ValueError('producer JSON bound')
  receipt=h.jsonrow(producer_raw,'phase','user-bundle-reopened')
  if receipt.get('status')!='PASS' or receipt.get('four_typed_failures_preserved')is not True or receipt.get('fresh_negative_cases',{}).get('valid_pair_mix_tested')is not False:raise ValueError('example scope receipt')
  result['producer_receipt']=receipt
  good=command('inspect',['inspect-user-bundle','--bundle-root',n.win(bundle)])
  if good.get('status')!='INSPECTED_USER_BUNDLE_DEVELOPMENT_ONLY' or good.get('mutation_may_have_occurred')is not False:raise ValueError('inspect receipt')
  for name,args in [('missing-option',['inspect-user-bundle']),('duplicate',['inspect-user-bundle','--bundle-root',n.win(bundle),'--bundle-root',n.win(bundle)]),('unknown',['inspect-user-bundle','--bundle-root',n.win(bundle),'--unknown','x']),('timeout-zero',['inspect-user-bundle','--bundle-root',n.win(bundle),'--timeout-seconds','0']),('foreign-command',['inspect','--bundle-root',n.win(bundle)])]:
   v=command(name,args,1)
   if v.get('status')!='FAIL' or v.get('phase')!='preflight' or v.get('mutation_may_have_occurred')is not False:raise ValueError('parser rejection')
  # Missing path is an explicit fresh owned child, never a reused target.
  import tempfile
  owned=h.plain(Path(tempfile.mkdtemp(prefix='dlpad-bundle-reopen-runner-',dir=n.TOOLS/'tmp')));missing=owned/'missing'
  v=command('missing-root',['inspect-user-bundle','--bundle-root',n.win(missing)],1)
  if v.get('status')!='FAIL' or v.get('kind')!='NotFound' or v.get('mutation_may_have_occurred')is not False or missing.exists():raise ValueError('missing root preservation')
  final_ids=observe('after')
  if original_ids!=final_ids:raise ValueError('11 native root/file IDs or file hashes changed')
  if any(h.sha(Path(f))!=s for f,s in {**original,**pins}.items()) or n.snapshot()!=before:raise ValueError('input/source changed')
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),native_before=original_ids,native_after=final_ids,eleven_original_native_ids_preserved=True,owned_missing_fixture=str(owned))
 except BaseException as e:result['failure_kind']=type(e).__name__;raise
 finally:h.write(out/'result.json',result)
if __name__=='__main__':main()
