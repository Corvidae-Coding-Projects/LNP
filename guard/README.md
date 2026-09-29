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

## Recovery and retention

Snapshot names include a sequence number, so two requests in one second do not
collide. Snapshot, cleanup, restore, and the entire guarded update share
`/run/lnp/operation.lock`. A competing operation stops with a retry message.
A restore awaiting restart prevents a new snapshot or update.

Cleanup keeps the requested number of dated snapshots (five after a snapshot,
three from Welcome), plus the newest pre-update snapshot if it is older.
Replaced roots are stored under `snapshots/replaced-roots/` at the filesystem's
top level. Cleanup keeps the two newest replaced roots and preserves any
mounted, running, or default root. Legacy `root.before-restore-*` subvolumes
are moved there only when they are not in use. Cleanup stops if it cannot
establish which subvolumes are in use.

The restart hold lasts at most 30 minutes, with a 31-minute service limit as a
fallback. Failure releases the hold and reports that graphics may not work
after restarting; it does not claim success. Both `akmod-nvidia` and
`akmod-nvidia-open` require a module for the newest installed Fedora kernel.
If `/usr/local/libexec/lnp-driver-guard` exists, its `ready` command adds local
hardware checks. The packaged guard still owns the bounded wait.
