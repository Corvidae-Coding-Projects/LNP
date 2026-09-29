"""Issue #1: fake commands and temporary trees; never mutate the host system."""
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


def local_source(relative):
    return (ROOT / relative).read_text().replace(
        '/usr/libexec/lnp-btrfs-common', shlex.quote(str(ROOT / 'guard/lnp-btrfs-common')))



class RecoveryTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="lnp-recovery-test-")
        self.addCleanup(self.tmp.cleanup)
        self.base = Path(self.tmp.name)
        self.top = self.base / "top"
        self.snapshot = self.top / "snapshots/20260928-000000-pre-update"
        for path in (self.snapshot / "usr/lib/modules/1/kernel", self.top / "root",
                     self.base / "modules/1/kernel", self.base / "boot"):
            path.mkdir(parents=True)
        (self.top / "root/identity").write_text("old")
        (self.snapshot / "identity").write_text("restored")
        for name in ("vmlinuz-1", "initramfs-1.img", "vmlinuz-2"):
            (self.base / "boot" / name).write_text("fixture")
        self.default = self.base / "default"
        self.default.write_text(str(self.base / "boot/vmlinuz-2") + "\n")
        self.script = self.base / "restore"
        source = local_source("recovery/lnp-restore")
        source = source.replace("/run/lnp/top", str(self.top))
        source = source.replace('"/usr/lib/modules/', '"' + str(self.base / "modules") + '/')
        source = source.replace('/boot/', str(self.base / "boot") + '/')
        self.script.write_text(source)
        self.env = dict(os.environ, TEST_BASE=str(self.base), DEFAULT=str(self.default))

    def run_restore(self, extra="", command='cmd_restore 20260928-000000-pre-update'):
        mocks = r'''
mount_top() { :; }
umount_top() { :; }
btrfs() {
    echo "$*" >> "$TEST_BASE/trace"
    case "$2" in
        show) [ -d "$3" ] ;;
        snapshot) cp -a -- "$3" "$4" ;;
        delete) rm -r -- "$3" ;;
        *) return 99 ;;
    esac
}
grubby() {
    case "$1" in
        --default-kernel) cat "$DEFAULT" ;;
        --info=*) printf 'args="ro rootflags=subvol=root quiet"\ninitrd="%s/boot/initramfs-1.img $tuned_initrd"\n' "$TEST_BASE" ;;
        --set-default=*) printf '%s\n' "${1#*=}" > "$DEFAULT" ;;
        *) return 99 ;;
    esac
}
'''
        return subprocess.run(["bash", "-c", "source " + shlex.quote(str(self.script)) + "\n" + mocks + extra + "\n" + command], env=self.env, text=True, capture_output=True, timeout=10)

    def test_01_restore_selects_kernel_with_modules_and_preserves_old_root(self):
        result = self.run_restore()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.default.read_text().strip(), str(self.base / 'boot/vmlinuz-1'))
        self.assertEqual((self.top / 'root/identity').read_text(), 'restored')
        saved = list(self.top.glob('snapshots/replaced-roots/root.before-restore-*'))
        self.assertEqual(len(saved), 1)
        self.assertEqual((saved[0] / 'identity').read_text(), 'old')

    def test_01_missing_boot_image_refuses_without_mutation(self):
        for name in ('vmlinuz-1', 'initramfs-1.img'):
            with self.subTest(name=name):
                path = self.base / 'boot' / name
                path.unlink()
                result = self.run_restore()
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual((self.top / 'root/identity').read_text(), 'old')
                self.assertTrue(self.default.read_text().strip().endswith('vmlinuz-2'))
                path.write_text('fixture')

    def test_01_bootloader_failure_keeps_root(self):
        result = self.run_restore(r'''
grubby() {
    case "$1" in
        --default-kernel) cat "$DEFAULT" ;;
        --info=*) printf 'args="rootflags=subvol=root"\ninitrd="%s/boot/initramfs-1.img"\n' "$TEST_BASE" ;;
        --set-default=*) [[ "$1" == *vmlinuz-2 ]] ;;
    esac
}
''')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual((self.top / 'root/identity').read_text(), 'old')
        self.assertFalse(list(self.top.glob('snapshots/replaced-roots/root.before-restore-*')))

    def test_01_failed_root_switch_restores_root_and_boot_default(self):
        result = self.run_restore(r'''
mv() {
    [[ "$3" == *'/.lnp-restore.'* ]] && return 1
    command mv "$@"
}
''')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual((self.top / 'root/identity').read_text(), 'old')
        self.assertTrue(self.default.read_text().strip().endswith('vmlinuz-2'))

    def test_01_path_traversal_is_rejected(self):
        result = self.run_restore(command='cmd_restore ../root')
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.base / 'trace').exists())

    def test_01_failed_snapshot_does_not_change_root_or_boot(self):
        result = self.run_restore(r'''
btrfs() { case "$2" in show) return 0 ;; snapshot) return 1 ;; *) return 99 ;; esac; }
''')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual((self.top / 'root/identity').read_text(), 'old')
        self.assertTrue(self.default.read_text().strip().endswith('vmlinuz-2'))

    def test_01_subvolume_id_boot_entry_is_rejected(self):
        result = self.run_restore(r'''
grubby() { printf 'args="rootflags=subvolid=256"\ninitrd="%s/boot/initramfs-1.img"\n' "$TEST_BASE"; }
''')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual((self.top / 'root/identity').read_text(), 'old')

    def test_01_removed_kernel_modules_are_rejected(self):
        (self.base / 'modules/1/kernel').rmdir()
        result = self.run_restore()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual((self.top / 'root/identity').read_text(), 'old')



