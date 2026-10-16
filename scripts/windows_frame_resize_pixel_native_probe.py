"""Collect bounded Windows private resize evidence; root reviews actual pixels separately."""
import argparse,hashlib,json,os,pathlib,re,sqlite3,stat,sys,time
ROOT=pathlib.Path(__file__).resolve().parents[1]

def require(ok,msg):
    if not ok: raise ValueError(msg)
def plain(p):
    require(p.is_absolute() and '..' not in p.parts and not any(x.is_symlink() for x in [p,*p.parents]),'unsafe path');return p
def read(p,limit):
    plain(p)
    with p.open('rb') as f:
        info=os.fstat(f.fileno());require(stat.S_ISREG(info.st_mode) and info.st_size<=limit,'regular input bound')
        data=f.read(limit+1);require(len(data)<=limit,'growing input bound');return data
def sha(p,limit=32*1024*1024):return hashlib.sha256(read(p,limit)).hexdigest()
def write(p,v):
    data=(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode();require(len(data)<=1024*1024,'JSON bound')
    with p.open('xb') as f:f.write(data)
def normalize_native_path(value):
    require(isinstance(value,str) and 0<len(value)<=32768,'DB path bound')
    if value.startswith('\\\\?\\'):
        value=value[4:]
    normalized=value.replace('\\','/').rstrip('/').casefold()
    require('..' not in normalized.split('/'),'DB path traversal')
    return normalized

def validate_rows(rows,expected_root):
    root=normalize_native_path(expected_root)
    require(len(rows)==10000,'DB row count')
    paths=set()
    for row in rows:
        require(len(row)==5,'DB row shape')
        require(normalize_native_path(row[0])==root,'DB scoped root mismatch')
        path=normalize_native_path(row[1])
        require(path.startswith(root+'/') and path not in paths,'DB path root/duplicate')
        paths.add(path)
        require(type(row[2]) is int and row[2]==3 and type(row[3]) is int and row[3]==1,'DB size/present')
        require(isinstance(row[4],str) and re.fullmatch(r'windows:v1:[0-9]+:[0-9a-f]{32}',row[4]) is not None,'DB identity shape')
    require(paths=={root+f'/media-{i:05d}.mp4' for i in range(10000)},'DB full inventory mismatch')

def main():
    a=argparse.ArgumentParser();a.add_argument('--output',type=pathlib.Path,required=True);a.add_argument('--proof',type=pathlib.Path,required=True);a.add_argument('--sha',required=True);args=a.parse_args()
    plain(ROOT);out=plain(args.output);require(out.is_relative_to(ROOT/'docs/validation/evidence') and not out.exists(),'fresh evidence required');require(re.fullmatch('[0-9a-f]{64}',args.sha),'SHA syntax')
    sys.path.insert(0,str(ROOT/'scripts'));import windows_current_workspace_probe as n
    from windows_library_periodic_functional_native_probe import synthetic_payload_intact
    before=n.snapshot();proof=json.loads(read(args.proof,1024*1024));require(proof.get('passed') is True,'full proof rejected')
    require(proof.get('source_baseline')==before and proof.get('source_final')==before,'proof exact current snapshot required (adapt only to observed original schema)')
    require(proof.get('cpu_artifact',{}).get('sha256')==args.sha and proof.get('source_unchanged') is True and proof.get('cargo_checks_complete') is True and len(proof.get('actual_operations',{}))==4 and all(o.get('passed') is True for o in proof['actual_operations'].values()),'four operations proof');
    binary=plain(n.TOOLS/'target/release/dlpad-cpu.exe');require(sha(binary)==args.sha,'default CPU mismatch');source=plain(ROOT/'scripts/windows_frame_resize_pixel_probe.cs')
    original={str(p):sha(p,1024*1024) for p in [source,pathlib.Path(__file__),ROOT/'scripts/windows_current_workspace_probe.py',ROOT/'scripts/windows_library_periodic_functional_native_probe.py',args.proof]}
    out.mkdir();inputs=out/'inputs';inputs.mkdir();operations=[];result={'status':'FAIL','pixel_correctness_verified':False,'operations':operations}
    try:
        for src in [binary,source]:
            with (inputs/src.name).open('xb') as f:f.write(read(src,32*1024*1024))
        pins=out/'source-pins.json';write(pins,{'production_source':before,'explicit_inputs':original,'cpu_sha256':args.sha});pinsha=sha(pins,1024*1024)
        _,env=n.configure();framework=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319';host=str(n.TOOLS/'windows-cli-host.exe');observer=out/'observer.exe'
        def run(name,argv,seconds):
            op=n.run(name,[host,str(seconds),*argv],env,out,seconds);operations.append(op);require(op['passed'],name+' rejected');return op
        refs=['System.Drawing','System.Web.Extensions'];argv=[framework+'\\csc.exe','/nologo',*['/reference:'+x+'.dll' for x in refs],*['/reference:'+framework+'\\WPF\\'+x+'.dll' for x in ['UIAutomationClient','UIAutomationTypes','WindowsBase']],'/out:'+n.win(observer),n.win(inputs/source.name)]
        run('compile',argv,45);observer_sha=sha(observer);run('gui',[n.win(observer),n.win(inputs/binary.name),args.sha,n.win(out/'gui'),n.win(pins),pinsha],120)
        raw=json.loads(read(out/'gui/native-result.json',262144));result['native']=raw
        for k in ['collection_passed','original_client_restored_exact','resize_geometry_verified','wm_close_exited','owned_job_naturally_empty','owned_job_empty_after_cleanup','owned_process_exited_after_job_close','private_desktop_closed','input_desktop_unchanged','sentinel_preserved']:require(raw.get(k) is True,k+' missing')
        require(raw.get('status')=='REVIEW_REQUIRED' and type(raw.get('exit_code')) is int and raw.get('exit_code')==0 and raw.get('pixel_correctness_verified') is False,'collection status')
        require(raw.get('binary_sha256_before')==args.sha==raw.get('binary_sha256_after') and raw.get('source_pins_sha256')==pinsha,'native pins')
        # No bulk review until original native Job is confirmed empty; never concurrent with measurement.
        import ntpath
        native_root=raw['owned_fixture_root'];require('..' not in native_root.replace('\\','/').split('/'),'native traversal');require(ntpath.dirname(native_root).casefold()==env['TEMP'].casefold(),'TEMP mismatch')
        root=plain(n.TOOLS/'tmp'/ntpath.basename(native_root));require(re.fullmatch('dlpad-library-owned-[0-9a-f]{32}',root.name),'root namespace');media=plain(root/'media');names=set();started=time.monotonic()
        with os.scandir(media) as entries:
            for e in entries:
                require(time.monotonic()-started<120 and len(names)<10000,'review count/deadline');require(re.fullmatch('media-[0-9]{5}\\.mp4',e.name) and not e.is_symlink() and synthetic_payload_intact(pathlib.Path(e.path)),'payload changed');names.add(e.name)
        require(names=={f'media-{i:05d}.mp4' for i in range(10000)},'inventory mismatch');require(read(root/'foreign-sentinel',128)==b'dlpad-library-sentinel-v1','sentinel mismatch')
        db=plain(root/'data/library.sqlite');dbsha=sha(db,16*1024*1024)
        with sqlite3.connect(db.resolve().as_uri()+'?mode=ro',uri=True) as c:
            c.execute('pragma query_only=ON');require(c.execute('pragma integrity_check').fetchall()==[('ok',)],'DB integrity');rows=c.execute('select root,path,size,present,file_identity from library_entries order by path limit 10001').fetchall()
        validate_rows(rows,n.win(media))
        require(sha(db,16*1024*1024)==dbsha,'DB changed during review')
        require(before==n.snapshot() and sha(binary)==args.sha and sha(observer)==observer_sha and all(sha(pathlib.Path(p),1024*1024)==v for p,v in original.items()) and sha(pins,1024*1024)==pinsha,'final pins changed');require(sha(inputs/binary.name)==args.sha and sha(inputs/source.name,1024*1024)==original[str(source)],'copy pins changed')
        result.update(status='REVIEW_REQUIRED',collection_passed=True,independent_rows=10000,db_sha256=dbsha,source_unchanged=True,pixel_review_required=True)
    except Exception as e:result['error']=type(e).__name__+': '+str(e)
    write(out/'result.json',result);return 0 if result['status']=='REVIEW_REQUIRED' else 2
if __name__=='__main__':sys.exit(main())
