import importlib.util,tempfile,struct,shutil,json,hashlib,zipfile
from unittest.mock import patch
from pathlib import Path
spec=importlib.util.spec_from_file_location('probe',str(Path(__file__).resolve().with_name('windows_font_payload_probe.py')));p=importlib.util.module_from_spec(spec);spec.loader.exec_module(p)
rows=[]
def reject(name,fn,match):
 try:fn()
 except ValueError as e:
  assert match in str(e),(name,str(e));rows.append({'case':name,'error':str(e)})
 else:raise AssertionError(name+' accepted')
def pe(payload):
 b=bytearray(512+len(payload));b[:2]=b'MZ';struct.pack_into('<I',b,60,64);b[64:68]=b'PE\0\0';struct.pack_into('<H',b,68,0x8664);struct.pack_into('<H',b,70,1);struct.pack_into('<H',b,84,112);struct.pack_into('<H',b,88,0x20b);b[200:208]=b'.rdata\0\0';struct.pack_into('<II',b,216,len(payload),512);b[512:]=payload;return bytes(b)
font=b'synthetic-font-bytes';assert p.assets(pe(font),{'synthetic':font})[0]['pe_file_offset']==512
reject('missing_payload',lambda:p.assets(pe(b'no'),{'synthetic':font}),'missing/duplicate')
reject('duplicate_payload',lambda:p.assets(pe(font+font),{'synthetic':font}),'missing/duplicate')
reject('malformed_PE',lambda:p.assets(b'MZ',{'synthetic':font}),'PE truncated')
bad=bytearray(pe(font));struct.pack_into('<I',bad,220,100)
reject('section_overlaps_header',lambda:p.sections(bytes(bad)),'section overlap/bound')
bad=bytearray(pe(font));struct.pack_into('<I',bad,216,len(bad)+1)
reject('section_outside_file',lambda:p.sections(bytes(bad)),'section overlap/bound')
root=Path(__file__).resolve().parents[1];source=next((root/'.local-tools/cargo/registry/src').glob('*/'+p.KEY));before={str(x):p.sha(x.read_bytes()) for x in [source/'src/lib.rs',*[source/'fonts'/n for pair in p.MAP for n in pair]]}
with tempfile.TemporaryDirectory(prefix='dlpad-font-negative-owned-') as td:
 own=Path(td);copy=own/p.KEY;(copy/'src').mkdir(parents=True);(copy/'fonts').mkdir()
 def make_zip(name,manifest,binary):
  zpath=own/name
  with zipfile.ZipFile(zpath,'w') as z:
   z.writestr('candidate-manifest.json',json.dumps(manifest));z.writestr('dlpad-cpu.exe',binary)
  return zpath
 many=own/'many.zip'
 with zipfile.ZipFile(many,'w') as z:
  for index in range(4097):z.writestr('entry-'+str(index),b'')
 with patch.object(p.zipfile,'ZipFile',side_effect=AssertionError('unbounded ZipInfo allocation')):
  reject('zip_declared_excess_entries_before_allocation',lambda:p.bounded_zip(many),'ZIP central directory bound')
 raw=bytearray(many.read_bytes());eocd=raw.rfind(b'PK\x05\x06');struct.pack_into('<HH',raw,eocd+8,1,1);forged=own/'forged-count.zip';forged.write_bytes(raw)
 with patch.object(p.zipfile,'ZipFile',side_effect=AssertionError('unbounded ZipInfo allocation')):
  reject('zip_forged_small_count_before_allocation',lambda:p.bounded_zip(forged),'ZIP central record bound')
 synthetic=pe(font);manifest={'binary_sha256':p.sha(synthetic),'binary_bytes':len(synthetic)}
 p.verify_zip_cpu(make_zip('positive.zip',manifest,synthetic),synthetic)
 reject('zip_wrong_manifest_cpu',lambda:p.verify_zip_cpu(make_zip('manifest.zip',dict(manifest,binary_sha256='0'*64),synthetic),synthetic),'ZIP candidate CPU mismatch')
 reject('zip_wrong_binary_bytes',lambda:p.verify_zip_cpu(make_zip('bytes.zip',manifest,synthetic[:-1]+b'x'),synthetic),'ZIP binary digest mismatch')
 reject('zip_wrong_binary_size',lambda:p.verify_zip_cpu(make_zip('size.zip',manifest,synthetic+b'x'),synthetic),'ZIP binary size mismatch')

 for rel in ['src/lib.rs',*['fonts/'+n for pair in p.MAP for n in pair]]:shutil.copyfile(source/rel,copy/rel)
 assert p.source_files(copy)
 for label,rel,expected in [('lib_declaration_tamper','src/lib.rs','lib source changed'),('asset_tamper','fonts/Hack-Regular.ttf','font/notice changed'),('notice_tamper','fonts/OFL.txt','font/notice changed')]:
  f=copy/rel;data=f.read_bytes();f.write_bytes(bytes([data[0]^1])+data[1:]);reject(label,lambda:p.source_files(copy),expected);f.write_bytes(data)
 archive=source.parent.parent.parent/'cache'/source.parent.name/(p.KEY+'.crate');a=own/(p.KEY+'.crate');shutil.copyfile(archive,a)
 with a.open('r+b') as f:x=f.read(1);f.seek(0);f.write(bytes([x[0]^1]))
 reject('original_crate_copy_tamper',lambda:p.original_crate(a,p.source_files(copy),p.PINS['crate']),'original crate checksum')
 reject('wrong_original_lock_checksum',lambda:p.original_crate(archive,p.source_files(copy),'0'*64),'original crate checksum')
assert before=={f:p.sha(Path(f).read_bytes()) for f in before}
report={'passed':True,'negative_cases':rows,'synthetic_PE_positive':True,'original_source_pins_unchanged':True,'native_run':False,'project_modified':False}
print(json.dumps(report,indent=2))
