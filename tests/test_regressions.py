"""Run with python3 -m unittest discover -s tests -v.

System commands are replaced by shell functions; all configuration writes use
temporary XDG directories. No real login, package, policy or desktop changes run.
The welcome tests use Qt's offscreen backend (python3-pyside6 required).
"""

import importlib.machinery
import importlib.util
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]


class ShellTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="lnp-regression-")
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.config = self.root / "config"
        self.config.mkdir()
        self.state = self.root / "state" / "lnp"
        self.state.mkdir(parents=True)
        self.trace = self.root / "trace"
        self.env = dict(
            os.environ,
            XDG_CONFIG_HOME=str(self.config),
            XDG_STATE_HOME=str(self.state.parent),
            TEST_TRACE=str(self.trace),
            TEST_LAYOUT_VERSION="2",
        )

    def shell(self, code):
        return subprocess.run(
            ["bash", "-c", code], env=self.env,
            text=True, capture_output=True, timeout=10,
        )

    def apply(self, command="main", overrides=""):
        return self.shell(
            "source " + shlex.quote(str(ROOT / "apply/lnp-apply-layout")) + "\n"
            + '''
shipped_version() { echo "$TEST_LAYOUT_VERSION"; }
in_plasma_session() { return 0; }
configure_touchpads() { return 0; }
plasma-apply-lookandfeel() {
    echo applied >> "$TEST_TRACE"
    printf 'LNP config\\n' > "$CONFIG_DIR/kdeglobals"
    printf 'new touchpad config\\n' > "$CONFIG_DIR/kcminputrc"
}
'''
            + overrides + "\n" + command
        )

    def assert_ok(self, result):
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_revert_preserves_opt_out_across_login_and_layout_upgrade(self):
        (self.config / "kdeglobals").write_text("original\n")
        self.assert_ok(self.apply())
        self.assert_ok(self.apply("main --revert"))
        self.assertEqual((self.config / "kdeglobals").read_text(), "original\n")
        self.assertFalse((self.config / "kcminputrc").exists())
        self.assertTrue((self.state / "auto-apply-disabled").exists())
        self.assert_ok(self.apply())
        self.env["TEST_LAYOUT_VERSION"] = "3"
        self.assert_ok(self.apply())
        self.assertEqual(self.trace.read_text(), "applied\n")
        self.assert_ok(self.apply("main --force"))
        self.assertFalse((self.state / "auto-apply-disabled").exists())
        self.assertEqual((self.state / "applied-version").read_text(), "3\n")
        self.assert_ok(self.apply())
        self.assertEqual(self.trace.read_text(), "applied\napplied\n")

    def test_failed_backup_keeps_latest_and_does_not_apply(self):
        (self.config / "kdeglobals").write_text("original\n")
        self.assert_ok(self.apply())
        latest = (self.state / "backup-latest").resolve()
        result = self.apply("main --force", "cp() { return 1; }")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual((self.state / "backup-latest").resolve(), latest)
        self.assertEqual((latest / "kdeglobals").read_text(), "original\n")
        self.assertEqual(self.trace.read_text(), "applied\n")

    def test_failed_publication_does_not_apply(self):
        (self.config / "kdeglobals").write_text("original\n")
        result = self.apply(overrides="ln() { return 1; }")
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.trace.exists())
        self.assertFalse((self.state / "backup-latest").exists())
        self.assertEqual((self.config / "kdeglobals").read_text(), "original\n")

    def test_backup_names_are_unique_within_same_second(self):
        self.assert_ok(self.apply(
            "main --force\nmain --force", "date() { echo 20260913-000000; }"
        ))
        backups = list(self.state.glob("backup-20260913-000000.*"))
        self.assertEqual(len(backups), 2)
        self.assertTrue(all((p / "manifest").is_file() for p in backups))

    def test_corrupt_backup_is_rejected_before_any_restore(self):
        (self.config / "kdeglobals").write_text("original\n")
        (self.config / "kscreenlockerrc").write_text("original lock config\n")
        self.assert_ok(self.apply())
        (self.state / "backup-latest" / "kscreenlockerrc").unlink()
        result = self.apply("main --revert")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual((self.config / "kdeglobals").read_text(), "LNP config\n")
        self.assertTrue((self.config / "kcminputrc").exists())

    def test_failed_restore_reports_failure_and_keeps_opt_out(self):
        (self.config / "kdeglobals").write_text("original\n")
        self.assert_ok(self.apply())
        result = self.apply("main --revert", "cp() { return 1; }")
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue((self.state / "auto-apply-disabled").exists())
        self.assert_ok(self.apply())
        self.assertEqual(self.trace.read_text(), "applied\n")

    def test_failed_explicit_apply_does_not_clear_opt_out(self):
        self.assert_ok(self.apply())
        self.assert_ok(self.apply("main --revert"))
        result = self.apply("main --force", "plasma-apply-lookandfeel() { return 1; }")
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue((self.state / "auto-apply-disabled").exists())

    def test_legacy_backup_preserves_unrecorded_files(self):
        backup = self.state / "backup-legacy"
        backup.mkdir()
        (backup / "kdeglobals").write_text("original\n")
        (self.state / "backup-latest").symlink_to(backup)
        (self.config / "kcminputrc").write_text("unknown origin\n")
        result = self.apply("main --revert")
        self.assert_ok(result)
        self.assertIn("legacy backup", result.stderr)
        self.assertEqual((self.config / "kdeglobals").read_text(), "original\n")
        self.assertEqual((self.config / "kcminputrc").read_text(), "unknown origin\n")

    def test_restore_replaces_symlink_without_overwriting_its_target(self):
        (self.config / "kdeglobals").write_text("original\n")
        self.assert_ok(self.apply())
        target = self.root / "unrelated"
        target.write_text("keep me\n")
        (self.config / "kdeglobals").unlink()
        (self.config / "kdeglobals").symlink_to(target)
        self.assert_ok(self.apply("main --revert"))
        self.assertEqual(target.read_text(), "keep me\n")
        self.assertFalse((self.config / "kdeglobals").is_symlink())
        self.assertEqual((self.config / "kdeglobals").read_text(), "original\n")

    def test_recovery_support_requires_login_even_when_authentication_fails(self):
        for status in (0, 1):
            with self.subTest(login_status=status):
                self.trace.write_text("")
                result = self.shell('''
read() { choice=3; }
clear() { :; }
sleep() { :; }
/usr/bin/login() { echo "login:$*" >> "$TEST_TRACE"; return ''' + str(status) + '''; }
/usr/bin/bash() { echo root-shell >> "$TEST_TRACE"; return 99; }
systemctl() { echo "systemctl:$*" >> "$TEST_TRACE"; }
source ''' + shlex.quote(str(ROOT / "recovery/lnp-recovery-prompt")))
                self.assert_ok(result)
                self.assertEqual(
                    self.trace.read_text(),
                    "login:\nsystemctl:restart display-manager.service\n",
                )

    def test_helper_rejects_legacy_unscoped_custom_policy_request(self):
        result = self.shell('''
id() { echo 0; }
ausearch() { echo audit-read >> "$TEST_TRACE"; }
audit2allow() { echo policy-build >> "$TEST_TRACE"; }
semodule() { echo policy-install >> "$TEST_TRACE"; }
source ''' + shlex.quote(str(ROOT / "selinux/lnp-selinux-fix")) + " allow-module example")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("usage: allow-module <alert-id>", result.stderr)
        self.assertFalse(self.trace.exists())


class WelcomeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        os.environ["QT_QPA_PLATFORM"] = "offscreen"
        try:
            from PySide6.QtWidgets import QApplication
        except ImportError:
            raise unittest.SkipTest("python3-pyside6 is needed for welcome UI tests")
        cls.app = QApplication.instance() or QApplication([])
        loader = importlib.machinery.SourceFileLoader("lnp_welcome", str(ROOT / "welcome/lnp-welcome"))
        spec = importlib.util.spec_from_loader(loader.name, loader)
        cls.welcome = importlib.util.module_from_spec(spec)
        loader.exec_module(cls.welcome)

    def page(self):
        page = self.welcome.ActionPage("codecs", "Install", "Ready")
        page.finish_layout()
        page.show()
        self.addCleanup(page.close)
        return page

    def test_partial_setup_failure_does_not_claim_rollback(self):
        page = self.page()
        page.log.appendPlainText("==> Added software source\nPlayback plugin installation failed")
        page.on_finished(1, self.welcome.QProcess.ExitStatus.NormalExit)
        self.assertIn("Some changes may already have been made", page.status.text())
        self.assertNotIn("nothing", page.status.text().lower())
        self.assertTrue(page.button.isEnabled())
        self.assertTrue(page.log.isVisible())

    def test_crashed_process_cannot_report_success_even_with_zero_code(self):
        page = self.page()
        page.on_finished(0, self.welcome.QProcess.ExitStatus.CrashExit)
        self.assertIn("did not finish", page.status.text())
        self.assertTrue(page.button.isEnabled())

    def test_helper_progress_prevents_false_authorization_only_failure(self):
        page = self.page()
        page.log.appendPlainText("==> Added software source\nError from authentication agent")
        page.on_finished(127, self.welcome.QProcess.ExitStatus.NormalExit)
        self.assertIn("Some changes may already have been made", page.status.text())

    def test_success_and_cancelled_authorization_stay_distinct(self):
        page = self.page()
        page.on_finished(126, self.welcome.QProcess.ExitStatus.NormalExit)
        self.assertIn("Nothing was changed", page.status.text())
        page.on_finished(0, self.welcome.QProcess.ExitStatus.NormalExit)
        self.assertIn("Ready", page.status.text())
        self.assertTrue(page.button.isHidden())


if __name__ == "__main__":
    unittest.main()
