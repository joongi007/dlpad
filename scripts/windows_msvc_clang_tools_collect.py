"""Root-run project-only clang CL aliases and no-SDK COFF smoke."""
import argparse, hashlib, json, os, re, stat, struct, sys
from pathlib import Path
sys.path.insert(0, '/mnt/d/project/dlpad/scripts')
import windows_current_workspace_probe as n
n.ROOT = Path('/mnt/d/project/dlpad')
n.TOOLS = n.ROOT / '.local-tools/windows'
LIMIT = 128 * 1024 * 1024

def plain(p):
    if not p.is_absolute() or '..' in p.parts or any(q.is_symlink() for q in (p,*p.parents)):
        raise ValueError('plain absolute path required')
    return p

def digest(p, limit=LIMIT):
    plain(p); before=p.stat()
    if not stat.S_ISREG(before.st_mode) or before.st_size>limit: raise ValueError('regular bounded file required')
    h=hashlib.sha256(); count=0
    with p.open('rb') as f:
        original=os.fstat(f.fileno())
        if (original.st_dev,original.st_ino)!=(before.st_dev,before.st_ino): raise ValueError('file identity changed')
        while b:=f.read(65536):
            count+=len(b)
            if count>before.st_size or count>limit: raise ValueError('file grew')
            h.update(b)
        after=os.fstat(f.fileno())
    current=p.stat()
    key=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns)
    if count!=before.st_size or key(before)!=key(after) or key(before)!=key(current): raise ValueError('file changed')
    return {'sha256':h.hexdigest(),'bytes':count}

def copy(src,dst):
    expected=digest(src); plain(dst)
    count=0
    with src.open('rb') as f,dst.open('xb') as out:
        while b:=f.read(65536):
            count+=len(b)
            if count>expected['bytes']: raise ValueError('copy grew')
            out.write(b)
        out.flush();os.fsync(out.fileno())
    if digest(src)!=expected or digest(dst)!=expected: raise ValueError('copy pin mismatch')
    return expected

def save(path,value):
    with path.open('x',encoding='utf8') as f: json.dump(value,f,indent=2);f.write('\n')

def validate_pins(rows):
    if not isinstance(rows,list) or len(rows)!=13: raise ValueError('exact thirteen original pins required')
    seen=set()
    for row in rows:
        s=row['path'];p=Path(s)
        if p.is_absolute() or '..' in p.parts or not s.startswith('.local-tools/windows/llvm-mingw/') or s in seen: raise ValueError('original pin path')
        seen.add(s)
        if digest(n.ROOT/p)!={'sha256':row['sha256'],'bytes':row['bytes']}: raise ValueError('original pin mismatch')
    needed={'.local-tools/windows/llvm-mingw/bin/clang-23.exe','.local-tools/windows/llvm-mingw/bin/ld.lld.exe','.local-tools/windows/llvm-mingw/lib/clang/23/include/stddef.h'}
    if not needed<=seen: raise ValueError('missing original tools/resource')

def native_guard(op,log):
    lines=[s for s in log.splitlines() if s.startswith(b'dlpad-owned-job-terminal ')]
    if not op['passed'] or op['exit_code']!=0 or not op['source_unchanged'] or len(lines)!=1 or not all(v in lines[0] for v in (b'natural-zero=True',b'final-zero=True',b'final-active=0',b'forced=False',b'job-closed=True',b'private-desktop-closed=True')): raise ValueError('original native terminal guard')
    if b'dlpad-input-desktop-unchanged=True' not in log: raise ValueError('input desktop guard')

