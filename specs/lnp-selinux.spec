%global debug_package %{nil}

Name:           lnp-selinux
Version:        0.1.0
Release:        4%{?dist}
Summary:        Security Alerts -- plain-language SELinux denials with one-click fixes

License:        GPL-3.0-or-later
URL:            https://example.invalid/lnp
# Self-contained: vendor/ holds every crate dependency.
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gcc
BuildRequires:  libxkbcommon-devel
BuildRequires:  systemd-rpm-macros

# The analysis engine. Without it there are no alerts to show, so this is a
# hard requirement rather than a suggestion.
Requires:       setroubleshoot-server
Requires:       polkit
# The fix helper's tools.
Requires:       policycoreutils
Requires:       policycoreutils-python-utils
Requires:       audit
%{?systemd_requires}

%description
When the built-in security system (SELinux) blocks something, this explains
what happened in ordinary words and offers the remedy as a button.

setroubleshoot's suggested shell commands are never executed. They are parsed
into a closed set of four validated actions -- repair a file's label, flip a
documented policy switch, record a file type, or build a permission rule --
and applied by a privileged helper that validates them independently. Actions
are graded by risk; the one that permits exactly what was blocked is never a
single click and says plainly why.

A background service turns each new alert into a calm desktop notification
with a button that opens the window.

%prep
%autosetup

%build
cargo build --release --offline

%install
install -Dpm 0755 target/release/lnp-selinux %{buildroot}%{_bindir}/lnp-selinux
install -Dpm 0755 lnp-selinux-fix %{buildroot}%{_libexecdir}/lnp-selinux-fix
install -Dpm 0644 org.lnp.selinux.policy %{buildroot}%{_datadir}/polkit-1/actions/org.lnp.selinux.policy
install -Dpm 0644 lnp-selinux.desktop %{buildroot}%{_datadir}/applications/lnp-selinux.desktop
install -Dpm 0644 lnp-selinux-watch.service %{buildroot}%{_userunitdir}/lnp-selinux-watch.service
install -Dpm 0644 83-lnp-selinux.preset %{buildroot}%{_userpresetdir}/83-lnp-selinux.preset

%post
%systemd_user_post lnp-selinux-watch.service
# The alert database is empty unless the analyser is actually running.
systemctl enable --now setroubleshootd.service >/dev/null 2>&1 || :

%preun
%systemd_user_preun lnp-selinux-watch.service

%files
%license LICENSE
%{_bindir}/lnp-selinux
%{_libexecdir}/lnp-selinux-fix
%{_datadir}/polkit-1/actions/org.lnp.selinux.policy
%{_datadir}/applications/lnp-selinux.desktop
%{_userunitdir}/lnp-selinux-watch.service
%{_userpresetdir}/83-lnp-selinux.preset

%changelog
* Fri Aug 07 2026 LNP Project <lnp@example.invalid> - 0.1.0-4
- Actually ship the watcher unit without ConditionEnvironment; the 0.1.0-3
  build packaged a stale copy.

* Fri Aug 07 2026 LNP Project <lnp@example.invalid> - 0.1.0-3
- Drop ConditionEnvironment=WAYLAND_DISPLAY from the watcher: it needs the
  session and system buses, not the compositor, and the condition silently
  skipped the unit for the whole session after a reboot.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.1.0-2
- Force snake_case D-Bus member names: zbus PascalCases them and
  setroubleshootd allowlists each exact name, so every call was refused.
- Watch by polling instead of signals: setroubleshootd is D-Bus activated
  and exits when idle, which kills any signal subscription.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.1.0-1
- Initial release: alert browser with risk-graded one-click fixes, a
  notification watcher, and a whitelist-only privileged helper.
