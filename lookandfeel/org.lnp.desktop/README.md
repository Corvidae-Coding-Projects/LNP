# `org.lnp.desktop`

This is the installable Plasma Look and Feel package for LNP.

- `metadata.json` identifies the package to Plasma.
- `contents/defaults` selects the appearance defaults Plasma supports through
  the Look and Feel API.
- `contents/layouts/org.kde.plasma.desktop-layout.js` creates the shipped Plasma
  panel layout.

The layout is applied by `../../apply/lnp-apply-layout`, which takes a backup
before invoking Plasma and records the installed layout version.
