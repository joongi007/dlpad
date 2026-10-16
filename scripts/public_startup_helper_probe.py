#!/usr/bin/env python3
"""Public engine startup cleanup using a copied test binary and adjacent owned fake helper."""
import argparse,json,os,select,shutil,signal,subprocess,sys,tempfile,time
from pathlib import Path
import systemd_user_guardian_probe as base
from accessible_download_probe import file_hash

HELPER=r'''#!/usr/bin/python3
import json,os,socket,sys,time
from pathlib import Path
root=Path(os.environ['DLPAD_SERVICE_TEST_ROOT']);mode=os.environ['DLPAD_PUBLIC_STARTUP_MODE']
def write(name,value):
 path=root/name;temporary=root/(name+'.owned.tmp');temporary.write_text(json.dumps(value));temporary.replace(path)
fields=Path('/proc/self/stat').read_text().rsplit(')',1)[1].split()
record={'pid':os.getpid(),'start_ticks':int(fields[19]),'parent_pid':os.getppid(),'mode':mode,'started_monotonic':time.monotonic()}
control=socket.socket(fileno=int(sys.argv[2]));control.settimeout(20)
write('launcher.json',record)
os.write(1,b'owned startup stdout marker\n');os.write(2,b'owned startup stderr marker\n')
if mode=='EP':control.sendall(b'EP')
data=control.recv(16)
write('control-eof.json',{'eof':data==b'','observed_monotonic':time.monotonic()})
# Remain alive with stdout/stderr open, requiring the public owned-kill path.
time.sleep(30)
'''

def execute(name,folder,out,source):
 result={'case':name,'started_utc':base.utc_now(),'actions':[]};child=None;records={};fds={}
 env=dict(os.environ)
 for key in ('DLPAD_TEST_SYSTEMD_BUS_PATH','DLPAD_TEST_SYSTEMD_STARTUP_FAULT'):env.pop(key,None)
 env.update(DLPAD_GUARDIAN_BACKEND='direct',DLPAD_SERVICE_TEST_ROOT=str(folder),DLPAD_SERVICE_TEST_CASE='public_startup_'+name,DLPAD_PUBLIC_STARTUP_MODE='EP' if name=='EP' else 'noR')
 driver=folder/'copied-engine-driver';shutil.copyfile(source,driver);driver.chmod(0o500)
 helper=folder/'dlpad-process-guard';helper.write_text(HELPER);helper.chmod(0o500)
 (folder/'worker.py').write_text("from pathlib import Path\nimport sys\nPath(sys.argv[1],'UNEXPECTED_TOOL_START').write_text('not permitted')\n")
 try:
  if file_hash(driver)!=file_hash(source):raise RuntimeError('copied driver mismatch')
  started=time.monotonic()
  with (out/(name+'.driver.log')).open('xb') as log:
   child=subprocess.Popen([str(driver),'--ignored','--exact','user_service_driver','--nocapture'],env=env,stdin=subprocess.DEVNULL,stdout=log,stderr=subprocess.STDOUT,close_fds=True)
   records['owner']=base.identity(child.pid);fds['owner']=os.pidfd_open(child.pid)
   deadline=time.monotonic()+5
   while not (folder/'launcher.json').exists():
    if child.poll() is not None:raise RuntimeError('public driver exited before helper observation')
    if time.monotonic()>deadline:raise RuntimeError('launcher startup observation timed out')
    time.sleep(.01)
   launcher=json.loads((folder/'launcher.json').read_text());records['launcher']=launcher
   if not base.same_live(launcher) or launcher['parent_pid']!=child.pid:raise RuntimeError('launcher identity/parent mismatch')
   fds['launcher']=os.pidfd_open(launcher['pid']);result['owned_identities']=records
   result['driver_exit_code']=child.wait(timeout=max(1,23-(time.monotonic()-started)))
  result['elapsed_seconds']=time.monotonic()-started
  data=(folder/'result.json').read_bytes()
  if len(data)>128*1024:raise RuntimeError('driver evidence bound exceeded')
  execution=json.loads(data);result['execution']=execution
  eof=json.loads((folder/'control-eof.json').read_text());result['control_eof']=eof
  result['eof_before_launcher_reap']=eof['eof'] and eof['observed_monotonic']<time.monotonic()
  result['tool_not_started']=not (folder/'UNEXPECTED_TOOL_START').exists()
  result['owned_processes_gone']={role:not base.same_identity(value) for role,value in records.items()}
  result['pidfds_ready']={role:bool(select.select([fd],[],[],0)[0]) for role,fd in fds.items()}
  result['fd_stable']=execution['fd_before']==execution['fd_after']
  r=execution['runs'][0]
  result['expected_error']=not r['ok'] and ((name=='EP' and r.get('error_kind')=='Other' and r.get('error')=='Linux user-service protection setup failed') or (name=='noR' and r.get('error_kind') in ('WouldBlock','TimedOut')))
  result['finite_deadline_passed']=result['elapsed_seconds']<23
  result['readiness_timeout_observed']=name!='noR' or eof['observed_monotonic']-launcher['started_monotonic']>=9
  result['natural_grace_before_kill_observed']=result['elapsed_seconds']-(eof['observed_monotonic']-launcher['started_monotonic'])>=4.5
  result['driver_log_bytes']=(out/(name+'.driver.log')).stat().st_size
  result['passed']=result['driver_exit_code']==0 and result['driver_log_bytes']<=128*1024 and all(result['owned_processes_gone'].values()) and all(result['pidfds_ready'].values()) and all(result[key] for key in ('eof_before_launcher_reap','tool_not_started','fd_stable','expected_error','finite_deadline_passed','readiness_timeout_observed','natural_grace_before_kill_observed'))
  result['state']='completed'
 except Exception as error:result.update(passed=False,state='failed',error=str(error))
 finally:
  for role,fd in fds.items():
   if base.same_live(records[role]):signal.pidfd_send_signal(fd,signal.SIGKILL);result['actions'].append('cleanup exact owned '+role+' pidfd')
   os.close(fd)
  if child is not None:
   if child.poll() is None:child.kill()
   child.wait(timeout=5)
  result['ended_utc']=base.utc_now();base.write_json(out/(name+'.json'),result)
 return result

