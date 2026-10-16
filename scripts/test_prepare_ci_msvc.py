import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import prepare_ci_msvc as tool


class PrivateCompiler(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / 'installed'
        self.source.mkdir()
        for name in (*tool.REQUIRED, 'vctip.exe', 'compiler.dll'):
            (self.source / name).write_bytes(name.encode() + b'\x00\xff')
        (self.source / '1033').mkdir()
        (self.source / '1033/messages.dll').write_bytes(b'resources')
        self.output = self.root / 'private'

    def test_exact_copy_preserves_original_and_omits_optional_helper(self):
        original = {p.relative_to(self.source): p.read_bytes() for p in self.source.rglob('*') if p.is_file()}
        result = tool.prepare(self.source, self.output)
        self.assertTrue((self.source / 'vctip.exe').is_file())
        self.assertFalse((self.output / 'vctip.exe').exists())
        for row in result['files']:
            raw = original[Path(row['path'])]
            self.assertEqual((self.output / row['path']).read_bytes(), raw)
            self.assertEqual(row['sha256'], hashlib.sha256(raw).hexdigest())
        self.assertEqual(original, {p.relative_to(self.source): p.read_bytes() for p in self.source.rglob('*') if p.is_file()})

    def test_existing_destination_is_preserved(self):
        self.output.mkdir()
        (self.output / 'user-data').write_bytes(b'preserve')
        with self.assertRaises(FileExistsError):
            tool.prepare(self.source, self.output)
        self.assertEqual((self.output / 'user-data').read_bytes(), b'preserve')

    def test_missing_compiler_is_rejected_before_copy(self):
        (self.source / 'cl.exe').unlink()
        with self.assertRaisesRegex(ValueError, 'missing'):
            tool.prepare(self.source, self.output)
        self.assertFalse(self.output.exists())

    def test_bounds_rejected_before_copy(self):
        for setting, value in [('MAX_FILES', 1), ('MAX_FILE', 1), ('MAX_TOTAL', 1)]:
            with self.subTest(setting=setting), patch.object(tool, setting, value):
                with self.assertRaisesRegex(ValueError, 'bounds'):
                    tool.prepare(self.source, self.output)
                self.assertFalse(self.output.exists())

    def test_nested_destination_rejected(self):
        with self.assertRaisesRegex(ValueError, 'independent'):
            tool.prepare(self.source, self.source / 'private')

    @unittest.skipIf(__import__('os').name == 'nt', 'symlink creation may require Windows privilege')
    def test_link_rejected_without_following_external_source(self):
        (self.source / 'linked').symlink_to(self.root, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, 'link'):
            tool.prepare(self.source, self.output)
        self.assertFalse(self.output.exists())


if __name__ == '__main__':
    unittest.main()
