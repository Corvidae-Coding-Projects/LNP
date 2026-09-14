# First-Run Welcome

This folder contains the LNP first-run wizard and its explicitly authorized
setup helper.

## Files

- `lnp-welcome` is the unprivileged PySide6 wizard.
- `lnp-setup` performs the narrow root actions selected in the wizard.
- `org.lnp.setup.policy` defines the polkit authorization prompt.
- `lnp-welcome.desktop` exposes the wizard in the application menu.
- `lnp-welcome-autostart.desktop` launches it for first-run handling.

The GUI streams helper progress and distinguishes success, failure,
authorization cancellation, and a crashed helper. Packaging is in
`../specs/lnp.spec`; regression coverage is in `../tests/test_regressions.py`.
