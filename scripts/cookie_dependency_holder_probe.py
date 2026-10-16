#!/usr/bin/env python3
"""Synthetic owned EOF keeper experiment; no parent Stop or signals."""
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

def pair(bus, reader, keeper, parent, read_fd, outcome):
    reader_code = 'import time,sys,os,signal;time.sleep(2);'+ {'success':'sys.exit(0)', 'exit7':'sys.exit(7)', 'selfsigterm':'os.kill(os.getpid(),signal.SIGTERM)'}[outcome]
    pipe_id = os.fstat(read_fd)
    keeper_code = ("import os,fcntl,sys;ids="+repr((pipe_id.st_dev,pipe_id.st_ino))+";"
        "fds=os.listdir('/proc/self/fd');"
        "exec(\"for n in fds:\\n try:\\n  fd=int(n);s=os.fstat(fd)\\n except OSError: continue\\n if (s.st_dev,s.st_ino)==ids: assert fcntl.fcntl(fd,fcntl.F_GETFL)&os.O_ACCMODE==os.O_RDONLY\");"
        "assert sys.stdin.buffer.read(1)==b''")
    with open('/dev/null','rb') as null:
        def build(basic,opened,closed):
            def prop(name,signature,value):
                opened('r','sv');basic('s',name);opened('v',signature);basic(signature,value);closed();closed()
            def names(name,values):
                opened('r','sv');basic('s',name);opened('v','as');opened('a','s')
                for value in values:basic('s',value)
                closed();closed();closed()
            def service(code,stdin,cap):
                for n,v in [('Type','exec'),('KillMode','control-group'),('Restart','no'),('CollectMode','inactive-or-failed')]:prop(n,'s',v)
                for n,v in [('AddRef',1),('RemainAfterExit',0),('Delegate',0)]:prop(n,'b',v)
                for n,v in [('TimeoutStartUSec',3000000),('RuntimeMaxUSec',cap*1000000),('TimeoutStopUSec',2000000)]:prop(n,'t',v)
                for n,v in [('StandardInputFileDescriptor',stdin),('StandardOutputFileDescriptor',null.fileno()),('StandardErrorFileDescriptor',null.fileno())]:prop(n,'h',v)
                opened('r','sv');basic('s','ExecStartEx');opened('v','a(sasas)');opened('a','(sasas)');opened('r','sasas');basic('s','/usr/bin/python3');opened('a','s')
                for v in ['/usr/bin/python3','-c',code]:basic('s',v)
                closed();opened('a','s');basic('s','no-env-expand');closed();closed();closed();closed();closed()
            basic('s',reader);basic('s','fail');opened('a','(sv)')
            service(reader_code,null.fileno(),3);prop('Slice','s',parent);names('Wants',[keeper]);names('After',[keeper]);closed()
            opened('a','(sa(sv))')
            opened('r','sa(sv)');basic('s',keeper);opened('a','(sv)');service(keeper_code,read_fd,12);prop('Slice','s','app.slice');names('Requires',[parent]);names('After',[parent]);closed();closed()
            opened('r','sa(sv)');basic('s',parent);opened('a','(sv)');prop('AddRef','b',1);prop('StopWhenUnneeded','b',1);prop('CollectMode','s','inactive-or-failed');closed();closed();closed()
        return bus.call('/org/freedesktop/systemd1','org.freedesktop.systemd1.Manager','StartTransientUnit',build,True)

def pid(value,fds):
    p=base.identity(int(value['MainPID']))
    if p is None or not base.same_live(p) or base.cgroup_of(p['pid'])!=value['ControlGroup']:raise RuntimeError('owned PID identity unavailable')
    fd=os.pidfd_open(p['pid']);fds.append(fd);return p,fd

def ended(value,p,fd):
    if base.same_identity(p) or not select.select([fd],[],[],0)[0]:raise RuntimeError('owned process terminal absent')
    if value.get('MainPID')!='0':raise RuntimeError('owned process MainPID nonzero')

