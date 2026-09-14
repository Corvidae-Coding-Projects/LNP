# Agent Guide: Error Daemon

- Notify only when there is a clear story the user can understand or act on.
  Avoid surfacing routine transient-unit churn.
- Never claim a repair occurred; this daemon is read-only and unprivileged.
- Preserve rate limits and deduplication so repeated system failures do not
  train users to dismiss notifications.
- Keep copy calm, specific, non-blaming, and free of raw identifiers when a
  sentence conveys the same information.
- Service startup must tolerate session-environment timing without silently
  disappearing for the entire login.
- Update `../specs/lnp.spec` for dependency or installed-file changes and add
  mocked tests for new parsers or classification rules.
