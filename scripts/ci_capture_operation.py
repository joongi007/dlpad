"""Windows CI argv capture: replaces one existing stage, never reruns it.
A gated worker is bound to a kill-on-close Job before it launches cargo.
No GUI, installation, remote authentication or clean-descendant receipt claim.
"""
import argparse,ctypes,json,ntpath,os,subprocess,sys,time,threading
from pathlib import Path
from ci_whole_receipt import REQUIRED,path,publish,no_reparse,reject

def worker():
    request=json.loads(sys.stdin.readline())
    return subprocess.run(request['argv'],cwd=request['root'],shell=False,stdin=subprocess.DEVNULL).returncode

def job_for(process):
    from ctypes import wintypes as w
    class BASIC(ctypes.Structure):
        _fields_=[('user',ctypes.c_longlong),('job',ctypes.c_longlong),('flags',w.DWORD),('min',ctypes.c_size_t),('max',ctypes.c_size_t),('active',w.DWORD),('affinity',ctypes.c_size_t),('priority',w.DWORD),('scheduling',w.DWORD)]
    class IO(ctypes.Structure):_fields_=[(x,ctypes.c_ulonglong) for x in ['ro','wo','oo','rb','wb','ob']]
    class LIMIT(ctypes.Structure):_fields_=[('basic',BASIC),('io',IO),('process',ctypes.c_size_t),('job',ctypes.c_size_t),('peakprocess',ctypes.c_size_t),('peakjob',ctypes.c_size_t)]
    k=ctypes.WinDLL('kernel32',use_last_error=True)
    k.CreateJobObjectW.argtypes=[ctypes.c_void_p,w.LPCWSTR];k.CreateJobObjectW.restype=w.HANDLE
    k.SetInformationJobObject.argtypes=[w.HANDLE,ctypes.c_int,ctypes.c_void_p,w.DWORD];k.SetInformationJobObject.restype=w.BOOL
    k.AssignProcessToJobObject.argtypes=[w.HANDLE,w.HANDLE];k.AssignProcessToJobObject.restype=w.BOOL
    k.CloseHandle.argtypes=[w.HANDLE];k.CloseHandle.restype=w.BOOL
    k.QueryInformationJobObject.argtypes=[w.HANDLE,ctypes.c_int,ctypes.c_void_p,w.DWORD,ctypes.c_void_p]
    k.QueryInformationJobObject.restype=w.BOOL
    job=k.CreateJobObjectW(None,None);reject(bool(job),'CreateJobObject')
    try:
        limits=LIMIT();limits.basic.flags=0x2000
        reject(bool(k.SetInformationJobObject(job,9,ctypes.byref(limits),ctypes.sizeof(limits))),'kill-on-close Job limits')
        reject(bool(k.AssignProcessToJobObject(job,w.HANDLE(int(process._handle)))),'gated original worker Job assignment')
    except BaseException:k.CloseHandle(job);raise
    return k,job

LOG_LIMIT=8*1024*1024
RETAINED=[]  # Failed-drain original streams/files stay owned until process exit.

class ACCOUNTING(ctypes.Structure):
    _fields_=[(n,ctypes.c_longlong) for n in ('user','kernel','period_user','period_kernel')]+[(n,ctypes.c_uint32) for n in ('faults','total','active','terminated')]

def job_active(k,job):
    value=ACCOUNTING()
    if not k.QueryInformationJobObject(job,1,ctypes.byref(value),ctypes.sizeof(value),None):
        raise OSError(ctypes.get_last_error(),'original Job accounting query failed')
    return value.active

