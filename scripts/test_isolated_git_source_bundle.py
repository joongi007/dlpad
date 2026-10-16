"""Pure guards only. No subprocess or Git command is executed."""
import hashlib,json,os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from types import SimpleNamespace
import isolated_git_source_bundle as m

class Guards(unittest.TestCase):
 def fixture(self,root):
  source=root/'original';out=root/'export';source.mkdir();out.mkdir()
  original=b'original';(source/'README.md').write_bytes(original);(out/'README.md').write_bytes(original)
  receipt={'schema':2,'release_ready':False,'files':[{'path':'README.md','bytes':len(original),'sha256':hashlib.sha256(original).hexdigest(),'original_sha256':hashlib.sha256(original).hexdigest()}]}
  (out/'SOURCE_EXPORT.json').write_text(json.dumps(receipt));return source,out,receipt
 def test_environment_drops_repo_auth_hooks_and_global_redirects(self):
  with patch.dict(os.environ,{'GIT_DIR':'bad','GIT_WORK_TREE':'bad','GIT_CONFIG_PARAMETERS':'bad','SSH_AUTH_SOCK':'secret','AWS_SECRET_ACCESS_KEY':'secret','HOME':'secret'}):
   env=m.clean_env(Path('/tmp/owned'))
  for key in ['GIT_DIR','GIT_WORK_TREE','GIT_CONFIG_PARAMETERS','SSH_AUTH_SOCK','AWS_SECRET_ACCESS_KEY','HOME']:self.assertNotIn(key,env)
  self.assertEqual(env['GIT_CONFIG_GLOBAL'],'/dev/null');self.assertEqual(env['GIT_ALLOW_PROTOCOL'],'')
 def test_paths_reject_escape_git_symlink_and_destination_reuse(self):
  for name in ['../x','/x','a/../x','.git/config','a\\x','a//x','a/./x']:
   with self.assertRaises(ValueError):m.relative(name)
  with tempfile.TemporaryDirectory()as tmp:
   root=Path(tmp);source=root/'source';source.mkdir();alias=root/'alias';alias.symlink_to(source)
   with self.assertRaises(ValueError):m.plain(alias)
   with self.assertRaises(ValueError):m.fresh_root(source,source)
   with self.assertRaises(ValueError):m.fresh_root(source/'out',source)
 def test_export_original_final_and_exact_namespace(self):
  with tempfile.TemporaryDirectory()as tmp:
   source,out,_=self.fixture(Path(tmp));expected=m.verify_export(out,source);self.assertEqual(set(expected),{'README.md','SOURCE_EXPORT.json'})
   (out/'unexpected').write_text('x')
   with self.assertRaises(ValueError):m.verify_export(out,source)
   (out/'unexpected').unlink();(source/'README.md').write_bytes(b'changed')
   with self.assertRaises(ValueError):m.verify_export(out,source)
 def test_export_bytes_and_duplicate_receipt_fail(self):
  with tempfile.TemporaryDirectory()as tmp:
   source,out,r=self.fixture(Path(tmp));(out/'README.md').write_bytes(b'changed')
   with self.assertRaises(ValueError):m.verify_export(out,source)
   (out/'README.md').write_bytes(b'original');r['files']*=2;(out/'SOURCE_EXPORT.json').write_text(json.dumps(r))
   with self.assertRaises(ValueError):m.verify_export(out,source)
  with self.assertRaises(ValueError):m.exact_json(b'{"x":1,"x":2}')
 def test_credential_filename_rejected_before_commit(self):
  with tempfile.TemporaryDirectory()as tmp:
   source,out,r=self.fixture(Path(tmp));r['files'][0]['path']='.env';(out/'SOURCE_EXPORT.json').write_text(json.dumps(r))
   with self.assertRaisesRegex(ValueError,'credential'):m.verify_export(out,source)
 def test_tree_exact_regular_sha1_bytes(self):
  with tempfile.TemporaryDirectory()as tmp:
   source,out,_=self.fixture(Path(tmp));expected=m.verify_export(out,source)
   raw=b''.join(b'100644 blob '+proof['blob_sha1'].encode()+b'\t'+name.encode()+b'\0'for name,proof in expected.items())
   m.verify_tree(raw,expected)
   for wrong in [raw.replace(b'100644',b'120000',1),raw.replace(b'blob',b'commit',1),raw+raw.split(b'\0')[0]+b'\0',raw.replace(next(iter(expected.values()))['blob_sha1'].encode(),b'0'*40,1)]:
    with self.assertRaises(ValueError):m.verify_tree(wrong,expected)
 def test_actual_committed_sha256_guard_with_mocked_process_only(self):
  with tempfile.TemporaryDirectory()as tmp:
   root=Path(tmp);source,out,_=self.fixture(root);proof={'README.md':m.hashes(out/'README.md')}
   def correct(*args,**kwargs):
    kwargs['stdout'].write(b'original');return SimpleNamespace(returncode=0)
   def wrong(*args,**kwargs):
    kwargs['stdout'].write(b'ORIGINAL');return SimpleNamespace(returncode=0)
   with patch.object(m.subprocess,'run',side_effect=correct):m.verify_committed_sha256(Path('/usr/bin/git'),out,root,proof)
   with patch.object(m.subprocess,'run',side_effect=wrong):
    with self.assertRaisesRegex(ValueError,'SHA256'):m.verify_committed_sha256(Path('/usr/bin/git'),out,root,proof)
if __name__=='__main__':unittest.main()
