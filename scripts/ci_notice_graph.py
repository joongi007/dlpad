"""Read CI-produced Cargo tree/metadata/lock into an explicitly incomplete graph receipt."""
import argparse
import hashlib
import json
import re
from windows_candidate_package import toml_inventory
from pathlib import Path

TARGET = 'x86_64-pc-windows-msvc'
NAME = r'[A-Za-z0-9_][A-Za-z0-9_-]{0,127}'
VERSION = r'(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)(?:-[0-9A-Za-z]+(?:[.-][0-9A-Za-z]+)*)?(?:\+[0-9A-Za-z]+(?:[.-][0-9A-Za-z]+)*)?'
ROW = re.compile(r'('+NAME+r') v('+VERSION+r')(?: \(([^()\r\n\x00]{1,4096})\))?(?: \(\*\))?\Z')
DIGEST = re.compile(r'[0-9a-f]{64}\Z')

def text(value, limit, nullable=False):
    if value is None and nullable:
        return value
    if not isinstance(value,str) or not 1 <= len(value) <= limit or any(ord(c)<32 or ord(c)==127 for c in value):
        raise ValueError('bounded printable string required')
    return value

def source(value):
    if value is None:
        return None
    text(value,4096)
    if not re.fullmatch(r'(?:registry|sparse)\+https://[^\s]+|git\+(?:https|ssh|file)://[^\s]+',value):
        raise ValueError('source domain')
    return value

def index(rows, metadata=False):
    if not isinstance(rows,list) or not 1 <= len(rows) <= 4096:
        raise ValueError('package count domain')
    result={}
    for row in rows:
        if not isinstance(row,dict):
            raise ValueError('package row object required')
        name,version=text(row.get('name'),128),text(row.get('version'),256)
        if not re.fullmatch(NAME,name) or not re.fullmatch(VERSION,version):
            raise ValueError('package identity domain')
        src=source(row.get('source'))
        checksum=row.get('checksum')
        if not metadata:
            if checksum is not None and (not isinstance(checksum,str) or not DIGEST.fullmatch(checksum)):
                raise ValueError('checksum domain')
            if src is not None and src.startswith(('registry+','sparse+')) and checksum is None:
                raise ValueError('registry checksum required')
            if src is None and checksum is not None:
                raise ValueError('path package checksum forbidden')
        else:
            identity=text(row.get('id'),8192)
            from urllib.parse import unquote, urlsplit
            short_path_id = False
            if src is None and identity.startswith('path+file://'):
                parsed = urlsplit(identity[len('path+'):])
                short_path_id = parsed.fragment == version and unquote(parsed.path).rstrip('/').rsplit('/',1)[-1] == name
            if not identity.endswith('#'+name+'@'+version) and not short_path_id:
                raise ValueError('metadata ID identity mismatch')
            if src is not None and not identity.startswith(src+'#'):
                raise ValueError('metadata ID source mismatch')
            if src is None and not identity.startswith('path+file://'):
                raise ValueError('path metadata ID required')
            manifest=text(row.get('manifest_path'),32768)
            from pathlib import PureWindowsPath
            if not (Path(manifest).is_absolute() or PureWindowsPath(manifest).is_absolute()) or '..' in manifest.replace('\\','/').split('/'):
                raise ValueError('absolute manifest path required')
            text(row.get('license'),4096,True)
            text(row.get('license_file'),32768,True)
        key=(name,version)
        if key in result:
            raise ValueError('ambiguous package name/version sources')
        result[key]=row
    return result

def bounded(path, limit):
    path = Path(path)
    if any(p.is_symlink() for p in [path, *path.parents]) or not path.is_file():
        raise ValueError('regular nonsymlink input required')
    with path.open('rb') as stream:
        raw = stream.read(limit + 1)
    if len(raw) > limit:
        raise ValueError('input byte bound')
    return raw

def unique(pairs):
    result = {}
    for key,value in pairs:
        if key in result:
            raise ValueError('duplicate JSON field')
        result[key] = value
    return result

def tree(raw):
    result = set()
    try:
        lines=raw.decode('utf-8-sig').splitlines()
    except UnicodeError as error:
        raise ValueError('tree UTF8 required') from error
    for row in lines:
        if not row:
            continue
        match = ROW.fullmatch(row)
        if not match:
            raise ValueError('unexpected Cargo tree row')
        annotation=match[3]
        if annotation is not None and annotation not in {'*','proc-macro'}:
            from pathlib import PureWindowsPath
            if not (Path(annotation).is_absolute() or PureWindowsPath(annotation).is_absolute() or re.fullmatch(r'https://[^\s()]+',annotation)):
                raise ValueError('tree source annotation shape')
        result.add((match[1],match[2]))
        if len(result) > 4096:
            raise ValueError('tree count bound')
    if not result:
        raise ValueError('empty graph')
    return result

