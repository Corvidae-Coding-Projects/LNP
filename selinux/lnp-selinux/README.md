# `lnp-selinux` Rust Crate

`lnp-selinux` is an `egui`/`eframe` application that reads alerts from
setroubleshoot over D-Bus, translates supported suggestions into structured
fixes, presents their risk, and invokes the separately packaged privileged
helper only after confirmation. A watch mode can notify users of new alerts.

## Source map

- `src/main.rs` owns GUI state, watch mode, helper invocation, and result copy.
- `src/alerts.rs` owns the setroubleshoot D-Bus model and alert selection.
- `src/fixes.rs` parses suggestions into a closed set of validated actions.

## Build and test

```sh
cargo build --locked
cargo test --locked
```

The companion helper, policy generator, service, desktop file, and polkit
policy live one directory above.
