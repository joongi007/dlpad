"""Bounded binding of an already-pinned font proof; no legal/release approval."""
import argparse, hashlib, json, re, stat, struct, zipfile, io
from pathlib import Path
HEX = re.compile(r'[0-9a-f]{64}\Z')
FONTS = {
'Hack-Regular.ttf':(309408,'15f55cc0c85a2988d2b4b3a8cdb5d77fdfbaf319e1bb5309d725db9818fb7125','Hack-Regular.txt','47c0cccbeec7e8614548cc485588b28149e7874188df5f41b36efebcee285c87'),
'NotoEmoji-Regular.ttf':(418804,'415dc6290378574135b64c808dc640c1df7531973290c4970c51fdeb849cb0c5','OFL.txt','6a73f9541c2de74158c0e7cf6b0a58ef774f5a780bf191f2d7ec9cc53efe2bf2'),
'Ubuntu-Light.ttf':(361676,'80307b8da7649aa4ee4d484b232140e3ce1ec0ca093073d3c53c8f5a5ced7a70','UFL.txt','2f0015108d68627bd788d313f529c21ff4da2c2c42a5e1f3883acc83480f9002'),
'emoji-icon-font.ttf':(317864,'3a706840d2f888b83c184f7d41900dbaa0151a806c4d130cc88539d53fedef0a','emoji-icon-font-mit-license.txt','b9d2c1d909aa149996fd4c91dcb92b2362a04431640c1d200959da94caf8cde1')}
CRATE='5c4fbe202b6578d3d56428fa185cdf114a05e49da05f477b3c7f0fbb221f1862'
LIB='da81ac92c840666bab7f8fcc223c8e474b642db95b14ee5e5b7e01a1fa9c43b6'
FALSE=('runtime_use_verified','legal_coverage_complete','stack_adopted','release_ready')
def require(ok,why):
 if not ok: raise ValueError(why)
def pin(v):
 require(isinstance(v,str) and HEX.fullmatch(v) is not None,'invalid digest type/format');return v
def plain(p):
 p=Path(p);require(p.is_absolute() and '..' not in p.parts,'absolute path required')
 require(not any(x.is_symlink() for x in [p,*p.parents]),'symlink path rejected');return p
def bounded(p,cap):
 p=plain(p);s=p.stat();require(stat.S_ISREG(s.st_mode) and s.st_size<=cap,'regular bounded file required')
 with p.open('rb') as f:b=f.read(cap+1)
 require(len(b)<=cap,'read bound');return b
def digest(p,cap):
 p=plain(p);s=p.stat();require(stat.S_ISREG(s.st_mode) and s.st_size<=cap,'hash input bound')
 h=hashlib.sha256();n=0
 with p.open('rb') as f:
  while True:
   b=f.read(65536)
   if not b:break
   n+=len(b);require(n<=cap,'stream bound');h.update(b)
 require(n==s.st_size,'file size changed');return h.hexdigest(),n
# Duplicate JSON members cannot silently replace original pins.
def decode(raw):
 def pairs(xs):
  d={}
  for k,v in xs:require(k not in d,'duplicate JSON key');d[k]=v
  return d
 return json.loads(raw,object_pairs_hook=pairs)
def obj(p,cap):
 return decode(bounded(p,cap))
