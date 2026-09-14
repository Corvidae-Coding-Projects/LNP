# Agent Guide: `lnp-selinux`

- Parse external suggestions into typed `Fix` variants; never execute raw
  setroubleshoot text.
- Maintain the risk ordering and make confirmation copy describe scope without
  implying that a generated rule affects only one path or process.
- Keep D-Bus failures, authorization cancellation, helper failure, and success
  distinct in both state and user-facing messages.
- Watch mode must seed existing alerts as history and notify only on genuinely
  new records.
- Add unit tests for parsers and selection rules. Run `cargo test --locked` and
  the mocked Python security-boundary tests in `../../tests/`.
- Coordinate any helper arguments or result semantics with the parent folder's
  scripts and documentation.
