import copy,json,os,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
import ci_whole_orchestrate as o
import ci_whole_receipt as r

def symlink_or_skip(test,path,target,directory=False):
    try:path.symlink_to(target,target_is_directory=directory)
    except OSError as error:
        if os.name=='nt' and getattr(error,'winerror',None)==1314:
            test.skipTest('Windows symlink creation privilege unavailable; no system changes')
        raise

class Tests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);self.root=Path(self.temp.name)
        for name in ['Cargo.lock','rust-toolchain.toml','source.rs']:(self.root/name).write_bytes(name.encode())
        self.raw=b'Cargo.lock\0rust-toolchain.toml\0source.rs\0';self.identity=dict(commit='a'*40,run_id='123',run_attempt='1',repository='owned/repo')
        self.calls=[]
        def git(root,argv):
            self.calls.append(argv)
            if argv==o.GIT_ARGS[0]:return self.raw
            if argv==o.GIT_ARGS[1]:return (self.identity['commit']+'\n').encode()
            return b''
        self.git=git;o.before(self.root,'captures',self.identity,self.git);self.folder=self.root/'captures'
        bins=self.root/'target/release';bins.mkdir(parents=True)
        for name in r.BINS:(bins/name).write_bytes(name.encode())
        for name,argv in r.REQUIRED.items():
            for kind in ['stdout','stderr']:(self.folder/(name+'.'+kind)).write_bytes(b'raw\xff\x00' if kind=='stdout' else b'')
            r.publish(self.folder/(name+'.json'),dict(name=name,argv=argv,exit_code=0,stdout='captures/'+name+'.stdout',stderr='captures/'+name+'.stderr'))
            life=dict(schema=1,kind='windows-ci-operation-lifecycle',name=name,native_run_performed=True,original_worker_job_assigned=True,natural_job_empty=True,job_closed=True,forced_cleanup=False,reader_drained=True,remote_execution_authenticated=False,native_gui_acceptance_verified=False,legal_coverage_complete=False,release_ready=False,worker_exit_code=0,final_active=0,accounting_queries=1,passed=True,streams=[dict(bytes=5,drained=True),dict(bytes=0,drained=True)])
            r.publish(self.folder/(name+'.lifecycle.json'),life)
    def run_case(self):return o.after(self.root,'captures',self.identity,self.git)
    def change(self,name,key,value):
        file=self.folder/name;data=json.loads(file.read_bytes());data[key]=value;file.write_text(json.dumps(data))
    def reject_case(self):
        with self.assertRaises((ValueError,OSError)):self.run_case()
        self.assertFalse((self.folder/'whole.json').exists())
        self.assertTrue((self.folder/'fmt.stdout').exists())
    def test_exact_scope_and_actual_before_after_inventory(self):
        value=self.run_case();self.assertEqual(set(value['capture_lifecycle_bindings']),set(r.REQUIRED))
        self.assertEqual(self.calls.count(o.GIT_ARGS[0]),6)
        for key in ['release_ready','remote_execution_authenticated','legal_coverage_complete']:self.assertIs(value[key],False)
        self.assertFalse(value['orchestration_scope']['graph_integration_complete'])
    def test_changed_source_and_lock(self):
        for name in ['source.rs','Cargo.lock']:
            original=(self.root/name).read_bytes();(self.root/name).write_bytes(b'changed');self.reject_case();(self.root/name).write_bytes(original)
    def test_changed_git_inventory(self):self.raw+=b'other.rs\0';(self.root/'other.rs').write_bytes(b'x');self.reject_case()
    def test_claimed_identity_mismatch(self):
        wrong=copy.deepcopy(self.identity);wrong['commit']='b'*40
        with self.assertRaises(ValueError):o.after(self.root,'captures',wrong,self.git)
    def test_missing_extra_operation(self):
        file=self.folder/'test.json';raw=file.read_bytes();file.unlink();self.reject_case();file.write_bytes(raw)
        (self.folder/'extra.json').write_bytes(b'{}');self.reject_case()
    def test_failed_forced_unconfirmed_lifecycle(self):
        file=self.folder/'test.lifecycle.json';raw=file.read_bytes()
        for key,value in [('passed',False),('forced_cleanup',True),('reader_drained',False),('worker_exit_code',False),('final_active',1),('natural_job_empty',False)]:
            self.change(file.name,key,value);self.reject_case();file.write_bytes(raw)
    def test_stream_bytes_mismatch(self):
        self.change('fmt.lifecycle.json','streams',[dict(bytes=4,drained=True),dict(bytes=0,drained=True)]);self.reject_case()
    def test_wrong_argv_and_boolean_exit(self):
        file=self.folder/'fmt.json';raw=file.read_bytes()
        for key,value in [('argv',['cargo','test']),('exit_code',False)]:self.change(file.name,key,value);self.reject_case();file.write_bytes(raw)
    def test_duplicate_json(self):
        file=self.folder/'fmt.json';file.write_bytes(b'{"name":"fmt","name":"fmt"}');self.reject_case()
    def test_existing_outputs_preserved(self):
        file=self.folder/'after.json';file.write_bytes(b'original');self.reject_case();self.assertEqual(file.read_bytes(),b'original')
        with self.assertRaises(ValueError):o.before(self.root,'captures',self.identity,self.git)
    def test_log_hardlink_and_path_alias(self):
        file=self.folder/'test.stdout';file.unlink();os.link(self.folder/'fmt.stdout',file);self.reject_case()
    def test_tracked_alias_and_reparse(self):
        self.raw=b'Cargo.lock\0rust-toolchain.toml\0source.rs\0SOURCE.rs\0';(self.root/'SOURCE.rs').write_bytes(b'x')
        with self.assertRaises(ValueError):o.inventory(self.root,self.identity,self.git)
        self.raw=b'Cargo.lock\0rust-toolchain.toml\0source.rs\0';(self.root/'source.rs').unlink();symlink_or_skip(self,self.root/'source.rs',self.root/'Cargo.lock')
        with self.assertRaises(ValueError):o.inventory(self.root,self.identity,self.git)
    def test_stale_before_binary_rejected_and_preserved(self):
        files={name:(self.root/'target/release'/name).read_bytes() for name in r.BINS}
        with self.assertRaisesRegex(ValueError,'stale expected binary'):o.before(self.root,'fresh-captures',self.identity,self.git)
        self.assertFalse((self.root/'fresh-captures').exists())
        for name,raw in files.items():self.assertEqual((self.root/'target/release'/name).read_bytes(),raw)
    def test_dirty_git(self):
        def dirty(root,argv):return b'unexpected' if argv==o.GIT_ARGS[2] else self.git(root,argv)
        with self.assertRaises(ValueError):o.after(self.root,'captures',self.identity,dirty)
    def test_changed_lifecycle_during_binding(self):
        original=r.construct
        def changed(*args):
            value=original(*args);self.change('test.lifecycle.json','passed',False);return value
        with patch.object(r,'construct',side_effect=changed):self.reject_case()
