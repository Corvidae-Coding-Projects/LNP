# System Recovery

This folder provides a plain-language recovery path when the graphical login
cannot start and a root-side tool for restoring a Btrfs snapshot.

## Files

- `lnp-recovery-prompt` presents the console recovery menu.
- `lnp-recovery-prompt.service` owns that prompt on the first virtual console.
- `sddm-lnp-recovery.conf` connects display-manager failure to the prompt for
  both supported service names.
- `lnp-restore` lists snapshots or selects a snapshot for the next boot while
  leaving the separate home subvolume untouched.

Snapshots are created by `../guard/lnp-guard`. Packaging is in
`../specs/lnp.spec`.
