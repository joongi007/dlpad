"""Root-run draft. Prebuilt fixture must be supplied; import performs no native run."""
import argparse,json,sys,hashlib
from pathlib import Path
ROOT=Path('/mnt/d/project/dlpad');sys.path.insert(0,str(ROOT/'scripts'))
import windows_current_workspace_probe as n
from windows_msvc_run_selected_cli_collect import plain,sha,bounded,copy,write,guard,jsonrow,winpath
from windows_msvc_workspace_probe import require_current_whole

def canonical(path):
 raw=bounded(path,65536);value=json.loads(raw)
 if json.dumps(value,ensure_ascii=False,separators=(',',':')).encode('utf8')!=raw:raise ValueError('canonical original record bytes')
 return value,raw

def match_identity(actual,expected):
 w=expected['Windows']
 if len(w['file_id'])!=16 or actual['volume_serial']!=w['volume_serial'] or actual['file_id_hex']!=bytes(w['file_id']).hex():raise ValueError('original native ID mismatch')

def verify_case_binding(root,case,row,observations):
 paths={label:root/(label+'-'+case)for label in ['registered','receipt','active']}
 rr,rawreg=canonical(paths['registered']/'install-registration.json');seal,_=canonical(paths['registered']/'install-registration-seal.json');ready,_=canonical(paths['registered']/'install-ready.json');receipt,rawreceipt=canonical(paths['receipt']/'registration-receipt.json');active,rawactive=canonical(paths['active']/'active-selection.json')
 if rr['prepared']['target']!='x86_64-pc-windows-msvc' or rr['prepared']!=ready or receipt['registration']!=rr or receipt['seal']!=seal:raise ValueError('original MSVC pair binding')
 digest=hashlib.sha256(b'dlpad-app-registration-record-v1\0'+rawreg).hexdigest()
 if seal['record_sha256']!=digest or active['registration_sha256']!=digest or active['receipt_sha256']!=hashlib.sha256(rawreceipt).hexdigest()or row['active_canonical_sha256']!=hashlib.sha256(rawactive).hexdigest():raise ValueError('original canonical domain digest')
 if active['selected_executable']!=row['selected_executable']:raise ValueError('selected executable DTO')
 expected={'registered':{'root':rr['root_identity'],'install-ready.json':rr['ready_identity'],'install-registration.json':rr['record_identity'],'install-registration-seal.json':rr['seal_identity'],**{e['name']:e['identity']for e in rr['prepared']['entries']}},'receipt':{'root':receipt['receipt_root_identity'],'registration-receipt.json':receipt['receipt_identity']},'active':{'root':active['active_root_identity'],'active-selection.json':active['marker_identity']}}
 total=0
 for label,path in paths.items():
  obs=observations[case+'-'+label]
  if set(obs['entries'])!=set(expected[label])-{'root'}or set(f.name for f in path.iterdir())!=set(obs['entries']):raise ValueError('exact original namespace')
  match_identity(obs['root'],expected[label]['root']);total+=1
  for name,item in obs['entries'].items():
   match_identity(item,expected[label][name]);total+=1
   if sha(path/name)!=item['sha256']or(path/name).stat().st_size!=item['bytes']:raise ValueError('original bytes')
 if total!=10 or len(row['original_preservation_pins'])!=7:raise ValueError('ten original IDs/seven files required')
 return total

