#!/usr/bin/env python3
"""Actual immutable dlpad guardian in owned transient user services; no product mutations."""
import argparse, ctypes, hashlib, json, os, select, signal, socket, stat, subprocess, sys, tempfile, time, uuid
from pathlib import Path
import systemd_user_guardian_probe as base
BusError=base.BusError
EXPECTED_SHA='166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417'
EXPECTED=['$DLPAD_FIXTURE_VAR','${DLPAD_FIXTURE_VAR}','prefix-${DLPAD_FIXTURE_VAR}','literal with spaces','한글','%n',"quote'\"literal"]
WRAPPER=r"""import fcntl,json,os,sys
from pathlib import Path
binary,worker,root,case=sys.argv[1:5]
fd=fcntl.fcntl(0,fcntl.F_DUPFD,3);os.set_inheritable(fd,True);os.close(0)
Path(root,'wrapper.json').write_text(json.dumps({'pid':os.getpid(),'fd':fd}))
os.execv(binary,[binary,'--dlpad-internal-guardian',str(fd),'--',sys.executable,worker,root,case,*sys.argv[5:]])
"""
WORKER=r"""import json,os,signal,subprocess,sys,time
from pathlib import Path
root=Path(sys.argv[1]);case=sys.argv[2]
def info():
 f=Path('/proc/self/stat').read_text().rsplit(')',1)[1].split()
 return {'pid':os.getpid(),'start_ticks':int(f[19]),'parent_pid':os.getppid(),'setsid':os.getsid(0)==os.getpid()}
if case in ('child','leaf'):
 if case=='child':
  os.setsid();subprocess.Popen([sys.executable,__file__,str(root),'leaf'],stdin=subprocess.DEVNULL,close_fds=True)
 (root/(case+'.json')).write_text(json.dumps(info()));time.sleep(40)
else:
 signal.signal(signal.SIGTERM,signal.SIG_IGN)
 subprocess.Popen([sys.executable,__file__,str(root),'child'],stdin=subprocess.DEVNULL,close_fds=True)
 report=info();report['argv']=sys.argv[3:];report['stdin_null']=os.readlink('/proc/self/fd/0')=='/dev/null'
 (root/'worker.json').write_text(json.dumps(report))
 os.write(1,b'dlpad-fixture-stdout\n');os.write(2,b'dlpad-fixture-stderr\n')
 deadline=time.monotonic()+40
 while time.monotonic()<deadline:
  if (root/'exit42').exists():sys.exit(42)
  time.sleep(.01)
"""
def sha(path):
 h=hashlib.sha256()
 with path.open('rb') as stream:
  for block in iter(lambda:stream.read(65536),b''):h.update(block)
 return h.hexdigest()
def status(unit,env):
 args=['/usr/bin/systemctl','--user','show',unit,'--property=InvocationID','--property=ActiveState','--property=LoadState','--property=ExecMainCode','--property=ExecMainStatus','--property=Result']
 p=subprocess.run(args,env=env,capture_output=True,timeout=5)
 if len(p.stdout)+len(p.stderr)>65536:raise RuntimeError('status bound')
 return dict(line.split('=',1) for line in p.stdout.decode().splitlines() if '=' in line)
def wait_files(folder):
 deadline=time.monotonic()+5
 while not all((folder/(role+'.json')).exists() for role in ('worker','child','leaf')):
  if time.monotonic()>deadline:raise RuntimeError('fixture readiness timeout')
  time.sleep(.02)
 return {role:json.loads((folder/(role+'.json')).read_text()) for role in ('worker','child','leaf')}
