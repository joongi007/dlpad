import contextlib,ctypes,io,json,os,subprocess,sys,tempfile,time,types,unittest
from pathlib import Path
from unittest.mock import Mock,patch
import ci_capture_operation as capture

class CaptureFailures(unittest.TestCase):
    def test_raw_non_utf8_and_nul_are_preserved_exactly(self):
        raw=b'\xff\xfe\x00Korean\r\n\x81'*10000
        with tempfile.TemporaryFile() as f:
            r={};capture.pump(io.BytesIO(raw),f,r);f.seek(0)
            self.assertEqual(f.read(),raw);self.assertTrue(r['drained'])
    def test_overflow_is_rejected_before_excess_bytes_are_written(self):
        with tempfile.TemporaryFile() as f:
            r={};capture.pump(io.BytesIO(b'x'*(capture.LOG_LIMIT+1)),f,r)
            self.assertFalse(r['drained']);self.assertEqual(r['error'],'ValueError')
            self.assertEqual(f.tell(),capture.LOG_LIMIT)
    def test_reader_error_is_not_an_eof_ack(self):
        class Broken:
            def read(self,_):raise OSError('injected read failure')
            def close(self):pass
        with tempfile.TemporaryFile() as f:
            r={};capture.pump(Broken(),f,r);self.assertFalse(r['drained'])
    def test_leader_exit_does_not_replace_descendant_empty_ack(self):
        seq=iter([2,1,0]);ack=capture.wait_natural_empty(lambda:next(seq),5,now=lambda:0,pause=lambda _:None)
        self.assertEqual(ack,{'active':0,'queries':3})
    def test_live_descendant_deadline_is_rejected(self):
        with self.assertRaisesRegex(ValueError,'descendants'):
            capture.wait_natural_empty(lambda:1,5,now=lambda:5,pause=lambda _:None)
    def test_query_failure_is_not_zero(self):
        def fail():raise OSError('actual query failure equivalent boundary')
        with self.assertRaises(OSError):capture.wait_natural_empty(fail,5)
    def test_negative_accounting_rejected(self):
        with self.assertRaises(ValueError):capture.wait_natural_empty(lambda:-1,5)
class CaptureMainFailures(unittest.TestCase):
    def test_successful_command_with_live_descendant_reports_real_failure_and_log_tail(self):
        with tempfile.TemporaryDirectory(prefix='dlpad-ci-capture-failure-') as directory:
            root=Path(directory);(root/'ci-captures').mkdir()
            process=Mock();process.stdin=io.BytesIO();process.stdout=io.BytesIO()
            process.stderr=io.BytesIO(b'Finished dev profile successfully\n');process.returncode=0;process.poll.return_value=0
            kernel=Mock();kernel.CloseHandle.return_value=True
            output=io.StringIO()
            with patch.object(capture,'os',types.SimpleNamespace(name='nt',fsync=os.fsync)), patch.object(capture.sys,'argv',['capture','--root',str(root),'--directory','ci-captures','--name','clippy']), patch.object(capture.subprocess,'Popen',return_value=process), patch.object(capture,'job_for',return_value=(kernel,1)), patch.object(capture,'wait_natural_empty',side_effect=ValueError('original Job descendants did not naturally terminate')), patch.object(capture,'job_processes',return_value=[{'pid':42,'image':'fixture-helper.exe','membership_confirmed':True}],create=True), contextlib.redirect_stdout(output):
                code=capture.main()
            self.assertEqual(code,125)
            facts=json.loads((root/'ci-captures/clippy.lifecycle.json').read_text())
            self.assertEqual(facts['worker_exit_code'],0)
            self.assertFalse(facts['natural_job_empty']);self.assertTrue(facts['forced_cleanup'])
            self.assertFalse(facts['passed']);self.assertTrue(facts['reader_drained'])
            self.assertIn('original Job descendants did not naturally terminate',output.getvalue())
            self.assertIn('Finished dev profile successfully',output.getvalue())
            self.assertIn('fixture-helper.exe',output.getvalue())
            self.assertEqual(facts['failure_message'],'original Job descendants did not naturally terminate')
            self.assertFalse((root/'ci-captures/clippy.json').exists())

class ReportBounds(unittest.TestCase):
    def test_error_tail_is_bounded_prefixed_and_original_bytes_are_unchanged(self):
        with tempfile.TemporaryDirectory()as directory:
            out=Path(directory)/'stdout';err=Path(directory)/'stderr'
            raw=b'not-in-tail\n'+b'x'*50000+b'\n::error::synthetic tool text\n'
            out.write_bytes(raw);err.write_bytes(b'')
            facts={'worker_exit_code':101,'natural_job_empty':True,'reader_drained':True,'forced_cleanup':False}
            console=io.StringIO()
            with contextlib.redirect_stdout(console):capture.report('clippy',facts,None,out,err)
            text=console.getvalue();self.assertLess(len(text),18000)
            self.assertNotIn('not-in-tail',text)
            self.assertIn('CI clippy stdout | ::error::synthetic tool text',text)
            self.assertFalse(any(line.startswith('::')for line in text.splitlines()))
            self.assertEqual(out.read_bytes(),raw)

@unittest.skipUnless(os.name=='nt','actual Windows Job APIs; no mocked native success')
class NativeJobDiagnostics(unittest.TestCase):
    def test_original_job_identifies_live_descendant_after_leader_exit_and_closes_it(self):
        process=None;kernel=None;job=None;descendant=None
        code="import subprocess,sys;sys.stdin.buffer.read(1);subprocess.Popen([sys.executable,'-c','import time;time.sleep(60)'],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)"
        try:
            process=subprocess.Popen([sys.executable,'-c',code],stdin=subprocess.PIPE,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,creationflags=subprocess.CREATE_NO_WINDOW)
            kernel,job=capture.job_for(process)
            process.stdin.write(b'x');process.stdin.close()
            self.assertEqual(process.wait(timeout=15),0)
            rows=capture.job_processes(kernel,job)
            owned=[row for row in rows if row.get('membership_confirmed')]
            expected=Path(sys.executable).name.lower()
            children=[row for row in owned if row.get('image','').lower()==expected]
            self.assertEqual(len(children),1,rows)
            self.assertTrue(all(row.get('image','').lower()in(expected,'conhost.exe')for row in owned),rows)
            print('Native original Job residuals: '+json.dumps(rows),flush=True)
            descendant=kernel.OpenProcess(0x100000,False,children[0]['pid'])
            self.assertTrue(descendant)
            with self.assertRaisesRegex(ValueError,'descendants'):
                capture.wait_natural_empty(lambda:capture.job_active(kernel,job),time.monotonic()+.1)
        finally:
            if job:self.assertTrue(kernel.CloseHandle(job))
            if process:
                if process.poll()is None:process.kill()
                process.wait(timeout=15)
            if descendant:
                kernel.WaitForSingleObject.argtypes=[ctypes.c_void_p,ctypes.c_uint32]
                kernel.WaitForSingleObject.restype=ctypes.c_uint32
                try:self.assertEqual(kernel.WaitForSingleObject(descendant,10000),0)
                finally:self.assertTrue(kernel.CloseHandle(descendant))

if __name__=='__main__':unittest.main()
