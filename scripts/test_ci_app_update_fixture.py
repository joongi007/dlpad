import contextlib
import copy
import io
import json
import os
from pathlib import Path
import tempfile
import subprocess
import sys
import types
import unittest
from unittest.mock import patch
import ci_app_update_fixture as fixture
import ci_whole_receipt as receipt


def payload(case='success'):
    row={'success':(1,2,None,'0.1.2',3,1),'stale':(1,2,None,'0.1.2',3,1),'start-fail':(1,2,2,None,5,2),'runtime-crash':(3,2,2,None,5,2),'bad-hash':(0,0,None,None,3,1),'cancel':(0,0,None,None,3,1),'busy':(0,0,None,None,3,1)}[case]
    return dict(case=case,**dict(zip(['generation','highest','failed','active','requests','workers'],row)),preserved=True)

class Gate(io.BytesIO):
    def __init__(self,events):super().__init__();self.events=events
    def write(self,value):self.events.append(('gate',value));return super().write(value)
class Process:
    def __init__(self,output=b'',code=0):
        self.events=[];self.stdin=Gate(self.events);self.stdout=io.BytesIO(output);self.stderr=io.BytesIO();self.returncode=code;self._handle=1
    def poll(self):return self.returncode
    def kill(self):self.events.append('kill');self.returncode=126
    def wait(self,timeout=None):return self.returncode
class Kernel:
    def __init__(self,events):self.events=events
    def CloseHandle(self,handle):self.events.append('closed-job');return True

