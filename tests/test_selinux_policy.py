"""No live policy changes: system-bus, compiler and installer calls are mocked."""

import importlib.machinery
import importlib.util
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]
loader = importlib.machinery.SourceFileLoader("lnp_selinux_policy", str(ROOT / "selinux/lnp-selinux-policy"))
spec = importlib.util.spec_from_loader(loader.name, loader)
policy = importlib.util.module_from_spec(spec)
loader.exec_module(policy)

UUID = "12345678-1234-1234-1234-123456789abc"
OTHER_UUID = "87654321-1234-1234-1234-123456789abc"
DATE = "1700000000000000"
RECORD = (
    'type=AVC msg=audit(1700000000.123:17): avc: denied { read } for pid=10 '
    'comm="httpd" name="secret" scontext=system_u:system_r:httpd_t:s0 '
    'tcontext=system_u:object_r:shadow_t:s0 tclass=file permissive=0\n'
)
UNRELATED = RECORD.replace("httpd_t", "sshd_t").replace(":17)", ":18)")
SYSCALL = 'type=SYSCALL msg=audit(1700000000.123:17): arch=c000003e syscall=0\n'


def alert(records=None, uuid=UUID, date=DATE):
    return (uuid, "Selected alert", 1, records if records is not None else [RECORD, SYSCALL], [], 0, int(date), "red")


class PolicyTests(unittest.TestCase):
    def setUp(self):
        self.fetch = patch.object(policy, "load_alert").start()
        self.fetch.return_value = alert()
        self.run = patch.object(policy.subprocess, "run").start()
        self.run.side_effect = self.fake_tool
        patch.object(policy.syslog, "syslog").start()
        self.addCleanup(patch.stopall)

    def fake_tool(self, args, **kwargs):
        if args[0] == "/usr/bin/audit2allow":
            (Path(kwargs["cwd"]) / (args[-1] + ".pp")).write_bytes(b"compiled fixture")
        return subprocess.CompletedProcess(args, 0, "", "")

    def apply(self):
        return policy.apply_policy(UUID, DATE, RECORD)

    def test_compiler_receives_only_the_selected_authoritative_denial(self):
        def fetch(uuid):
            return {UUID: alert(), OTHER_UUID: alert([UNRELATED], uuid=OTHER_UUID)}[uuid]
        self.fetch.side_effect = fetch
        name = self.apply()
        self.assertTrue(name.startswith("lnp_alert_12345678123412341234123456789abc_"))
        self.assertEqual([call.args[0] for call in self.fetch.call_args_list], [UUID, UUID])
        compile_call, install_call = self.run.call_args_list
        self.assertEqual(compile_call.args[0], ["/usr/bin/audit2allow", "-N", "-M", name])
        self.assertEqual(compile_call.kwargs["input"], RECORD)
        self.assertNotIn(SYSCALL, compile_call.kwargs["input"])
        self.assertNotIn(UNRELATED, compile_call.kwargs["input"])
        self.assertEqual(install_call.args[0][:4], ["/usr/sbin/semodule", "-X", "300", "-i"])
        self.assertFalse(Path(compile_call.kwargs["cwd"]).exists())

    def test_module_identity_is_stable_for_retries_and_distinct_for_other_denials(self):
        original = self.apply()
        self.assertEqual(self.apply(), original)
        self.fetch.return_value = alert([UNRELATED])
        different = policy.apply_policy(UUID, DATE, UNRELATED)
        self.assertNotEqual(original, different)

    def test_forged_or_stale_record_never_reaches_compiler(self):
        for trusted in (alert([UNRELATED]), alert(date="1700000001000000"), alert(uuid=OTHER_UUID)):
            with self.subTest(trusted=trusted):
                self.fetch.return_value = trusted
                with self.assertRaises(ValueError):
                    self.apply()
        self.run.assert_not_called()

    def test_missing_alert_never_reaches_compiler(self):
        self.fetch.side_effect = RuntimeError("Alert not found")
        with self.assertRaisesRegex(RuntimeError, "Alert not found"):
            self.apply()
        self.run.assert_not_called()

    def test_multiple_denials_in_same_alert_event_are_not_combined(self):
        for records in ([RECORD, UNRELATED], [SYSCALL], []):
            self.fetch.return_value = alert(records)
            with self.assertRaisesRegex(ValueError, "unambiguous"):
                self.apply()
        self.run.assert_not_called()

    def test_invalid_request_is_rejected_before_bus_access(self):
        for args in (("../../module", DATE, RECORD), (UUID, "-1", RECORD), (UUID, DATE, "x" * 16385)):
            with self.assertRaises(ValueError):
                policy.apply_policy(*args)
        self.fetch.assert_not_called()
        self.run.assert_not_called()

    def test_multiline_and_granted_records_are_not_compiled(self):
        for record in (RECORD + UNRELATED, RECORD.replace("denied", "granted"), RECORD + "\x00"):
            self.fetch.return_value = alert([record])
            with self.assertRaisesRegex(ValueError, "supported access denial"):
                policy.apply_policy(UUID, DATE, record)
        self.run.assert_not_called()

    def test_build_failure_does_not_install_and_cleans_up(self):
        self.run.side_effect = lambda args, **kwargs: subprocess.CompletedProcess(args, 1, "", "compile failed")
        with self.assertRaisesRegex(RuntimeError, "compile failed"):
            self.apply()
        self.assertEqual(self.run.call_count, 1)
        self.assertFalse(Path(self.run.call_args.kwargs["cwd"]).exists())

    def test_missing_compiled_package_does_not_install(self):
        self.run.side_effect = lambda args, **kwargs: subprocess.CompletedProcess(args, 0, "", "")
        with self.assertRaisesRegex(RuntimeError, "did not build"):
            self.apply()
        self.assertEqual(self.run.call_count, 1)

    def test_changed_alert_during_compilation_does_not_install(self):
        self.fetch.side_effect = [alert(), alert([UNRELATED])]
        with self.assertRaisesRegex(ValueError, "changed"):
            self.apply()
        self.assertEqual(self.run.call_count, 1)

    def test_installation_failure_is_reported_and_cleans_up(self):
        def tool(args, **kwargs):
            if args[0].endswith("semodule"):
                return subprocess.CompletedProcess(args, 1, "", "install failed")
            return self.fake_tool(args, **kwargs)
        self.run.side_effect = tool
        with self.assertRaisesRegex(RuntimeError, "install failed"):
            self.apply()
        self.assertFalse(Path(self.run.call_args.kwargs["cwd"]).exists())

    def test_user_cannot_invoke_privileged_entrypoint_directly(self):
        with patch.object(policy.os, "geteuid", return_value=1000):
            self.assertEqual(policy.main(), 1)
        self.fetch.assert_not_called()
        self.run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
