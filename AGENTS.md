# Agent Guide

## Mission

LNP should be easy to understand and recover without Linux training. Use plain
language. Make changes reversible. Ask before administrator actions. Report
failures honestly.

## Repository rules

- Read the local `README.md` and nearest `AGENTS.md` before changing a folder.
- Keep normal apps separate from administrator helpers. Never widen a polkit
  rule or pass untrusted text through a shell.
- Do not test against a live desktop, package database, SELinux policy, or
  Btrfs root when a mocked or disposable environment can establish the result.
- Keep service units, presets, desktop files, and the matching RPM spec in sync.
- Keep state files easy to edit by hand and do not break documented restore
  paths.
- Follow `docs/product-design.md` for LNP product and safety decisions. Follow
  `hdn-linux/docs/SOURCE_CONTROL.md` for the source of the HDN release patch.

## Baseline validation

Run the checks relevant to the touched area:

```sh
python3 -m unittest discover -s tests -v
(cd dock/lnp-dock && cargo test --locked)
(cd selinux/lnp-selinux && cargo test --locked)
```

For shell changes, run `bash -n` on each changed shell script. Keep generated
build output, vendored dependencies, caches, credentials, and local environment
files out of Git.
