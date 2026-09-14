# Agent Guide: Update and Reboot Guard

- Assume every command here runs as root. Validate exact devices, subvolumes,
  paths, and snapshot names before a mutating operation.
- Never change the user's home subvolume. Cleanup may remove only snapshots in
  the documented LNP snapshot directory.
- If a safety check fails, do not say the system is ready to restart. Block
  restart only while the unsafe condition still exists.
- Maintain compatibility with `../recovery/lnp-restore` and keep unit/preset
  changes synchronized with `../specs/lnp.spec`.
- Run `bash -n guard/lnp-guard`. Use disposable Btrfs filesystems and mocked
  package/kernel state for behavioral tests; never test mutations on the active
  root filesystem.
