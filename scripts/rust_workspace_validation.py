#!/usr/bin/env python3
"""고정 Rust workspace 검증과 소유 실행 원시 수집. 임의 명령은 받지 않는다."""
import argparse
import datetime
import hashlib
import fcntl
import json
import os
from pathlib import Path
import selectors
import shutil
import signal
import stat
import subprocess
import sys
import time
import uuid
ROOT=Path(__file__).resolve().parents[1]
MAX_LOG=8*1024*1024
CANCEL_REQUESTED=False
def request_cancel(_signal,_frame):
    global CANCEL_REQUESTED
    CANCEL_REQUESTED=True
COMMANDS={
 'cargo-version':['cargo','--version','--verbose'],
 'rustc-version':['rustc','--version','--verbose'],
 'fmt':['cargo','fmt','--all','--','--check'],
 'clippy':['cargo','clippy','--workspace','--all-targets','--locked','--','-D','warnings'],
 'test':['cargo','test','--workspace','--locked'],
 'release':['cargo','build','--workspace','--release','--locked'],
 'cpu-clippy':['cargo','clippy','--workspace','--all-targets','--locked','--features','cpu','--','-D','warnings'],
 'cpu-test':['cargo','test','--workspace','--locked','--features','cpu'],
 'cpu-release':['cargo','build','--workspace','--release','--locked','--features','cpu'],
}

def utc():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def boot():
    raw=Path('/proc/sys/kernel/random/boot_id').read_text().strip()
    if str(uuid.UUID(raw))!=raw:raise ValueError('invalid kernel boot UUID')
    return raw

def identity(pid):
    raw=Path(f'/proc/{pid}/stat').read_text();parts=raw.rsplit(')',1)[1].split()
    return {'pid':pid,'start_ticks':int(parts[19]),'process_group':int(parts[2]),'session':int(parts[3])}

def sha(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda:f.read(65536),b''):h.update(block)
    return h.hexdigest()

def directory_fd(path):
    path=Path(path)
    if not path.is_absolute() or '..' in path.parts:raise ValueError('absolute plain directory required')
    fd=os.open('/',os.O_RDONLY|os.O_DIRECTORY|os.O_CLOEXEC)
    try:
        for component in path.parts[1:]:
            next_fd=os.open(component,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=fd)
            os.close(fd);fd=next_fd
        return fd
    except BaseException:
        os.close(fd);raise

def project_directory_fd(path):
    path=Path(path)
    path.relative_to(ROOT)
    return directory_fd(path)

def source_sha(path):
    parent=project_directory_fd(path.parent)
    try:fd=os.open(path.name,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC,dir_fd=parent)
    finally:os.close(parent)
    with os.fdopen(fd,'rb') as f:
        before=os.fstat(f.fileno())
        if not stat.S_ISREG(before.st_mode):raise ValueError('nonregular source input')
        h=hashlib.sha256()
        for block in iter(lambda:f.read(65536),b''):h.update(block)
        after=os.fstat(f.fileno())
        keys=lambda v:(v.st_dev,v.st_ino,v.st_size,v.st_mtime_ns,v.st_ctime_ns)
        if keys(before)!=keys(after):raise ValueError('source input changed while hashing')
    return h.hexdigest()

def snapshot():
    files={ROOT/n for n in ['Cargo.toml','Cargo.lock','rust-toolchain.toml','scripts/rust_workspace_validation.py']}
    files.update((ROOT/'crates').rglob('*.rs'));files.update((ROOT/'crates').rglob('Cargo.toml'))
    files.update(p for p in (ROOT/'crates').rglob('*') if p.is_symlink())
    files.update(p for p in (ROOT/'vendor').rglob('*') if p.is_file() or p.is_symlink())
    return {str(p.relative_to(ROOT)):source_sha(p) for p in sorted(files)}

def mkdir_evidence(out):
    output_path(str(out))
    parent=project_directory_fd(out.parent)
    try:
        os.mkdir(out.name,0o700,dir_fd=parent)
        # Persist the new directory entry before publishing any runner manifest.
        os.fsync(parent)
    finally:os.close(parent)

def durable(path,value):
    data=(json.dumps(value,indent=2)+'\n').encode()
    parent=project_directory_fd(path.parent)
    try:
        fd=os.open(path.name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW|os.O_CLOEXEC,0o600,dir_fd=parent)
        with os.fdopen(fd,'wb') as f:f.write(data);f.flush();os.fsync(f.fileno())
        os.fsync(parent)
    finally:os.close(parent)

def command(name):
    argv=COMMANDS[name][:];tool=shutil.which(argv[0])
    if tool is None:raise ValueError('required PATH tool unavailable')
    argv[0]=str(Path(tool).absolute())
    return argv

