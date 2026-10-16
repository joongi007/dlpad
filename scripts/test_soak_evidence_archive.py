"""Synthetic safety/terminal regressions; no soak or subprocess launch."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
import uuid
from unittest import mock
import soak_evidence_archive as archive


class ArchiveTests(unittest.TestCase):
    def setUp(self):
        self.download = Path(tempfile.mkdtemp(prefix="dlpad-archive-test-", dir="/tmp"))
        self.runner = self.download.with_name(self.download.name + "-runner")
        self.runner.mkdir()
        self.guardian = Path(tempfile.mkdtemp(prefix="dlpad-archive-guardian-test-", dir="/tmp"))
        self.roots = {"download": self.download, "runner": self.runner, "guardian": self.guardian}
        self.destination = Path(archive.__file__).absolute().parent.parent / "docs/validation/evidence" / ("archive-synthetic-" + uuid.uuid4().hex)
        boot = Path("/proc/sys/kernel/random/boot_id").read_text().strip()
        ticks = int(Path(f"/proc/{os.getpid()}/stat").read_text().rsplit(")", 1)[1].split()[19])
        self.manifest = {"boot_id": boot, "roots": {name: str(path) for name, path in self.roots.items()},
                         "processes": {"download_owner": {"pid": os.getpid(), "start_ticks": ticks}}}
        self.collector = None

    def tearDown(self):
        if self.collector is not None:
            self.collector.close()
        for path in [self.download, self.runner, self.guardian, self.destination]:
            if path.is_symlink():
                path.unlink()
            elif path.exists():
                shutil.rmtree(path)

    def start(self):
        self.collector = archive.Archive(self.manifest, self.roots, self.destination)
        return self.collector

    def write_results(self, state="completed"):
        values = {"download": {"state": state, "elapsed_seconds": 28801, "eight_hours_verified": True},
                  "runner": {"state": state, "exit_code": 0},
                  "guardian": {"state": state, "elapsed_seconds": 28801, "eight_hours_elapsed": True, "descendants_reaped": True}}
        for name, value in values.items():
            (self.roots[name] / "result.json").write_text(json.dumps(value))

    def test_live_hash_allowlist_and_no_terminal(self):
        data = b"synthetic-only\n" * 5000
        (self.runner / "test.log").write_bytes(data)
        (self.runner / "not-allowed.secret").write_text("synthetic private sentinel")
        value = self.start().step()
        self.assertEqual(value["state"], "watching")
        self.assertEqual(value["files"]["download-test.log"]["sha256"], hashlib.sha256(data).hexdigest())
        self.assertFalse((self.destination / "not-allowed.secret").exists())
        self.assertFalse((self.destination / "archive-result.json").exists())
        self.assertEqual(value["process_actions"], [])

    def test_completed_requires_full_terminal_proof(self):
        self.write_results()
        value = self.start().step()
        self.assertEqual(value["state"], "completed")
        self.assertEqual(json.loads((self.destination / "archive-result.json").read_text())["state"], "completed")

    def test_boot_change_never_copies_new_boot_roots(self):
        self.write_results()
        self.manifest["boot_id"] = "00000000-0000-0000-0000-000000000000"
        value = self.start().step()
        self.assertEqual(value["state"], "interrupted")
        self.assertFalse(value["processes"]["download_owner"]["matches"])
        self.assertFalse((self.destination / "download-result.json").exists())

    def test_reused_pid_is_interrupted(self):
        self.manifest["processes"]["download_owner"]["start_ticks"] += 1
        self.assertEqual(self.start().step()["state"], "interrupted")

    def test_missing_root_is_interrupted(self):
        self.runner.rmdir()
        self.assertEqual(self.start().step()["state"], "interrupted")

    def test_partial_terminal_is_not_completed(self):
        (self.download / "result.json").write_text(json.dumps({"state": "completed", "elapsed_seconds": 28801, "eight_hours_verified": True}))
        self.assertEqual(self.start().step()["state"], "watching")

    def test_failed_and_unverified_terminal_are_failed(self):
        self.write_results("failed")
        collector = self.start()
        self.assertEqual(collector.step()["state"], "failed")
        self.write_results()
        (self.guardian / "result.json").write_text(json.dumps({"state": "completed", "elapsed_seconds": 10}))
        self.assertEqual(collector.step()["state"], "failed")

    def test_nonfinite_elapsed_is_rejected(self):
        self.write_results()
        (self.download / "result.json").write_text(json.dumps({"state": "completed", "elapsed_seconds": float("inf"), "eight_hours_verified": True}))
        self.assertEqual(self.start().step()["state"], "failed")

    def test_oversized_log_omitted_no_temp_left(self):
        (self.runner / "test.log").write_bytes(b"x" * (archive.FILE_LIMIT + 1))
        value = self.start().step()
        self.assertEqual(value["files"]["download-test.log"]["omitted"], "file limit")
        self.assertFalse((self.destination / "download-test.log").exists())
        self.assertFalse(list(self.destination.glob("*.tmp")))

    def test_symlink_file_and_root_are_rejected(self):
        (self.runner / "test.log").symlink_to(self.download / "sentinel")
        (self.download / "sentinel").write_text("synthetic sentinel")
        collector = self.start()
        with self.assertRaises(OSError):
            collector.step()
        (self.runner / "test.log").unlink()
        collector.step()
        shutil.rmtree(self.guardian)
        self.guardian.symlink_to(self.download, target_is_directory=True)
        with self.assertRaises(archive.EvidenceError):
            collector.step()

    def test_root_manifest_mismatch_and_existing_destination(self):
        self.manifest["roots"]["download"] += "-wrong"
        with self.assertRaises(archive.EvidenceError):
            self.start()
        self.manifest["roots"]["download"] = str(self.download)
        self.destination.mkdir()
        (self.destination / "sentinel").write_text("keep")
        with self.assertRaises(archive.EvidenceError):
            self.start()
        self.assertEqual((self.destination / "sentinel").read_text(), "keep")

    def test_destination_injected_symlink_is_rejected(self):
        collector = self.start()
        (self.destination / "download-test.log").symlink_to(self.download / "sentinel")
        with self.assertRaises(archive.EvidenceError):
            collector.step()

    def test_corrupt_oversized_json_fail_closed(self):
        (self.download / "result.json").write_text("not-json")
        collector = self.start()
        with self.assertRaises(archive.EvidenceError):
            collector.step()
        (self.download / "result.json").write_bytes(b"x" * (archive.JSON_LIMIT + 1))
        with self.assertRaises(archive.EvidenceError):
            collector.step()

    def test_history_rows_and_aggregate_bound(self):
        collector = self.start()
        row = {"observed_utc": "synthetic", "state": "watching", "boot_id": self.manifest["boot_id"], "processes": {}}
        collector.history = [row] * 600
        collector.step()
        lines = (self.destination / "progress-history.jsonl").read_bytes().splitlines()
        self.assertEqual(len(lines), archive.HISTORY_LIMIT)
        self.assertLessEqual(sum(path.stat().st_size for path in self.destination.iterdir()), archive.TOTAL_LIMIT)
        self.assertLessEqual(max(path.stat().st_size for path in self.destination.iterdir()), archive.FILE_LIMIT)

    def test_aggregate_reserve_omits_excess_allowed_files(self):
        for root_name, files in archive.ALLOWLIST.items():
            for relative in files:
                if relative.endswith("result.json"):
                    continue
                target = self.roots[root_name] / relative
                target.parent.mkdir(exist_ok=True)
                target.write_bytes(b"x" * archive.FILE_LIMIT)
        collector = self.start()
        value = collector.step()
        self.assertTrue(any(item.get("omitted") == "aggregate reserve" for item in value["files"].values()))
        self.assertLessEqual(sum(path.stat().st_size for path in self.destination.iterdir()), archive.TOTAL_LIMIT)

    def test_existing_connected_root_disappearance_is_interrupted(self):
        collector = self.start()
        self.assertEqual(collector.step()["state"], "watching")
        self.runner.rmdir()
        self.assertEqual(collector.step()["state"], "interrupted")

    def test_fifo_source_is_rejected_without_blocking(self):
        os.mkfifo(self.runner / "test.log")
        with self.assertRaises(archive.EvidenceError):
            self.start().step()

    def scoped_observation(self, pid, expected):
        return {"pid": pid, "expected_start_ticks": expected, "actual_start_ticks": expected if pid == os.getpid() else None,
                "matches": pid == os.getpid()}

    def test_guardian_finishes_first_and_reaped_group_keeps_archive_watching(self):
        self.manifest["processes"]["guardian_owner"] = {"pid": 999999, "start_ticks": 1}
        (self.guardian / "result.json").write_text(json.dumps({"state": "completed", "elapsed_seconds": 28801,
                                                             "eight_hours_elapsed": True, "descendants_reaped": True}))
        with mock.patch.object(archive, "process_observation", side_effect=self.scoped_observation):
            collector = self.start()
            value = collector.step()
            self.assertEqual(value["state"], "watching")
            self.assertTrue(value["processes"]["guardian_owner"]["expected_terminal_exit"])
            self.assertFalse(value["processes"]["guardian_owner"]["matches"])
            self.assertFalse((self.destination / "archive-result.json").exists())
            self.write_results()
            self.assertEqual(collector.step()["state"], "completed")

    def test_download_finishes_first_and_reaped_group_keeps_archive_watching(self):
        self.manifest["processes"]["guardian_owner"] = dict(self.manifest["processes"]["download_owner"])
        self.manifest["processes"]["download_owner"] = {"pid": 999999, "start_ticks": 1}
        self.write_results()
        (self.guardian / "result.json").unlink()
        with mock.patch.object(archive, "process_observation", side_effect=self.scoped_observation):
            value = self.start().step()
        self.assertEqual(value["state"], "watching")
        self.assertTrue(value["processes"]["download_owner"]["expected_terminal_exit"])

    def test_one_complete_scope_does_not_excuse_other_scope_unexpected_death(self):
        self.manifest["processes"]["guardian_owner"] = {"pid": 999998, "start_ticks": 1}
        self.manifest["processes"]["download_owner"] = {"pid": 999999, "start_ticks": 1}
        (self.guardian / "result.json").write_text(json.dumps({"state": "completed", "elapsed_seconds": 28801,
                                                             "eight_hours_elapsed": True, "descendants_reaped": True}))
        with mock.patch.object(archive, "process_observation", side_effect=self.scoped_observation):
            value = self.start().step()
        self.assertEqual(value["state"], "interrupted")
        self.assertFalse(value["processes"]["download_owner"]["expected_terminal_exit"])

    def test_invalid_single_scope_terminal_is_failed_even_if_other_running(self):
        (self.guardian / "result.json").write_text(json.dumps({"state": "completed", "elapsed_seconds": 3,
                                                             "eight_hours_elapsed": True, "descendants_reaped": True}))
        self.assertEqual(self.start().step()["state"], "failed")

    def test_unknown_process_role_is_fail_closed(self):
        self.manifest["processes"]["unrelated_owner"] = self.manifest["processes"].pop("download_owner")
        with self.assertRaises(archive.EvidenceError):
            self.start()

    def test_runner_terminal_appears_during_copy_uses_preserved_snapshot(self):
        self.write_results()
        (self.runner / "result.json").unlink()
        collector = self.start()
        original = collector.copy_file

        def publish_during_copy(source, relative, target):
            if target == "runner-result.json":
                (self.runner / "result.json").write_text(json.dumps({"state": "completed", "exit_code": 0}))
            return original(source, relative, target)

        gone = {"pid": os.getpid(), "expected_start_ticks": 1,
                "actual_start_ticks": None, "matches": False}
        with mock.patch.object(collector, "copy_file", side_effect=publish_during_copy), \
                mock.patch.object(archive, "process_observation", return_value=gone):
            value = collector.step()
        self.assertEqual(value["state"], "completed")
        self.assertEqual(value["scope_terminal_proofs"]["download"], "completed")
        saved = (self.destination / "runner-result.json").read_bytes()
        self.assertEqual(value["files"]["runner-result.json"]["sha256"], hashlib.sha256(saved).hexdigest())
        self.assertTrue(value["processes"]["download_owner"]["expected_terminal_exit"])

    def test_current_step_missing_terminal_does_not_reuse_old_copy(self):
        self.write_results()
        collector = self.start()
        self.assertEqual(collector.step()["state"], "completed")
        (self.runner / "result.json").unlink()
        gone = {"pid": os.getpid(), "expected_start_ticks": 1,
                "actual_start_ticks": None, "matches": False}
        with mock.patch.object(archive, "process_observation", return_value=gone):
            value = collector.step()
        self.assertEqual(value["state"], "interrupted")
        self.assertIsNone(value["scope_terminal_proofs"]["download"])

    def test_owner_death_during_copy_without_terminal_is_interrupted(self):
        collector = self.start()
        original = collector.copy_file
        copying_started = False

        def copying(source, relative, target):
            nonlocal copying_started
            copying_started = True
            return original(source, relative, target)

        def observe(pid, ticks):
            return {"pid": pid, "expected_start_ticks": ticks,
                    "actual_start_ticks": None if copying_started else ticks,
                    "matches": not copying_started}

        with mock.patch.object(collector, "copy_file", side_effect=copying), \
                mock.patch.object(archive, "process_observation", side_effect=observe):
            value = collector.step()
        self.assertEqual(value["state"], "interrupted")
        self.assertFalse(value["processes"]["download_owner"]["matches"])

    def test_omitted_terminal_does_not_reuse_previous_completed_bytes(self):
        self.write_results()
        collector = self.start()
        self.assertEqual(collector.step()["state"], "completed")
        (self.runner / "result.json").write_bytes(b"x" * (archive.FILE_LIMIT + 1))
        gone = {"pid": os.getpid(), "expected_start_ticks": 1,
                "actual_start_ticks": None, "matches": False}
        with mock.patch.object(archive, "process_observation", return_value=gone):
            value = collector.step()
        self.assertEqual(value["state"], "interrupted")
        self.assertNotIn("runner-result.json", value["terminal_snapshot_files"])
        self.assertEqual(value["files"]["runner-result.json"]["omitted"], "file limit")

    def test_copy_snapshot_remains_authoritative_if_source_changes_after_copy(self):
        self.write_results()
        collector = self.start()
        original = collector.copy_file

        def change_source_after_copy(source, relative, target):
            copied = original(source, relative, target)
            if target == "runner-result.json":
                (self.runner / "result.json").write_text(json.dumps({"state": "failed", "exit_code": 42}))
            return copied

        with mock.patch.object(collector, "copy_file", side_effect=change_source_after_copy):
            value = collector.step()
        self.assertEqual(value["state"], "completed")
        self.assertEqual(json.loads((self.destination / "runner-result.json").read_text())["exit_code"], 0)
        self.assertEqual(json.loads((self.runner / "result.json").read_text())["exit_code"], 42)

    def test_changed_copied_terminal_hash_is_rejected(self):
        self.write_results()
        collector = self.start()
        original = collector.copy_file

        def change_destination_after_copy(source, relative, target):
            copied = original(source, relative, target)
            if target == "runner-result.json":
                (self.destination / target).write_text(json.dumps({"state": "completed", "exit_code": 0, "extra": True}))
            return copied

        with mock.patch.object(collector, "copy_file", side_effect=change_destination_after_copy):
            with self.assertRaisesRegex(archive.EvidenceError, "copied terminal snapshot changed"):
                collector.step()

    def test_late_nonzero_runner_exit_is_failed(self):
        self.write_results()
        (self.runner / "result.json").unlink()
        collector = self.start()
        original = collector.copy_file

        def publish_failure(source, relative, target):
            if target == "runner-result.json":
                (self.runner / "result.json").write_text(json.dumps({"state": "completed", "exit_code": 42}))
            return original(source, relative, target)

        with mock.patch.object(collector, "copy_file", side_effect=publish_failure):
            self.assertEqual(collector.step()["state"], "failed")

    def test_atomic_failure_cleans_only_owned_temporary(self):
        collector = self.start()
        with mock.patch.object(archive.os, "rename", side_effect=OSError("synthetic failure")):
            with self.assertRaises(OSError):
                archive.atomic_bytes(collector.dest_fd, "archive-status.json", b"new")
        self.assertFalse(any(path.name.startswith(".archive-") for path in self.destination.iterdir()))
        self.assertTrue((self.destination / "ownership-manifest.json").exists())


if __name__ == "__main__":
    unittest.main()
