#!/usr/bin/env python3
"""Read-only bounded review of two development candidate ZIP archives."""
import argparse,hashlib,json,os,zipfile,struct
from pathlib import Path
LIMIT=256*1024*1024
DOC_LIMIT=16*1024*1024
FIXED={'dlpad-cpu.exe','README.txt','candidate-manifest.json','license-inventory.json','build-evidence/rust-toolchain.toml','build-evidence/CICD.md','build-evidence/0001-native-ui.md'}
def plain(path):
    path=Path(os.path.abspath(path))
    if any(p.is_symlink() for p in [path,*path.parents]) or not path.is_file():raise ValueError('original regular nonsymlink file required')
    return path

def sha(path):
    size=path.stat().st_size
    if not 0<=size<=LIMIT+DOC_LIMIT+4*1024*1024:raise ValueError('hash input bound')
    h=hashlib.sha256();total=0
    with path.open('rb') as f:
        while data:=f.read(65536):
            total+=len(data)
            if total>size:raise ValueError('hash input grew')
            h.update(data)
    if total!=size or path.stat().st_size!=size:raise ValueError('hash input size changed')
    return h.hexdigest()

def member_hash(z,info):
    if info.file_size<0 or info.file_size>(LIMIT if info.filename in {'dlpad-cpu.exe','dlpad-local-install.exe'} else DOC_LIMIT):raise ValueError('member bytes bound')
    h=hashlib.sha256();total=0
    with z.open(info) as f:
        while data:=f.read(65536):
            total+=len(data)
            if total>info.file_size:raise ValueError('stream exceeded declaration')
            h.update(data)
    if total!=info.file_size:raise ValueError('short member stream')
    return h.hexdigest()

def proof(path):
    if path.stat().st_size>8*1024*1024:raise ValueError('proof bound')
    with path.open('rb') as stream:raw=stream.read(8*1024*1024+1)
    if len(raw)>8*1024*1024:raise ValueError('proof stream bound')
    d=json.loads(raw)
    d['_proof_sha']=hashlib.sha256(raw).hexdigest()
    if 'records' in d:
        if d.get('scope')!='default' or d.get('feature')!='cpu' or d.get('complete') is not True:raise ValueError('default CPU proof required; fixture proof refused')
        records=d.get('records')
    else:
        if d.get('scope')!='Windows GNU CPU CLI; GUI/IME/MSVC/macOS/Linux keeper runtime unverified' or d.get('passed') is not True or d.get('cargo_checks_complete') is not True or d.get('source_unchanged') is not True:raise ValueError('canonical Windows CPU terminal proof required')
        operations=d.get('actual_operations')
        required={'fmt','clippy','test','release'}
        if not isinstance(operations,dict) or set(operations) not in [required,required|{'capability'}]:raise ValueError('exact canonical checks and optional capability required')
        if 'capability' in operations:
            capability=operations['capability']
            final=d.get('source_final')
            if not isinstance(capability,dict) or capability.get('operation')!='capability' or capability.get('passed') is not True or capability.get('source_unchanged') is not True or type(capability.get('exit_code')) is not int or capability['exit_code']!=0 or capability.get('collector_failure') is not None or capability.get('input_desktop_unchanged') is not True or capability.get('private_desktop_reported') is not True or capability.get('source_before')!=final or capability.get('source_after')!=final or capability.get('argv',[])[3:]!=['--version','--verbose']:raise ValueError('optional capability proof mismatch')
        records=[]
        for operation in ['fmt','clippy','test','release']:
            record=operations[operation]
            if not isinstance(record,dict) or record.get('operation')!=operation:raise ValueError('canonical operation key mismatch')
            records.append(record)
        if d.get('source_baseline')!=d.get('source_final'):raise ValueError('canonical terminal source changed')
        artifact=d.get('cpu_artifact')
        if not isinstance(artifact,dict) or set(artifact)!={'sha256','bytes'}:raise ValueError('canonical CPU artifact missing')
        d['binary_sha']=artifact['sha256'];d['binary_bytes']=artifact['bytes']
    if not isinstance(records,list) or len(records)!=4 or {r.get('operation') for r in records}!={'fmt','clippy','test','release'}:raise ValueError('four unique actual checks required')
    expected={'fmt':['fmt','--all','--','--check'],'clippy':['clippy','--workspace','--all-targets','--locked','--features','cpu','--','-D','warnings'],'test':['test','--workspace','--locked','--features','cpu'],'release':['build','--workspace','--release','--locked','--features','cpu']}
    final=d.get('source_final')
    if not isinstance(final,dict) or not 1<=len(final)<=6000:raise ValueError('source snapshot bound')
    for r in records:
        if r.get('passed') is not True or r.get('source_unchanged') is not True or type(r.get('exit_code')) is not int or r['exit_code']!=0 or r.get('collector_failure') is not None or r.get('input_desktop_unchanged') is not True or r.get('private_desktop_reported') is not True or r.get('source_before')!=final or r.get('source_after')!=final or r.get('argv',[])[3:]!=expected[r['operation']]:raise ValueError('actual default CPU proof mismatch')
    if 'local_install_artifact' in d:
        artifact=d['local_install_artifact']
        if not isinstance(artifact,dict) or set(artifact)!={'sha256','bytes'} or not isinstance(artifact['sha256'],str) or len(artifact['sha256'])!=64 or any(c not in '0123456789abcdef' for c in artifact['sha256']) or type(artifact['bytes'])is not int or not 64<=artifact['bytes']<=LIMIT:raise ValueError('strict companion proof artifact')
    return d

