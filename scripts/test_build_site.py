import copy
import json
import tempfile
import unittest
from pathlib import Path

from build_site import FILES, REPO, build, release_index


def release(version="1.2.3", preview=False):
    return {"draft": False, "prerelease": preview, "published_at": "2026-10-18T00:00:00Z", "tag_name": f"v{version}", "html_url": f"{REPO}/releases/tag/v{version}", "assets": [{"name": f"dlpad-{version}-windows-x86_64-{kind}.zip", "size": 1024, "browser_download_url": f"{REPO}/releases/download/v{version}/dlpad-{version}-windows-x86_64-{kind}.zip"} for kind in ("installer", "portable")]}


class SiteTests(unittest.TestCase):
    def index(self, rows):
        return release_index(json.dumps(rows).encode())["release"]

    def test_empty_and_draft_have_no_download(self):
        self.assertIsNone(self.index([]))
        row = release(); row["draft"] = True
        self.assertIsNone(self.index([row]))

    def test_stable_preferred_and_numeric_order(self):
        self.assertEqual(self.index([release("9.0.0", True), release("1.9.0"), release("1.10.0")])["version"], "1.10.0")
        self.assertTrue(self.index([release(preview=True)])["preview"])

    def test_incomplete_pair_not_offered(self):
        row = release(); row["assets"].pop()
        self.assertIsNone(self.index([row]))

    def test_duplicate_assets_not_offered(self):
        row = release(); row["assets"].append(copy.deepcopy(row["assets"][0]))
        self.assertIsNone(self.index([row]))

    def test_unsafe_asset_url_or_size_rejected(self):
        for value in ["https://evil.example/a.zip", release()["assets"][0]["browser_download_url"] + "?x=1"]:
            row = release(); row["assets"][0]["browser_download_url"] = value
            with self.assertRaises(ValueError): self.index([row])
        for value in [0, -1, True, 256 * 1024 * 1024 + 1]:
            row = release(); row["assets"][0]["size"] = value
            with self.assertRaises(ValueError): self.index([row])

    def test_duplicate_json_and_oversize_rejected(self):
        with self.assertRaises(ValueError): release_index(b'[{"draft":false,"draft":true}]')
        with self.assertRaises(ValueError): release_index(b' ' * (1024 * 1024 + 1))

    def test_allowlist_and_existing_output_preserved(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for name in FILES:
                path = root / "site" / name; path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(b"fixture")
            (root / "site" / "private.txt").write_text("private")
            output = root / "public"
            build(root, output)
            self.assertEqual({p.relative_to(output).as_posix() for p in output.rglob("*") if p.is_file()}, set(FILES) | {"release.json"})
            with self.assertRaises(FileExistsError): build(root, output)
            self.assertEqual((output / "index.html").read_bytes(), b"fixture")

    def test_symlink_source_rejected_before_output_creation(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for name in FILES:
                path = root / "site" / name; path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(b"fixture")
            path = root / "site" / "index.html"; path.unlink(); path.symlink_to(root / "site" / "app.js")
            with self.assertRaises(ValueError): build(root, root / "public")
            self.assertFalse((root / "public").exists())


if __name__ == "__main__":
    unittest.main()
