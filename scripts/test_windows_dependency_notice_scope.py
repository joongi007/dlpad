import importlib.util, copy, unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('scope',str(Path(__file__).with_name('windows_dependency_notice_scope.py')))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Tests(unittest.TestCase):
 def fixture(self):
  pin={'Cargo.lock':'a'*64,'Cargo.toml':'b'*64}
  r={'argv':['host.exe','60','cargo.exe']+m.TAIL,'passed':True,'exit_code':0,'collector_failure':None,'source_before':pin,'source_after':dict(pin)}
  log=b'dlpad-owned-cli pid=23 private-desktop=true\na v1.2.3 (D:\\owned path)\nb v2.0.0 (*)\ndlpad-owned-job-terminal natural-zero=True final-zero=True final-active=0 forced=False job-closed=True private-desktop-closed=True stdin-cancel=False result=0\ndlpad-input-desktop-unchanged=True\n'
  i={'records':[{'name':x,'version':v} for x,v in [('a','1.2.3'),('b','2.0.0'),('c','3.0.0')]],'missing':['b-2.0.0: notice absent','c-3.0.0: notice absent']}
  return r,log,i,pin
 def run_case(self,change=None):
  r,l,i,p=self.fixture()
  if change:r,l,i,p=change(r,l,i,p)
  return m.classify(r,l,i,p,'host.exe','cargo.exe', {'a-1.2.3','b-2.0.0','c-3.0.0'} | {x['name']+'-'+x['version'] for x in i['records'] if x['name'].startswith('dlpad')})
 def test_positive_retains_whole_missing(self):
  x=self.run_case();self.assertEqual(x['missing_in_tree'],['b-2.0.0']);self.assertEqual(x['missing_not_in_this_tree'],['c-3.0.0']);self.assertEqual(len(x['whole_lock_missing_unchanged']),2);self.assertFalse(x['release_ready'])
 def test_missing_sources_need_not_have_notice_records(self):
  x=self.run_case(lambda r,l,i,p:(r,l,dict(i,records=i['records'][:1]),p))
  self.assertEqual(x['missing_in_tree'],['b-2.0.0'])
  self.assertEqual(x['whole_lock_packages'],['a-1.2.3','b-2.0.0','c-3.0.0'])
 def test_unaccounted_lock_package_is_rejected(self):
  with self.assertRaises(ValueError):self.run_case(lambda r,l,i,p:(r,l,dict(i,missing=i['missing'][:1],records=i['records'][:2]),p))
 def test_actual_workspace_package_names_are_not_host_banners(self):
  def change(r,l,i,p):
   l=l.replace(b'a v1.2.3',b'dlpad v0.1.0 (D:\\owned workspace)\ndlpad-core v0.1.0 (D:\\owned workspace\\crates\\core)\ndlpad-engine v0.1.0 (D:\\owned workspace\\crates\\engine)\na v1.2.3')
   i['records'].extend({'name':name,'version':'0.1.0'} for name in ['dlpad','dlpad-core','dlpad-engine'])
   return r,l,i,p
  x=self.run_case(change)
  self.assertTrue({'dlpad-0.1.0','dlpad-core-0.1.0','dlpad-engine-0.1.0'} <= set(x['tree_packages']))
 def test_bom_and_native_warning_are_explicit_rejections(self):
  for prefix in [b'\xef\xbb\xbf',b'warning: native warning\n']:
   with self.assertRaises(ValueError):self.run_case(lambda r,l,i,p:(r,prefix+l,i,p))
 def test_changed_current_source_explicit(self):
  x=self.run_case(lambda r,l,i,p:(r,l,i,dict(p,**{'Cargo.toml':'c'*64})));self.assertFalse(x['source_current'])
 def test_negative_receipts(self):
  for field,value in [('passed',False),('exit_code',125),('collector_failure','timeout')]:
   with self.subTest(field=field),self.assertRaises(ValueError):self.run_case(lambda r,l,i,p:(dict(r,**{field:value}),l,i,p))
 def test_wrong_command(self):
  for token in ['cpu','normal,build','x86_64-pc-windows-gnu','--offline']:
   def change(r,l,i,p):r['argv'][r['argv'].index(token)]='wrong';return r,l,i,p
   with self.subTest(token=token),self.assertRaises(ValueError):self.run_case(change)
 def test_terminal_failure(self):
  for old,new in [(b'natural-zero=True',b'natural-zero=False'),(b'private-desktop-closed=True',b'private-desktop-closed=False'),(b'final-active=0',b'final-active=1')]:
   with self.subTest(old=old),self.assertRaises(ValueError):self.run_case(lambda r,l,i,p:(r,l.replace(old,new),i,p))
 def test_bad_logs(self):
  for extra in [b'warning: unexpected\n',b'bad vnotversion\n',b'x'* (m.MAX_LOG+1)]:
   with self.assertRaises(ValueError):self.run_case(lambda r,l,i,p:(r,l+extra,i,p))
 def test_inventory_gap(self):
  def change(r,l,i,p):i['records'].pop(0);return r,l,i,p
  with self.assertRaises(ValueError):self.run_case(change)
 def test_wrong_lock_or_after(self):
  for after in [True,False]:
   def change(r,l,i,p):
    if after:r['source_after']={'Cargo.lock':'c'*64}
    else:p=dict(p,**{'Cargo.lock':'c'*64})
    return r,l,i,p
   with self.assertRaises(ValueError):self.run_case(change)
if __name__=='__main__':unittest.main()