def start_service(unit, command, control_fd, stdout_fd, stderr_fd):
    """Only StartTransientUnit(mode=fail) for this UUID; no privileged auth or shell."""
    lib=ctypes.CDLL("libsystemd.so.0")
    pointer=ctypes.c_void_p
    signatures={
        "sd_bus_new":([ctypes.POINTER(pointer)],ctypes.c_int),
        "sd_bus_set_address":([pointer,ctypes.c_char_p],ctypes.c_int),
        "sd_bus_set_bus_client":([pointer,ctypes.c_int],ctypes.c_int),
        "sd_bus_negotiate_fds":([pointer,ctypes.c_int],ctypes.c_int),
        "sd_bus_start":([pointer],ctypes.c_int),
        "sd_bus_message_new_method_call":([pointer,ctypes.POINTER(pointer),ctypes.c_char_p,ctypes.c_char_p,ctypes.c_char_p,ctypes.c_char_p],ctypes.c_int),
        "sd_bus_message_set_allow_interactive_authorization":([pointer,ctypes.c_int],ctypes.c_int),
        "sd_bus_message_open_container":([pointer,ctypes.c_char,ctypes.c_char_p],ctypes.c_int),
        "sd_bus_message_close_container":([pointer],ctypes.c_int),
        "sd_bus_message_append_basic":([pointer,ctypes.c_char,pointer],ctypes.c_int),
        "sd_bus_message_read_basic":([pointer,ctypes.c_char,pointer],ctypes.c_int),
        "sd_bus_call":([pointer,pointer,ctypes.c_uint64,ctypes.POINTER(BusError),ctypes.POINTER(pointer)],ctypes.c_int),
        "sd_bus_message_unref":([pointer],pointer),
        "sd_bus_unref":([pointer],pointer),
        "sd_bus_error_free":([ctypes.POINTER(BusError)],None),
    }
    for name,(args,result) in signatures.items():
        function=getattr(lib,name);function.argtypes=args;function.restype=result
    bus=pointer();message=pointer();reply=pointer();error=BusError()
    def check(code):
        if code<0:
            detail=(error.message or b"sd-bus call failed").decode(errors="replace")[:4096]
            raise RuntimeError(f"sd-bus error {-code}: {detail}")
        return code
    def opened(kind,contents):check(lib.sd_bus_message_open_container(message,kind.encode(),contents.encode()))
    def close():check(lib.sd_bus_message_close_container(message))
    def basic(kind,value):
        if kind=="s":item=ctypes.c_char_p(value.encode());address=ctypes.cast(item,pointer)
        else:
            item=(ctypes.c_uint64(value) if kind=="t" else ctypes.c_int(value));address=ctypes.cast(ctypes.byref(item),pointer)
        check(lib.sd_bus_message_append_basic(message,kind.encode(),address))
    def scalar_property(name,signature,value):
        opened("r","sv");basic("s",name);opened("v",signature);basic(signature,value);close();close()
    def strings_property(name,values):
        opened("r","sv");basic("s",name);opened("v","as");opened("a","s")
        for value in values:basic("s",value)
        close();close();close()
    try:
        check(lib.sd_bus_new(ctypes.byref(bus)))
        address=f"unix:path=/run/user/{os.getuid()}/bus".encode()
        check(lib.sd_bus_set_address(bus,address));check(lib.sd_bus_set_bus_client(bus,1));check(lib.sd_bus_negotiate_fds(bus,1));check(lib.sd_bus_start(bus))
        check(lib.sd_bus_message_new_method_call(bus,ctypes.byref(message),b"org.freedesktop.systemd1",b"/org/freedesktop/systemd1",b"org.freedesktop.systemd1.Manager",b"StartTransientUnit"))
        check(lib.sd_bus_message_set_allow_interactive_authorization(message,0))
        basic("s",unit);basic("s","fail");opened("a","(sv)")
        for name,value in [("Type","exec"),("KillMode","control-group"),("Restart","no"),("CollectMode","inactive")]:scalar_property(name,"s",value)
        for name,value in [("SendSIGKILL",1),("RemainAfterExit",0)]:scalar_property(name,"b",value)
        for name,value in [("TimeoutStopUSec",2000000),("RuntimeMaxUSec",45000000)]:scalar_property(name,"t",value)
        for name,value in [("StandardInputFileDescriptor",control_fd),("StandardOutputFileDescriptor",stdout_fd),("StandardErrorFileDescriptor",stderr_fd)]:scalar_property(name,"h",value)
        strings_property("Environment",["DLPAD_FIXTURE_VAR=expanded sentinel"])
        opened("r","sv");basic("s","ExecStartEx");opened("v","a(sasas)");opened("a","(sasas)");opened("r","sasas")
        basic("s",command[0]);opened("a","s")
        for arg in command:basic("s",arg)
        close();opened("a","s");basic("s","no-env-expand");close();close();close();close();close()
        close();opened("a","(sa(sv))");close()
        check(lib.sd_bus_call(bus,message,5000000,ctypes.byref(error),ctypes.byref(reply)))
        job_path=ctypes.c_char_p();check(lib.sd_bus_message_read_basic(reply,b"o",ctypes.byref(job_path)))
        return {"method":"StartTransientUnit","mode":"fail","ExecStartEx_signature":"a(sasas)","flags":["no-env-expand"],"job_path":job_path.value.decode(),"interactive_authorization":False,"stdlib_ctypes":True}
    finally:
        lib.sd_bus_error_free(ctypes.byref(error))
        if reply:lib.sd_bus_message_unref(reply)
        if message:lib.sd_bus_message_unref(message)
        if bus:lib.sd_bus_unref(bus)

