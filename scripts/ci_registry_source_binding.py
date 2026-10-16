"""Explicit registry archive -> selected original source bytes binding; not legal proof."""
import gzip,hashlib,io,json,os,re,stat,tarfile
from pathlib import Path
MAX=256*1024*1024
D=re.compile('[0-9a-f]{64}')
def plain(path):
 try:p=Path(path)
 except (TypeError,ValueError)as error:raise ValueError('path domain')from error
 if not p.is_absolute()or '..'in p.parts:raise ValueError('plain absolute path')
 for ancestor in [p,*p.parents]:
  try:info=ancestor.lstat()
  except FileNotFoundError:continue
  if stat.S_ISLNK(info.st_mode)or getattr(info,'st_file_attributes',0)&getattr(stat,'FILE_ATTRIBUTE_REPARSE_POINT',0x400):raise ValueError('symlink/reparse path rejected')
 return p

def leaf(name):
 if not isinstance(name,str)or not 1<=len(name)<=4096 or '\\'in name or ':'in name or any(ord(x)<32 for x in name)or name.startswith('/')or any(x in ['', '.', '..']for x in name.split('/')):raise ValueError('relative file name')
 return name

def hashed(stream,maximum):
 h=hashlib.sha256();count=0
 while block:=stream.read(65536):
  count+=len(block)
  if count>maximum:raise ValueError('stream bound')
  h.update(block)
 return count,h.hexdigest()

def unique(items):
 out={}
 for k,v in items:
  if k in out:raise ValueError('duplicate JSON key')
  out[k]=v
 return out

class Limited(io.RawIOBase):
 def __init__(self,stream):self.stream=stream;self.total=0
 def read(self,size=-1):
  if size<0 or size>1024*1024:raise ValueError('archive read request bound')
  data=self.stream.read(size);self.total+=len(data)
  if self.total>MAX+20*1024*1024:raise ValueError('expanded archive bound')
  return data

def validate(source,archive,name,version,lock_checksum,required):
 source=plain(source);archive=plain(archive)
 if not isinstance(lock_checksum,str)or not D.fullmatch(lock_checksum):raise ValueError('exact lock checksum')
 if not isinstance(name,str)or not isinstance(version,str):raise ValueError('package identity string domain')
 prefix=leaf(name+'-'+version)
 if '/'in prefix or source.name!=prefix or not source.is_dir():raise ValueError('exact package directory')
 if not isinstance(required,dict)or not 1<=len(required)<=64:raise ValueError('required original file map bound')
 expected={}
 for key,value in required.items():
  key=leaf(key)
  if not isinstance(value,dict)or set(value)!={'sha256','bytes'}or not isinstance(value.get('sha256'),str)or not D.fullmatch(value['sha256'])or type(value['bytes'])is not int or not 0<=value['bytes']<=8*1024*1024:raise ValueError('explicit original file pin')
  expected[key]=value
 archive_stat=archive.stat()
 if not stat.S_ISREG(archive_stat.st_mode)or not 0<=archive_stat.st_size<=MAX:raise ValueError('regular bounded archive required')
 archive_identity=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns)
 identity=archive_identity(archive_stat)
 with archive.open('rb')as pin:
  if archive_identity(os.fstat(pin.fileno()))!=identity:raise ValueError('archive original handle mismatch')
  if hashed(pin,MAX)[1]!=lock_checksum:raise ValueError('original crate archive lock SHA mismatch')
  pin.seek(0);seen=set();found={};regular=0
  with gzip.GzipFile(fileobj=pin,mode='rb')as gz,tarfile.open(fileobj=Limited(gz),mode='r|')as tar:
   for item in tar:
    if len(seen)>=65536:raise ValueError('archive entry bound')
    path=leaf(item.name.rstrip('/')if item.isdir()else item.name)
    if path.casefold()in seen:raise ValueError('duplicate/case-colliding archive path')
    seen.add(path.casefold())
    if not path.startswith(prefix+'/')and path!=prefix:raise ValueError('archive package root')
    if item.isdir():continue
    if not item.isfile()or item.size<0:raise ValueError('archive regular files only')
    regular+=item.size
    if regular>MAX:raise ValueError('archive aggregate bytes')
    relative=path[len(prefix)+1:]
    if relative in expected:
     with tar.extractfile(item)as f:count,digest=hashed(f,8*1024*1024)
     if {'bytes':count,'sha256':digest}!=expected[relative]:raise ValueError('original archive required file mismatch')
     found[relative]={'bytes':count,'sha256':digest}
  if set(found)!=set(expected):raise ValueError('archive required original absent')
  pin.seek(0)
  if hashed(pin,MAX)[1]!=lock_checksum:raise ValueError('archive changed during validation')
  plain(archive);post=archive.stat()
  if archive_identity(os.fstat(pin.fileno()))!=identity or archive_identity(post)!=identity:raise ValueError('archive identity changed')
 checksum=plain(source/'.cargo-checksum.json');checksum_proven=False
 if checksum.exists():
  if not stat.S_ISREG(checksum.stat().st_mode):raise ValueError('regular checksum manifest required')
  with checksum.open('rb')as f:raw=f.read(1024*1024+1)
  if len(raw)>1024*1024:raise ValueError('checksum manifest bound')
  value=json.loads(raw,object_pairs_hook=unique)
  if not isinstance(value,dict)or set(value)!={'files','package'}or value['package']!=lock_checksum or not isinstance(value['files'],dict)or len(value['files'])>65536:raise ValueError('checksum manifest package binding')
  for key,digest in value['files'].items():
   leaf(key)
   if not isinstance(digest,str)or not D.fullmatch(digest):raise ValueError('checksum file map digest')
  for key,value_pin in expected.items():
   if value['files'].get(key)!=value_pin['sha256']:raise ValueError('checksum original file map mismatch')
  checksum_proven=True
 for key,value in expected.items():
  path=plain(source/key);before=path.stat()
  if not stat.S_ISREG(before.st_mode):raise ValueError("regular source original required")
  with path.open('rb')as f:
   identity=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns)
   if identity(os.fstat(f.fileno()))!=identity(before)or hashed(f,8*1024*1024)!= (value['bytes'],value['sha256'])or identity(os.fstat(f.fileno()))!=identity(before):raise ValueError('actual source original mismatch')
  plain(path)
  if identity(path.stat())!=identity(before):raise ValueError('source original changed')
 return {'package':prefix,'archive_sha256':lock_checksum,'original_files':found,'cargo_checksum_map_present_validated':checksum_proven,'release_ready':False,'legal_coverage_complete':False,'exact_linked_payload_audited':False}
