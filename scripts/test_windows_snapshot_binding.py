"""Synthetic fixtures call the actual source snapshot; no native/Cargo/tool calls."""
from contextlib import contextmanager
import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest
sys.dont_write_bytecode = True
MODULE = None
MODULE_PATH = None
SQL = 'crates/engine/src/tools/staging_schema.sql'

@contextmanager
def synthetic_root():
    previous = MODULE.ROOT
    with tempfile.TemporaryDirectory(prefix='dlpad-snapshot-SQL-') as directory:
        root = Path(directory)
        for name in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml',
                     'scripts/windows_cli_host.cs', 'scripts/windows_current_workspace_probe.py',
                     '.local-tools/windows/windows-cli-host.exe', 'crates/engine/Cargo.toml',
                     'crates/engine/src/tools.rs', 'crates/app/assets/test-resource.bin',
                     'vendor/test-vendor/resource.txt', SQL]:
            p = root/name
            p.parent.mkdir(parents=True, exist_ok=True)
            if name == SQL:
                p.write_bytes(b'CREATE TABLE fixture_a(id INTEGER);\n')
            elif name == 'scripts/windows_current_workspace_probe.py':
                p.write_bytes(MODULE_PATH.read_bytes())
            else:
                p.write_bytes(b'synthetic-inert-not-executed\n')
        MODULE.ROOT = root
        try:
            yield root
        finally:
            MODULE.ROOT = previous
    assert MODULE.ROOT == previous

class CompiledResourceBinding(unittest.TestCase):
    def test_included_SQL_mutation_changes_actual_snapshot_and_restore_matches(self):
        with synthetic_root() as root:
            before = MODULE.snapshot()
            sql = root/SQL
            original = sql.read_bytes()
            sql.write_bytes(b'CREATE TABLE fixture_b(id INTEGER, marker TEXT);\n')
            changed = MODULE.snapshot()
            self.assertNotEqual(before, changed, 'actual source snapshot must change for compiled SQL mutation')
            self.assertIn(SQL, before)
            self.assertIn(SQL, changed)
            self.assertNotEqual(before[SQL], changed[SQL])
            self.assertEqual({k:v for k,v in before.items() if k != SQL},
                             {k:v for k,v in changed.items() if k != SQL},
                             'unchanged Rust/Cargo/host/assets/vendor bindings remain exact')
            sql.write_bytes(original)
            self.assertEqual(MODULE.snapshot(), before, 'restored exact SQL restores source snapshot')

    def test_missing_required_compiled_SQL_refuses_actual_snapshot(self):
        with synthetic_root() as root:
            (root/SQL).unlink()
            with self.assertRaises(FileNotFoundError):
                MODULE.snapshot()

    def test_existing_assets_vendor_Rust_mutations_still_bind_actual_snapshot(self):
        for name in ['crates/engine/src/tools.rs', 'crates/app/assets/test-resource.bin',
                     'vendor/test-vendor/resource.txt']:
            with self.subTest(name=name), synthetic_root() as root:
                before = MODULE.snapshot()
                (root/name).write_bytes(b'synthetic-source-changed\n')
                changed = MODULE.snapshot()
                self.assertNotEqual(before, changed)
                self.assertNotEqual(before[name], changed[name])

MODULE_PATH = Path(__file__).resolve().with_name('windows_current_workspace_probe.py')
spec = importlib.util.spec_from_file_location('actual_snapshot_under_test', MODULE_PATH)
MODULE = importlib.util.module_from_spec(spec)
spec.loader.exec_module(MODULE)

if __name__ == '__main__':
    unittest.main()
