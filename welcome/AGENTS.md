# Agent Guide: First-Run Welcome

- Keep the GUI unprivileged. Root work must go through the fixed subcommands of
  `lnp-setup` and the exact executable path in the polkit action.
- Validate every helper argument again after privilege elevation. Do not accept
  arbitrary packages, repositories, commands, or paths from the GUI.
- Tell the user what will change and show useful progress. Do not mix up a
  cancelled password prompt, partial failure, crash, and success.
- Do not claim rollback when earlier helper steps may already have changed the
  system.
- Running first-use setup again must be safe. Keep a clear way to reopen it.
- Run `python3 -m py_compile` on both Python files and the welcome regression
  tests with Qt's offscreen backend. Update `../specs/lnp.spec` for dependencies
  or installed-file changes.
