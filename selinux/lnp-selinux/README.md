# `lnp-selinux` Rust Crate

`lnp-selinux` reads alerts from setroubleshoot over D-Bus. It turns supported
suggestions into a short list of known fixes, explains the risk, and asks for
confirmation before calling the administrator helper. It can also notify the
user when a new alert appears.

## Source map

- `src/main.rs` handles the app window, watch mode, helper calls, and messages.
- `src/alerts.rs` reads and selects setroubleshoot alerts.
- `src/fixes.rs` parses suggestions into a closed set of validated actions.

## Build and test

```sh
cargo build --locked
cargo test --locked
```

The helper, policy generator, service, desktop file, and polkit rule are in the
parent folder.
