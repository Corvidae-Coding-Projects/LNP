# Agent Guide: LNP Dock

- Keep the Rust client, systemd unit, preset, desktop identity, and
  `../specs/lnp-dock.spec` synchronized.
- The service may start before every graphical-session environment variable is
  imported. Rely on explicit runtime errors and restart behavior, not one-shot
  `ConditionEnvironment` checks.
- Preserve the plain-text pin format under `~/.config/lnp-dock/pins` and do not
  discard unresolved entries during reorder or migration.
- Run the crate tests after implementation changes. Compositor-dependent manual
  checks belong in a disposable graphical session.
- Follow `lnp-dock/AGENTS.md` for implementation-specific rules.
