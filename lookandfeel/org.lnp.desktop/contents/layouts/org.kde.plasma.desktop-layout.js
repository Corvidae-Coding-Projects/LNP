// LNP desktop layout
//
// Shape: a slim menu bar across the top, a floating icon dock at the bottom.
// Deliberately close to what someone arriving from macOS already knows, built
// entirely from stock Plasma applets so there is nothing bespoke to maintain.
//
// API reference: /usr/share/plasma/layout-templates/org.kde.plasma.desktop.defaultPanel

// Start from a known state. Without this, re-applying stacks duplicate panels
// on top of the existing ones instead of replacing them.
for (var i = panelIds.length - 1; i >= 0; --i) {
    panelById(panelIds[i]).remove();
}

// Wallpaper on every desktop of the current activity.
var desktops = desktopsForActivity(currentActivity());
for (var d = 0; d < desktops.length; ++d) {
    desktops[d].wallpaperPlugin = "org.kde.image";
}

// ------------------------------------------------------------------ top bar
// Global application menu on the left, status items and clock on the right.
// The spacer between them is what pushes the right-hand group over.

var menuBar = new Panel;
menuBar.location = "top";
menuBar.height = Math.round(gridUnit * 1.5);
menuBar.floating = false;
menuBar.hiding = "none";
menuBar.lengthMode = "fill";

menuBar.addWidget("org.kde.plasma.appmenu");
menuBar.addWidget("org.kde.plasma.panelspacer");
menuBar.addWidget("org.kde.plasma.systemtray");

var clock = menuBar.addWidget("org.kde.plasma.digitalclock");
clock.currentConfigGroup = ["Appearance"];
clock.writeConfig("showDate", true);

// --------------------------------------------------------------------- dock
// There is deliberately no bottom panel any more: the dock is lnp-dock, a
// standalone Wayland layer-shell client (see the lnp-dock package). It took
// over the launcher menu, the pinned apps and the window list, and being
// edge-anchored with real magnification it does the job better than the
// panel it replaced. This script now only shapes the top bar and wallpaper.
