"""Notification tests use fake journals and a mocked desktop bus."""
import importlib.machinery
import importlib.util
import io
from pathlib import Path
import subprocess
import unittest
from unittest.mock import Mock, patch

ROOT = Path(__file__).resolve().parents[1]
loader = importlib.machinery.SourceFileLoader('lnp_errord', str(ROOT / 'errord/lnp-errord'))
spec = importlib.util.spec_from_loader(loader.name, loader)
errord = importlib.util.module_from_spec(spec)
loader.exec_module(errord)


class ErrordTests(unittest.TestCase):
    def test_04_clean_journal_exit_is_logged_and_restart_policy_covers_it(self):
        proc = Mock(stdout=io.StringIO(''))
        proc.wait.return_value = 0
        with patch.object(errord.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0)), \
             patch.object(errord.subprocess, 'Popen', return_value=proc), \
             patch.object(errord.threading, 'Thread'), patch('sys.stdout', new_callable=io.StringIO) as output:
            self.assertEqual(errord.follow(), 0)
        self.assertIn('journal follower stopped (status 0)', output.getvalue())
        unit = (ROOT / 'errord/lnp-errord.service').read_text()
        self.assertIn('Restart=always', unit)
        self.assertIn('StartLimitBurst=10', unit)

    def test_02_timeout_message_does_not_claim_restart_is_safe(self):
        story = errord.t_guard({'_SYSTEMD_UNIT': 'lnp-reboot-guard.service',
                               'MESSAGE': 'driver preparation failed or timed out; restart hold released'})
        self.assertEqual(story[0], 'guard:failed')
        self.assertIn('may not work', story[2])

    def test_07_distinct_burst_has_five_details_and_one_summary(self):
        limiter = errord.NotificationLimiter()
        with patch.object(errord.time, 'monotonic', return_value=0), \
             patch.object(errord, 'notify', return_value=True) as notify:
            for number in range(20):
                limiter.emit((f'unit:{number}', f'Failure {number}', 'body'))
            self.assertEqual(notify.call_count, 6)
            self.assertEqual(notify.call_args.args[0], 'Several parts of the system are having problems')
        with patch.object(errord.time, 'monotonic', return_value=300), \
             patch.object(errord, 'notify', return_value=True) as notify:
            limiter.emit(('new', 'New failure', 'body'))
            notify.assert_called_once_with('New failure', 'body')

    def test_07_disk_and_journal_share_budget_and_keep_repeat_limits(self):
        limiter = errord.NotificationLimiter()
        with patch.object(errord.time, 'monotonic', return_value=0), \
             patch.object(errord, 'notify', return_value=True) as notify:
            limiter.emit(errord.disk_story(96), errord.DISK_RATE_LIMIT)
            for number in range(20):
                limiter.emit((f'unit:{number}', 'title', 'body'))
            self.assertEqual(notify.call_count, 6)
        with patch.object(errord.time, 'monotonic', return_value=301), \
             patch.object(errord, 'notify', return_value=True) as notify:
            limiter.emit(errord.disk_story(96), errord.DISK_RATE_LIMIT)
            notify.assert_not_called()

    def test_07_concurrent_threads_cannot_exceed_budget(self):
        from concurrent.futures import ThreadPoolExecutor
        limiter = errord.NotificationLimiter()
        with patch.object(errord.time, 'monotonic', return_value=0), \
             patch.object(errord, 'notify', return_value=True) as notify:
            with ThreadPoolExecutor(max_workers=8) as pool:
                list(pool.map(lambda n: limiter.emit((str(n), 'title', 'body')), range(30)))
            self.assertEqual(notify.call_count, 6)

    def test_08_untrusted_process_and_unit_text_are_escaped_at_bus_boundary(self):
        entries = [
            {'MESSAGE': 'Out of memory: Killed process 42 (<b>&oops</b>)'},
            {'MESSAGE': 'Failed with result exit-code', 'UNIT': '<img>&bad.service'},
        ]
        for entry in entries:
            with self.subTest(entry=entry), patch.object(errord.subprocess, 'run',
                    return_value=subprocess.CompletedProcess([], 0, '', '')) as run:
                _, title, body = errord.translate(entry)
                errord.notify(title, body)
                args = run.call_args.args[0]
                sent_body = args[args.index('dialog-warning') + 2]
                self.assertNotIn('<', sent_body)
                self.assertNotIn('>', sent_body)
                self.assertIn('&lt;', sent_body)
                self.assertIn('&amp;', sent_body)
                self.assertIsInstance(args, list)
                self.assertNotIn('shell', run.call_args.kwargs)
