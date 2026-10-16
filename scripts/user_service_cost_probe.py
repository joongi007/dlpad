#!/usr/bin/env python3
"""Owned native default user-service app per-PID cost; no source/helper mutation."""
import argparse, json, os, signal, subprocess, threading, time
from pathlib import Path
import user_service_app_probe as app_probe
import cpu_background_probe as background
import systemd_user_guardian_probe as service
from accessible_download_probe import file_hash

class Sampler:
 def __init__(self,binary):
  self.binary=binary;self.targets={};self.rows=[];self.phase='warmup';self.lock=threading.Lock();self.stop=threading.Event();self.omitted=0;self.idle_child_scans=[];self.thread=threading.Thread(target=self.loop,daemon=True)
 def add(self,role,record):
  if not record or not service.same_live(record):raise RuntimeError('owned cost target missing')
  if role!='worker':
   actual=Path('/proc')/str(record['pid'])/'exe';want=self.binary.stat();got=actual.stat()
   if (want.st_dev,want.st_ino)!=(got.st_dev,got.st_ino) or file_hash(actual)!=file_hash(self.binary):raise RuntimeError('cost executable ownership mismatch')
  with self.lock:self.targets[role]=record
 def set_phase(self,name):
  with self.lock:self.phase=name
 def loop(self):
  while not self.stop.wait(.1):
   with self.lock:targets=dict(self.targets);phase=self.phase
   if phase=='idle' and 'app' in targets and len(self.idle_child_scans)<200:
    app=targets['app'];children=set();marker_children=[]
    try:
     for task in list(Path(f"/proc/{app['pid']}/task").iterdir())[:128]:
      try:children.update(int(v) for v in (task/'children').read_text().split())
      except OSError:continue
     for pid in sorted(children)[:128]:
      try:
       argv=Path(f'/proc/{pid}/cmdline').read_bytes().split(b'\0')
       if len(argv)>1 and argv[1] in [b'--dlpad-internal-user-service',b'--dlpad-internal-guardian',b'--dlpad-internal-user-guardian']:marker_children.append(service.identity(pid))
      except OSError:continue
     self.idle_child_scans.append({'time':time.monotonic(),'owned_app':app,'children_examined':len(children),'guardian_marker_children':marker_children})
    except OSError:pass
   for role,record in targets.items():
    try:
     fields=Path(f"/proc/{record['pid']}/stat").read_text().rsplit(') ',1)[1].split()
     if int(fields[19])!=record['start_ticks'] or fields[0] in ('Z','X'):continue
     status=Path(f"/proc/{record['pid']}/status").read_text().splitlines();values={line.split(':',1)[0]:line.split(':',1)[1].strip() for line in status if ':' in line}
     row={'time':time.monotonic(),'phase':phase,'role':role,'pid':record['pid'],'start_ticks':record['start_ticks'],'cpu_ticks':int(fields[11])+int(fields[12]),'rss_kib':int(values['VmRSS'].split()[0]),'hwm_kib':int(values['VmHWM'].split()[0])}
     if len(self.rows)<2000:self.rows.append(row)
     else:self.omitted+=1
    except (OSError,ValueError,KeyError,IndexError):continue
 def finish(self):self.stop.set();self.thread.join(3)
 def summary(self):
  result={};hz=os.sysconf('SC_CLK_TCK')
  for phase in ['idle','active']:
   result[phase]={}
   for role in ['app','launcher','guardian','worker']:
    rows=[r for r in self.rows if r['phase']==phase and r['role']==role]
    if len(rows)<2:result[phase][role]={'observed_registered_pid':bool(rows),'measured':False,'samples':len(rows),'reason':'no registered owned PID sampled in this phase'};continue
    span=rows[-1]['time']-rows[0]['time'];result[phase][role]={'present':True,'measured':True,'samples':len(rows),'seconds':span,'cpu_percent_single_core':100*(rows[-1]['cpu_ticks']-rows[0]['cpu_ticks'])/hz/span,'rss_max_kib':max(r['rss_kib'] for r in rows),'hwm_max_kib':max(r['hwm_kib'] for r in rows)}
  return result

class Capture:
 def __init__(self,binary,sampler):self.binary=binary;self.sampler=sampler
 def __getattr__(self,name):return getattr(subprocess,name)
 def Popen(self,args,*positional,**kwargs):
  child=subprocess.Popen(args,*positional,**kwargs)
  if args and Path(args[0])==self.binary and '--fixture' in args and 'app' not in self.sampler.targets:self.sampler.add('app',service.identity(child.pid))
  return child

class IdleTime:
 def __init__(self,sampler):self.sampler=sampler;self.observed=False
 def __getattr__(self,name):return getattr(time,name)
 def sleep(self,seconds):
  time.sleep(seconds)
  if seconds==2 and not self.observed:
   self.observed=True;self.sampler.set_phase('idle');time.sleep(10);self.sampler.set_phase('pre-active')

