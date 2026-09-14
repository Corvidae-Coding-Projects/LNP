# Agent Guide: Cross-Component Tests

- Tests must be hermetic: replace privileged commands and write into temporary
  XDG or build directories.
- Never let a test invoke live `pkexec`, `semodule`, `audit2allow`, package
  managers, login, systemd mutation, Btrfs restoration, or desktop reconfiguration.
- Assert both the requested outcome and forbidden side effects, especially on
  failure paths and authorization cancellation.
- Keep fixtures small and explicit. Preserve exact denial-record distinctions
  in SELinux tests rather than weakening comparisons to make a test pass.
- Add tests near the narrow unit when possible; reserve this folder for
  cross-language, cross-process, and security-boundary behavior.
- Run the complete discovery command after modifying shared fixtures or helpers.
