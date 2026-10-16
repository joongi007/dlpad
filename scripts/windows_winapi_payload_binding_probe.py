#!/usr/bin/env python3
"""Original cache is read-only; all grant tampering is confined to owned tempfile."""
import sys,json,copy,tempfile,shutil,hashlib
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
import windows_candidate_package as c
import windows_winapi_payload_binding as b
root=c.ROOT
import argparse
parser=argparse.ArgumentParser(description='Read-only original Winapi payload and owned grant-tamper regression')
parser.add_argument('output',type=Path)
args=parser.parse_args()
out=c.plain(args.output)
if not (out.is_relative_to(root) or out.is_relative_to(Path('/tmp'))):raise ValueError('owned project or temporary output required')

v=c.upstream_manifest();key=b.KEY;source=next((root/'.local-tools/cargo/registry/src').glob('*/'+key));pkg=c.toml_inventory((source/'Cargo.toml').read_text())['package'];lock=c.toml_inventory((root/'Cargo.lock').read_text(),lock=True);record=next(x for x in lock['package'] if x['name']+'-'+x['version']==key);checksum=record['checksum'];assert checksum==b.CRATE_SHA
selected=c.mapped_notices(v,key,source,pkg,checksum);assert len(selected)==2
rejected=[]
for label,change in [('checksum',lambda m:m.update(crate_sha256='0'*64)),('count',lambda m:m.update(archive_count=1415)),('tree',lambda m:m.update(target_tree_sha256='0'*64))]:
 bad=copy.deepcopy(v);change(bad['packages'][key]['payload_binding'])
 try:c.mapped_notices(bad,key,source,pkg,checksum)
 except ValueError:rejected.append(label)
 else:raise AssertionError('bad payload accepted')
try:c.mapped_notices(v,key,source,pkg,'0'*64)
except ValueError:rejected.append('original_lock_checksum')
else:raise AssertionError('bad lock accepted')
with tempfile.TemporaryDirectory(prefix='dlpad-winapi-grant-owned-') as td:
 fixture=Path(td);(fixture/'docs/licenses/upstream').mkdir(parents=True)
 for x in v['sources'].values():
  if x['repository']=='retep998/winapi-rs':shutil.copyfile(root/x['file'],fixture/x['file'])
 for name in ['winapi-gnu-bump-root-tree.json','winapi-gnu-bump-x86_64-tree.json']:shutil.copyfile(root/'docs/licenses/upstream'/name,fixture/'docs/licenses/upstream'/name)
 bad=copy.deepcopy(v);identifier=bad['packages'][key]['sources'][0];notice=fixture/bad['sources'][identifier]['file'];data=notice.read_bytes().replace(b'Permission',b'PermissioX',1);notice.write_bytes(data);bad['sources'][identifier]['sha256']=hashlib.sha256(data).hexdigest()
 c.ROOT=fixture
 try:c.mapped_notices(bad,key,source,pkg,checksum)
 except ValueError as e:
  assert 'Git blob' in str(e);rejected.append('notice_and_claimed_digest_tamper')
 else:raise AssertionError('bad grant accepted')
 finally:c.ROOT=root
out.mkdir(exist_ok=False); (out/'result.json').write_text(json.dumps({'passed':True,'payload_members':1420,'import_archives':1416,'notices':len(selected),'negative_rejected':rejected,'historical_upload_commit_verified':False,'coverage_complete':False},indent=2)+'\n');print('PASS exact original payload + grant binding; negatives',rejected)
