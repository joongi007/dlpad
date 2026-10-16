"""Pure /tmp fixture tests; no native processes or runner main invocation."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

SOURCE=Path(__file__).resolve().with_name('windows_library_watch_native_runner.py')
spec=importlib.util.spec_from_file_location('watch_runner_under_test',SOURCE)
runner=importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)

class Helpers(unittest.TestCase):
    def test_plain_rejects_traversal_relative_and_symlink_ancestor(self):
        with tempfile.TemporaryDirectory(prefix='dlpad-watch-pure-') as raw:
            root=Path(raw)
            with self.assertRaises(ValueError):runner.plain(Path('relative'))
            with self.assertRaises(ValueError):runner.plain(root/'..'/'outside')
            real=root/'real';real.mkdir();link=root/'alias';link.symlink_to(real,target_is_directory=True)
            with self.assertRaises(ValueError):runner.plain(link/'child')
            self.assertEqual(runner.plain(real/'fresh'),real/'fresh')
    def test_copy_create_new_preserves_existing_foreign_bytes(self):
        with tempfile.TemporaryDirectory(prefix='dlpad-watch-pure-') as raw:
            root=Path(raw);source=root/'source';target=root/'foreign'
            source.write_bytes(b'input');target.write_bytes(b'foreign-sentinel')
            with self.assertRaises(FileExistsError):runner.copy_new(source,target,64)
            self.assertEqual(target.read_bytes(),b'foreign-sentinel')
            self.assertEqual(source.read_bytes(),b'input')
    def test_copy_exact_bound_and_oversize_preserve_source(self):
        with tempfile.TemporaryDirectory(prefix='dlpad-watch-pure-') as raw:
            root=Path(raw);source=root/'source';source.write_bytes(b'abcd')
            runner.copy_new(source,root/'exact',4)
            self.assertEqual((root/'exact').read_bytes(),b'abcd')
            with self.assertRaises(ValueError):runner.copy_new(source,root/'partial',3)
            self.assertEqual(source.read_bytes(),b'abcd')
            # Failure output is owned partial; helper promises no atomic cleanup.
            self.assertTrue((root/'partial').exists())
            self.assertLessEqual((root/'partial').stat().st_size,3)
    def test_read_json_bom_exact_bound_and_oversize(self):
        with tempfile.TemporaryDirectory(prefix='dlpad-watch-pure-') as raw:
            p=Path(raw)/'record';data=b'\xef\xbb\xbf{"safe":true}';p.write_bytes(data)
            self.assertEqual(runner.read_json(p,len(data)),{'safe':True})
            with self.assertRaises(ValueError):runner.read_json(p,len(data)-1)
            self.assertEqual(p.read_bytes(),data)
    def test_read_json_malformed_and_invalid_utf8_preserve_bytes(self):
        with tempfile.TemporaryDirectory(prefix='dlpad-watch-pure-') as raw:
            p=Path(raw)/'record'
            for data in [b'{',b'\xff']:
                p.write_bytes(data)
                with self.assertRaises((ValueError,UnicodeError)):runner.read_json(p,64)
                self.assertEqual(p.read_bytes(),data)

if __name__=='__main__':unittest.main()
