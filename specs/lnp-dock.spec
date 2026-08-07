# Rust release builds carry no debuginfo by default, and this is a local
# package; skip the empty debuginfo subpackage rather than fight for one.
%global debug_package %{nil}

Name:           lnp-dock
Version:        0.5.1
Release:        1%{?dist}
Summary:        Dock for Linux for Normal People

License:        GPL-3.0-or-later
URL:            https://example.invalid/lnp
# Self-contained: vendor/ holds every crate dependency, .cargo/config.toml
# redirects cargo to it, so %%build needs no network.
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gcc
BuildRequires:  libxkbcommon-devel
BuildRequires:  systemd-rpm-macros

# The applications the default pins fall back to are Recommends, not Requires:
# the dock resolves what is actually installed and drops what is not.
Recommends:     dolphin
Recommends:     konsole

%{?systemd_requires}

%description
A Wayland layer-shell dock for KDE Plasma, built for people who should never
need to open a terminal.

Icons magnify on hover without ever moving their centres, so there is no
moving-target penalty (the design follows the expanding-target and fisheye
pointing literature rather than copying the macOS Dock). The dock anchors
flush to the screen edge, preserving the throw-the-pointer-at-the-edge
guarantee, and magnified icons overflow upward without pushing windows around.

Live windows come over org_kde_plasma_window_management -- the same protocol
Plasma's own task manager uses. KWin only reveals that interface to a client
whose binary matches the Exec of an installed desktop file declaring it, which
is why this package ships lnp-dock.desktop: it is the dock's credential, not a
menu entry.

%prep
%autosetup

%build
cargo build --release --offline

%install
install -Dpm 0755 target/release/lnp-dock %{buildroot}%{_bindir}/lnp-dock
install -Dpm 0644 lnp-dock.desktop %{buildroot}%{_datadir}/applications/lnp-dock.desktop
install -Dpm 0644 lnp-dock.service %{buildroot}%{_userunitdir}/lnp-dock.service
install -Dpm 0644 81-lnp-dock.preset %{buildroot}%{_userpresetdir}/81-lnp-dock.preset

%post
%systemd_user_post lnp-dock.service

%preun
%systemd_user_preun lnp-dock.service

%files
%license LICENSE
%{_bindir}/lnp-dock
%{_datadir}/applications/lnp-dock.desktop
%{_userunitdir}/lnp-dock.service
%{_userpresetdir}/81-lnp-dock.preset

%changelog
* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.5.1-1
- Fix leaked popup grabs that broke context menus in every other
  application. Opening a second dock menu created the new popup and took
  its grab before the old popup was dropped, destroying popups out of
  order; the compositor kept the stale grab and dismissed other clients'
  menus on sight. Destroy before create.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.5.0-1
- Apps button (first slot): a categorized, scrollable application menu --
  category rail, icon grid, wheel scrolling, click to launch. Scans the XDG
  application directories with NoDisplay/Hidden/OnlyShowIn/Terminal filtering
  and first-in-precedence dedupe. This replaces the old panel's Kickoff.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.4.0-1
- Drag to reorder pins: hold and move a pinned icon past 8px to pick it up;
  the row previews the new order live with an insertion gap, release saves
  it. Unresolvable pins keep their file positions. Activation now happens on
  release rather than press, as everywhere else.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.3.0-1
- Hover tooltips: app name plus up to four window titles (ellipsized, with
  an overflow count) in an input-transparent popup above the hovered icon.
  Appears after the pointer settles; switches instantly while sliding along
  the dock; yields to the right-click menu.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.2.0-2
- Fix the menu never opening: the seat handle was only stored in new_seat,
  which does not fire for a seat that already exists at startup. Store it in
  new_capability as well. Menu lifecycle now logged for diagnosability.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.2.0-1
- Right-click pin management: xdg_popup menu with pointer grab offering
  "Pin to dock" on running unpinned apps and "Unpin from dock" on pins.
- Pins persist in ~/.config/lnp-dock/pins (plain text, hand-editable);
  unresolvable pins are kept but not shown.
- Text rendering via fontdue with fc-match font discovery.

* Thu Aug 06 2026 LNP Project <lnp@example.invalid> - 0.1.0-1
- Initial packaging: layer-shell dock with fixed-centre magnification,
  live window list via the plasma-window-management trust grant, and a
  systemd user service enabled by preset for autostart.