def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('test_binary',type=Path);parser.add_argument('output',type=Path);args=parser.parse_args()
 source=args.test_binary
 if not source.is_absolute() or source.is_symlink() or not source.is_file():parser.error('explicit existing test binary required')
 project=Path(__file__).resolve().parent.parent;evidence=project/'docs/validation/evidence';out=args.output
 if not out.is_absolute() or '..' in out.parts or not out.is_relative_to(evidence) or out==evidence or out.exists() or any(p.is_symlink() for p in [out,*out.parents]):parser.error('new explicit project evidence required')
 out.mkdir(mode=0o700);result={'started_utc':base.utc_now(),'test_binary':str(source),'test_sha256_before':file_hash(source),'probe_sha256':file_hash(Path(__file__)),'scope':'public process::run startup child cleanup; adjacent fake helper; no manager/cgroup claim','cases':{}}
 with tempfile.TemporaryDirectory(prefix='dlpad-public-startup-owned-',dir='/tmp') as tmp:
  folder=Path(tmp);sentinel=subprocess.Popen([sys.executable,'-c','import time;time.sleep(120)'],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL);record=base.identity(sentinel.pid)
  try:
   for name in ('EP','noR'):
    case=folder/name;case.mkdir();result['cases'][name]=execute(name,case,out,source)
   result['sentinel_unchanged_alive']=base.same_live(record)
  finally:sentinel.terminate();sentinel.wait(timeout=5)
 result['owned_temp_removed']=not folder.exists();result['test_sha256_after']=file_hash(source);result['passed']=all(c['passed'] for c in result['cases'].values()) and result['sentinel_unchanged_alive'] and result['owned_temp_removed'] and result['test_sha256_before']==result['test_sha256_after'];result['ended_utc']=base.utc_now();base.write_json(out/'result.json',result)
 print(json.dumps({'passed':result['passed'],'cases':{name:c['passed'] for name,c in result['cases'].items()}}));return 0 if result['passed'] else 2
if __name__=='__main__':sys.exit(main())
