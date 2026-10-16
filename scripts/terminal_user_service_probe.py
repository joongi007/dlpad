#!/usr/bin/env python3
"""Explicit native terminal acknowledgment/typed ownership regression; synthetic owned fixtures."""
import argparse, hashlib, json, os, select, shutil, signal, socket, stat, subprocess, sys, tempfile, time
from pathlib import Path
import user_service_engine_probe as engine
import systemd_user_guardian_probe as base

API_CASES=['exit0','exit42','exit126','metadata','cancel','timeout','owner_sigkill','guardian_sigkill','owner_guardian_sigkill','missing_manager','unknown_backend','fdleak','backpressure','outputlimit',*engine.STARTUP_CASES]
RAW_CASES=['raw_fast0','raw_fast42','raw_fast126']
FAKE_CASES=['missing_guardian_ack','missing_launcher_ack']
OLD_CHECK=engine.check_execution
engine.WORKER=engine.WORKER.replace("('exit0','exit42')", "('exit0','exit42','exit126')").replace("sys.exit(42 if case=='exit42' else 0)","sys.exit({'exit0':0,'exit42':42,'exit126':126}[case])")

def check_api(name,value):
 runs=value['runs']
 if name in ['guardian_sigkill','missing_manager',*engine.STARTUP_CASES]:
  return len(runs)==1 and not runs[0]['ok'] and runs[0].get('cleanup_unconfirmed') is True
 if name=='exit126':
  return len(runs)==1 and runs[0]['ok'] and runs[0]['exit_code']==126 and runs[0]['outcome']=='Exited' and 'engine stdout marker' in runs[0]['logs']
 return OLD_CHECK(name,value)
engine.check_execution=check_api