def validate_result(r,cpu,zip_sha,proof_sha,verifier_sha,exe_bytes):
 for v in [cpu,zip_sha,proof_sha,verifier_sha]:pin(v)
 require(isinstance(r,dict),'font result object')
 require(r.get('passed') is True and r.get('exact_stored_payload_verified') is True,'original verification absent')
 require(all(r.get(k) is False for k in FALSE),'unverified claims must remain false')
 require(r.get('cpu_sha256')==cpu and r.get('zip_sha256')==zip_sha and r.get('verifier_sha256')==verifier_sha,'historical CPU/ZIP/verifier mismatch')
 pr=r.get('proof');require(isinstance(pr,dict),'proof object')
 require(pr.get('sha256')==proof_sha and pr.get('cpu_artifact')=={'sha256':cpu,'bytes':exe_bytes},'workspace proof mismatch')
 require(pr.get('operations')==['fmt','clippy','test','release'] and pr.get('proof_format')=='workspace-actual-operations-v1','canonical default proof required')
 require(type(pr.get('source_files')) is int and 1<=pr['source_files']<=4096,'source inventory bound')
 expected={'lib':LIB,'crate':CRATE,'assets':{}}
 for f,(_,sha,notice,ns) in FONTS.items():expected['assets'][f]=sha;expected['assets'][notice]=ns
 require(r.get('source_pins')==expected,'source font/grant mapping mismatch');pin(r.get('whole_lock_sha256'))
 rows=r.get('assets');require(isinstance(rows,list) and len(rows)==4,'exact four assets required');seen=set();ranges=[]
 for a in rows:
  require(isinstance(a,dict) and set(a)=={'font','bytes','sha256','pe_file_offset'},'asset shape')
  f=a['font'];require(isinstance(f,str) and f in FONTS and f not in seen,'asset membership');seen.add(f)
  n,sha,_,_=FONTS[f];off=a['pe_file_offset'];require(a['bytes']==n and type(a['bytes']) is int and a['sha256']==sha,'asset digest/size')
  require(type(off) is int and 0<=off and off+n<=exe_bytes,'asset offset bound');ranges.append((off,off+n))
 ranges.sort();require(all(a[1]<=b[0] for a,b in zip(ranges,ranges[1:])),'overlap assets')
 return {'schema':1,'cpu_sha256':cpu,'zip_sha256':zip_sha,'workspace_proof_sha256':proof_sha,'verifier_sha256':verifier_sha,'stored_payload_verified':True,'assets':rows,'source_pins':expected,**{k:False for k in FALSE}}
def zip_read(raw):
 at=raw.rfind(b'PK\x05\x06',max(0,len(raw)-65557));require(at>=0 and at+22<=len(raw),'EOCD')
 _,disk,cd_disk,disk_count,count,size,off,comment=struct.unpack_from('<4s4H2IH',raw,at)
 require(not disk and not cd_disk and 1<=count<=4096 and disk_count==count and off+size==at and at+22+comment==len(raw),'central bound')
 pos=off;seen=0
 while pos<at:
  require(pos+46<=at and raw[pos:pos+4]==b'PK\x01\x02','central entry')
  ns,ex,cm=struct.unpack_from('<3H',raw,pos+28);seen+=1;require(seen<=4096 and 1<=ns<=4096,'central count/name bound');pos+=46+ns+ex+cm;require(pos<=at,'central extent')
 require(pos==at and seen==count,'central count mismatch');return zipfile.ZipFile(io.BytesIO(raw))
