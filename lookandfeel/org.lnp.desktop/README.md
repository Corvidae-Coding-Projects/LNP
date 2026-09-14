# `org.lnp.desktop`

This is the installable Plasma Look and Feel package for LNP.

- `metadata.json` gives Plasma the package name and details.
- `contents/defaults` sets the appearance options supported by Plasma's Look
  and Feel system.
- `contents/layouts/org.kde.plasma.desktop-layout.js` creates the shipped Plasma
  panel layout.

`../../apply/lnp-apply-layout` applies the layout. It saves the old settings
first and records which layout version was installed.