class GuardTests(unittest.TestCase):
    def shell(self, code):
        return subprocess.run(['bash', '-c', local_source('guard/lnp-guard').split('if [[ "${BASH_SOURCE[0]}" = "$0" ]]')[0] + '\n' + code], text=True, capture_output=True, timeout=10)

    def test_02_timeout_releases_hold_without_claiming_success(self):
        result = self.shell('timeout() { echo "timeout:$*"; return 124; }; cmd_wait')
        self.assertEqual(result.returncode, 124)
        self.assertIn('--kill-after=10s 30m', result.stdout)
        self.assertIn('restart hold released', result.stdout)
        self.assertNotIn('restart is safe again', result.stdout)

    def test_02_readiness_success_reports_success(self):
        result = self.shell('timeout() { return 0; }; cmd_wait')
        self.assertEqual(result.returncode, 0)
        self.assertIn('restart is safe again', result.stdout)

    def test_02_failed_poll_is_not_reported_as_ready(self):
        result = self.shell('timeout() { return 1; }; cmd_wait')
        self.assertEqual(result.returncode, 1)
        self.assertIn('restart hold released', result.stdout)

    def test_02_unit_bounds_entire_process_tree(self):
        unit = (ROOT / 'guard/lnp-reboot-guard.service').read_text()
        self.assertIn('RuntimeMaxSec=31min', unit)
        self.assertIn('TimeoutStopSec=10s', unit)
        self.assertNotIn('KillMode=process', unit)


