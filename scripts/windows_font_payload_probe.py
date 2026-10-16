"""Bounded exact stored-font proof. No runtime-use or legal-completeness claims."""
import argparse,hashlib,io,json,re,struct,sys,tarfile,zipfile
from pathlib import Path
PINS = {
  "lib": "da81ac92c840666bab7f8fcc223c8e474b642db95b14ee5e5b7e01a1fa9c43b6",
  "crate": "5c4fbe202b6578d3d56428fa185cdf114a05e49da05f477b3c7f0fbb221f1862",
  "assets": {
    "emoji-icon-font-mit-license.txt": "b9d2c1d909aa149996fd4c91dcb92b2362a04431640c1d200959da94caf8cde1",
    "emoji-icon-font.ttf": "3a706840d2f888b83c184f7d41900dbaa0151a806c4d130cc88539d53fedef0a",
    "Hack-Regular.ttf": "15f55cc0c85a2988d2b4b3a8cdb5d77fdfbaf319e1bb5309d725db9818fb7125",
    "Hack-Regular.txt": "47c0cccbeec7e8614548cc485588b28149e7874188df5f41b36efebcee285c87",
    "NotoEmoji-Regular.ttf": "415dc6290378574135b64c808dc640c1df7531973290c4970c51fdeb849cb0c5",
    "OFL.txt": "6a73f9541c2de74158c0e7cf6b0a58ef774f5a780bf191f2d7ec9cc53efe2bf2",
    "Ubuntu-Light.ttf": "80307b8da7649aa4ee4d484b232140e3ce1ec0ca093073d3c53c8f5a5ced7a70",
    "UFL.txt": "2f0015108d68627bd788d313f529c21ff4da2c2c42a5e1f3883acc83480f9002"
  }
}
MAP=[('Hack-Regular.ttf','Hack-Regular.txt'),('NotoEmoji-Regular.ttf','OFL.txt'),('Ubuntu-Light.ttf','UFL.txt'),('emoji-icon-font.ttf','emoji-icon-font-mit-license.txt')]
KEY='epaint_default_fonts-0.33.3'
def sha(b):return hashlib.sha256(b).hexdigest()
def path(p):
 p=Path(p)
 if not p.is_absolute() or '..' in p.parts or any(x.is_symlink() for x in [p,*p.parents]):raise ValueError('absolute no-symlink path required')
 return p

def read(p,cap):
 p=path(p)
 if not p.is_file():raise ValueError('regular file required')
 with p.open('rb') as f:b=f.read(cap+1)
 if len(b)>cap:raise ValueError('read bound')
 return b

def sections(b):
 def u(fmt,off):
  if off<0 or off+struct.calcsize(fmt)>len(b):raise ValueError('PE truncated')
  return struct.unpack_from(fmt,b,off)[0]
 if len(b)>32*1024*1024 or b[:2]!=b'MZ':raise ValueError('PE size/signature')
 off=u('<I',60)
 if b[off:off+4]!=b'PE\0\0' or u('<H',off+4)!=0x8664 or u('<H',off+24)!=0x20b:raise ValueError('AMD64 PE32+ required')
 count=u('<H',off+6);opt=u('<H',off+20)
 if not 1<=count<=96 or not 112<=opt<=4096:raise ValueError('bounded section header')
 header_end=off+24+opt+count*40
 if header_end>len(b):raise ValueError('section truncated')
 ranges=[];rdata=[]
 for i in range(count):
  at=off+24+opt+i*40
  if at+40>len(b):raise ValueError('section truncated')
  size=u('<I',at+16);start=u('<I',at+20);end=start+size
  if end>len(b) or (size and start<header_end) or (size and any(start<e and s<end for s,e in ranges)):raise ValueError('section overlap/bound')
  if size:ranges.append((start,end))
  if b[at:at+8].rstrip(b'\0')==b'.rdata':rdata.append((start,end))
 if len(rdata)!=1:raise ValueError('exact .rdata required')
 return rdata[0]

def assets(b,fonts):
 start,end=sections(b);rows=[]
 for name,data in fonts.items():
  at=b.find(data);second=b.find(data,at+1) if at>=0 else -1
  if at<start or at+len(data)>end or second!=-1:raise ValueError('font full payload missing/duplicate/outside .rdata')
  rows.append({'font':name,'bytes':len(data),'sha256':sha(data),'pe_file_offset':at})
 return rows

def source_files(source):
 code=read(source/'src/lib.rs',65536)
 if sha(code)!=PINS['lib']:raise ValueError('lib source changed')
 declared=re.findall(rb'include_bytes!\("../fonts/([^"\n]+)"\)',code)
 if sorted(n.decode() for n in declared)!=sorted(f for f,_ in MAP):raise ValueError('unknown font declaration')
 result={'src/lib.rs':code}
 for font,notice in MAP:
  for name in [font,notice]:
   b=read(source/'fonts'/name,1024*1024)
   if sha(b)!=PINS['assets'][name]:raise ValueError('font/notice changed')
   result['fonts/'+name]=b
 if sum(len(result['fonts/'+f]) for f,_ in MAP)>4*1024*1024:raise ValueError('asset aggregate bound')
 return result

def original_crate(archive,files,checksum):
 if checksum!=PINS['crate'] or sha(read(archive,4*1024*1024))!=checksum:raise ValueError('original crate checksum')
 found=set();count=total=0
 with tarfile.open(archive,'r|gz') as t:
  for member in t:
   count+=1;total+=member.size
   if count>64 or total>8*1024*1024 or member.size>1024*1024 or not member.isfile() or not member.name.startswith(KEY+'/') or '..' in member.name.split('/'):raise ValueError('original crate bounds')
   name=member.name[len(KEY)+1:]
   if name in files:
    if name in found:raise ValueError('duplicate selected crate member')
    f=t.extractfile(member);b=f.read(1024*1024+1)
    if b!=files[name]:raise ValueError('original selected member mismatch')
    found.add(name)
 if found!=set(files) or sha(read(archive,4*1024*1024))!=checksum:raise ValueError('original selected member missing/changed')

