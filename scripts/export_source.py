"""Create-only source export; never accesses or modifies source .git."""
import argparse
import hashlib
import json
import posixpath
import re
from urllib.parse import unquote, urlsplit
import os
from pathlib import Path
import shutil
import stat

ROOT_FILES = {"AGENTS.md", "README.md", "DEVELOPMENT_HISTORY.md", "PRODUCT.md", "CONTEXT.md", "DESIGN.md", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "LICENSE", "LICENSE.md", ".gitignore", ".gitattributes", ".editorconfig", "CONTRIBUTING.md", "CLAUDE.md"}
ROOT_DIRS = {"crates", "scripts", "vendor", "docs", ".github"}
EXCLUDED_DIRS = {".git", ".aws", ".agents", ".codex", "target", "__pycache__", "evidence"}
EXCLUDED_SUFFIXES = {".exe", ".dll", ".zip", ".sqlite", ".sqlite3", ".db", ".pyc", ".log"}

OMISSION_NOTE = "docs/EXPORT-EVIDENCE-OMITTED.md"
MAX_MARKDOWN_BYTES = 16 * 1024 * 1024
LINK = re.compile(r"(!?)\[([^\]]*)\]\(([^)]+)\)")

def omitted(relative):
    parts = Path(relative).parts
    return (any(part in EXCLUDED_DIRS for part in parts)
            or Path(relative).suffix.lower() in EXCLUDED_SUFFIXES
            or relative.startswith(".local-tools/")
            or (relative.startswith("docs/validation/") and Path(relative).suffix.lower() != ".md"))

def transform_markdown(raw, relative):
    if len(raw) > MAX_MARKDOWN_BYTES:
        raise ValueError("markdown transformation byte bound")
    content = raw.decode("utf-8")
    changes = []
    parent = posixpath.dirname(relative)
    replacement = posixpath.relpath(OMISSION_NOTE, parent or ".")
    def replace(match):
        original = match.group(3).strip().split(' "', 1)[0].strip("<>")
        parsed = urlsplit(original)
        if parsed.scheme or parsed.netloc or not parsed.path or parsed.path.startswith("/"):
            return match.group(0)
        target = posixpath.normpath(posixpath.join(parent, unquote(parsed.path)))
        if target == ".." or target.startswith("../") or not omitted(target):
            return match.group(0)
        changes.append(dict(original_target=match.group(3), original_relative_path=target))
        return "[" + match.group(2) + " (증거 미포함)](" + replacement + ")"
    # Preserve fenced examples verbatim; this policy handles only inline Markdown links.
    segments = re.split(r"(```.*?```|~~~.*?~~~)", content, flags=re.S)
    updated = "".join(segment if index % 2 else LINK.sub(replace, segment) for index, segment in enumerate(segments))
    return updated.encode("utf-8"), changes

def export(source, destination):
    source = source.resolve(strict=True)
    destination = destination.absolute()
    if destination.exists() or destination.is_symlink():
        raise ValueError("destination must be absent")
    if source == destination or source in destination.parents:
        raise ValueError("destination must be outside source")
    # Preflight the full allowlist before creating output. Fail closed on symlinks.
    files = []
    for entry in source.iterdir():
        if entry.name not in ROOT_FILES | ROOT_DIRS:
            continue
        if entry.is_symlink():
            raise ValueError("symlink in source allowlist")
        if entry.is_file():
            files.append(entry)
        elif entry.name in ROOT_DIRS:
            for current, directories, names in os.walk(entry, followlinks=False):
                directories[:] = sorted(d for d in directories if d not in EXCLUDED_DIRS)
                for directory in directories:
                    if (Path(current) / directory).is_symlink():
                        raise ValueError("symlink directory")
                for name in sorted(names):
                    item = Path(current) / name
                    if item.is_symlink():
                        raise ValueError("symlink file")
                    if not omitted(item.relative_to(source).as_posix()):
                        files.append(item)
    if not {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml"}.issubset({str(f.relative_to(source)) for f in files}):
        raise ValueError("missing build contract")
    if any(f.relative_to(source).as_posix() == OMISSION_NOTE for f in files):
        raise ValueError("reserved export omission note already exists in source")
    destination.mkdir(parents=False, exist_ok=False)
    records = []
    transformations = []
    for item in sorted(files):
        relative = item.relative_to(source)
        out = destination / relative
        out.parent.mkdir(parents=True, exist_ok=True)
        # O_NOFOLLOW and exclusive destination creation; source reparse/native pins
        # still require native Windows integration before security-sensitive use.
        fd = os.open(item, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
        with os.fdopen(fd, "rb") as src, out.open("xb") as dst:
            if not stat.S_ISREG(os.fstat(src.fileno()).st_mode):
                raise ValueError("non-regular source")
            shutil.copyfileobj(src, dst, 1024 * 1024)
        digest = hashlib.sha256()
        with out.open("rb") as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
        original_sha256 = digest.hexdigest()
        links = []
        if relative.suffix.lower() == ".md":
            with out.open("rb") as copied:
                raw = copied.read(MAX_MARKDOWN_BYTES + 1)
            updated, links = transform_markdown(raw, relative.as_posix())
            if links:
                with out.open("wb") as authored_export:
                    authored_export.write(updated)
                digest = hashlib.sha256(updated)
                transformations.append(dict(path=relative.as_posix(), original_sha256=original_sha256, exported_sha256=digest.hexdigest(), links=links))
        records.append(dict(path=relative.as_posix(), bytes=out.stat().st_size, sha256=digest.hexdigest(), original_sha256=original_sha256, transformed=bool(links)))
    note = destination / OMISSION_NOTE
    note.parent.mkdir(parents=True, exist_ok=True)
    explanation = "# Source export: evidence omitted\n\n이 소스 복사본에는 개발 실행 증거, private 로그·스크린샷, 실행 파일·ZIP 및 원시 검증 산출물을 포함하지 않았습니다. 제품 빌드에 필요한 아이콘·resource 소스 자산은 유지했습니다. 이 문서는 검증 결과나 PASS 판정을 생성하지 않습니다. 원본 프로젝트 문서와 증거는 수정하지 않았습니다. 원문의 과거 검증 주장에는 이 복사본으로 독립 확인할 수 없는 근거가 있습니다.\n\nOnly inline links to intentionally excluded paths were redirected here. Original targets and original/exported document SHA256 are recorded in SOURCE_EXPORT.json. Code examples and external URLs remain unchanged. Exclusion is not proof that an artifact ever existed or that any test passed. This export is not release ready and does not confer publisher trust.\n\n## Original link targets\n\n"
    for change in transformations:
        explanation += "Document: " + json.dumps(change["path"], ensure_ascii=False) + "\n\n```text\n"
        for link in change["links"]:
            explanation += json.dumps(link, ensure_ascii=False) + "\n"
        explanation += "```\n\n"
    with note.open("x", encoding="utf-8") as stream:
        stream.write(explanation)
    note_raw = explanation.encode("utf-8")
    records.append(dict(path=OMISSION_NOTE, bytes=len(note_raw), sha256=hashlib.sha256(note_raw).hexdigest(), generated=True))
    receipt = dict(schema=2, release_ready=False, files=records, omission_note=OMISSION_NOTE, markdown_transformations=transformations)
    with (destination / "SOURCE_EXPORT.json").open("x", encoding="utf-8") as stream:
        stream.write(json.dumps(receipt, indent=2, ensure_ascii=False) + "\n")
    return records

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    export(args.source, args.destination)
