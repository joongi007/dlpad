"""Synthetic observations only; no host changes or real process signals."""
import unittest
from unittest.mock import patch

import wsl_interruption_snapshot as snapshot

BOOT = "0bc0c937-eee9-4d90-95dc-31a2122b6352"
OTHER = "794988eb-539b-437e-8396-330738b565bb"


class ObservationTests(unittest.TestCase):
    def test_boot_validation_rejects_hyphen_only_and_malformed(self):
        for value in ["-" * 36, "0" * 36, BOOT[:-1], None, True]:
            self.assertFalse(snapshot.valid_boot(value))
        self.assertTrue(snapshot.valid_boot(BOOT.upper()))

    def test_changed_boot_never_reads_old_pid(self):
        with patch.object(snapshot, "read", side_effect=AssertionError("old PID read")):
            self.assertEqual(snapshot.process({"pid": 12, "start_ticks": 42}, OTHER, BOOT),
                             {"status": "different-boot-not-inspected"})

    def test_invalid_boot_never_reads_pid(self):
        with patch.object(snapshot, "read", side_effect=AssertionError("PID read")):
            with self.assertRaisesRegex(ValueError, "boot identity"):
                snapshot.process({"pid": 12, "start_ticks": 42}, "bad", BOOT)

    def test_same_boot_case_and_identity_match(self):
        data = "12 (synthetic worker) " + " ".join(["S"] + ["0"] * 18 + ["42"])
        with patch.object(snapshot, "read", return_value=data):
            value = snapshot.process({"pid": 12, "start_ticks": 42}, BOOT.upper(), BOOT)
        self.assertEqual(value["status"], "identity-matches-live")

    def test_permission_and_malformed_observations_are_unavailable(self):
        for failure in [PermissionError(), ValueError("bound")]:
            with patch.object(snapshot, "read", side_effect=failure):
                self.assertEqual(snapshot.process({"pid": 12, "start_ticks": 42}, BOOT, BOOT)["status"],
                                 "observation-unavailable")
        with patch.object(snapshot, "read", return_value="malformed"):
            self.assertEqual(snapshot.process({"pid": 12, "start_ticks": 42}, BOOT, BOOT)["status"],
                             "observation-unavailable")

    def test_absence_is_only_local_observation(self):
        with patch.object(snapshot, "read", side_effect=FileNotFoundError()):
            value = snapshot.process({"pid": 12, "start_ticks": 42}, BOOT, BOOT)
        self.assertEqual(value["status"], "absent")
        with patch.object(snapshot.os, "stat", side_effect=PermissionError()):
            scope = snapshot.observation_scope()
        self.assertFalse(scope["host_visibility_verified"])
        self.assertTrue(scope["absence_is_not_wsl_shutdown_proof"])
        self.assertEqual(scope["pid_namespaces"]["self"]["read_error"], "PermissionError")


if __name__ == "__main__":
    unittest.main()
