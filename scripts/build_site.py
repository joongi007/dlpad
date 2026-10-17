#!/usr/bin/env python3
"""Build the public site from an exact allowlist; never publish the checkout."""
import argparse
import json
import re
import shutil
import urllib.request
from pathlib import Path

FILES = ("index.html", "styles.css", "app.js", ".nojekyll", "assets/logo.svg", "assets/app-download.png")
REPO = "https://github.com/joongi007/dlpad"
LIMIT = 1024 * 1024


def unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON key")
        result[key] = value
    return result


def release_index(raw):
    if len(raw) > LIMIT:
        raise ValueError("release response exceeds limit")
    releases = json.loads(raw, object_pairs_hook=unique_pairs)
    if not isinstance(releases, list) or len(releases) > 100:
        raise ValueError("expected bounded release list")
    candidates = []
    for release in releases:
        if not isinstance(release, dict) or release.get("draft") is not False or not release.get("published_at"):
            continue
        tag = release.get("tag_name", "")
        if not isinstance(tag, str) or not re.fullmatch(r"v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)", tag):
            continue
        version = tag[1:]
        if release.get("html_url") != f"{REPO}/releases/tag/{tag}" or type(release.get("prerelease")) is not bool:
            raise ValueError("unexpected release origin or channel")
        assets = release.get("assets")
        if not isinstance(assets, list) or len(assets) > 100:
            raise ValueError("invalid release assets")
        row = {"version": version, "preview": release["prerelease"], "signed": False}
        for kind in ("installer", "portable"):
            name = f"dlpad-{version}-windows-x86_64-{kind}.zip"
            matches = [asset for asset in assets if isinstance(asset, dict) and asset.get("name") == name]
            if len(matches) != 1:
                break
            asset = matches[0]
            expected = f"{REPO}/releases/download/{tag}/{name}"
            if asset.get("browser_download_url") != expected or type(asset.get("size")) is not int or not 0 < asset["size"] <= 256 * 1024 * 1024:
                raise ValueError("invalid release asset URL or size")
            row[kind] = expected
        else:
            candidates.append(row)
    candidates.sort(key=lambda row: (not row["preview"], tuple(map(int, row["version"].split(".")))), reverse=True)
    return {"schema": 1, "release": candidates[0] if candidates else None}


def build(root, output, raw=b"[]"):
    source = root / "site"
    # Validate completely before touching the destination. Existing trees are never cleared.
    index = release_index(raw)
    for name in FILES:
        path = source / name
        if path.is_symlink() or not path.is_file() or any(p.is_symlink() for p in path.parents if p != root.parent):
            raise ValueError(f"missing or indirect site asset: {name}")
        if path.stat().st_size > 2 * 1024 * 1024:
            raise ValueError("site asset exceeds limit")
    output.mkdir(parents=True, exist_ok=False)
    for name in FILES:
        destination = output / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source / name, destination)
    (output / "release.json").write_text(json.dumps(index, ensure_ascii=False, separators=(",", ":")) + "\n", encoding="utf-8")
    return index


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output", type=Path, required=True)
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--release-api-file", type=Path)
    group.add_argument("--fetch-releases", action="store_true")
    args = parser.parse_args()
    raw = b"[]"
    if args.release_api_file:
        with args.release_api_file.open("rb") as stream:
            raw = stream.read(LIMIT + 1)
    elif args.fetch_releases:
        request = urllib.request.Request("https://api.github.com/repos/joongi007/dlpad/releases?per_page=100", headers={"Accept": "application/vnd.github+json", "User-Agent": "dlpad-site-build", "X-GitHub-Api-Version": "2022-11-28"})
        with urllib.request.urlopen(request, timeout=20) as response:
            if response.url != request.full_url:
                raise ValueError("release API redirect rejected")
            raw = response.read(LIMIT + 1)
    index = build(args.root.resolve(), args.output.absolute(), raw)
    print(f"Site built: {len(FILES) + 1} files; public_release={index['release'] is not None}")


if __name__ == "__main__":
    main()
