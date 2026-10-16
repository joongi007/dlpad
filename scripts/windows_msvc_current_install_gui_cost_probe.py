"""Current fixed CPU private 3x60s/18-window budget collection; root review required."""
import argparse,json,os,sys
from pathlib import Path
sys.path.insert(0,'/mnt/d/project/dlpad/scripts')
import windows_current_workspace_probe as n
from windows_msvc_workspace_probe import require_current_whole
from windows_selected_gui_collect import copy as bounded_copy
SHA='121f02cd59b2cd645fd47ae90961b6000ab6547c8883eb64d4e5ef4cd9c76022'
def plain(p):
 if not p.is_absolute()or '..'in p.parts or any(x.is_symlink()for x in [p,*p.parents]):raise ValueError('plain absolute')
 return p
def write(p,v):
 with p.open('x',encoding='utf-8')as f:json.dump(v,f,indent=2);f.write('\n')
def copy(a,b):
 bounded_copy(a,b)
def main():
 p=argparse.ArgumentParser();p.add_argument('proof',type=Path);p.add_argument('output',type=Path);p.add_argument('--no-concurrent-dlpad-probes',action='store_true',required=True);a=p.parse_args()
 proof=plain(a.proof);out=plain(a.output)
 if not out.is_relative_to(n.ROOT/'docs/validation/evidence')or out.exists():raise ValueError('fresh project output')
 with proof.open('rb')as f:raw=f.read(8*1024*1024+1)
 if len(raw)>8*1024*1024:raise ValueError('proof bound')
 d=json.loads(raw);before=n.snapshot()
 require_current_whole(d,before)
 if d.get('artifacts',{}).get('dlpad-cpu.exe')!={'sha256':SHA,'bytes':8709632}:raise ValueError('current fixed MSVC whole proof')
 gnu_pins={str(x):n.sha(x)for x in [n.TOOLS/'target/release/dlpad-cpu.exe',n.TOOLS/'target/release/dlpad-local-install.exe']}
 cpu=plain(n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release/dlpad-cpu.exe');source=plain(n.ROOT/'scripts/windows_library_thread_window_cost60_probe.cs');host=plain(n.TOOLS/'windows-cli-host.exe')
 if n.sha(cpu)!=SHA:raise ValueError('current CPU mismatch')
 out.mkdir(mode=0o700);inputs=out/'inputs';inputs.mkdir(mode=0o700)
 for src,name in [(cpu,'cpu.exe'),(source,'observer.cs'),(Path(__file__).absolute(),'collector.py'),(proof,'whole-proof.json')]:copy(src,inputs/name)
 pins={str(x):n.sha(x)for x in [cpu,source,host,proof,*inputs.iterdir()]};_,env=n.configure();exe=out/'observer.exe';framework=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319';ops=[];result={'schema':1,'status':'FAIL','source_before':before,'pins':pins,'operations':ops,'quiet_host_verified':False,'physical_ime_dpi_reader_verified':False,'root_db_pixel_review_pending':True,'target':'x86_64-pc-windows-msvc','gnu_preservation_pins':gnu_pins,'historical_gnu_cost_reused':False}
 try:
  op=n.run('compile',[str(host),'45',framework+'\\csc.exe','/nologo','/reference:System.Drawing.dll','/reference:System.Web.Extensions.dll',*['/reference:'+framework+'\\WPF\\'+x+'.dll'for x in ['UIAutomationClient','UIAutomationTypes','WindowsBase']],'/out:'+n.win(exe),n.win(inputs/'observer.cs')],env,out,45);ops.append(op)
  if not op['passed']:raise ValueError('compile failed')
  pins[str(exe)]=n.sha(exe);write(out/'compiled-pins.json',pins)
  op=n.run('idle',[str(host),'470',n.win(exe),n.win(inputs/'cpu.exe'),SHA,n.win(out/'gui'),n.win(inputs/'observer.cs'),'--no-concurrent-dlpad-probes'],env,out,470);ops.append(op)
  with(out/'gui/result.json').open('rb')as f:raw=f.read(4*1024*1024+1)
  if len(raw)>4*1024*1024:raise ValueError('metrics bound')
  metrics=json.loads(raw)
  if not op['passed']or metrics['status']!='PASS'or len(metrics['rounds'])!=3:raise ValueError('actual strict budget failure')
  with(out/'idle.log').open('r',encoding='utf-8-sig')as f:terminal_log=f.read(8*1024*1024+1)
  if len(terminal_log)>8*1024*1024:raise ValueError('terminal raw bound')
  terminals=[line for line in terminal_log.splitlines()if line.startswith('dlpad-owned-job-terminal ')]
  if len(terminals)!=1 or not all(flag in terminals[0]for flag in ['natural-zero=True','final-zero=True','final-active=0','forced=False','job-closed=True','private-desktop-closed=True']):raise ValueError('explicit outer natural terminal')
  for round in metrics['rounds']:
   cost=round['cost'];windows=cost['fixed_nonoverlapping_ten_second_windows']
   if len(windows)!=6 or cost['all_six_windows_cpu_budget_met']is not True or cost['all_six_windows_memory_budget_met']is not True:raise ValueError('all eighteen fixed windows required')
   if any(round.get(k)is not True for k in ['original_process_exited','owned_job_naturally_empty','owned_job_empty_after_cleanup','private_desktop_closed','input_desktop_unchanged','mta_desktop_restored'])or round.get('owned_job_active_before_cleanup')!=0 or ('owned_job_terminated'in round and round['owned_job_terminated']is not False):raise ValueError('natural private cleanup')
  if before!=n.snapshot()or any(n.sha(Path(x))!=h for x,h in {**pins,**gnu_pins}.items()):raise ValueError('current inputs changed')
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),actual_collector_passed=True,cleanup_flag_policy='Absent owned_job_terminated accepted only with original natural-empty and active0 plus explicit outer forcedFalse terminal; no synthesized field',metrics_result_sha256=n.sha(out/'gui/result.json'))
 except BaseException as e:result['failure_kind']=type(e).__name__;raise
 finally:write(out/'collector-result.json',result)
 return 0
if __name__=='__main__':raise SystemExit(main())
