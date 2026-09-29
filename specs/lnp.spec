%global lnf_id org.lnp.desktop

# The layout version is deliberately NOT the package version. The applier
# re-runs (and rebuilds every user's panels) whenever this number changes, so
# it must change only when the shipped layout actually does -- a routine
# package release must never cost a user their panel tweaks. Bump by hand,
# with a changelog entry saying what changed in the layout.
%global layout_version 2

Name:           lnp
Version:        0.3.0
Release:        8%{?dist}
Summary:        Linux for Normal People -- sane desktop defaults for Plasma

License:        GPL-3.0-or-later
URL:            https://example.invalid/lnp
Source0:        %{name}-%{version}.tar.gz

BuildArch:      noarch
BuildRequires:  systemd-rpm-macros

%description
Linux for Normal People (LNP) is an opinionated desktop configuration for
Fedora KDE Plasma, aimed at people who want a desktop that works rather than a
desktop to configure.

It is mostly data rather than code: Plasma already supports everything needed
to reshape the desktop, so LNP supplies a different set of defaults through
Plasma's own supported mechanisms and forks nothing.

This source package builds subpackages only. Install lnp-desktop.


# ---------------------------------------------------------------- metapackage
%package -n lnp-desktop
Summary:        Linux for Normal People desktop (metapackage)
Requires:       lnp-look-and-feel = %{version}-%{release}
Requires:       lnp-defaults = %{version}-%{release}
Requires:       lnp-apply = %{version}-%{release}
Requires:       lnp-welcome = %{version}-%{release}
Requires:       lnp-errord = %{version}-%{release}
Requires:       lnp-guard = %{version}-%{release}
Requires:       lnp-recovery = %{version}-%{release}
# Built from its own spec (arch-specific Rust), so unversioned.
Requires:       lnp-dock

%description -n lnp-desktop
Metapackage pulling in the complete LNP desktop: the look-and-feel package,
the system-wide defaults, and the applier that puts them in place.

Installing this changes the Plasma layout at each user's next login. The
previous configuration is backed up first and can be restored with
"lnp-apply-layout --revert".


# ------------------------------------------------------------- look-and-feel
%package -n lnp-look-and-feel
Summary:        LNP Plasma look-and-feel package
# The layout script builds its panels out of stock applets shipped by these.
Requires:       plasma-workspace
Requires:       plasma-desktop

%description -n lnp-look-and-feel
The Plasma look-and-feel package org.lnp.desktop: a slim menu bar across the
top and a floating, centred icon dock at the bottom, with window controls on
the left. Built entirely from stock Plasma applets.

Also owns the theme, colour scheme, icon theme and cursor defaults, so those
have exactly one owner.


# ------------------------------------------------------------------ defaults
%package -n lnp-defaults
Summary:        LNP system-wide Plasma defaults
Requires:       plasma-workspace

%description -n lnp-defaults
System-wide defaults in /etc/xdg for KWin, Dolphin, font rendering and the
screen locker.

These sit below ~/.config in the XDG_CONFIG_DIRS cascade, so they apply to
users who have not changed the corresponding setting and are silently
overridden the moment they do. Nothing here is enforced.


# ------------------------------------------------------------------- welcome
%package -n lnp-welcome
Summary:        LNP first-run welcome app
Requires:       python3-pyside6
Requires:       polkit
Requires:       pciutils
# The welcome app is the GUI face of the applier's revert.
Requires:       lnp-apply = %{version}-%{release}

%description -n lnp-welcome
The first-run welcome app: checks media playback and graphics drivers, sets
them up with one click through polkit (no terminal, ever), gives a short tour,
and can restore the user's pre-LNP desktop from the backup the applier took.

Shows itself once per user via XDG autostart, then only when opened from the
menu (search for "Welcome").


# --------------------------------------------------------------------- guard
%package -n lnp-guard
Summary:        Guarded updates with system snapshots
Requires:       btrfs-progs
Requires:       dnf
Requires:       kmod
Requires:       /usr/bin/flock
%{?systemd_requires}

%description -n lnp-guard
Guarded automatic updates: a read-only btrfs snapshot before every update,
a daily timer that runs only on AC power, and a reboot guard that waits up to
30 minutes for NVIDIA and local driver checks. A failed check releases the
hold with a warning rather than claiming that restart is safe.

Snapshots restore with lnp-restore (from lnp-recovery) or from the recovery
prompt if the desktop fails to start.


