# Plain-Language Error Daemon

`lnp-errord` watches the journals available to the current user and turns a
small set of actionable failures into desktop notifications. It explains
SELinux denials, failed services, out-of-memory events, and disk pressure in
ordinary language while rate-limiting repeated stories.

## Files

- `lnp-errord` is the unprivileged Python daemon.
- `lnp-errord.service` runs it in the Plasma user session.
- `82-lnp-errord.preset` enables the service by default.

The daemon does not repair the system and holds no elevated privileges.
Packaging is in `../specs/lnp.spec`.

## Validation

```sh
python3 -m py_compile errord/lnp-errord
python3 -m unittest discover -s tests -v
```
