import copy
import tempfile
import unittest
from pathlib import Path
from ci_whole_receipt import snapshot,construct,publish,REQUIRED,BINS
class Tests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);self.root=Path(self.temp.name)
        for name in ['Cargo.lock','rust-toolchain.toml','src.rs']:(self.root/name).write_bytes(name.encode())
        self.before=snapshot(self.root,b'Cargo.lock\0rust-toolchain.toml\0src.rs\0')
        (self.root/'logs').mkdir();(self.root/'bins').mkdir()
        self.records=[]
        for name,argv in REQUIRED.items():
            for kind in ['stdout','stderr']:(self.root/'logs'/f'{name}-{kind}.log').write_bytes(b'actual bounded fixture log')
            self.records.append(dict(name=name,argv=argv,exit_code=0,stdout=f'logs/{name}-stdout.log',stderr=f'logs/{name}-stderr.log'))
        for name in BINS:(self.root/'bins'/name).write_bytes(b'owned synthetic binary bytes')
        self.identity=dict(commit='a'*40,run_id='123',run_attempt='1',repository='owned/repo')
    def run_case(self):return construct(self.root,self.before,self.before,self.records,self.identity,'bins')
    def test_exact_pins_and_no_native_remote_readiness_claim(self):
        result=self.run_case();self.assertEqual(set(result['artifacts']),set(BINS))
        for name in ['clippy', 'test']:
            argv = result['operations'][name]['argv']
            self.assertEqual(argv[argv.index('--features') + 1], 'dlpad/cpu')
        for key in ['release_ready','remote_execution_authenticated','private_job_cleanup_verified','native_gui_acceptance_verified','legal_coverage_complete']:self.assertIs(result[key],False)
    def test_default_feature_only_clippy_and_test_records_are_rejected(self):
        # A successful default-feature process is insufficient for CPU UI coverage.
        for name in ['clippy', 'test']:
            original = copy.deepcopy(self.records)
            record = next(v for v in self.records if v['name'] == name)
            index = record['argv'].index('--features')
            record['argv'] = record['argv'][:index] + record['argv'][index + 2:]
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, 'exact successful command'):
                self.run_case()
            self.records = original

    def test_other_renderer_or_unlocked_cpu_records_are_rejected(self):
        for name in ['clippy', 'test']:
            for change in ['wrong-feature', 'unlocked']:
                original = copy.deepcopy(self.records)
                record = next(v for v in self.records if v['name'] == name)
                record['argv'] = list(record['argv'])
                if change == 'wrong-feature':
                    record['argv'][record['argv'].index('--features') + 1] = 'alternative'
                else:
                    record['argv'].remove('--locked')
                with self.subTest(name=name, change=change), self.assertRaises(ValueError):
                    self.run_case()
                self.records = original

    def test_actual_source_change_rejected_even_when_supplied_receipts_equal(self):
        (self.root/'src.rs').write_bytes(b'changed')
        with self.assertRaises(ValueError):self.run_case()
    def test_failed_or_wrong_operation_rejected(self):
        for key,value in [('exit_code',1),('exit_code',False),('argv',['cargo','test','--ignored'])]:
            saved=copy.deepcopy(self.records);self.records[0][key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):self.run_case()
            self.records=saved
    def test_log_changed_produces_distinct_bound_pin(self):
        before=self.run_case()['operations']['fmt']['logs']['stdout']['sha256']
        (self.root/'logs/fmt-stdout.log').write_bytes(b'different raw output')
        self.assertNotEqual(before,self.run_case()['operations']['fmt']['logs']['stdout']['sha256'])
    def test_symlink_log_and_output_rejected_without_target_write(self):
        original=self.root/'logs/fmt-stdout.log';original.unlink()
        try:original.symlink_to(self.root/'src.rs')
        except OSError as error:
            if getattr(error,'winerror',None)==1314:
                self.skipTest('Windows symlink creation privilege unavailable; no system changes')
            raise
        with self.assertRaises(ValueError):self.run_case()
        alias=self.root/'alias';alias.symlink_to(self.root/'bins',target_is_directory=True)
        with self.assertRaises(ValueError):publish(alias/'proof.json',{})
        self.assertFalse((self.root/'bins/proof.json').exists())
    def test_duplicate_or_traversal_tracked_path_rejected(self):
        for raw in [b'Cargo.lock\0Cargo.lock\0',b'../Cargo.lock\0',b'Cargo.lock',b'Cargo.lock\0rust-toolchain.toml\0\xff\0']:
            with self.subTest(raw=raw),self.assertRaises(ValueError):snapshot(self.root,raw)
    def test_existing_output_preserved(self):
        output=self.root/'proof.json';output.write_bytes(b'original')
        with self.assertRaises(ValueError):publish(output,{})
        self.assertEqual(output.read_bytes(),b'original')

    def test_windows_components_rejected(self):
        from ci_whole_receipt import relative_parts
        for value in ['logs/a:stream','logs/C:/out','logs/CON.txt','logs/LPT1','logs/name.','logs/name ','logs/a\x00b']:
            with self.subTest(value=value),self.assertRaises(ValueError):relative_parts(value)
    def test_duplicate_log_paths_and_hardlinks_rejected(self):
        import os
        self.records[1]['stdout']=self.records[0]['stdout']
        with self.assertRaises(ValueError):self.run_case()
        self.records[1]['stdout']='logs/clippy-stdout.log'
        alias=self.root/self.records[1]['stdout'];alias.unlink();os.link(self.root/self.records[0]['stdout'],alias)
        with self.assertRaises(ValueError):self.run_case()
    def test_binary_hardlink_rejected(self):
        import os
        target=self.root/'bins'/BINS[1];target.unlink();os.link(self.root/'bins'/BINS[0],target)
        with self.assertRaises(ValueError):self.run_case()
    def test_reparse_attribute_rejected(self):
        from unittest.mock import patch
        from types import SimpleNamespace
        from ci_whole_receipt import no_reparse
        with patch.object(Path,'lstat',return_value=SimpleNamespace(st_mode=0o100600,st_file_attributes=0x400)):
            with self.assertRaises(ValueError):no_reparse(self.root/'anything')
    def test_changed_open_file_rejected(self):
        from unittest.mock import patch
        import ci_whole_receipt as c
        original=c.os.fstat;calls=0
        def changed(fd):
            nonlocal calls
            value=original(fd);calls+=1
            if calls==2:
                from types import SimpleNamespace
                return SimpleNamespace(st_dev=value.st_dev,st_ino=value.st_ino,st_size=value.st_size+1,st_mtime_ns=value.st_mtime_ns,st_ctime_ns=value.st_ctime_ns)
            return value
        with patch.object(c.os,'fstat',side_effect=changed):
            with self.assertRaises(ValueError):c.pin(self.root/'src.rs',1024)

    def test_zero_native_file_id_is_rejected_without_stat_fallback(self):
        import ctypes
        import ci_whole_receipt as c
        from unittest.mock import patch
        class Fn:
            def __call__(self, *args): return 1
        class Kernel:
            GetFileInformationByHandleEx = Fn()
        # Actual production reader reaches its validation after three successful
        # native-info calls; all-zero FileIdInfo must not become an identity.
        with patch.object(ctypes, 'WinDLL', return_value=Kernel(), create=True):
            with self.assertRaisesRegex(ValueError, 'zero native file ID'):
                c.windows_handle_state(123)

    def test_native_identity_preserves_full_128_bits_and_volume(self):
        import ctypes
        import ci_whole_receipt as c
        from unittest.mock import patch
        class Fn:
            def __call__(self, handle, kind, pointer, size):
                if kind == 18:
                    pointer._obj.volume = 0xfedcba9876543210
                    for index in range(16): pointer._obj.file_id[index] = index + 1
                return 1
        class Kernel:
            GetFileInformationByHandleEx = Fn()
        with patch.object(ctypes, 'WinDLL', return_value=Kernel(), create=True):
            identity = c.windows_handle_state(123)[0]
        self.assertEqual(identity, (0xfedcba9876543210, bytes(range(1, 17))))

    def test_binary_replaced_after_hash_is_rejected_and_both_objects_preserved(self):
        import ci_whole_receipt as c
        from unittest.mock import patch
        target = self.root/'bins'/BINS[0]
        original_bytes = target.read_bytes()
        original_pin = c.pin
        replaced = False
        def replace_after_hash(file, limit):
            nonlocal replaced
            value = original_pin(file, limit)
            if file == target and not replaced:
                target.rename(target.with_suffix('.original'))
                with target.open('xb') as fresh: fresh.write(original_bytes)
                replaced = True
            return value
        with patch.object(c, 'pin', side_effect=replace_after_hash):
            with self.assertRaisesRegex(ValueError, 'binary.*identity'):
                self.run_case()
        self.assertEqual(target.read_bytes(), original_bytes)
        self.assertEqual(target.with_suffix('.original').read_bytes(), original_bytes)

    def test_binary_replaced_during_final_source_check_is_rejected(self):
        import ci_whole_receipt as c
        from unittest.mock import patch
        target = self.root/'bins'/BINS[0]
        source = self.root/'src.rs'
        original_bytes = target.read_bytes()
        original_pin = c.pin
        source_reads = 0
        def replace_during_final_source(file, limit):
            nonlocal source_reads
            value = original_pin(file, limit)
            if file == source:
                source_reads += 1
                if source_reads == 2:
                    target.rename(target.with_suffix('.original'))
                    with target.open('xb') as fresh: fresh.write(original_bytes)
            return value
        with patch.object(c, 'pin', side_effect=replace_during_final_source):
            with self.assertRaisesRegex(ValueError, 'binary.*identity'):
                self.run_case()
        self.assertEqual(target.read_bytes(), original_bytes)
        self.assertEqual(target.with_suffix('.original').read_bytes(), original_bytes)

if __name__=='__main__':unittest.main()
