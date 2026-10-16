#!/usr/bin/env python3
"""Explicit production engine/user-service capability tests; owned fixtures only."""
import argparse, hashlib, json, os, select, signal, stat, subprocess, sys, tempfile, time
from pathlib import Path
import systemd_user_guardian_probe as base

STARTUP_CASES={'startup_after_start':'after-start','startup_after_get_unit':'after-get-unit','startup_after_invocation':'after-invocation'}
EXPECTED=['$DLPAD_FIXTURE_VAR','${DLPAD_FIXTURE_VAR}','prefix-${DLPAD_FIXTURE_VAR}','literal with spaces','한글','%n',"quote'\"literal"]
WORKER=r'''import json,os,signal,subprocess,sys,time
from pathlib import Path
root=Path(sys.argv[1]);case=sys.argv[2]
if case not in ('child','leaf'):(root/'tool-started').write_text('owned synthetic start marker')
def record():
 fields=Path('/proc/self/stat').read_text().rsplit(')',1)[1].split()
 return {'pid':os.getpid(),'parent_pid':os.getppid(),'start_ticks':int(fields[19]),'setsid':os.getsid(0)==os.getpid()}
if case in ('child','leaf'):
 if case=='child':
  os.setsid();subprocess.Popen([sys.executable,__file__,str(root),'leaf'],stdin=subprocess.DEVNULL,close_fds=True)
 (root/(case+'.json')).write_text(json.dumps(record()));time.sleep(30)
else:
 signal.signal(signal.SIGTERM,signal.SIG_IGN)
 subprocess.Popen([sys.executable,__file__,str(root),'child'],stdin=subprocess.DEVNULL,close_fds=True)
 deadline=time.monotonic()+5
 while not all((root/(role+'.json')).exists() for role in ('child','leaf')):
  if time.monotonic()>deadline:raise RuntimeError('fixture descendants startup timeout')
  time.sleep(.01)
 value=record();value.update(argv=sys.argv[3:],environment=os.environ.get('DLPAD_FIXTURE_VAR'),stdin_null=os.readlink('/proc/self/fd/0')=='/dev/null')
 (root/'worker.json').write_text(json.dumps(value))
 if case=='metadata':
  print(json.dumps({'argv':sys.argv[3:],'environment':os.environ.get('DLPAD_FIXTURE_VAR'),'stdin_null':value['stdin_null']},ensure_ascii=False),flush=True)
  print('metadata stderr marker',file=sys.stderr,flush=True);time.sleep(.4);sys.exit(0)
 print('engine stdout marker',flush=True);print('engine stderr marker',file=sys.stderr,flush=True)
 if case=='backpressure':
  time.sleep(.4)
  for i in range(20000):
   os.write(1,b'x'*100+b'\n');os.write(2,b'y'*100+b'\n')
  sys.exit(0)
 if case=='outputlimit':
  time.sleep(.4)
  os.write(1,b'z'*100000);time.sleep(30)
 elif case in ('exit0','exit42'):
  time.sleep(.4);sys.exit(42 if case=='exit42' else 0)
 else:time.sleep(30)
'''

def sha(path):
 h=hashlib.sha256()
 with path.open('rb') as stream:
  for block in iter(lambda:stream.read(65536),b''):h.update(block)
 return h.hexdigest()

def wait_child(child,seconds):
 deadline=time.monotonic()+seconds
 while child.poll() is None:
  if time.monotonic()>deadline:raise RuntimeError('owned driver deadline exceeded')
  time.sleep(.02)
 return child.returncode

def wait_fixture(folder,driver):
 deadline=time.monotonic()+6
 while not (folder/'worker.json').exists():
  if driver.poll() is not None:return None
  if time.monotonic()>deadline:raise RuntimeError('fixture startup timeout')
  time.sleep(.01)
 return {role:json.loads((folder/(role+'.json')).read_text()) for role in ('worker','child','leaf')}

