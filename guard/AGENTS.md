# Agent Guide: Update and Reboot Guard

- Assume every command here runs as root. Validate exact devices, subvolumes,
  paths, and snapshot names before a mutating operation.
- Preserve the user's home subvolume and make cleanup operate only on snapshots
  created under the documented LNP snapshot directory.
- A failed safety check must prevent a reboot claim, not guess that the system
  is ready. Keep inhibitors bounded to the actual unsafe condition.
- Maintain compatibility with `../recovery/lnp-restore` and keep unit/preset
  changes synchronized with `../specs/lnp.spec`.
- Run `bash -n guard/lnp-guard`. Use disposable Btrfs filesystems and mocked
  package/kernel state for behavioral tests; never test mutations on the active
  root filesystem.
