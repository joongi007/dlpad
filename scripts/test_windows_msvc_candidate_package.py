"""Packager entry-point regressions on owned fixtures; native proof and license I/O mocked."""
import copy,hashlib,json,sys,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parent))
import windows_msvc_candidate_package as m
# Preserve the original four regression cases against the candidate implementation.
class LegacyTests(unittest.TestCase):
 def fixture(self):
  ops={n:{'passed':True,'exit_code':0,'source_unchanged':True}for n in ['fmt','clippy','test','release']};art={'dlpad-cpu.exe':{'bytes':100,'sha256':'a'*64},'dlpad-local-install.exe':{'bytes':50,'sha256':'b'*64}};s={'source':'pin'};w={'passed':True,'target':'x86_64-pc-windows-msvc','source_before':s,'source_after':s,'operations':ops,'artifacts':art};ps=[{'name':f'p{i}','version':'1','source':'registry+test','lock_checksum':'c'*64}for i in range(258)];g={'status':'REVIEW_REQUIRED','source_before':s,'source_after':s,'pins':{'whole':'d'*64},'artifacts':art,'union_count':258,'resolved_packages':ps};lock={p['name']+'-1':{'source':p['source'],'checksum':p['lock_checksum']}for p in ps};return w,g,s,lock,art
 def test_wrong_target(self):
  w,g,s,l,a=self.fixture();self.assertEqual(len(m.validate_proofs(w,g,s,'d'*64,l,a)),258);w['target']='x86_64-pc-windows-gnu'
  with self.assertRaises(ValueError):m.validate_proofs(w,g,s,'d'*64,l,a)
 def test_wrong_original_proof_hash(self):
  w,g,s,l,a=self.fixture()
  with self.assertRaises(ValueError):m.validate_proofs(w,g,s,'e'*64,l,a)
 def test_wrong_artifact(self):
  w,g,s,l,a=self.fixture();wrong=copy.deepcopy(a);wrong['dlpad-cpu.exe']['sha256']='e'*64
  with self.assertRaises(ValueError):m.validate_proofs(w,g,s,'d'*64,l,wrong)
 def test_missing_selected_notice(self):
  entries=m.base.BudgetEntries();entries.add_bytes('licenses/a-1/LICENSE',b'grant');inv={'records':[{'name':'a','version':'1','notices':[]}]}
  with self.assertRaises(ValueError):m.select_notices(entries,inv,{'a-1'})