def check_execution(name,value):
 runs=value['runs']
 if name in ('missing_manager','unknown_backend'):
  if len(runs)!=1 or runs[0]['ok']:return False
  return name=='unknown_backend' or (runs[0].get('error_kind')=='Other' and runs[0].get('error')=='Linux user-service protection setup failed')
 if name in STARTUP_CASES:
  if len(runs)!=1:return False
  r=runs[0]
  return (not r['ok'] and r.get('error_kind')=='Other' and r.get('error')=='Linux user-service protection setup failed') or (r['ok'] and r.get('outcome')=='Exited' and r.get('exit_code')==126 and len(r.get('logs','').encode())<=4096)
 if name=='fdleak':return len(runs)==12 and all(r['ok'] and r['exit_code']==0 for r in runs) and value['fd_after']<=value['fd_before']+1
 r=runs[0]
 if not r['ok']:return name in ('guardian_sigkill','owner_guardian_sigkill')
 if len(r['logs'].encode())>4096:return False
 if name=='metadata':
  data=json.loads(r['metadata']);return r['exit_code']==0 and data=={'argv':EXPECTED,'environment':'expanded sentinel','stdin_null':True} and 'metadata stderr marker' in r['logs']
 if name=='cancel':return r['outcome']=='Cancelled'
 if name=='timeout':return r['outcome']=='TimedOut'
 if name=='outputlimit':return r['outcome']=='OutputLimit'
 if name=='backpressure':return r['exit_code']==0 and r['truncated']
 if name in ('exit0','exit42'):return r['exit_code']==(42 if name=='exit42' else 0) and all(s in r['logs'] for s in ('engine stdout marker','engine stderr marker'))
 return name=='guardian_sigkill'

