"""Read-only exact generated payload binding validation for native collector."""
import hashlib,json,re
from pathlib import Path
NAMES={'dlpad.exe','LICENSE','THIRD-PARTY-NOTICES.txt','README.txt','binding-record.txt'}

def exact_json(raw):
 def pairs(items):
  result={}
  for key,value in items:
   if key in result:raise ValueError('duplicate JSON field')
   result[key]=value
  return result
 return json.loads(raw,object_pairs_hook=pairs)

def validate(h,root,manifest_sha,binding_sha,whole):
 root=h.plain(root)
 if not root.is_relative_to(h.n.ROOT/'docs/validation/evidence'):raise ValueError('generated project evidence only')
 if {p.name for p in root.iterdir()}!={'dlpad-install.exe','payload'}:raise ValueError('exact outer namespace')
 payload=h.plain(root/'payload')
 if not payload.is_dir()or {p.name for p in payload.iterdir()}!=NAMES|{'manifest.json'}:raise ValueError('exact payload namespace')
 for value in [manifest_sha,binding_sha]:
  if not re.fullmatch('[0-9a-f]{64}',value):raise ValueError('original lowercase SHA required')
 files={str(h.plain(root/'dlpad-install.exe')):h.sha(root/'dlpad-install.exe')}
 raw=h.bounded(payload/'manifest.json',65536)
 if hashlib.sha256(raw).hexdigest()!=manifest_sha:raise ValueError('manifest original pin')
 manifest=exact_json(raw)
 if set(manifest)!={'schema','version','target','entries','development_only'}or type(manifest['schema'])is not int or manifest['schema']!=1 or manifest['development_only']is not True or manifest['target']!='x86_64-pc-windows-msvc' or not re.fullmatch(r'(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})',manifest['version']):raise ValueError('exact development manifest')
 entries=manifest['entries'];proofs={};total=len(raw)
 if not isinstance(entries,list)or len(entries)!=5:raise ValueError('exact five entries')
 for entry in entries:
  if not isinstance(entry,dict)or set(entry)!={'name','bytes','sha256'}:raise ValueError('entry schema')
  name=entry['name'];limit=134217728 if name=='dlpad.exe'else 16777216
  if name not in NAMES or name in proofs or type(entry['bytes'])is not int or not 0<entry['bytes']<=limit or not isinstance(entry['sha256'],str)or not re.fullmatch('[0-9a-f]{64}',entry['sha256']):raise ValueError('entry domain')
  path=h.plain(payload/name)
  if not path.is_file()or path.stat().st_size!=entry['bytes']or h.sha(path)!=entry['sha256']:raise ValueError('original payload bytes')
  proofs[name]=entry;files[str(path)]=entry['sha256'];total+=entry['bytes']
 if proofs['binding-record.txt']['sha256']!=binding_sha:raise ValueError('binding original pin')
 files[str(payload/'manifest.json')]=manifest_sha
 binding=exact_json(h.bounded(payload/'binding-record.txt',16777216))
 if set(binding)!={'schema','development_only','release_ready','complete_legal_coverage','signed_distribution','candidate_zip_sha256','candidate_manifest_sha256','mapping','binaries','project_license','notices'}or type(binding['schema'])is not int or binding['schema']!=1 or binding['development_only']is not True or any(binding[k]is not False for k in ['release_ready','complete_legal_coverage','signed_distribution']):raise ValueError('unsigned development binding')
 if binding['mapping']!={'dlpad-cpu.exe':'payload/dlpad.exe','dlpad-install.exe':'dlpad-install.exe'}:raise ValueError('mapping')
 for key in ['candidate_zip_sha256','candidate_manifest_sha256']:
  if not isinstance(binding[key],str)or not re.fullmatch('[0-9a-f]{64}',binding[key]):raise ValueError('candidate pin domain')
 binaries=binding['binaries']
 if set(binaries)!={'dlpad-cpu.exe','dlpad-install.exe'}:raise ValueError('binary namespace')
 for name,path in [('dlpad-cpu.exe',payload/'dlpad.exe'),('dlpad-install.exe',root/'dlpad-install.exe')]:
  expected=whole['artifacts'][name]
  if binaries[name]!=expected or h.sha(path)!=expected['sha256']or path.stat().st_size!=expected['bytes']:raise ValueError('current whole binary binding')
 total+=(root/'dlpad-install.exe').stat().st_size
 if total>268435456:raise ValueError('package aggregate bound')
 license=binding['project_license']
 if set(license)!={'source_name','bytes','sha256'}or license['bytes']!=proofs['LICENSE']['bytes']or license['sha256']!=proofs['LICENSE']['sha256']:raise ValueError('project license binding')
 if not isinstance(binding['notices'],dict)or not 0<len(binding['notices'])<=4096:raise ValueError('notice bounded inventory')
 return files,proofs
