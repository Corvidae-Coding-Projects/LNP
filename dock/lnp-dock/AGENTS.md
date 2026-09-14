# Agent Guide: `lnp-dock`

- Keep Wayland protocol work on the event-loop thread. Follow the required
  configure and commit order for every surface.
- Do not replace compositor behavior with X11 assumptions or silently fall back
  to a different window system.
- Keep unresolved pins and stable ordering. Save hand-editable state atomically.
- Keep rendering geometry, hit testing, magnification, and input regions in
  agreement when changing sizes or animations.
- Add focused unit tests for pure state/geometry and boundary tests for startup.
  Do not require a user's live compositor for the normal test suite.
- Run `cargo test --locked`. Use `cargo fmt` only on Rust files intentionally
  included in the change; avoid unrelated repository-wide formatting churn.