def job_processes(k,job):
    """Bounded diagnostic names from the original Job, never host-wide enumeration."""
    from ctypes import wintypes as w
    class PIDS(ctypes.Structure):
        _fields_=[('assigned',w.DWORD),('listed',w.DWORD),('pids',ctypes.c_size_t*64)]
    ids=PIDS()
    if not k.QueryInformationJobObject(job,3,ctypes.byref(ids),ctypes.sizeof(ids),None):
        error=ctypes.get_last_error()
        return [{'query_error':error,'truncated':error==234}]
    reject(ids.listed<=64,'Job diagnostic process bound')
    k.OpenProcess.argtypes=[w.DWORD,w.BOOL,w.DWORD];k.OpenProcess.restype=w.HANDLE
    k.IsProcessInJob.argtypes=[w.HANDLE,w.HANDLE,ctypes.POINTER(w.BOOL)];k.IsProcessInJob.restype=w.BOOL
    k.QueryFullProcessImageNameW.argtypes=[w.HANDLE,w.DWORD,w.LPWSTR,ctypes.POINTER(w.DWORD)];k.QueryFullProcessImageNameW.restype=w.BOOL
    result=[]
    for pid in list(ids.pids)[:ids.listed]:
        row={'pid':pid,'membership_confirmed':False};result.append(row)
        process=k.OpenProcess(0x1000,False,pid)
        if not process:
            row['query_error']=ctypes.get_last_error();continue
        try:
            member=w.BOOL()
            if not k.IsProcessInJob(process,job,ctypes.byref(member))or not member.value:
                continue
            row['membership_confirmed']=True
            image=ctypes.create_unicode_buffer(4096);length=w.DWORD(len(image))
            if k.QueryFullProcessImageNameW(process,0,image,ctypes.byref(length)):
                row['image']=ntpath.basename(image.value)
            else:row['query_error']=ctypes.get_last_error()
        finally:k.CloseHandle(process)
    return result

def report(name,facts,failure,stdout,stderr):
    print(f"CI {name}: worker_exit={facts.get('worker_exit_code')}; natural_job_empty={facts['natural_job_empty']}; reader_drained={facts['reader_drained']}; forced_cleanup={facts['forced_cleanup']}",flush=True)
    if failure is None and facts.get('worker_exit_code')==0:return
    if failure is not None:
        message=str(failure)[:1024].replace('\r','\\r').replace('\n','\\n').replace('\x00','\ufffd')
        print(f"CI {name} capture failure: {type(failure).__name__}: {message}",flush=True)
    if 'remaining_job_processes'in facts:
        print('Original Job remaining processes: '+json.dumps(facts['remaining_job_processes']),flush=True)
    for label,file in [('stdout',stdout),('stderr',stderr)]:
        try:
            with file.open('rb')as source:
                source.seek(0,2);source.seek(max(0,source.tell()-16384));raw=source.read(16384)
            if raw:
                print(f'CI {name} {label}: last at most 16384 bytes (full raw stream retained in artifact)',flush=True)
                for line in raw.decode('utf-8',errors='replace').splitlines():
                    # Prefix every line so captured tool output cannot inject Actions commands.
                    print(f'CI {name} {label} | '+line.replace('\x00','\ufffd'),flush=True)
        except OSError as error:
            print(f'CI {name} {label} tail unavailable: {type(error).__name__}',flush=True)

def wait_natural_empty(query,deadline,now=time.monotonic,pause=time.sleep):
    count=0
    while True:
        active=query();count+=1
        reject(type(active) is int and active>=0,'Job accounting domain')
        if active==0:return {'active':0,'queries':count}
        reject(now()<deadline,'original Job descendants did not naturally terminate')
        pause(.02)

def pump(source,target,result):
    count=0
    try:
        while True:
            chunk=source.read(65536)
            if not chunk:break
            reject(count+len(chunk)<=LOG_LIMIT,'operation raw log bound')
            target.write(chunk);count+=len(chunk)
        target.flush();os.fsync(target.fileno())
        result.update(bytes=count,drained=True)
    except BaseException as error:
        result.update(bytes=count,drained=False,error=type(error).__name__)
    finally:source.close()

