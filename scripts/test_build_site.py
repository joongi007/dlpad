import copy
import json
import tempfile
import unittest
import struct
import xml.etree.ElementTree as ET
from html.parser import HTMLParser
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

    def test_published_korean_page_metadata_and_crawl_assets(self):
        class Page(HTMLParser):
            def __init__(self):
                super().__init__()
                self.lang = None
                self.links = []
                self.meta = []
                self.title = []
                self.in_title = False

            def handle_starttag(self, tag, attrs):
                values = dict(attrs)
                if tag == "html": self.lang = values.get("lang")
                if tag == "link": self.links.append(values)
                if tag == "meta": self.meta.append(values)
                if tag == "title": self.in_title = True

            def handle_endtag(self, tag):
                if tag == "title": self.in_title = False

            def handle_data(self, data):
                if self.in_title: self.title.append(data)

        root = Path(__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder) / "public"
            build(root, output)
            page = Page()
            page.feed((output / "index.html").read_text(encoding="utf-8"))
            self.assertEqual(page.lang, "ko")
            self.assertIn("dlpad", "".join(page.title))
            self.assertIn("yt-dlp GUI", "".join(page.title))
            canonical = [x["href"] for x in page.links if x.get("rel") == "canonical"]
            self.assertEqual(canonical, ["https://joongi007.github.io/dlpad/"])
            self.assertEqual([x["content"] for x in page.meta if x.get("property") == "og:url"], canonical)
            self.assertEqual([x["content"] for x in page.meta if x.get("name") == "google-site-verification"], ["9ttkOi3mp0nQ0sbqtMzv7OmcxgYa1eDZCf-rPxq4KUU"])
            self.assertFalse(any(x.get("hreflang") for x in page.links))
            sitemap = ET.parse(output / "sitemap.xml")
            self.assertEqual([x.text for x in sitemap.findall(".//{http://www.sitemaps.org/schemas/sitemap/0.9}loc")], canonical)
            icon = next(x for x in page.links if x.get("rel") == "icon")
            self.assertEqual(icon.get("type"), "image/png")
            data = (output / icon["href"]).read_bytes()
            self.assertEqual(data[:8], b"\x89PNG\r\n\x1a\n")
            width, height = struct.unpack(">II", data[16:24])
            self.assertEqual(width, height)
            self.assertGreater(width, 48)
            self.assertEqual(icon["sizes"], f"{width}x{height}")


if __name__ == "__main__":
    unittest.main()