def raw_fast(name,folder,out,binary,env):
 code=int(name.removeprefix('raw_fast'));helper=binary.parent/'dlpad-process-guard'
 app,peer=socket.socketpair();child=None;pin=None;record=None;data=b'';eof=False
 result={'case':name,'scope':'raw owned launcher socket, no media/network','requested_exit_code':code}
 try:
  child=subprocess.Popen([str(helper),'--dlpad-internal-user-service',str(peer.fileno()),'--','/usr/bin/python3','-c',f"import sys; print('synthetic fast terminal'); sys.exit({code})"],pass_fds=(peer.fileno(),),env=env,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
  record=base.identity(child.pid);pin=os.pidfd_open(child.pid);peer.close();app.setblocking(False)
  deadline=time.monotonic()+12
  while not eof:
   if time.monotonic()>deadline:raise RuntimeError('raw terminal socket deadline')
   ready=select.select([app],[],[],.02)[0]
   if ready:
    chunk=app.recv(16)
    if not chunk:eof=True
    else:
     data+=chunk
     if len(data)>16:raise RuntimeError('raw fixed protocol byte bound')
  stdout,stderr=child.communicate(timeout=3)
  result.update(protocol_ascii=data.decode('ascii','replace'),socket_eof=eof,exit_code=child.returncode,stdout_bytes=len(stdout),stderr_bytes=len(stderr),stderr=stderr.decode('utf8','replace')[:2048],owned_identity=record,owned_identity_gone=not base.same_identity(record),pidfd_ready=bool(select.select([pin],[],[],0)[0]),pipe_eof=True)
  result['passed']=data==b'RGCLC' and child.returncode==code and result['owned_identity_gone'] and result['pidfd_ready'] and len(stdout)<=4096 and len(stderr)<=4096
 except Exception as error:result.update(passed=False,error=str(error),protocol_ascii=data.decode('ascii','replace'))
 finally:
  app.close();peer.close()
  if child is not None:
   if child.poll() is None:
    if pin is not None:signal.pidfd_send_signal(pin,signal.SIGKILL)
    else:child.kill()
   child.wait(timeout=5)
  if pin is not None:os.close(pin)
 base.write_json(out/(name+'.json'),result);return result

FAKE='''#!/usr/bin/python3
import json,os,socket,sys,time
from pathlib import Path
root=Path(os.environ['DLPAD_SERVICE_TEST_ROOT']);fd=int(sys.argv[2]);s=socket.socket(fileno=fd);os.set_inheritable(fd,False)
fields=Path('/proc/self/stat').read_text().rsplit(')',1)[1].split()
(root/'fakehelper.json').write_text(json.dumps({'pid':os.getpid(),'start_ticks':int(fields[19]),'parent_pid':os.getppid()}))
s.sendall(FRAME)
print('synthetic helper stdout',flush=True)
time.sleep(.01)
'''

def missing_ack(name,folder,out,binary,env):
 copied=folder/'driver';shutil.copyfile(binary,copied);copied.chmod(0o500)
 frame=b'RLC' if name=='missing_guardian_ack' else b'RGC'
 helper=folder/'dlpad-process-guard';helper.write_text(FAKE.replace('FRAME',repr(frame)));helper.chmod(0o500)
 caseenv=dict(env,DLPAD_GUARDIAN_BACKEND='user-systemd',DLPAD_SERVICE_TEST_ROOT=str(folder),DLPAD_SERVICE_TEST_CASE='exit0')
 result={'case':name,'scope':'manager-independent copied driver/fake owned helper; no tool/service started','fake_frame':frame.decode(),'fake_helper_sha256':engine.sha(helper)}
 child=None
 try:
  with (out/(name+'.driver.log')).open('xb') as log:
   child=subprocess.Popen([str(copied),'--ignored','--exact','user_service_driver','--nocapture'],env=caseenv,stdin=subprocess.DEVNULL,stdout=log,stderr=subprocess.STDOUT)
   result['driver_exit_code']=engine.wait_child(child,12)
  value=json.loads((folder/'result.json').read_text());record=json.loads((folder/'fakehelper.json').read_text())
  result.update(execution=value,helper_identity=record,helper_identity_gone=not base.same_identity(record),tool_not_started=not (folder/'tool-started').exists())
  result['passed']=child.returncode==0 and len(value['runs'])==1 and not value['runs'][0]['ok'] and value['runs'][0].get('cleanup_unconfirmed') is True and value['fd_before']==value['fd_after'] and result['helper_identity_gone'] and result['tool_not_started']
 except Exception as error:result.update(passed=False,error=str(error))
 finally:
  if child is not None:
   if child.poll() is None:child.kill()
   child.wait(timeout=5)
 base.write_json(out/(name+'.json'),result);return result

def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('test_binary',type=Path);parser.add_argument('output',type=Path);parser.add_argument('--case',action='append',choices=API_CASES+RAW_CASES+FAKE_CASES);args=parser.parse_args()
 binary=args.test_binary.resolve();out=args.output;project=Path(__file__).resolve().parent.parent;evidence=project/'docs/validation/evidence'
 if not out.is_absolute() or '..' in out.parts or not out.is_relative_to(evidence) or out.exists() or any(p.is_symlink() for p in [out,*out.parents]):parser.error('new explicit project evidence path required')
 helper=binary.parent.parent/'dlpad-process-guard' if binary.parent.name=='deps' else binary.parent/'dlpad-process-guard'
 if not binary.is_file() or not helper.is_file():parser.error('immutable driver and adjacent helper required')
 # Raw socket uses same helper location even for Cargo deps binaries.
 if binary.parent.name=='deps' and any(n in RAW_CASES for n in (args.case or RAW_CASES)):parser.error('raw cases require owned immutable copy beside helper')
 runtime=Path('/run/user')/str(os.getuid());env=dict(os.environ)
 for path in [runtime/'bus',runtime/'systemd/private']:
  info=path.lstat()
  if not stat.S_ISSOCK(info.st_mode) or info.st_uid!=os.getuid():parser.error('owned user-manager sockets required')
 env.update(XDG_RUNTIME_DIR=str(runtime),DBUS_SESSION_BUS_ADDRESS='unix:path='+str(runtime/'bus'))
 env.pop('DLPAD_TEST_SYSTEMD_BUS_PATH',None);env.pop('DLPAD_TEST_SYSTEMD_STARTUP_FAULT',None)
 out.mkdir(mode=0o700)
 result={'started_utc':base.utc_now(),'binary':str(binary),'binary_sha256':engine.sha(binary),'helper_sha256':engine.sha(helper),'script_sha256':engine.sha(Path(__file__)),'reused_engine_script_sha256':engine.sha(Path(engine.__file__)),'reused_base_script_sha256':engine.sha(Path(base.__file__)),'cases':{},'existing_units_targeted':False}
 with tempfile.TemporaryDirectory(prefix='dlpad-terminal-owned-',dir='/tmp') as tmp:
  folder=Path(tmp);sentinel=subprocess.Popen([sys.executable,'-c','import time;time.sleep(300)'],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL);sentinel_id=base.identity(sentinel.pid)
  try:
   for name in args.case or API_CASES+RAW_CASES+FAKE_CASES:
    work=folder/name;work.mkdir()
    if name in RAW_CASES:value=raw_fast(name,work,out,binary,env)
    elif name in FAKE_CASES:value=missing_ack(name,work,out,binary,env)
    else:
     value=engine.execute(name,work,out,binary,env)
     if name in engine.STARTUP_CASES:value['startup_failure_path']='typed cleanup-unconfirmed (G/L terminal contract)';base.write_json(out/(name+'.json'),value)
    result['cases'][name]=value
   result['sentinel_preserved']=base.same_live(sentinel_id)
  finally:sentinel.terminate();sentinel.wait(timeout=5)
 result.update(owned_temp_removed=not folder.exists(),binary_unchanged=engine.sha(binary)==result['binary_sha256'] and engine.sha(helper)==result['helper_sha256'],ended_utc=base.utc_now())
 result['passed']=all(v['passed'] for v in result['cases'].values()) and result['sentinel_preserved'] and result['owned_temp_removed'] and result['binary_unchanged']
 base.write_json(out/'result.json',result);print(json.dumps({'passed':result['passed'],'cases':{k:v['passed'] for k,v in result['cases'].items()}}));return 0 if result['passed'] else 2
if __name__=='__main__':raise SystemExit(main())
