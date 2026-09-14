# Update and Reboot Guard

This folder owns the root-side safety mechanisms around system updates.
`lnp-guard` can create and prune read-only Btrfs snapshots, verify that the
newest installed kernel has the graphics module needed for a safe reboot, hold
a shutdown inhibitor while it is unsafe, and run guarded updates.

## Files

- `lnp-guard` implements snapshot, reboot-safety, inhibitor, and update modes.
- `lnp-guard-update.service` and `.timer` schedule guarded updates.
- `lnp-reboot-guard.path` notices kernel changes.
- `lnp-reboot-guard.service` holds the inhibitor until restart is safe.

Restoration is implemented by `../recovery/lnp-restore`. RPM integration is in
`../specs/lnp.spec`.
