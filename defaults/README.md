# Desktop Defaults

This folder contains LNP's system-wide KDE and Plasma defaults. RPM packaging
installs the files below `/etc/xdg`, where they provide initial values without
overriding settings a user has already customized.

## Contents

- `etc/xdg/kdeglobals` supplies shared KDE appearance and behavior defaults.
- `etc/xdg/kwinrc` supplies KWin defaults, including window behavior.
- `etc/xdg/dolphinrc` supplies file-manager defaults.
- `etc/xdg/kscreenlockerrc` supplies screen-locker defaults.

The look-and-feel package owns theme and panel layout; these files own settings
that Plasma does not reliably apply from look-and-feel metadata. Installation
paths are defined in `../specs/lnp.spec`.
