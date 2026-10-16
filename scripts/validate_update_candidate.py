"""Read-only, bounded candidate validation. Does not confer installation authority."""
import json
import re
from urllib.parse import urlsplit

MAX_MANIFEST_BYTES = 65536
MAX_PACKAGE_BYTES = 256 * 1024 * 1024
FIELDS = {"schema", "version", "channel", "target", "package_url", "package_bytes", "package_sha256", "release_ready"}

def version(value):
    if not isinstance(value, str) or len(value) > 64 or not re.fullmatch(r"(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})", value):
        raise ValueError("invalid version")
    return tuple(map(int, value.split(".")))

def unique_object(pairs):
    obj = {}
    for key, value in pairs:
        if key in obj:
            raise ValueError("duplicate field")
        obj[key] = value
    return obj

def validate_candidate(data, *, installed_version, channel, allowed_hosts):
    if not isinstance(data, bytes) or len(data) > MAX_MANIFEST_BYTES:
        raise ValueError("manifest byte bound")
    try:
        value = json.loads(data, object_pairs_hook=unique_object)
    except (UnicodeError, json.JSONDecodeError) as error:
        raise ValueError("invalid JSON") from error
    if not isinstance(value, dict) or set(value) != FIELDS:
        raise ValueError("invalid fields")
    if type(value["schema"]) is not int or value["schema"] != 1:
        raise ValueError("invalid schema")
    if value["target"] != "x86_64-pc-windows-msvc":
        raise ValueError("unsupported target")
    if channel not in ("stable", "preview") or value["channel"] != channel:
        raise ValueError("channel mismatch")
    if version(value["version"]) <= version(installed_version):
        raise ValueError("candidate must be newer; rollback requires separate authority")
    if type(value["package_bytes"]) is not int or not 0 < value["package_bytes"] <= MAX_PACKAGE_BYTES:
        raise ValueError("package byte bound")
    digest = value["package_sha256"]
    if not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
        raise ValueError("invalid digest")
    url = value["package_url"]
    if not isinstance(url, str) or len(url) > 2048 or any(ord(c) <= 32 or ord(c) >= 127 for c in url) or "\\" in url:
        raise ValueError("invalid URL")
    try:
        parsed = urlsplit(url)
        port = parsed.port
    except ValueError as error:
        raise ValueError("invalid URL") from error
    if parsed.scheme != "https" or parsed.hostname not in allowed_hosts or port not in (None, 443) or parsed.username is not None or parsed.password is not None or parsed.query or parsed.fragment or not parsed.path.startswith("/"):
        raise ValueError("URL outside explicit release policy")
    # No signature verifier/trust root exists in this draft. Never accept readiness.
    if value["release_ready"] is not False:
        raise ValueError("unsigned candidates cannot be release ready")
    return value
