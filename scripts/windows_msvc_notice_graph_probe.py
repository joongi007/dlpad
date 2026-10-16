"""Root-run resolved MSVC notice graph collector; author does not execute native."""
import argparse,json,hashlib,os,re,sys
from pathlib import Path
sys.path.insert(0,'/mnt/d/project/dlpad/scripts')
import windows_current_workspace_probe as n
from windows_candidate_package import toml_inventory
from windows_msvc_workspace_probe import require_current_whole

def plain(p):
 if not p.is_absolute()or '..'in p.parts or any(q.is_symlink()for q in [p,*p.parents]):raise ValueError('plain absolute')
 return p

def read(p,cap):
 with plain(p).open('rb')as f:b=f.read(cap+1)
 if len(b)>cap:raise ValueError('input bound')
 return b

def sha(p):
 q=hashlib.sha256();total=0;initial=plain(p).stat()
 if not p.is_file()or initial.st_size>128*1024**2:raise ValueError('hash bound')
 with p.open('rb')as f:
  while b:=f.read(65536):
   total+=len(b)
   if total>initial.st_size:raise ValueError('source growth')
   q.update(b)
 after=p.stat()
 if total!=initial.st_size or(initial.st_dev,initial.st_ino,initial.st_size,initial.st_mtime_ns)!=(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns):raise ValueError('source changed')
 return q.hexdigest()

def save(p,v):
 text=json.dumps(v,indent=2)+'\n'
 if len(text.encode())>8*1024**2:raise ValueError('result bound')
 with p.open('x',encoding='utf8')as f:f.write(text)

def copy(p,d,cap):
 raw=read(p,cap)
 with d.open('xb')as f:f.write(raw);f.flush();os.fsync(f.fileno())
 if sha(p)!=sha(d):raise ValueError('archive changed')

def guard(o,raw):
 t=[v for v in raw.splitlines()if v.startswith(b'dlpad-owned-job-terminal ')]
 if o['passed']is not True or o['exit_code']!=0 or o['source_unchanged']is not True or len(t)!=1 or not all(x in t[0]for x in [b'natural-zero=True',b'final-zero=True',b'final-active=0',b'forced=False',b'job-closed=True',b'private-desktop-closed=True'])or b'dlpad-input-desktop-unchanged=True'not in raw:raise ValueError('original native host guard')

def keys(raw):
 if len(raw)>1024**2:raise ValueError('tree cap')
 rows=raw.decode('utf8').splitlines();found=set()
 if len(rows)>8192:raise ValueError('tree rows')
 for row in rows:
  if not row:continue
  if len(row)>4096:raise ValueError('row bound')
  m=re.fullmatch(r'([A-Za-z0-9_-]+) v([^\s]+)(?: \([^\r\n]*\))?(?: \(\*\))?',row)
  if not m:raise ValueError('tree row')
  found.add((m[1],m[2]))
  if len(found)>1024:raise ValueError('keys bound')
 if not found:raise ValueError('empty graph')
 return found

