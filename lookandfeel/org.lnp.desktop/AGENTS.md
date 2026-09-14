# Agent Guide: `org.lnp.desktop`

- Do not change the package ID without updating every consumer and providing an
  explicit migration path.
- Keep metadata valid JSON and use only Plasma-supported keys in `contents/`.
- Layout changes must remain deterministic when run in a clean profile and must
  not assume a fixed display count or resolution.
- Coordinate behavioral layout changes with `../../apply/`, the layout version
  in `../../specs/lnp.spec`, and regression documentation.
- Validate in a disposable Plasma user profile; never apply experimental layout
  code to the active user's session as an automated check.
