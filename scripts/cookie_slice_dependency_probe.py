#!/usr/bin/env python3
"""Owned service+auxiliary slice dependency experiment; no normal parent Stop."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import select
import stat
import time
import uuid
import cookie_slice_release_probe as release
import cookie_dedicated_slice_probe as dedicated
import cookie_cgroup_lifetime_probe as lifetime
import systemd_user_guardian_probe as base
PROJECT = Path(__file__).resolve().parent.parent

def start_pair(bus, unit, parent):
    argv = ['/usr/bin/python3', '-c', 'import time;time.sleep(2)']
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
            closed(); opened('a','s'); basic('s','no-env-expand'); closed(); closed(); closed(); closed(); closed(); closed()
            opened('a','(sa(sv))'); opened('r','sa(sv)'); basic('s',parent); opened('a','(sv)')
            prop('AddRef','b',1); prop('StopWhenUnneeded','b',1); prop('CollectMode','s','inactive-or-failed')
            closed(); closed(); closed()
        return bus.call('/org/freedesktop/systemd1','org.freedesktop.systemd1.Manager','StartTransientUnit',build,True)

def stop_exact(bus, unit, path, invocation):
    # Never target a reused invocation or a different unit.
    if bus.get(unit) != path: raise RuntimeError('stop unit path changed')
    def build(basic, opened, closed): basic('s',unit); basic('s','fail')
    bus.call('/org/freedesktop/systemd1','org.freedesktop.systemd1.Manager','StopUnit',build,True)

def execute(case, env):
    nonce=uuid.uuid4().hex; parent='dlpadcookie'+nonce+'.slice'; proof='dlpad-dependency-'+nonce+'.service'; sibling='dlpad-sibling-'+nonce+'.service'
    r={'case':case,'parent':parent,'service':proof,'sibling':sibling,'normal_parent_stop':False,'worker_seconds':2,'runtime_max_seconds':3,'started_utc':base.utc_now()}
    baseline=len(list(Path('/proc/self/fd').iterdir())); bus=None; fds=[]; owned=[]; pin=None; parent_inv=None; parent_path=None
    try:
        bus=lifetime.Bus(); r['transaction_job']=start_pair(bus,proof,parent)
        proof_path=bus.get(proof); parent_path=bus.get(parent)
        active=release.wait(proof,env,lambda p:p.get('ActiveState')=='active' and int(p.get('MainPID','0'))>1)
        pa=release.wait(parent,env,lambda p:p.get('ActiveState')=='active' and bool(p.get('ControlGroup')))
        inv=active['InvocationID']; parent_inv=pa['InvocationID']; owned.append((proof,proof_path,inv))
        if pa.get('StopWhenUnneeded')!='yes' or active.get('Slice')!=parent: raise RuntimeError('dependency properties missing')
        pid,pidfd=release.own_pid(active,pa['ControlGroup'],fds); pin=dedicated.pin_group(pa['ControlGroup'],fds)
        r.update(initial_service=active,initial_parent=pa,worker_pid=pid,parent_native_identity=lifetime.fd_identity(pin[1]),parent_initial=dedicated.snapshot(pin))
        terminal=release.wait(proof,env,lambda p:p.get('ActiveState')=='active' and p.get('SubState')=='exited' and p.get('MainPID')=='0',inv)
        if terminal.get('ExecMainCode')!='1' or terminal.get('ExecMainStatus')!='0': raise RuntimeError('worker natural success absent')
        r.update(reader_terminal=terminal,worker_gone=not base.same_identity(pid),pidfd_ready=bool(select.select([pidfd],[],[],0)[0]),strict_empty=dedicated.snapshot(pin))
        if not r['worker_gone'] or not r['pidfd_ready'] or not r['strict_empty']['held_empty'] or not r['strict_empty']['new_openat_empty']: raise RuntimeError('strict terminal empty absent')
        if case!='empty':
            r['sibling_job']=release.start_service(bus,sibling,parent,'oneshot' if case=='pending' else 'exec'); sp=bus.get(sibling)
            expected='activating' if case=='pending' else 'active'
            sa=release.wait(sibling,env,lambda p:p.get('ActiveState')==expected and int(p.get('MainPID','0'))>1)
            sinv=sa['InvocationID']; owned.append((sibling,sp,sinv)); sid,sfd=release.own_pid(sa,pa['ControlGroup'],fds)
            if case=='pending' and (not sa.get('Job') or sa['Job'].split()[0]=='0'): raise RuntimeError('pending job absent')
            r.update(sibling_before_release=sa,sibling_pid=sid,parent_populated=dedicated.snapshot(pin))
            if r['parent_populated']['held_empty']: raise RuntimeError('sibling mistaken as empty')
        if release.properties(proof,env).get('InvocationID')!=inv: raise RuntimeError('reader stop invocation changed')
        r['reader_stop_job']=stop_exact(bus,proof,proof_path,inv)
        release.wait(proof,env,lambda p:p.get('ActiveState') in ['inactive','failed'],inv)
        bus.unref(proof_path); r['reader_gc']=dedicated.wait_gone(bus,proof)
        if case!='empty':
            after=release.properties(sibling,env); pafter=release.properties(parent,env)
            r.update(sibling_after_release=after,parent_after_release=pafter,parent_after_release_native=dedicated.snapshot(pin))
            if after.get('InvocationID')!=sinv or after.get('ActiveState')!=expected or not base.same_live(sid) or select.select([sfd],[],[],0)[0]: raise RuntimeError('sibling interrupted by reader release')
            if pafter.get('InvocationID')!=parent_inv or pafter.get('ActiveState')!='active': raise RuntimeError('parent dependency not retained')
            if case=='pending' and after.get('Job')!=sa.get('Job'): raise RuntimeError('pending job altered')
            r['sibling_terminal']=release.normal_terminal(sibling,env,sinv)
            if base.same_identity(sid) or not select.select([sfd],[],[],0)[0]: raise RuntimeError('sibling terminal missing')
            bus.unref(sp);r['sibling_gc']=dedicated.wait_gone(bus,sibling)
        # Parent must stop itself from its dependency lifetime, never from our Stop.
        r['parent_terminal']=release.wait(parent,env,lambda p:p.get('ActiveState') in ['inactive','failed'],parent_inv)
        bus.unref(parent_path); r['parent_gc']=dedicated.wait_gone(bus,parent)
        r['parent_cgroup_gone']=not pin[0].exists(); r['completed']=r['parent_cgroup_gone']
    except Exception as error: r.update(completed=False,error_type=type(error).__name__,error=str(error))
    finally:
        r['failure_cleanup']=[]
        if not r.get('completed') and bus is not None:
            # Bounded workers finish naturally before exact-owned residual cleanup.
            time.sleep(3.1)
            for unit,path,inv in owned:
                try:
                    value=release.properties(unit,env)
                    if value.get('InvocationID')!=inv: continue
                    if value.get('ActiveState')=='active' and value.get('SubState')=='exited': stop_exact(bus,unit,path,inv)
                    bus.unref(path);r['failure_cleanup'].append({'unit':unit,'unref':True})
                except Exception as error:r['failure_cleanup'].append({'unit':unit,'error':str(error)})
            if parent_path and parent_inv and pin:
                try:
                    value=release.properties(parent,env); empty=dedicated.snapshot(pin)
                    if value.get('InvocationID')==parent_inv and empty['held_empty'] and empty['new_openat_empty']:
                        stop_exact(bus,parent,parent_path,parent_inv);bus.unref(parent_path)
                        r['failure_cleanup'].append({'unit':parent,'explicit_failure_only_parent_stop':True,'strict_empty':empty,'gc':dedicated.wait_gone(bus,parent)})
                except Exception as error:r['failure_cleanup'].append({'unit':parent,'error':str(error)})
        for fd in reversed(fds): os.close(fd)
        if bus: bus.close()
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
    for case in ['empty','sibling','pending']:
        r=execute(case,env);data=(json.dumps(r,indent=2)+'\n').encode()
        if len(data)>262144:raise RuntimeError('evidence bound')
        with (out/(case+'.json')).open('xb') as f:f.write(data)
        results.append(r)
        if not r['passed']:break
    result={'source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'reused_sources':{Path(m.__file__).name:hashlib.sha256(Path(m.__file__).read_bytes()).hexdigest() for m in [release,dedicated,lifetime,base]},'cases':{r['case']:r['passed'] for r in results},'passed':len(results)==3 and all(r['passed'] for r in results)}
    with (out/'result.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
    print(json.dumps(result['cases']));return 0 if result['passed'] else 2
if __name__=='__main__':raise SystemExit(main())