class InstallerTests(unittest.TestCase):
 def setup_case(self,root,installer=True):
  directory=root/'tools/msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release';directory.mkdir(parents=True)
  for name in ['dlpad-cpu.exe','dlpad-local-install.exe','dlpad-install.exe']:(directory/name).write_bytes(name.encode())
  parent=root/'docs/validation/evidence';parent.mkdir(parents=True);wp=parent/'whole.json';gp=parent/'graph.json'
  w,g,s,l,a=LegacyTests().fixture();names=['dlpad-cpu.exe','dlpad-local-install.exe']+(['dlpad-install.exe']if installer else [])
  a={name:{'bytes':(directory/name).stat().st_size,'sha256':hashlib.sha256((directory/name).read_bytes()).hexdigest()}for name in names};w['artifacts']=a;g['artifacts']=a
  for name,op in w['operations'].items():op['operation']=name
  g['operations']=[];wr=json.dumps(w).encode();wp.write_bytes(wr);g['pins']={str(wp):hashlib.sha256(wr).hexdigest()};gp.write_text(json.dumps(g));(root/'Cargo.lock').write_text('fixture');(root/'rust-toolchain.toml').write_text('fixture')
  terminal=b'dlpad-owned-job-terminal natural-zero=True final-zero=True final-active=0 forced=False job-closed=True private-desktop-closed=True\ndlpad-input-desktop-unchanged=True\n'
  for name in w['operations']:(parent/(name+'.log')).write_bytes(terminal)
  entries=m.base.BudgetEntries();entries.add_bytes('licenses/fixture/LICENSE',b'grant');digest=hashlib.sha256(b'grant').hexdigest();inv={'records':[{'name':p['name'],'version':p['version'],'notices':[{'entry':'licenses/fixture/LICENSE','sha256':digest}]}for p in g['resolved_packages']],'missing':[]}
  return directory,parent,wp,gp,w,g,s,l,entries,inv
 def run_case(self,root,case,args,pe=None):
  directory,parent,wp,gp,w,g,s,l,entries,inv=case
  with patch.object(m,'ROOT',root),patch.object(m.n,'TOOLS',root/'tools'),patch.object(m.n,'snapshot',return_value=s),patch.object(m,'require_current_whole'),patch.object(m.base,'toml_inventory',return_value={'package':[dict(name=k.rsplit('-',1)[0],version='1',**v)for k,v in l.items()]}),patch.object(m.base,'licenses',return_value=(entries,inv)),patch.object(m.base,'pe',side_effect=pe or (lambda p,sub:{'subsystem':sub})),patch.object(sys,'argv',['package','--whole-proof',str(wp),'--graph-proof',str(gp),'--output',str(parent/'out.zip'),*args]):m.main()
 def test_optional_pair_missing_rejected_before_reads(self):
  for args in [['--native-installer-exe','/missing.exe'],['--native-installer-sha256','a'*64]]:
   with patch.object(sys,'argv',['package','--whole-proof','/missing','--graph-proof','/missing','--output','/missing.zip',*args]),patch.object(m.base,'bounded',side_effect=AssertionError('must not read')):
    with self.assertRaisesRegex(ValueError,'paired'):m.main()
 def test_installer_wrong_sha_and_missing_file_create_no_zip(self):
  for missing in [False,True]:
   with tempfile.TemporaryDirectory()as d:
    root=Path(d);case=self.setup_case(root);directory,parent=case[:2];exe=directory/'dlpad-install.exe'
    if missing:exe.unlink()
    with self.assertRaises((ValueError,OSError)):self.run_case(root,case,['--native-installer-exe',str(exe),'--native-installer-sha256','0'*64])
    self.assertFalse((parent/'out.zip').exists())
 def test_installer_graph_artifact_mismatch_refused(self):
  with tempfile.TemporaryDirectory()as d:
   root=Path(d);case=self.setup_case(root);directory,parent,wp,gp,w,g=case[:6];g['artifacts']['dlpad-install.exe']=dict(g['artifacts']['dlpad-install.exe'],sha256='0'*64);gp.write_text(json.dumps(g))
   with self.assertRaisesRegex(ValueError,'exact binaries'):self.run_case(root,case,['--native-installer-exe',str(directory/'dlpad-install.exe'),'--native-installer-sha256',hashlib.sha256(b'dlpad-install.exe').hexdigest()])
   self.assertFalse((parent/'out.zip').exists())
 def test_installer_wrong_subsystem_checked_before_output(self):
  with tempfile.TemporaryDirectory()as d:
   root=Path(d);case=self.setup_case(root);directory,parent=case[:2];calls=[]
   def pe(path,sub):
    calls.append((path.name,sub))
    if path.name=='dlpad-install.exe':raise ValueError('exact Windows executable subsystem required')
    return {'subsystem':sub}
   with self.assertRaisesRegex(ValueError,'subsystem'):self.run_case(root,case,['--native-installer-exe',str(directory/'dlpad-install.exe'),'--native-installer-sha256',hashlib.sha256(b'dlpad-install.exe').hexdigest()],pe)
   self.assertIn(('dlpad-install.exe',2),calls);self.assertFalse((parent/'out.zip').exists())
 def test_installer_entry_collision_refused_before_output(self):
  with tempfile.TemporaryDirectory()as d:
   root=Path(d);case=self.setup_case(root);directory,parent=case[:2];case[-2].add_bytes('dlpad-install.exe',b'notice');case[-1]['records'][0]['notices'].append({'entry':'dlpad-install.exe','sha256':hashlib.sha256(b'notice').hexdigest()})
   with self.assertRaisesRegex(ValueError,'collision'):self.run_case(root,case,['--native-installer-exe',str(directory/'dlpad-install.exe'),'--native-installer-sha256',hashlib.sha256(b'dlpad-install.exe').hexdigest()])
   self.assertFalse((parent/'out.zip').exists())
 def test_existing_two_binary_and_optional_three_binary_calls(self):
  import zipfile
  for installer in [False,True]:
   with tempfile.TemporaryDirectory()as d:
    root=Path(d);case=self.setup_case(root,installer);directory,parent=case[:2];args=['--native-installer-exe',str(directory/'dlpad-install.exe'),'--native-installer-sha256',hashlib.sha256(b'dlpad-install.exe').hexdigest()]if installer else []
    self.run_case(root,case,args)
    with zipfile.ZipFile(parent/'out.zip')as z:
     manifest=json.loads(z.read('candidate-manifest.json'));self.assertEqual('native_installer'in manifest,installer);self.assertEqual('dlpad-install.exe'in z.namelist(),installer)
     if installer:self.assertEqual(z.read('dlpad-install.exe'),b'dlpad-install.exe');self.assertFalse(manifest['native_installer']['installer_complete']);self.assertEqual((z.getinfo('dlpad-install.exe').external_attr>>16)&0o777,0o755)
 def test_installer_growth_preserves_failed_output_and_original(self):
  with tempfile.TemporaryDirectory()as d:
   root=Path(d);case=self.setup_case(root);directory,parent=case[:2];exe=directory/'dlpad-install.exe'
   def pe(path,sub):
    if path.name=='dlpad-install.exe':
     with path.open('ab')as stream:stream.write(b'growth')
    return {'subsystem':sub}
   with self.assertRaisesRegex(ValueError,'binary grew'):self.run_case(root,case,['--native-installer-exe',str(exe),'--native-installer-sha256',hashlib.sha256(b'dlpad-install.exe').hexdigest()],pe)
   self.assertTrue((parent/'out.zip').is_file());self.assertEqual(exe.read_bytes(),b'dlpad-install.exegrowth')
if __name__=='__main__':unittest.main()
