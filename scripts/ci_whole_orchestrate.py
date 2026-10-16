"""Bind claimed Windows CI captures to actual Git inventory; no CI authentication."""
import argparse,hashlib,json,os,subprocess,sys,threading,time
from pathlib import Path
import ci_whole_receipt as receipt

LIMIT=2*1024*1024
TARGET='x86_64-pc-windows-msvc'
GIT_ARGS=[['git','ls-files','-z'],['git','rev-parse','HEAD'],['git','diff','--quiet'],['git','diff','--cached','--quiet']]

def git_worker():
    request=json.loads(sys.stdin.buffer.readline(8193))
    receipt.reject(request['argv'] in GIT_ARGS+[['rustc','-vV']],'exact Git argv')
    return subprocess.run(request['argv'],cwd=request['root'],stdin=subprocess.DEVNULL,shell=False).returncode

def cleanup_git(kernel,job,process,threads):
    errors=[]
    if job:
        try:receipt.reject(bool(kernel.CloseHandle(job)),'Git original Job close failure')
        except BaseException as error:errors.append(error)
    if process:
        try:
            if process.poll() is None:process.kill()
        except BaseException as error:errors.append(error)
        try:process.wait(timeout=10)
        except BaseException as error:errors.append(error)
    for thread in threads:
        try:
            thread.join(timeout=5)
            receipt.reject(not thread.is_alive(),'Git cleanup reader terminal unconfirmed')
        except BaseException as error:errors.append(error)
    if errors:raise errors[0]

def run_git(root,argv):
    """Bounded argv output and original gated Job on Windows; errors fail closed."""
    receipt.reject(argv in GIT_ARGS+[['rustc','-vV']],'Git/toolchain command domain')
    gate=os.name=='nt';job=None;process=None;threads=[];buffers=[bytearray(),bytearray()];errors=[]
    def read(stream,dest):
        try:
            while True:
                block=stream.read(65536)
                if not block:break
                receipt.reject(len(dest)+len(block)<=LIMIT,'Git output bound');dest.extend(block)
        except BaseException as error:errors.append(error)
        finally:stream.close()
    try:
        process=subprocess.Popen([sys.executable,str(Path(__file__).absolute()),'--git-worker'] if gate else argv,cwd=root,stdin=subprocess.PIPE if gate else subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,shell=False)
        if gate:
            import ci_capture_operation as capture
            kernel,job=capture.job_for(process)
        for stream,buffer in zip((process.stdout,process.stderr),buffers):
            thread=threading.Thread(target=read,args=(stream,buffer),daemon=True);thread.start();threads.append(thread)
        if gate:process.stdin.write((json.dumps({'argv':argv,'root':str(root)})+'\n').encode());process.stdin.close()
        deadline=time.monotonic()+30
        while process.poll() is None:
            receipt.reject(not errors and time.monotonic()<deadline,'Git output/deadline failure');time.sleep(.02)
        if gate:capture.wait_natural_empty(lambda:capture.job_active(kernel,job),min(deadline,time.monotonic()+5))
        for thread in threads:thread.join(timeout=5)
        receipt.reject(not errors and all(not thread.is_alive() for thread in threads),'Git raw reader unconfirmed')
        receipt.reject(process.returncode==0,'Git dirty/failed command')
        return bytes(buffers[0])
    finally:
        cleanup_git(kernel if job else None,job,process,threads)

def inventory(root,identity,git=run_git):
    raw=git(root,GIT_ARGS[0]);head=git(root,GIT_ARGS[1])
    receipt.reject(head== (identity['commit']+'\n').encode() or head==(identity['commit']+'\r\n').encode(),'actual Git HEAD differs from claimed CI commit')
    for argv in GIT_ARGS[2:]:receipt.reject(git(root,argv)==b'','Git dirty output')
    source=receipt.snapshot(root,raw);paths=set();ids=set()
    for name in source:
        receipt.reject(name.casefold() not in paths,'tracked Windows path aliases');paths.add(name.casefold())
        native=receipt.native_identity(receipt.path(root,name));receipt.reject(native not in ids,'tracked native identity aliases');ids.add(native)
    receipt.reject(receipt.snapshot(root,raw)==source and git(root,GIT_ARGS[0])==raw,'source/identity/inventory sandwich changed')
    return raw,source

