# Agent Guide: Plasma Look and Feel

- Put the theme and panel layout here, XDG defaults in `../defaults/`, and
  per-user setup or device checks in `../apply/`.
- Keep the package ID `org.lnp.desktop` the same in metadata, commands, specs,
  and the layout tool.
- Panel-layout JavaScript changes a person's desktop. Test it with a disposable
  Plasma profile and update the layout version when shipped behavior changes.
- Update `../specs/lnp.spec` when package contents or paths change.
- Follow `org.lnp.desktop/AGENTS.md` inside the package root.
