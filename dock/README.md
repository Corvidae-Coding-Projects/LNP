# LNP Dock

This folder combines the dock implementation with the files that integrate it
into a Fedora KDE Plasma session.

## Layout

- `lnp-dock/` is the Rust Wayland client.
- `lnp-dock.service` starts it as a systemd user service.
- `81-lnp-dock.preset` enables that service by default.
- `lnp-dock.desktop` supplies the desktop application identity used for launch
  metadata and packaging.

The standalone RPM definition is `../specs/lnp-dock.spec`.

## Validation

```sh
(cd dock/lnp-dock && cargo test --locked)
```

Changes to service, preset, or desktop integration should also be checked
against the paths and dependencies in the RPM spec.
