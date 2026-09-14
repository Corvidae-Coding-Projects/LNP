# LNP Security Alerts

This folder contains the plain-language SELinux alert application and the
small privileged boundary used to apply a reviewed fix.

## Layout

- `lnp-selinux/` is the Rust GUI and background alert watcher.
- `lnp-selinux-fix` is the root helper with a fixed action vocabulary.
- `lnp-selinux-policy` independently reloads and verifies one selected denial
  before generating a custom policy module.
- `org.lnp.selinux.policy` defines polkit authorization.
- The desktop file, user service, and preset integrate the application into the
  Plasma session.

The flow is documented in `../docs/selinux-policy-generation.md`; packaging is
in `../specs/lnp-selinux.spec`.

## Validation

```sh
(cd selinux/lnp-selinux && cargo test --locked)
python3 -m unittest discover -s tests -p 'test_selinux_policy.py' -v
```
