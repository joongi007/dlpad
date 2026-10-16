import os
import hashlib,io,json,tarfile,tempfile,unittest
from unittest.mock import patch
from types import SimpleNamespace
from pathlib import Path
import ci_registry_source_binding as v
def symlink_or_skip(test,path,target,directory=False):
    try:path.symlink_to(target,target_is_directory=directory)
    except OSError as error:
        if os.name=='nt' and getattr(error,'winerror',None)==1314:
            test.skipTest('Windows symlink creation privilege unavailable; no system changes')
        raise

class Binding(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name);self.source=self.root/'fixture-1.0';self.source.mkdir();self.archive=self.root/'fixture.crate';self.data=b'original notice\n';(self.source/'LICENSE').write_bytes(self.data);self.required={'LICENSE':{'bytes':len(self.data),'sha256':hashlib.sha256(self.data).hexdigest()}};self.pack([('fixture-1.0/LICENSE',self.data)])
 def pack(self,items):
  with tarfile.open(self.archive,'w:gz')as t:
   for name,data in items:
    i=tarfile.TarInfo(name);i.size=len(data);t.addfile(i,io.BytesIO(data))
  self.sha=hashlib.sha256(self.archive.read_bytes()).hexdigest()
 def run_case(self):return v.validate(self.source,self.archive,'fixture','1.0',self.sha,self.required)
 def test_exact_archive_and_source(self):
  result=self.run_case();self.assertFalse(result['release_ready']);self.assertFalse(result['legal_coverage_complete']);self.assertFalse(result['cargo_checksum_map_present_validated'])
 def test_checksum_optional_but_if_present_exact(self):
  p=self.source/'.cargo-checksum.json';p.write_text(json.dumps({'package':self.sha,'files':{'LICENSE':self.required['LICENSE']['sha256']}}));self.assertTrue(self.run_case()['cargo_checksum_map_present_validated']);p.write_text(json.dumps({'package':'a'*64,'files':{}}))
  with self.assertRaises(ValueError):self.run_case()
 def test_same_size_source_corruption(self):
  (self.source/'LICENSE').write_bytes(b'x'*len(self.data))
  with self.assertRaises(ValueError):self.run_case()
 def test_checksum_claim_cannot_authorize_corrupt_archive(self):
  self.pack([('fixture-1.0/LICENSE',b'x'*len(self.data))]);(self.source/'.cargo-checksum.json').write_text(json.dumps({'package':self.sha,'files':{'LICENSE':self.required['LICENSE']['sha256']}}))
  with self.assertRaises(ValueError):self.run_case()
 def test_archive_sha_mismatch(self):
  self.sha='a'*64
  with self.assertRaises(ValueError):self.run_case()
 def test_traversal_duplicate_and_missing(self):
  for items in [[('fixture-1.0/../LICENSE',self.data)],[('fixture-1.0/LICENSE',self.data),('fixture-1.0/license',self.data)],[('fixture-1.0/OTHER',self.data)]]:
   self.pack(items)
   with self.subTest(items=items),self.assertRaises(ValueError):self.run_case()
 def test_symlink_source_rejected(self):
  path=self.source/'LICENSE';path.unlink();other=self.root/'other';other.write_bytes(self.data);symlink_or_skip(self,path,other)
  with self.assertRaises(ValueError):self.run_case()
 def test_duplicate_checksum_json(self):
  (self.source/'.cargo-checksum.json').write_text('{"package":"'+self.sha+'","files":{},"files":{}}')
  with self.assertRaises(ValueError):self.run_case()
 def test_archive_symlink_and_nonregular(self):
  original=self.root/'original.crate';self.archive.rename(original)
  self.archive.mkdir()
  with self.assertRaises(ValueError):self.run_case()
  self.archive.rmdir()  # Only this fresh empty fixture directory.
  symlink_or_skip(self,self.archive,original)
  with self.assertRaises(ValueError):self.run_case()
 def test_source_ancestor_symlink(self):
  alias=self.root/'alias';symlink_or_skip(self,alias,self.root,True)
  with self.assertRaises(ValueError):v.validate(alias/'fixture-1.0',self.archive,'fixture','1.0',self.sha,self.required)
 def test_windows_reparse_attribute_guard_simulated(self):
  original=Path.lstat
  def info(path):
   actual=original(path)
   return SimpleNamespace(st_mode=actual.st_mode,st_file_attributes=0x400)if path==self.source else actual
  with patch.object(Path,'lstat',info),self.assertRaises(ValueError):self.run_case()
 def test_required_schema_types_and_limits(self):
  for required in [None,[],{}, {'LICENSE':None},{'LICENSE':{'sha256':None,'bytes':1}},{'LICENSE':{'sha256':False,'bytes':1}},{'LICENSE':{'sha256':'a'*64,'bytes':True}},{'LICENSE':{'sha256':'a'*64,'bytes':8*1024*1024+1}}]:
   with self.subTest(required=required),self.assertRaises(ValueError):v.validate(self.source,self.archive,'fixture','1.0',self.sha,required)
  for checksum in [None,False,[],1]:
   with self.subTest(checksum=checksum),self.assertRaises(ValueError):v.validate(self.source,self.archive,'fixture','1.0',checksum,self.required)
  with self.assertRaises(ValueError):v.validate(self.source,self.archive,None,'1.0',self.sha,self.required)
 def test_archive_preopen_identity_mismatch(self):
  import os
  original=os.fstat
  def changed(fd):
   actual=original(fd)
   return SimpleNamespace(st_dev=actual.st_dev,st_ino=actual.st_ino+1,st_size=actual.st_size,st_mtime_ns=actual.st_mtime_ns)
  with patch.object(v.os,'fstat',changed),self.assertRaises(ValueError):self.run_case()
if __name__=='__main__':unittest.main()
