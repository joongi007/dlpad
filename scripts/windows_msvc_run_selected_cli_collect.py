"""Root-run selected GUI collection; native execution is root responsibility."""
import argparse,json,os,re,sys,hashlib
from pathlib import Path
sys.path.insert(0,'/mnt/d/project/dlpad/scripts')
import windows_current_workspace_probe as n
from windows_msvc_workspace_probe import require_current_whole
MAX=8*1024*1024

def plain(p):
 if not p.is_absolute()or '..'in p.parts or any(q.is_symlink()for q in [p,*p.parents]):raise ValueError('plain absolute path required')
 return p

def sha(path):
 path=plain(path);limit=128*1024*1024
 initial=path.stat()
 if not path.is_file()or initial.st_size>limit:raise ValueError('hash bound')
 h=hashlib.sha256();total=0
 with path.open('rb')as f:
  identity=os.fstat(f.fileno())
  if(identity.st_dev,identity.st_ino,identity.st_size)!=(initial.st_dev,initial.st_ino,initial.st_size):raise ValueError('hash original changed')
  while block:=f.read(65536):
   total+=len(block)
   if total>initial.st_size or total>limit:raise ValueError('hash grew')
   h.update(block)
  final=os.fstat(f.fileno())
 current=path.stat()
 if total!=initial.st_size or(final.st_dev,final.st_ino,final.st_size,final.st_mtime_ns)!=(initial.st_dev,initial.st_ino,initial.st_size,initial.st_mtime_ns)or(current.st_dev,current.st_ino,current.st_size,current.st_mtime_ns)!=(initial.st_dev,initial.st_ino,initial.st_size,initial.st_mtime_ns):raise ValueError('hash original postcheck')
 return h.hexdigest()

def bounded(p,maximum=MAX):
 p=plain(p)
 if not p.is_file()or p.stat().st_size>maximum:raise ValueError('bounded input')
 with p.open('rb')as f:b=f.read(maximum+1)
 if len(b)>maximum:raise ValueError('bounded growth')
 return b

def write(p,x):
 with p.open('x',encoding='utf8')as f:json.dump(x,f,indent=2);f.write('\n')

def copy(src,dst):
 initial=plain(src).stat().st_size
 if not src.is_file()or initial>128*1024*1024:raise ValueError('copy size bound')
 count=0
 with src.open('rb')as f,plain(dst).open('xb')as o:
  while b:=f.read(65536):
   count+=len(b)
   if count>initial:raise ValueError('copy source grew')
   o.write(b)
  o.flush();os.fsync(o.fileno())
 if count!=initial or src.stat().st_size!=initial or sha(src)!=sha(dst):raise ValueError('copy pin')

def winpath(s):
 if isinstance(s,str)and s.startswith('\\\\?\\'):s=s[4:]
 if not isinstance(s,str)or not s.lower().startswith('d:\\project\\dlpad\\'):raise ValueError('owned Windows path only')
 return plain(n.ROOT/s[len('D:\\project\\dlpad\\'):].replace('\\','/'))

def guard(op,raw):
 terminals=[b for b in raw.splitlines()if b.startswith(b'dlpad-owned-job-terminal ')]
 if op['passed']is not True or op['exit_code']!=0 or op['source_unchanged']is not True or len(terminals)!=1 or not all(x in terminals[0]for x in [b'natural-zero=True',b'final-zero=True',b'final-active=0',b'forced=False',b'job-closed=True',b'private-desktop-closed=True'])or b'dlpad-input-desktop-unchanged=True'not in raw:raise ValueError('native terminal guard')

def jsonrow(raw,key,value):
 rows=[json.loads(line)for line in raw.splitlines()if line.startswith(b'{')and len(line)<=65536];rows=[r for r in rows if r.get(key)==value]
 if len(rows)!=1:raise ValueError('exact bounded native JSON required')
 return rows[0]

