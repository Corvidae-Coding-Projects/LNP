# Linux for Normal People (LNP)

An opinionated desktop configuration for Fedora KDE Plasma. The north star is
that someone who has never used Linux should be able to sit down at it and not
need a forum post.

## What it is

LNP is mostly **data, not code**. Plasma already supports everything needed to
reshape the desktop; it just ships defaults aimed at people who enjoy
configuring things. LNP supplies a different set of defaults through Plasma's
own supported mechanisms, so there is no forked code to keep in sync.

| Package | Contents |
| --- | --- |
| `lnp-desktop` | Metapackage. This is the one to install. |
| `lnp-look-and-feel` | Plasma look-and-feel package `org.lnp.desktop`: the panel layout, theme, colours, cursors. |
| `lnp-defaults` | System-wide defaults in `/etc/xdg` for KWin (including window button order), Dolphin, fonts and the screen locker. |
| `lnp-apply` | The applier and its systemd user service. |
| `lnp-welcome` | First-run wizard: one-click codecs and drivers via polkit, dock toggle, GUI restore of the old desktop, short tour. |
| `lnp-errord` | Journal watcher that translates SELinux denials, service crashes and OOM kills into plain-language notifications. |
| `lnp-dock` | The dock: Rust, Wayland layer-shell, fixed-centre magnification, live window list. Own spec (arch-specific). |

## The layout

A slim menu bar across the top (global application menu on the left, status
items and clock on the right) and a floating, centred icon dock at the bottom
that steps out of the way of windows rather than sitting permanently on top.
Window controls sit on the left, macOS-style.

Everything is built from stock Plasma applets: `appmenu`, `panelspacer`,
`systemtray`, `digitalclock`, `kickoff`, `icontasks`, `trash`.

## How defaults are delivered

Two mechanisms, both native to Plasma:

1. **`/etc/xdg/*rc`** sits *below* `~/.config/*rc` in the `XDG_CONFIG_DIRS`
   cascade. Values apply to users who have not touched that setting, and are
   silently overridden the moment they do. Nothing is enforced.

2. **The look-and-feel package** owns theme, colour scheme, icons, cursors and
   the panel layout, so there is exactly one owner for those.

   One caveat, found by testing rather than documentation:
   `plasma-apply-lookandfeel` **ignores the `[kwinrc]` sections** of a
   look-and-feel `defaults` file. Applying `org.lnp.desktop` rewrote the colour
   blocks in `kdeglobals` but left `kwinrc` untouched. Anything KWin-related --
   window button order included -- therefore has to go through `/etc/xdg`.

Touchpad settings are the exception: Plasma keys them per physical device
under `[Libinput][vendor][product][name]`, so no static file can express them.
The applier enumerates the machine's actual touchpads instead.

## Applying and reverting

Installing `lnp-apply` enables a systemd **user** service. It runs at the next
Plasma login, compares the shipped layout version against a per-user stamp,
and applies the layout once. It is a no-op on every subsequent login.

This runs as the user rather than from RPM `%post` on purpose: Plasma's
per-user configuration cannot be safely rewritten by root while a session is
live.

**Applying rebuilds the Plasma panel layout.** The previous configuration is
always backed up first, to `~/.local/state/lnp/backup-<timestamp>/`.

```console
$ lnp-apply-layout --status    # what is shipped, what is applied, what backups exist
$ lnp-apply-layout --force     # re-apply even if the stamp says it is current
$ lnp-apply-layout --revert    # restore the most recent backup
```

Restoring pauses automatic layout application, including at later logins and
layout updates. Use `--force`, or “Apply the LNP desktop layout” in Welcome,
to apply LNP again and resume automatic application.

New backups record both saved files and files that were absent, so reverting
also removes configuration files created by the applier. Older backups lack
that record: their saved files can be restored, but files with no saved copy
are left in place. Failed backups stop application and never replace the latest
completed backup.

## Product and safety design

The authoritative design package is
[`docs/product-design.md`](docs/product-design.md). It defines the product
doctrine, authorization threat model and architecture, core-journey research
matrix, accessibility audit, release gates, and delivery sequence.

[`docs/consent-design.html`](docs/consent-design.html) is an earlier exploration
kept as design history. Its proposed consent protocol is superseded by the
security and accessibility requirements in the authoritative specification.

## Status

Early. The data packages are the working part. Still to come: a magnifying
dock plasmoid (Plasma has no dock magnification and Latte Dock is dead), a
first-run welcome wizard, one-click codec and driver setup, and a
human-readable error translator.
