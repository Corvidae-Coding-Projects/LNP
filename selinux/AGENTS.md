# Agent Guide: LNP Security Alerts

- Preserve two independent checks across the GUI/helper boundary. Caller text
  is an expected value, never authoritative compiler input.
- The root helper has a closed action vocabulary. Do not add shell evaluation,
  command strings, or permissive pass-through arguments.
- Bind custom-policy generation to one current alert, timestamp, and
  unambiguous denial record; recheck after compilation before installation.
- Keep user-facing risk labels honest and require explicit authorization for
  permanent policy changes.
- Update `../docs/selinux-policy-generation.md`, tests, polkit policy, and
  `../specs/lnp-selinux.spec` when the boundary changes.
- Follow `lnp-selinux/AGENTS.md` for Rust UI changes. Never install a test policy
  module on the host; the Python suite mocks compiler and installer calls.
