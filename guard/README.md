# Update and Reboot Guard

`lnp-guard` makes system updates easier to undo. It takes read-only Btrfs
snapshots, removes old LNP snapshots, and checks that the newest kernel has the
graphics driver it needs before allowing a restart. It can also run scheduled
updates.

## Files

- `lnp-guard` takes snapshots, checks restarts, and runs updates.
- `lnp-guard-update.service` and `.timer` schedule guarded updates.
- `lnp-reboot-guard.path` notices kernel changes.
- `lnp-reboot-guard.service` blocks restart until the check passes.

Restoration is implemented by `../recovery/lnp-restore`. RPM integration is in
`../specs/lnp.spec`.