NESTED_MIT_PACKAGES = {'gpu-alloc-0.6.2', 'gpu-alloc-types-0.3.1', 'gpu-descriptor-0.3.2', 'gpu-descriptor-types-0.2.0'}

def safe_notice_member(name):
    parts = name.split('/')
    shape = len(parts) == 3 or (len(parts) == 4 and parts[1] in NESTED_MIT_PACKAGES and parts[2:] == ['license', 'MIT'])
    return shape and parts[0] == 'licenses' and all(part and part not in ['.', '..'] and '\\' not in part and ':' not in part and '\x00' not in part for part in parts)

def companion_pe(z,info):
    size=info.file_size
    if not 64<=size<=LIMIT:raise ValueError('PE size outside candidate bound')
    with z.open(info) as f:
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
        if subsystem!=3:raise ValueError('Windows console subsystem required')
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
        return {'machine':'AMD64','format':'PE32+','subsystem':'WindowsConsole','import_dlls':sorted(set(imports),key=lambda name:(name.lower(),name)),'minimum_os_verified':False,'delay_imports_audited':False,'dll_runtime_presence_verified':False}

def review(path,expected):
    if not 64<=path.stat().st_size<=LIMIT+DOC_LIMIT+4*1024*1024:raise ValueError('archive file size bound')
    with zipfile.ZipFile(path) as z:
        infos=z.infolist();names=[i.filename for i in infos]
        if not 1<=len(infos)<=4096 or len(set(names))!=len(names) or names!=sorted(names):raise ValueError('member duplicate/count/order')
        if not FIXED.issubset(names):raise ValueError('fixed member missing')
        companions={'dlpad-local-install.exe'}if 'dlpad-local-install.exe'in names else set()
        extra=set(names)-FIXED-companions
        for name in extra:
            if not safe_notice_member(name):raise ValueError('unexpected/unsafe member path')
        executables={'dlpad-cpu.exe'}|companions
        if sum(i.file_size for i in infos if i.filename in executables)>LIMIT:raise ValueError('combined executable bound')
        if sum(i.file_size for i in infos if i.filename not in executables)>DOC_LIMIT:raise ValueError('notice total bound')
        pins={}
        for info in infos:
            expected_mode=0o100755 if info.filename in executables else 0o100644
            if info.date_time!=(1980,1,1,0,0,0) or info.create_system!=3 or info.external_attr>>16!=expected_mode or info.compress_type!=zipfile.ZIP_STORED or info.flag_bits&1 or info.is_dir() or info.extra or info.comment:raise ValueError('fixed metadata/member type mismatch')
            pins[info.filename]=member_hash(z,info)
        if z.comment:raise ValueError('archive comment prohibited')
        def obj(name):
            if z.getinfo(name).file_size>DOC_LIMIT:raise ValueError('JSON bound')
            return json.loads(z.read(name))
        m=obj('candidate-manifest.json');inventory=obj('license-inventory.json')
        if m.get('schema')!=1 or m.get('development_candidate') is not True or m.get('release_ready') is not False or m.get('entrypoint')!='dlpad-cpu.exe' or m.get('target')!='x86_64-pc-windows-gnu' or m.get('features')!=['cpu'] or m.get('stack_decision')!='Proposed' or m.get('license_inventory_complete') is not False or inventory.get('complete') is not False:raise ValueError('development-only manifest policy mismatch')
        if m.get('binary_sha256')!=expected.get('binary_sha') or m.get('binary_bytes')!=expected.get('binary_bytes') or pins['dlpad-cpu.exe']!=expected.get('binary_sha') or z.getinfo('dlpad-cpu.exe').file_size!=expected.get('binary_bytes'):raise ValueError('original executable hash/bytes mismatch')
        companion=m.get('local_install_companion')
        if bool(companions)!=('local_install_companion'in m):raise ValueError('companion manifest/member presence mismatch')
        if companions:
            if 'records' in expected:raise ValueError('companion requires canonical workspace proof')
            artifact=expected.get('local_install_artifact')
            if not isinstance(artifact,dict) or set(artifact)!={'sha256','bytes'}or type(artifact['bytes'])is not int or not 64<=artifact['bytes']<=LIMIT or not isinstance(artifact['sha256'],str)or len(artifact['sha256'])!=64 or any(c not in '0123456789abcdef'for c in artifact['sha256']):raise ValueError('explicit companion whole proof required')
            if not isinstance(companion,dict)or set(companion)!={'entry','role','sha256','bytes','pe','development_only','installer_complete'}:raise ValueError('strict companion typed fields')
            if companion['entry']!='dlpad-local-install.exe'or companion['role']!='development-local-install'or companion['development_only']is not True or companion['installer_complete']is not False or type(companion['bytes'])is not int or companion['bytes']!=artifact['bytes']or companion['sha256']!=artifact['sha256']or pins['dlpad-local-install.exe']!=artifact['sha256']or z.getinfo('dlpad-local-install.exe').file_size!=artifact['bytes']:raise ValueError('exact companion proof/member mismatch')
            if companion['pe']!=companion_pe(z,z.getinfo('dlpad-local-install.exe')):raise ValueError('actual companion PE metadata mismatch')
        if m.get('windows_checks',{}).get('sha256')!=expected['_proof_sha']:raise ValueError('exact original proof digest mismatch')
        if m.get('toolchain_file_sha256')!=pins['build-evidence/rust-toolchain.toml'] or m.get('cargo_lock_sha256')!=expected['source_final'].get('Cargo.lock'):raise ValueError('original toolchain/lock pin mismatch')
        declared={}
        records=inventory.get('records')
        if not isinstance(records,list) or len(records)>4096 or not isinstance(inventory.get('missing'),list):raise ValueError('license inventory bound')
        for record in records:
            if record.get('transitive_embedded_notice_review_complete') is not False:raise ValueError('unsupported license completeness claim')
            for notice in record.get('notices',[]):
                entry=notice.get('entry')
                if entry in declared or entry not in extra or notice.get('sha256')!=pins[entry]:raise ValueError('license notice digest/member mismatch')
                if entry.split('/')[1]!=record.get('name','')+'-'+record.get('version',''):raise ValueError('license package mapping mismatch')
                declared[entry]=notice['sha256']
        if set(declared)!=extra:raise ValueError('undeclared notice member')
        return {'archive_sha256':sha(path),'bytes':path.stat().st_size,'entries':len(infos),'binary_sha256':pins['dlpad-cpu.exe'],'proof_sha256':expected['_proof_sha'],'license_missing_count':len(inventory['missing']),'release_ready':False,'local_install_companion':companion}

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--first',required=True,type=Path);p.add_argument('--second',required=True,type=Path);p.add_argument('--final-proof',required=True,type=Path);a=p.parse_args()
    original=plain(a.final_proof);d=proof(original)
    first,second=plain(a.first),plain(a.second)
    if first==second:raise ValueError('two distinct candidate archives required')
    before={p:sha(p) for p in [first,second]}
    before[original]=d['_proof_sha']
    results=[review(p,d) for p in [first,second]]
    if results[0]['archive_sha256']!=results[1]['archive_sha256']:raise ValueError('same-input ZIP digest mismatch')
    if any(sha(p)!=value for p,value in before.items()):raise ValueError('reviewed input changed')
    print(json.dumps({'status':'DEVELOPMENT_ARCHIVE_REVIEW_PASS','release_ready':False,'identical_archives':True,'reviews':results,'scope':'same-input development archive; no native installer/independent rebuild/license complete proof'}))
if __name__=='__main__':main()
