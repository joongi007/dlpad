#!/usr/bin/env python3
"""Dependency-free repository documentation checks; no network access."""

import argparse
import os
from pathlib import Path, PurePosixPath
from functools import lru_cache
import hashlib
import json
import re
import sys
from urllib.parse import unquote, urlsplit

from export_source import EXCLUDED_DIRS, omitted

ROOT = Path(__file__).resolve().parents[1]
REQUIRED = (
    "README.md", "AGENTS.md", "CLAUDE.md", "CONTRIBUTING.md", "PRODUCT.md",
    "DESIGN.md", "CONTEXT.md", "docs/REQUIREMENTS.md", "docs/ARCHITECTURE.md",
    "docs/STACK.md", "docs/UX.md", "docs/INTEGRATIONS.md", "docs/PERFORMANCE.md",
    "docs/DIAGNOSTICS.md",
    "docs/ROADMAP.md", "docs/CICD.md", "docs/SOURCES.md", "docs/adr/README.md",
    "docs/agents/README.md", "docs/agents/WORK_ITEM_TEMPLATE.md",
)


def link_parent(path: Path) -> Path:
    """Resolve immutable evidence copies against their hash-pinned document origin."""
    evidence = ROOT / "docs/validation/evidence"
    if path.parent.name != "inputs" or not path.is_relative_to(evidence):
        return path.parent
    pins_path = path.parent.parent / "input-pins.json"
    try:
        with pins_path.open("rb") as source:
            raw = source.read(256 * 1024 + 1)
        if len(raw) > 256 * 1024:
            return path.parent
        pins = json.loads(raw)["pins"]
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        def project_path(name):
            original = Path(name)
            if not original.is_absolute() or ".." in original.parts:
                return None
            if any(p.is_symlink() for p in [original, *original.parents]):
                return None
            # Windows mounts may expose the same root with different casing.
            # Match the existing root identity, never an arbitrary path suffix.
            for parent in original.parents:
                try:
                    if parent.samefile(ROOT):
                        return ROOT / original.relative_to(parent)
                except OSError:
                    continue
            return None

        if not isinstance(pins, dict) or len(pins) > 4096:
            return path.parent
        pinned_copy = [name for name, expected in pins.items()
                       if expected == digest and project_path(name) == path]
        if len(pinned_copy) != 1:
            return path.parent
        origins = []
        for name, expected in pins.items():
            original = project_path(name)
            if (expected == digest and original is not None
                    and original.name == path.name
                    and not original.is_relative_to(evidence)
                    ):
                origins.append(original)
        return origins[0].parent if len(origins) == 1 else path.parent
    except (OSError, ValueError, KeyError, TypeError, AttributeError):
        return path.parent



@lru_cache(maxsize=128)
def packaged_notice_hashes(bundle: Path) -> dict[str, str]:
    """Read bounded package declarations; preserve upstream notice bytes verbatim."""
    try:
        manifest = bundle / "candidate-manifest.json"
        inventory = bundle / "license-inventory.json"
        if (not manifest.is_file() or manifest.is_symlink()
                or inventory.is_symlink()):
            return {}
        with inventory.open("rb") as source:
            raw = source.read(4 * 1024 * 1024 + 1)
        if len(raw) > 4 * 1024 * 1024:
            return {}
        records = json.loads(raw)["records"]
        if not isinstance(records, list) or len(records) > 1024:
            return {}
        hashes = {}
        for record in records:
            notices = record["notices"]
            if not isinstance(notices, list) or len(notices) > 64:
                return {}
            for notice in notices:
                entry, digest = notice["entry"], notice["sha256"]
                relative = PurePosixPath(entry)
                if (relative.is_absolute() or ".." in relative.parts
                        or not relative.parts or relative.parts[0] != "licenses"
                        or "\\" in entry
                        or re.fullmatch(r"[0-9a-f]{64}", digest) is None
                        or entry in hashes or len(hashes) >= 4096):
                    return {}
                hashes[entry] = digest
        return hashes
    except (OSError, ValueError, KeyError, TypeError):
        return {}


