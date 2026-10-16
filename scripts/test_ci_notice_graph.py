import json
import os
import unittest
from ci_notice_graph import classify, publish
from pathlib import Path
import tempfile
import copy
def symlink_or_skip(test,path,target,directory=False):
    try:path.symlink_to(target,target_is_directory=directory)
    except OSError as error:
        if os.name=='nt' and getattr(error,'winerror',None)==1314:
            test.skipTest('Windows symlink creation privilege unavailable; no system changes')
        raise

class Tests(unittest.TestCase):
    def setUp(self):
        self.row=b'owned v0.1.0 (C:\\fixture)\n'
        self.meta=dict(version=1,packages=[dict(name='owned',version='0.1.0',id='path+file:///fixture#owned@0.1.0',source=None,manifest_path='C:/fixture/Cargo.toml')])
        self.lock=b'version = 4\n[[package]]\nname="owned"\nversion="0.1.0"\n'
    def run_case(self):return classify(self.row,self.row,json.dumps(self.meta).encode(),self.lock)
    def test_format_binding_is_never_remote_success_or_readiness(self):
        result=self.run_case();self.assertEqual(result['union_count'],1);self.assertFalse(result['remote_execution_verified']);self.assertFalse(result['release_ready'])
    def test_real_cargo_proc_macro_annotation_is_supported(self):
        self.row=b'owned v0.1.0 (proc-macro)\nowned v0.1.0 (proc-macro) (*)\n'
        self.assertEqual(self.run_case()['union_count'],1)
    def test_cargo_path_id_matching_directory_uses_short_version(self):
        self.meta['packages'][0]['id']='path+file:///fixture/owned#0.1.0'
        self.assertEqual(self.run_case()['union_count'],1)
        self.meta['packages'][0]['id']='path+file:///fixture/other#0.1.0'
        with self.assertRaises(ValueError):self.run_case()
    def test_wrong_source_rejected(self):
        self.meta['packages'][0]['source']='registry+wrong'
        with self.assertRaises(ValueError):self.run_case()
    def test_ambiguous_sources_rejected(self):
        self.meta['packages'].append(dict(self.meta['packages'][0]))
        with self.assertRaises(ValueError):self.run_case()
    def test_unknown_tree_line_rejected(self):
        self.row=b'Finished release\n'
        with self.assertRaises(ValueError):self.run_case()
    def test_duplicate_json_and_invalid_metadata_domains(self):
        raws=[b'{"version":1,"version":1}',b'[]',b'{"version":true}',b'{"version":1,"packages":[null]}',b'{"version":1,"packages":[{}]}',b'{"version":1,"packages":"wrong"}']
        for raw in raws:
            with self.subTest(raw=raw),self.assertRaises(ValueError):
                classify(self.row,self.row,raw,self.lock)
    def test_invalid_identity_id_and_manifest(self):
        for key,value in [('version','bad'),('name',[]),('id','path+file:///fixture#wrong@0.1.0'),('manifest_path','relative/Cargo.toml'),('source',False),('license',[]),('license_file','x\x00y')]:
            saved=copy.deepcopy(self.meta)
            self.meta['packages'][0][key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):self.run_case()
            self.meta=saved
    def test_malformed_checksum_and_missing_registry_checksum(self):
        src='registry+https://github.com/rust-lang/crates.io-index'
        self.meta['packages'][0].update(source=src,id=src+'#owned@0.1.0')
        for checksum in [None,'A'*64,'z'*64,'a'*63]:
            self.lock=('version=4\n[[package]]\nname="owned"\nversion="0.1.0"\nsource="'+src+'"\n'+('' if checksum is None else 'checksum="'+checksum+'"\n')).encode()
            with self.subTest(checksum=checksum),self.assertRaises(ValueError):self.run_case()
    def test_lock_authority_fields_not_silently_omitted(self):
        for extra in [b'source=1\n',b'checksum=true\n',b'name="owned"\n',b'checksum="a"\nchecksum="b"\n']:
            self.lock=b'version=4\n[[package]]\nname="owned"\nversion="0.1.0"\n'+extra
            with self.subTest(extra=extra),self.assertRaises(ValueError):self.run_case()
    def test_tree_shape_and_encoding_rejection(self):
        for row in [b'owned v0.1.0 (a) (b)\n',b'owned vbad\n',b'owned v0.1.0 (())\n',b'owned v0.1.0 ()\n',b'\xff']:
            self.row=row
            with self.subTest(row=row),self.assertRaises(ValueError):self.run_case()
    def test_output_ancestor_symlink_and_nooverwrite(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);target=root/'target';target.mkdir()
            output=root/'receipt.json';publish(output,{'release_ready':False})
            before=output.read_bytes()
            with self.assertRaises(ValueError):publish(output,{})
            self.assertEqual(output.read_bytes(),before)
            alias=root/'alias';symlink_or_skip(self,alias,target,True)
            with self.assertRaises(ValueError):publish(alias/'receipt.json',{})
            self.assertFalse((target/'receipt.json').exists())
if __name__=='__main__':unittest.main()