def execute(case,env):
    token=uuid.uuid4().hex;reader='dlpadholderreader'+token+'.service';keeper='dlpadholderkeeper'+token+'.service';parent='dlpadcookie'+token+'.slice';sibling='dlpadholdersibling'+token+'.service'
    r={'case':case,'reader':reader,'keeper':keeper,'parent':parent,'parent_stop_called':False,'signals_sent':False,'started_utc':base.utc_now()}
    before=len(list(Path('/proc/self/fd').iterdir()));fds=[];bus=None;writer=None
    try:
        bus=lifetime.Bus();read_fd,writer=os.pipe2(os.O_CLOEXEC);fds.append(read_fd)
        outcome=case if case in ['exit7','selfsigterm'] else 'success'
        r['transaction_job']=pair(bus,reader,keeper,parent,read_fd,outcome)
        os.close(read_fd);fds.remove(read_fd)
        if case=='preidentity-owner-exit':
            # The owner loses its bus and EOF writer before GetUnit/Invocation capture.
            os.close(writer);writer=None;bus.close();bus=None;r['owner_closed_before_identity']=True
            time.sleep(3.2)
            observer=lifetime.Bus()
            try:r['gc']={name:dedicated.wait_gone(observer,name) for name in [reader,keeper,parent]}
            finally:observer.close()
            r['completed']=True
        else:
            rp=bus.get(reader);kp=bus.get(keeper);pp=bus.get(parent)
            ra=release.wait(reader,env,lambda p:p.get('ActiveState')=='active' and int(p.get('MainPID','0'))>1)
            ka=release.wait(keeper,env,lambda p:p.get('ActiveState')=='active' and int(p.get('MainPID','0'))>1)
            pa=release.wait(parent,env,lambda p:p.get('ActiveState')=='active' and bool(p.get('ControlGroup')))
            reader_pid,reader_fd=pid(ra,fds);keeper_pid,keeper_fd=pid(ka,fds)
            if ra.get('Slice')!=parent or ka.get('Slice')!='app.slice' or ka['ControlGroup'].startswith(pa['ControlGroup']+'/') or pa.get('StopWhenUnneeded')!='yes':raise RuntimeError('holder namespace relation invalid')
            pin=dedicated.pin_group(pa['ControlGroup'],fds)
            r.update(reader_initial=ra,keeper_initial=ka,parent_initial=pa,reader_pid=reader_pid,keeper_pid=keeper_pid,parent_native=lifetime.fd_identity(pin[1]),initial_native=dedicated.snapshot(pin))
            terminal=release.wait(reader,env,lambda p:p.get('ActiveState') in ['inactive','failed'],ra['InvocationID']);ended(terminal,reader_pid,reader_fd)
            code,status={'success':('1','0'),'exit7':('1','7'),'selfsigterm':('2','15')}[outcome]
            if (terminal.get('ExecMainCode'),terminal.get('ExecMainStatus'))!=(code,status):raise RuntimeError('original reader exit changed')
            now=release.properties(parent,env)
            if now['InvocationID']!=pa['InvocationID'] or now['ActiveState']!='active' or now['ControlGroup']!=pa['ControlGroup']:raise RuntimeError('holder failed to preserve parent')
            r.update(reader_terminal=terminal,strict_empty=dedicated.snapshot(pin))
            if not r['strict_empty']['held_empty'] or not r['strict_empty']['new_openat_empty']:raise RuntimeError('holder strict empty absent')
            if not base.same_live(keeper_pid) or select.select([keeper_fd],[],[],0)[0]:raise RuntimeError('keeper did not hold writer lifetime')
            bus.unref(rp);r['reader_gc']=dedicated.wait_gone(bus,reader)
            if case in ['sibling','pending']:
                r['sibling_job']=release.start_service(bus,sibling,parent,'oneshot' if case=='pending' else 'exec');sp=bus.get(sibling)
                expected='activating' if case=='pending' else 'active'
                sa=release.wait(sibling,env,lambda p:p.get('ActiveState')==expected and int(p.get('MainPID','0'))>1);sid,sfd=pid(sa,fds)
                if case=='pending' and (not sa.get('Job') or sa['Job'].split()[0]=='0'):raise RuntimeError('pending job absent')
                r.update(sibling_before=sa,sibling_pid=sid,parent_populated=dedicated.snapshot(pin))
                if r['parent_populated']['held_empty']:raise RuntimeError('sibling populated missed')
            os.close(writer);writer=None;r['writer_closed']=True
            kt=release.normal_terminal(keeper,env,ka['InvocationID']);ended(kt,keeper_pid,keeper_fd)
            r['keeper_terminal']=kt;bus.unref(kp);r['keeper_gc']=dedicated.wait_gone(bus,keeper)
            if case in ['sibling','pending']:
                after=release.properties(sibling,env);pafter=release.properties(parent,env)
                r.update(sibling_after=after,parent_after=pafter)
                if after['InvocationID']!=sa['InvocationID'] or after['ActiveState']!=expected or not base.same_live(sid) or select.select([sfd],[],[],0)[0] or pafter['InvocationID']!=pa['InvocationID'] or pafter['ActiveState']!='active':raise RuntimeError('sibling interrupted by holder EOF')
                if case=='pending' and after.get('Job')!=sa.get('Job'):raise RuntimeError('pending job changed')
                st=release.normal_terminal(sibling,env,sa['InvocationID']);ended(st,sid,sfd);r['sibling_terminal']=st;bus.unref(sp);r['sibling_gc']=dedicated.wait_gone(bus,sibling)
            pt=release.wait(parent,env,lambda p:p.get('ActiveState') in ['inactive','failed'],pa['InvocationID']);r['parent_terminal']=pt;bus.unref(pp);r['parent_gc']=dedicated.wait_gone(bus,parent)
            r['parent_cgroup_absent']=not pin[0].exists()
            if not r['parent_cgroup_absent']:raise RuntimeError('parent native cgroup not collected')
            r['completed']=True
    except Exception as e:r.update(completed=False,error_type=type(e).__name__,error=str(e))
    finally:
        if writer is not None:os.close(writer)
        for fd in reversed(fds):os.close(fd)
        if bus is not None:bus.close()
        r.update(fd_before=before,fd_after=len(list(Path('/proc/self/fd').iterdir())),ended_utc=base.utc_now())
        r['passed']=r.get('completed') is True and r['fd_before']==r['fd_after']
    return r

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=Path);a=p.parse_args();out=a.output
    if not out.is_absolute() or '..' in out.parts or not out.is_relative_to(PROJECT/'docs/validation/evidence') or out.exists() or any(x.is_symlink() for x in [out,*out.parents]):p.error('fresh owned evidence required')
    runtime=Path('/run/user')/str(os.getuid())
    for path in [runtime/'bus',runtime/'systemd/private']:
        info=path.lstat()
        if info.st_uid!=os.getuid() or not stat.S_ISSOCK(info.st_mode):p.error('owned user manager required')
    env=dict(os.environ,XDG_RUNTIME_DIR=str(runtime),DBUS_SESSION_BUS_ADDRESS='unix:path='+str(runtime/'bus'),LC_ALL='C');out.mkdir(mode=0o700);results=[]
    for case in ['success','exit7','selfsigterm','preidentity-owner-exit','sibling','pending']:
        result=execute(case,env);data=(json.dumps(result,indent=2)+'\n').encode()
        if len(data)>262144:raise RuntimeError('evidence bound')
        with (out/(case+'.json')).open('xb') as f:f.write(data)
        results.append(result)
        if not result['passed']:break
    result={'source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'reused_sources':{Path(m.__file__).name:hashlib.sha256(Path(m.__file__).read_bytes()).hexdigest() for m in [release,dedicated,lifetime,base]},'cases':{r['case']:r['passed'] for r in results},'passed':len(results)==6 and all(r['passed'] for r in results)}
    with (out/'result.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
    print(json.dumps(result['cases']));return 0 if result['passed'] else 2
if __name__=='__main__':raise SystemExit(main())
