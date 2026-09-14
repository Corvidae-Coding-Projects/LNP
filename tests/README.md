# Cross-Component Tests

This folder contains Python `unittest` coverage for behavior that crosses
scripts, privilege boundaries, or project folders.

- `test_regressions.py` covers layout backup/revert safety, recovery
  authentication, welcome result reporting, and helper entrypoint constraints.
- `test_selinux_policy.py` covers selection and revalidation of authoritative
  SELinux denial records, compiler/install failures, cleanup, and direct
  unprivileged invocation.

Run the suite from the repository root:

```sh
python3 -m unittest discover -s tests -v
```

System commands are mocked and configuration writes use temporary directories;
the suite must not make live desktop, package, policy, or login changes.