def bind(args):
 for name in ['cpu_sha','zip_sha','proof_sha','font_result_sha','verifier_sha']:pin(getattr(args,name))
 cpu,n=digest(args.exe,32*1024*1024);require(cpu==args.cpu_sha,'supplied exe digest')
 font_raw=bounded(args.font_result,1024*1024);require(hashlib.sha256(font_raw).hexdigest()==args.font_result_sha,'font result digest')
 proof_raw=bounded(args.proof,8*1024*1024);require(hashlib.sha256(proof_raw).hexdigest()==args.proof_sha,'original proof digest')
 require(digest(args.verifier,1024*1024)[0]==args.verifier_sha,'original verifier source digest')
 r=decode(font_raw);binding=validate_result(r,cpu,args.zip_sha,args.proof_sha,args.verifier_sha,n)
 proof=decode(proof_raw);require(proof.get('passed') is True and proof.get('source_unchanged') is True,'workspace source proof')
 require(proof.get('source_baseline')==proof.get('source_final') and isinstance(proof.get('source_final'),dict) and len(proof['source_final'])==r['proof']['source_files'],'source set agreement')
 require(proof.get('cpu_artifact')==r['proof']['cpu_artifact'],'workspace artifact')
 ops=proof.get('actual_operations');require(isinstance(ops,dict) and set(ops)=={'fmt','clippy','test','release'},'exact four operations')
 for name,operation in ops.items():
  require(isinstance(operation,dict) and operation.get('exit_code')==0 and operation.get('passed') is True and operation.get('source_before')==proof['source_final']==operation.get('source_after'),'actual operation source/exit')
  argv=operation.get('argv');require(isinstance(argv,list) and all(isinstance(x,str) for x in argv),'operation argv')
  tails={'fmt':['fmt','--all','--','--check'],'clippy':['clippy','--workspace','--all-targets','--locked','--features','cpu','--','-D','warnings'],'test':['test','--workspace','--locked','--features','cpu'],'release':['build','--workspace','--release','--locked','--features','cpu']}
  require(len(argv)>=4 and argv[3:]==tails[name],'exact default CPU four-command argv required')
 require(proof['source_final'].get('Cargo.lock')==r['whole_lock_sha256'],'lock agreement')
 with plain(args.exe).open('rb') as exe:
  for a in binding['assets']:
   exe.seek(a['pe_file_offset']);require(hashlib.sha256(exe.read(a['bytes'])).hexdigest()==a['sha256'],'actual payload bytes')
 zraw=bounded(args.zip,32*1024*1024);require(hashlib.sha256(zraw).hexdigest()==args.zip_sha,'supplied ZIP digest')
 with zip_read(zraw) as z:
  names=z.namelist();require(len(names)==len(set(names)),'duplicate ZIP names')
  for _,(_,_,notice,sha) in FONTS.items():
   i=z.getinfo('licenses/epaint_default_fonts-0.33.3/'+notice);require(i.file_size<=65536 and hashlib.sha256(z.read(i)).hexdigest()==sha,'exact notice bytes')
  i=z.getinfo('candidate-manifest.json');require(i.file_size<=1024*1024,'manifest bound');m=decode(z.read(i));require(m.get('binary_sha256')==cpu and m.get('binary_bytes')==n and m.get('release_ready') is False,'candidate manifest')
  i=z.getinfo('dlpad-cpu.exe');require(i.file_size==n,'archive executable size');h=hashlib.sha256();total=0
  with z.open(i) as f:
   while True:
    b=f.read(65536)
    if not b:break
    total+=len(b);require(total<=32*1024*1024,'archive exe bound');h.update(b)
  require(total==n and h.hexdigest()==cpu,'archive executable digest')
 for file,sha,cap in [(args.exe,args.cpu_sha,32*1024*1024),(args.zip,args.zip_sha,32*1024*1024),(args.proof,args.proof_sha,8*1024*1024),(args.font_result,args.font_result_sha,1024*1024),(args.verifier,args.verifier_sha,1024*1024)]:require(digest(file,cap)[0]==sha,'input changed before publication')
 binding['font_result_sha256']=args.font_result_sha
 out=plain(args.output);root=plain(args.project)/'docs/validation/evidence';require(out.is_relative_to(root) and not out.exists(),'fresh project evidence output')
 payload=(json.dumps(binding,sort_keys=True,indent=2)+'\n').encode();require(len(payload)<=65536,'binding output bound')
 # Exclusive create only; no replace, cleanup or source mutation.
 with out.open('xb') as f:f.write(payload)
 return binding
if __name__=='__main__':
 a=argparse.ArgumentParser()
 for key in ['project','exe','zip','proof','font_result','verifier','output']:a.add_argument('--'+key.replace('_','-'),type=Path,required=True)
 for key in ['cpu_sha','zip_sha','proof_sha','font_result_sha','verifier_sha']:a.add_argument('--'+key.replace('_','-'),required=True)
 bind(a.parse_args())