def identity(native,expected):
 w=expected['Windows']
 return native['volume_serial']==w['volume_serial']and native['file_id_hex']==bytes(w['file_id']).hex()


def canonical(raw):
 value=json.loads(raw)
 if json.dumps(value,ensure_ascii=False,separators=(',',':')).encode('utf8')!=raw:raise ValueError('canonical typed JSON bytes')
 return value

def names(path,expected):
 entries=[]
 for f in path.iterdir():
  entries.append(f.name)
  if len(entries)>len(expected):raise ValueError('unexpected namespace overflow')
 if sorted(entries)!=sorted(expected):raise ValueError('exact installed namespace')



def terminal(op,raw,code):
 terminals=[line for line in raw.splitlines()if line.startswith(b'dlpad-owned-job-terminal ')]
 if (code==0 and op['passed']is not True)or op['exit_code']!=code or op['source_unchanged']is not True or len(terminals)!=1 or not all(v in terminals[0]for v in [b'natural-zero=True',b'final-zero=True',b'final-active=0',b'forced=False',b'job-closed=True',b'private-desktop-closed=True'])or b'dlpad-input-desktop-unchanged=True'not in raw:raise ValueError('original host lifecycle')
def main():
 import tempfile
 p=argparse.ArgumentParser();p.add_argument('--proof',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--gui-result',type=Path,required=True);p.add_argument('--natural-process-result',type=Path,required=True);a=p.parse_args()
 gnu_originals={str(p):sha(p)for p in [n.TOOLS/'target/release/dlpad-cpu.exe',n.TOOLS/'target/release/dlpad-local-install.exe']};before=n.snapshot();proof=plain(a.proof);pr=bounded(proof);whole=json.loads(pr);prior=plain(a.gui_result);gui=json.loads(bounded(prior));ready=gui['first']['ready']
 require_current_whole(whole,before)
 if gui.get('adapter_passed')is not True:raise ValueError('prior actual GUI proof required')
 if gui.get('source_before')!=before or gui.get('source_after')!=before or gui.get('source_unchanged')is not True or gui.get('experiment_target')!='x86_64-pc-windows-msvc':raise ValueError('prior GUI current source/target proof required')
 cpu=plain(n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release/dlpad-cpu.exe');cli=plain(n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release/dlpad-local-install.exe')
 for file,key in [(cpu,'dlpad-cpu.exe'),(cli,'dlpad-local-install.exe')]:
  if whole['artifacts'][key]!={'sha256':sha(file),'bytes':file.stat().st_size}:raise ValueError('whole artifact')
 descriptor=ready['selected_executable_descriptor'];installed=winpath(ready['selected_executable'])
 if descriptor['sha256']!=whole['artifacts']['dlpad-cpu.exe']['sha256']or descriptor['bytes']!=cpu.stat().st_size or sha(installed)!=descriptor['sha256']:raise ValueError('prior installed current CPU')
 reg=winpath(ready['registration_path']);receipt=winpath(ready['receipt_path']);active=winpath(ready['active_path']);owned=winpath(ready['owned_root'])
 original=dict(gui['original_preservation_pins']);namespaces={str(path):sorted(f.name for f in path.iterdir())for path in [reg,receipt,active]}
 source=winpath(ready['source_path']);original[str(source)]=ready['source_sha256']
 for path,digest in original.items():
  if sha(Path(path))!=digest:raise ValueError('prior original SHA')
 out=plain(a.output)
 if out.exists()or not out.is_relative_to(n.ROOT/'docs/validation/evidence'):raise ValueError('fresh output')
 out.mkdir(mode=0o700);inputs=out/'inputs';inputs.mkdir(mode=0o700)
 observer=n.ROOT/'scripts/windows_run_selected_cli_probe.cs';native=n.ROOT/'scripts/windows_active_selection_native_ids.cs';host=n.TOOLS/'windows-cli-host.exe'
 for src,name in [(proof,'whole-proof.json'),(prior,'prior-gui.json'),(cli,'cli.exe'),(cpu,'cpu.exe'),(observer,'observer.cs'),(native,'native-ids.cs'),(Path(__file__).absolute(),'collector.py')]:copy(src,inputs/name)
 pins={str(f):sha(f)for f in [proof,prior,cli,cpu,observer,native,host,*inputs.iterdir()]}
 if sha(inputs/'whole-proof.json')!=hashlib.sha256(pr).hexdigest():raise ValueError('decoded proof pin')
 data_root=Path(tempfile.mkdtemp(prefix='dlpad-launch-cli-owned-',dir=n.TOOLS/'tmp'));plain(data_root)
 for leaf in ['data','media','tmp']:(data_root/leaf).mkdir(mode=0o700)
 (data_root/'data/theme.json').write_bytes(b'"System"')
 (data_root/'foreign-sentinel').write_bytes(b'owned launch CLI sentinel');sentinelsha=sha(data_root/'foreign-sentinel')
 ops=[];rows=[];result={'schema':1,'status':'FAIL','source_before':before,'operations':ops,'pins':pins,'rows':rows,'owned_data_root':str(data_root),'root_review_pending':True,'pixels_verified':False,'sqlite_verified':False,'installer_complete':False};_,env=n.configure()
 try:
  def run(name,argv,seconds=100,code=0):
   op=n.run(name,[str(host),str(seconds),*argv],env,out,seconds);ops.append(op);raw=bounded(out/(name+'.log'));terminal(op,raw,code);return raw
  framework=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319';exe=out/'observer.exe';ids=out/'native-ids.exe'
  run('compile-observer',[framework+'\\csc.exe','/nologo','/reference:System.Drawing.dll','/reference:System.Web.Extensions.dll',*['/reference:'+framework+'\\WPF\\'+v+'.dll'for v in ['UIAutomationClient','UIAutomationTypes','WindowsBase']],'/out:'+n.win(exe),n.win(inputs/'observer.cs')],45)
  run('compile-native-ids',[framework+'\\csc.exe','/nologo','/reference:System.Web.Extensions.dll','/out:'+n.win(ids),n.win(inputs/'native-ids.cs')],45)
  for src,name in [(exe,'observer.exe'),(ids,'native-ids.exe')]:copy(src,inputs/name);pins[str(src)]=sha(src);pins[str(inputs/name)]=sha(inputs/name)
  write(out/'compiled-pins.json',pins)
  run('gui',[n.win(inputs/'observer.exe'),n.win(inputs/'cli.exe'),sha(inputs/'cli.exe'),n.win(installed),descriptor['sha256'],n.win(reg),n.win(receipt),n.win(active),n.win(data_root),n.win(out/'gui')],600)
  summary=json.loads(bounded(out/'gui/run-selected-cli-result.json',1048576));result['observer_summary']=summary
  if summary['status']!='REVIEW_REQUIRED':raise ValueError('observer scope')
  for case,reason,kind,code in [('first','Natural',None,0),('restart','Natural',None,0),('cancel','Cancelled','Interrupted',1)]:
   evidence=json.loads(bounded(out/'gui'/case/'result.json',262144));value=json.loads(bounded(out/'gui'/case/'cli-receipt.json',4096));rows.append({'case':case,'observer':evidence,'receipt':value})
   if value['status']!=('RAN_SELECTED_DEVELOPMENT_ONLY'if code==0 else'FAIL')or value['reason']!=reason or value['job_terminal_confirmed']is not True or value.get('kind')!=kind:raise ValueError('typed actual terminal')
   if value.get('phase')!='run-wait' or value.get('startup_timeout_seconds')!=2 or 'runtime_deadline_seconds'not in value or value['runtime_deadline_seconds']is not None or value.get('runtime_until_cancel_or_exit')is not True or value.get('permanent_detach')is not False:raise ValueError('startup-only runtime contract')
   if evidence.get('runtime_survived_startup_budget')is not True or evidence.get('startup_budget_seconds')!=2 or evidence.get('runtime_observed_elapsed_ms',0)<4000:raise ValueError('original runtime did not outlive startup budget')
   if any(value.get(k)is not False for k in ['publisher_trust','installer_complete','deletion_authority','upgrade_verified','trusted_provenance','capabilities_retained_after_exit']):raise ValueError('untruthful scope/capability receipt')
   if value['retained_file_caps']is not None or value['retained_file_cap_count_reported']is not False or value['selection_capabilities_held_before_drop']is not True:raise ValueError('unknown cap count policy')
   # Exact adapter cleanup fields must be agreed before root publication.
   if evidence['status']!='REVIEW_REQUIRED' or evidence['producer_natural_exit_code']!=code:raise ValueError('observer case lifecycle')
   for key in ['resume_ack_observed_by_original_hwnd','child_actual_image_id_sha_matched','child_parent_job_verified','parent_job_naturally_empty','parent_job_empty_after_cleanup','worker_drained','stdout_reader_drained','mta_desktop_restored','private_desktop_closed','input_desktop_unchanged']:
    if evidence.get(key)is not True:raise ValueError('actual resume/original Job/private lifecycle')
   if evidence.get('forced_cleanup')is not False or evidence.get('original_desktop_handle_retained_until_process_exit',False)is not False:raise ValueError('forced or retained cleanup')
  def command(r=reg,e=receipt,t=active):return ['run-selected','--target',n.win(r),'--receipt-target',n.win(e),'--active-target',n.win(t)]
  def refused(name,args,kind):
   raw=run(name,[n.win(inputs/'cli.exe'),*args],code=1);v=jsonrow(raw,'status','FAIL');rows.append({'case':name,'receipt':v})
   if v['kind']!=kind or v.get('mutation_may_have_occurred')is not False:raise ValueError('typed refusal/no mutation')
   if v.get('phase')=='preflight':
    if 'child_created'in v or v.get('retained_file_caps')!=0:raise ValueError('preflight source contract')
   elif v.get('phase')!='launch-inspect' or v.get('child_created')is not False:raise ValueError('launch-inspect no child')
  for name,args in [('missing',['run-selected','--target',n.win(reg)]),('unknown',command()+['--unknown','x']),('duplicate',command()+['--target',n.win(reg)]),('startup0',command()+['--startup-timeout-seconds','0']),('startup601',command()+['--startup-timeout-seconds','601']),('legacy-timeout-rejected',command()+['--timeout-seconds','2']),('duplicate-startup',command()+['--startup-timeout-seconds','2','--startup-timeout-seconds','2']),('legacy-extra',['inspect','--target',n.win(reg),'--active-target',n.win(active)])]:refused(name,args,'InvalidInput')
  priorfile=plain(a.natural_process_result);pins[str(priorfile)]=sha(priorfile);copy(priorfile,inputs/'prior-process.json');pins[str(inputs/'prior-process.json')]=sha(inputs/'prior-process.json')
  priorprocess=json.loads(bounded(priorfile));proot=winpath(priorprocess['producer_receipt']['owned_fixture_root'])
  for casevalue in priorprocess['producer_receipt']['cases']:
   for path,digest in casevalue['original_preservation_pins']:
    original[str(winpath(path))]=digest
  for leaf in ['foreign-sentinel','notice.txt']:original[str(proot/leaf)]=sha(proot/leaf)
  for casevalue in priorprocess['producer_receipt']['cases']:
   src=winpath(casevalue['source_path']) if 'source_path'in casevalue else None
   if src is not None:original[str(src)]=casevalue['source_sha256']
  for path,digest in original.items():
   if sha(Path(path))!=digest:raise ValueError('prior natural fixture original pin')
  raw=run('natural17',[n.win(inputs/'cli.exe'),*command(proot/'registered-natural',proot/'receipt-natural',proot/'active-natural')],code=1);v=jsonrow(raw,'status','FAIL');rows.append({'case':'natural17','receipt':v})
  if v.get('phase')!='run-wait'or v.get('startup_timeout_seconds')!=300 or v.get('runtime_deadline_seconds','missing')is not None or v.get('runtime_until_cancel_or_exit')is not True or v.get('permanent_detach')is not False or v['reason']!='Natural'or v['exit_code']!=17 or v['kind']!='Other'or v['job_terminal_confirmed']is not True:raise ValueError('Natural nonzero preserved')
  refused('other-valid-pair',command(proot/'registered-cancel',proot/'receipt-cancel',active),'InvalidData')
  # Foreign active is an independent fixture; never mutate earlier proof.
  foreign=data_root/'active-foreign';foreign.mkdir();(foreign/'foreign.txt').write_bytes(b'owned foreign');original[str(foreign/'foreign.txt')]=sha(foreign/'foreign.txt');namespaces[str(foreign)]=['foreign.txt'];refused('foreign',command(t=foreign),'PermissionDenied')
  observations={}
  for label,path in [('registered',reg),('receipt',receipt),('active',active)]:observations[label]=jsonrow(run('native-'+label,[n.win(inputs/'native-ids.exe'),n.win(path),label],45),'status','OBSERVED_NATIVE_IDS')
  ar=canonical(bounded(active/'active-selection.json',65536));rr=canonical(bounded(reg/'install-registration.json',65536));rc=canonical(bounded(receipt/'registration-receipt.json',65536))
  if rr['prepared']['target']!='x86_64-pc-windows-msvc':raise ValueError('MSVC installed fixture target')
  bindings=[(observations['active']['root'],ar['active_root_identity']),(observations['active']['entries']['active-selection.json'],ar['marker_identity']),(observations['registered']['root'],rr['root_identity']),(observations['receipt']['root'],rc['receipt_root_identity']),(observations['receipt']['entries']['registration-receipt.json'],rc['receipt_identity'])]
  for leaf,key in [('install-ready.json','ready_identity'),('install-registration.json','record_identity'),('install-registration-seal.json','seal_identity')]:bindings.append((observations['registered']['entries'][leaf],rr[key]))
  for entry in rr['prepared']['entries']:bindings.append((observations['registered']['entries'][entry['name']],entry['identity']))
  if len(bindings)!=10 or not all(identity(observed,expected)for observed,expected in bindings):raise ValueError('original ten native IDs')
  for label,path in [('registered',reg),('receipt',receipt),('active',active)]:
   for leaf,entry in observations[label]['entries'].items():
    if sha(path/leaf)!=entry['sha256']or(path/leaf).stat().st_size!=entry['bytes']:raise ValueError('native SHA/size')
  result['native_observations']=observations
  if any(sha(Path(p))!=h for p,h in original.items())or any(sorted(f.name for f in Path(p).iterdir())!=names for p,names in namespaces.items())or sha(data_root/'foreign-sentinel')!=sentinelsha:raise ValueError('original preservation')
  if n.snapshot()!=before or any(sha(Path(p))!=h for p,h in {**pins,**gnu_originals}.items()):raise ValueError('source/input pins')
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),original_preservation_pins=original,source_unchanged=True)
 except BaseException as error:result['failure_kind']=type(error).__name__;raise
 finally:
  for case in ['run-selected-cli-result.json','first/result.json','restart/result.json','cancel/result.json']:
   f=out/'gui'/case
   if f.exists():
    try:result.setdefault('retained_gui_raw',{})[case]=json.loads(bounded(f,1048576))
    except (OSError,ValueError):result.setdefault('raw_read_failures',[]).append(case)
  write(out/'result.json',result)
 print(json.dumps({'status':result['status'],'root_review_pending':True}))
if __name__=='__main__':main()
