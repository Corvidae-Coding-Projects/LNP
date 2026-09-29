# Cross-Component Tests

These Python tests check behavior that crosses scripts or security boundaries.

- `test_regressions.py` covers layout backup/revert safety, recovery
  authentication, welcome result reporting, and helper entrypoint constraints.
- `test_selinux_policy.py` checks that SELinux records are loaded again and
  matched exactly before use. It also checks failures, cleanup, and attempts to
  run the administrator tool without permission.

Run the suite from the repository root:

```sh
python3 -m unittest discover -s tests -v
```

The tests use fake system commands and temporary folders. They must not change
the real desktop, installed packages, SELinux policy, or login service.

Issue #1 regression coverage lives in `test_guard_recovery.py` and
`test_errord.py`: boot compatibility, restore rollback, saved-root retention,
restart-hold failure, journal exit, both NVIDIA packages, snapshot collisions,
notification floods/markup, and cross-process operation locking.

For the real Btrfs integration check, explicitly run:

```sh
sudo tests/check-btrfs-recovery
```

It creates a disposable 512 MiB sparse loop image in a private mount namespace
and removes it afterward. Bootloader calls use fixtures; root renames,
snapshots, mounted-root preservation, and pruning use real Btrfs operations.
It does not restore, update, or prune the host filesystem.

`test_setup.py` covers issue #2 with fake cleanup, package, service, and security
commands plus temporary DNS configuration. It passes actual helper results to
the offscreen Welcome UI and checks failures, mixed results, retry, success,
skipped optional components, configuration-write errors, and split output.
