# Agent Guide: Layout Applier

- Preserve the invariant that no user configuration is changed until a complete
  backup has been published successfully.
- Revert must validate the backup before restoring any file. It must replace a
  hostile symlink rather than write through it.
- An explicit successful apply may resume automatic updates; failed attempts
  must not silently clear a user's opt-out.
- Keep the script usable without root privileges and inside the user's session.
- Update `../tests/test_regressions.py`, `../README.md`, and
  `../specs/lnp.spec` when behavior, installed files, or dependencies change.
- Do not exercise the applier against a real home directory during automated
  work; use temporary `XDG_CONFIG_HOME` and `XDG_STATE_HOME` locations.
