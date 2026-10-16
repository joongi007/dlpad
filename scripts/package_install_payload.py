#!/usr/bin/env python3
"""Create-only deterministic ZIP of an explicitly pinned unsigned development payload."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import zipfile
import generate_install_payload as base

ENTRIES = {"dlpad.exe", "LICENSE", "THIRD-PARTY-NOTICES.txt", "README.txt", "binding-record.txt"}
FIELDS = {"schema", "version", "target", "entries", "development_only"}
STAMP = (1980, 1, 1, 0, 0, 0)

def require(ok, message):
    if not ok:
        raise ValueError(message)

def pin(value):
    require(isinstance(value, str) and re.fullmatch("[0-9a-f]{64}", value) is not None, "lowercase SHA pin")

def small(path, limit):
    with base.plain_file(path).open("rb") as stream:
        value = stream.read(limit + 1)
        require(len(value) <= limit and not stream.read(1), "small document bound")
        return value

def namespace(root):
    require(root.is_dir() and not root.is_symlink(), "plain payload root")
    for item in [root, *root.parents]:
        require(not item.is_symlink(), "symlink root/ancestor")
    require({item.name for item in root.iterdir()} == {"dlpad-install.exe", "payload"}, "exact outer namespace")
    payload = root / "payload"
    require(payload.is_dir() and not payload.is_symlink(), "plain payload directory")
    require({item.name for item in payload.iterdir()} == ENTRIES | {"manifest.json"}, "exact payload namespace")
    for item in payload.iterdir():
        base.plain_file(item)
    base.plain_file(root / "dlpad-install.exe")

def package(root, output, manifest_sha256, binding_sha256, installer_sha256):
    for value in (manifest_sha256, binding_sha256, installer_sha256):
        pin(value)
    root = Path(root).absolute()
    output = Path(output).absolute()
    require(".." not in root.parts and ".." not in output.parts, "normalized source/output paths required")
    receipt_path = Path(str(output) + ".receipt.json")
    require(root not in output.parents and root != output, "output outside source")
    for item in [output, receipt_path]:
        require(not item.exists() and not item.is_symlink(), "fresh output and receipt required")
        require(item.parent.is_dir(), "existing output parent")
        for parent in item.parents:
            require(not parent.is_symlink(), "symlink output ancestor")
    namespace(root)
    manifest_raw = small(root / "payload/manifest.json", 65536)
    require(hashlib.sha256(manifest_raw).hexdigest() == manifest_sha256, "manifest pin")
    manifest = base.json_exact(manifest_raw)
    require(isinstance(manifest, dict) and set(manifest) == FIELDS and type(manifest["schema"]) is int and manifest["schema"] == 1 and manifest["development_only"] is True and manifest["target"] == base.TARGET, "manifest schema/readiness")
    require(isinstance(manifest["version"], str) and re.fullmatch(r"(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})", manifest["version"]) is not None, "manifest version")
    require(isinstance(manifest["entries"], list) and len(manifest["entries"]) == 5, "exact five entries")
    observed = {}
    total = len(manifest_raw)
    for entry in manifest["entries"]:
        require(isinstance(entry, dict) and set(entry) == {"name", "bytes", "sha256"}, "entry fields")
        name = entry["name"]
        require(isinstance(name, str) and name in ENTRIES and name not in observed, "exact unique entry name")
        pin(entry["sha256"])
        limit = base.MAX_EXE if name == "dlpad.exe" else base.MAX_DOC
        require(type(entry["bytes"]) is int and 0 < entry["bytes"] <= limit, "entry size")
        digest, size = base.file_sha(root / "payload" / name, limit)
        require((digest, size) == (entry["sha256"], entry["bytes"]), "payload entry SHA/size")
        observed[name] = dict(sha256=digest, bytes=size)
        total += size
    require(set(observed) == ENTRIES, "exact payload entries")
    binding_raw = small(root / "payload/binding-record.txt", base.MAX_DOC)
    require(hashlib.sha256(binding_raw).hexdigest() == binding_sha256, "binding pin")
    binding = base.json_exact(binding_raw)
    require(isinstance(binding, dict) and set(binding) == {"schema", "development_only", "release_ready", "complete_legal_coverage", "signed_distribution", "candidate_zip_sha256", "candidate_manifest_sha256", "mapping", "binaries", "project_license", "notices"} and type(binding.get("schema")) is int and binding["schema"] == 1 and binding.get("development_only") is True and all(binding.get(key) is False for key in ["release_ready", "complete_legal_coverage", "signed_distribution"]), "unsigned binding readiness")
    for key in ["candidate_zip_sha256", "candidate_manifest_sha256"]:
        pin(binding.get(key))
    require(binding.get("mapping") == {"dlpad-cpu.exe": "payload/dlpad.exe", "dlpad-install.exe": "dlpad-install.exe"}, "explicit binary mapping")
    binaries = binding.get("binaries")
    require(isinstance(binaries, dict) and set(binaries) == {"dlpad-cpu.exe", "dlpad-install.exe"} and binaries["dlpad-cpu.exe"] == observed["dlpad.exe"], "original generator binary binding")
    for proof in binaries.values():
        require(isinstance(proof, dict) and set(proof) == {"bytes", "sha256"} and type(proof["bytes"]) is int and 0 < proof["bytes"] <= base.MAX_EXE, "binary proof domain")
        pin(proof["sha256"])
    require(isinstance(binding.get("project_license"), dict) and set(binding["project_license"]) == {"source_name", "bytes", "sha256"} and type(binding["project_license"]["bytes"]) is int, "license proof domain")
    require(isinstance(binding.get("notices"), dict) and 0 < len(binding["notices"]) <= 4096, "notice inventory bound")
    installer_digest, installer_size = base.file_sha(root / "dlpad-install.exe", base.MAX_EXE)
    installer = dict(sha256=installer_digest, bytes=installer_size)
    require(installer_digest == installer_sha256 and 0 < installer_size <= base.MAX_EXE and binaries["dlpad-install.exe"] == installer, "installer SHA/size binding")
    require(binding.get("project_license", {}).get("sha256") == observed["LICENSE"]["sha256"] and binding["project_license"].get("bytes") == observed["LICENSE"]["bytes"], "project license binding")
    total += installer_size
    require(total <= base.MAX_ZIP, "expanded package byte bound")
    files = {"dlpad-install.exe": installer, "payload/manifest.json": dict(sha256=manifest_sha256, bytes=len(manifest_raw))}
    files.update({"payload/" + name: proof for name, proof in observed.items()})
    # No extraction. Stream each exact regular file into fixed-metadata archive members.
    with output.open("xb") as target, zipfile.ZipFile(target, "w", compression=zipfile.ZIP_STORED, allowZip64=False) as archive:
        for name, proof in sorted(files.items()):
            info = zipfile.ZipInfo(name, STAMP)
            info.create_system = 3
            info.external_attr = (stat.S_IFREG | (0o755 if name.endswith(".exe") else 0o644)) << 16
            info.compress_type = zipfile.ZIP_STORED
            source = root / name
            fd = os.open(base.plain_file(source), os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
            digest = hashlib.sha256()
            count = 0
            with os.fdopen(fd, "rb") as reader, archive.open(info, "w") as writer:
                require(stat.S_ISREG(os.fstat(reader.fileno()).st_mode), "regular pinned stream")
                while chunk := reader.read(65536):
                    count += len(chunk)
                    require(count <= proof["bytes"], "copy growth rejected")
                    digest.update(chunk)
                    writer.write(chunk)
            require(dict(sha256=digest.hexdigest(), bytes=count) == proof, "source changed during ZIP copy")
    namespace(root)
    for name, proof in files.items():
        require(base.file_sha(root / name, base.MAX_EXE if name.endswith(".exe") else base.MAX_DOC) == (proof["sha256"], proof["bytes"]), "original source preserved/current")
    # Re-read CRC and member hashes through the independent ZIP reader, with no extraction.
    members = {}
    with output.open("rb") as stream, zipfile.ZipFile(stream) as archive:
        infos = archive.infolist()
        require(len(infos) == 7 and [info.filename for info in infos] == sorted(files), "exact deterministic archive namespace")
        for info in infos:
            require(info.date_time == STAMP and stat.S_IFMT(info.external_attr >> 16) == stat.S_IFREG and not info.flag_bits & 1, "regular deterministic member")
            with archive.open(info) as reader:
                digest, size = base.sha_stream(reader, base.MAX_EXE if info.filename.endswith(".exe") else base.MAX_DOC)
            require(dict(sha256=digest, bytes=size) == files[info.filename], "archive hash/CRC binding")
            members[info.filename] = dict(sha256=digest, bytes=size, crc32=f"{info.CRC:08x}", regular_mode=info.external_attr >> 16)
    zip_sha, zip_bytes = base.file_sha(output, base.MAX_ZIP)
    receipt = dict(schema=1, development_only=True, release_ready=False, signed_distribution=False, complete_legal_coverage=False, zip_sha256=zip_sha, zip_bytes=zip_bytes, manifest_sha256=manifest_sha256, binding_sha256=binding_sha256, installer_sha256=installer_sha256, members=members, source_files_preserved=True)
    with receipt_path.open("xb") as stream:
        stream.write(base.encoded(receipt))
    return receipt

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    for name in ("manifest", "binding", "installer"):
        parser.add_argument("--" + name + "-sha256", required=True)
    package(**vars(parser.parse_args()))

if __name__ == "__main__":
    main()
