# Agent Guide

## Mission

LNP makes Fedora KDE Plasma understandable and recoverable for people who do
not administer Linux. Preserve plain language, reversible behavior, explicit
authorization, and safe failure modes.

## Repository rules

- Read the local `README.md` and nearest `AGENTS.md` before changing a folder.
- Keep unprivileged UI/session code separate from root helpers. Never broaden a
  polkit action or pass untrusted text through a shell.
- Do not test against a live desktop, package database, SELinux policy, or
  Btrfs root when a mocked or disposable environment can establish the result.
- Keep service units, presets, desktop files, and the matching RPM spec in sync.
- Preserve hand-editable state formats and the documented backup/revert paths.
- Treat `docs/product-design.md` as the authoritative LNP product and safety
  specification. Treat `hdn-linux/docs/SOURCE_CONTROL.md` as authoritative for
  the provenance of the HDN release patch.

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