def worker(name,gate):
    if gate<3:raise ValueError('worker gate invalid')
    info=os.fstat(gate)
    if not stat.S_ISFIFO(info.st_mode) or info.st_uid!=os.geteuid():raise ValueError('worker gate ownership invalid')
    flags=os.O_ACCMODE & fcntl.fcntl(gate,fcntl.F_GETFL)
    if flags!=os.O_RDONLY:raise ValueError('worker gate access invalid')
    with os.fdopen(gate,'rb',buffering=0) as f:
        if f.read(1)!=b'G' or f.read(1)!=b'':return 125
    argv=command(name)
    os.chdir(ROOT)
    os.execv(argv[0],argv)
    return 125

def group_members(group):
    members=[];count=0
    for p in Path('/proc').iterdir():
        if not p.name.isdecimal():continue
        count+=1
        if count>4096:return None
        try:
            raw=(p/'stat').read_text();parts=raw.rsplit(')',1)[1].split()
            if int(parts[2])==group and parts[0]!='Z':members.append(int(p.name))
        except (OSError,ValueError,IndexError):continue
    return members

def cancel(child,owned,expected_boot):
    record={'requested':True,'identity_verified':False,'signals':[]}
    try:
        current=identity(child.pid)
        if boot()!=expected_boot or current!=owned or owned['process_group']!=child.pid or owned['session']!=child.pid:
            record['error']='ownership changed';return record
        record['identity_verified']=True
        # Keep the leader unreaped while signaling: its PID cannot be reused.
        os.killpg(child.pid,signal.SIGTERM);record['signals'].append('TERM')
        deadline=time.monotonic()+2
        while time.monotonic()<deadline:
            members=group_members(child.pid)
            if members==[]:break
            time.sleep(.02)
        if group_members(child.pid)!=[]:
            if boot()!=expected_boot or identity(child.pid)!=owned:raise RuntimeError('ownership changed before KILL')
            os.killpg(child.pid,signal.SIGKILL);record['signals'].append('KILL')
        record['exit_code']=child.wait(timeout=8)
        record['remaining_members']=group_members(owned['process_group'])
        record['terminal_confirmed']=record['identity_verified'] and record['remaining_members']==[]
    except (OSError,ValueError,RuntimeError,subprocess.TimeoutExpired) as e:record['error']=type(e).__name__
    finally:
        # A Popen-owned Child can always be reaped without signaling a reused PID.
        if 'exit_code' not in record:
            try:record['exit_code']=child.wait(timeout=8)
            except subprocess.TimeoutExpired:record['reap_unconfirmed']=True
        if 'exit_code' in record:
            record['remaining_members']=group_members(owned['process_group'])
            record['terminal_confirmed']=record['identity_verified'] and record['remaining_members']==[]
    return record

def run(name,out,seconds,runner,expected_boot):
    before=snapshot();argv=command(name);r={'operation':name,'role':'workspace-validation-command','argv':argv,'source_before':before,'boot_id':expected_boot,'started_utc':utc(),'timeout_seconds':seconds,'max_log_bytes':MAX_LOG}
    read,write=os.pipe2(os.O_CLOEXEC);child=None;owned=None;selector=selectors.DefaultSelector();total=0;failure=None;cleanup=None;code=None;started=time.monotonic()
    try:
        child=subprocess.Popen([sys.executable,str(Path(__file__).resolve()),'--worker',name,'--gate-fd',str(read)],cwd=ROOT,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,pass_fds=(read,),start_new_session=True)
        os.close(read);read=None;owned=identity(child.pid)
        if owned['process_group']!=child.pid or owned['session']!=child.pid:raise RuntimeError('command group ownership invalid')
        launch={'state':'waiting-for-durable-exec-gate','boot_id':expected_boot,'archive_owner':runner,'runner':runner,'command':dict(owned,role='workspace-validation-command'),'operation':name,'argv':argv,'expected_executable_sha256':sha(Path(argv[0])),'recorded_utc':utc()}
        durable(out/(name+'-launch.json'),launch)
        r['command']=launch['command'];r['launch_manifest']=name+'-launch.json'
        selector.register(child.stdout,selectors.EVENT_READ)
        with (out/(name+'.log')).open('xb') as log:
            try:
                if CANCEL_REQUESTED:raise InterruptedError('runner cancellation requested')
                # Gate opens only after command identity+boot+argv are durable.
                os.write(write,b'G');os.close(write);write=None
                while selector.get_map():
                    if CANCEL_REQUESTED:raise InterruptedError('runner cancellation requested')
                    if time.monotonic()-started>=seconds:raise TimeoutError('command absolute deadline')
                    for key,_ in selector.select(min(.25,max(0,seconds-(time.monotonic()-started)))):
                        data=os.read(key.fd,65536)
                        if not data:selector.unregister(key.fileobj);continue
                        if total+len(data)>MAX_LOG:
                            available=MAX_LOG-total
                            if available:log.write(data[:available]);total+=available
                            raise RuntimeError('command output bound exceeded')
                        log.write(data);total+=len(data)
            finally:
                log.flush();os.fsync(log.fileno())
        members=group_members(child.pid)
        if members is None or any(pid!=child.pid for pid in members):raise RuntimeError('command descendants unconfirmed')
        code=child.wait(timeout=max(.01,seconds-(time.monotonic()-started)))
    except BaseException as e:
        failure=type(e).__name__
        # If launch identity recording fails, EOF stops the gated worker before exec.
        if write is not None:os.close(write);write=None
        if child is not None and owned is not None:cleanup=cancel(child,owned,expected_boot);code=cleanup.get('exit_code')
        elif child is not None:
            cleanup={'requested':True,'identity_verified':False,'signals':[],'terminal_confirmed':False}
            try:code=child.wait(timeout=8);cleanup['exit_code']=code
            except subprocess.TimeoutExpired:cleanup['reap_unconfirmed']=True

    finally:
        if read is not None:os.close(read)
        if write is not None:os.close(write)
        selector.close()
        if child is not None and child.stdout is not None:child.stdout.close()
    after=snapshot();r.update(source_after=after,source_unchanged=before==after,exit_code=code,collector_failure=failure,cleanup=cleanup,log_bytes=total,elapsed_seconds=time.monotonic()-started,ended_utc=utc())
    r['passed']=code==0 and failure is None and before==after
    durable(out/(name+'-result.json'),r)
    return r

