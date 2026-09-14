# Agent Guide: Desktop Defaults

- These files are defaults, not enforced policy. Preserve the XDG precedence
  model in which a user's own configuration wins.
- Avoid assigning the same setting here and in `../lookandfeel/` unless the
  duplication is documented and required by Plasma behavior.
- Keep section and key names compatible with the Fedora KDE Plasma version the
  package targets; do not infer keys from another desktop environment.
- When adding or removing a file, update the install and `%files` sections of
  `../specs/lnp.spec`.
- Validate changes in a disposable user profile or package test environment,
  never by overwriting the active user's KDE configuration.
