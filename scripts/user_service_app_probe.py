#!/usr/bin/env python3
"""Single immutable CPU app: default service backend with private native GUI fixtures."""
import argparse, json, os, select, signal, subprocess, sys, time
from pathlib import Path
import cpu_background_probe as background
import systemd_user_guardian_probe as service
from accessible_download_probe import file_hash

class CapturePopen:
    """Only this imported module's launches are observed; no stdlib mutation."""
    def __init__(self,binary,records):self.binary=binary;self.records=records
    def __getattr__(self,name):return getattr(subprocess,name)
    def Popen(self,args,*positional,**kwargs):
        child=subprocess.Popen(args,*positional,**kwargs)
        if args and Path(args[0])==self.binary:
            env=kwargs.get('env',os.environ)
            self.records.append({'identity':service.identity(child.pid),'backend_key_absent_in_launch_env':'DLPAD_GUARDIAN_BACKEND' not in env,'debug_bus_key_absent_in_launch_env':'DLPAD_TEST_SYSTEMD_BUS_PATH' not in env})
        return child

def environment_key_present(pid,key):
    data=bytearray()
    with Path(f'/proc/{pid}/environ').open('rb') as stream:
        for block in iter(lambda:stream.read(65536),b''):
            data.extend(block)
            if len(data)>1024*1024:raise RuntimeError('app environment inspection bound exceeded')
    return any(item.startswith(key.encode()+b'=') for item in data.split(b'\0'))

