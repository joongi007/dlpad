#!/usr/bin/env python3
"""Pure CLI failure paths; never launches a native process."""
import contextlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import windows_library_periodic_functional_native_probe as probe


class Arguments(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='dlpad-periodic-cli-')
        self.root = Path(self.temp.name)
        self.evidence = self.root / 'docs/validation/evidence'
        self.evidence.mkdir(parents=True)
        self.scope = patch.object(probe.n, 'ROOT', self.root)
        self.scope.start()

    def tearDown(self):
        self.scope.stop()
        self.temp.cleanup()

    def args(self, output=None, case='stop'):
        return ['--project', str(self.root), '--output', str(output or self.evidence / 'fresh'), '--case', case]

    def rejected(self, argv):
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as error:
            probe.arguments(argv)
        self.assertEqual(error.exception.code, 2)
        self.assertFalse((self.evidence / 'fresh').exists())

    def test_valid_arguments_have_no_execution_or_creation(self):
        with patch.object(probe.n, 'configure', side_effect=AssertionError('native setup forbidden')):
            args = probe.arguments(self.args())
        self.assertEqual(args.case, 'stop')
        self.assertFalse(args.output.exists())

    def test_thread_cpu_observer_is_explicit_and_has_no_execution(self):
        self.assertFalse(probe.arguments(self.args()).thread_cpu_observer)
        with patch.object(probe.n, 'configure', side_effect=AssertionError('native setup forbidden')):
            args=probe.arguments(self.args()+['--thread-cpu-observer'])
        self.assertTrue(args.thread_cpu_observer)
        self.assertFalse(args.output.exists())

    def test_unknown_case_and_missing_required_option_rejected(self):
        self.rejected(self.args(case='desktop'))
        self.rejected(['--project', str(self.root), '--case', 'next'])

    def test_existing_and_outside_evidence_rejected(self):
        self.rejected(self.args(self.evidence))
        self.rejected(self.args(self.root / 'outside'))

    def test_symlink_parent_and_traversal_rejected(self):
        link = self.evidence / 'link'
        link.symlink_to(self.root, target_is_directory=True)
        self.rejected(self.args(link / 'fresh'))
        self.rejected(self.args(self.evidence / '..' / 'fresh'))


class NativeProof(unittest.TestCase):
    def record(self, scenario):
        return dict(scenario=scenario, natural_exitcode=0, exact_inherited_event_caps=4,
                    periodic_commit_ms=120000, next_response_ms=200,
                    stop_pre_release_ack_false=True, stop_library_controls_disabled=True)
    def test_all_original_case_shapes(self):
        for scenario in ['completion','next','stop']:
            probe.validate_native_case(self.record(scenario), scenario)
    def test_wrong_scenario_is_not_review_required_success(self):
        with self.assertRaisesRegex(ValueError, 'scenario'):
            probe.validate_native_case(self.record('stop'), 'completion')
    def test_missing_case_specific_observation_fails(self):
        for scenario, key in [('completion','periodic_commit_ms'),('next','next_response_ms'),
                              ('stop','stop_pre_release_ack_false'),('stop','stop_library_controls_disabled')]:
            record=self.record(scenario);del record[key]
            with self.subTest(scenario=scenario,key=key),self.assertRaises(ValueError):
                probe.validate_native_case(record, scenario)
    def test_duration_bounds_and_json_bool_are_rejected(self):
        for scenario,key in [('completion','periodic_commit_ms'),('next','next_response_ms')]:
            for value in [-1,True,'2',180001,float('nan')]:
                record=self.record(scenario);record[key]=value
                with self.subTest(scenario=scenario,value=value),self.assertRaises(ValueError):
                    probe.validate_native_case(record, scenario)
    def test_input_binding_mismatch_rejected_before_database_io(self):
        with tempfile.TemporaryDirectory(prefix='dlpad-periodic-binding-') as temporary:
            root = Path(temporary)
            (root / 'result.json').write_text(json.dumps(self.record('completion')))
            with patch.object(probe, 'database_rows', side_effect=AssertionError('DB read forbidden')):
                with self.assertRaisesRegex(ValueError, 'binding'):
                    probe.review_fixture(root, root, {}, 'completion', {'binary_sha256': 'expected'})

    def test_wrong_capability_or_bool_exit_rejected(self):
        for key,value in [('exact_inherited_event_caps',3),('natural_exitcode',False)]:
            record=self.record('completion');record[key]=value
            with self.assertRaises(ValueError):probe.validate_native_case(record,'completion')


class PayloadReader(unittest.TestCase):
    def test_regular_bytes_and_failure_paths_preserve_foreign_target(self):
        with tempfile.TemporaryDirectory(prefix='dlpad-periodic-payload-') as temporary:
            root = Path(temporary)
            good = root / 'good'
            good.write_bytes(b'\x01\x02\x03')
            self.assertTrue(probe.synthetic_payload_intact(good))
            wrong = root / 'wrong'
            wrong.write_bytes(b'\x01\x02\x03\x04')
            self.assertFalse(probe.synthetic_payload_intact(wrong))
            self.assertFalse(probe.synthetic_payload_intact(root))
            self.assertFalse(probe.synthetic_payload_intact(root / 'missing'))
            link = root / 'link'
            link.symlink_to(good)
            self.assertFalse(probe.synthetic_payload_intact(link))
            self.assertEqual(good.read_bytes(), b'\x01\x02\x03')


if __name__ == '__main__':
    unittest.main()