def execute(name,folder,out,binary,env):
 result={'case':name,'started_utc':base.utc_now(),'actions':[]};records={};fds={};driver=None;unit=None;invocation=None
 (folder/'worker.py').write_text(WORKER)
 caseenv=dict(env);caseenv.update(DLPAD_GUARDIAN_BACKEND='unknown-fixture-backend' if name=='unknown_backend' else 'user-systemd',DLPAD_SERVICE_TEST_CASE=name,DLPAD_SERVICE_TEST_ROOT=str(folder),DLPAD_FIXTURE_VAR='expanded sentinel')
 caseenv.pop('DLPAD_TEST_SYSTEMD_STARTUP_FAULT',None)
 caseenv.pop('DLPAD_TEST_SYSTEMD_BUS_PATH',None)
 if name=='missing_manager':caseenv['DLPAD_TEST_SYSTEMD_BUS_PATH']=str(folder/'missing-owned-bus')
 if name in STARTUP_CASES:caseenv['DLPAD_TEST_SYSTEMD_STARTUP_FAULT']=STARTUP_CASES[name]
 log=out/(name+'.driver.log')
 try:
  with log.open('xb') as stream:
   driver=subprocess.Popen([str(binary),'--ignored','--exact','user_service_driver','--nocapture'],env=caseenv,stdin=subprocess.DEVNULL,stdout=stream,stderr=subprocess.STDOUT,close_fds=True)
   records['owner']=base.identity(driver.pid);fds['owner']=os.pidfd_open(driver.pid)
   fixture=wait_fixture(folder,driver) if name not in ('fdleak','missing_manager','unknown_backend') else None
   if name in STARTUP_CASES and fixture is None:raise RuntimeError('startup fault tool/unit not observed: acceptance unverified')
   if fixture:
    records.update(fixture);guardian_pid=fixture['worker']['parent_pid'];guardian=base.identity(guardian_pid)
    if guardian is None:raise RuntimeError('guardian exited before ownership verification')
    records['guardian']=guardian;cgroup=base.cgroup_of(guardian_pid);unit=cgroup.rsplit('/',1)[-1]
    if not unit.endswith('.service') or not cgroup.startswith(f'/user.slice/user-{os.getuid()}.slice/user@{os.getuid()}.service/'):raise RuntimeError('fixture is not in expected user service')
    p=base.props(unit,env)['properties'];invocation=p.get('InvocationID')
    if not invocation or len(invocation)!=32:raise RuntimeError('InvocationID missing')
    result.update(unit=unit,invocation_id=invocation,control_group=cgroup,initial_unit=base.verify_unit(unit,invocation,guardian,env))
    for role in ('worker','child','leaf','guardian'):
     record=records[role]
     if not base.same_live(record) or base.cgroup_of(record['pid'])!=cgroup:raise RuntimeError('fixture identity/cgroup mismatch')
     fds[role]=os.pidfd_open(record['pid'])
    result['literal_argv_passed']=fixture['worker']['argv']==EXPECTED
    result['environment_passed']=fixture['worker']['environment']=='expanded sentinel'
    result['stdin_null']=fixture['worker']['stdin_null'];result['setsid_child']=fixture['child']['setsid']
    if name in ('owner_sigkill','guardian_sigkill','owner_guardian_sigkill'):
     base.verify_unit(unit,invocation,guardian,env)
     for role in (('owner','guardian') if name=='owner_guardian_sigkill' else ('owner',) if name=='owner_sigkill' else ('guardian',)):
      signal.pidfd_send_signal(fds[role],signal.SIGKILL);result['actions'].append('SIGKILL exact '+role+' pidfd')
   result['driver_exit_code']=wait_child(driver,16)
  result['driver_log_bytes']=log.stat().st_size
  if result['driver_log_bytes']>128*1024:raise RuntimeError('driver log bound exceeded')
  if (folder/'result.json').exists():
   data=(folder/'result.json').read_bytes()
   if len(data)>128*1024:raise RuntimeError('driver result bound exceeded')
   result['execution']=json.loads(data);result['execution_passed']=check_execution(name,result['execution'])
   if name in STARTUP_CASES:
    result['startup_stage']=STARTUP_CASES[name]
    first=result['execution']['runs'][0]
    result['startup_failure_path']='EP-before-readiness' if not first['ok'] else 'R-before-launcher-failure'
    result['startup_fd_stable']=result['execution']['fd_after']==result['execution']['fd_before']
    result['startup_pipe_deadline_passed']=result['execution']['elapsed_millis']<12000
    result['execution_passed']=result['execution_passed'] and result['startup_fd_stable'] and result['startup_pipe_deadline_passed']
  else:result['execution_passed']=name in ('owner_sigkill','owner_guardian_sigkill') and result['driver_exit_code']==-signal.SIGKILL
  if name in ('missing_manager','unknown_backend'):result['tool_not_started']=not (folder/'tool-started').exists()
  deadline=time.monotonic()+6
  while any(base.same_identity(v) for v in records.values()) and time.monotonic()<deadline:time.sleep(.02)
  result['owned_identities']=records;result['owned_processes_gone']={role:not base.same_identity(v) for role,v in records.items()}
  result['pidfds_ready']={role:bool(select.select([fd],[],[],0)[0]) for role,fd in fds.items()}
  if unit:
   cp=Path('/sys/fs/cgroup')/result['control_group'].lstrip('/');result['cgroup_removed']=not cp.exists()
   deadline=time.monotonic()+3
   while time.monotonic()<deadline:
    terminal=base.props(unit,env)['properties']
    if terminal.get('LoadState')=='not-found':break
    time.sleep(.02)
   result['terminal_unit']=terminal;result['owned_unit_gc']=terminal.get('LoadState')=='not-found'
  result['passed']=result['execution_passed'] and all(result['owned_processes_gone'].values()) and all(result['pidfds_ready'].values()) and (not unit or (result['cgroup_removed'] and result['owned_unit_gc'])) and result.get('tool_not_started',True) and all(result.get(key,True) for key in ('literal_argv_passed','environment_passed','stdin_null','setsid_child'))
  result['state']='completed'
 except Exception as error:result.update(state='failed',error=str(error),passed=False)
 finally:
  for role,fd in fds.items():
   if base.same_live(records[role]):signal.pidfd_send_signal(fd,signal.SIGKILL);result['actions'].append('cleanup exact owned '+role+' pidfd')
   os.close(fd)
  if driver is not None:
   if driver.poll() is None:driver.kill()
   driver.wait(timeout=5)
  result['ended_utc']=base.utc_now();base.write_json(out/(name+'.json'),result)
 return result