# ------------------------------------------------------------------ recovery
%package -n lnp-recovery
Summary:        Plain-language recovery when the desktop cannot start
Requires:       btrfs-progs
# Authenticated console login for the support option.
Requires:       /usr/bin/login
Requires:       lnp-guard = %{version}-%{release}
# Select a boot entry compatible with the restored root.
Requires:       grubby
%{?systemd_requires}

%description -n lnp-recovery
If the display manager gives up, a calm plain-language prompt takes the
console instead of a black screen: undo the last system change (restores the
pre-update snapshot lnp-guard took), just try again, or a shell for whoever
helps with this computer. Personal files live on a separate subvolume and are
never touched by a restore.


# -------------------------------------------------------------------- errord
%package -n lnp-errord
Summary:        Plain-language desktop notifications for system failures
Requires:       glib2
Requires:       python3
%{?systemd_requires}

%description -n lnp-errord
Watches the journal and translates the failures a person can do something
about -- SELinux denials, background service crashes, out-of-memory kills --
into desktop notifications written in plain language: what happened, whether
it matters, what to do next. Everything else stays in the journal.

Hard rate limiting: the first notification is information, the fifth is spam.


# -------------------------------------------------------------------- applier
%package -n lnp-apply
Summary:        LNP layout applier
Requires:       lnp-look-and-feel = %{version}-%{release}
# plasma-apply-lookandfeel
Requires:       plasma-workspace
# kwriteconfig6, for the per-device touchpad settings
Requires:       kf6-kconfig
# pgrep, for detecting a live session
Requires:       procps-ng
# Serialize application and restoration of per-user configuration.
Requires:       /usr/bin/flock
%{?systemd_requires}

%description -n lnp-apply
Applies the LNP layout to a user account, once, at their next Plasma login.

Runs as a systemd user service rather than from an RPM scriptlet because
Plasma's per-user configuration cannot be safely rewritten by root while a
session is live. Idempotent, and always backs up the previous configuration to
~/.local/state/lnp/ before making a change.


%prep
%autosetup


%build
# Nothing to build: this package is configuration data and one shell script.


%install
# Look-and-feel package.
install -d %{buildroot}%{_datadir}/plasma/look-and-feel
cp -a lookandfeel/%{lnf_id} %{buildroot}%{_datadir}/plasma/look-and-feel/

