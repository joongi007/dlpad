"""Reject mismatched snapshot pins and unsupported evidence claims."""
import copy, importlib.util, unittest, tempfile, struct, hashlib
from argparse import Namespace
from unittest import mock
from pathlib import Path
spec=importlib.util.spec_from_file_location('binding',Path(__file__).with_name('windows_font_package_evidence.py'))
b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
class BindingTests(unittest.TestCase):
 def setUp(self):
  self.cpu='8f21c5164fa9b0f9fdb488029fb577615b5b38fb1edc64fd5b8cb8ed9904a7ed';self.z='2'*64;self.p='3'*64;self.v='4'*64;off=5000;assets=[]
  pins={'lib':b.LIB,'crate':b.CRATE,'assets':{}}
  for f,(n,s,notice,ns) in b.FONTS.items():assets.append({'font':f,'bytes':n,'sha256':s,'pe_file_offset':off});off+=n;pins['assets'][f]=s;pins['assets'][notice]=ns
  self.r={'passed':True,'exact_stored_payload_verified':True,'cpu_sha256':self.cpu,'zip_sha256':self.z,'verifier_sha256':self.v,'proof':{'sha256':self.p,'cpu_artifact':{'sha256':self.cpu,'bytes':7895040},'operations':['fmt','clippy','test','release'],'proof_format':'workspace-actual-operations-v1','source_files':541},'source_pins':pins,'assets':assets,'whole_lock_sha256':'5'*64,**{k:False for k in b.FALSE}}
 def validate(self,r=None,cpu=None):return b.validate_result(r or self.r,cpu or self.cpu,self.z,self.p,self.v,7895040)
 def test_exact_mapping_keeps_unverified_false(self):
  x=self.validate();self.assertTrue(x['stored_payload_verified']);self.assertTrue(all(x[k] is False for k in b.FALSE));self.assertEqual(len(x['assets']),4)
 def test_historical8f_cannot_bind_new_cpu(self):
  with self.assertRaises(ValueError):self.validate(cpu='c'*64)
 def test_wrong_zip_proof_and_verifier_are_rejected(self):
  for key in ['zip_sha256','verifier_sha256']:
   r=copy.deepcopy(self.r);r[key]='0'*64
   with self.subTest(key=key),self.assertRaises(ValueError):self.validate(r)
  r=copy.deepcopy(self.r);r['proof']['sha256']='0'*64
  with self.assertRaises(ValueError):self.validate(r)
 def test_failed_or_missing_original_verification_is_rejected(self):
  for key in ['passed','exact_stored_payload_verified']:
   for value in [False,None,1]:
    r=copy.deepcopy(self.r);r[key]=value
    with self.subTest(key=key,value=value),self.assertRaises(ValueError):self.validate(r)
 def test_overlapping_payload_ranges_are_rejected(self):
  r=copy.deepcopy(self.r);r['assets'][1]['pe_file_offset']=r['assets'][0]['pe_file_offset']+1
  with self.assertRaises(ValueError):self.validate(r)
 def test_duplicate_json_keys_are_rejected(self):
  with tempfile.TemporaryDirectory() as tmp:
   path=Path(tmp)/'proof.json';path.write_text('{"passed":true,"passed":false}')
   with self.assertRaises(ValueError):b.obj(path,4096)
 def test_oversized_proof_is_rejected_before_decode(self):
  with tempfile.TemporaryDirectory() as tmp:
   path=Path(tmp)/'proof.json';path.write_bytes(b' '*17)
   with self.assertRaises(ValueError):b.obj(path,16)
 def test_zip_entry_budget_is_checked_before_zipfile(self):
  raw=struct.pack('<4s4H2IH',b'PK\x05\x06',0,0,5000,5000,0,0,0)
  with mock.patch.object(b.zipfile,'ZipFile') as constructor:
   with self.assertRaises(ValueError):b.zip_read(raw)
   constructor.assert_not_called()
 def test_decoder_uses_supplied_bytes_without_file_reads(self):
  with mock.patch.object(b,'bounded',side_effect=AssertionError('unexpected reread')):
   self.assertEqual(b.decode(b'{"passed":true}'),{'passed':True})
   with self.assertRaises(ValueError):b.decode(b'{"passed":true,"passed":false}')
 def test_bind_rejects_hashed_failed_proof_without_reopening_for_parse(self):
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp);exe=root/'app.exe';exe.write_bytes(b'synthetic executable')
   verifier=root/'verifier.py';verifier.write_bytes(b'synthetic verifier')
   font=root/'font.json';font.write_bytes(b'{}')
   proof=root/'proof.json';proof.write_bytes(b'{"passed":false}')
   def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
   args=Namespace(project=root,exe=exe,zip=root/'unused.zip',proof=proof,
    font_result=font,verifier=verifier,output=root/'unused-output.json',
    cpu_sha=sha(exe),zip_sha='1'*64,proof_sha=sha(proof),
    font_result_sha=sha(font),verifier_sha=sha(verifier))
   # Isolate hash-to-parser handoff from the independent font mapping checks.
   with mock.patch.object(b,'validate_result',return_value={'assets':[]}),mock.patch.object(
    b,'obj',side_effect=AssertionError('unverified path reread')):
    with self.assertRaisesRegex(ValueError,'workspace source proof'):b.bind(args)
   self.assertFalse(args.output.exists())
 def test_failure_shapes(self):
  cases=[]
  for key in b.FALSE:r=copy.deepcopy(self.r);r[key]=True;cases.append(r)
  r=copy.deepcopy(self.r);r['assets']=r['assets'][:3];cases.append(r)
  r=copy.deepcopy(self.r);r['assets'][1]=r['assets'][0];cases.append(r)
  r=copy.deepcopy(self.r);r['assets'][0]['pe_file_offset']=True;cases.append(r)
  r=copy.deepcopy(self.r);r['assets'][0]['pe_file_offset']=7895040;cases.append(r)
  r=copy.deepcopy(self.r);r['source_pins']['assets']['OFL.txt']='0'*64;cases.append(r)
  r=copy.deepcopy(self.r);r['proof']['cpu_artifact']['sha256']='0'*64;cases.append(r)
  r=copy.deepcopy(self.r);r['proof']['operations']=['test'];cases.append(r)
  r=copy.deepcopy(self.r);r['proof']['source_files']=False;cases.append(r)
  for r in cases:
   with self.subTest(r=r),self.assertRaises(ValueError):self.validate(r)
if __name__=='__main__':unittest.main()
