# Agent Guide: `lnp-dock`

- Keep Wayland protocol work on the event-loop thread and preserve correct
  configure/commit sequencing for every surface.
- Do not replace compositor behavior with X11 assumptions or silently fall back
  to a different window system.
- Preserve unresolved pins, stable ordering, and atomic hand-editable state.
- Keep rendering geometry, hit testing, magnification, and input regions in
  agreement when changing sizes or animations.
- Add focused unit tests for pure state/geometry and boundary tests for startup.
  Do not require a user's live compositor for the normal test suite.
- Run `cargo test --locked`. Use `cargo fmt` only on Rust files intentionally
  included in the change; avoid unrelated repository-wide formatting churn.
