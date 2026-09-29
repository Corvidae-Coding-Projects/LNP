"""Issue #2: helper failures use fake commands and temporary configuration."""
import importlib.machinery
import importlib.util
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
MOCKS = r'''
id() { echo 0; }
attempt() {
    echo "$1" >> "$TRACE"
    if [[ ",$FAIL," == *",$1,"* ]]; then
        echo "fixture error: $1" >&2
        return 1
    fi
}
rpm() { return 1; }
getenforce() { attempt selinux || return 1; echo Enforcing; }
journalctl() { attempt logs; }
dnf() { if [ "$1" = install ]; then attempt explainer; else attempt packages; fi; }
flatpak() { attempt runtimes; }
/usr/libexec/lnp-guard() { attempt snapshots; }
flatpak_available() { [ "$OPTIONAL" != absent ]; }
guard_available() { [ "$OPTIONAL" != absent ]; }
df() { attempt measure || return 1; printf 'Avail\n100\n'; }
systemctl() {
    case "$1" in
        show)
            attempt query || return 1
            if [ "$OPTIONAL" = absent ]; then echo not-found; else echo loaded; fi ;;
        enable)
            if [ "$3" = firewalld ]; then attempt firewall; else attempt firmware; fi ;;
        try-restart) attempt resolver ;;
        is-enabled) return 1 ;;
        *) echo "unexpected systemctl call: $*" >&2; return 99 ;;
    esac
}
mkdir() { attempt mkdir || return 1; command mkdir "$@"; }
mktemp() { attempt mktemp || return 1; command mktemp "$@"; }
chmod() { attempt chmod || return 1; command chmod "$@"; }
mv() { attempt publish || return 1; command mv "$@"; }
printf() {
    if [[ "$1" == '[Resolve]'* ]]; then attempt write || return 1; fi
    builtin printf "$@"
}
'''


class SetupFixture:
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='lnp-setup-test-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.config = self.root / 'resolved.conf.d'
        self.config.mkdir()
        (self.config / 'lnp-security.conf').write_text('original config\n')
        self.helper = self.root / 'setup'
        self.helper.write_text((ROOT / 'welcome/lnp-setup').read_text().replace(
            '/etc/systemd/resolved.conf.d', str(self.config)))
        self.trace = self.root / 'trace'

    def run_helper(self, action, fail='', optional='present', extra=''):
        return subprocess.run(['bash', '-c', 'source ' + shlex.quote(str(self.helper))
                               + '\n' + MOCKS + '\n' + extra + '\nmain ' + action],
                              env=dict(os.environ, FAIL=fail, OPTIONAL=optional, TRACE=str(self.trace)),
                              text=True, capture_output=True, timeout=10)


class SetupTests(SetupFixture, unittest.TestCase):
    def test_each_cleanup_failure_is_nonzero_and_preserves_error_details(self):
        for step in ('logs', 'packages', 'runtimes', 'snapshots', 'measure'):
            with self.subTest(step=step):
                result = self.run_helper('clean', step)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('fixture error: ' + step, result.stderr)
                self.assertIn('Some changes may already have been made', result.stderr)
                self.assertNotIn('Done.', result.stdout)

    def test_each_security_failure_is_nonzero_and_preserves_error_details(self):
        for step in ('explainer', 'selinux', 'firewall', 'firmware', 'query',
                     'mkdir', 'mktemp', 'write', 'chmod', 'publish', 'resolver'):
            with self.subTest(step=step):
                result = self.run_helper('secure', step)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('fixture error: ' + step, result.stderr)
                self.assertIn('Failed:', result.stdout)
                self.assertNotIn('Done.', result.stdout)

    def test_mixed_failure_retains_completed_steps_and_all_failure_count(self):
        result = self.run_helper('clean', 'logs,runtimes')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Completed: Clear downloaded package files', result.stdout)
        self.assertIn('Completed: Trim old system snapshots', result.stdout)
        self.assertIn('2 step(s) failed', result.stderr)

    def test_all_success_returns_zero_for_both_actions(self):
        for action in ('clean','secure'):
            with self.subTest(action=action):
                result = self.run_helper(action)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertIn('Done.', result.stdout)
                self.assertNotIn('Failed:', result.stdout)

    def test_absent_optional_components_are_skipped_without_running_them(self):
        for action in ('clean','secure'):
            with self.subTest(action=action):
                self.trace.write_text('')
                result = self.run_helper(action, optional='absent')
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn('Skipped:', result.stdout)
                self.assertIn('Completed available steps', result.stdout)
                calls = self.trace.read_text().splitlines()
                for forbidden in ('runtimes','snapshots','firewall','firmware','resolver','mkdir'):
                    self.assertNotIn(forbidden, calls)

    def test_failed_config_publication_preserves_existing_file_and_cleans_temp(self):
        for step in ('mkdir','mktemp','write','chmod','publish'):
            with self.subTest(step=step):
                result = self.run_helper('secure', step)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual((self.config / 'lnp-security.conf').read_text(), 'original config\n')
                self.assertEqual(list(self.config.iterdir()), [self.config / 'lnp-security.conf'])
                self.assertNotIn('resolver', self.trace.read_text().splitlines())

    def test_invalid_measurement_never_enters_shell_arithmetic(self):
        result = self.run_helper('clean', extra="df() { builtin printf 'Avail\\nbad[1]\\n'; }")
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn('Free space increased', result.stdout)

    def test_retry_after_partial_failure_completes(self):
        self.assertNotEqual(self.run_helper('secure', 'resolver').returncode, 0)
        result = self.run_helper('secure')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.config / 'lnp-security.conf').read_text(), '[Resolve]\nDNSOverTLS=opportunistic\n')

    def test_unprivileged_entrypoint_cannot_run_operations(self):
        result = self.run_helper('secure', extra='id() { echo 1000; }')
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.trace.exists())