def validate_identity(identity):
    # Reuse exact constructor identity domain without inventing authenticated IDs.
    import re
    receipt.reject(isinstance(identity,dict) and set(identity)=={'commit','run_id','run_attempt','repository'},'claimed identity fields')
    receipt.reject(isinstance(identity['commit'],str) and re.fullmatch('[0-9a-f]{40}',identity['commit']),'claimed commit')
    for name in ['run_id','run_attempt']:receipt.reject(isinstance(identity[name],str) and re.fullmatch('[1-9][0-9]{0,19}',identity[name]),'claimed run identity')
    receipt.reject(isinstance(identity['repository'],str) and re.fullmatch('[A-Za-z0-9_.-]{1,100}/[A-Za-z0-9_.-]{1,100}',identity['repository']),'claimed repository')

def raw_create(file,raw):
    receipt.no_reparse(file)
    with file.open('xb') as stream:stream.write(raw)

def before(root,directory,identity,git=run_git):
    validate_identity(identity);folder=receipt.path(root,directory);receipt.reject(not folder.exists(),'fresh captures directory')
    raw,source=inventory(root,identity,git)
    for name in receipt.BINS:
        artifact=receipt.path(root,'target/release/'+name)
        receipt.reject(not artifact.exists(),'stale expected binary before compilation')
    folder.mkdir() # Existing files/source are never removed or overwritten.
    raw_create(folder/'tracked-before.bin',raw)
    receipt.publish(folder/'before.json',dict(schema=1,kind='claimed-ci-source-before',identity=identity,source=source,tracked_sha256=hashlib.sha256(raw).hexdigest(),remote_execution_authenticated=False,release_ready=False))

def lifecycle(root,directory,name,record):
    file=receipt.path(root,directory+'/'+name+'.lifecycle.json');original_pin=receipt.pin(file,65536);value=receipt.read_json(file,65536)
    fields={'schema','kind','name','native_run_performed','original_worker_job_assigned','natural_job_empty','job_closed','forced_cleanup','reader_drained','remote_execution_authenticated','native_gui_acceptance_verified','legal_coverage_complete','release_ready','worker_exit_code','final_active','accounting_queries','streams','passed'}
    receipt.reject(isinstance(value,dict) and set(value)==fields and value.get('schema')==1 and type(value.get('schema')) is int and value.get('kind')=='windows-ci-operation-lifecycle' and value.get('name')==name,'lifecycle identity')
    for key in ['passed','native_run_performed','original_worker_job_assigned','natural_job_empty','job_closed','reader_drained']:receipt.reject(value.get(key) is True,'lifecycle terminal '+key)
    receipt.reject(value.get('forced_cleanup') is False,'forced cleanup')
    for key in ['worker_exit_code','final_active']:receipt.reject(type(value.get(key)) is int and value[key]==0,'lifecycle naturalzero '+key)
    receipt.reject(type(value.get('accounting_queries')) is int and value['accounting_queries']>0,'actual accounting query')
    for key in ['remote_execution_authenticated','native_gui_acceptance_verified','legal_coverage_complete','release_ready']:receipt.reject(value.get(key) is False,'lifecycle readiness claim')
    streams=value.get('streams');receipt.reject(isinstance(streams,list) and len(streams)==2,'raw streams')
    pins={}
    for key,stream in zip(['stdout','stderr'],streams):
        receipt.reject(isinstance(stream,dict) and set(stream)=={'bytes','drained'} and stream['drained'] is True and type(stream['bytes']) is int,'raw stream terminal fields')
        receipt.reject(record[key]==directory+'/'+name+'.'+key,'exact raw capture path')
        pins[key]=receipt.pin(receipt.path(root,record[key]),8*1024*1024);receipt.reject(stream['bytes']==pins[key]['bytes'],'raw stream bytes differ')
    receipt.reject(receipt.pin(file,65536)==original_pin,'lifecycle changed while reading')
    return dict(path=directory+'/'+name+'.lifecycle.json',**original_pin),pins

