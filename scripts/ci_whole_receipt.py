"""Bounded CI-format receipt constructor; never executes commands or authenticates CI."""
import argparse
import hashlib
import json
import re
import os
import stat
from pathlib import Path

SHA = re.compile(r'[0-9a-f]{64}\Z')
REQUIRED = {
    'fmt':['cargo','fmt','--all','--','--check'],
    'clippy':['cargo','clippy','--workspace','--features','dlpad/cpu','--all-targets','--locked','--offline','--','-D','warnings'],
    'test':['cargo','test','--workspace','--features','dlpad/cpu','--locked','--offline'],
    'release':['cargo','build','--workspace','--release','--locked','--offline'],
    'cpu-release':['cargo','build','-p','dlpad','--bin','dlpad-cpu','--features','cpu','--release','--locked','--offline'],
    'installer-release':['cargo','build','-p','dlpad','--bin','dlpad-install','--features','cpu','--release','--locked','--offline'],
    'companion-release':['cargo','build','-p','dlpad-engine','--bin','dlpad-local-install','--bin','dlpad-process-guard','--release','--locked','--offline'],
}
BINS=('dlpad-cpu.exe','dlpad-install.exe','dlpad-local-install.exe','dlpad-process-guard.exe')

def reject(ok,message):
    if not ok:raise ValueError(message)

def relative_parts(relative):
    reject(isinstance(relative,str) and 0<len(relative)<=32768 and '\\' not in relative,'relative path domain')
    parts=relative.split('/')
    for part in parts:
        reject(part not in ('','.', '..') and not part.endswith((' ','.')) and not any(ord(c)<32 or c in ':<>"|?*' for c in part),'Windows normalized component')
        reject(not re.fullmatch(r'(CON|PRN|AUX|NUL|COM[1-9¹²³]|LPT[1-9¹²³])',part.split('.')[0],re.I),'Windows device component')
    return parts

def no_reparse(file):
    for item in [file,*file.parents]:
        try:info=item.lstat()
        except FileNotFoundError:continue
        reject(not stat.S_ISLNK(info.st_mode) and not getattr(info,'st_file_attributes',0)&0x400,'symlink/reparse path')

def path(root,relative):
    result=root.joinpath(*relative_parts(relative));no_reparse(result);return result

def state(info):
    return (info.st_dev,info.st_ino,info.st_size,info.st_mtime_ns,info.st_ctime_ns)

def windows_handle_state(handle):
    """Query one retained HANDLE; path stat/CRT fstat disagree about ctime."""
    import ctypes
    from ctypes import wintypes as w
    class Basic(ctypes.Structure):
        _fields_=[(name,ctypes.c_longlong) for name in
                  ('created','accessed','written','changed')]+[('attributes',w.DWORD)]
    class Standard(ctypes.Structure):
        _fields_=[('allocated',ctypes.c_longlong),('size',ctypes.c_longlong),
                  ('links',w.DWORD),('deleted',ctypes.c_ubyte),('directory',ctypes.c_ubyte)]
    class Identity(ctypes.Structure):
        _fields_=[('volume',ctypes.c_ulonglong),('file_id',ctypes.c_ubyte*16)]
    kernel=ctypes.WinDLL('kernel32',use_last_error=True)
    kernel.GetFileInformationByHandleEx.argtypes=[w.HANDLE,ctypes.c_int,ctypes.c_void_p,w.DWORD]
    kernel.GetFileInformationByHandleEx.restype=w.BOOL
    identity,basic,standard=Identity(),Basic(),Standard()
    for kind,value in [(18,identity),(0,basic),(1,standard)]:
        if not kernel.GetFileInformationByHandleEx(handle,kind,ctypes.byref(value),ctypes.sizeof(value)):
            raise OSError(ctypes.get_last_error(),'original file HANDLE query failed')
    reject(not standard.directory and not standard.deleted and not basic.attributes&0x400,
           'regular non-reparse original HANDLE required')
    reject(any(identity.file_id),'zero native file ID is unsupported')
    return ((identity.volume,bytes(identity.file_id)),standard.size,basic.created,
            basic.written,basic.changed,basic.attributes)