def validate_lock_fields(raw):
    # The shared Python3.10 reader extracts strings only. Reject malformed or
    # duplicate authority fields rather than letting it silently omit them.
    try:
        lines=raw.decode('utf-8').splitlines()
    except UnicodeError as error:
        raise ValueError('lock UTF8 required') from error
    in_package=False
    seen=set()
    for line in lines:
        stripped=line.strip()
        if stripped=='[[package]]':
            in_package=True;seen=set();continue
        if stripped.startswith('['):
            in_package=False
        if not in_package:
            continue
        match=re.match(r'\s*(name|version|source|checksum)\s*=',line)
        if match:
            key=match[1]
            if key in seen or not re.fullmatch(r'\s*(?:name|version|source|checksum)\s*=\s*(?:"(?:[^"\\]|\\.)*"|\'[^\']*\')\s*(?:#.*)?',line):
                raise ValueError('lock authority field domain/duplicate')
            seen.add(key)

def classify(app_raw, engine_raw, metadata_raw, lock_raw):
    limits=[2*1024*1024,2*1024*1024,8*1024*1024,4*1024*1024]
    if any(not isinstance(raw,bytes) or len(raw)>limit for raw,limit in zip([app_raw,engine_raw,metadata_raw,lock_raw],limits)):
        raise ValueError('raw input byte domain')
    app,engine = tree(app_raw),tree(engine_raw)
    union = app | engine
    try:
        metadata = json.loads(metadata_raw,object_pairs_hook=unique)
    except (ValueError,UnicodeError,TypeError) as error:
        raise ValueError('invalid metadata JSON') from error
    if not isinstance(metadata,dict) or type(metadata.get('version')) is not int or metadata['version'] != 1:
        raise ValueError('metadata version domain')
    validate_lock_fields(lock_raw)
    try:
        lock=toml_inventory(lock_raw.decode('utf-8'),lock=True)
    except (UnicodeError,ValueError,TypeError,KeyError) as error:
        raise ValueError('invalid lock inventory') from error
    if not isinstance(lock,dict):
        raise ValueError('lock object required')
    locks,packages=index(lock.get('package')),index(metadata.get('packages'),metadata=True)
    selected=[]
    for key in sorted(union):
        if key not in locks or key not in packages:
            raise ValueError('tree package absent in exact lock/metadata')
        lock,package=locks[key],packages[key]
        if lock.get('source') != package.get('source'):
            raise ValueError('source identity mismatch')
        selected.append(dict(name=key[0],version=key[1],metadata_id=package['id'],source=package.get('source'),lock_checksum=lock.get('checksum'),manifest_path=package['manifest_path'],declared_license=package.get('license'),license_file=package.get('license_file'),in_app_graph=key in app,in_engine_graph=key in engine))
    return dict(target=TARGET,status='REVIEW_REQUIRED',proof_kind='ci-cargo-graph-format-binding',release_ready=False,legal_coverage_complete=False,remote_execution_verified=False,normal_build_proc_macro_superset=True,exact_linked_code_inventory=False,resolved_packages=selected,union_count=len(union),app_count=len(app),engine_count=len(engine),pins={name:hashlib.sha256(raw).hexdigest() for name,raw in [('app-tree',app_raw),('engine-tree',engine_raw),('metadata',metadata_raw),('Cargo.lock',lock_raw)]})

def main():
    p=argparse.ArgumentParser()
    for name in ['app-tree','engine-tree','metadata','lock','output']:
        p.add_argument('--'+name,type=Path,required=True)
    a=p.parse_args()
    raw=[bounded(a.app_tree,2*1024*1024),bounded(a.engine_tree,2*1024*1024),bounded(a.metadata,8*1024*1024),bounded(a.lock,4*1024*1024)]
    result=classify(*raw)
    publish(a.output,result)

def publish(output,result):
    output=Path(output).absolute()
    if '..' in output.parts or any(p.is_symlink() for p in [output,*output.parents]) or output.exists() or not output.parent.is_dir():
        raise ValueError('fresh output with nonsymlink ancestors required')
    try:
        with output.open('x',encoding='utf-8') as f:
            json.dump(result,f,ensure_ascii=False,sort_keys=True,indent=2)
            f.write('\n')
    except OSError as error:
        raise ValueError('output publication failed; partial evidence retained') from error

if __name__=='__main__':
    main()
