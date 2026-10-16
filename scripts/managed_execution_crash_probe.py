#!/usr/bin/env python3
"""Owned synthetic Linux process crash test; no reader or automatic recovery proof."""
import argparse, hashlib, json, os, selectors, signal, subprocess, tempfile, time
from pathlib import Path

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def command(exe, mode, root):
    result=subprocess.run([str(exe),mode,str(root)],capture_output=True,timeout=10)
    if len(result.stdout)+len(result.stderr)>4096 or result.returncode!=0:
        raise RuntimeError('fixture command failed')
    return json.loads(result.stdout)

def identity(pid):
    text=Path(f'/proc/{pid}/stat').read_text()
    return {'pid':pid,'start_ticks':int(text[text.rfind(')')+2:].split()[19])}

def case(exe, mode):
    with tempfile.TemporaryDirectory(prefix='dlpad-managed-crash-owned-') as temp:
        root=Path(temp);os.chmod(root,0o700)
        sentinel=root/'foreign-sentinel';sentinel.write_bytes(b'preserve synthetic foreign')
        assert command(exe,'setup',root)['status']=='setup'
        installed=root/'managed'/'fixture'/'1'/'synthetic'/'tool'
        original={'source':sha(root/'source'),'sentinel':sha(sentinel),'installed':sha(installed)}
        child=subprocess.Popen([str(exe),'hold',str(root)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        began=time.monotonic();captured=b''
        try:
            with selectors.DefaultSelector() as selector:
                selector.register(child.stdout,selectors.EVENT_READ)
                deadline=time.monotonic()+10
                while b'\n' not in captured:
                    if time.monotonic()>=deadline:raise RuntimeError('ready deadline')
                    for key,_ in selector.select(min(.1,max(0,deadline-time.monotonic()))):
                        block=os.read(key.fileobj.fileno(),512)
                        if not block:raise RuntimeError('ready EOF')
                        captured+=block
                        if len(captured)>4096:raise RuntimeError('ready bound')
            assert json.loads(captured.splitlines()[0])['status']=='ready'
            owned=identity(child.pid)
            if mode=='crash':
                child.kill();child.stdin.close();child.stdin=None
            else:
                child.stdin.write(b'F');child.stdin.flush();child.stdin.close();child.stdin=None
            output,error=child.communicate(timeout=10)
            if len(output)+len(error)+len(captured)>4096:raise RuntimeError('output bound')
            assert child.returncode==(-signal.SIGKILL if mode=='crash' else 0)
            expected='pending' if mode=='crash' else 'clear'
            actual=command(exe,'probe-'+expected,root)
            assert actual['status']==expected
            assert sha(root/'source')==original['source'] and sha(sentinel)==original['sentinel']
            installed_after=sha(installed)
            assert installed_after==original['installed']
            files=[]
            for path in (root/'managed').rglob('*'):
                if path.is_file():
                    if len(files)>=32 or path.stat().st_size>1048576:raise RuntimeError('evidence bound')
                    files.append({'name':str(path.relative_to(root/'managed')),'bytes':path.stat().st_size,'sha256':sha(path)})
            return {'case':mode,'status':'PASS','original_process':owned,'exit_code':child.returncode,'mutation':actual['status'],'source_sha256':original['source'],'sentinel_sha256':original['sentinel'],'installed_sha256_before':original['installed'],'installed_sha256_after':installed_after,'files':files,'elapsed_seconds':time.monotonic()-began}
        finally:
            if child.poll() is None:
                child.kill();child.wait(timeout=5)
            for stream in (child.stdin,child.stdout,child.stderr):
                if stream:stream.close()

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--fixture',type=Path,required=True);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
    if os.name!='posix' or not Path('/proc/self/stat').is_file():raise SystemExit('Linux only')
    exe=args.fixture.resolve(strict=True);output=args.output
    if output.exists():raise SystemExit('Fresh output required')
    output.mkdir(parents=True,mode=0o700)
    result={'scope':'Linux owned process only; no guardian handoff or automatic pending recovery','fixture_sha256_before':sha(exe),'fixture_bytes':exe.stat().st_size,'status':'FAIL'}
    try:
        result['cases']=[case(exe,'normal'),case(exe,'crash')]
        result['fixture_sha256_after']=sha(exe)
        assert result['fixture_sha256_after']==result['fixture_sha256_before']
        result['status']='PASS'
    except Exception as error:
        result['error']=type(error).__name__+': '+str(error)
    finally:
        (output/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'status':result['status'],'cases':len(result.get('cases',[]))}))
    return 0 if result['status']=='PASS' else 2
if __name__=='__main__':raise SystemExit(main())
