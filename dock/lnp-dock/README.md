# `lnp-dock` Rust Crate

`lnp-dock` is a Wayland dock for KDE Plasma. It finds installed apps and open
windows, remembers pinned apps, supports drag-and-drop ordering, and enlarges
icons near the pointer.

## Source map

- `src/main.rs` handles Wayland setup, events, surfaces, input, and layout.
- `apps.rs` and `appsmenu.rs` discover and present installed applications.
- `launchers.rs` parses desktop entries and launches resolved applications.
- `pins.rs` reads, saves, and reorders pinned apps.
- `windows.rs` tracks Plasma windows.
- `magnify.rs`, `render.rs`, and `text.rs` handle sizing and drawing.
- `menu.rs` and `tooltip.rs` handle temporary popups.
- `tests/startup.rs` checks startup when no compositor is available.

## Build and test

```sh
cargo build --locked
cargo test --locked
```

Running the dock requires a Wayland compositor that supports layer shell and
KDE Plasma's window-management protocol.
