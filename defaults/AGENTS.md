# Agent Guide: Desktop Defaults

- These are starting settings, not rules. A user's own settings must win.
- Avoid assigning the same setting here and in `../lookandfeel/` unless the
  duplication is documented and required by Plasma behavior.
- Check section and key names against the supported Fedora KDE Plasma version.
  Do not copy keys from another desktop.
- When adding or removing a file, update the install and `%files` sections of
  `../specs/lnp.spec`.
- Validate changes in a disposable user profile or package test environment,
  never by overwriting the active user's KDE configuration.
