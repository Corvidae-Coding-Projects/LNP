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
