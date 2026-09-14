# Plasma Look and Feel

This folder contains the `org.lnp.desktop` Plasma look-and-feel package. It
defines the LNP visual identity and the initial panel layout installed by the
`lnp-look-and-feel` RPM subpackage.

The package intentionally does not own every KDE setting. Values that Plasma
does not reliably apply from look-and-feel metadata live under `../defaults/`,
while per-device touchpad configuration is handled by `../apply/`.

See `org.lnp.desktop/README.md` for the package layout. Installation and file
ownership are defined in `../specs/lnp.spec`.