def windows_path_state(file):
    import ctypes
    from ctypes import wintypes as w
    kernel=ctypes.WinDLL('kernel32',use_last_error=True)
    kernel.CreateFileW.argtypes=[w.LPCWSTR,w.DWORD,w.DWORD,ctypes.c_void_p,w.DWORD,w.DWORD,w.HANDLE]
    kernel.CreateFileW.restype=w.HANDLE
    kernel.CloseHandle.argtypes=[w.HANDLE];kernel.CloseHandle.restype=w.BOOL
    # Read attributes, share R/W/delete, open the leaf reparse point itself.
    handle=kernel.CreateFileW(str(file),0x80,7,None,3,0x00200000,None)
    if handle in (None,ctypes.c_void_p(-1).value):
        raise OSError(ctypes.get_last_error(),'original pathname HANDLE open failed')
    try:return windows_handle_state(handle)
    finally:
        if not kernel.CloseHandle(handle):
            raise OSError(ctypes.get_last_error(),'original pathname HANDLE close failed')

def native_identity(file):
    no_reparse(file);info=file.lstat();reject(stat.S_ISREG(info.st_mode),'regular input required')
    if os.name=='nt':return windows_path_state(file)[0]
    return (info.st_dev,info.st_ino)

def pin(file,limit):
    no_reparse(file);original=file.lstat();reject(stat.S_ISREG(original.st_mode),'regular input file required')
    h=hashlib.sha256();count=0
    native_before=windows_path_state(file) if os.name=='nt' else None
    with file.open('rb') as stream:
        opened=os.fstat(stream.fileno());reject(stat.S_ISREG(opened.st_mode),'regular opened input required')
        if os.name=='nt':
            import msvcrt
            handle=msvcrt.get_osfhandle(stream.fileno())
            reject(windows_handle_state(handle)==native_before and opened.st_size==native_before[1],
                   'original opened file changed')
        else:reject(state(original)==state(opened),'original opened file changed')
        reject(opened.st_size<=limit,'file bound')
        while chunk:=stream.read(1024*1024):
            count+=len(chunk);reject(count<=limit,'file bound');h.update(chunk)
        reject(state(os.fstat(stream.fileno()))==state(opened) and count==opened.st_size,'opened input changed while hashing')
        no_reparse(file)
        if os.name=='nt':
            reject(windows_handle_state(handle)==native_before and windows_path_state(file)==native_before,
                   'original HANDLE or pathname changed while hashing')
        else:reject(state(file.lstat())==state(opened),'pathname input changed while hashing')
    return dict(sha256=h.hexdigest(),bytes=count)

def unique(pairs):
    out={}
    for k,v in pairs:
        reject(k not in out,'duplicate JSON field');out[k]=v
    return out

def read_json(file,limit):
    with file.open('rb') as f:raw=f.read(limit+1)
    reject(len(raw)<=limit,'JSON byte bound')
    try:return json.loads(raw,object_pairs_hook=unique)
    except (ValueError,UnicodeError,TypeError) as e:raise ValueError('JSON domain') from e

def snapshot(root,tracked):
    reject(isinstance(tracked,bytes) and len(tracked)<=2*1024*1024 and tracked.endswith(b'\0'),'git tracked NUL list required')
    try:names=tracked[:-1].decode('utf-8').split('\0')
    except UnicodeError as e:raise ValueError('tracked UTF8') from e
    reject(0<len(names)<=16384 and len(set(names))==len(names),'tracked path count/duplicates')
    result={};total=0
    for name in sorted(names):
        value=pin(path(root,name),16*1024*1024);total+=value['bytes'];reject(total<=256*1024*1024,'tracked aggregate bound');result[name]=value
    reject({'Cargo.lock','rust-toolchain.toml'}<=set(result),'lock/toolchain tracked required')
    return result

def validate_snapshot(value):
    reject(isinstance(value,dict) and 0<len(value)<=16384,'snapshot object/count')
    for name,p in value.items():
        relative_parts(name)
        reject(isinstance(p,dict) and set(p)=={'sha256','bytes'} and isinstance(p['sha256'],str) and SHA.fullmatch(p['sha256']) and type(p['bytes']) is int and 0<=p['bytes']<=16*1024*1024,'snapshot pin domain')
    reject(sum(p['bytes'] for p in value.values())<=256*1024*1024,'snapshot aggregate bound')
    reject({'Cargo.lock','rust-toolchain.toml'}<=set(value),'snapshot lock/toolchain required')

