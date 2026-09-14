# First-Run Welcome

This folder contains the app shown on first use and the administrator helper it
calls when needed.

## Files

- `lnp-welcome` is the unprivileged PySide6 wizard.
- `lnp-setup` performs a fixed set of administrator actions chosen in the app.
- `org.lnp.setup.policy` defines the polkit authorization prompt.
- `lnp-welcome.desktop` exposes the wizard in the application menu.
- `lnp-welcome-autostart.desktop` launches it for first-run handling.

The app shows progress and tells the difference between success, failure, a
cancelled password prompt, and a crashed helper. Packaging is in
`../specs/lnp.spec`. Tests are in `../tests/test_regressions.py`.
