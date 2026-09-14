# LNP Security Alerts

This folder contains an app that explains SELinux blocks and a small set of
administrator tools that can apply a reviewed fix.

## Layout

- `lnp-selinux/` is the Rust GUI and background alert watcher.
- `lnp-selinux-fix` is the administrator helper. It accepts only a fixed list
  of actions.
- `lnp-selinux-policy` reloads and checks one selected block before creating a
  custom policy module.
- `org.lnp.selinux.policy` defines polkit authorization.
- The desktop file, user service, and preset integrate the application into the
  Plasma session.

The safety checks are explained in `../docs/selinux-policy-generation.md`.
Packaging is in `../specs/lnp-selinux.spec`.

## Validation

```sh
(cd selinux/lnp-selinux && cargo test --locked)
python3 -m unittest discover -s tests -p 'test_selinux_policy.py' -v
```
