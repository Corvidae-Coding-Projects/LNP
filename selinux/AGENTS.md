# Agent Guide: LNP Security Alerts

- Keep two separate checks between the app and administrator helper. Text from
  the app is only an expected value; it is never trusted compiler input.
- The root helper has a closed action vocabulary. Do not add shell evaluation,
  command strings, or permissive pass-through arguments.
- Bind custom-policy generation to one current alert, timestamp, and
  unambiguous denial record; recheck after compilation before installation.
- Explain the risk clearly and require administrator approval for permanent
  policy changes.
- Update `../docs/selinux-policy-generation.md`, tests, polkit policy, and
  `../specs/lnp-selinux.spec` when the boundary changes.
- Follow `lnp-selinux/AGENTS.md` for Rust UI changes. Never install a test policy
  module on the host; the Python suite mocks compiler and installer calls.
