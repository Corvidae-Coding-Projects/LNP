# LNP Dock

This folder contains the dock and the files that start it in KDE Plasma.

## Layout

- `lnp-dock/` is the Rust Wayland client.
- `lnp-dock.service` starts it as a systemd user service.
- `81-lnp-dock.preset` enables that service by default.
- `lnp-dock.desktop` gives the dock its application name and identity.

The standalone RPM definition is `../specs/lnp-dock.spec`.

## Validation

```sh
(cd dock/lnp-dock && cargo test --locked)
```

If a service, preset, or desktop file changes, check the paths and dependencies
in the RPM spec too.
