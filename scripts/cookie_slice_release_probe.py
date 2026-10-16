#!/usr/bin/env python3
"""Owned user-manager runtime StopWhenUnneeded release experiment; never StopUnit."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import select
import stat
import subprocess
import time
import uuid
import cookie_cgroup_lifetime_probe as lifetime
import cookie_dedicated_slice_probe as dedicated
import systemd_user_guardian_probe as base

PROJECT = Path(__file__).resolve().parent.parent

def properties(unit, env):
    keys = ['LoadState', 'ActiveState', 'SubState', 'InvocationID', 'MainPID', 'ExecMainPID', 'ExecMainCode', 'ExecMainStatus', 'ControlGroup', 'Slice', 'Delegate', 'StopWhenUnneeded', 'Result', 'Job']
    child = subprocess.run(['/usr/bin/systemctl', '--user', '--no-ask-password', 'show', unit, *['--property='+key for key in keys]], env=env, stdin=subprocess.DEVNULL, capture_output=True, timeout=3)
    if len(child.stdout)>65536 or len(child.stderr)>65536:raise RuntimeError('property bound exceeded')
    if child.returncode:raise RuntimeError('owned property query failed')
    return dict(line.split('=',1) for line in child.stdout.decode().splitlines() if '=' in line)

def start_service(bus, unit, parent, kind):
    argv=['/usr/bin/python3','-c','import time;time.sleep(2)']
    with open('/dev/null','rb') as null:
        def build(basic,opened,closed):
            basic('s',unit);basic('s','fail');opened('a','(sv)')
            def prop(name,signature,value):
                opened('r','sv');basic('s',name);opened('v',signature);basic(signature,value);closed();closed()
            for name,value in [('Type',kind),('Slice',parent),('KillMode','control-group'),('Restart','no'),('CollectMode','inactive-or-failed')]:prop(name,'s',value)
            for name,value in [('AddRef',1),('SendSIGKILL',1),('RemainAfterExit',0),('Delegate',0)]:prop(name,'b',value)
            for name,value in [('TimeoutStartUSec',3000000),('RuntimeMaxUSec',3000000),('TimeoutStopUSec',2000000)]:prop(name,'t',value)
            for name in ['StandardInputFileDescriptor','StandardOutputFileDescriptor','StandardErrorFileDescriptor']:prop(name,'h',null.fileno())
            opened('r','sv');basic('s','ExecStartEx');opened('v','a(sasas)');opened('a','(sasas)');opened('r','sasas');basic('s',argv[0]);opened('a','s')
            for arg in argv:basic('s',arg)
            closed();opened('a','s');basic('s','no-env-expand');closed();closed();closed();closed();closed();closed();opened('a','(sa(sv))');closed()
        return bus.call('/org/freedesktop/systemd1','org.freedesktop.systemd1.Manager','StartTransientUnit',build,True)

def set_unneeded(bus,path):
    def build(basic,opened,closed):
        basic('b',1);opened('a','(sv)');opened('r','sv');basic('s','StopWhenUnneeded');opened('v','b');basic('b',1);closed();closed();closed()
    bus.call(path,'org.freedesktop.systemd1.Unit','SetProperties',build)

def wait(unit, env, predicate, invocation=None):
    deadline=time.monotonic()+5
    while True:
        value=properties(unit,env)
        if invocation is not None and value.get('InvocationID')!=invocation:raise RuntimeError('owned invocation changed')
        if predicate(value):return value
        if time.monotonic()>=deadline:raise RuntimeError('owned observation expired')
        time.sleep(.02)

def own_pid(value,parent_group,fdlist):
    pid=base.identity(int(value['MainPID']))
    if pid is None or not base.same_live(pid) or value.get('Delegate')!='no' or base.cgroup_of(pid['pid'])!=value['ControlGroup'] or not value['ControlGroup'].startswith(parent_group+'/'):raise RuntimeError('owned service PID/membership rejected')
    fd=os.pidfd_open(pid['pid']);fdlist.append(fd);return pid,fd

def normal_terminal(unit,env,invocation):
    value=wait(unit,env,lambda p:p.get('ActiveState') in ['inactive','failed'],invocation)
    if value.get('Result')!='success' or value.get('ExecMainCode')!='1' or value.get('ExecMainStatus')!='0':raise RuntimeError('synthetic worker did not exit normally')
    return value

def execute(case,env):
    token=uuid.uuid4().hex;parent='dlpadcookie'+token+'.slice';proof='dlpadreleaseproof'+token+'.service';sibling='dlpadreleasesibling'+token+'.service'
    r={'case':case,'slice':parent,'proof_service':proof,'sibling_service':sibling if case!='empty' else None,'started_utc':base.utc_now(),'parent_stop_called':False,'signals_sent':False}
    baseline=len(list(Path('/proc/self/fd').iterdir()));bus=None;fds=[]
    try:
        bus=lifetime.Bus();r['slice_job']=dedicated.start_slice(bus,parent);parent_path=bus.get(parent)
        r['proof_job']=start_service(bus,proof,parent,'exec');proof_path=bus.get(proof)
        active=wait(proof,env,lambda p:p.get('ActiveState')=='active' and int(p.get('MainPID','0'))>1)
        pactive=wait(parent,env,lambda p:p.get('ActiveState')=='active' and bool(p.get('ControlGroup')))
        pin=dedicated.pin_group(pactive['ControlGroup'],fds);native=lifetime.fd_identity(pin[1]);proof_pid,proof_fd=own_pid(active,pactive['ControlGroup'],fds)
        parent_inv=pactive['InvocationID'];proof_inv=active['InvocationID']
        if active.get('Slice')!=parent:raise RuntimeError('proof service Slice mismatch')
        r.update(parent_initial=pactive,proof_initial=active,parent_native_identity=native,proof_pid=proof_pid,initial_parent=dedicated.snapshot(pin))
        r['proof_terminal']=normal_terminal(proof,env,proof_inv)
        r['proof_pid_gone']=not base.same_identity(proof_pid);r['proof_pidfd_ready']=bool(select.select([proof_fd],[],[],0)[0])
        if not r['proof_pid_gone'] or not r['proof_pidfd_ready']:raise RuntimeError('proof reader not terminal')
        current=properties(parent,env)
        if current.get('InvocationID')!=parent_inv or current.get('ActiveState')!='active' or current.get('ControlGroup')!=pactive['ControlGroup']:raise RuntimeError('empty parent identity changed')
        r['strict_empty_before_release']=dedicated.snapshot(pin)
        if not r['strict_empty_before_release']['held_empty'] or not r['strict_empty_before_release']['new_openat_empty']:raise RuntimeError('strict empty proof absent')
        bus.unref(proof_path);r['proof_final']=dedicated.wait_gone(bus,proof)
        if case!='empty':
            r['sibling_job']=start_service(bus,sibling,parent,'oneshot' if case=='pending' else 'exec');sibling_path=bus.get(sibling)
            expected='activating' if case=='pending' else 'active'
            sactive=wait(sibling,env,lambda p:p.get('ActiveState')==expected and int(p.get('MainPID','0'))>1)
            sibling_inv=sactive['InvocationID'];sibling_pid,sibling_fd=own_pid(sactive,pactive['ControlGroup'],fds)
            if sactive.get('Slice')!=parent:raise RuntimeError('sibling Slice mismatch')
            if case=='pending' and (not sactive.get('Job') or sactive['Job'].split()[0]=='0'):raise RuntimeError('pending start job not observed')
            r.update(sibling_before_release=sactive,sibling_pid=sibling_pid,parent_with_sibling=dedicated.snapshot(pin))
            if r['parent_with_sibling']['held_empty']:raise RuntimeError('populated descendant mistaken for empty')
        # Exact same invocation only; setter and release never send a parent Stop request.
        before=properties(parent,env)
        if before.get('InvocationID')!=parent_inv or before.get('ControlGroup')!=pactive['ControlGroup'] or lifetime.fd_identity(pin[1])!=native:raise RuntimeError('release ownership changed')
        set_unneeded(bus,parent_path);r['runtime_stop_when_unneeded_set']=True
        verified=properties(parent,env)
        if verified.get('InvocationID')!=parent_inv or verified.get('StopWhenUnneeded')!='yes':raise RuntimeError('runtime setter not confirmed')
        bus.unref(parent_path);r['parent_unref']=True
        if case!='empty':
            after=properties(sibling,env);parent_after=properties(parent,env)
            r.update(sibling_after_release=after,parent_after_release=parent_after,parent_after_release_native=dedicated.snapshot(pin))
            if after.get('InvocationID')!=sibling_inv or after.get('ActiveState')!=expected or not base.same_live(sibling_pid) or select.select([sibling_fd],[],[],0)[0]:raise RuntimeError('sibling was interrupted by release')
            if case=='pending' and after.get('Job')!=sactive.get('Job'):raise RuntimeError('pending job changed during release')
            if parent_after.get('InvocationID')!=parent_inv or parent_after.get('ActiveState')!='active' or lifetime.fd_identity(pin[1])!=native:raise RuntimeError('parent did not preserve active dependency')
            r['sibling_terminal']=normal_terminal(sibling,env,sibling_inv)
            r['sibling_pid_gone']=not base.same_identity(sibling_pid);r['sibling_pidfd_ready']=bool(select.select([sibling_fd],[],[],0)[0])
            if not r['sibling_pid_gone'] or not r['sibling_pidfd_ready']:raise RuntimeError('sibling terminal not confirmed')
            if properties(sibling,env).get('InvocationID')!=sibling_inv:raise RuntimeError('sibling release identity changed')
            bus.unref(sibling_path);r['sibling_final']=dedicated.wait_gone(bus,sibling)
        r['slice_final']=dedicated.wait_gone(bus,parent)
        r['parent_cgroup_gone']=not pin[0].exists();r['proof_cgroup_gone']=not Path('/sys/fs/cgroup'+active['ControlGroup']).exists()
        if case!='empty':r['sibling_cgroup_gone']=not Path('/sys/fs/cgroup'+sactive['ControlGroup']).exists()
        r['completed']=True
    except Exception as error:r.update(completed=False,error_type=type(error).__name__,error=str(error))
    finally:
        for fd in reversed(fds):os.close(fd)
        if bus is not None:bus.close()
        r['fd_before']=baseline;r['fd_after']=len(list(Path('/proc/self/fd').iterdir()));r['ended_utc']=base.utc_now()
        r['passed']=r.get('completed') is True and r.get('parent_cgroup_gone') is True and r.get('proof_cgroup_gone') is True and r['fd_before']==r['fd_after']
    return r

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('output',type=Path);a=parser.parse_args();out=a.output
    if not out.is_absolute() or '..' in out.parts or not out.is_relative_to(PROJECT/'docs/validation/evidence') or out.exists() or any(p.is_symlink() for p in [out,*out.parents]):parser.error('fresh project evidence required')
    runtime=Path('/run/user')/str(os.getuid())
    for p in [runtime/'bus',runtime/'systemd/private']:
        info=p.lstat()
        if not stat.S_ISSOCK(info.st_mode) or info.st_uid!=os.getuid():parser.error('owned user manager required')
    env=dict(os.environ);env.update(XDG_RUNTIME_DIR=str(runtime),DBUS_SESSION_BUS_ADDRESS='unix:path='+str(runtime/'bus'),LC_ALL='C')
    out.mkdir(mode=0o700);results=[]
    for case in ['empty','sibling','pending']:
        r=execute(case,env);data=(json.dumps(r,indent=2)+'\n').encode()
        if len(data)>262144:raise RuntimeError('case evidence bound')
        with (out/(case+'.json')).open('xb') as f:f.write(data)
        results.append(r)
        if not r['passed']:break
    r={'source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'reused_sources':{Path(module.__file__).name:hashlib.sha256(Path(module.__file__).read_bytes()).hexdigest() for module in [lifetime,dedicated,base]},'cases':{item['case']:item['passed'] for item in results},'passed':len(results)==3 and all(item['passed'] for item in results)}
    with (out/'result.json').open('x') as f:json.dump(r,f,indent=2);f.write('\n')
    print(json.dumps(r['cases']));return 0 if r['passed'] else 2

if __name__=='__main__':raise SystemExit(main())
