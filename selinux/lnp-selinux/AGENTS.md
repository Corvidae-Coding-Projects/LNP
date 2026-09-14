# Agent Guide: `lnp-selinux`

- Parse external suggestions into typed `Fix` variants; never execute raw
  setroubleshoot text.
- Maintain the risk ordering and make confirmation copy describe scope without
  implying that a generated rule affects only one path or process.
- Do not mix up D-Bus errors, a cancelled password prompt, helper errors, and
  success. Each needs its own state and message.
- Watch mode must seed existing alerts as history and notify only on genuinely
  new records.
- Add unit tests for parsers and selection rules. Run `cargo test --locked` and
  the mocked Python security-boundary tests in `../../tests/`.
- Coordinate any helper arguments or result semantics with the parent folder's
  scripts and documentation.