def is_packaged_notice(path: Path) -> bool:
    evidence = ROOT / "docs/validation/evidence"
    if not path.is_relative_to(evidence):
        return False
    for parent in path.parents:
        if parent == evidence:
            break
        if parent.name != "licenses":
            continue
        bundle = parent.parent
        if any(p.is_symlink() for p in [path, *path.parents]):
            return False
        expected = packaged_notice_hashes(bundle).get(path.relative_to(bundle).as_posix())
        if expected is None:
            return False
        try:
            with path.open("rb") as source:
                raw = source.read(1024 * 1024 + 1)
            return len(raw) <= 1024 * 1024 and hashlib.sha256(raw).hexdigest() == expected
        except OSError:
            return False
    return False


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-only", action="store_true",
                        help="Check the source checkout; explicitly omit local-only evidence links.")
    args = parser.parse_args(argv)
    errors = []
    omitted_links = 0
    if args.source_only and not (ROOT / "docs/SOURCE-CHECKOUT.md").is_file():
        errors.append("Source-only checks require docs/SOURCE-CHECKOUT.md")
    for name in REQUIRED:
        if not (ROOT / name).is_file():
            errors.append(f"Missing required document: {name}")

    if args.source_only:
        documents = sorted(ROOT.glob("*.md"))
        for directory in [ROOT / "docs", ROOT / ".github"]:
            for current, directories, files in os.walk(directory, followlinks=False):
                directories[:] = [name for name in directories if name not in EXCLUDED_DIRS]
                documents.extend(Path(current) / name for name in sorted(files)
                                 if name.endswith(".md")
                                 and not omitted((Path(current) / name).relative_to(ROOT).as_posix()))
    else:
        documents = sorted(ROOT.glob("*.md")) + sorted((ROOT / "docs").rglob("*.md"))
        documents += sorted((ROOT / ".github").rglob("*.md"))
    original_notice_count = sum(is_packaged_notice(path) for path in documents)
    documents = [path for path in documents if not is_packaged_notice(path)]
    for path in documents:
        name = path.relative_to(ROOT)
        content = path.read_text(encoding="utf-8")
        if not content.strip():
            errors.append(f"{name}: empty document")
        if not content.endswith("\n"):
            errors.append(f"{name}: missing final newline")
        # Check inline local file links; external URLs and heading anchors are excluded.
        prose = re.sub(r"```.*?```", "", content, flags=re.S)
        parent = link_parent(path)
        for match in re.finditer(r"\[[^\]]*\]\(([^)]+)\)", prose):
            target = match.group(1).strip().split(' "', 1)[0].strip("<>")
            parsed = urlsplit(target)
            if parsed.scheme or parsed.netloc or not parsed.path:
                continue
            destination = (parent / unquote(parsed.path)).resolve()
            if not destination.is_relative_to(ROOT):
                errors.append(f"{name}: nonportable local link: {target}")
            elif args.source_only and omitted(destination.relative_to(ROOT).as_posix()):
                omitted_links += 1
            elif not destination.exists():
                errors.append(f"{name}: broken local link: {target}")

    manifest = ROOT / "Cargo.toml"
    if manifest.exists():
        for name in ("Cargo.lock", "rust-toolchain.toml"):
            if not (ROOT / name).is_file():
                errors.append(f"Rust workspace requires committed {name}")
        toolchain = ROOT / "rust-toolchain.toml"
        if toolchain.exists() and not re.search(
            r'channel\s*=\s*"\d+\.\d+\.\d+"', toolchain.read_text(encoding="utf-8")
        ):
            errors.append("rust-toolchain.toml must pin an exact Rust release")

    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"PASS: {len(documents)} documents; required files and local file links checked.")
    print(f"Preserved {original_notice_count} hash-matched packaged upstream notices; authored-document formatting does not apply.")
    if args.source_only:
        print(f"Source-only scope: {omitted_links} links to local-only evidence/artifacts omitted; runtime evidence was NOT validated.")
    print("External URLs and heading anchors are not validated by this checker.")
    if not manifest.exists():
        print("Native build not checked: Rust workspace has not been introduced.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
