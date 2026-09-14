# Agent Guide: First-Run Welcome

- Keep the GUI unprivileged. Root work must go through the fixed subcommands of
  `lnp-setup` and the exact executable path in the polkit action.
- Validate every helper argument again after privilege elevation. Do not accept
  arbitrary packages, repositories, commands, or paths from the GUI.
- User-facing copy must say what will change, show useful progress, and keep
  cancellation, partial failure, crash, and success distinct.
- Do not claim rollback when earlier helper steps may already have changed the
  system.
- Preserve first-run idempotence and the explicit path for reopening the wizard.
- Run `python3 -m py_compile` on both Python files and the welcome regression
  tests with Qt's offscreen backend. Update `../specs/lnp.spec` for dependencies
  or installed-file changes.