def coff(path):
    pin=digest(path,1024*1024);raw=path.read_bytes()
    if len(raw)<20: raise ValueError('short COFF')
    machine,sections,stamp,symbols,count,optional,flags=struct.unpack_from('<HHIIIHH',raw)
    if machine!=0x8664 or not 1<=sections<=32 or optional!=0 or 20+40*sections>len(raw): raise ValueError('AMD64 COFF header')
    for i in range(sections):
        s=20+40*i;size,offset,relocs=struct.unpack_from('<III',raw,s+16);nr=struct.unpack_from('<H',raw,s+32)[0]
        if size and (offset<20+40*sections or offset+size>len(raw)): raise ValueError('COFF section bounds')
        if nr and (relocs<20+40*sections or relocs+nr*10>len(raw)): raise ValueError('COFF relocation bounds')
    if count and (symbols+count*18+4>len(raw)): raise ValueError('COFF symbol bounds')
    if digest(path,1024*1024)!=pin: raise ValueError('COFF changed')
    return dict(pin,machine=machine,sections=sections,object_only=True)

def main():
    p=argparse.ArgumentParser();p.add_argument('--pins',type=Path,required=True);p.add_argument('--base',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    for x,parent in ((a.base,n.TOOLS),(a.output,n.ROOT/'docs/validation/evidence')):
        plain(x)
        if not x.is_relative_to(parent) or x==parent or x.exists(): raise ValueError('fresh project namespace')
    rows=json.loads(plain(a.pins).read_text());validate_pins(rows);source=n.snapshot();a.output.mkdir(mode=0o700);a.base.mkdir(mode=0o700);tools=a.base/'clang-tools';tools.mkdir(mode=0o700)
    result={'passed':False,'scope':'CL-driver object smoke only','sdk_verified':False,'crt_verified':False,'msvc_application_link_verified':False,'source_before':source,'original_pins':rows,'operations':{}}
    try:
        copy(Path(__file__).absolute(),a.output/'collector.py');copy(a.pins,a.output/'original-pins.json')
        original=n.TOOLS/'llvm-mingw';aliases={}
        for src,name in [('clang-23.exe','clang-cl.exe'),('ld.lld.exe','lld-link.exe')]:aliases[name]=copy(original/'bin'/src,tools/name)
        # Preserve actual notice bytes and relative source path; no completeness claim.
        notice=original/'LICENSE.TXT';notice_dir=tools/'notices/llvm-mingw';notice_dir.mkdir(parents=True,mode=0o700)
        result['notice_inventory']=[{'source':str(notice.relative_to(n.ROOT)),'copy':'notices/llvm-mingw/LICENSE.TXT',**copy(notice,notice_dir/'LICENSE.TXT')}]
        result['notice_coverage_complete']=False;result['aliases']=aliases
        c=a.base/'smoke.c'
        with c.open('xb') as f:f.write(b'int dlpad_coff_smoke(void) { return 17; }\n')
        obj=a.base/'smoke.obj';values,env=n.configure();env['DLPAD_NATIVE_TOOL_PATH']=n.win(original/'bin');n.MAX_LOG=65536
        commands={'clang-version':[n.win(tools/'clang-cl.exe'),'--version'],'lld-help':[n.win(tools/'lld-link.exe'),'/?'],'coff-object':[n.win(tools/'clang-cl.exe'),'--driver-mode=cl','--target=x86_64-pc-windows-msvc','-resource-dir',n.win(original/'lib/clang/23'),'/nologo','/c','/TC','/Zl','/GS-','/Fo'+n.win(obj),n.win(c)]}
        for name,args in commands.items():
            op=n.run(name,[str(n.TOOLS/'windows-cli-host.exe'),'30',*args],env,a.output,30);result['operations'][name]=op
            native_guard(op,(a.output/(name+'.log')).read_bytes())
        result['coff']=coff(obj);validate_pins(rows)
        for name,pin in aliases.items():
            if digest(tools/name)!=pin:raise ValueError('alias changed')
        result['source_after']=n.snapshot()
        if result['source_after']!=source:raise ValueError('source changed')
        result['passed']=True
    except Exception as e:result['error']=type(e).__name__+': '+str(e)
    save(a.output/'result.json',result);return 0 if result['passed'] else 1
if __name__=='__main__':raise SystemExit(main())
