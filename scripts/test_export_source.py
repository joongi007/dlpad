from pathlib import Path
import tempfile
import unittest
import hashlib
import json
from export_source import export, transform_markdown

class ExportTests(unittest.TestCase):
    def fixture(self, root):
        source = root / "source"
        source.mkdir()
        for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml"):
            (source / name).write_text("fixture")
        return source
    def test_export_preserves_git_and_excludes_evidence(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = self.fixture(root)
            (source / ".git").mkdir()
            (source / ".git" / "sentinel").write_text("original")
            (source / "docs" / "validation" / "evidence").mkdir(parents=True)
            (source / "docs" / "validation" / "evidence" / "secret.txt").write_text("private")
            (source / "docs" / "contract.md").write_text("contract")
            output = root / "output"
            export(source, output)
            self.assertFalse((output / ".git").exists())
            self.assertFalse((output / "docs" / "validation" / "evidence").exists())
            self.assertEqual((source / ".git" / "sentinel").read_text(), "original")
            with self.assertRaises(ValueError):
                export(source, output)
            with self.assertRaises(ValueError):
                export(source, source / "nested")
    def test_omitted_links_have_auditable_export_only_transformation(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = self.fixture(root)
            (source / "docs" / "validation" / "evidence").mkdir(parents=True)
            (source / "docs" / "validation" / "private.log").write_text("private")
            (source / "docs" / "validation" / "raw.json").write_text('{"private":true}')
            (source / "docs" / "validation" / "screen.png").write_bytes(b"private screenshot")
            original = b"# Evidence\n\n[raw](validation/evidence/run/result.json)\n![image](validation/screen.png)\n[log](validation/private.log)\n[public](https://example.test/evidence/x)\n[contract](contract.md)\n```text\n[example](validation/evidence/example.json)\n```\n"
            document = source / "docs" / "guide.md"
            document.write_bytes(original)
            (source / "docs" / "contract.md").write_text("contract\n")
            output = root / "output"
            export(source, output)
            self.assertEqual(document.read_bytes(), original)
            self.assertFalse((output / "docs" / "validation" / "private.log").exists())
            self.assertFalse((output / "docs" / "validation" / "raw.json").exists())
            self.assertFalse((output / "docs" / "validation" / "screen.png").exists())
            text = (output / "docs" / "guide.md").read_text()
            self.assertEqual(text.count("(증거 미포함)"), 3)
            self.assertIn("[example](validation/evidence/example.json)", text)
            self.assertIn("[contract](contract.md)", text)
            receipt = json.loads((output / "SOURCE_EXPORT.json").read_text())
            self.assertIs(receipt["release_ready"], False)
            change = receipt["markdown_transformations"][0]
            self.assertEqual(change["original_sha256"], hashlib.sha256(original).hexdigest())
            self.assertEqual(change["exported_sha256"], hashlib.sha256((output / "docs" / "guide.md").read_bytes()).hexdigest())
            self.assertEqual(change["links"][0]["original_target"], "validation/evidence/run/result.json")
            self.assertTrue((output / "docs" / "EXPORT-EVIDENCE-OMITTED.md").is_file())
    def test_encoded_excluded_target_and_external_escape_remain_scoped(self):
        original = b"[encoded](../validation/%65vidence/run.json) [outside](../../../evidence/run.json) [external](https://example.test/docs/validation/evidence/a)\n"
        updated, changes = transform_markdown(original, "docs/work-items/item.md")
        self.assertEqual(len(changes), 1)
        self.assertIn(b"[outside](../../../evidence/run.json)", updated)
        self.assertIn(b"https://example.test/docs/validation/evidence/a", updated)
    def test_reserved_omission_note_collision_is_create_free(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = self.fixture(root)
            (source / "docs").mkdir()
            (source / "docs" / "EXPORT-EVIDENCE-OMITTED.md").write_text("original")
            output = root / "output"
            with self.assertRaises(ValueError):
                export(source, output)
            self.assertFalse(output.exists())
    def test_symlink_rejected_before_destination_created(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = self.fixture(root)
            (source / "scripts").mkdir()
            (source / "scripts" / "alias.py").symlink_to(source / "Cargo.toml")
            output = root / "output"
            with self.assertRaises(ValueError):
                export(source, output)
            self.assertFalse(output.exists())

if __name__ == "__main__":
    unittest.main()