def bounded_zip(zip_path):
 # Validate the entire central-directory record count before ZipFile allocates
 # its ZipInfo list; EOCD's declared count alone can be forged.
 raw=read(zip_path,32*1024*1024)
 at=raw.rfind(b'PK\x05\x06',max(0,len(raw)-65557))
 if at<0 or at+22>len(raw):raise ValueError('ZIP EOCD absent/truncated')
 _,disk,cd_disk,disk_count,count,size,offset,comment=struct.unpack_from('<4s4H2IH',raw,at)
 if disk or cd_disk or disk_count!=count or not 1<=count<=4096 or offset+size!=at or at+22+comment!=len(raw):raise ValueError('ZIP central directory bound')
 pos=offset;seen=0
 while pos<at:
  if pos+46>at or raw[pos:pos+4]!=b'PK\x01\x02':raise ValueError('ZIP central record malformed')
  names,extra,note=struct.unpack_from('<3H',raw,pos+28)
  length=46+names+extra+note;seen+=1
  if seen>4096 or not 1<=names<=4096 or pos+length>at:raise ValueError('ZIP central record bound')
  pos+=length
 if pos!=at or seen!=count:raise ValueError('ZIP central count mismatch')
 return zipfile.ZipFile(io.BytesIO(raw))

def zip_notices(zip_path,files):
 with bounded_zip(zip_path) as z:
  names=z.namelist()
  if len(names)>4096 or len(names)!=len(set(names)):raise ValueError('ZIP inventory bound')
  for _,notice in MAP:
   name='licenses/'+KEY+'/'+notice;i=z.getinfo(name)
   if i.file_size>1024*1024 or z.read(i)!=files['fonts/'+notice]:raise ValueError('ZIP original notice mismatch')

def verify_zip_cpu(zip_path,b):
 with bounded_zip(zip_path) as z:
  info=z.getinfo('candidate-manifest.json')
  if info.file_size>1024*1024:raise ValueError('candidate manifest bound')
  manifest=json.loads(z.read(info))
  if manifest.get('binary_sha256')!=sha(b) or manifest.get('binary_bytes')!=len(b):raise ValueError('ZIP candidate CPU mismatch')
  info=z.getinfo('dlpad-cpu.exe')
  if info.file_size!=len(b):raise ValueError('ZIP binary size mismatch')
  h=hashlib.sha256();total=0
  with z.open(info) as f:
   while True:
    data=f.read(65536)
    if not data:break
    total+=len(data)
    if total>32*1024*1024:raise ValueError('ZIP binary bound')
    h.update(data)
  if total!=len(b) or h.hexdigest()!=sha(b):raise ValueError('ZIP binary digest mismatch')

def main():
 a=argparse.ArgumentParser();a.add_argument('--project',type=Path,required=True);a.add_argument('--exe',type=Path,required=True);a.add_argument('--proof',type=Path,required=True);a.add_argument('--zip',type=Path,required=True);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 root=path(args.project);out=path(args.output);zip_path=path(args.zip)
 zip_sha=sha(read(zip_path,32*1024*1024));self_sha=sha(read(Path(__file__).resolve(),1024*1024))
 if not out.is_relative_to(root/'docs/validation/evidence') or out.exists():raise ValueError('fresh project evidence required')
 sys.path.insert(0,str(root/'scripts'));import windows_candidate_package as c
 if c.ROOT.resolve()!=root.resolve():raise ValueError('project helper mismatch')
 proof=c.checks(path(args.proof));b=read(args.exe,32*1024*1024)
 verify_zip_cpu(zip_path,b)
 if proof['cpu_artifact']!={'sha256':sha(b),'bytes':len(b)}:raise ValueError('actual proof CPU mismatch')
 sources=list((root/'.local-tools/cargo/registry/src').glob('*/'+KEY))
 if len(sources)!=1:raise ValueError('exact one original source required')
 source=sources[0];files=source_files(source)
 lock=c.toml_inventory(read(root/'Cargo.lock',8*1024*1024).decode(),True);record=next(r for r in lock['package'] if r['name']=='epaint_default_fonts' and r['version']=='0.33.3')
 archive=source.parent.parent.parent/'cache'/source.parent.name/(KEY+'.crate');original_crate(archive,files,record.get('checksum'));rows=assets(b,{f:files['fonts/'+f] for f,_ in MAP});zip_notices(path(args.zip),files)
 if sha(read(args.exe,32*1024*1024))!=sha(b):raise ValueError('CPU changed')
 if source_files(source)!=files:raise ValueError('original fonts changed after inspection')
 original_crate(archive,files,record.get('checksum'));zip_notices(path(args.zip),files)
 c.checks(path(args.proof))
 if zip_sha!=sha(read(zip_path,32*1024*1024)) or self_sha!=sha(read(Path(__file__).resolve(),1024*1024)):raise ValueError('ZIP or verifier changed')
 out.mkdir(exist_ok=False)
 result={'passed':True,'zip_sha256':zip_sha,'verifier_sha256':self_sha,'cpu_sha256':sha(b),'proof':proof,'whole_lock_sha256':sha(read(root/'Cargo.lock',8*1024*1024)),'assets':rows,'source_pins':PINS,'exact_stored_payload_verified':True,'runtime_use_verified':False,'legal_coverage_complete':False,'stack_adopted':False,'release_ready':False}
 (out/'result.json').write_text(json.dumps(result,indent=2)+'\n')
if __name__=='__main__':main()
