#!/usr/bin/env python3
"""Owned synthetic user-service capability probe. No product changes or other-unit actions."""
import argparse
import ctypes
import datetime
import hashlib
import json
import os
from pathlib import Path
import select
import signal
import socket
import stat
import subprocess
import sys
import tempfile
import time
import uuid

LIMIT = 1024 * 1024
FIXTURE = r'''import json,os,socket,stat,subprocess,sys,time
from pathlib import Path
role,root,case=sys.argv[1:4];root=Path(root)
def identity():
 st=Path('/proc/self/stat').read_text().rsplit(')',1)[1].split()
 return {'pid':os.getpid(),'start_ticks':int(st[19]),'parent_pid':os.getppid()}
if role=='child':
 os.setsid()
 child=subprocess.Popen([sys.executable,__file__,'leaf',str(root),case],stdin=subprocess.DEVNULL,close_fds=True)
 info=identity();info['setsid']=os.getsid(0)==os.getpid();(root/'child.json').write_text(json.dumps(info))
 time.sleep(40)
elif role=='leaf':
 info=identity();(root/'leaf.json').write_text(json.dumps(info));time.sleep(40)
else:
 control=socket.socket(fileno=0);control.settimeout(25)
 fd0_socket=stat.S_ISSOCK(os.fstat(0).st_mode)
 # Keep the receiver private; tool descendants cannot retain the control endpoint.
 os.set_inheritable(0,False)
 child=subprocess.Popen([sys.executable,__file__,'child',str(root),case],stdin=subprocess.DEVNULL,close_fds=True)
 deadline=time.monotonic()+5
 while not all((root/name).exists() for name in ['child.json','leaf.json']):
  if time.monotonic()>deadline:raise RuntimeError('owned descendants not ready')
  time.sleep(.02)
 report={'guardian':identity(),'child':json.loads((root/'child.json').read_text()),'leaf':json.loads((root/'leaf.json').read_text()),'stdin_is_socket':fd0_socket,'argv':sys.argv[4:]}
 control.sendall((json.dumps(report)+'\n').encode())
 received=control.recv(1)
 if received==b'':
  (root/'eof.json').write_text(json.dumps({'control_eof':True,'observed_monotonic':time.monotonic()}))
 elif received!=b'Q':raise RuntimeError('unexpected synthetic control command')
'''


def utc_now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def write_json(path, value):
    data=(json.dumps(value,indent=2,ensure_ascii=True)+"\n").encode()
    if len(data)>LIMIT:raise RuntimeError("owned evidence exceeds bound")
    temporary=path.with_name(path.name+".owned.tmp")
    with temporary.open("xb") as output:
        output.write(data)
    temporary.replace(path)


def identity(pid):
    try:
        fields=Path(f"/proc/{pid}/stat").read_text().rsplit(")",1)[1].split()
        return {"pid":pid,"start_ticks":int(fields[19]),"state":fields[0]}
    except (OSError,ValueError,IndexError):
        return None


def same_identity(record):
    actual=identity(record["pid"])
    return actual is not None and actual["start_ticks"]==record["start_ticks"]


def same_live(record):
    actual=identity(record["pid"])
    return actual is not None and actual["start_ticks"]==record["start_ticks"] and actual["state"]!="Z"


def props(unit, env):
    wanted=["LoadState","ActiveState","SubState","InvocationID","MainPID","ControlGroup","Type","KillMode","SendSIGKILL","TimeoutStopUSec","RemainAfterExit","Restart"]
    args=["/usr/bin/systemctl","--user","show",unit,*["--property="+key for key in wanted]]
    result=subprocess.run(args,env=env,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=5)
    if len(result.stdout)>65536 or len(result.stderr)>65536:raise RuntimeError("property output exceeds bound")
    values=dict(line.split("=",1) for line in result.stdout.decode().splitlines() if "=" in line)
    return {"argv":args,"exit_code":result.returncode,"properties":values,"stderr":result.stderr.decode(errors="replace")}


def verify_unit(unit, invocation, guardian, env):
    observed=props(unit,env);values=observed["properties"]
    if (values.get("InvocationID")!=invocation or values.get("MainPID")!=str(guardian["pid"])
            or not same_live(guardian) or values.get("Type")!="exec" or values.get("KillMode")!="control-group"):
        raise RuntimeError("owned unit/MainPID/start identity changed")
    return observed


def read_ready(control):
    control.settimeout(8)
    data=bytearray()
    while b"\n" not in data:
        chunk=control.recv(min(4096,65536-len(data)))
        if not chunk:raise RuntimeError("guardian readiness EOF")
        data.extend(chunk)
        if len(data)>=65536:raise RuntimeError("guardian readiness exceeds bound")
    return json.loads(data.split(b"\n",1)[0])