# System-wide defaults.
install -d %{buildroot}%{_sysconfdir}/xdg
install -pm 0644 defaults/etc/xdg/* %{buildroot}%{_sysconfdir}/xdg/

# Applier, its user unit, and the preset that enables it for everyone.
install -Dpm 0755 apply/lnp-apply-layout  %{buildroot}%{_bindir}/lnp-apply-layout
install -Dpm 0644 apply/lnp-apply.service %{buildroot}%{_userunitdir}/lnp-apply.service
install -Dpm 0644 apply/80-lnp.preset     %{buildroot}%{_userpresetdir}/80-lnp.preset

# The applier compares this against a per-user stamp to decide whether to run;
# see the %%{layout_version} comment at the top of this spec.
install -d %{buildroot}%{_datadir}/lnp
echo '%{layout_version}' > %{buildroot}%{_datadir}/lnp/layout-version

# Error translator.
install -Dpm 0755 errord/lnp-errord %{buildroot}%{_bindir}/lnp-errord
install -Dpm 0644 errord/lnp-errord.service %{buildroot}%{_userunitdir}/lnp-errord.service
install -Dpm 0644 errord/82-lnp-errord.preset %{buildroot}%{_userpresetdir}/82-lnp-errord.preset

# Guard: snapshots, guarded updates, reboot guard.
install -Dpm 0644 guard/lnp-btrfs-common %{buildroot}%{_libexecdir}/lnp-btrfs-common
install -Dpm 0755 guard/lnp-guard %{buildroot}%{_libexecdir}/lnp-guard
install -Dpm 0644 guard/lnp-guard-update.service %{buildroot}%{_unitdir}/lnp-guard-update.service
install -Dpm 0644 guard/lnp-guard-update.timer %{buildroot}%{_unitdir}/lnp-guard-update.timer
install -Dpm 0644 guard/lnp-reboot-guard.service %{buildroot}%{_unitdir}/lnp-reboot-guard.service
install -Dpm 0644 guard/lnp-reboot-guard.path %{buildroot}%{_unitdir}/lnp-reboot-guard.path
install -d %{buildroot}%{_presetdir}
cat > %{buildroot}%{_presetdir}/80-lnp-guard.preset <<'EOF'
enable lnp-guard-update.timer
enable lnp-reboot-guard.path
EOF

# Recovery: restore tool, console prompt, sddm failure hook.
install -Dpm 0755 recovery/lnp-restore %{buildroot}%{_libexecdir}/lnp-restore
install -Dpm 0755 recovery/lnp-recovery-prompt %{buildroot}%{_libexecdir}/lnp-recovery-prompt
install -Dpm 0644 recovery/lnp-recovery-prompt.service %{buildroot}%{_unitdir}/lnp-recovery-prompt.service
install -Dpm 0644 recovery/sddm-lnp-recovery.conf %{buildroot}%{_unitdir}/sddm.service.d/lnp-recovery.conf
install -Dpm 0644 recovery/sddm-lnp-recovery.conf %{buildroot}%{_unitdir}/plasmalogin.service.d/lnp-recovery.conf

# Welcome app, its privileged helper, and the polkit policy that lets the
# helper ask for authentication graphically.
install -Dpm 0755 welcome/lnp-welcome %{buildroot}%{_bindir}/lnp-welcome
install -Dpm 0755 welcome/lnp-setup %{buildroot}%{_libexecdir}/lnp-setup
install -Dpm 0644 welcome/org.lnp.setup.policy %{buildroot}%{_datadir}/polkit-1/actions/org.lnp.setup.policy
install -Dpm 0644 welcome/lnp-welcome.desktop %{buildroot}%{_datadir}/applications/lnp-welcome.desktop
install -Dpm 0644 welcome/lnp-welcome-autostart.desktop %{buildroot}%{_sysconfdir}/xdg/autostart/lnp-welcome.desktop


%post -n lnp-apply
%systemd_user_post lnp-apply.service

%preun -n lnp-apply
%systemd_user_preun lnp-apply.service

%post -n lnp-errord
%systemd_user_post lnp-errord.service

%preun -n lnp-errord
%systemd_user_preun lnp-errord.service

%post -n lnp-guard
%systemd_post lnp-guard-update.timer lnp-reboot-guard.path

%preun -n lnp-guard
%systemd_preun lnp-guard-update.timer lnp-reboot-guard.path

%post -n lnp-recovery
%systemd_post lnp-recovery-prompt.service

%preun -n lnp-recovery
%systemd_preun lnp-recovery-prompt.service


%files -n lnp-desktop
# Metapackage: intentionally owns no files.

%files -n lnp-look-and-feel
%license LICENSE
%doc README.md
%{_datadir}/plasma/look-and-feel/%{lnf_id}/

%files -n lnp-defaults
%license LICENSE
%config(noreplace) %{_sysconfdir}/xdg/kdeglobals
%config(noreplace) %{_sysconfdir}/xdg/kwinrc
%config(noreplace) %{_sysconfdir}/xdg/dolphinrc
%config(noreplace) %{_sysconfdir}/xdg/kscreenlockerrc

%files -n lnp-welcome
%license LICENSE
%{_bindir}/lnp-welcome
%{_libexecdir}/lnp-setup
%{_datadir}/polkit-1/actions/org.lnp.setup.policy
%{_datadir}/applications/lnp-welcome.desktop
%config(noreplace) %{_sysconfdir}/xdg/autostart/lnp-welcome.desktop

%files -n lnp-errord
%license LICENSE
%{_bindir}/lnp-errord
%{_userunitdir}/lnp-errord.service
%{_userpresetdir}/82-lnp-errord.preset

%files -n lnp-guard
%license LICENSE
%{_libexecdir}/lnp-guard
%{_libexecdir}/lnp-btrfs-common
%{_unitdir}/lnp-guard-update.service
%{_unitdir}/lnp-guard-update.timer
%{_unitdir}/lnp-reboot-guard.service
%{_unitdir}/lnp-reboot-guard.path
%{_presetdir}/80-lnp-guard.preset

%files -n lnp-recovery
%license LICENSE
%{_libexecdir}/lnp-restore
%{_libexecdir}/lnp-recovery-prompt
%{_unitdir}/lnp-recovery-prompt.service
%{_unitdir}/sddm.service.d/lnp-recovery.conf
%{_unitdir}/plasmalogin.service.d/lnp-recovery.conf

%files -n lnp-apply
%license LICENSE
%{_bindir}/lnp-apply-layout
%{_userunitdir}/lnp-apply.service
%{_userpresetdir}/80-lnp.preset
%dir %{_datadir}/lnp
%{_datadir}/lnp/layout-version


%changelog
* Mon Sep 28 2026 LNP Project <lnp@example.invalid> - 0.3.0-8
- Local Welcome repair rebuild for issue #2, retaining installed version pins.
- Propagate cleanup and security step failures with retained command details;
  distinguish skipped components, completed work, and partial failure.
- Preserve previous DNS configuration on write failure and keep UI retry
  available after failed helper operations.

* Mon Sep 28 2026 LNP Project <lnp@example.invalid> - 0.3.0-8
- Local repair rebuild for issue #1; retain the release to preserve the
  installed desktop metapackage's exact version dependencies.
- Select a compatible boot entry before restoring root and preserve rollback.
- Bound driver waits, retain saved roots safely, and serialize Btrfs operations.
- Cover open NVIDIA modules and generate collision-free snapshot names.
- Restart the notifier after clean journal exit; cap and escape notifications.

* Sat Aug 15 2026 LNP Project <lnp@example.invalid> - 0.3.0-8
- Make the error notification daemon tolerate null and non-text journal fields
  instead of crashing and restarting on malformed entries.

* Sat Aug 15 2026 LNP Project <lnp@example.invalid> - 0.3.0-7
- Fix the error notification service's Plasma startup ordering cycle by
  ordering it after plasmashell rather than graphical-session.target.

* Fri Aug 07 2026 LNP Project <lnp@example.invalid> - 0.3.0-6
- apply: drop ConditionEnvironment=XDG_CURRENT_DESKTOP too. Same silent
  skip as the other units; the applier already detects a live plasmashell
  and exits cleanly without one.

* Fri Aug 07 2026 LNP Project <lnp@example.invalid> - 0.3.0-5
- errord: stop reporting D-Bus activation wrappers as problems. Several are
  designed to fail -- KDE ships stubs such as org.kde.klipper.desktop with
  Exec=/usr/bin/false now the feature lives in plasmashell -- so "a
  background helper had a problem" was a false alarm naming something
  nobody could act on.
- errord: drop ConditionEnvironment=WAYLAND_DISPLAY. It needs the session
  bus, not the compositor, and the condition silently skipped the unit for
  the whole session after a reboot.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.3.0-4
- apply: stop setting DisableWhileTyping. It deadens the touchpad for a
  moment after every keystroke, which makes selecting text in a terminal
  feel like the mouse has stopped working. Reported as exactly that.
- apply: stop setting NaturalScroll; it is a preference, not a default.
- apply: touchpad settings are now only written when the user has no value
  for them. Writes into ~/.config have no cascade beneath them, so an
  unconditional write destroys a preference rather than setting a default.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.3.0-1
- New lnp-guard: btrfs snapshot before every update, daily guarded update
  timer (AC power only), and a logind reboot inhibitor that blocks restarts
  until a new kernel's NVIDIA module is built.
- New lnp-recovery: lnp-restore (root subvolume restore that keeps the
  replaced system aside) and a plain-language console prompt hooked to
  sddm failure offering undo-last-update / retry / helper shell.
- errord: disk-space guardian (90/95% thresholds, slow rate limit),
  setroubleshoot analyses preferred over raw AVC denials, and friendly
  notifications for guarded updates and the reboot hold.
- lnp-setup: clean (logs, package caches, unused runtimes, old snapshots)
  and secure (firewall, firmware checks, DNS-over-TLS opportunistic,
  setroubleshoot install; SSH and SELinux state reported, never flipped).
- welcome: Storage and Security pages driving clean/secure.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.2.3-1
- Layout version 2: the Plasma bottom panel is gone. lnp-dock (with its new
  Apps button) has taken over launcher, pins and window list. The top bar
  is unchanged. Applies at next login; the previous layout is backed up.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.2.2-1
- welcome: pass --disable-internal-agent to pkexec and translate the
  no-authentication-agent failure honestly instead of blaming the password.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.2.1-1
- New lnp-errord subpackage: journal watcher translating SELinux denials,
  service failures and OOM kills into plain-language notifications.
- Layout version decoupled from the package version (now %%{layout_version}),
  so routine releases no longer rebuild user panels. Existing stamps migrate.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.2.0-1
- New lnp-welcome subpackage: first-run wizard with one-click codec and
  NVIDIA driver setup via polkit, dock toggle, GUI restore of the pre-LNP
  desktop, and a short tour. Autostarts once per user, then lives in the menu.
- lnp-desktop now pulls in lnp-welcome and lnp-dock.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.1.1-1
- Move window button order from the look-and-feel defaults to /etc/xdg/kwinrc.
  plasma-apply-lookandfeel does not apply [kwinrc] sections of a look-and-feel
  defaults file, so the setting never reached KWin.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.1.0-1
- Initial packaging: look-and-feel, system defaults, and the layout applier.
