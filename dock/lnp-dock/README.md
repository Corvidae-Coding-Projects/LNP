# `lnp-dock` Rust Crate

`lnp-dock` is a native Wayland layer-shell dock for KDE Plasma. It renders its
own surface, discovers applications and windows, supports persistent pins and
drag reordering, provides application and context menus, and implements
pointer-responsive magnification.

## Source map

- `src/main.rs` owns Wayland setup, event dispatch, surfaces, input, and layout.
- `apps.rs` and `appsmenu.rs` discover and present installed applications.
- `launchers.rs` parses desktop entries and launches resolved applications.
- `pins.rs` owns persistent pin parsing and reorder behavior.
- `windows.rs` tracks Plasma windows.
- `magnify.rs`, `render.rs`, and `text.rs` own geometry and drawing.
- `menu.rs` and `tooltip.rs` own transient UI models.
- `tests/startup.rs` checks startup behavior at the compositor boundary.

## Build and test

```sh
cargo build --locked
cargo test --locked
```

Runtime integration requires a Wayland compositor with layer-shell and KDE
Plasma window-management protocols.
