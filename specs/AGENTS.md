# Agent Guide: Fedora Packaging

- Treat specs as executable installation manifests: keep source paths,
  dependencies, file modes, units, presets, desktop files, and `%files` lists
  synchronized with the repository.
- Keep runtime dependencies separate from build dependencies and assign files
  to the narrowest correct subpackage.
- Use Fedora RPM macros for systemd lifecycle integration and preserve the
  distinction between system and user units.
- Do not commit generated RPMs, SRPMs, source archives, vendored crates, or
  build roots.
- When changing versions or changelog entries, verify all cross-package version
  constraints and the source-archive layout used by `%prep`.
- Build in a disposable RPM environment when validating packaging; do not
  install test packages over the active system without explicit authorization.