class Tests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);self.root=Path(self.temp.name);self.folder=self.root/'captures';self.folder.mkdir()
        self.environment=patch.dict(os.environ,{},clear=True);self.environment.start();self.addCleanup(self.environment.stop)
    def write_result(self,value=None,raw=None):
        p=self.folder/'case.stdout';p.write_bytes(raw if raw is not None else b'CASE_PASS '+json.dumps(value).encode()+b'\n');return p
    def capture(self,operation='case-success',process=None,remaining=False):
        process=process or Process(b'CASE_PASS '+json.dumps(payload()).encode()+b'\n')
        kernel=Kernel(process.events)
        def assigned(value):self.assertIs(value,process);process.events.append('assigned-job');return kernel,1
        with patch.object(fixture.subprocess,'Popen',return_value=process),patch.object(fixture.capture,'job_for',side_effect=assigned),patch.object(fixture.capture,'job_active',return_value=0),patch.object(fixture.capture,'job_processes',return_value=[{'pid':42,'membership_confirmed':True}]),contextlib.redirect_stdout(io.StringIO()):
            if remaining:
                with patch.object(fixture.capture,'wait_natural_empty',side_effect=ValueError('owned descendant remains')):value=fixture.run_operation(self.root,self.folder,operation,1,self.root)
            else:value=fixture.run_operation(self.root,self.folder,operation,1,self.root)
        return value,process
    def test_isolated_python_loads_sibling_helpers_for_entry_and_worker(self):
        script=str(Path(fixture.__file__).resolve())
        result=subprocess.run([sys.executable,'-I',script,'--help'],cwd=self.root,stdin=subprocess.DEVNULL,capture_output=True,timeout=10)
        self.assertEqual(result.returncode,0,result.stderr.decode(errors='replace'))
        self.assertIn(b'--owned-parent',result.stdout)
        # Even on a non-Windows host, the worker must reach its platform guard,
        # rather than fail importing the capture helpers before it can be gated.
        result=subprocess.run([sys.executable,'-I',script,'--worker'],cwd=self.root,stdin=subprocess.DEVNULL,capture_output=True,timeout=10)
        self.assertNotEqual(result.returncode,0)
        self.assertNotIn(b'ModuleNotFoundError',result.stderr)
        self.assertIn(b'Windows gated worker arguments',result.stderr)
    def test_fixed_argv_and_no_eighth_whole_operation(self):
        before=copy.deepcopy(receipt.REQUIRED)
        self.assertEqual(fixture.argv_for(self.root,'fixture-build'),['cargo','build','-p','dlpad-engine','--bin','app-update-fixture','--features','app-update-fixture','--locked','--offline'])
        self.assertEqual(fixture.argv_for(self.root,'case-cancel',self.root)[1:],['--case','cancel','--owned-parent',str(self.root)])
        with self.assertRaises(ValueError):fixture.argv_for(self.root,'release')
        with self.assertRaises(ValueError):fixture.argv_for(self.root,'case-anything',self.root)
        with self.assertRaises(ValueError):fixture.argv_for(self.root,'case-success',Path('relative'))
        self.assertEqual(before,receipt.REQUIRED);self.assertEqual(len(before),7)
    def test_all_seven_case_transitions(self):
        for case in fixture.CASES:self.assertEqual(fixture.case_result(self.write_result(payload(case)),case),payload(case))
    def test_failed_wrong_duplicate_and_ambiguous_results_are_rejected(self):
        for key,value in [('preserved',False),('case','busy'),('workers',True),('generation',2**32),('highest',-1),('failed',True),('active',55),('requests',0)]:
            row=payload();row[key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):fixture.case_result(self.write_result(row),'success')
        for raw in [b'',b'CASE_PASS {}\n',b'CASE_PASS {"case":"success","case":"busy"}\n',b'X'*(fixture.MAX_LINE+1),b'CASE_PASS '+json.dumps(payload()).encode()+b'\nCASE_PASS '+json.dumps(payload()).encode()+b'\n']:
            with self.assertRaises((ValueError,UnicodeError)):fixture.case_result(self.write_result(raw=raw),'success')
    def test_gated_original_job_and_pump_produce_natural_zero_receipt(self):
        value,process=self.capture();self.assertTrue(value['passed']);self.assertEqual(process.events[:2],['assigned-job',('gate',b'G')]);self.assertIn('closed-job',process.events)
        self.assertTrue(value['natural_job_empty']);self.assertEqual(value['final_active'],0);self.assertFalse(value['forced_cleanup']);self.assertFalse(value['native_gui_acceptance_verified']);self.assertFalse(value['release_ready'])
        self.assertEqual(value['case_result'],payload());self.assertEqual(json.loads((self.folder/'case-success.json').read_text()),value)
    def test_remaining_owned_descendant_preserves_failure_logs_not_success(self):
        value,_=self.capture(remaining=True);self.assertFalse(value['passed']);self.assertTrue(value['forced_cleanup']);self.assertFalse(value['natural_job_empty'])
        self.assertEqual(value['remaining_job_processes'][0]['pid'],42);self.assertTrue((self.folder/'case-success.stdout').is_file());self.assertTrue((self.folder/'case-success.json').is_file())
        self.assertFalse((self.folder/'whole.json').exists())
    def test_pass_banner_does_not_override_nonzero_exit_or_bad_json(self):
        value,_=self.capture(process=Process(b'CASE_PASS '+json.dumps(payload()).encode()+b'\n',code=1));self.assertFalse(value['passed']);self.assertNotIn('case_result',value)
        self.folder=self.root/'second';self.folder.mkdir();value,_=self.capture(process=Process(b'CASE_PASS {}\n'));self.assertFalse(value['passed']);self.assertIn('failure_kind',value)
    def test_existing_receipt_is_not_overwritten(self):
        record=self.folder/'case-success.json';record.write_bytes(b'original')
        with self.assertRaises(ValueError),patch.object(fixture.subprocess,'Popen') as spawn:fixture.run_operation(self.root,self.folder,'case-success',1,self.root)
        spawn.assert_not_called();self.assertEqual(record.read_bytes(),b'original')
    def make_build(self):
        binary=fixture.binary_path(self.root);binary.parent.mkdir(parents=True);binary.write_bytes(b'synthetic fixture image')
        bindings={}
        for name in fixture.BUILD:
            value,_=self.capture(operation=name,process=Process());self.assertTrue(value['passed']);bindings[name]=receipt.pin(self.folder/(name+'.json'),65536)
        receipt.publish(self.folder/'build.json',dict(schema=1,kind='windows-app-update-fixture-build',operations=list(fixture.BUILD),operation_bindings=bindings,binary={'path':str(binary),**receipt.pin(binary,128*1024*1024)},release_ready=False,native_gui_acceptance_verified=False))
        return binary
    def test_run_rejects_mutated_executable_or_original_build_logs_before_spawn(self):
        binary=self.make_build();binary.write_bytes(b'modified')
        with patch.object(fixture,'run_operation') as operation,self.assertRaises(ValueError):fixture.run(self.root,self.folder,self.root,10)
        operation.assert_not_called()
        binary.write_bytes(b'synthetic fixture image');(self.folder/'fixture-clippy.stdout').write_bytes(b'changed')
        with patch.object(fixture,'run_operation') as operation,self.assertRaises(ValueError):fixture.run(self.root,self.folder,self.root,10)
        operation.assert_not_called()
    def test_all_case_names_are_required_and_one_failure_prevents_whole(self):
        self.make_build();called=[]
        def ok(root,folder,operation,seconds,parent):called.append(operation);value={'name':operation,'passed':True};receipt.publish(folder/(operation+'.json'),value);return value
        with patch.object(fixture,'run_operation',side_effect=ok):self.assertTrue(fixture.run(self.root,self.folder,self.root,10))
        self.assertEqual(called,['case-'+case for case in fixture.CASES]);whole=json.loads((self.folder/'whole.json').read_text());self.assertEqual(whole['cases'],list(fixture.CASES));self.assertFalse(whole['release_ready'])
        (self.folder/'whole.json').unlink();called.clear()
        for file in self.folder.glob('case-*.json'):file.unlink()
        def fail(root,folder,operation,seconds,parent):called.append(operation);return {'name':operation,'passed':operation!='case-start-fail'}
        with patch.object(fixture,'run_operation',side_effect=fail):self.assertFalse(fixture.run(self.root,self.folder,self.root,10))
        self.assertEqual(called,['case-success','case-start-fail']);self.assertFalse((self.folder/'whole.json').exists())
    def test_target_selection_uses_environment_not_a_user_home_path(self):
        with patch.dict(os.environ,{'CARGO_TARGET_DIR':str(self.root/'owned-target'),'CARGO_BUILD_TARGET':'x86_64-pc-windows-msvc'}):self.assertEqual(fixture.binary_path(self.root),self.root/'owned-target/x86_64-pc-windows-msvc/debug/app-update-fixture.exe')
        with patch.dict(os.environ,{'CARGO_BUILD_TARGET':'x86_64-unknown-linux-gnu'}):
            with self.assertRaises(ValueError):fixture.binary_path(self.root)

if __name__=='__main__':unittest.main()