def output_path(value):
    out=Path(value)
    if not out.is_absolute() or '..' in out.parts or not out.is_relative_to(ROOT/'docs/validation/evidence') or out.exists():raise ValueError('fresh absolute project evidence required')
    if any(p.is_symlink() for p in [out,*out.parents]) or out.parent.resolve()!=out.parent:raise ValueError('canonical nonsymlink evidence required')
    return out

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',nargs='?',type=output_path);p.add_argument('--execute',action='store_true');p.add_argument('--seconds',type=int,default=1800);p.add_argument('--worker',choices=tuple(COMMANDS),help=argparse.SUPPRESS);p.add_argument('--gate-fd',type=int,help=argparse.SUPPRESS);a=p.parse_args()
    if a.worker:
        if a.gate_fd is None or a.output is not None or a.execute:p.error('fixed worker framing invalid')
        return worker(a.worker,a.gate_fd)
    if a.output is None or a.gate_fd is not None or not 1<=a.seconds<=1800:p.error('output and 1..1800 seconds required')
    baseline=snapshot();expected_boot=boot();runner=dict(identity(os.getpid()),role='workspace-validation-runner');out=a.output;mkdir_evidence(out)
    guide={'PATH':'root supplies project/user-only installed tool PATH; no hardcoded user home','CARGO_TARGET_DIR':'root may provide dedicated target directory','RUSTFLAGS':'root supplied only; record value when explicit','no_native_claim':'workspace test excludes explicit ignored native acceptance; do not equate to P0 complete'}
    fixed_env={key:os.environ[key] for key in ['CARGO_TARGET_DIR','RUSTFLAGS','CARGO_BUILD_JOBS','CARGO_NET_OFFLINE'] if key in os.environ}
    manifest={'state':'prepared','boot_id':expected_boot,'archive_owner':runner,'runner':runner,'recorded_utc':utc(),'source_baseline':baseline,'fixed_environment':fixed_env,'environment_guide':guide,'plans':{n:COMMANDS[n] for n in COMMANDS},'execute_requested':a.execute}
    durable(out/'launch.json',manifest);results={}
    if a.execute:
        signal.signal(signal.SIGTERM,request_cancel);signal.signal(signal.SIGINT,request_cancel)
        for name in COMMANDS:
            if CANCEL_REQUESTED:break
            print('workspace operation='+name,flush=True)
            r=run(name,out,10 if name.endswith('-version') else a.seconds,runner,expected_boot);results[name]=r
            if not r['passed']:break
    final=snapshot();result={'state':'completed' if a.execute and len(results)==len(COMMANDS) and all(r['passed'] for r in results.values()) and baseline==final else 'prepared' if not a.execute else 'failed','boot_id':expected_boot,'runner':runner,'operations':{n:{k:r[k] for k in ['passed','exit_code','elapsed_seconds','log_bytes','source_unchanged','collector_failure']} for n,r in results.items()},'source_final':final,'source_unchanged':baseline==final,'workspace_checks_complete':a.execute and len(results)==len(COMMANDS) and all(r['passed'] for r in results.values()) and baseline==final,'ignored_native_acceptance_complete':False}
    durable(out/'result.json',result);print(json.dumps({'state':result['state'],'workspace_checks_complete':result['workspace_checks_complete']}));return 0 if result['state'] in ['completed','prepared'] else 2
if __name__=='__main__':raise SystemExit(main())
