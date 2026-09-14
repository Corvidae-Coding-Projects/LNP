# Agent Guide: Plasma Look and Feel

- Keep ownership boundaries clear: theme and panel layout belong here; XDG
  application/window defaults belong in `../defaults/`; per-user migration and
  device discovery belong in `../apply/`.
- Preserve the package ID `org.lnp.desktop` across metadata, commands, specs,
  and the layout applier.
- Treat panel-layout JavaScript as migration-sensitive user-facing code. Test it
  with a disposable Plasma profile and keep the applier's layout version in
  sync when shipped layout behavior changes.
- Update `../specs/lnp.spec` when package contents or paths change.
- Follow `org.lnp.desktop/AGENTS.md` inside the package root.
