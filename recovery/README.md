# System Recovery

These tools help when the graphical login will not start. They show a simple
text screen and can restore a Btrfs snapshot.

## Files

- `lnp-recovery-prompt` presents the console recovery menu.
- `lnp-recovery-prompt.service` shows that prompt on the first text console.
- `sddm-lnp-recovery.conf` connects display-manager failure to the prompt for
  both supported service names.
- `lnp-restore` lists snapshots or chooses one for the next boot. It does not
  change the separate home subvolume.

Snapshots are created by `../guard/lnp-guard`. Packaging is in
`../specs/lnp.spec`.

## Boot compatibility and undo

Restore does not roll back separate `/boot` or EFI partitions. Before changing
the root, it finds the newest remaining boot entry whose kernel image and
initramfs exist and whose modules are present in both the saved and current
systems. It requires the supported `rootflags=subvol=root` boot layout and
refuses entries pinned to a subvolume ID. If no compatible entry remains,
restore stops before changing the boot selection or root.

It prepares a writable copy first, selects and verifies the matching boot
entry, and then switches roots. If switching roots fails, it attempts to
restore the previous root and boot selection and reports any rollback failure.
This is not a guarantee of recovery from power loss during the root switch.

The replaced root remains under `snapshots/replaced-roots/` at the Btrfs top
level. `lnp-restore list` includes these saved roots; pass a listed
`root.before-restore-*` name to `lnp-restore restore` to restore one. Older
saved roots at the top level are also accepted. Guard cleanup retains the two
newest copies plus roots still in use; it never deletes a mounted, running, or
default root. Recovery and guard share a lock for their entire operations.
