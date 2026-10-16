#!/usr/bin/env python3
"""Owned auxiliary slice proof lifetime after real failure/signal exit."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import select
import stat
import time
import uuid
import cookie_slice_dependency_probe as dependency
import cookie_slice_release_probe as release
import cookie_dedicated_slice_probe as dedicated
import cookie_cgroup_lifetime_probe as lifetime
import systemd_user_guardian_probe as base
PROJECT = Path(__file__).resolve().parent.parent

def start_pair(bus, unit, parent, case):
    argv = ['/usr/bin/python3', '-c', 'import time,os,signal;time.sleep(2);' + ('os._exit(7)' if case == 'exit7' else 'os.kill(os.getpid(),signal.SIGTERM)')]
    with open('/dev/null', 'rb') as null:
        def build(basic, opened, closed):
            def prop(name, signature, value):
                opened('r','sv'); basic('s',name); opened('v',signature); basic(signature,value); closed(); closed()
            basic('s',unit); basic('s','fail'); opened('a','(sv)')
            for name,value in [('Type','exec'),('Slice',parent),('KillMode','control-group'),('Restart','no'),('CollectMode','inactive-or-failed')]: prop(name,'s',value)
            for name,value in [('AddRef',1),('SendSIGKILL',1),('RemainAfterExit',1),('Delegate',0)]: prop(name,'b',value)
            for name,value in [('TimeoutStartUSec',3000000),('RuntimeMaxUSec',3000000),('TimeoutStopUSec',2000000)]: prop(name,'t',value)
            for name in ['StandardInputFileDescriptor','StandardOutputFileDescriptor','StandardErrorFileDescriptor']: prop(name,'h',null.fileno())
            opened('r','sv'); basic('s','ExecStartEx'); opened('v','a(sasas)'); opened('a','(sasas)'); opened('r','sasas'); basic('s',argv[0]); opened('a','s')
            for arg in argv: basic('s',arg)
            closed(); opened('a','s'); basic('s','no-env-expand'); basic('s','ignore-failure'); closed(); closed(); closed(); closed(); closed(); closed()
            opened('a','(sa(sv))'); opened('r','sa(sv)'); basic('s',parent); opened('a','(sv)')
            prop('AddRef','b',1); prop('StopWhenUnneeded','b',1); prop('CollectMode','s','inactive-or-failed')
            closed(); closed(); closed()
        return bus.call('/org/freedesktop/systemd1','org.freedesktop.systemd1.Manager','StartTransientUnit',build,True)

def execute(case, env):
    token=uuid.uuid4().hex; parent='dlpadcookie'+token+'.slice'; unit='dlpad-failure-'+token+'.service'
    r={'case':case,'parent':parent,'service':unit,'normal_parent_stop':False,'worker_seconds':2,'runtime_max_seconds':3,'flags':['no-env-expand','ignore-failure'],'started_utc':base.utc_now()}
    baseline=len(list(Path('/proc/self/fd').iterdir())); bus=None; fds=[]; pin=None; inv=None; pinv=None; path=None; ppath=None
    try:
        bus=lifetime.Bus();r['start_job']=start_pair(bus,unit,parent,case);path=bus.get(unit);ppath=bus.get(parent)
        active=release.wait(unit,env,lambda p:p.get('ActiveState')=='active' and int(p.get('MainPID','0'))>1)
        pa=release.wait(parent,env,lambda p:p.get('ActiveState')=='active' and bool(p.get('ControlGroup')))
        inv=active['InvocationID'];pinv=pa['InvocationID']
        if pa.get('StopWhenUnneeded')!='yes' or active.get('Slice')!=parent:raise RuntimeError('pair dependency missing')
        pid,pidfd=release.own_pid(active,pa['ControlGroup'],fds);pin=dedicated.pin_group(pa['ControlGroup'],fds);native=lifetime.fd_identity(pin[1])
        r.update(initial_service=active,initial_parent=pa,worker_pid=pid,native_identity=native,initial_native=dedicated.snapshot(pin))
        terminal=release.wait(unit,env,lambda p:p.get('ActiveState')=='active' and p.get('SubState')=='exited' and p.get('MainPID')=='0',inv)
        expected=('1','7') if case=='exit7' else ('2','15')
        if (terminal.get('ExecMainCode'),terminal.get('ExecMainStatus'))!=expected:raise RuntimeError('actual exit code/signal changed')
        after=release.properties(parent,env);empty=dedicated.snapshot(pin)
        r.update(actual_terminal=terminal,parent_after_terminal=after,strict_empty=empty,worker_gone=not base.same_identity(pid),pidfd_ready=bool(select.select([pidfd],[],[],0)[0]))
        if after.get('InvocationID')!=pinv or after.get('ActiveState')!='active' or after.get('ControlGroup')!=pa['ControlGroup'] or lifetime.fd_identity(pin[1])!=native:raise RuntimeError('parent lifetime changed')
        if not r['worker_gone'] or not r['pidfd_ready'] or not empty['held_empty'] or not empty['new_openat_empty']:raise RuntimeError('strict terminal proof missing')
        if release.properties(unit,env).get('InvocationID')!=inv:raise RuntimeError('reader stop identity changed')
        r['reader_stop_job']=dependency.stop_exact(bus,unit,path,inv);release.wait(unit,env,lambda p:p.get('ActiveState') in ['inactive','failed'],inv)
        bus.unref(path);r['reader_gc']=dedicated.wait_gone(bus,unit)
        r['parent_terminal']=release.wait(parent,env,lambda p:p.get('ActiveState') in ['inactive','failed'],pinv)
        bus.unref(ppath);r['parent_gc']=dedicated.wait_gone(bus,parent);r['parent_cgroup_gone']=not pin[0].exists();r['completed']=r['parent_cgroup_gone']
    except Exception as error:r.update(completed=False,error_type=type(error).__name__,error=str(error))
    finally:
        r['failure_cleanup']=[]
        if not r.get('completed') and bus is not None:
            time.sleep(3.1)
            try:
                if path and inv:
                    p=release.properties(unit,env)
                    if p.get('InvocationID')==inv and p.get('MainPID')=='0':
                        dependency.stop_exact(bus,unit,path,inv);bus.unref(path);r['failure_cleanup'].append({'service_stop':True})
                if ppath and pinv and pin:
                    p=release.properties(parent,env);empty=dedicated.snapshot(pin)
                    if p.get('InvocationID')==pinv and empty['held_empty'] and empty['new_openat_empty']:
                        dependency.stop_exact(bus,parent,ppath,pinv);bus.unref(ppath);r['failure_cleanup'].append({'failure_only_parent_stop':True,'strict_empty':empty,'gc':dedicated.wait_gone(bus,parent)})
            except Exception as error:r['failure_cleanup'].append({'error':str(error)})
        for fd in reversed(fds):os.close(fd)
        if bus:bus.close()
        r.update(fd_before=baseline,fd_after=len(list(Path('/proc/self/fd').iterdir())),ended_utc=base.utc_now())
        r['passed']=r.get('completed') is True and r['fd_before']==r['fd_after']
    return r

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('output',type=Path);a=parser.parse_args();out=a.output
    if not out.is_absolute() or '..' in out.parts or not out.is_relative_to(PROJECT/'docs/validation/evidence') or out.exists() or any(p.is_symlink() for p in [out,*out.parents]):parser.error('fresh project evidence required')
    runtime=Path('/run/user')/str(os.getuid())
    for p in [runtime/'bus',runtime/'systemd/private']:
        info=p.lstat()
        if not stat.S_ISSOCK(info.st_mode) or info.st_uid!=os.getuid():parser.error('owned manager required')
    env=dict(os.environ);env.update(XDG_RUNTIME_DIR=str(runtime),DBUS_SESSION_BUS_ADDRESS='unix:path='+str(runtime/'bus'),LC_ALL='C')
    out.mkdir(mode=0o700);results=[]
    for case in ['exit7','selfsigterm']:
        r=execute(case,env);data=(json.dumps(r,indent=2)+'\n').encode()
        if len(data)>262144:raise RuntimeError('evidence bound')
        with (out/(case+'.json')).open('xb') as f:f.write(data)
        results.append(r)
        if not r['passed']:break
    result={'source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'reused_sources':{Path(m.__file__).name:hashlib.sha256(Path(m.__file__).read_bytes()).hexdigest() for m in [dependency,release,dedicated,lifetime,base]},'cases':{r['case']:r['passed'] for r in results},'passed':len(results)==2 and all(r['passed'] for r in results)}
    with (out/'result.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
    print(json.dumps(result['cases']));return 0 if result['passed'] else 2
if __name__=='__main__':raise SystemExit(main())
