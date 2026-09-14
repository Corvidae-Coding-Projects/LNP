# Fedora RPM Specifications

These files tell Fedora how to build and install LNP packages.

- `lnp.spec` builds the desktop metapackage and the look-and-feel, defaults,
  apply, welcome, error, guard, and recovery subpackages.
- `lnp-dock.spec` builds and installs the Rust dock plus its session files.
- `lnp-selinux.spec` builds and installs the SELinux alert UI, helpers, polkit
  action, and session files.

The version in an RPM spec may differ from the version inside a Rust crate.
Every installed file must appear in the matching `%install` and `%files`
sections with the right permissions and package name.
