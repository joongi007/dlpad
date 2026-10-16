"""Root integration draft: exact byte-identical source, never fabricated VCS metadata."""
import hashlib,json,pathlib,re,tarfile
KEY='winapi-x86_64-pc-windows-gnu-0.4.0';COMMIT='9497609ef44cc9bcd16cd2411c0ee6ccaf5483aa'
CRATE_SHA='712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f'
TREE_SHA='fe41ec7c95ccde956af6906c24c7477e86284542d313504ba100da2f6a589292'
ROOT_TREE_SHA='0e1a2a42d797d993c55361d1e4092671f3f17ac3a1063e3162c63fa8f32970cf'
MAX_TOTAL=64*1024*1024;MAX_FILE=8*1024*1024

def plain(path):
 path=pathlib.Path(path)
 if not path.is_absolute() or '..' in path.parts:raise ValueError('absolute pinned path required')
 for ancestor in [*path.parents,path]:
  if ancestor.is_symlink():raise ValueError('symlink rejected')
 return path

def bounded_hash(path,maximum,git=False):
 path=plain(path);size=path.stat().st_size
 if not path.is_file() or not 0<=size<=maximum:raise ValueError('regular file byte bound')
 h=hashlib.sha1() if git else hashlib.sha256()
 if git:h.update(f'blob {size}\0'.encode())
 total=0
 with path.open('rb') as f:
  while data:=f.read(min(65536,maximum-total+1)):
   total+=len(data)
   if total>maximum or total>size:raise ValueError('file grew beyond bound')
   h.update(data)
 if total!=size:raise ValueError('file shortened')
 return size,h.hexdigest()

def tree(path,expected):
 path=plain(path)
 with path.open('rb') as stream:raw=stream.read(2*1024*1024+1)
 if len(raw)>2*1024*1024 or hashlib.sha256(raw).hexdigest()!=expected:raise ValueError('exact bounded tree pin changed')
 data=json.loads(raw)
 if data.get('truncated') is not False or len(data['tree'])>4096:raise ValueError('complete bounded tree required')
 return {r['path']:r for r in data['tree'] if r['type']=='blob'}

def validate(source,archive,target_tree,root_tree,mapping,package,lock_checksum):
 if set(mapping)!={'repository','commit','path_in_vcs','sources','payload_binding'}:raise ValueError('exact mapping schema')
 if mapping['repository']!='retep998/winapi-rs' or mapping['commit']!=COMMIT or mapping['path_in_vcs']!='x86_64':raise ValueError('exact byte-identical revision required')
 if mapping['sources']!=['retep998-winapi-rs-'+COMMIT+'-LICENSE-MIT','retep998-winapi-rs-'+COMMIT+'-LICENSE-APACHE']:raise ValueError('exact original grants required')
 binding=mapping['payload_binding']
 expected={'schema':1,'kind':'exact-crate-git-tree','crate_sha256':CRATE_SHA,'target_tree_sha256':TREE_SHA,'root_tree_sha256':ROOT_TREE_SHA,'archive_count':1416}
 if binding!=expected or lock_checksum!=CRATE_SHA:raise ValueError('exact payload binding required')
 if package.get('name')!='winapi-x86_64-pc-windows-gnu' or package.get('version')!='0.4.0' or package.get('license')!='MIT/Apache-2.0' or package.get('repository')!='https://github.com/retep998/winapi-rs':raise ValueError('exact package metadata required')
 source=plain(source);archive=plain(archive)
 if source.name!=KEY or archive.name!=KEY+'.crate' or bounded_hash(archive,4*1024*1024)[1]!=CRATE_SHA:raise ValueError('original crate source checksum changed')
 target=tree(target_tree,TREE_SHA);root=tree(root_tree,ROOT_TREE_SHA)
 libraries={n for n in target if n.startswith('lib/') and n.endswith('.a')}
 if len(libraries)!=1416:raise ValueError('fixed upstream archive count')
 expected_files=libraries|{'Cargo.toml.orig','Cargo.toml','build.rs','src/lib.rs'}
 cached_files=set();stack=[(source,0)]
 while stack:
  directory,depth=stack.pop()
  for child in directory.iterdir():
   relative=child.relative_to(source).as_posix()
   if child.is_symlink():raise ValueError('cached symlink rejected')
   if child.is_dir():
    if depth>=1 or relative not in ['lib','src']:raise ValueError('unknown cached directory')
    stack.append((child,depth+1))
   elif child.is_file():
    if len(cached_files)>=1421:raise ValueError('cached inventory count')
    cached_files.add(relative)
   else:raise ValueError('cached special file')
 if cached_files!=expected_files|{'.cargo-ok'}:raise ValueError('unknown/missing cached file')
 bounded_hash(source/'.cargo-ok',64)
 visited=set();total=0
 # Original gzip CRC and exact Cargo archive checksum were validated; never extract.
 with tarfile.open(archive,'r|gz') as stream:
  for member in stream:
   if len(visited)>=1420 or not member.isfile() or member.name.startswith('/') or '..' in member.name.split('/'):raise ValueError('strict original archive members')
   prefix=KEY+'/'
   if not member.name.startswith(prefix):raise ValueError('foreign package member')
   name=member.name[len(prefix):]
   if name not in expected_files or name in visited or not 0<=member.size<=MAX_FILE:raise ValueError('unknown/duplicate/oversize member')
   total+=member.size
   if total>MAX_TOTAL:raise ValueError('expanded payload bound')
   content=stream.extractfile(member)
   if content is None:raise ValueError('missing original payload')
   h=hashlib.sha1();h.update(f'blob {member.size}\0'.encode());sha=hashlib.sha256();observed=0
   while data:=content.read(65536):
    observed+=len(data)
    if observed>member.size:raise ValueError('member byte bound')
    h.update(data);sha.update(data)
   if observed!=member.size:raise ValueError('truncated member')
   # Cached source must match the exact published crate member, not only SPDX.
   cached_size,cached_sha=bounded_hash(source/name,MAX_FILE)
   if cached_size!=member.size or cached_sha!=sha.hexdigest():raise ValueError('cached original payload changed')
   upstream='Cargo.toml' if name=='Cargo.toml.orig' else name
   if name!='Cargo.toml':
    record=target.get(upstream)
    if record is None or record['sha']!=h.hexdigest() or record['size']!=member.size:raise ValueError('original payload Git tree mismatch')
   visited.add(name)
 if visited!=expected_files:raise ValueError('original payload missing')
 if bounded_hash(archive,4*1024*1024)[1]!=CRATE_SHA:raise ValueError('original crate changed after stream')
 # The same root tree contains license grants; caller separately validates supplied
 # exact LICENSE bytes SHA256 and Gitblob versus these original records.
 return {'payload_files':len(visited),'import_archives':len(libraries),'expanded_bytes':total,'root_license_blobs':{n:root[n] for n in ['LICENSE-MIT','LICENSE-APACHE']},'mapping_basis':'byte-identical official source; historical upload revision unverified','transitive_embedded_notice_review_complete':False}