def construct(root,before,after,records,identity,binaries):
    validate_snapshot(before);validate_snapshot(after);reject(before==after,'tracked checkout changed')
    reject(isinstance(identity,dict) and set(identity)=={'commit','run_id','run_attempt','repository'} and isinstance(identity['commit'],str) and re.fullmatch(r'[0-9a-f]{40}',identity['commit']),'checkout commit domain')
    for key in ('run_id','run_attempt'):
        reject(isinstance(identity[key],str) and re.fullmatch(r'[1-9][0-9]{0,19}',identity[key]),'run identity domain')
    reject(isinstance(identity['repository'],str) and re.fullmatch(r'[A-Za-z0-9_.-]{1,100}/[A-Za-z0-9_.-]{1,100}',identity['repository']),'repository domain')
    reject(isinstance(records,list) and len(records)==len(REQUIRED),'operation count')
    operations={};used_paths=set();used_ids={native_identity(path(root,name)) for name in before};log_pins=[]
    for record in records:
        reject(isinstance(record,dict) and set(record)=={'name','argv','exit_code','stdout','stderr'},'operation fields')
        name=record['name'];reject(isinstance(name,str) and name in REQUIRED and name not in operations,'operation name')
        reject(record['argv']==REQUIRED[name] and type(record['exit_code']) is int and record['exit_code']==0,'exact successful command required')
        logs={}
        for key in ('stdout','stderr'):
            relative=record[key];relative_parts(relative);normalized=relative.casefold()
            reject(normalized not in used_paths,'duplicate log path');used_paths.add(normalized)
            file=path(root,relative);native_id=native_identity(file);reject(native_id not in used_ids,'log original identity alias');used_ids.add(native_id)
            value=pin(file,8*1024*1024);logs[key]=dict(path=relative,**value);log_pins.append((file,value,native_id))
        operations[name]=dict(argv=record['argv'],exit_code=0,logs=logs)
    # Rehash checkout immediately before binding binaries; supplied snapshots cannot
    # substitute modified current source. Still no atomic/transient-change guarantee.
    current={name:pin(path(root,name),16*1024*1024) for name in before}
    reject(current==before,'current source differs from before/after receipts')
    artifacts={};binary_pins=[]
    for name in BINS:
        file=path(root,binaries+'/'+name);native_id=native_identity(file);reject(native_id not in used_ids,'binary original identity alias');used_ids.add(native_id)
        value=pin(file,128*1024*1024)
        reject(native_identity(file)==native_id,'binary original identity changed while hashing')
        artifacts[name]=value;binary_pins.append((file,native_id))
    for file,value,native_id in log_pins:
        reject(native_identity(file)==native_id and pin(file,8*1024*1024)==value,'log changed while binding artifacts')
    reject({name:pin(path(root,name),16*1024*1024) for name in before}==before,'source changed while binding artifacts')
    for file,native_id in binary_pins:
        reject(native_identity(file)==native_id,'binary original identity changed after final checks')
    # Point-in-time sandwiches do not prove atomic or transient-change exclusion.
    return dict(proof_kind='ci-whole-build-format-binding',target='x86_64-pc-windows-msvc',identity=identity,source_before=before,source_after=after,operations=operations,artifacts=artifacts,lock=before['Cargo.lock'],toolchain=before['rust-toolchain.toml'],development_only=True,release_ready=False,signed_distribution=False,remote_execution_authenticated=False,private_job_cleanup_verified=False,native_gui_acceptance_verified=False,legal_coverage_complete=False)

def publish(output,value):
    output=Path(output).absolute()
    relative_parts('/'.join(output.parts[1:]));no_reparse(output)
    reject('..' not in output.parts and not output.exists() and output.parent.is_dir(),'fresh output/ancestor required')
    with output.open('x',encoding='utf-8') as f:json.dump(value,f,sort_keys=True,indent=2);f.write('\n')

def main():
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['snapshot','whole']);p.add_argument('--root',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    for key in ['tracked','before','after','operations','identity']:p.add_argument('--'+key)
    p.add_argument('--binaries',default='target/release');a=p.parse_args();root=a.root.absolute()
    no_reparse(root);reject(root.is_dir(),'root domain')
    if a.mode=='snapshot':
        reject(a.tracked is not None,'tracked list required');f=path(root,a.tracked)
        with f.open('rb') as stream:tracked=stream.read(2*1024*1024+1)
        value=snapshot(root,tracked)
    else:
        reject(all(getattr(a,k) is not None for k in ['before','after','operations','identity']),'whole inputs required')
        value=construct(root,*[read_json(path(root,getattr(a,k)),8*1024*1024) for k in ['before','after','operations','identity']],a.binaries)
    publish(a.output,value)
if __name__=='__main__':main()
