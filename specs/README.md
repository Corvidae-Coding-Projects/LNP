# Fedora RPM Specifications

This folder is the packaging authority for the repository's installable LNP
components.

- `lnp.spec` builds the desktop metapackage and the look-and-feel, defaults,
  apply, welcome, error, guard, and recovery subpackages.
- `lnp-dock.spec` builds and installs the Rust dock plus its session files.
- `lnp-selinux.spec` builds and installs the SELinux alert UI, helpers, polkit
  action, and session files.

The spec versions are release versions and may intentionally differ from an
internal Cargo package version. Every installed source file must appear in the
corresponding `%install` and `%files` sections with the correct mode and owning
subpackage.