def main():
    p=argparse.ArgumentParser();p.add_argument('--root',type=Path,required=True);p.add_argument('--name',choices=REQUIRED,required=True);p.add_argument('--directory',required=True);p.add_argument('--seconds',type=int,default=900);a=p.parse_args()
    reject(os.name=='nt' and 1<=a.seconds<=1800,'Windows bounded runner required')
    reject('..' not in a.root.parts,'root traversal rejected')
    root=a.root.absolute();no_reparse(root);folder=path(root,a.directory);reject(folder.is_dir(),'existing owned capture directory')
    stdout=path(root,a.directory+'/'+a.name+'.stdout');stderr=path(root,a.directory+'/'+a.name+'.stderr');record=path(root,a.directory+'/'+a.name+'.json')
    lifecycle=path(root,a.directory+'/'+a.name+'.lifecycle.json')
    reject(not record.exists() and not lifecycle.exists(),'fresh operation/lifecycle records')
    process=None;job=None;exit_code=None;threads=[];results=[{},{}]
    facts={'schema':1,'kind':'windows-ci-operation-lifecycle','name':a.name,'native_run_performed':False,'original_worker_job_assigned':False,'natural_job_empty':False,'job_closed':False,'forced_cleanup':False,'reader_drained':False,'remote_execution_authenticated':False,'native_gui_acceptance_verified':False,'legal_coverage_complete':False,'release_ready':False}
    failure=None
    # Files are retained on every failure; no path-derived deletion authority.
    out=stdout.open('xb');err=stderr.open('xb')
    print('CI '+a.name+' starting: '+json.dumps(REQUIRED[a.name]),flush=True)
    try:
        process=subprocess.Popen([sys.executable,str(Path(__file__).absolute()),'--worker'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,shell=False,cwd=root)
        k,job=job_for(process);facts['original_worker_job_assigned']=True
        for stream,target,result in zip((process.stdout,process.stderr),(out,err),results):
            thread=threading.Thread(target=pump,args=(stream,target,result),daemon=True);thread.start();threads.append(thread)
        deadline=time.monotonic()+a.seconds
        process.stdin.write((json.dumps({'argv':REQUIRED[a.name],'root':str(root)})+'\n').encode());process.stdin.close();facts['native_run_performed']=True
        while process.poll() is None:
            reject(not any('error' in result for result in results),'raw capture failed')
            reject(time.monotonic()<deadline,'operation timeout')
            time.sleep(.02)
        exit_code=process.returncode;facts['worker_exit_code']=exit_code
        ack=wait_natural_empty(lambda:job_active(k,job),min(deadline,time.monotonic()+5))
        facts.update(natural_job_empty=True,final_active=ack['active'],accounting_queries=ack['queries'])
    except BaseException as error:
        failure=error;facts['failure_kind']=type(error).__name__
        facts['failure_message']=str(error)[:1024]
        if job:
            try:facts['remaining_job_processes']=job_processes(k,job)
            except BaseException as diagnostic: facts['job_diagnostics_error']=type(diagnostic).__name__
    finally:
        if job:
            if not facts['natural_job_empty']:facts['forced_cleanup']=True
            facts['job_closed']=bool(k.CloseHandle(job))
            if not facts['job_closed'] and failure is None:failure=OSError('Job close failed')
        if process and process.poll() is None:
            facts['forced_cleanup']=True
            process.kill()
            try:process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                failure=failure or TimeoutError('original worker terminal unconfirmed')
        for thread in threads:thread.join(timeout=5)
        facts['reader_drained']=bool(threads) and all(not t.is_alive() for t in threads) and all(v.get('drained') is True for v in results)
        if not facts['reader_drained'] and failure is None:failure=ValueError('raw reader terminal not confirmed')
    if facts['reader_drained']:
        out.close();err.close();facts['streams']=results
    else:
        RETAINED.extend([out,err,process,*threads]);facts['streams']={'terminal_unconfirmed':True}
    facts['passed']=failure is None and exit_code==0 and facts['natural_job_empty'] and facts['job_closed'] and facts['reader_drained'] and not facts['forced_cleanup']
    if failure is not None:
        facts.setdefault('failure_kind',type(failure).__name__)
        facts.setdefault('failure_message',str(failure)[:1024])
    publish(lifecycle,facts)
    report(a.name,facts,failure,stdout,stderr)
    if failure is not None:return 125
    # Preserve legacy format binding; this record alone is NOT a Job cleanup proof.
    publish(record,dict(name=a.name,argv=REQUIRED[a.name],exit_code=exit_code,stdout=a.directory+'/'+a.name+'.stdout',stderr=a.directory+'/'+a.name+'.stderr'))
    return exit_code
if __name__=='__main__':sys.exit(worker() if sys.argv[1:]==['--worker'] else main())