def main():
 a=argparse.ArgumentParser();a.add_argument('--proof',required=True,type=Path);a.add_argument('--configuration',required=True,type=Path);a.add_argument('--output',required=True,type=Path);args=a.parse_args();root=n.ROOT;proof=plain(args.proof);configpath=plain(args.configuration);pr=read(proof,8*1024**2);whole=json.loads(pr);configraw=read(configpath,65536);config=json.loads(configraw)
 if configraw!=read(proof.parent/'effective-configuration.json',65536):raise ValueError('whole effective config mismatch')
 before=n.snapshot();out=plain(args.output)
 require_current_whole(whole,before)
 target=n.TOOLS/'msvc-supply-20261006/target-msvc';artifacts={}
 for name,pin in whole['artifacts'].items():
  file=plain(target/'x86_64-pc-windows-msvc/release'/name)
  if pin!={'sha256':sha(file),'bytes':file.stat().st_size}:raise ValueError('MSVC artifact pin')
  artifacts[str(file)]=pin
 originals=whole['gnu_originals']
 for path,pin in originals.items():
  if pin!={'sha256':sha(Path(path)),'bytes':Path(path).stat().st_size}:raise ValueError('GNU originals')
 if out.exists()or not out.is_relative_to(root/'docs/validation/evidence'):raise ValueError('fresh evidence')
 out.mkdir();inputs=out/'inputs';inputs.mkdir();host=n.TOOLS/'windows-cli-host.exe';cargo=n.TOOLS/'rust/bin/cargo.exe'
 for p,name,cap in [(proof,'whole-proof.json',8*1024**2),(configpath,'configuration.json',65536),(root/'Cargo.lock','Cargo.lock',1024**2),(Path(__file__).absolute(),'collector.py',1024**2)]:copy(p,inputs/name,cap)
 pins={str(p):sha(p)for p in [proof,configpath,root/'Cargo.lock',root/'scripts/windows_current_workspace_probe.py',root/'scripts/windows_candidate_package.py',host,cargo,*inputs.iterdir()]}
 if sha(inputs/'whole-proof.json')!=hashlib.sha256(pr).hexdigest():raise ValueError('decoded proof pin')
 _,env=n.configure();removed=set(config['remove_inherited'])
 for k in removed:env.pop(k,None)
 forward=[x for x in env.get('WSLENV','').split(':')if x and x.split('/')[0]not in removed];values=dict(config['values']);values['CARGO_BUILD_JOBS']='2';env.update(values)
 for k in values:
  if k not in {v.split('/')[0]for v in forward}:forward.append(k)
 env['WSLENV']=':'.join(forward)
 cmds=[('app-tree',['tree','--locked','--offline','--target','x86_64-pc-windows-msvc','--features','cpu','-p','dlpad','--edges','normal,build','--prefix','none','--format','{p}','--charset','ascii'],1024**2),('engine-tree',['tree','--locked','--offline','--target','x86_64-pc-windows-msvc','-p','dlpad-engine','--edges','normal,build','--prefix','none','--format','{p}','--charset','ascii'],1024**2),('metadata',['metadata','--locked','--offline','--format-version','1','--filter-platform','x86_64-pc-windows-msvc','--features','cpu'],8*1024**2)]
 result={'schema':1,'status':'FAIL','source_before':before,'pins':pins,'operations':[],'artifacts':artifacts,'GNU_originals':originals,'exact_linked_code_proven':False,'notice_coverage_complete':False,'publication_provenance_verified':False,'legal_coverage_complete':False}
 try:
  parsed={}
  for name,argv,cap in cmds:
   op=n.run(name,[str(host),'180',n.win(cargo),*argv],env,out,180);result['operations'].append(op);raw=read(out/(name+'.log'),cap+65536);guard(op,raw)
   lines=[line for line in raw.splitlines()if not line.startswith((b'dlpad-owned-cli ',b'dlpad-owned-job-terminal ',b'dlpad-input-desktop-unchanged=',b'dlpad-msvc-rustc-wrapper '))]
   payload=b'\n'.join(lines)
   if len(payload)>cap:raise ValueError('payload cap')
   with(out/(name+'.payload')).open('xb')as f:f.write(payload)
   parsed[name]=json.loads(payload)if name=='metadata'else keys(payload)
  metadata=parsed['metadata'];packages=metadata['packages']
  if len(packages)>4096:raise ValueError('metadata package bound')
  lock=toml_inventory(read(inputs/'Cargo.lock',1024**2).decode(),lock=True);lockpkgs=lock['package'];mapped=[]
  union=parsed['app-tree']|parsed['engine-tree']
  for name,version in sorted(union):
   matches=[v for v in packages if v['name']==name and v['version']==version];lm=[v for v in lockpkgs if v['name']==name and v['version']==version]
   if len(matches)!=1 or len(lm)!=1:raise ValueError('ambiguous/missing exact lock package')
   v=matches[0];l=lm[0]
   if v['source']!=l.get('source'):raise ValueError('lock source mismatch')
   if v['source']and v['source'].startswith('registry+')and not re.fullmatch('[0-9a-f]{64}',l.get('checksum','')):raise ValueError('registry checksum missing')
   mapped.append({'name':name,'version':version,'metadata_id':v['id'],'source':v['source'],'lock_checksum':l.get('checksum'),'manifest_path':v['manifest_path'],'license':v.get('license'),'license_file':v.get('license_file'),'in_app_graph':(name,version)in parsed['app-tree'],'in_engine_graph':(name,version)in parsed['engine-tree']})
  if before!=n.snapshot()or any(sha(Path(p))!=v for p,v in pins.items())or any(sha(Path(p))!=v['sha256']for p,v in {**originals,**artifacts}.items()):raise ValueError('source/artifact original changed')
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),resolved_packages=mapped,app_count=len(parsed['app-tree']),engine_count=len(parsed['engine-tree']),union_count=len(union),normal_build_proc_macro_superset=True)
 except BaseException as error:result['failure_kind']=type(error).__name__;raise
 finally:save(out/'result.json',result)
if __name__=='__main__':main()