def cgroup_of(pid):
    for line in Path(f"/proc/{pid}/cgroup").read_text().splitlines():
        if line.startswith("0::"):return line[3:]
    raise RuntimeError("cgroup v2 membership missing")


class BusError(ctypes.Structure):
    _fields_=[("name",ctypes.c_char_p),("message",ctypes.c_char_p),("need_free",ctypes.c_int)]


def start_execstartex(unit, command, control_fd, log_fd):
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
        for name,value in [("Type","exec"),("KillMode","control-group"),("Restart","no"),("CollectMode","inactive-or-failed")]:scalar_property(name,"s",value)
        for name,value in [("SendSIGKILL",1),("RemainAfterExit",0)]:scalar_property(name,"b",value)
        for name,value in [("TimeoutStopUSec",2000000),("RuntimeMaxUSec",45000000)]:scalar_property(name,"t",value)
        for name,value in [("StandardInputFileDescriptor",control_fd),("StandardOutputFileDescriptor",log_fd),("StandardErrorFileDescriptor",log_fd)]:scalar_property(name,"h",value)
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


def execute_case(name, folder, out, env, transport="cli"):
    unit="dlpad-fixture-"+uuid.uuid4().hex+".service"
    before=props(unit,env)
    if before["properties"].get("LoadState")!="not-found":raise RuntimeError("generated unit collision")
    left,right=socket.socketpair();launcher=None;pidfds={};records={};invocation=None;cgroup=None
    expected=["$DLPAD_FIXTURE_VAR","${DLPAD_FIXTURE_VAR}","prefix-${DLPAD_FIXTURE_VAR}","literal with spaces","한글","%n","quote'\"literal"]
    result={"case":name,"transport":transport,"unit":unit,"expected_argv":expected,"started_utc":utc_now(),"process_actions":[]}
    log=out/(name+".log")
    fixture=folder/"fixture.py";fixture.write_text(FIXTURE)
    argv=["/usr/bin/systemd-run","--user","--no-ask-password","--pipe","--wait","--quiet","--service-type=exec","--collect","--unit="+unit,
          "--property=KillMode=control-group","--property=SendSIGKILL=yes","--property=Restart=no","--property=RemainAfterExit=no",
          "--property=TimeoutStopSec=2s","--property=RuntimeMaxSec=45s","--setenv=DLPAD_FIXTURE_VAR=expanded sentinel",
          str(Path(sys.executable).resolve()),str(fixture),"guardian",str(folder),name,*expected]
    result["argv"]=argv
    try:
        with log.open("xb") as output:
            if transport=="cli":
                launcher=subprocess.Popen(argv,stdin=right,stdout=output,stderr=subprocess.STDOUT,env=env,close_fds=True)
            else:
                command=[str(Path(sys.executable).resolve()),str(fixture),"guardian",str(folder),name,*expected]
                result["argv"]=command
                result["bus_start"]=start_execstartex(unit,command,right.fileno(),output.fileno())
            right.close()
            ready=read_ready(left);result["ready"]=ready
            observed=props(unit,env);values=observed["properties"]
            invocation=values.get("InvocationID")
            if not invocation or len(invocation)!=32 or any(c not in "0123456789abcdef" for c in invocation):raise RuntimeError("InvocationID missing")
            cgroup=values.get("ControlGroup")
            if not cgroup or not cgroup.startswith(f"/user.slice/user-{os.getuid()}.slice/user@{os.getuid()}.service/") or not cgroup.endswith("/"+unit):raise RuntimeError("unexpected owned cgroup")
            records={role:ready[role] for role in ("guardian","child","leaf")}
            result["initial_unit"]=verify_unit(unit,invocation,records["guardian"],env)
            result["invocation_id"]=invocation;result["control_group"]=cgroup
            for role,record in records.items():
                if not same_live(record) or cgroup_of(record["pid"])!=cgroup:raise RuntimeError("owned child identity/cgroup mismatch")
                pidfds[role]=os.pidfd_open(record["pid"])
            result["owned_identities"]=records;result["all_membership_verified"]=True
            result["literal_argv_passed"]=ready["argv"]==expected
            if not ready["stdin_is_socket"] or not ready["child"].get("setsid"):raise RuntimeError("FD0/setsid capability missing")
            started=time.monotonic()
            if name=="eof":
                left.close();result["process_actions"].append("close owned control endpoint")
            else:
                verify_unit(unit,invocation,records["guardian"],env)
                signal.pidfd_send_signal(pidfds["guardian"],signal.SIGKILL)
                result["process_actions"].append("SIGKILL exact guardian pidfd after unit identity verification")
            result["launcher_exit_code"]=launcher.wait(timeout=10) if launcher is not None else None
            deadline=time.monotonic()+10
            while any(same_identity(record) for record in records.values()) and time.monotonic()<deadline:time.sleep(.05)
            result["cleanup_seconds"]=time.monotonic()-started
            result["owned_processes_gone"]={role:not same_identity(record) for role,record in records.items()}
            result["pidfds_ready"]={role:bool(select.select([fd],[],[],0)[0]) for role,fd in pidfds.items()}
            cgroup_path=Path("/sys/fs/cgroup")/cgroup.lstrip("/")
            result["cgroup_removed"]=not cgroup_path.exists()
            if cgroup_path.exists():
                data=(cgroup_path/"cgroup.procs").read_text()[:4096]
                result["remaining_owned_cgroup_pids"]=data.split()
                result["cgroup_empty"]=not data.strip()
            else:result["cgroup_empty"]=True
            result["control_eof_observed"]=(folder/"eof.json").exists() if name=="eof" else None
            result["terminal_unit"]=props(unit,env)
            result["capability_passed"]=(all(result["owned_processes_gone"].values()) and all(result["pidfds_ready"].values())
                                         and result["cgroup_empty"] and (name!="eof" or (result["control_eof_observed"] and (transport!="cli" or result["launcher_exit_code"]==0)))
                                         and (name!="sigkill" or transport!="cli" or result["launcher_exit_code"]!=0))
            result["state"]="completed"
    except Exception as error:
        result.update(state="failed",error=str(error),capability_passed=False)
    finally:
        # Only verified pidfd identities from this fixture may be signalled; no unit-wide guesses.
        for role,fd in pidfds.items():
            if same_live(records[role]):
                signal.pidfd_send_signal(fd,signal.SIGKILL)
                result["process_actions"].append("cleanup exact owned "+role+" pidfd")
            os.close(fd)
        left.close();right.close()
        if launcher is not None and launcher.poll() is None:
            try:launcher.wait(timeout=50)
            except subprocess.TimeoutExpired:
                launcher.terminate();launcher.wait(timeout=5)
                result["process_actions"].append("terminate owned launcher child")
        if log.exists() and log.stat().st_size>LIMIT:raise RuntimeError("owned log bound exceeded")
        result["ended_utc"]=utc_now();write_json(out/(name+".json"),result)
    return result


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument("output",type=Path);parser.add_argument("--transport",choices=("cli","execstartex"),default="cli");args=parser.parse_args()
    project=Path(__file__).resolve().parent.parent;out=args.output
    evidence=project/"docs/validation/evidence"
    if not out.is_absolute() or not out.is_relative_to(evidence) or out==evidence or out.exists() or out.is_symlink():parser.error("new explicit project evidence directory required")
    out.mkdir(mode=0o700)
    env=dict(os.environ);runtime=Path('/run/user')/str(os.getuid())
    for socket_path in [runtime/'bus',runtime/'systemd/private']:
        info=socket_path.lstat()
        if not stat.S_ISSOCK(info.st_mode) or info.st_uid!=os.getuid():raise RuntimeError("owned user manager socket missing")
    env['XDG_RUNTIME_DIR']=str(runtime);env['DBUS_SESSION_BUS_ADDRESS']='unix:path='+str(runtime/'bus')
    result={"started_utc":utc_now(),"boot_id":Path('/proc/sys/kernel/random/boot_id').read_text().strip(),"scope":"synthetic transient user guardian; product/actual media/8h/GUI not verified","transport":args.transport,"cases":{},"global_settings_changed":False,"existing_units_changed":False}
    with tempfile.TemporaryDirectory(prefix='dlpad-systemd-owned-',dir='/tmp') as temporary:
        folder=Path(temporary)
        sentinel=subprocess.Popen([sys.executable,'-c','import time;time.sleep(60)'],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,close_fds=True)
        sentinel_record=identity(sentinel.pid)
        try:
            for name in ('eof','sigkill'):
                case_folder=folder/name;case_folder.mkdir()
                result['cases'][name]=execute_case(name,case_folder,out,env,args.transport)
            result['sentinel_unchanged_alive']=same_live(sentinel_record)
        finally:
            sentinel.terminate();sentinel.wait(timeout=5)
            result['sentinel_owned_cleanup_exit']=sentinel.returncode
    result['temporary_owned_folder_removed']=not folder.exists()
    result['capability_passed']=all(value['capability_passed'] for value in result['cases'].values()) and result['sentinel_unchanged_alive']
    result['literal_argv_passed']=all(value.get('literal_argv_passed',False) for value in result['cases'].values())
    result['overall_passed']=result['capability_passed'] and result['literal_argv_passed']
    result['ended_utc']=utc_now();result['script_sha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    write_json(out/'result.json',result)
    print(json.dumps({key:result[key] for key in ['capability_passed','literal_argv_passed','overall_passed','temporary_owned_folder_removed']},indent=2))
    return 0 if result['overall_passed'] else 2


if __name__=='__main__':raise SystemExit(main())
