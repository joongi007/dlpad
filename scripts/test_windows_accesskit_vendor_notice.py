"""Verify the actual path patch and its complete upstream notice trio."""
import copy
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import windows_candidate_package as package


class AccessKitNoticeTests(unittest.TestCase):
    def setUp(self):
        self.mapping = package.upstream_manifest()
        self.source = package.patched_source(
            {"name": "accesskit_windows", "version": "0.29.2"}
        )
        self.metadata = package.toml_inventory(
            (self.source / "Cargo.toml").read_text()
        )["package"]

    def test_actual_vendor_and_three_original_notices(self):
        self.assertEqual(self.source, package.ROOT / "vendor/accesskit_windows")
        notices = package.mapped_notices(
            self.mapping, "accesskit_windows-0.29.2", self.source, self.metadata
        )
        self.assertEqual(
            [leaf for leaf, _, _ in notices],
            ["LICENSE-MIT", "LICENSE-APACHE", "LICENSE.chromium"],
        )
        for leaf, original, record in notices:
            self.assertEqual((self.source / leaf).read_bytes(), original.read_bytes())
            self.assertEqual(package.digest(original), record["sha256"])

    def test_missing_chromium_notice_is_rejected(self):
        mapping = copy.deepcopy(self.mapping)
        mapping["packages"]["accesskit_windows-0.29.2"]["sources"].pop()
        with self.assertRaisesRegex(ValueError, "notice trio"):
            package.mapped_notices(
                mapping, "accesskit_windows-0.29.2", self.source, self.metadata
            )

    def test_foreign_revision_is_rejected(self):
        mapping = copy.deepcopy(self.mapping)
        mapping["packages"]["accesskit_windows-0.29.2"]["commit"] = "0" * 40
        with self.assertRaisesRegex(ValueError, "notice trio"):
            package.mapped_notices(
                mapping, "accesskit_windows-0.29.2", self.source, self.metadata
            )

    def test_registry_and_unknown_versions_are_not_path_patches(self):
        self.assertIsNone(package.patched_source({
            "name": "accesskit_windows", "version": "0.29.2", "source": "registry+fixture"
        }))
        self.assertIsNone(package.patched_source({
            "name": "accesskit_windows", "version": "0.29.3"
        }))


if __name__ == "__main__":
    unittest.main()
