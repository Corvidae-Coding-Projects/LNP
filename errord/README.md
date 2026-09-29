# Plain-Language Error Daemon

`lnp-errord` watches the system messages a user is allowed to read. It turns a
small set of useful errors into desktop notifications, including SELinux
blocks, failed services, low-memory events, and low disk space. Repeated errors
are limited so they do not become notification spam.

## Files

- `lnp-errord` is the unprivileged Python daemon.
- `lnp-errord.service` runs it in the Plasma user session.
- `82-lnp-errord.preset` enables the service by default.

The program only reports problems. It does not repair the system or run as an
administrator. Packaging is in `../specs/lnp.spec`.

## Validation

```sh
python3 -m py_compile errord/lnp-errord
python3 -m unittest discover -s tests -v
```

The daemon restarts when its journal follower exits, including a clean exit.
Journal and disk notifications share a thread-safe five-minute limit: at most
five individual notices and one summary. Repeated stories retain their own
limits, including the six-hour disk-warning limit. Notification bodies are
escaped before being sent to the desktop, so names from journal entries are
shown as text rather than markup.