def main():
 p=argparse.ArgumentParser();p.add_argument('--whole-proof',type=Path,required=True);p.add_argument('--fixture-build-proof',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 before=n.snapshot();proof=plain(a.whole_proof);whole=json.loads(bounded(proof));require_current_whole(whole,before)
 buildpath=plain(a.fixture_build_proof);build=json.loads(bounded(buildpath))
 if build.get('passed')is not True or build.get('source_before')!=before or build.get('source_after')!=before or build.get('target')!='x86_64-pc-windows-msvc':raise ValueError('current native fixture build proof')
 fixture=plain(Path(build['immutable_fixture_exe']));pin=build['immutable_fixture_artifact']
 if {'sha256':sha(fixture),'bytes':fixture.stat().st_size}!=pin:raise ValueError('original fixture artifact')
 buildop=build['operation'];guard(buildop,bounded(buildpath.parent/'build-producer.log'))
 cpu=plain(n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release/dlpad-cpu.exe')
 if {'sha256':sha(cpu),'bytes':cpu.stat().st_size}!=whole['artifacts']['dlpad-cpu.exe']:raise ValueError('current CPU')
 out=plain(a.output)
 if out.exists()or not out.is_relative_to(ROOT/'docs/validation/evidence'):raise ValueError('fresh evidence')
 out.mkdir();inputs=out/'inputs';inputs.mkdir()
 helper=ROOT/'scripts/windows_selected_descendant_helper.cs';native=ROOT/'scripts/windows_active_selection_native_ids.cs';host=n.TOOLS/'windows-cli-host.exe'
 for src,name in [(proof,'whole.json'),(buildpath,'fixture-build.json'),(fixture,'fixture.exe'),(cpu,'cpu.exe'),(helper,'helper.cs'),(native,'native.cs'),(Path(__file__).absolute(),'collector.py')]:copy(src,inputs/name)
 originals={str(f):sha(f)for f in [proof,buildpath,fixture,cpu,helper,native,host,ROOT/'Cargo.lock',n.TOOLS/'target/release/dlpad-cpu.exe',n.TOOLS/'target/release/dlpad-local-install.exe',*inputs.iterdir()]}
 ops=[];result={'schema':1,'status':'FAIL','source_before':before,'operations':ops,'pins':originals,'release_ready':False,'gui_verified':False,'historical_ownership_verified':False};_,env=n.configure()
 try:
  def run(name,args,seconds=45):
   op=n.run(name,[str(host),str(seconds),*args],env,out,seconds);ops.append(op);raw=bounded(out/(name+'.log'));guard(op,raw);return raw
  csc=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe'
  run('compile-helper',[csc,'/nologo','/platform:x64','/target:exe','/out:'+n.win(inputs/'helper.exe'),n.win(inputs/'helper.cs')])
  run('compile-native',[csc,'/nologo','/reference:System.Web.Extensions.dll','/out:'+n.win(inputs/'native.exe'),n.win(inputs/'native.cs')])
  originals[str(inputs/'helper.exe')]=sha(inputs/'helper.exe');originals[str(inputs/'native.exe')]=sha(inputs/'native.exe')
  value=jsonrow(run('fixture',[n.win(inputs/'fixture.exe'),'--selected-process-owned-fixture',n.win(inputs/'cpu.exe'),sha(inputs/'cpu.exe'),n.win(inputs/'helper.exe'),sha(inputs/'helper.exe')],120),'status','PASS_SELECTED_PROCESS_SCOPED');result['producer_receipt']=value
  cases=value['cases']
  if len(cases)!=3 or [x['case']for x in cases]!=['natural','cancel','deadline']:raise ValueError('three exact cases')
  root=winpath(value['owned_fixture_root']);observations={};native_count=0
  if sha(root/'foreign-sentinel')!=value['sentinel_sha256']or sha(root/'notice.txt')!=value['notice_sha256']:raise ValueError('source root sentinel/notice pin')
  for case,reason,code in [('natural','Natural',17),('cancel','Cancelled',17),('deadline','TimedOut',17)]:
   row=next(x for x in cases if x['case']==case)
   if row['reason']!=reason or row['exit_code']!=code or row['job_terminal_confirmed']is not True or row['leader_terminal_observed_before_wait']is not True or row['actual_job_active_after_leader_terminal']<1:raise ValueError('original leader/Job branch proof')
   if case=='natural'and row['descendant_wait_elapsed_ms']<1000:raise ValueError('natural live descendant wait')
   if row['source_sha256']!=originals[str(inputs/'helper.exe')]or row['source_bytes']!=(inputs/'helper.exe').stat().st_size or row['source_cursor_before']!=17 or row['source_cursor_after']!=17:raise ValueError('original helper bytes/cursor')
   for path,digest in row['original_preservation_pins']:
    file=winpath(path)
    if sha(file)!=digest:raise ValueError('original package pin')
   for label in ['registered','receipt','active']:
    path=root/(label+'-'+case);obs=jsonrow(run('native-'+case+'-'+label,[n.win(inputs/'native.exe'),n.win(path),label]),'status','OBSERVED_NATIVE_IDS');observations[case+'-'+label]=obs
    for name,item in obs['entries'].items():
     if sha(path/name)!=item['sha256']or(path/name).stat().st_size!=item['bytes']:raise ValueError('native original bytes')
   native_count+=verify_case_binding(root,case,row,observations)
  if native_count!=30:raise ValueError('thirty canonical native IDs required')
  if sha(root/'foreign-sentinel')!=value['sentinel_sha256']or sha(root/'notice.txt')!=value['notice_sha256']:raise ValueError('root sentinel/notice changed')
  result['native_observations']=observations;result['original_native_identity_count']=native_count
  if n.snapshot()!=before or any(sha(Path(f))!=h for f,h in originals.items()):raise ValueError('original input preservation')
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),source_unchanged=True,all_three_branches_observed=True,native_id_canonical_binding_verified=True,native_ids_are_installed_namespace_observations=True)
 except BaseException as e:result['failure_kind']=type(e).__name__;raise
 finally:write(out/'result.json',result)
if __name__=='__main__':main()