class SetupGuiTests(SetupFixture, unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        os.environ['QT_QPA_PLATFORM'] = 'offscreen'
        try:
            from PySide6.QtWidgets import QApplication
        except ImportError:
            raise unittest.SkipTest('PySide6 is needed for UI tests')
        cls.app = QApplication.instance() or QApplication([])
        loader = importlib.machinery.SourceFileLoader('lnp_setup_gui_test', str(ROOT / 'welcome/lnp-welcome'))
        spec = importlib.util.spec_from_loader(loader.name, loader)
        cls.gui = importlib.util.module_from_spec(spec)
        loader.exec_module(cls.gui)

    def complete_page(self, action, result):
        page = self.gui.SecurePage() if action == 'secure' else self.gui.CleanPage({'disk_percent': 96})
        page.show()
        self.addCleanup(page.close)
        page.button.setEnabled(False)
        page.log.appendPlainText(result.stdout + result.stderr)
        page.on_finished(result.returncode, self.gui.QProcess.ExitStatus.NormalExit)
        return page

    def test_real_helper_failures_show_partial_state_and_allow_retry(self):
        for action, steps in [('clean', ('logs','packages','runtimes','snapshots')),
                              ('secure', ('explainer','firewall','firmware','resolver','publish'))]:
            for step in steps:
                with self.subTest(action=action, step=step):
                    page = self.complete_page(action, self.run_helper(action, step))
                    self.assertIn('Some changes may already have been made', page.status.text())
                    self.assertFalse(page.status.text().startswith('✓'))
                    self.assertTrue(page.button.isEnabled())
                    self.assertFalse(page.button.isHidden())
                    self.assertTrue(page.log.isVisible())
                    self.assertIn('fixture error: ' + step, page.log.toPlainText())

    def test_real_helper_success_and_skips_have_distinct_ui_results(self):
        for action in ('clean','secure'):
            with self.subTest(action=action):
                page = self.complete_page(action, self.run_helper(action))
                self.assertTrue(page.status.text().startswith('✓'))
                self.assertTrue(page.button.isHidden())
                page = self.complete_page(action, self.run_helper(action, optional='absent'))
                self.assertIn('skipped', page.status.text())
                self.assertFalse(page.status.text().startswith('✓'))
                self.assertTrue(page.log.isVisible())

    def test_split_output_preserves_skipped_marker_and_final_line(self):
        from types import SimpleNamespace
        page = self.gui.SecurePage()
        page.show()
        self.addCleanup(page.close)
        chunks = iter([b'==> Ski', b'pped: Missing component\nfinal detail', b''])
        page.proc = SimpleNamespace(readAllStandardOutput=lambda: next(chunks))
        page.on_output()
        page.on_output()
        page.on_finished(0, self.gui.QProcess.ExitStatus.NormalExit)
        self.assertIn('skipped', page.status.text())
        self.assertIn('==> Skipped: Missing component', page.log.toPlainText())
        self.assertTrue(page.log.toPlainText().endswith('final detail'))