class RetentionTests(unittest.TestCase):
    def test_03_retention_preserves_mounted_default_and_two_newest_roots(self):
        with tempfile.TemporaryDirectory(prefix='lnp-retention-') as directory:
            top = Path(directory)
            archive = top / 'snapshots/replaced-roots'
            archive.mkdir(parents=True)
            paths = []
            for n in range(1, 7):
                path = archive / f'root.before-restore-202609{n:02}-000000'
                path.mkdir()
                (path / 'id').write_text(str(n))
                paths.append(path)
            legacy = top / 'root.before-restore-20260801-000000'
            legacy.mkdir()
            (legacy / 'id').write_text('20')
            unrelated = archive / 'keep-me'
            unrelated.mkdir()
            source = local_source('guard/lnp-guard').replace('/run/lnp/top', str(top))
            script = top / 'guard'
            script.write_text(source)
            result = subprocess.run(['bash', '-c', 'source ' + shlex.quote(str(script)) + r'''
findmnt() { echo 'rw,subvolid=1,subvol=/root'; }
btrfs() {
    case "$1:$2" in
        inspect-internal:rootid) if [ "$3" = / ]; then echo 1; else cat "$3/id"; fi ;;
        subvolume:get-default) echo 'ID 2 gen 1 top level 5 path root' ;;
        subvolume:show) [ -f "$3/id" ] ;;
        subvolume:delete) rm -r -- "$3" ;;
        *) return 99 ;;
    esac
}
prune_replaced_roots
'''], text=True, capture_output=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual([p.exists() for p in paths], [True, True, False, False, True, True])
            self.assertFalse(legacy.exists())
            self.assertFalse((archive / legacy.name).exists())
            self.assertTrue(unrelated.exists())

    def test_03_unknown_mount_state_stops_cleanup(self):
        result = GuardTests().shell('findmnt() { return 1; }; btrfs() { echo UNEXPECTED; }; prune_replaced_roots')
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn('UNEXPECTED', result.stdout)


class DriverTests(unittest.TestCase):
    def test_05_both_nvidia_packages_require_module(self):
        for package in ('akmod-nvidia', 'akmod-nvidia-open'):
            for present in (True, False):
                with self.subTest(package=package, present=present):
                    code = '''
additional_driver_checks() { return 0; }
newest_kernel() { echo test-kernel; }
rpm() { [ "$2" = ''' + shlex.quote(package) + ''' ]; }
modinfo() { return ''' + ('0' if present else '1') + '''; }
cmd_reboot_safe
'''
                    result = GuardTests().shell(code)
                    self.assertEqual(result.returncode, 0 if present else 1, result.stderr)

    def test_05_no_nvidia_and_unknown_kernel_are_distinct(self):
        no_driver = GuardTests().shell('additional_driver_checks() { :; }; rpm() { return 1; }; cmd_reboot_safe')
        self.assertEqual(no_driver.returncode, 0)
        unknown = GuardTests().shell('additional_driver_checks() { :; }; rpm() { return 0; }; newest_kernel() { return 1; }; cmd_reboot_safe')
        self.assertEqual(unknown.returncode, 1)

    def test_05_local_checks_cannot_be_bypassed(self):
        result = GuardTests().shell('additional_driver_checks() { return 1; }; rpm() { return 1; }; cmd_reboot_safe')
        self.assertEqual(result.returncode, 1)


class SnapshotTests(unittest.TestCase):
    def test_06_same_second_snapshots_remain_distinct_and_discoverable(self):
        with tempfile.TemporaryDirectory(prefix='lnp-snapshot-') as directory:
            top = Path(directory)
            (top / 'root').mkdir()
            (top / 'snapshots').mkdir()
            script = top / 'guard'
            script.write_text(local_source('guard/lnp-guard').replace('/run/lnp/top', str(top)))
            result = subprocess.run(['bash', '-c', 'source ' + shlex.quote(str(script)) + r'''
mount_top() { :; }
umount_top() { :; }
findmnt() { echo btrfs; }
date() { echo 20260928-010203; }
btrfs() { if [ "$2" = rootid ]; then echo 256; else [ "$2" = snapshot ] && mkdir "$5"; fi; }
prune_snapshots() { :; }
prune_replaced_roots() { :; }
cmd_snapshot pre-update
cmd_snapshot pre-update
'''], text=True, capture_output=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr)
            names = sorted(p.name for p in (top / 'snapshots').glob('*-pre-update'))
            self.assertEqual(names, ['20260928-010203-000000-pre-update', '20260928-010203-000001-pre-update'])

    def test_06_snapshot_tag_cannot_escape_directory(self):
        result = GuardTests().shell('mount_top() { echo UNEXPECTED; }; cmd_snapshot ../bad')
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn('UNEXPECTED', result.stdout)


class LockTests(unittest.TestCase):
    def test_09_update_holds_same_lock_as_restore_through_package_work(self):
        with tempfile.TemporaryDirectory(prefix='lnp-lock-') as directory:
            base = Path(directory)
            common = base / 'common'
            common.write_text((ROOT / 'guard/lnp-btrfs-common').read_text().replace('/run/lnp', str(base)))
            scripts = []
            for name in ('guard/lnp-guard', 'recovery/lnp-restore'):
                path = base / Path(name).name
                path.write_text((ROOT / name).read_text().replace('/usr/libexec/lnp-btrfs-common', str(common)))
                scripts.append(path)
            update = subprocess.Popen(['bash', '-c', 'source ' + shlex.quote(str(scripts[0])) + r'''
cmd_snapshot() { return 0; }
dnf() { echo READY; read -r answer; }
systemctl() { :; }
# Avoid tail buffering in this mock so the parent can synchronize with dnf.
tail() { cat; }
cmd_update
'''], text=True, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            try:
                while True:
                    line = update.stdout.readline()
                    self.assertTrue(line, 'update exited before reaching package work')
                    if line.strip() == 'READY':
                        break
                command = ['bash', '-c', 'source ' + shlex.quote(str(scripts[1])) + '\nlock_top']
                blocked = subprocess.run(command, text=True, capture_output=True, timeout=5)
                self.assertNotEqual(blocked.returncode, 0)
                self.assertIn('another LNP', blocked.stderr)
                update.communicate('continue\n', timeout=5)
                self.assertEqual(update.returncode, 0)
                accepted = subprocess.run(command, text=True, capture_output=True, timeout=5)
                self.assertEqual(accepted.returncode, 0, accepted.stderr)
            finally:
                if update.poll() is None:
                    update.kill()
                    update.communicate()

    def test_09_preexisting_mount_is_never_unmounted(self):
        common = (ROOT / 'guard/lnp-btrfs-common').read_text()
        result = subprocess.run(['bash', '-c', common + '\nTOP=/unused; umount() { echo UNEXPECTED; }; umount_top'], text=True, capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 0)
        self.assertNotIn('UNEXPECTED', result.stdout)

    def test_09_only_owned_mount_is_unmounted_once(self):
        common = (ROOT / 'guard/lnp-btrfs-common').read_text()
        result = subprocess.run(['bash', '-c', common + '\nTOP=/unused; TOP_MOUNTED=1; umount() { echo unmounted; }; umount_top; umount_top'], text=True, capture_output=True, timeout=5)
        self.assertEqual(result.stdout, 'unmounted\n')