def case(name,folder,out,binary,env):
 unit='dlpad-fixture-'+uuid.uuid4().hex+'.service'
 result={'case':name,'unit':unit,'started_utc':base.utc_now(),'actions':[]}
 left,right=socket.socketpair();fds={};records={};owner=None;invocation=None
 try:
  if base.props(unit,env)['properties'].get('LoadState')!='not-found':raise RuntimeError('unit collision')
  worker=folder/'worker.py';worker.write_text(WORKER);wrapper=folder/'wrapper.py';wrapper.write_text(WRAPPER)
  stdout=out/(name+'.stdout.log');stderr=out/(name+'.stderr.log')
  with stdout.open('xb') as sout,stderr.open('xb') as serr:
   command=[str(Path(sys.executable).resolve()),str(wrapper),str(binary),str(worker),str(folder),name,*EXPECTED]
   result['bus_start']=start_service(unit,command,right.fileno(),sout.fileno(),serr.fileno());right.close()
   left.settimeout(8)
   if left.recv(1)!=b'R':raise RuntimeError('actual guardian R missing')
   result['product_readiness_R']=True
   records=wait_files(folder);p=base.props(unit,env)['properties'];invocation=p.get('InvocationID')
   if not invocation or len(invocation)!=32:raise RuntimeError('invocation missing')
   guard=base.identity(int(p['MainPID']));records['guardian']=guard
   result['initial_unit']=base.verify_unit(unit,invocation,guard,env);result['invocation_id']=invocation
   cgroup=p['ControlGroup'];result['control_group']=cgroup
   if not cgroup.endswith('/'+unit) or not cgroup.startswith(f'/user.slice/user-{os.getuid()}.slice/user@{os.getuid()}.service/'):raise RuntimeError('cgroup mismatch')
   wrapper_info=json.loads((folder/'wrapper.json').read_text());result['wrapper']=wrapper_info
   if wrapper_info['pid']!=guard['pid'] or wrapper_info['fd']<3:raise RuntimeError('exec same MainPID/FD contract failed')
   actualexe=Path(f"/proc/{guard['pid']}/exe").stat();snap=binary.stat()
   if (actualexe.st_dev,actualexe.st_ino)!=(snap.st_dev,snap.st_ino):raise RuntimeError('running immutable binary mismatch')
   result['running_binary_identity']={'device':actualexe.st_dev,'inode':actualexe.st_ino}
   for role,record in records.items():
    if not base.same_live(record) or base.cgroup_of(record['pid'])!=cgroup:raise RuntimeError('owned membership identity mismatch')
    fds[role]=os.pidfd_open(record['pid'])
   result['owned_identities']=dict(records);result['all_cgroup_membership_verified']=True
   result['literal_argv_passed']=records['worker']['argv']==EXPECTED
   result['worker_stdin_null']=records['worker']['stdin_null'];result['escaped_setsid']=records['child']['setsid']
   base.verify_unit(unit,invocation,guard,env);started=time.monotonic()
   if name=='eof':left.close();result['actions'].append('close owned control endpoint')
   elif name in ('K','T','cancel'):
    left.sendall(b'T' if name!='K' else b'K');result['actions'].append('control '+('T' if name!='K' else 'K'))
    if name=='cancel':time.sleep(.02);left.sendall(b'K');result['actions'].append('control K after T')
   elif name in ('sigkill','sigterm'):
    signal.pidfd_send_signal(fds['guardian'],signal.SIGKILL if name=='sigkill' else signal.SIGTERM);result['actions'].append('exact guardian pidfd '+name)
   elif name=='owner_guardian_sigkill':
    owner=subprocess.Popen([sys.executable,'-c','import time;time.sleep(40)'],pass_fds=(left.fileno(),),stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    record=base.identity(owner.pid);records['endpoint_owner']=record;fds['endpoint_owner']=os.pidfd_open(owner.pid);left.close()
    base.verify_unit(unit,invocation,guard,env)
    signal.pidfd_send_signal(fds['endpoint_owner'],signal.SIGKILL);signal.pidfd_send_signal(fds['guardian'],signal.SIGKILL)
    result['actions'].append('exact endpoint owner and guardian pidfds SIGKILL in sequence');owner.wait(timeout=5)
   elif name=='exit42':(folder/'exit42').write_text('owned natural exit trigger')
   else:raise RuntimeError('unknown case')
   deadline=time.monotonic()+10
   while any(base.same_identity(r) for r in records.values()) and time.monotonic()<deadline:time.sleep(.02)
   result['owned_identities']=dict(records)
   result['cleanup_seconds']=time.monotonic()-started
   result['owned_processes_gone']={r:not base.same_identity(v) for r,v in records.items()}
   result['pidfds_ready']={r:bool(select.select([f],[],[],0)[0]) for r,f in fds.items()}
   terminal=status(unit,env);result['terminal_unit']=terminal
   result['exit_status_verified']=terminal.get('InvocationID')==invocation and terminal.get('ExecMainCode')==('1' if name=='exit42' else '2') and terminal.get('ExecMainStatus')==('42' if name=='exit42' else '9')
   cp=Path('/sys/fs/cgroup')/cgroup.lstrip('/');result['cgroup_removed']=not cp.exists()
   result['cgroup_empty']=not cp.exists() or not (cp/'cgroup.procs').read_text().strip()
  result['stdout_bytes']=stdout.stat().st_size;result['stderr_bytes']=stderr.stat().st_size
  if max(result['stdout_bytes'],result['stderr_bytes'])>65536:raise RuntimeError('synthetic stream bound exceeded')
  result['stdout_stderr_verified']=stdout.read_bytes()==b'dlpad-fixture-stdout\n' and stderr.read_bytes()==b'dlpad-fixture-stderr\n'
  result['passed']=all(result['owned_processes_gone'].values()) and all(result['pidfds_ready'].values()) and result['cgroup_empty'] and all(result[k] for k in ('exit_status_verified','literal_argv_passed','worker_stdin_null','escaped_setsid','stdout_stderr_verified'))
  result['state']='completed'
 except Exception as error:result.update(state='failed',error=str(error),passed=False)
 finally:
  left.close();right.close()
  for role,fd in fds.items():
   if base.same_live(records[role]):signal.pidfd_send_signal(fd,signal.SIGKILL);result['actions'].append('cleanup exact owned '+role+' pidfd')
   os.close(fd)
  if owner is not None:owner.wait(timeout=5)
  if invocation:
   terminal=status(unit,env)
   if terminal.get('InvocationID')==invocation and terminal.get('ActiveState')=='failed':
    reset=subprocess.run(['/usr/bin/systemctl','--user','reset-failed',unit],env=env,capture_output=True,timeout=5)
    result['owned_reset_failed_exit']=reset.returncode
  result['post_cleanup_unit']=status(unit,env);result['ended_utc']=base.utc_now();base.write_json(out/(name+'.json'),result)
 return result

def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('binary',type=Path);parser.add_argument('output',type=Path);args=parser.parse_args()
 project=Path(__file__).resolve().parent.parent;evidence=project/'docs/validation/evidence';out=args.output
 if not out.is_absolute() or '..' in out.parts or not out.is_relative_to(evidence) or out==evidence or out.exists():parser.error('new explicit evidence path required')
 for parent in [out,*out.parents]:
  if parent.is_symlink():parser.error('symlink path rejected')
 binary=args.binary
 if binary.is_symlink() or not binary.is_file() or sha(binary)!=EXPECTED_SHA:parser.error('immutable snapshot SHA mismatch')
 runtime=Path('/run/user')/str(os.getuid());env=dict(os.environ)
 for path in (runtime/'bus',runtime/'systemd/private'):
  s=path.lstat()
  if s.st_uid!=os.getuid() or not stat.S_ISSOCK(s.st_mode):parser.error('owned bus socket required')
 env['XDG_RUNTIME_DIR']=str(runtime);env['DBUS_SESSION_BUS_ADDRESS']='unix:path='+str(runtime/'bus');out.mkdir(mode=0o700)
 result={'started_utc':base.utc_now(),'boot_id':Path('/proc/sys/kernel/random/boot_id').read_text().strip(),'binary':str(binary),'binary_sha256_before':sha(binary),'probe_sha256':sha(Path(__file__)),'helper_sha256':sha(Path(base.__file__)),'scope':'immutable actual guardian with synthetic workers; product integration/8h/GUI not claimed','existing_units_targeted':False,'cases':{}}
 with tempfile.TemporaryDirectory(prefix='dlpad-systemd-product-owned-',dir='/tmp') as tmp:
  folder=Path(tmp);sentinel=subprocess.Popen([sys.executable,'-c','import time;time.sleep(120)'],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL);record=base.identity(sentinel.pid)
  try:
   for name in ('eof','K','T','cancel','sigterm','sigkill','owner_guardian_sigkill','exit42'):
    work=folder/name;work.mkdir();result['cases'][name]=case(name,work,out,binary,env)
   result['sentinel_unchanged_alive']=base.same_live(record)
  finally:sentinel.terminate();sentinel.wait(timeout=5);result['sentinel_owned_cleanup_exit']=sentinel.returncode
 result['owned_temp_removed']=not folder.exists();result['binary_sha256_after']=sha(binary)
 result['passed']=all(c['passed'] for c in result['cases'].values()) and result['sentinel_unchanged_alive'] and result['owned_temp_removed'] and result['binary_sha256_after']==EXPECTED_SHA
 result['ended_utc']=base.utc_now();base.write_json(out/'result.json',result)
 print(json.dumps({'passed':result['passed'],'cases':{k:v['passed'] for k,v in result['cases'].items()}}));return 0 if result['passed'] else 2
if __name__=='__main__':sys.exit(main())
