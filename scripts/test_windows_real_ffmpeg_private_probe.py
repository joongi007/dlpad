import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
spec=importlib.util.spec_from_file_location('probe',str(Path(__file__).with_name('windows_real_ffmpeg_private_probe.py')))
p=importlib.util.module_from_spec(spec);spec.loader.exec_module(p)
def valid():
 return {'schema':1,**{k:{'path':'D:\\owned\\'+k,'bytes':1,'sha256':p.PINS.get(k,'a'*64)} for k in ['yt_dlp','ffmpeg','ffprobe','license']}}
class Tests(unittest.TestCase):
 def test_exact_shape_valid_and_fixed_pin(self):
  self.assertEqual(p.manifest_shape(valid()),valid())
  v=valid();v['ffmpeg']['sha256']='a'*64
  with self.assertRaises(ValueError):p.manifest_shape(v)
 def test_unknown_schema_and_tool_fields_rejected(self):
  for v in [dict(valid(),unknown=1),dict(valid(),schema=True),dict(valid(),schema=2)]:
   with self.assertRaises(ValueError):p.manifest_shape(v)
  v=valid();v['yt_dlp']['extra']=1
  with self.assertRaises(ValueError):p.manifest_shape(v)
 def test_path_authority_and_digest_bounds(self):
  for raw in ['relative.exe','D:\\owned\\..\\foreign.exe','D:\\owned\\x\0','\\\\host\\share\\x','D:\\owned\\"x','D:/owned/x']:
   v=valid();v['yt_dlp']['path']=raw
   with self.assertRaises(ValueError):p.manifest_shape(v)
  for value in [0,p.MAX_TOOL+1,True]:
   v=valid();v['yt_dlp']['bytes']=value
   with self.assertRaises(ValueError):p.manifest_shape(v)
 def test_duplicate_json_keys_rejected(self):
  with self.assertRaises(ValueError):json.loads('{"schema":1,"schema":1}',object_pairs_hook=p.duplicate_rejected)
 def test_actual_oversize_manifest_and_symlink_preserve_source(self):
  with tempfile.TemporaryDirectory() as folder:
   root=Path(folder);source=root/'sentinel';source.write_bytes(b'owned sentinel');link=root/'link';link.symlink_to(source)
   with self.assertRaises(ValueError):p.regular(link,65536)
   huge=root/'huge';huge.write_bytes(b' '*65537)
   with self.assertRaises(ValueError):p.load_manifest(huge)
   self.assertEqual(source.read_bytes(),b'owned sentinel')
 def test_actual_tool_hash_mismatch_fails_without_mutation(self):
  with tempfile.TemporaryDirectory() as folder:
   root=Path(folder);tool=root/'tool';tool.write_bytes(b'x');manifest=root/'manifest.json';manifest.write_text(json.dumps(valid()))
   with patch.object(p,'native_file',return_value=tool):
    with self.assertRaises(ValueError):p.load_manifest(manifest)
   self.assertEqual(tool.read_bytes(),b'x');self.assertEqual(json.loads(manifest.read_text()),valid())
 def test_native_prefix_mapping_is_deterministic(self):
  self.assertEqual(p.native_file('\\\\?\\D:\\owned\\한글.exe'),Path('/mnt/d/owned/한글.exe'))
 def test_project_output_manifest_and_native_traversal_rejected(self):
  for raw in ['/tmp/evidence/../../outside','/tmp/project/../other','/tmp/manifest/../m.json']:
   with self.assertRaises(ValueError):p.checked_parts(Path(raw))
  with self.assertRaises(ValueError):p.native_file('D:\\owned\\..\\foreign.exe')
 def test_file_growth_after_stat_is_still_hash_read_bounded(self):
  with tempfile.TemporaryDirectory() as folder:
   file=Path(folder)/'owned-growing';file.write_bytes(b'x')
   original_regular=p.regular
   def checked_then_grow(path,limit):
    result=original_regular(path,limit)
    with file.open('ab') as owned:owned.write(b'z'*32)
    return result
   with patch.object(p,'regular',side_effect=checked_then_grow):
    with self.assertRaisesRegex(ValueError,'actual hashing read bound'):p.sha(file,8)
   self.assertEqual(file.stat().st_size,33)
if __name__=='__main__':unittest.main()
