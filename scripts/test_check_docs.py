"""Source-only checkout must omit evidence explicitly, without hiding missing source."""
import contextlib
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_docs


class SourceCheckoutTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="dlpad-doc-check-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for name in [*check_docs.REQUIRED, "docs/SOURCE-CHECKOUT.md"]:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("# Fixture\n", encoding="utf-8")

    def run_check(self, *args):
        output, errors = io.StringIO(), io.StringIO()
        with patch.object(check_docs, "ROOT", self.root), contextlib.redirect_stdout(output), contextlib.redirect_stderr(errors):
            code = check_docs.main(list(args))
        return code, output.getvalue(), errors.getvalue()

    def readme(self, links):
        (self.root / "README.md").write_text("# Fixture\n\n" + links + "\n", encoding="utf-8")

    def test_strict_default_rejects_absent_evidence(self):
        self.readme("[report](docs/validation/evidence/report.json)")
        code, _, errors = self.run_check()
        self.assertEqual(code, 1)
        self.assertIn("broken local link", errors)

    def test_source_mode_reports_omissions_and_does_not_require_local_artifacts(self):
        self.readme("[report](docs/validation/evidence/report.json) [tool](.local-tools/tool.exe) [trace](docs/validation/run/raw.json)")
        code, output, errors = self.run_check("--source-only")
        self.assertEqual((code, errors), (0, ""))
        self.assertIn("3 links", output)
        self.assertIn("runtime evidence was NOT validated", output)

    def test_missing_authored_documents_source_and_native_assets_still_fail(self):
        for link in ["docs/missing.md", "crates/app/src/missing.rs", "crates/app/assets/missing.ico", "docs/validation/missing.md"]:
            with self.subTest(link=link):
                self.readme("[missing](" + link + ")")
                code, _, errors = self.run_check("--source-only")
                self.assertEqual(code, 1)
                self.assertIn("broken local link", errors)

    def test_omission_looking_links_cannot_escape_root(self):
        for link in ["../evidence/report.json", "docs/../../../evidence/report.json", "..%2Fevidence%2Freport.json"]:
            with self.subTest(link=link):
                self.readme("[outside](" + link + ")")
                code, _, errors = self.run_check("--source-only")
                self.assertEqual(code, 1)
                self.assertIn("nonportable local link", errors)

    def test_required_documents_and_checkout_notice_cannot_be_omitted(self):
        for name in ["PRODUCT.md", "docs/SOURCE-CHECKOUT.md"]:
            with self.subTest(name=name):
                path = self.root / name
                original = path.read_bytes()
                path.unlink()
                code, _, errors = self.run_check("--source-only")
                self.assertEqual(code, 1)
                self.assertIn(name, errors)
                path.write_bytes(original)

    def test_source_mode_prunes_local_evidence_documents_but_checks_authored_nested_docs(self):
        ignored = self.root / "docs/validation/evidence/private.md"
        ignored.parent.mkdir(parents=True)
        ignored.write_text("[broken](../../../../private.txt)", encoding="utf-8")
        self.assertEqual(self.run_check("--source-only")[0], 0)
        authored = self.root / "docs/validation/nested/authored.md"
        authored.parent.mkdir(parents=True)
        authored.write_text("# Missing newline", encoding="utf-8")
        self.assertEqual(self.run_check("--source-only")[0], 1)

    def test_missing_exact_toolchain_remains_a_failure(self):
        (self.root / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
        code, _, errors = self.run_check("--source-only")
        self.assertEqual(code, 1)
        self.assertIn("Cargo.lock", errors)
        self.assertIn("rust-toolchain.toml", errors)


if __name__ == "__main__":
    unittest.main()
