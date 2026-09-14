# Layout Applier

This folder owns delivery of the LNP Plasma layout to an individual user. The
applier runs inside the graphical session, backs up every configuration file it
may replace, records the applied layout version, and supports status, explicit
reapplication, and restoration of the latest valid backup.

## Files

- `lnp-apply-layout` implements apply, backup, status, and revert behavior.
- `lnp-apply.service` runs the idempotent applier in the Plasma user session.
- `80-lnp.preset` enables the user service by default for new installations.

Packaging lives in `../specs/lnp.spec`. Regression coverage lives in
`../tests/test_regressions.py`.

## Validation

```sh
bash -n apply/lnp-apply-layout
python3 -m unittest discover -s tests -p 'test_regressions.py' -v
```

The tests replace desktop commands and write only into temporary XDG paths.
