# Linux for Normal People (LNP)

LNP is a Fedora KDE Plasma desktop for people who do not want to become Linux
experts just to use their computer.

## The goal

The end goal is an easy-to-use, secure Linux desktop backed up by software
agents. Those agents should help maintain the system, test updates, explain
problems, and guide recovery. They must follow the same safety rules as the
rest of LNP: no hidden changes, no automatic administrator access, and no
pretending that a failed task succeeded.

## What it is

Much of LNP is configuration rather than replacement desktop code. It uses
Plasma's supported settings and package formats, then adds small tools where
configuration alone is not enough.

| Package | Contents |
| --- | --- |
| `lnp-desktop` | Main package. Install this to get the full LNP desktop. |
| `lnp-look-and-feel` | Panel layout, theme, colours, and cursors. |
| `lnp-defaults` | Default settings for KWin, Dolphin, fonts, and the screen locker. |
| `lnp-apply` | Applies the layout, backs up the old one, and can restore it. |
| `lnp-welcome` | First-run setup for media support, drivers, the dock, and restoring the previous desktop. |
| `lnp-errord` | Turns selected system errors into useful desktop messages. |
| `lnp-guard` | Takes a Btrfs snapshot before updates and checks whether it is safe to restart. |
| `lnp-recovery` | Offers a simple recovery screen if the graphical login fails. |
| `lnp-dock` | Native Wayland dock with magnification, pinned apps, and a live window list. |
| `lnp-selinux` | Explains SELinux alerts and offers a small set of reviewed fixes. |

## Repository guide

Each folder has a short README that explains what it is for and how to check
changes made there.

| Folder | Responsibility |
| --- | --- |
| [`apply/`](apply/) | Applies, backs up, checks, and restores the desktop layout. |
| [`defaults/`](defaults/) | Default KDE and Plasma settings. |
| [`dock/`](dock/) | Dock code and session setup. |
| [`docs/`](docs/) | Product, safety, and security design notes. |
| [`errord/`](errord/) | Desktop messages for selected system failures. |
| [`guard/`](guard/) | Safe-update snapshots and restart checks. |
| [`hdn-linux/`](hdn-linux/) | HDN kernel-hardening patch, design notes, and test records. |
| [`lookandfeel/`](lookandfeel/) | Plasma theme and panel layout. |
| [`recovery/`](recovery/) | Snapshot restore tool and text-mode recovery screen. |
| [`selinux/`](selinux/) | SELinux alert app and carefully limited repair tools. |
| [`specs/`](specs/) | Fedora RPM package files. |
| [`tests/`](tests/) | Tests that cover more than one part of LNP. |
| [`welcome/`](welcome/) | First-run app and its administrator helper. |

## The layout

LNP uses a slim menu bar at the top and a centred icon dock at the bottom. The
dock moves out of the way of windows instead of always covering part of the
screen. Window controls appear on the left.

Everything is built from stock Plasma applets: `appmenu`, `panelspacer`,
`systemtray`, `digitalclock`, `kickoff`, `icontasks`, `trash`.

## How defaults are delivered

LNP uses two standard Plasma features:

1. Files in **`/etc/xdg`** provide starting values. A person's own settings in
   `~/.config` take priority, so LNP does not lock these choices.

2. The **look-and-feel package** provides the theme, colours, icons, cursors,
   and panel layout.

Testing found that `plasma-apply-lookandfeel` ignores `[kwinrc]` sections in a
look-and-feel defaults file. KWin settings, including window button order,
therefore come from `/etc/xdg`.

Touchpad settings are tied to each physical device. The layout tool finds the
computer's touchpads and writes the right settings for them.

## Applying and reverting

Installing `lnp-apply` enables a service for each user. At the next Plasma
login, it checks the layout version and applies a new layout once. Later logins
do nothing unless the shipped layout changes.

It runs as the signed-in user because changing live Plasma settings as root is
unsafe.

Applying LNP rebuilds the Plasma panel layout. It first saves the old settings
under `~/.local/state/lnp/backup-<timestamp>/`.

```console
$ lnp-apply-layout --status    # what is shipped, what is applied, what backups exist
$ lnp-apply-layout --force     # re-apply even if the stamp says it is current
$ lnp-apply-layout --revert    # restore the most recent backup
```

Restoring also pauses automatic layout changes. Use `--force`, or choose
“Apply the LNP desktop layout” in Welcome, to turn them back on.

New backups remember which files did not exist before LNP, so a restore can
remove files that LNP created. Older backups only restore saved files. If a
backup fails, LNP stops before applying the layout and keeps the last good
backup.

## Product and safety design

[`docs/product-design.md`](docs/product-design.md) contains the current product
and safety plan. It covers security, accessibility, user testing, architecture,
and release checks.

[`docs/consent-design.html`](docs/consent-design.html) is an older idea kept for
reference. If it disagrees with `product-design.md`, use `product-design.md`.

## HDN Linux

[`hdn-linux/`](hdn-linux/) is LNP's kernel-security project. This repository
includes a patch for upstream Linux `7.0.12`, the design behind it, a comparison
with other hardening work, test results, and a script that checks the published
patch. HDN Linux is not currently one of the LNP desktop RPM packages.

Start with the [HDN Linux README](hdn-linux/README.md). The actual kernel source
and release process are explained in
[`hdn-linux/docs/SOURCE_CONTROL.md`](hdn-linux/docs/SOURCE_CONTROL.md). The patch
in this repository is generated and checked from that source.

## Status

LNP is under active development. The repository includes working versions of
the layout, dock, welcome app, error messages, update guard, recovery tools,
and SELinux alert app. It is not ready to promise as a finished desktop until
the release, accessibility, security, and full-system tests in the product plan
have passed.