def run_case(args,name,host_env):
    proof={'case':name,'started_utc':service.utc_now(),'backend_env_explicitly_selected':False,'user_display_used':False,'observations':[],'apps':[],'actions':[],'release_fault_env':args.release_fault_env}
    captured=[];sentinels=[];fds=[];real_pin=background.pin_worker;real_snapshot=background.snapshot_database;real_subprocess=background.subprocess
    sentinel=subprocess.Popen([sys.executable,'-c','import time;time.sleep(900)'],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL);sentinel_identity=service.identity(sentinel.pid)
    def observed_pin(root,token):
        info,worker_fd=real_pin(root,token)
        try:
            fields=Path(f"/proc/{info['pid']}/stat").read_text().rsplit(')',1)[1].split();guardian_pid=int(fields[1]);guardian=service.identity(guardian_pid)
            if guardian is None:raise RuntimeError('worker parent guardian absent')
            cgroup=service.cgroup_of(guardian_pid);unit=cgroup.rsplit('/',1)[-1]
            if not unit.startswith('dlpad-job-') or not unit.endswith('.service') or not cgroup.startswith(f'/user.slice/user-{os.getuid()}.slice/user@{os.getuid()}.service/'):raise RuntimeError('default app did not use owned user service')
            props=service.props(unit,host_env)['properties'];invocation=props.get('InvocationID')
            if not invocation or len(invocation)!=32:raise RuntimeError('InvocationID missing')
            initial=service.verify_unit(unit,invocation,guardian,host_env)
            if service.cgroup_of(info['pid'])!=cgroup:raise RuntimeError('worker/guardian cgroup mismatch')
            executable=Path(f'/proc/{guardian_pid}/exe');actual=executable.stat();expected=args.binary.stat()
            if (actual.st_dev,actual.st_ino)!=(expected.st_dev,expected.st_ino) or file_hash(executable)!=args.expected_sha:raise RuntimeError('guardian is not the same immutable app executable')
            if list(args.binary.parent.iterdir())!=[args.binary]:raise RuntimeError('single executable snapshot gained helper/extra files')
            descriptor=os.pidfd_open(guardian_pid)
            if not service.same_live(guardian):os.close(descriptor);raise RuntimeError('guardian identity changed while pinning')
            fds.append((guardian,descriptor))
            apps=[]
            for app in proof['apps']:
                pid=app['identity']['pid']
                if service.same_live(app['identity']):
                    app['backend_key_absent_in_actual_environ']=not environment_key_present(pid,'DLPAD_GUARDIAN_BACKEND')
                    app['debug_bus_key_absent_in_actual_environ']=not environment_key_present(pid,'DLPAD_TEST_SYSTEMD_BUS_PATH')
                    app['startup_fault_key_present_in_actual_environ']=environment_key_present(pid,'DLPAD_TEST_SYSTEMD_STARTUP_FAULT')
                    apps.append(app)
            if args.release_fault_env and (not all(app['startup_fault_key_present_in_actual_environ'] for app in apps) or not environment_key_present(guardian_pid,'DLPAD_TEST_SYSTEMD_STARTUP_FAULT')):raise RuntimeError('release fault env was not propagated to actual app/guardian')
            if not apps or not all(all(app.get(key,False) for key in ('backend_key_absent_in_launch_env','debug_bus_key_absent_in_launch_env','backend_key_absent_in_actual_environ','debug_bus_key_absent_in_actual_environ')) for app in apps):raise RuntimeError('actual app backend/debug override was present')
            marker=root/'media'/'.owned-library-sentinel.bin';marker.write_bytes(b'owned unrelated library sentinel')
            meta=marker.stat();sentinels.append((marker,meta.st_dev,meta.st_ino,file_hash(marker)))
            observation={'worker':info,'guardian':guardian,'unit':unit,'invocation_id':invocation,'control_group':cgroup,'initial_unit':initial,'guardian_exe_device':actual.st_dev,'guardian_exe_inode':actual.st_ino,'guardian_exe_sha256':file_hash(executable),'helper_absent':True,'actual_apps_backend_env_absent':True,'release_startup_fault_env_present':bool(args.release_fault_env)}
            captured.append(observation);proof['observations']=captured
            return info,worker_fd
        except Exception:
            os.close(worker_fd)
            raise
    def observed_snapshot(data,destination):
        proof['library_sentinels_preserved_before_owned_temp_cleanup']=all(path.exists() and (path.stat().st_dev,path.stat().st_ino,file_hash(path))==(dev,ino,sha) for path,dev,ino,sha in sentinels)
        final=background.job(data);proof['final_job']=final
        if name=='stop-exit':proof['close_stop_cancelled']=final is not None and final['job']['state']=='Cancelled'
        if name=='complete-hidden':
            artifact=Path(final['job']['artifacts'][0]['path']) if final and final['job'].get('artifacts') else None
            proof['download_artifact_verified']=artifact is not None and artifact.is_relative_to(data.parent/'media') and artifact.read_bytes()==b'synthetic candidate artifact'
        return real_snapshot(data,destination)
    background.pin_worker=observed_pin;background.snapshot_database=observed_snapshot;background.subprocess=CapturePopen(args.binary,proof['apps'])
    try:
        ui=background.run_case(args,name);proof['ui']=ui
        deadline=time.monotonic()+8
        while any(service.same_identity(observation['guardian']) or service.same_identity(observation['worker']) for observation in captured) and time.monotonic()<deadline:time.sleep(.05)
        for observation in captured:
            observation['guardian_identity_gone']=not service.same_identity(observation['guardian']);observation['worker_identity_gone']=not service.same_identity(observation['worker'])
            path=Path('/sys/fs/cgroup')/observation['control_group'].lstrip('/');observation['cgroup_removed']=not path.exists()
            deadline=time.monotonic()+3
            while time.monotonic()<deadline:
                terminal=service.props(observation['unit'],host_env)['properties']
                if terminal.get('LoadState')=='not-found':break
                time.sleep(.05)
            observation['terminal_unit']=terminal;observation['owned_unit_gc']=terminal.get('LoadState')=='not-found'
        proof['sentinel_unchanged_alive']=service.same_live(sentinel_identity)
        proof['passed']=ui['passed'] and bool(captured) and all(all(observation[key] for key in ('guardian_identity_gone','worker_identity_gone','cgroup_removed','owned_unit_gc','helper_absent','actual_apps_backend_env_absent')) for observation in captured) and proof.get('library_sentinels_preserved_before_owned_temp_cleanup',False) and proof['sentinel_unchanged_alive'] and proof.get('close_stop_cancelled',True) and proof.get('download_artifact_verified',True)
    except Exception as error:proof.update(passed=False,error=str(error))
    finally:
        background.pin_worker=real_pin;background.snapshot_database=real_snapshot;background.subprocess=real_subprocess
        for record,fd in fds:
            if service.same_live(record):signal.pidfd_send_signal(fd,signal.SIGKILL);proof['actions'].append('cleanup exact observed guardian pidfd')
            proof.setdefault('guardian_pidfds_ready',[]).append(bool(select.select([fd],[],[],0)[0]));os.close(fd)
        proof['passed']=proof.get('passed',False) and all(proof.get('guardian_pidfds_ready',[]))
        sentinel.terminate();sentinel.wait(timeout=5);proof['sentinel_owned_cleanup_exit']=sentinel.returncode
        proof['ended_utc']=service.utc_now();service.write_json(args.output/name/'service-proof.json',proof)
    return proof

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--binary',type=Path,required=True);parser.add_argument('--expected-sha',required=True);parser.add_argument('--output',type=Path,required=True);parser.add_argument('--font',type=Path,default=Path('/mnt/c/Windows/Fonts/malgun.ttf'));parser.add_argument('--release-fault-env',choices=['after-start']);parser.add_argument('--case',action='append',choices=['active-hidden','complete-hidden','stop-exit']);args=parser.parse_args()
    binary=args.binary
    if not binary.is_absolute() or binary.is_symlink() or binary.name!='dlpad-cpu' or not binary.is_relative_to(Path('/tmp')) or not binary.is_file() or any(p.is_symlink() for p in binary.parents):parser.error('explicit immutable /tmp single dlpad-cpu required')
    if sorted(binary.parent.iterdir())!=[binary] or file_hash(binary)!=args.expected_sha:parser.error('single executable snapshot SHA/contents mismatch')
    project=Path(__file__).resolve().parent.parent;evidence=project/'docs/validation/evidence';out=args.output
    if not out.is_absolute() or '..' in out.parts or not out.is_relative_to(evidence) or out==evidence or out.exists() or any(p.is_symlink() for p in [out,*out.parents]):parser.error('new explicit project evidence path required')
    for key in ('DLPAD_GUARDIAN_BACKEND','DLPAD_TEST_SYSTEMD_BUS_PATH','DLPAD_TEST_SYSTEMD_STARTUP_FAULT'):os.environ.pop(key,None)
    if args.release_fault_env:os.environ['DLPAD_TEST_SYSTEMD_STARTUP_FAULT']=args.release_fault_env
    runtime=Path('/run/user')/str(os.getuid());host_env=dict(os.environ);host_env['XDG_RUNTIME_DIR']=str(runtime);host_env['DBUS_SESSION_BUS_ADDRESS']='unix:path='+str(runtime/'bus')
    out.mkdir(mode=0o700);args.binary=binary
    result={'started_utc':service.utc_now(),'scope':'Linux registered CPU app default user-service; synthetic native UI on isolated Xvfb only','boot_id':Path('/proc/sys/kernel/random/boot_id').read_text().strip(),'binary':str(binary),'binary_sha256_before':file_hash(binary),'probe_sha256':file_hash(Path(__file__)),'backend_env_selected':False,'release_fault_env':args.release_fault_env,'desktop_fallback':False,'existing_units_targeted':False,'cases':{}}
    for name in args.case or ['active-hidden','complete-hidden','stop-exit']:result['cases'][name]=run_case(args,name,host_env)
    result['binary_sha256_after']=file_hash(binary);result['single_executable_snapshot_after']=sorted(binary.parent.iterdir())==[binary];result['passed']=all(r['passed'] for r in result['cases'].values()) and result['single_executable_snapshot_after'] and result['binary_sha256_after']==args.expected_sha;result['ended_utc']=service.utc_now();service.write_json(out/'result.json',result)
    print(json.dumps({'passed':result['passed'],'cases':{name:r['passed'] for name,r in result['cases'].items()}}));return 0 if result['passed'] else 2
if __name__=='__main__':sys.exit(main())
