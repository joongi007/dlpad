"""Private frame-phase single60s/six-window diagnosis; no default budget acceptance."""
import argparse,json,os,sys,hashlib,re
from pathlib import Path
sys.path.insert(0,'/mnt/d/project/dlpad/scripts')
import windows_current_workspace_probe as n
from windows_msvc_workspace_probe import require_current_whole
from windows_selected_gui_collect import copy as bounded_copy, sha
WAKE_NAMES=('RootScheduled', 'CallbackScheduled', 'ActionScheduled', 'RootDue', 'CallbackDue', 'ActionDue', 'RepaintEvents', 'WindowRepaintEvents', 'AccessInitialEvents', 'AccessActionEvents', 'CloseEvents', 'RedrawEvents', 'MetricsChecks', 'MetricsUnchanged', 'MetricsRedraws')
MAX_CPU=128*1024*1024
def plain(p):
 if not p.is_absolute()or '..'in p.parts or any(x.is_symlink()for x in [p,*p.parents]):raise ValueError('plain absolute')
 return p
def write(p,v):
 with p.open('x',encoding='utf-8')as f:json.dump(v,f,indent=2);f.write('\n')
def copy(a,b):
 bounded_copy(a,b)
def main():
 p=argparse.ArgumentParser();p.add_argument('proof',type=Path);p.add_argument('output',type=Path);p.add_argument('--feature-proof',type=Path,required=True);p.add_argument('--no-concurrent-dlpad-probes',action='store_true',required=True);a=p.parse_args()
 proof=plain(a.proof);out=plain(a.output)
 if not out.is_relative_to(n.ROOT/'docs/validation/evidence')or out.exists():raise ValueError('fresh project output')
 with proof.open('rb')as f:raw=f.read(8*1024*1024+1)
 if len(raw)>8*1024*1024:raise ValueError('proof bound')
 d=json.loads(raw);before=n.snapshot()
 require_current_whole(d,before)
 featurepath=plain(a.feature_proof)
 with featurepath.open('rb')as f:feature_raw=f.read(8*1024*1024+1)
 if len(feature_raw)>8*1024*1024:raise ValueError('feature proof bound')
 if sha(featurepath)!=hashlib.sha256(feature_raw).hexdigest():raise ValueError("feature proof changed after decode")
 feature=json.loads(feature_raw)
 if feature.get('passed')is not True or feature.get('status')!='REVIEW_REQUIRED' or feature.get('diagnostic_only')is not True or feature.get('default_budget_acceptance')is not False or feature.get('features')!='cpu,first-paint-fixture' or feature.get('target')!='x86_64-pc-windows-msvc' or feature.get('source_before')!=before or feature.get('source_after')!=before:raise ValueError('current diagnostic feature proof')
 from windows_selected_gui_collect import guard,bounded
 if set(feature.get('operations',{}))!={'fmt','clippy','test','release'}:raise ValueError('four exact feature operations')
 for name,op in feature['operations'].items():
  guard(op,bounded(featurepath.parent/(name+'.log')))
  if op.get('source_before')!=before or op.get('source_after')!=before:raise ValueError('feature operation exact source')
  argv=op.get('argv',[]);command={'fmt':'fmt','clippy':'clippy','test':'test','release':'build'}[name]
  if len(argv)<4 or argv[3]!=command:raise ValueError('feature actual Cargo command')
  if name=='fmt':
   if argv[4:]!=['--all','--','--check']:raise ValueError('feature whole format check')
  else:
   if '--workspace'not in argv or '--locked'not in argv or argv.count('--features')!=1 or argv[argv.index('--features')+1:argv.index('--features')+2]!=['cpu,first-paint-fixture'] or argv.count('--target')!=1 or argv[argv.index('--target')+1:argv.index('--target')+2]!=['x86_64-pc-windows-msvc']:raise ValueError('feature locked exact workspace target')
   if name=='clippy'and('--all-targets'not in argv or argv[-3:]!=['--','-D','warnings']):raise ValueError('feature strict all targets')
   if name=='release'and'--release'not in argv:raise ValueError('feature actual release')
 for path,pin in feature['originals'].items():
  file=plain(Path(path))
  if {'sha256':sha(file),'bytes':file.stat().st_size}!=pin:raise ValueError('feature original input preservation')
 artifact=feature.get('artifact')
 if not isinstance(artifact,dict) or set(artifact)!={'sha256','bytes'}:raise ValueError('exact CPU artifact required')
 SHA=artifact['sha256'];cpu_bytes=artifact['bytes']
 if not isinstance(SHA,str) or re.fullmatch('[0-9a-f]{64}',SHA)is None or type(cpu_bytes)is not int or not 0<cpu_bytes<=MAX_CPU:raise ValueError('malformed CPU artifact')
 decoded_proof_sha=hashlib.sha256(raw).hexdigest()
 if sha(proof)!=decoded_proof_sha:raise ValueError('proof replaced after decode')
 gnu_pins={str(x):sha(x)for x in [n.TOOLS/'target/release/dlpad-cpu.exe',n.TOOLS/'target/release/dlpad-local-install.exe']}
 cpu=plain(Path(feature['immutable_binary']));source=plain(n.ROOT/'scripts/windows_frame_phase_cost_probe.cs');host=plain(n.TOOLS/'windows-cli-host.exe')
 if cpu.stat().st_size!=cpu_bytes or sha(cpu)!=SHA:raise ValueError('current CPU mismatch')
 out.mkdir(mode=0o700);inputs=out/'inputs';inputs.mkdir(mode=0o700)
 for src,name in [(cpu,'cpu.exe'),(source,'observer.cs'),(Path(__file__).absolute(),'collector.py'),(proof,'whole-proof.json'),(featurepath,'feature-proof.json')]:copy(src,inputs/name)
 if sha(inputs/'feature-proof.json')!=hashlib.sha256(feature_raw).hexdigest() or sha(inputs/'whole-proof.json')!=decoded_proof_sha or sha(inputs/'cpu.exe')!=SHA or (inputs/'cpu.exe').stat().st_size!=cpu_bytes:raise ValueError('immutable proof/CPU copy mismatch')
 pins={str(x):sha(x)for x in [cpu,source,host,proof,featurepath,*inputs.iterdir()]};_,env=n.configure();exe=out/'observer.exe';framework=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319';ops=[];result={'schema':1,'status':'FAIL','source_before':before,'pins':pins,'operations':ops,'quiet_host_verified':False,'physical_ime_dpi_reader_verified':False,'root_db_pixel_review_pending':True,'target':'x86_64-pc-windows-msvc','gnu_preservation_pins':gnu_pins,'historical_gnu_cost_reused':False,'cpu_artifact':artifact,'whole_proof_sha256':decoded_proof_sha,'historical_msvc_cost_reused':False,'default_budget_acceptance':False,'diagnostic_only':True}
 try:
  op=n.run('compile',[str(host),'45',framework+'\\csc.exe','/nologo','/reference:System.Drawing.dll','/reference:System.Web.Extensions.dll',*['/reference:'+framework+'\\WPF\\'+x+'.dll'for x in ['UIAutomationClient','UIAutomationTypes','WindowsBase']],'/out:'+n.win(exe),n.win(inputs/'observer.cs')],env,out,45);ops.append(op)
  if not op['passed']:raise ValueError('compile failed')
  pins[str(exe)]=sha(exe);write(out/'compiled-pins.json',pins)
  op=n.run('idle',[str(host),'170',n.win(exe),n.win(inputs/'cpu.exe'),SHA,n.win(out/'gui'),n.win(inputs/'observer.cs'),'--no-concurrent-dlpad-probes'],env,out,170);ops.append(op)
  with(out/'gui/result.json').open('rb')as f:raw=f.read(4*1024*1024+1)
  if len(raw)>4*1024*1024:raise ValueError('metrics bound')
  metrics=json.loads(raw)
  if not op['passed']or metrics['status']!='DIAGNOSTIC_ONLY'or len(metrics['rounds'])!=1:raise ValueError('actual strict budget failure')
  with(out/'idle.log').open('r',encoding='utf-8-sig')as f:terminal_log=f.read(8*1024*1024+1)
  if len(terminal_log)>8*1024*1024:raise ValueError('terminal raw bound')
  terminals=[line for line in terminal_log.splitlines()if line.startswith('dlpad-owned-job-terminal ')]
  if len(terminals)!=1 or not all(flag in terminals[0]for flag in ['natural-zero=True','final-zero=True','final-active=0','forced=False','job-closed=True','private-desktop-closed=True']):raise ValueError('explicit outer natural terminal')
  for round in metrics['rounds']:
   cost=round['cost'];windows=cost['fixed_nonoverlapping_ten_second_windows']
   if len(windows)!=6:raise ValueError('six diagnostic windows required')
   for window in windows:
    phase=window['frame_phase'];begin=phase['begin'];end=phase['end']
    if phase['generation_snapshots_consistent']is not True or phase['job_thread_phase_boundaries_atomic']is not False or phase['instrumented_artifact_default_acceptance']is not False:raise ValueError('diagnostic snapshot scope')
    for snapshot in [begin,end]:
     if type(snapshot['generation'])is not int or snapshot['generation']<=0 or snapshot['generation']%2 or snapshot['QueryFailed']!=0 or snapshot['Overflow']!=0:raise ValueError('invalid property snapshot')
    for name in ['Frames','CacheHits','HiddenFrames','FrameWallUs','FrameCpu100ns','UiCpu100ns','TessCpu100ns','PaintCpu100ns','PresentCpu100ns','QueryFailed','Overflow','UiWallUs','TessWallUs','PaintWallUs','PresentWallUs','FrameCycles','UiCycles','TessCycles','PaintCycles','PresentCycles']:
     if type(begin[name])is not int or type(end[name])is not int or end[name]<begin[name]or phase[name]!=end[name]-begin[name]:raise ValueError('phase exact deltas')
     cap=300000000 if name.endswith('WallUs')else 10000000000000 if name.endswith('Cycles')else 3000000000 if name.endswith('100ns')else 10000 if name in ['Frames','CacheHits','HiddenFrames']else 0
     if begin[name]<0 or end[name]>cap:raise ValueError('bounded cumulative phase fields')
    if phase['wake_counts_are_exclusive_frame_causes']is not False or phase['egui_callback_semantic_caller_attributed']is not False:raise ValueError('wake attribution scope')
    for name in WAKE_NAMES:
     if any(type(snapshot[name])is not int for snapshot in [begin,end,phase]) or not 0<=begin[name]<=end[name]<=1000000 or phase[name]!=end[name]-begin[name]:raise ValueError('wake exact bounded monotonic delta')
    for snapshot in [begin,end,phase]:
     if snapshot['MetricsChecks']!=snapshot['MetricsUnchanged']+snapshot['MetricsRedraws']:raise ValueError('atomic metrics decision equality')
     if snapshot['CacheHits']>snapshot['Frames']:raise ValueError('phase cache hit count exceeds frames')
     for suffix,total in [('WallUs','FrameWallUs'),('Cycles','FrameCycles'),('Cpu100ns','FrameCpu100ns')]:
      if sum(snapshot[prefix+suffix]for prefix in ['Ui','Tess','Paint','Present'])>snapshot[total]:raise ValueError('phase sum exceeds total')
    cycle=window['main_thread_cycles']
    if type(cycle['begin_cycles'])is not int or type(cycle['end_cycles'])is not int or cycle['begin_cycles']<0 or cycle['end_cycles']<cycle['begin_cycles']or cycle['delta_cycles']!=cycle['end_cycles']-cycle['begin_cycles']or cycle['delta_cycles']>10000000000000:raise ValueError('main original cycle delta')
    matching=[r for r in round['cost']['thread_attribution']['threads']if r['role']=='original PI main thread'and r['tid']==cycle['main_tid']]
    if len(matching)!=1 or matching[0]['creation']!=cycle['main_thread_creation_filetime']or cycle['cycles_are_time']is not False or cycle['hardware_cpu_variation_uncontrolled']is not True or cycle['phase_job_thread_boundaries_atomic']is not False:raise ValueError('original cycle identity/scope')

   for previous,current in zip(windows,windows[1:]):
    if previous['main_thread_cycles']['end_cycles']!=current['main_thread_cycles']['begin_cycles']:raise ValueError('cycle window continuity')
    if previous['frame_phase']['end']!=current['frame_phase']['begin']:raise ValueError('frame window snapshot continuity')
   if any(round.get(k)is not True for k in ['original_process_exited','owned_job_naturally_empty','owned_job_empty_after_cleanup','private_desktop_closed','input_desktop_unchanged','mta_desktop_restored'])or round.get('owned_job_active_before_cleanup')!=0 or ('owned_job_terminated'in round and round['owned_job_terminated']is not False):raise ValueError('natural private cleanup')
  if before!=n.snapshot()or any(sha(Path(x))!=h for x,h in {**pins,**gnu_pins}.items()):raise ValueError('current inputs changed')
  result.update(status='DIAGNOSTIC_ONLY',default_budget_acceptance=False,feature_proof_sha256=sha(featurepath),source_after=n.snapshot(),actual_collector_passed=True,cleanup_flag_policy='Absent owned_job_terminated accepted only with original natural-empty and active0 plus explicit outer forcedFalse terminal; no synthesized field',metrics_result_sha256=sha(out/'gui/result.json'))
 except BaseException as e:result['failure_kind']=type(e).__name__;raise
 finally:write(out/'collector-result.json',result)
 return 0
if __name__=='__main__':raise SystemExit(main())
