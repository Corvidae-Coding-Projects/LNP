# Agent Guide: Fedora Packaging

- RPM specs control what gets installed. Keep source paths, dependencies, file
  permissions, services, presets, desktop files, and `%files` lists in sync.
- Keep runtime dependencies separate from build dependencies and assign files
  to the narrowest correct subpackage.
- Use Fedora RPM macros for systemd setup. Do not mix up system services and
  services that run for one user.
- Do not commit generated RPMs, SRPMs, source archives, vendored crates, or
  build roots.
- When changing versions or changelog entries, verify all cross-package version
  constraints and the source-archive layout used by `%prep`.
- Build in a disposable RPM environment when validating packaging; do not
  install test packages over the active system without explicit authorization.