def after(root,directory,identity,git=run_git):
    validate_identity(identity);folder=receipt.path(root,directory)
    receipt.reject(all(not (folder/name).exists() for name in ['whole.json','after.json','tracked-after.bin','operations.json']),'fresh final outputs')
    allowed=set(receipt.REQUIRED)|{'before','after','operations','whole'}
    for file in folder.iterdir():
        if file.name.endswith('.lifecycle.json'):receipt.reject(file.name[:-15] in receipt.REQUIRED,'extra lifecycle')
        elif file.name.endswith('.json'):receipt.reject(file.stem in allowed,'extra operation JSON')
    original=receipt.read_json(folder/'before.json',8*1024*1024)
    receipt.reject(isinstance(original,dict) and set(original)=={'schema','kind','identity','source','tracked_sha256','remote_execution_authenticated','release_ready'} and type(original['schema']) is int and original['schema']==1 and original['kind']=='claimed-ci-source-before' and original['identity']==identity and original['remote_execution_authenticated'] is False and original['release_ready'] is False,'before structure/identity')
    receipt.pin(folder/'tracked-before.bin',LIMIT)
    with (folder/'tracked-before.bin').open('rb') as stream:tracked=stream.read(LIMIT+1)
    receipt.reject(hashlib.sha256(tracked).hexdigest()==original['tracked_sha256'],'before inventory SHA')
    raw,source=inventory(root,identity,git);receipt.reject(raw==tracked and source==original['source'],'actual tracked inventory/source changed')
    records=[];bindings={};inputs=[folder/'before.json',folder/'tracked-before.bin']
    for name in receipt.REQUIRED:
        record_file=folder/(name+'.json');record_pin=receipt.pin(record_file,65536);record=receipt.read_json(record_file,65536)
        receipt.reject(receipt.pin(record_file,65536)==record_pin,'operation record changed while reading')
        receipt.reject(isinstance(record,dict) and set(record)=={'name','argv','exit_code','stdout','stderr'} and record['name']==name and record['argv']==receipt.REQUIRED[name] and type(record['exit_code']) is int and record['exit_code']==0,'exact successful operation')
        life,logs=lifecycle(root,directory,name,record);bindings[name]=dict(record=record_pin,lifecycle=life,raw=logs);records.append(record);inputs.extend([record_file,receipt.path(root,life['path']),*[receipt.path(root,record[k]) for k in ['stdout','stderr']]])
    initial={str(file):receipt.pin(file,8*1024*1024) for file in inputs}
    whole=receipt.construct(root,original['source'],source,records,identity,'target/release')
    for name in receipt.REQUIRED:
        for key in ['stdout','stderr']:
            bound=whole['operations'][name]['logs'][key]
            receipt.reject({k:bound[k] for k in ['sha256','bytes']}==bindings[name]['raw'][key],'raw lifecycle binding changed')
    final_raw,final_source=inventory(root,identity,git);receipt.reject(final_raw==tracked and final_source==source,'final Git/source changed')
    receipt.reject(all(receipt.pin(Path(file),8*1024*1024)==pin for file,pin in initial.items()),'capture/before input changed')
    raw_create(folder/'tracked-after.bin',raw);receipt.publish(folder/'after.json',source);receipt.publish(folder/'operations.json',records)
    whole['capture_lifecycle_bindings']=bindings
    whole['orchestration_scope']=dict(actual_git_inventory_checked=True,graph_integration_complete=False,registry_original_source_binding_complete=False,remote_execution_authenticated=False)
    receipt.publish(folder/'whole.json',whole) # Final publication only after every check.
    return whole

def main():
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['before','after']);p.add_argument('--root',type=Path,required=True);p.add_argument('--directory',default='ci-captures');a=p.parse_args()
    receipt.reject(os.name=='nt' and os.environ.get('RUNNER_OS')=='Windows','claimed Windows runner only')
    receipt.reject('..' not in a.root.parts,'root traversal');root=a.root.absolute();receipt.no_reparse(root)
    host=run_git(root,['rustc','-vV'])
    receipt.reject(host.splitlines().count(('host: '+TARGET).encode())==1,'exact MSVC rustc host required')
    receipt.reject(os.environ.get('CARGO_BUILD_TARGET',TARGET)==TARGET,'MSVC cargo environment target required')
    identity={key:os.environ.get(env) for key,env in [('commit','GITHUB_SHA'),('run_id','GITHUB_RUN_ID'),('run_attempt','GITHUB_RUN_ATTEMPT'),('repository','GITHUB_REPOSITORY')]}
    (before if a.mode=='before' else after)(root,a.directory,identity)
if __name__=='__main__':sys.exit(git_worker() if sys.argv[1:]==['--git-worker'] else main())
