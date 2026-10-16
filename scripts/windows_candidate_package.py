#!/usr/bin/env python3
"""Create-only deterministic Windows GNU native CPU DEVELOPMENT candidate ZIP."""
import argparse, hashlib, json, os, re, struct, zipfile
try:
    import tomllib
except ImportError:
    tomllib=None
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
LIMIT=256*1024*1024

def toml_inventory(text,lock=False):
    if tomllib is not None:return tomllib.loads(text)
    # Python3.10 stdlib fallback only extracts single-line inventory fields.
    # It never treats this limited reader as complete TOML or license approval.
    records=[];current=None
    for line in text.splitlines():
        if line.strip()==('[[package]]' if lock else '[package]'):
            current={};records.append(current);continue
        if line.lstrip().startswith('['):current=None;continue
        if current is None:continue
        match=re.fullmatch(r'\s*(name|version|source|checksum|license|license-file|repository)\s*=\s*("(?:[^"\\]|\\.)*"|\'[^\']*\')\s*(?:#.*)?',line)
        if match:
            value=match.group(2)
            current[match.group(1)]=json.loads(value) if value.startswith('"') else value[1:-1]
    if lock:
        if not records or any('name' not in p or 'version' not in p for p in records):raise ValueError('lock inventory string unsupported')
        return {'package':records}
    return {'package':records[0] if records else {}}

def plain(path):
    path=Path(os.path.abspath(path))
    if any(p.is_symlink() for p in [path,*path.parents]):raise ValueError('symlink input/output rejected')
    return path

def digest(path):
    size=path.stat().st_size
    if not 0<=size<=LIMIT+20*1024*1024:raise ValueError('hash input bound')
    h=hashlib.sha256();total=0
    with path.open('rb') as f:
        while data:=f.read(65536):
            total+=len(data)
            if total>size:raise ValueError('hash input grew')
            h.update(data)
    if total!=size or path.stat().st_size!=size:raise ValueError('hash input size changed')
    return h.hexdigest()

def bounded(path,maximum=8*1024*1024):
    if not path.is_file() or path.stat().st_size>maximum:raise ValueError('bounded regular file required')
    with path.open('rb') as source:
        content=source.read(maximum+1)
    if len(content)>maximum:raise ValueError('bounded file stream grew')
    return content

