# Layout Applier

This is the tool that applies the LNP layout for a user. It runs after sign-in,
saves the settings it may replace, and records the layout version. It can also
show its status, apply the layout again, or restore the last good backup.

## Files

- `lnp-apply-layout` applies, backs up, checks, and restores the layout.
- `lnp-apply.service` runs the idempotent applier in the Plasma user session.
- `80-lnp.preset` enables the user service by default for new installations.

Packaging lives in `../specs/lnp.spec`. Regression coverage lives in
`../tests/test_regressions.py`.

## Validation

```sh
bash -n apply/lnp-apply-layout
python3 -m unittest discover -s tests -p 'test_regressions.py' -v
```

The tests use fake desktop commands and temporary settings folders.
