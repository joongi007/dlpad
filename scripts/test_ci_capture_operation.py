import io,tempfile,unittest
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
if __name__=='__main__':unittest.main()
