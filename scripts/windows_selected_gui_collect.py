"""Root-run selected GUI collection; native execution is root responsibility."""
import argparse,json,os,re,sys,hashlib
from pathlib import Path
sys.path.insert(0,'/mnt/d/project/dlpad/scripts')
import windows_current_workspace_probe as n
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


def main():
 p=argparse.ArgumentParser();p.add_argument('--proof',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 proof=plain(a.proof);rawproof=bounded(proof);proofsha=hashlib.sha256(rawproof).hexdigest();whole=json.loads(rawproof);before=n.snapshot();out=plain(a.output)
 if whole.get('passed')is not True or whole.get('cargo_checks_complete')is not True or whole['source_final']!=before:raise ValueError('current whole proof required')
 cpu=plain(n.TOOLS/'target/release/dlpad-cpu.exe');artifact=whole['cpu_artifact']
 if artifact!={'sha256':sha(cpu),'bytes':cpu.stat().st_size}:raise ValueError('current CPU proof mismatch')
 if out.exists()or not out.is_relative_to(n.ROOT/'docs/validation/evidence'):raise ValueError('fresh evidence only')
 out.mkdir(mode=0o700);inputs=out/'inputs';inputs.mkdir(mode=0o700)
 fixture=n.ROOT/'crates/engine/examples/windows_selected_gui_fixture.rs';adapter=n.ROOT/'scripts/windows_selected_gui_probe.cs';host=n.TOOLS/'windows-cli-host.exe'
 for source,name in [(cpu,'cpu.exe'),(proof,'whole-proof.json'),(fixture,'fixture.rs'),(adapter,'adapter.cs'),(n.ROOT/'scripts/windows_active_selection_native_ids.cs','native-ids.cs'),(Path(__file__).absolute(),'collector.py')]:copy(source,inputs/name)
 pins={str(f):sha(f)for f in [cpu,proof,fixture,adapter,host,*inputs.iterdir()]}
 if pins[str(proof)]!=proofsha or pins[str(inputs/'whole-proof.json')]!=proofsha or sha(inputs/'cpu.exe')!=artifact['sha256']:raise ValueError('decoded proof archive mismatch')
 ops=[];result={'schema':1,'status':'FAIL','source_before':before,'pins':pins,'operations':ops,'root_review_pending':True,'actual_gui_pixels_verified':False,'sqlite_theme_verified':False,'installer_complete':False};_,env=n.configure()
 try:
  def invoke(name,args,seconds):
   op=n.run(name,[str(host),str(seconds),*args],env,out,seconds);ops.append(op);raw=bounded(out/(name+'.log'));guard(op,raw);return raw
  invoke('build-fixture',[n.win(n.TOOLS/'rust/bin/cargo.exe'),'build','-p','dlpad-engine','--example','windows_selected_gui_fixture','--offline','--locked'],180)
  framework=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319';compiled=out/'adapter.exe'
  invoke('build-adapter',[framework+'\\csc.exe','/nologo','/reference:System.Drawing.dll','/reference:System.Web.Extensions.dll',*['/reference:'+framework+'\\WPF\\'+v+'.dll'for v in ['UIAutomationClient','UIAutomationTypes','WindowsBase']],'/out:'+n.win(compiled),n.win(inputs/'adapter.cs')],45)
  native=out/'native-ids.exe'
  invoke('build-native-ids',[framework+'\\csc.exe','/nologo','/reference:System.Web.Extensions.dll','/out:'+n.win(native),n.win(inputs/'native-ids.cs')],45)
  original=n.TOOLS/'target/debug/examples/windows_selected_gui_fixture.exe'
  for source,name in [(original,'fixture.exe'),(compiled,'adapter.exe'),(native,'native-ids.exe')]:copy(source,inputs/name);pins[str(source)]=sha(source);pins[str(inputs/name)]=sha(inputs/name)
  write(out/'compiled-pins.json',pins)
  # Keep raw and adapter result on native failure; invoke checks original host terminal.
  invoke('gui',[n.win(inputs/'adapter.exe'),n.win(inputs/'fixture.exe'),sha(inputs/'fixture.exe'),n.win(inputs/'cpu.exe'),artifact['sha256'],n.win(out/'gui')],400)
  summary=json.loads(bounded(out/'gui/selected-gui-result.json',1048576));result['adapter_summary']=summary
  if summary.get('status')!='REVIEW_REQUIRED':raise ValueError('adapter scope status')
  preserved={};readyrows=[]
  for mode in ['first','restart']:
   run=json.loads(bounded(out/'gui'/mode/'result.json',262144));ready=json.loads(bounded(out/'gui'/mode/'ready.json',16384));terminal=json.loads(bounded(out/'gui'/mode/'terminal.json',16384))
   result[mode]={'result':run,'ready':ready,'terminal':terminal};readyrows.append(ready)
   if run.get('status')!='REVIEW_REQUIRED' or run.get('child_natural_exit_code')!=0 or run.get('producer_natural_exit_code')!=0:raise ValueError('natural original exits')
   for key in ['child_actual_image_id_sha_matched','mta_desktop_restored','parent_job_naturally_empty','parent_job_empty_after_cleanup','worker_drained','stdout_reader_drained','private_desktop_closed','input_desktop_unchanged']:
    if run.get(key)is not True:raise ValueError('adapter original lifecycle guard')
   if run.get('forced_cleanup')is not False or run.get('parent_job_active_before_cleanup')!=0 or run.get('original_desktop_handle_retained_until_process_exit',False)is not False:raise ValueError('forced/retained cleanup rejected')
   if ready.get('phase')!='selected-gui-ready' or terminal.get('phase')!='selected-gui-terminal' or terminal.get('status')!='PASS_SELECTED_GUI_PROCESS_ONLY' or terminal.get('job_terminal_confirmed')is not True or terminal.get('exit_code')!=0 or terminal.get('original_files_preserved')is not True:raise ValueError('producer terminal scope')
   if ready['child_pid']!=terminal['child_pid']or ready['child_creation_filetime']!=terminal['child_creation_filetime']or ready['child_creation_filetime']<=0:raise ValueError('original child creation binding')
   owned=winpath(ready['owned_root'])
   if not owned.is_relative_to(n.TOOLS/'tmp')or not owned.name.startswith('dlpad-selected-gui-owned-'):raise ValueError('owned temporary fixture')
   registered=winpath(ready['registration_path']);external=winpath(ready['receipt_path']);active=winpath(ready['active_path'])
   names(registered,['dlpad.exe','notice.txt','install-ready.json','install-registration.json','install-registration-seal.json']);names(external,['registration-receipt.json']);names(active,['active-selection.json'])
   expected={str(registered/leaf)for leaf in ['dlpad.exe','notice.txt','install-ready.json','install-registration.json','install-registration-seal.json']}|{str(external/'registration-receipt.json'),str(active/'active-selection.json')}
   actual=[str(winpath(path))for path,digest in terminal['original_preservation_pins']]
   if len(actual)!=7 or len(set(actual))!=7 or set(actual)!=expected or ready['original_preservation_pins']!=terminal['original_preservation_pins']:raise ValueError('exact unique original7 READY/TERMINAL')
   if mode=='first'and(ready['source_cursor_before']!=17 or terminal['source_cursor_after']!=17 or ready['source_sha256']!=artifact['sha256']or ready['source_bytes']!=artifact['bytes']or winpath(ready['source_path'])!=inputs/'cpu.exe'):raise ValueError('first original source/cursor17')
   ar=canonical(bounded(active/'active-selection.json',65536));rrraw=bounded(registered/'install-registration.json',65536);rr=canonical(rrraw);rcraw=bounded(external/'registration-receipt.json',65536);rc=canonical(rcraw);seal=canonical(bounded(registered/'install-registration-seal.json',65536))
   for dto,domain in [(ar,'dlpad-explicit-active-selection-v1'),(rr,'dlpad-app-registration-v1'),(rc,'dlpad-app-registration-receipt-v1'),(seal,'dlpad-app-registration-seal-v1')]:
    if dto['schema']!=1 or dto['domain']!=domain:raise ValueError('canonical domain')
   regdigest=hashlib.sha256(b'dlpad-app-registration-record-v1\0'+rrraw).hexdigest()
   if ar['registration_sha256']!=regdigest or seal['record_sha256']!=regdigest or ar['receipt_sha256']!=hashlib.sha256(rcraw).hexdigest()or rc['registration']!=rr or rc['seal']!=seal or ar['selected_executable']!=ready['selected_executable_descriptor']or hashlib.sha256(bounded(active/'active-selection.json',65536)).hexdigest()!=ready['active_sha256']:raise ValueError('canonical pair/descriptor digest')
   for active_key,reg_key in [('registration_root_identity','root_identity'),('registration_record_identity','record_identity'),('registration_seal_identity','seal_identity'),('registration_ready_identity','ready_identity')]:
    if ar[active_key]!=rr[reg_key]:raise ValueError('active registration native DTO')
   if ar['receipt_root_identity']!=rc['receipt_root_identity']or ar['receipt_identity']!=rc['receipt_identity']:raise ValueError('active receipt native DTO')
   descriptor=ready['selected_executable_descriptor']
   if descriptor['sha256']!=artifact['sha256']or descriptor['bytes']!=artifact['bytes']:raise ValueError('selected CPU descriptor')
   if mode=='first'and ready['source_identity'] is None:raise ValueError('missing original source native DTO')
   if mode=='restart'and(ready['owned_root_identity']!=readyrows[0]['owned_root_identity']or ready['selected_executable_descriptor']!=readyrows[0]['selected_executable_descriptor']or ready['original_preservation_pins']!=readyrows[0]['original_preservation_pins']):raise ValueError('restart original DTO/pins')
   for path,digest in terminal['original_preservation_pins']:
    f=winpath(path)
    if not f.is_relative_to(owned)or sha(f)!=digest:raise ValueError('original file SHA changed')
    preserved[str(f)]=digest
   for leaf,key in [('foreign-sentinel','sentinel_sha256'),('notice.txt','notice_sha256')]:
    f=owned/leaf
    if sha(f)!=terminal[key]:raise ValueError('synthetic external source preservation')
    preserved[str(f)]=terminal[key]
  observed={}
  for label,path,which in [('registered',registered,'registered'),('receipt',external,'receipt'),('active',active,'active')]:
   raw=invoke('native-'+label,[n.win(inputs/'native-ids.exe'),n.win(path),which],45);observed[label]=jsonrow(raw,'status','OBSERVED_NATIVE_IDS')
   for leaf,entry in observed[label]['entries'].items():
    if entry['sha256']!=sha(path/leaf)or entry['bytes']!=(path/leaf).stat().st_size:raise ValueError('native SHA/bytes')
  bindings=[(observed['active']['root'],ar['active_root_identity']),(observed['active']['entries']['active-selection.json'],ar['marker_identity']),(observed['registered']['root'],rr['root_identity']),(observed['receipt']['root'],rc['receipt_root_identity']),(observed['receipt']['entries']['registration-receipt.json'],rc['receipt_identity'])]
  for leaf,key in [('install-ready.json','ready_identity'),('install-registration.json','record_identity'),('install-registration-seal.json','seal_identity')]:bindings.append((observed['registered']['entries'][leaf],rr[key]))
  for entry in rr['prepared']['entries']:bindings.append((observed['registered']['entries'][entry['name']],entry['identity']))
  if len(bindings)!=10 or not all(identity(native,dto)for native,dto in bindings)or len({(native['volume_serial'],native['file_id_hex'])for native,dto in bindings})!=10:raise ValueError('ten distinct original native IDs')
  result['native_observations']=observed;result['original_native_identity_count']=10
  for key in ['owned_root','data_root','registration_path','receipt_path','active_path','selected_executable','active_sha256']:
   if readyrows[0][key]!=readyrows[1][key]:raise ValueError('restart explicit same selected paths')
  if any(sha(Path(k))!=v for k,v in pins.items())or n.snapshot()!=before:raise ValueError('source/input pins changed')
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),source_unchanged=True,adapter_passed=True,original_preservation_pins=preserved,actual_launch=True,sqlite_review_required=True)
 except BaseException as error:
  result['failure_kind']=type(error).__name__
  raise
 finally:
  for name in ['selected-gui-result.json','first/result.json','restart/result.json']:
   file=out/'gui'/name
   if file.exists():
    try:result.setdefault('retained_gui_raw',{})[name]=json.loads(bounded(file,1048576))
    except (ValueError,OSError):result.setdefault('retained_gui_read_failures',[]).append(name)
  write(out/'result.json',result)
 print(json.dumps({'status':result['status'],'root_review_pending':True}))
if __name__=='__main__':main()
