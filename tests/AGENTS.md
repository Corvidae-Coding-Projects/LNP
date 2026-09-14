# Agent Guide: Cross-Component Tests

- Tests must stand on their own. Replace administrator commands with fakes and
  write only to temporary XDG or build folders.
- Never let a test invoke live `pkexec`, `semodule`, `audit2allow`, package
  managers, login, systemd mutation, Btrfs restoration, or desktop reconfiguration.
- Assert both the requested outcome and forbidden side effects, especially on
  failure paths and authorization cancellation.
- Keep test data small and clear. SELinux tests must compare the exact denial
  records; do not weaken a check just to make a test pass.
- Add tests near the narrow unit when possible; reserve this folder for
  cross-language, cross-process, and security-boundary behavior.
- Run the complete discovery command after modifying shared fixtures or helpers.
