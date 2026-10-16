"""Regressions for probes that must not interrupt the user's desktop."""
import os
import io
from pathlib import Path
import tempfile
import unittest
from unittest import mock
import native_probe


class IsolatedProbeTests(unittest.TestCase):
    def test_clipboard_exercise_rejects_desktop_before_launch(self):
        with mock.patch('sys.argv', ['probe', 'binary', '/tmp/probe.json', '--page', 'diagnostics', '--desktop', '--copy-report']), mock.patch('sys.stderr', new_callable=io.StringIO), mock.patch('native_probe.probe_display') as display:
            with self.assertRaises(SystemExit) as error:
                native_probe.main()
            self.assertEqual(error.exception.code, 2)
            display.assert_not_called()

    def test_pause_exercise_rejects_desktop_before_launch(self):
        with mock.patch('sys.argv', ['probe', 'binary', '/tmp/probe.json', '--page', 'history', '--recover-jobs', '--resume-job', '--pause-job', '--desktop']), mock.patch('sys.stderr', new_callable=io.StringIO), mock.patch('native_probe.probe_display') as display:
            with self.assertRaises(SystemExit) as error:
                native_probe.main()
            self.assertEqual(error.exception.code, 2)
            display.assert_not_called()

    def test_resume_exercise_rejects_desktop_before_launch(self):
        with mock.patch('sys.argv', ['probe', 'binary', '/tmp/probe.json', '--page', 'history', '--recover-jobs', '--resume-job', '--desktop']), mock.patch('sys.stderr', new_callable=io.StringIO), mock.patch('native_probe.probe_display') as display:
            with self.assertRaises(SystemExit) as error:
                native_probe.main()
            self.assertEqual(error.exception.code, 2)
            display.assert_not_called()

    def test_scroll_exercise_rejects_desktop_before_launch(self):
        with mock.patch('sys.argv', ['probe', 'binary', '/tmp/probe.json', '--desktop', '--scroll-end']), mock.patch('sys.stderr', new_callable=io.StringIO), mock.patch('native_probe.probe_display') as display:
            with self.assertRaises(SystemExit) as error:
                native_probe.main()
            self.assertEqual(error.exception.code, 2)
            display.assert_not_called()

    def test_missing_virtual_server_never_falls_back_to_desktop(self):
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch('native_probe.shutil.which', return_value=None), mock.patch('pathlib.Path.is_file', return_value=False), mock.patch('native_probe.subprocess.Popen') as launch:
                with self.assertRaisesRegex(RuntimeError, 'refusing to open'):
                    with native_probe.probe_display(Path(directory) / 'probe.json'):
                        self.fail('Missing Xvfb must not yield a display')
                launch.assert_not_called()

    def test_server_is_reaped_when_probe_raises(self):
        original_display = os.environ.get('DISPLAY')
        with tempfile.TemporaryDirectory() as directory:
            server = mock.Mock(pid=12345)
            server.poll.return_value = None
            with mock.patch('native_probe.shutil.which', return_value='/fake/Xvfb'), mock.patch('native_probe.subprocess.Popen', return_value=server) as launch, mock.patch('native_probe.select.select', return_value=([1], [], [])), mock.patch('native_probe.os.read', return_value=b'91\n'), mock.patch('native_probe.os.killpg') as kill:
                with self.assertRaisesRegex(RuntimeError, 'probe fixture failure'):
                    with native_probe.probe_display(Path(directory) / 'probe.json') as (display, kind):
                        self.assertEqual(display, ':91')
                        self.assertEqual(kind, 'Xvfb isolated')
                        raise RuntimeError('probe fixture failure')
                kill.assert_called_once_with(server.pid, native_probe.signal.SIGTERM)
                server.wait.assert_called_once_with(timeout=3)
                self.assertIn('-nolisten', launch.call_args.args[0])
                self.assertIn('tcp', launch.call_args.args[0])
        self.assertEqual(os.environ.get('DISPLAY'), original_display)


if __name__ == '__main__':
    unittest.main()