def launcher_of(app,binary):
 candidates=set()
 for task in list(Path(f"/proc/{app['pid']}/task").iterdir())[:128]:
  try:candidates.update(int(v) for v in (task/'children').read_text().split())
  except OSError:continue
 for pid in sorted(candidates)[:128]:
  try:
   argv=Path(f'/proc/{pid}/cmdline').read_bytes().split(b'\0')
   if len(argv)>1 and argv[1]==b'--dlpad-internal-user-service':return service.identity(pid)
  except OSError:continue
 raise RuntimeError('owned launcher was not captured')

def main():
 def deadline(signum,frame):raise TimeoutError('owned user-service cost deadline')
 signal.signal(signal.SIGALRM,deadline);signal.alarm(180)
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--expected-sha',required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--font',type=Path,default=Path('/mnt/c/Windows/Fonts/malgun.ttf'));a=p.parse_args();a.release_fault_env=None
 if not a.binary.is_absolute() or not a.binary.is_relative_to(Path('/tmp')) or a.binary.is_symlink() or a.binary.name!='dlpad-cpu' or sorted(a.binary.parent.iterdir())!=[a.binary] or file_hash(a.binary)!=a.expected_sha:p.error('immutable single CPU snapshot required')
 a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
 for key in ['DLPAD_GUARDIAN_BACKEND','DLPAD_TEST_SYSTEMD_BUS_PATH']:os.environ.pop(key,None)
 runtime=Path('/run/user')/str(os.getuid());host=dict(os.environ,XDG_RUNTIME_DIR=str(runtime),DBUS_SESSION_BUS_ADDRESS='unix:path='+str(runtime/'bus'))
 r={'scope':'Linux default native GUI user-service limited cost sample','binary_sha256_before':file_hash(a.binary),'binary_bytes':a.binary.stat().st_size,'probe_sha256':file_hash(Path(__file__)),'started_utc':service.utc_now(),'host_load_before':os.getloadavg(),'user_desktop_used':False,'user_clipboard_used':False,'existing_eight_hour_processes_targeted':False,'systemd_user_manager_in_cost_total':False,'shared_rss_pages_may_be_double_counted':True,'worker_is_synthetic_external_tool':True}
 sampler=Sampler(a.binary);real_pin=background.pin_worker;real_time=background.time;real_app_subprocess=app_probe.subprocess
 def cost_pin(root,token):
  info,fd=real_pin(root,token)
  try:
   fields=Path(f"/proc/{info['pid']}/stat").read_text().rsplit(') ',1)[1].split();guardian=service.identity(int(fields[1]));group=service.cgroup_of(guardian['pid']);unit=group.rsplit('/',1)[-1]
   if not unit.startswith('dlpad-job-') or not unit.endswith('.service'):raise RuntimeError('cost worker guardian not owned service')
   props=service.props(unit,host)['properties'];invocation=props.get('InvocationID');service.verify_unit(unit,invocation,guardian,host)
   if service.cgroup_of(info['pid'])!=group:raise RuntimeError('cost worker cgroup mismatch')
   launcher=launcher_of(sampler.targets['app'],a.binary);sampler.add('launcher',launcher);sampler.add('guardian',guardian);sampler.add('worker',{'pid':info['pid'],'start_ticks':info['start_ticks']})
   r['owned_unit']={'unit':unit,'invocation_id':invocation,'guardian':guardian,'control_group':group};r['launcher']=launcher
   sampler.set_phase('active');time.sleep(10);sampler.set_phase('post-active');return info,fd
  except Exception:os.close(fd);raise
 background.pin_worker=cost_pin;background.time=IdleTime(sampler);app_probe.subprocess=Capture(a.binary,sampler);sampler.thread.start()
 try:r['native_service_proof']=app_probe.run_case(a,'active-hidden',host)
 except Exception as error:r['error']=str(error)
 finally:
  sampler.finish();background.pin_worker=real_pin;background.time=real_time;app_probe.subprocess=real_app_subprocess
 r['idle_owned_child_scans']=sampler.idle_child_scans;r['idle_owned_launcher_absence_sampled']=len(sampler.idle_child_scans)>=90 and all(not x['guardian_marker_children'] for x in sampler.idle_child_scans);r['targets']=sampler.targets;r['samples']=sampler.rows;r['omitted_samples']=sampler.omitted;r['costs']=sampler.summary();r['host_load_after']=os.getloadavg();r['binary_sha256_after']=file_hash(a.binary);r['ended_utc']=service.utc_now()
 r['owned_targets_gone']={role:not service.same_live(record) for role,record in sampler.targets.items()};r['passed']=not r.get('error') and r.get('native_service_proof',{}).get('passed',False) and all(r['owned_targets_gone'].values()) and r['binary_sha256_after']==a.expected_sha and r['idle_owned_launcher_absence_sampled'] and sampler.omitted==0 and all(r['costs']['active'][role].get('seconds',0)>=9 for role in ['app','launcher','guardian']) and r['costs']['idle']['app'].get('seconds',0)>=9
 data=json.dumps(r,ensure_ascii=False,indent=2)+'\n'
 if len(data.encode())>2*1024*1024:raise RuntimeError('cost evidence size bound')
 (a.output/'result.json').write_text(data);print(json.dumps({'passed':r['passed'],'costs':r['costs'],'error':r.get('error')},ensure_ascii=False));return 0 if r['passed'] else 1
if __name__=='__main__':raise SystemExit(main())
