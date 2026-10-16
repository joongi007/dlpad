"""Synthetic tests only; no native execution, real cache or candidate artifacts."""
import contextlib,copy,hashlib,importlib.util,io,json,os,pathlib,struct,sys,tempfile,unittest,zipfile
from unittest import mock
ROOT=pathlib.Path(__file__).resolve().parent

def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
P=load('pack_companion',os.environ.get('DLPAD_TEST_PACKAGER',str(ROOT/'windows_candidate_package.py')))
R=load('review_companion',os.environ.get('DLPAD_TEST_REVIEWER',str(ROOT/'windows_candidate_package_review.py')))

def sha(raw):return hashlib.sha256(raw).hexdigest()
def pe_bytes(subsystem):
    b=bytearray(512);b[:2]=b'MZ';struct.pack_into('<I',b,60,64);b[64:68]=b'PE\0\0';struct.pack_into('<HHIIIHH',b,68,0x8664,1,0,0,0,128,0);struct.pack_into('<H',b,88,0x20b);struct.pack_into('<H',b,156,subsystem);struct.pack_into('<I',b,196,2);return bytes(b)

class CompanionTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);self.root=pathlib.Path(self.temp.name)
        self.cpu=self.root/'cpu.exe';self.cli=self.root/'cli.exe';self.cpu.write_bytes(pe_bytes(2));self.cli.write_bytes(pe_bytes(3));self.out=self.root/'candidate.zip'
        for name in ['Cargo.lock','rust-toolchain.toml','docs/CICD.md','docs/adr/0001-native-ui.md']:
            f=self.root/name;f.parent.mkdir(parents=True,exist_ok=True);f.write_bytes(b'synthetic fixture\n')
        self.source={n:sha((self.root/n).read_bytes())for n in ['Cargo.lock','rust-toolchain.toml']}
        commands={'fmt':['fmt','--all','--','--check'],'clippy':['clippy','--workspace','--all-targets','--locked','--features','cpu','--','-D','warnings'],'test':['test','--workspace','--locked','--features','cpu'],'release':['build','--workspace','--release','--locked','--features','cpu']}
        operations={n:{'operation':n,'passed':True,'source_unchanged':True,'exit_code':0,'collector_failure':None,'input_desktop_unchanged':True,'private_desktop_reported':True,'source_before':self.source,'source_after':self.source,'argv':['host','60','cargo',*cmd]}for n,cmd in commands.items()}
        self.proof={'scope':'Windows GNU CPU CLI; GUI/IME/MSVC/macOS/Linux keeper runtime unverified','passed':True,'cargo_checks_complete':True,'source_unchanged':True,'source_baseline':self.source,'source_final':self.source,'actual_operations':operations,'cpu_artifact':{'bytes':512,'sha256':sha(self.cpu.read_bytes())},'local_install_artifact':{'bytes':512,'sha256':sha(self.cli.read_bytes())}}
        self.proofpath=self.root/'proof.json';self.saveproof();self.patch=mock.patch.object(P,'ROOT',self.root);self.patch.start();self.addCleanup(self.patch.stop)
    def saveproof(self):self.proofpath.write_text(json.dumps(self.proof))
    def package(self,flags=None):
        entries=P.BudgetEntries();inventory={'complete':False,'records':[],'missing':[],'scope':'synthetic only'}
        argv=['package','--exe',str(self.cpu),'--sha256',sha(self.cpu.read_bytes()),'--checks-proof',str(self.proofpath),'--output',str(self.out),'--allow-missing-licenses']
        if flags is None:flags=['--local-install-exe',str(self.cli),'--local-install-sha256',self.proof['local_install_artifact']['sha256']]
        with mock.patch.object(sys,'argv',argv+flags),mock.patch.object(P,'licenses',return_value=(entries,inventory)),contextlib.redirect_stdout(io.StringIO()):P.main()
    def expected(self):
        e=R.proof(self.proofpath);e['_proof_sha']=sha(self.proofpath.read_bytes());return e
    def rewrite(self,mutate):
        with zipfile.ZipFile(self.out)as z:members=[(i,z.read(i))for i in z.infolist()]
        members=mutate(members);replacement=self.root/'changed.zip'
        with zipfile.ZipFile(replacement,'w',compression=zipfile.ZIP_STORED)as z:
            for i,b in members:z.writestr(i,b)
        return replacement
    def test_actual_companion_and_legacy_cpu_only(self):
        self.package();R.review(self.out,self.expected())
        with zipfile.ZipFile(self.out)as z:self.assertEqual(z.read('dlpad-local-install.exe'),self.cli.read_bytes())
        self.out.unlink();del self.proof['local_install_artifact'];self.saveproof();self.package([]);R.review(self.out,self.expected())
        with zipfile.ZipFile(self.out)as z:self.assertNotIn('dlpad-local-install.exe',z.namelist());self.assertNotIn('local_install_companion',json.loads(z.read('candidate-manifest.json')))
    def test_legacy_records_allow_cpu_only_but_cannot_supply_companion(self):
        current=copy.deepcopy(self.proof)
        self.proof={'scope':'default','feature':'cpu','complete':True,'records':list(current['actual_operations'].values()),'source_final':self.source,'binary_sha':current['cpu_artifact']['sha256'],'binary_bytes':512}
        self.saveproof();self.package([]);R.review(self.out,self.expected())
        self.out.unlink();self.proof['local_install_artifact']=current['local_install_artifact'];self.saveproof()
        with self.assertRaisesRegex(ValueError,'canonical workspace'):self.package()
        self.assertFalse(self.out.exists())
        self.proof=current;self.saveproof();self.package();expected=self.expected();expected['records']=[]
        with self.assertRaisesRegex(ValueError,'canonical workspace'):R.review(self.out,expected)

    def test_checks_digest_matches_exact_decoded_bytes_without_proof_reopen(self):
        raw=self.proofpath.read_bytes();original=P.digest
        def digest(path):
            if path==self.proofpath:raise AssertionError('proof pathname reopened for hash')
            return original(path)
        with mock.patch.object(P,'digest',side_effect=digest):result=P.checks(self.proofpath)
        self.assertEqual(result['sha256'],sha(raw));self.assertEqual(result['local_install_artifact'],self.proof['local_install_artifact'])
        self.assertEqual(R.proof(self.proofpath)['_proof_sha'],sha(raw))

    def test_review_main_detects_proof_replaced_after_decode(self):
        self.package();second=self.root/'second.zip';second.write_bytes(self.out.read_bytes());original=R.proof
        def replaced(path):
            decoded=original(path);path.write_bytes(path.read_bytes()+b' ');return decoded
        argv=['review','--first',str(self.out),'--second',str(second),'--final-proof',str(self.proofpath)]
        with mock.patch.object(sys,'argv',argv),mock.patch.object(R,'proof',side_effect=replaced),contextlib.redirect_stdout(io.StringIO()):
            with self.assertRaisesRegex(ValueError,'input changed'):R.main()

    def test_failed_package_preserves_foreign_output_replacement(self):
        original=P.checks;calls=0;retired=self.root/'original-created.zip'
        def fail(path):
            nonlocal calls
            calls+=1
            if calls==2:
                self.out.rename(retired);self.out.write_bytes(b'foreign output sentinel');raise ValueError('synthetic post-check failure')
            return original(path)
        with mock.patch.object(P,'checks',side_effect=fail):
            with self.assertRaisesRegex(ValueError,'synthetic post-check'):self.package()
        self.assertTrue(retired.exists());self.assertTrue(self.out.exists());self.assertEqual(self.out.read_bytes(),b'foreign output sentinel')

    def test_digest_rejects_stream_growth_after_initial_size(self):
        fake=mock.MagicMock();fake.stat.return_value.st_size=64;fake.open.side_effect=lambda *a,**k:io.BytesIO(b'a'*65)
        for function in [P.digest,R.sha]:
            with self.subTest(function=function.__name__),self.assertRaises(ValueError):function(fake)

    def test_each_incomplete_argument_pair(self):
        for flags in [['--local-install-exe',str(self.cli)],['--local-install-sha256',sha(self.cli.read_bytes())]]:
            with self.assertRaises(ValueError):self.package(flags)
            self.assertFalse(self.out.exists())
    def test_missing_or_malformed_proof_artifact(self):
        for value in [None,{'sha256':sha(self.cli.read_bytes()),'bytes':True},{'sha256':'g'*64,'bytes':512},{'sha256':sha(self.cli.read_bytes()),'bytes':512,'extra':0}]:
            if value is None:self.proof.pop('local_install_artifact',None)
            else:self.proof['local_install_artifact']=value
            self.saveproof()
            with self.assertRaises(ValueError):self.package(['--local-install-exe',str(self.cli),'--local-install-sha256',sha(self.cli.read_bytes())])
            self.assertFalse(self.out.exists())
    def test_wrong_size_sha_and_middle_byte(self):
        original=copy.deepcopy(self.proof)
        for change in ['size','digest','middle']:
            self.proof=copy.deepcopy(original);self.cli.write_bytes(pe_bytes(3))
            if change=='size':self.proof['local_install_artifact']['bytes']=513
            elif change=='digest':self.proof['local_install_artifact']['sha256']='0'*64
            else:b=bytearray(self.cli.read_bytes());b[300]^=1;self.cli.write_bytes(b)
            self.saveproof()
            with self.assertRaises(ValueError):self.package()
            self.assertFalse(self.out.exists())
    def test_reviewer_rejects_changed_member_middle_byte(self):
        self.package()
        def mutate(ms):
            out=[]
            for i,b in ms:
                if i.filename=='dlpad-local-install.exe':b=b[:300]+bytes([b[300]^1])+b[301:]
                out.append((i,b))
            return out
        with self.assertRaises(ValueError):R.review(self.rewrite(mutate),self.expected())
    def test_reviewer_unknown_member_and_bad_metadata(self):
        self.package()
        def unknown(ms):
            i=zipfile.ZipInfo('unknown.exe',(1980,1,1,0,0,0));i.create_system=3;i.external_attr=0o100755<<16;return sorted(ms+[(i,b'unknown')],key=lambda x:x[0].filename)
        with self.assertRaises(ValueError):R.review(self.rewrite(unknown),self.expected())
        def metadata(ms):
            for i,b in ms:
                if i.filename=='dlpad-local-install.exe':i.date_time=(2000,1,1,0,0,0)
            return ms
        with self.assertRaises(ValueError):R.review(self.rewrite(metadata),self.expected())
    def test_reviewer_companion_policy_and_proof_mismatch(self):
        self.package()
        for field,value in [('development_only',False),('installer_complete',True),('role','installer'),('bytes',513),('sha256','0'*64)]:
            def mutation(ms):
                out=[]
                for i,b in ms:
                    if i.filename=='candidate-manifest.json':m=json.loads(b);m['local_install_companion'][field]=value;b=json.dumps(m).encode()
                    out.append((i,b))
                return out
            with self.assertRaises(ValueError):R.review(self.rewrite(mutation),self.expected())
        expected=self.expected();expected['local_install_artifact']['sha256']='0'*64
        with self.assertRaises(ValueError):R.review(self.out,expected)

    def test_combined_budget_exact_boundary_without_large_allocation(self):
        self.package()
        with zipfile.ZipFile(self.out)as z: originals=z.infolist()
        for excess in [0,1]:
            infos=copy.deepcopy(originals)
            for i in infos:
                if i.filename=='dlpad-cpu.exe':i.file_size=R.LIMIT-512
                if i.filename=='dlpad-local-install.exe':i.file_size=512+excess
            archive=mock.MagicMock();archive.__enter__.return_value=archive;archive.infolist.return_value=infos
            marker=RuntimeError('bounded preflight passed; no large bytes read')
            with mock.patch.object(R.zipfile,'ZipFile',return_value=archive),mock.patch.object(R,'member_hash',side_effect=marker)as hashing:
                if excess:
                    with self.assertRaisesRegex(ValueError,'combined'):R.review(self.out,self.expected())
                    hashing.assert_not_called()
                else:
                    with self.assertRaisesRegex(RuntimeError,'bounded preflight passed'):R.review(self.out,self.expected())
                    hashing.assert_called_once()
    def test_missing_cli_member_and_wrong_subsystem(self):
        self.package()
        with self.assertRaises(ValueError):R.review(self.rewrite(lambda ms:[(i,b)for i,b in ms if i.filename!='dlpad-local-install.exe']),self.expected())
        self.out.unlink();self.cli.write_bytes(pe_bytes(2));self.proof['local_install_artifact']['sha256']=sha(self.cli.read_bytes());self.saveproof()
        with self.assertRaises(ValueError):self.package()
        self.assertFalse(self.out.exists())

if __name__=='__main__':unittest.main()
