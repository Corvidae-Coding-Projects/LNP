# Agent Guide: System Recovery

- Recovery is root-level and potentially destructive. Resolve the current root
  subvolume, snapshot target, and mount state before any mutation.
- Never change the separate home subvolume. Restore and cleanup paths must stay
  inside the documented LNP Btrfs locations.
- Keep the console copy calm and usable without a graphical environment.
- The advanced shell path must require a real login; authentication failure
  must never fall through to an unauthenticated root shell.
- Keep service/drop-in paths synchronized with `../specs/lnp.spec` and snapshot
  assumptions synchronized with `../guard/`.
- Run shell syntax checks and the recovery tests in
  `../tests/test_regressions.py`; use disposable Btrfs state for deeper tests.