def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('test_binary',type=Path);parser.add_argument('output',type=Path);parser.add_argument('--case',action='append',choices=['exit0','exit42','metadata','cancel','timeout','owner_sigkill','guardian_sigkill','owner_guardian_sigkill','missing_manager','unknown_backend','fdleak','backpressure','outputlimit',*STARTUP_CASES]);args=parser.parse_args()
 project=Path(__file__).resolve().parent.parent;out=args.output;evidence=project/'docs/validation/evidence'
 if not out.is_absolute() or '..' in out.parts or not out.is_relative_to(evidence) or out==evidence or out.exists() or any(p.is_symlink() for p in [out,*out.parents]):parser.error('new explicit project evidence path required')
 binary=args.test_binary.resolve()
 if not binary.is_file() or binary.is_symlink():parser.error('existing integration test binary required')
 helper=binary.parent.parent/'dlpad-process-guard' if binary.parent.name=='deps' else binary.parent/'dlpad-process-guard'
 if not helper.is_file():parser.error('adjacent guardian helper required')
 runtime=Path('/run/user')/str(os.getuid());env=dict(os.environ)
 for path in (runtime/'bus',runtime/'systemd/private'):
  info=path.lstat()
  if not stat.S_ISSOCK(info.st_mode) or info.st_uid!=os.getuid():parser.error('owned user manager socket missing')
 env['XDG_RUNTIME_DIR']=str(runtime);env['DBUS_SESSION_BUS_ADDRESS']='unix:path='+str(runtime/'bus');out.mkdir(mode=0o700)
 result={'started_utc':base.utc_now(),'boot_id':Path('/proc/sys/kernel/random/boot_id').read_text().strip(),'test_binary':str(binary),'test_sha256':sha(binary),'helper':str(helper),'helper_sha256':sha(helper),'source_sha256':sha(Path(__file__)),'cases':{},'existing_units_targeted':False}
 names=args.case or ['exit0','exit42','metadata','cancel','timeout','owner_sigkill','guardian_sigkill','owner_guardian_sigkill','missing_manager','unknown_backend','fdleak','backpressure','outputlimit']
 with tempfile.TemporaryDirectory(prefix='dlpad-user-engine-owned-',dir='/tmp') as tmp:
  folder=Path(tmp);sentinel=subprocess.Popen([sys.executable,'-c','import time;time.sleep(300)'],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL);record=base.identity(sentinel.pid)
  try:
   for name in names:
    work=folder/name;work.mkdir();result['cases'][name]=execute(name,work,out,binary,env)
   result['sentinel_unchanged_alive']=base.same_live(record)
  finally:sentinel.terminate();sentinel.wait(timeout=5)
 result['owned_temp_removed']=not folder.exists();result['binary_unchanged']=sha(binary)==result['test_sha256'] and sha(helper)==result['helper_sha256'];result['passed']=all(c['passed'] for c in result['cases'].values()) and result['sentinel_unchanged_alive'] and result['owned_temp_removed'] and result['binary_unchanged'];result['ended_utc']=base.utc_now();base.write_json(out/'result.json',result)
 print(json.dumps({'passed':result['passed'],'cases':{k:v['passed'] for k,v in result['cases'].items()}}));return 0 if result['passed'] else 2
if __name__=='__main__':sys.exit(main())