class CleanupTests(unittest.TestCase):
    def test_job_close_failure_still_kills_waits_and_joins_all_readers(self):
        events=[]
        class Kernel:
            def CloseHandle(self,job):events.append('close');return False
        class Process:
            def poll(self):return None
            def kill(self):events.append('kill')
            def wait(self,timeout):events.append('wait')
        class Reader:
            def join(self,timeout):events.append('join')
            def is_alive(self):return False
        with self.assertRaisesRegex(ValueError,'Job close failure'):o.cleanup_git(Kernel(),123,Process(),[Reader(),Reader()])
        self.assertEqual(events,['close','kill','wait','join','join'])
    def test_timeout_or_reader_failure_runs_owned_cleanup(self):
        for failure in [TimeoutError('deadline'),ValueError('reader limit')]:
            events=[]
            class Kernel:
                def CloseHandle(self,job):events.append('close');return True
            class Process:
                def poll(self):return None
                def kill(self):events.append('kill')
                def wait(self,timeout):events.append('wait')
            class Reader:
                def join(self,timeout):events.append('join')
                def is_alive(self):return False
            with self.subTest(failure=type(failure).__name__),self.assertRaises(type(failure)):
                try:raise failure
                finally:o.cleanup_git(Kernel(),123,Process(),[Reader()])
            self.assertEqual(events,['close','kill','wait','join'])
    def test_kill_failure_does_not_skip_wait_or_later_reader(self):
        events=[]
        class Process:
            def poll(self):return None
            def kill(self):events.append('kill');raise OSError('kill')
            def wait(self,timeout):events.append('wait')
        class Reader:
            def join(self,timeout):events.append('join')
            def is_alive(self):return False
        with self.assertRaises(OSError):o.cleanup_git(None,None,Process(),[Reader(),Reader()])
        self.assertEqual(events,['kill','wait','join','join'])
if __name__=='__main__':unittest.main()
