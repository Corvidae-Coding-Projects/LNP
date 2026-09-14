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
