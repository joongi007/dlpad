import tempfile
from pathlib import Path
from unittest.mock import patch
import windows_msvc_local_export_collect as collector
import unittest
from windows_msvc_local_export_collect import validate_native_receipt


def receipt():
    native = dict.fromkeys(['wm_close_exited', 'original_handles_close_safe',
        'owned_job_naturally_empty', 'owned_job_empty_after_cleanup',
        'private_desktop_closed', 'input_desktop_unchanged', 'mta_desktop_restored',
        'actual_range_field_readback_verified', 'final_success_label_verified',
        'same_file_stage_final_verified'], True)
    native.update(natural_exit_code=0, owned_job_active_before_cleanup=0,
        owned_job_terminated=False, requested_start_ms=1100, requested_end_ms=1800)
    return dict(status='REVIEW_REQUIRED', scenario='success', native=native,
                global_input=False, clipboard_invoked=False)


class NativeReceiptFailures(unittest.TestCase):
    def test_observation_remains_review_required(self):
        value = receipt()
        self.assertIs(validate_native_receipt(value, 'success'), value['native'])
        self.assertEqual(value['status'], 'REVIEW_REQUIRED')

    def test_forced_cleanup_cannot_count_as_natural_completion(self):
        for key, value in [('owned_job_terminated', True),
                           ('owned_job_active_before_cleanup', 1), ('natural_exit_code', 1)]:
            candidate = receipt(); candidate['native'][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate_native_receipt(candidate, 'success')

    def test_no_global_fallback_and_no_wrong_range(self):
        for key in ['global_input', 'clipboard_invoked']:
            candidate = receipt(); candidate[key] = True
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate_native_receipt(candidate, 'success')
        candidate = receipt(); candidate['native']['requested_start_ms'] = 0
        with self.assertRaises(ValueError): validate_native_receipt(candidate, 'success')

    def test_cancel_or_close_must_observe_actual_running_action(self):
        for scenario, flag in [('cancel', 'cancel_invoked_while_actual_running'),
                              ('close', 'close_prompt_visible_and_return_verified')]:
            candidate = receipt(); candidate['scenario'] = scenario
            with self.subTest(scenario=scenario), self.assertRaises(ValueError):
                validate_native_receipt(candidate, scenario)
            candidate['native'][flag] = True
            validate_native_receipt(candidate, scenario)

    def test_success_label_without_native_link_fact_rejected(self):
        candidate = receipt(); candidate['native']['same_file_stage_final_verified'] = False
        with self.assertRaises(ValueError): validate_native_receipt(candidate, 'success')


class CloseStopFailures(unittest.TestCase):
    def valid(self):
        candidate = receipt(); candidate['scenario'] = 'close-stop'
        candidate['native'].update(dict.fromkeys(['export_running_observed',
            'close_stop_prompt_title_verified', 'stop_exit_invoked_while_actual_running',
            'close_stop_exit_observed', 'closewait_label_observed', 'closewait_inflight_verified',
            'worker_ack_inferred_from_production_natural_exit', 'publication_review_pending_after_close'], True))
        candidate['native']['direct_worker_ack_observed'] = False
        return candidate

    def test_natural_exit_without_waiting_observation_rejected(self):
        candidate = self.valid(); candidate['native']['closewait_label_observed'] = False
        with self.assertRaises(ValueError): validate_native_receipt(candidate, 'close-stop')

    def test_each_actual_action_fact_required(self):
        candidate = self.valid()
        validate_native_receipt(candidate, 'close-stop')
        for key in ['export_running_observed', 'close_stop_prompt_title_verified',
                    'stop_exit_invoked_while_actual_running', 'close_stop_exit_observed']:
            failed = self.valid(); failed['native'][key] = False
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate_native_receipt(failed, 'close-stop')

    def test_direct_ack_claim_or_unknown_scenario_rejected(self):
        candidate = self.valid(); candidate['native']['direct_worker_ack_observed'] = True
        with self.assertRaises(ValueError): validate_native_receipt(candidate, 'close-stop')
        candidate = self.valid(); candidate['scenario'] = 'invented'
        with self.assertRaises(ValueError): validate_native_receipt(candidate, 'invented')


class StreamPins(unittest.TestCase):
    def test_exact_copy_and_existing_destination_rejected(self):
        with tempfile.TemporaryDirectory() as root:
            source, destination = Path(root) / 'source', Path(root) / 'destination'
            source.write_bytes(b'original bytes')
            collector.stream_copy(source, destination)
            self.assertEqual(collector.stream_sha(source), collector.stream_sha(destination))
            with self.assertRaises(FileExistsError): collector.stream_copy(source, destination)
            self.assertEqual(destination.read_bytes(), b'original bytes')

    def test_bound_and_symlink_reject_before_destination_creation(self):
        with tempfile.TemporaryDirectory() as root:
            source, destination, link = [Path(root) / name for name in ['source', 'destination', 'link']]
            source.write_bytes(b'12345')
            with patch.object(collector, 'STREAM_LIMIT', 4):
                with self.assertRaises(ValueError): collector.stream_copy(source, destination)
            self.assertFalse(destination.exists())
            link.symlink_to(source)
            with self.assertRaises(ValueError): collector.stream_sha(link)


if __name__ == '__main__': unittest.main()