def pe(path,expected_subsystem=2):
    size=path.stat().st_size
    if not 64<=size<=LIMIT:raise ValueError('PE size outside candidate bound')
    with path.open('rb') as f:
        def read(offset,length):
            if offset<0 or length<0 or offset+length>size:raise ValueError('PE range outside file')
            f.seek(offset);data=f.read(length)
            if len(data)!=length:raise ValueError('short PE read')
            return data
        if read(0,2)!=b'MZ':raise ValueError('DOS signature')
        off=struct.unpack('<I',read(60,4))[0]
        if read(off,4)!=b'PE\0\0':raise ValueError('PE signature')
        machine,count,_,_,_,optional,_=struct.unpack('<HHIIIHH',read(off+4,20))
        if machine!=0x8664 or not 1<=count<=96 or optional<128 or optional>4096:raise ValueError('PE64 AMD64 headers required')
        header=read(off+24,optional)
        if struct.unpack_from('<H',header)[0]!=0x20b:raise ValueError('PE32+ required')
        subsystem=struct.unpack_from('<H',header,68)[0]
        if expected_subsystem not in (2,3) or subsystem!=expected_subsystem:raise ValueError('exact Windows executable subsystem required')
        directories=struct.unpack_from('<I',header,108)[0]
        if directories<2:raise ValueError('PE import data directory missing')
        import_rva,import_size=struct.unpack_from('<II',header,120)
        sections=[]
        for i in range(count):
            sec=read(off+24+optional+40*i,40)
            virtual_size,rva,raw_size,raw=struct.unpack_from('<IIII',sec,8)
            if raw+raw_size>size:raise ValueError('section file range')
            sections.append((rva,virtual_size,raw,raw_size))
        def offset(rva,length):
            matches=[raw+rva-start for start,_,raw,rawsize in sections if start<=rva and rva-start+length<=rawsize]
            if len(matches)!=1:raise ValueError('ambiguous/unbacked import RVA')
            return matches[0]
        imports=[]
        if import_rva:
            if import_size<20 or import_size>65536:raise ValueError('import descriptor bound')
            for i in range(min(import_size//20,128)):
                d=struct.unpack('<IIIII',read(offset(import_rva+20*i,20),20))
                if d==(0,0,0,0,0):break
                chars=[]
                for n in range(256):
                    byte=read(offset(d[3]+n,1),1)[0]
                    if byte==0:break
                    if not 32<=byte<127:raise ValueError('DLL name ASCII bound')
                    chars.append(byte)
                else:raise ValueError('DLL name length')
                if not chars:raise ValueError('empty import DLL')
                name=bytes(chars).decode('ascii')
                if '/' in name or '\\' in name or ':' in name:raise ValueError('DLL leaf required')
                imports.append(name)
            else:raise ValueError('import descriptor terminator missing')
        return {'machine':'AMD64','format':'PE32+','subsystem':'WindowsGUI' if expected_subsystem==2 else 'WindowsConsole','import_dlls':sorted(set(imports),key=lambda name:(name.lower(),name)),'minimum_os_verified':False,'delay_imports_audited':False,'dll_runtime_presence_verified':False}

def proof_adapter(proof):
    if 'records' in proof:
        if proof.get('scope')!='default' or proof.get('feature')!='cpu' or proof.get('complete') is not True:
            raise ValueError('default CPU terminal proof required; fixture proof rejected')
        records=proof['records']
        if not isinstance(records,list) or len(records)!=4:raise ValueError('exact four terminal records required')
        operations={}
        for record in records:
            if not isinstance(record,dict) or record.get('operation') in operations:raise ValueError('duplicate/invalid proof operation')
            operations[record.get('operation')]=record
        baseline=proof.get('source_final')
        artifact={'sha256':proof.get('binary_sha'),'bytes':proof.get('binary_bytes')}
        format_name='final-default-records-v1'
    else:
        if proof.get('cargo_checks_complete') is not True or proof.get('passed') is not True:
            raise ValueError('original workspace terminal proof required')
        operations=proof.get('actual_operations',{})
        baseline=proof.get('source_baseline')
        artifact=proof.get('cpu_artifact')
        format_name='workspace-actual-operations-v1'
    if not isinstance(operations,dict) or not {'fmt','clippy','test','release'}.issubset(operations):raise ValueError('four actual operations required')
    expected={
        'fmt':['fmt','--all','--','--check'],
        'clippy':['clippy','--workspace','--all-targets','--locked','--features','cpu','--','-D','warnings'],
        'test':['test','--workspace','--locked','--features','cpu'],
        'release':['build','--workspace','--release','--locked','--features','cpu'],
    }
    for name in expected:
        op=operations[name]
        if not isinstance(op,dict) or op.get('passed') is not True or op.get('source_unchanged') is not True or type(op.get('exit_code')) is not int or op['exit_code']!=0 or op.get('collector_failure') is not None:
            raise ValueError('actual Windows checks not all PASS')
        if op.get('input_desktop_unchanged') is not True or op.get('private_desktop_reported') is not True:raise ValueError('private native host proof required')
        argv=op.get('argv')
        if not isinstance(argv,list) or len(argv)<4 or argv[3:]!=expected[name]:raise ValueError('exact default CPU command required; fixture/extra features rejected')
        if op.get('source_before')!=baseline or op.get('source_after')!=baseline:raise ValueError('operation snapshots differ from terminal source')
    if not isinstance(baseline,dict) or not 1<=len(baseline)<=6000:raise ValueError('bounded exact source snapshot required')
    if not isinstance(artifact,dict) or set(artifact)!={'sha256','bytes'} or not isinstance(artifact['sha256'],str) or len(artifact['sha256'])!=64 or any(c not in '0123456789abcdef' for c in artifact['sha256']) or type(artifact['bytes']) is not int or not 64<=artifact['bytes']<=LIMIT:raise ValueError('exact artifact proof required')
    return operations,baseline,artifact,format_name

def checks(path):
    raw=bounded(path)
    proof=json.loads(raw)
    _,baseline,artifact,format_name=proof_adapter(proof)
    for name,sha in baseline.items():
        if not isinstance(name,str) or not isinstance(sha,str) or len(sha)!=64 or any(c not in '0123456789abcdef' for c in sha):raise ValueError('source pin shape invalid')
        source=plain(ROOT/name) if not Path(name).is_absolute() else plain(name)
        if not source.is_relative_to(ROOT) or digest(source)!=sha:raise ValueError('current source differs from checks')
    result={'sha256':hashlib.sha256(raw).hexdigest(),'recorded_utc':proof.get('recorded_utc'),'operations':['fmt','clippy','test','release'],'source_files':len(baseline),'cpu_artifact':artifact,'proof_format':format_name}
    if 'local_install_artifact' in proof:result['local_install_artifact']=companion_artifact(proof)
    return result

def companion_artifact(proof):
    # Optional companion never inferred from legacy CPU-only evidence.
    if 'records' in proof:raise ValueError('companion requires canonical workspace proof')
    value=proof.get('local_install_artifact')
    if not isinstance(value,dict) or set(value)!={'sha256','bytes'} or not isinstance(value['sha256'],str) or re.fullmatch('[0-9a-f]{64}',value['sha256']) is None or type(value['bytes']) is not int or not 64<=value['bytes']<=LIMIT:
        raise ValueError('exact local install artifact proof required')
    return value

def bounded_binary_digest(path,expected_bytes):
    if not path.is_file() or not 64<=expected_bytes<=LIMIT or path.stat().st_size!=expected_bytes:
        raise ValueError('binary size mismatch')
    h=hashlib.sha256();total=0
    with path.open('rb') as source:
        while data:=source.read(65536):
            total+=len(data)
            if total>expected_bytes:raise ValueError('binary input grew')
            h.update(data)
    if total!=expected_bytes or path.stat().st_size!=expected_bytes:raise ValueError('binary input short or changed')
    return h.hexdigest()

class BudgetEntries(dict):
    """Bound retained archive bytes before each read or encoded chunk."""
    def __init__(self):super().__init__();self.used=0
    def reserve(self,name,size):
        if name in self or len(self)>=4095 or size<0 or self.used+size>16*1024*1024:raise ValueError('incremental notice entry/byte budget exceeded')
    def add_bytes(self,name,content):
        self.reserve(name,len(content));self[name]=content;self.used+=len(content)
    def add_file(self,name,path,maximum=1024*1024):
        path=plain(path);size=path.stat().st_size
        if not path.is_file() or size>maximum:raise ValueError('bounded notice regular file required')
        self.reserve(name,size)
        content=bounded(path,min(maximum,16*1024*1024-self.used))
        if len(content)!=size:raise ValueError('notice changed size during read')
        self.add_bytes(name,content);return content
    def add_json(self,name,value):
        self.reserve(name,0);content=bytearray()
        for chunk in json.JSONEncoder(ensure_ascii=False,sort_keys=True,indent=2).iterencode(value):
            data=chunk.encode()
            self.reserve(name,len(content)+len(data)+1)
            content.extend(data)
        content.extend(b'\n');self.add_bytes(name,bytes(content))

def bounded_children(path,maximum):
    values=[]
    for child in path.iterdir():
        if len(values)>=maximum:raise ValueError('cache directory enumeration bound exceeded')
        values.append(child)
    return values

def embedded_font_inventory(source,files):
    mapping=[('Hack-Regular.ttf','Hack-Regular.txt'),('NotoEmoji-Regular.ttf','OFL.txt'),('Ubuntu-Light.ttf','UFL.txt'),('emoji-icon-font.ttf','emoji-icon-font-mit-license.txt')]
    code=bounded(plain(source/'src/lib.rs'),1024*1024)
    assets=[]
    for font,notice in mapping:
        if f'include_bytes!("../fonts/{font}")'.encode() not in code:raise ValueError('exact embedded font source declaration changed')
        original=plain(source/'fonts'/font);size=original.stat().st_size
        if not original.is_file() or not 1<=size<=8*1024*1024:raise ValueError('embedded font source size bound')
        h=hashlib.sha256();total=0
        with original.open('rb') as stream:
            while data:=stream.read(65536):
                total+=len(data)
                if total>size:raise ValueError('embedded font source grew')
                h.update(data)
        if total!=size:raise ValueError('embedded font source short')
        actual_notice=plain(source/'fonts'/notice)
        if not actual_notice.is_file():raise ValueError('exact embedded font notice missing')
        if actual_notice not in files:files.append(actual_notice)
        assets.append({'source_file':'fonts/'+font,'source_bytes':size,'source_sha256':h.hexdigest(),'notice_entry':'licenses/epaint_default_fonts-0.33.3/'+notice,'mapping_basis':'fixed exact local package font/source declarations; complete copyright coverage unverified','exact_binary_payload_verified':False})
    return {'source_declared_presence':True,'source_declarations_sha256':hashlib.sha256(code).hexdigest(),'assets':assets,'notice_coverage_complete':False,'exact_linked_payload_audited':False}

ICED_COMMIT='3997291f318a8bc06fa522f5579836fb3feb94df'
ICED_COMPONENTS={'iced_'+name+'-0.14.0' for name in ['core','debug','futures','graphics','program','renderer','runtime','wgpu']}
def validate_cached_iced_grant(record):
    binding=record.get('cache_binding')
    if not isinstance(binding,dict) or set(binding)!={'package_path','manifest_sha256','vcs_sha256','grant_sha256'}:raise ValueError('exact local Iced grant binding required')
    relative=binding['package_path']
    if not isinstance(relative,str) or not relative or len(relative)>4096 or Path(relative).is_absolute() or any(part in ['', '.', '..'] for part in relative.split('/')) or '\\' in relative or ':' in relative or '\x00' in relative:raise ValueError('relative original Iced cache path required')
    for pin in [binding['manifest_sha256'],binding['vcs_sha256'],binding['grant_sha256'],record.get('sha256')]:
        if not isinstance(pin,str) or re.fullmatch('[0-9a-f]{64}',pin) is None:raise ValueError('exact Iced SHA256 pin required')
    donor=plain(ROOT/relative)
    cache_roots=[ROOT/'.local-tools/windows/cargo-home/registry/src',ROOT/'.local-tools/cargo/registry/src']
    if donor.name!='iced-0.14.0' or not any(donor.is_relative_to(p) for p in cache_roots):raise ValueError('exact local Iced donor package required')
    cargo=plain(donor/'Cargo.toml');vcsfile=plain(donor/'.cargo_vcs_info.json');grant=plain(donor/'LICENSE')
    for path,maximum in [(cargo,1024*1024),(vcsfile,65536),(grant,65536)]:
        if not path.is_file() or not 1<=path.stat().st_size<=maximum:raise ValueError('original Iced regular file pre-hash size bound')
    if digest(cargo)!=binding['manifest_sha256'] or digest(vcsfile)!=binding['vcs_sha256'] or digest(grant)!=binding['grant_sha256'] or record['sha256']!=binding['grant_sha256']:raise ValueError('original cached Iced grant pins changed')
    pkg=toml_inventory(bounded(cargo,1024*1024).decode())['package'];vcs=json.loads(bounded(vcsfile,65536))
    if pkg.get('name')!='iced' or pkg.get('version')!='0.14.0' or pkg.get('license')!='MIT' or pkg.get('repository','').rstrip('/')!='https://github.com/iced-rs/iced' or vcs.get('git',{}).get('sha1')!=ICED_COMMIT or vcs.get('path_in_vcs')!='':raise ValueError('original cached Iced grant metadata mismatch')
    if record['repository']!='iced-rs/iced' or record['commit']!=ICED_COMMIT or record['leaf']!='LICENSE':raise ValueError('wrong local Iced grant revision')

GPU_NOTICE_GRANTS = {('zakarumych/gpu-descriptor', 'bb28b4e55b742e0ea3e7a0968e7ee6287ba0e06a', 'COPYING', 352, 'f72143f7e9d9fb8bd336b139fa22f825a299591c29f2f7338c5566347b137613'), ('zakarumych/gpu-descriptor', '9f2fb9546591322e7b77ee1e79385e01effd85e4', 'COPYING', 352, 'f72143f7e9d9fb8bd336b139fa22f825a299591c29f2f7338c5566347b137613'), ('zakarumych/gpu-descriptor', '9f2fb9546591322e7b77ee1e79385e01effd85e4', 'license/MIT', 1080, '5226e27d5fbc683db56c52853cdb9d15b69abadb525a60f95c862c2dbd77cdc8'), ('zakarumych/gpu-alloc', '7ddd1fe576fcfc503347eb97e476a47581479f22', 'license/MIT', 1075, 'f774bf68151714985324d90adbd59e8f271b7ba518608d871b6ba16464692879'), ('zakarumych/gpu-alloc', '7ddd1fe576fcfc503347eb97e476a47581479f22', 'COPYING', 347, '12121dfa588677007848ab922a45a1e3248868f6ff063598fceb3141d7897314'), ('zakarumych/gpu-alloc', '61020bdcb4eb04fca6bbf748aeb9570e859236e0', 'COPYING', 347, '12121dfa588677007848ab922a45a1e3248868f6ff063598fceb3141d7897314'), ('zakarumych/gpu-alloc', '61020bdcb4eb04fca6bbf748aeb9570e859236e0', 'license/MIT', 1075, 'f774bf68151714985324d90adbd59e8f271b7ba518608d871b6ba16464692879'), ('zakarumych/gpu-descriptor', 'bb28b4e55b742e0ea3e7a0968e7ee6287ba0e06a', 'license/MIT', 1080, '5226e27d5fbc683db56c52853cdb9d15b69abadb525a60f95c862c2dbd77cdc8')}
GPU_REVISION_COMPONENTS = {'gpu-alloc-0.6.2': ('zakarumych/gpu-alloc', '61020bdcb4eb04fca6bbf748aeb9570e859236e0', 'gpu-alloc'), 'gpu-alloc-types-0.3.1': ('zakarumych/gpu-alloc', '7ddd1fe576fcfc503347eb97e476a47581479f22', 'types'), 'gpu-descriptor-0.3.2': ('zakarumych/gpu-descriptor', 'bb28b4e55b742e0ea3e7a0968e7ee6287ba0e06a', 'gpu-descriptor'), 'gpu-descriptor-types-0.2.0': ('zakarumych/gpu-descriptor', '9f2fb9546591322e7b77ee1e79385e01effd85e4', 'types')}

def upstream_manifest():
    path=plain(ROOT/'docs/licenses/upstream/manifest.json')
    value=json.loads(bounded(path,1024*1024))
    if not isinstance(value,dict) or set(value)!={'schema','sources','packages','coverage_complete'} or value['schema']!=1 or value['coverage_complete'] is not False:raise ValueError('explicit upstream notice schema required')
    if not isinstance(value['sources'],dict) or len(value['sources'])>32 or not isinstance(value['packages'],dict) or len(value['packages'])>64:raise ValueError('upstream notice mapping bound')
    for source in value['sources'].values():
        base={'repository','commit','leaf','file','bytes','sha256','url'}
        if set(source)==base|{'cache_binding'}:
            validate_cached_iced_grant(source)
        elif set(source)!=base:raise ValueError('explicit notice source fields')
        elif source['leaf'] == 'LICENSE':
            if (source['repository'],source['commit'],source['bytes'],source['sha256']) not in [('DoumanAsh/clipboard-win', '3b27cf2bfd1adcfa6e0264eb51c1025ddaf0f342', 1338, 'c9bff75738922193e67fa726fa225535870d2aa1059f91452c411736284ad566'), ('iced-rs/iced', '0ecf60664df7b8ac7d7aef5f7279d5323027f693', 1073, 'fc9086ba4eba4b77e4c603906362742bc3715ffa2c7810c6074fe7780d3a0398'), ('iced-rs/iced', '54abf81d13fec06d4d9ac754b03ce2c3313e8a1f', 1073, 'fc9086ba4eba4b77e4c603906362742bc3715ffa2c7810c6074fe7780d3a0398'), ('iced-rs/iced', '38237dd294da3256c2ac4d3f8288fc73956c91e0', 1073, 'fc9086ba4eba4b77e4c603906362742bc3715ffa2c7810c6074fe7780d3a0398'), ('brendanzab/gl-rs', 'ea503e8d5fb6d73c6030e6191ce738cd3bf3433e', 11358, 'cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30'), ('brendanzab/gl-rs', 'f150967b1c44ae888e6676f93f639ebc82771bdc', 11358, 'cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30'), ('gfx-rs/rspirv', 'e91a5debc10c395b36db7721dfff6326d697a1e2', 11358, 'cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30')]:raise ValueError('exact original LICENSE grant required')
        elif source['leaf'] in ['COPYING', 'license/MIT']:
            if (source['repository'], source['commit'], source['leaf'], source['bytes'], source['sha256']) not in GPU_NOTICE_GRANTS:raise ValueError('exact GPU notice grant required')
        elif source['leaf'] == 'LICENSE.chromium':
            if (source['repository'],source['commit'],source['bytes'],source['sha256']) != ('AccessKit/accesskit','5801c4a49ff9b2c9ec3f77b3c31272d0da4c5cd3',1559,'845022e0c1db1abb41a6ba4cd3c4b674ec290f3359d9d3c78ae558d4c0ed9308'):raise ValueError('exact AccessKit Chromium notice required')
        elif source['leaf'] not in ['LICENSE-MIT','LICENSE-APACHE']:raise ValueError('explicit notice source fields')
        local=plain(ROOT/source['file'])
        if not local.is_relative_to(ROOT/'docs/licenses/upstream') or not 1<=source['bytes']<=65536 or local.stat().st_size!=source['bytes'] or digest(local)!=source['sha256']:raise ValueError('original notice bytes/digest mismatch')
        if source['url']!=f"https://raw.githubusercontent.com/{source['repository']}/{source['commit']}/{source['leaf']}":raise ValueError('exact upstream notice URL mismatch')
    return value

ICED_REVISION_COMPONENTS={'iced_tiny_skia-0.14.1': ('0ecf60664df7b8ac7d7aef5f7279d5323027f693', 'tiny_skia'), 'iced_widget-0.14.2': ('54abf81d13fec06d4d9ac754b03ce2c3313e8a1f', 'widget'), 'iced_winit-0.14.1': ('38237dd294da3256c2ac4d3f8288fc73956c91e0', 'winit')}

GL_REVISION_COMPONENTS={'gl_generator-0.14.0': ('ea503e8d5fb6d73c6030e6191ce738cd3bf3433e', '4daaaac088657de6c5b6223ca9f890b8b5621f073068c65c0d759b0625d36280'), 'khronos_api-3.1.0': ('f150967b1c44ae888e6676f93f639ebc82771bdc', '4a1ac66b226935a2ce69cca4c1c56be36cea83a5da382514db5ea967568e3ca9')}

def mapped_notices(value,key,source,pkg,lock_checksum=None):
    mapping=value['packages'].get(key)
    required={name+'-0.33.3' for name in ['ecolor','egui','eframe','egui-wgpu','egui-winit','egui_glow','emath','epaint','epaint_default_fonts']}|{'accesskit-0.21.1','accesskit_consumer-0.31.0','accesskit_macos-0.22.2','accesskit_unix-0.17.2','accesskit_windows-0.29.2','accesskit_winit-0.29.2'}
    required|=ICED_COMPONENTS
    required|={'clipboard-win-5.4.1','winapi-x86_64-pc-windows-gnu-0.4.0'}
    required|=set(ICED_REVISION_COMPONENTS)
    required|=set(GL_REVISION_COMPONENTS)
    required|=set(GPU_REVISION_COMPONENTS)
    required|={'spirv-0.3.0+sdk-1.3.268.0'}
    if mapping is None:
        if key in required:raise ValueError('required explicit upstream package mapping missing')
        return []
    expected_fields={'repository','commit','path_in_vcs','sources'}
    if key=='winapi-x86_64-pc-windows-gnu-0.4.0':expected_fields.add('payload_binding')
    max_sources=3 if key=='accesskit_windows-0.29.2' else 2
    if set(mapping)!=expected_fields or not isinstance(mapping['sources'],list) or not 1<=len(mapping['sources'])<=max_sources:raise ValueError('explicit package notice mapping fields')
    if key=='accesskit_windows-0.29.2' and ((mapping['repository'],mapping['commit'],mapping['path_in_vcs']) != ('AccessKit/accesskit','5801c4a49ff9b2c9ec3f77b3c31272d0da4c5cd3','platforms/windows') or [value['sources'].get(i,{}).get('leaf') for i in mapping['sources']] != ['LICENSE-MIT','LICENSE-APACHE','LICENSE.chromium']):raise ValueError('exact Windows AccessKit notice trio required')
    repo=pkg.get('repository','').replace('http://','https://').rstrip('/').split('/tree/')[0]
    if repo!='https://github.com/'+mapping['repository']:raise ValueError('cached repository notice mapping mismatch')
    if key in ICED_COMPONENTS and (pkg.get('name')+'-'+pkg.get('version','')!=key or pkg.get('license')!='MIT' or mapping['commit']!=ICED_COMMIT):raise ValueError('exact Iced target name/version/license/commit required')
    if key == 'clipboard-win-5.4.1' and (pkg.get('name') != 'clipboard-win' or pkg.get('version') != '5.4.1' or pkg.get('license') != 'BSL-1.0'):raise ValueError('exact clipboard target metadata required')
    if key in ICED_REVISION_COMPONENTS and (pkg.get('name','')+'-'+pkg.get('version','') != key or pkg.get('license') != 'MIT' or mapping['repository'] != 'iced-rs/iced' or (mapping['commit'],mapping['path_in_vcs']) != ICED_REVISION_COMPONENTS[key]):raise ValueError('exact distinct Iced revision component required')
    if key in GPU_REVISION_COMPONENTS and (pkg.get('name','')+'-'+pkg.get('version','') != key or pkg.get('license') != 'MIT OR Apache-2.0' or (mapping['repository'], mapping['commit'], mapping['path_in_vcs']) != GPU_REVISION_COMPONENTS[key] or [value['sources'].get(i, {}).get('leaf') for i in mapping['sources']] != ['COPYING', 'license/MIT']):raise ValueError('exact GPU revision and MIT grant pair required')
    if key == 'spirv-0.3.0+sdk-1.3.268.0' and (pkg.get('name') != 'spirv' or pkg.get('version') != '0.3.0+sdk-1.3.268.0' or pkg.get('license') != 'Apache-2.0' or (mapping['repository'], mapping['commit'], mapping['path_in_vcs']) != ('gfx-rs/rspirv', 'e91a5debc10c395b36db7721dfff6326d697a1e2', 'spirv') or len(mapping['sources']) != 1):raise ValueError('exact SPIR-V root notice binding required')
    if key in GL_REVISION_COMPONENTS:
        expected_commit, original_manifest_sha=GL_REVISION_COMPONENTS[key]
        original_manifest=plain(source/'Cargo.toml.orig')
        if pkg.get('name','')+'-'+pkg.get('version','') != key or pkg.get('license') != 'Apache-2.0' or mapping['repository'] != 'brendanzab/gl-rs' or mapping['commit'] != expected_commit or mapping['path_in_vcs'] is not None or hashlib.sha256(bounded(original_manifest,65536)).hexdigest() != original_manifest_sha:raise ValueError('exact gl-rs original package binding required')
    payload_proof=None
    if key=='winapi-x86_64-pc-windows-gnu-0.4.0':
        from windows_winapi_payload_binding import validate, bounded_hash
        archive=source.parent.parent.parent/'cache'/source.parent.name/(key+'.crate')
        payload_proof=validate(source,archive,ROOT/'docs/licenses/upstream/winapi-gnu-bump-x86_64-tree.json',ROOT/'docs/licenses/upstream/winapi-gnu-bump-root-tree.json',mapping,pkg,lock_checksum)
    else:
        vcs=json.loads(bounded(plain(source/'.cargo_vcs_info.json'),65536))
        if vcs.get('git',{}).get('sha1')!=mapping['commit'] or vcs.get('path_in_vcs')!=mapping['path_in_vcs']:raise ValueError('exact cached VCS notice mapping mismatch')
    output=[]
    for identifier in mapping['sources']:
        record=value['sources'].get(identifier)
        if record is None or record['repository']!=mapping['repository'] or record['commit']!=mapping['commit']:raise ValueError('package notice commit/repository mismatch')
        if payload_proof is not None:
            original=payload_proof['root_license_blobs'].get(record['leaf'])
            size,blob=bounded_hash(plain(ROOT/record['file']),65536,git=True)
            if original is None or original['size']!=size or original['sha']!=blob:raise ValueError('original root grant Git blob mismatch')
        output.append((record['leaf'],plain(ROOT/record['file']),record))
    return output

def patched_source(package):
    expected={'egui-0.33.3':'vendor/egui','winit-0.30.13':'vendor/winit','accesskit_windows-0.29.2':'vendor/accesskit_windows'}
    key=package['name']+'-'+package['version']
    if key not in expected:return None
    if 'source' in package: return None
    cargo=bounded(ROOT/'Cargo.toml',1024*1024).decode()
    patch=cargo.split('[patch.crates-io]',1)
    if len(patch)!=2:raise ValueError('explicit patch section missing')
    section=patch[1].split('\n[',1)[0]
    pattern=r'(?m)^'+re.escape(package['name'])+r'\s*=\s*\{\s*path\s*=\s*"'+re.escape(expected[key])+r'"\s*\}\s*$'
    if not re.search(pattern,section):raise ValueError('exact supported vendor patch path mismatch')
    return plain(ROOT/expected[key])

WORKSPACE_NOTICE_SHA = 'b59481a07033d1ed646f8feab084fe46481c83bfc3c0744ac3eea0d2cfdf8b70'
WORKSPACE_NOTICE_BYTES = 1075
WORKSPACE_NOTICE_PATHS = {'dlpad': 'crates/app', 'dlpad-core': 'crates/core', 'dlpad-engine': 'crates/engine'}

def workspace_notice(package):
    name = package.get('name')
    if name not in WORKSPACE_NOTICE_PATHS:return None
    if package.get('version') != '0.1.0' or 'source' in package:raise ValueError('exact workspace notice package required')
    root_manifest = bounded(plain(ROOT/'Cargo.toml'), 65536).decode()
    section = root_manifest.split('[workspace.package]', 1)
    if len(section) != 2:raise ValueError('workspace package license declaration missing')
    section = section[1].split('\n[', 1)[0]
    if not re.search(r'(?m)^license\s*=\s*"MIT"\s*$', section) or not re.search(r'(?m)^version\s*=\s*"0\.1\.0"\s*$', section):raise ValueError('exact workspace MIT/version required')
    path = plain(ROOT/WORKSPACE_NOTICE_PATHS[name]/'Cargo.toml')
    manifest = bounded(path, 65536).decode()
    if toml_inventory(manifest).get('package', {}).get('name') != name or not re.search(r'(?m)^license\.workspace\s*=\s*true\s*$', manifest) or not re.search(r'(?m)^version\.workspace\s*=\s*true\s*$', manifest):raise ValueError('exact workspace inherited license/version required')
    license_path = plain(ROOT/'LICENSE')
    raw = bounded(license_path, 65536)
    if len(raw) != WORKSPACE_NOTICE_BYTES or hashlib.sha256(raw).hexdigest() != WORKSPACE_NOTICE_SHA:raise ValueError('exact workspace MIT original bytes/digest required')
    text = raw.decode()
    if not text.startswith('MIT License\n\nCopyright (c) 2026 dlpad contributors\n\nPermission is hereby granted') or 'THE SOFTWARE IS PROVIDED "AS IS"' not in text:raise ValueError('workspace MIT original notice missing')
    return license_path

def licenses():
    lock=toml_inventory(bounded(ROOT/'Cargo.lock').decode(),lock=True)
    registry=ROOT/'.local-tools/cargo/registry/src'
    other=ROOT/'.local-tools/windows/cargo-home/registry/src'
    cache_roots=[p for parent in [registry,other] if parent.exists() for p in bounded_children(parent,128) if p.is_dir() and not p.is_symlink()]
    upstream=upstream_manifest()
    records=[];entries=BudgetEntries();missing=[]
    if len(lock.get('package',[]))>4096:raise ValueError('lock package count bound')
    for package in lock.get('package',[]):
        name,version=package['name'],package['version']
        if not all(isinstance(value,str) and 1<=len(value)<=256 and '/' not in value and '\\' not in value for value in [name,version]):raise ValueError('package name/version bound')
        key=f'{name}-{version}'
        workspace_file=workspace_notice(package)
        if workspace_file is not None:
            entry=f'licenses/{key}/LICENSE'
            content=entries.add_file(entry,workspace_file,65536)
            if len(content) != WORKSPACE_NOTICE_BYTES or hashlib.sha256(content).hexdigest() != WORKSPACE_NOTICE_SHA:raise ValueError('workspace notice changed during package copy')
            records.append({'name':name,'version':version,'declared_license':'MIT','source_kind':'explicit-workspace-path','workspace_path':WORKSPACE_NOTICE_PATHS[name],'notices':[{'entry':entry,'sha256':hashlib.sha256(content).hexdigest()}],'transitive_embedded_notice_review_complete':False})
            continue
        candidates=[p/key for p in cache_roots if (p/key/'Cargo.toml').is_file()]
        vendor=patched_source(package)
        if vendor is not None or (package.get('source','').startswith('registry+') and candidates):
            source=vendor if vendor is not None else plain(candidates[0]);manifest=toml_inventory(bounded(source/'Cargo.toml',1024*1024).decode());pkg=manifest.get('package',{})
            if pkg.get('name')!=name or pkg.get('version')!=version:raise ValueError('exact inventory package metadata mismatch')
            selected=mapped_notices(upstream,key,source,pkg,package.get('checksum'))
            files=[f for f in bounded_children(source,4096) if f.is_file() and f.name.upper().startswith(('LICENSE','COPYING','NOTICE'))]
            explicit=pkg.get('license-file')
            if explicit:
                file=plain(source/explicit)
                if not file.is_relative_to(source):raise ValueError('license path escapes package')
                if file.is_file() and file not in files:files.append(file)
            fonts=None
            if key=='epaint_default_fonts-0.33.3':
                if pkg.get('name')!=name or pkg.get('version')!=version:raise ValueError('exact font package metadata mismatch')
                fonts=embedded_font_inventory(source,files)
            record={'name':name,'version':version,'declared_license':pkg.get('license'),'notices':[],'transitive_embedded_notice_review_complete':False}
            if fonts is not None:record['embedded_fonts']=fonts
            if vendor is not None:record['source_kind']='explicit-path-patch';record['patch_path']=str(vendor.relative_to(ROOT))
            if selected:record['upstream_notice_sources']=[r for _,_,r in selected]
            for file in sorted(files):
                entry=f'licenses/{key}/{file.name}'
                content=entries.add_file(entry,file);record['notices'].append({'entry':entry,'sha256':hashlib.sha256(content).hexdigest()})
            for leaf,file,provenance in selected:
                entry=f'licenses/{key}/{leaf}'
                if entry in entries:
                    if hashlib.sha256(entries[entry]).hexdigest()!=provenance['sha256']:raise ValueError('vendor and original upstream notice differ')
                else:
                    content=entries.add_file(entry,file,65536);record['notices'].append({'entry':entry,'sha256':hashlib.sha256(content).hexdigest()})
            if vendor is not None and not files:raise ValueError('actual vendor notice missing')
            if not files and not selected:missing.append(key+': notice files absent')
            records.append(record)
        else:
            missing.append(key+': exact local source/license missing (workspace/git included)')
    if not any(r.get('name')=='epaint_default_fonts' and r.get('version')=='0.33.3' and r.get('embedded_fonts') for r in records):raise ValueError('exact embedded font source inventory unavailable')
    return entries,{'scope':'Cargo.lock inventory superset; final linked crate reachability/embedded notice review incomplete','records':records,'missing':missing,'complete':False}

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--exe',required=True,type=Path);ap.add_argument('--sha256',required=True);ap.add_argument('--checks-proof',required=True,type=Path)
    ap.add_argument('--local-install-exe',type=Path);ap.add_argument('--local-install-sha256');
    ap.add_argument('--output',required=True,type=Path);ap.add_argument('--allow-missing-licenses',action='store_true')
    a=ap.parse_args();exe=plain(a.exe);proof=plain(a.checks_proof);out=plain(a.output)
    if out.exists() or not out.is_relative_to(ROOT) or out.suffix!='.zip' or not out.parent.is_dir():raise ValueError('fresh project ZIP with existing parent required')
    if not exe.is_file() or not 64<=exe.stat().st_size<=LIMIT:raise ValueError('exe size bound before hashing')
    if len(a.sha256)!=64 or digest(exe)!=a.sha256.lower():raise ValueError('explicit exe SHA mismatch')
    actual_checks=checks(proof)
    if actual_checks['cpu_artifact']!={'sha256':a.sha256.lower(),'bytes':exe.stat().st_size}:raise ValueError('checks exact CPU artifact does not match candidate')
    companion=None;companion_meta=None
    if (a.local_install_exe is None)!=(a.local_install_sha256 is None):raise ValueError('paired companion arguments required')
    proof_sha=actual_checks['sha256']
    if a.local_install_exe is not None:
        companion=plain(a.local_install_exe)
        if re.fullmatch('[0-9a-f]{64}',a.local_install_sha256) is None:raise ValueError('exact companion SHA required')
        observed=companion_artifact(actual_checks)
        if observed!={'sha256':a.local_install_sha256,'bytes':companion.stat().st_size}:raise ValueError('companion differs from terminal proof')
        if exe.stat().st_size+observed['bytes']>LIMIT:raise ValueError('combined binary budget exceeded')
        if bounded_binary_digest(companion,observed['bytes'])!=a.local_install_sha256:raise ValueError('companion SHA mismatch')
        companion_meta={'entry':'dlpad-local-install.exe','role':'development-local-install','sha256':observed['sha256'],'bytes':observed['bytes'],'pe':pe(companion,3),'development_only':True,'installer_complete':False}
    binary=pe(exe);entries,inventory=licenses()
    if not a.allow_missing_licenses:raise ValueError('license inventory not release-complete; explicit development-only missing-license option required')
    entries.add_json('license-inventory.json',inventory)
    for name in ['rust-toolchain.toml','docs/CICD.md','docs/adr/0001-native-ui.md']:
        entries.add_file('build-evidence/'+Path(name).name,ROOT/name,8*1024*1024)
    manifest={'schema':1,'development_candidate':True,'release_ready':False,'entrypoint':'dlpad-cpu.exe','target':'x86_64-pc-windows-gnu','features':['cpu'],'toolchain_file_sha256':digest(ROOT/'rust-toolchain.toml'),'cargo_lock_sha256':digest(ROOT/'Cargo.lock'),'binary_sha256':a.sha256.lower(),'binary_bytes':exe.stat().st_size,'windows_checks':actual_checks,'pe':binary,'license_inventory_complete':False,'exact_checks_artifact_match':True,'independent_source_rebuild_verified':False,'stack_decision':'Proposed','unverified':['installer','signature','minimum Windows OS','actual site','IME','clean machine DLL runtime','all embedded/transitive notices','SBOM completeness'],'embedded_fonts':{'source_declared_presence':True,'exact_linked_payload_audited':False,'notice_coverage_complete':False,'inventory_package':'epaint_default_fonts-0.33.3'},'not_bundled':['tools','external_font_files','credentials','app data','user media'],'determinism_scope':'same immutable exe and same evidence; independent rebuild reproducibility unverified'}
    if companion_meta is not None:manifest['local_install_companion']=companion_meta
    entries.add_json('candidate-manifest.json',manifest)
    readme=b'DEVELOPMENT CANDIDATE ONLY. Not a signed installer or public release. Run dlpad-cpu.exe. External tools are not bundled. Read candidate-manifest.json and license-inventory.json.\n'
    if companion is not None:
        readme+=b'Includes dlpad-local-install.exe, a development-only local prepare/inspect/explicit removal CLI. Not an installer, automatic upgrade, active installation, signed publisher, or bundled external runtime. Explicit targets and external receipt are caller-selected; credentials and app data are not bundled.\n'
    entries.add_bytes('README.txt',readme)
    binaries={'dlpad-cpu.exe':(exe,manifest['binary_bytes'],manifest['binary_sha256'])}
    if companion is not None:binaries['dlpad-local-install.exe']=(companion,companion_meta['bytes'],companion_meta['sha256'])
    if set(entries)&set(binaries):raise ValueError('reserved executable entry collision')
    if sum(map(len,entries.values()))>16*1024*1024 or len(entries)+len(binaries)>4096:raise ValueError('bounded notice bundle required')
    try:
        with out.open('xb') as raw:
            with zipfile.ZipFile(raw,'w',compression=zipfile.ZIP_STORED,allowZip64=False) as archive:
                for name in sorted([*entries,*binaries]):
                    info=zipfile.ZipInfo(name,date_time=(1980,1,1,0,0,0));info.create_system=3;info.external_attr=(0o100755 if name in binaries else 0o100644)<<16
                    with archive.open(info,'w') as sink:
                        if name in binaries:
                            path,size,expected_sha=binaries[name]
                            with path.open('rb') as source:
                                total=0;stream_hash=hashlib.sha256()
                                while data:=source.read(65536):
                                    total+=len(data)
                                    if total>size:raise ValueError('input stream grew')
                                    stream_hash.update(data);sink.write(data)
                                if total!=size or stream_hash.hexdigest()!=expected_sha:raise ValueError('input stream short or hash changed')
                        else:sink.write(entries[name])
            raw.flush();os.fsync(raw.fileno())
        if digest(exe)!=a.sha256.lower():raise ValueError('exe changed during package')
        final_checks=checks(proof)
        if final_checks['sha256']!=proof_sha:raise ValueError('checks proof changed during package')
        if companion is not None:
            if bounded_binary_digest(companion,companion_meta['bytes'])!=companion_meta['sha256']:raise ValueError('companion changed during package')
            if companion_artifact(final_checks)!={'sha256':companion_meta['sha256'],'bytes':companion_meta['bytes']}:raise ValueError('companion proof changed')
        print(json.dumps({'status':'DEVELOPMENT_CANDIDATE','release_ready':False,'zip_sha256':digest(out),'zip_bytes':out.stat().st_size,'entry_count':len(entries)+len(binaries),'license_missing_count':len(inventory['missing'])}))
    except BaseException:
        # Preserve failed/partial output: a pathname is not deletion authority.
        raise
if __name__=='__main__':main()
