"""Validate release refs without turning a candidate-only request into a draft."""
import re

VERSION = r'(?:0|[1-9][0-9]{0,8})\.(?:0|[1-9][0-9]{0,8})\.(?:0|[1-9][0-9]{0,8})'


def validate(ref, version, candidate_only):
    if not re.fullmatch(VERSION, version):
        raise ValueError('Invalid workspace version')
    if candidate_only:
        if ref != 'refs/heads/main':
            raise ValueError('Candidate-only validation requires main; no draft will be created')
        return version, False
    match = re.fullmatch('refs/tags/v(' + VERSION + ')', ref)
    if not match or match[1] != version:
        raise ValueError('An exact vX.Y.Z tag matching Cargo workspace version is required')
    return version, True
