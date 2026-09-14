//! lnp-dock -- a dock for Linux for Normal People.
//!
//! A standalone Wayland layer-shell client rather than a Plasma applet. Two
//! reasons, both of which came out of measurement rather than preference:
//!
//! * An applet is clipped to its panel's thickness, which is exactly the
//!   constraint that makes real magnification awkward. Our own surface can be
//!   taller than the space it reserves, so magnified icons overflow upward
//!   over the wallpaper instead of being cut off.
//!
//! * A surface anchored to the screen edge keeps the Fitts "infinite depth"
//!   property: you can slam the pointer at the bottom of the screen and be
//!   certain of landing on the dock. A floating panel with a margin throws
//!   that away, which is a real and measurable regression.
//!
//! The magnification model itself lives in `magnify` and is where the
//! interesting design decisions are documented.

mod apps;
mod appsmenu;
mod launchers;
mod magnify;
mod menu;
mod pins;
mod render;
mod text;
mod tooltip;
mod windows;

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;

use anyhow::{Context, Result};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, FrameCallbackData, Region, Surface},
    delegate_dispatch2, delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        Capability, SeatHandler, SeatState,
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
    },
    shell::{
        WaylandSurface,
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        xdg::{
            XdgPositioner, XdgShell,
            popup::{Popup, PopupConfigure, PopupHandler},
        },
    },
    shm::{Shm, ShmHandler, slot::SlotPool},
};
use smithay_client_toolkit::reexports::client::{
    Connection, QueueHandle,
    globals::registry_queue_init,
    protocol::{wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
};
use tiny_skia::{Pixmap, Rect};

use wayland_protocols_plasma::plasma_window_management::client::org_kde_plasma_window_management::OrgKdePlasmaWindowManagement;

use appsmenu::{AppsMenuModel, Hover};
use launchers::Launcher;
use magnify::{MagnifyParams, SETTLE_AFTER, SpeedDamping};
use menu::{MenuAction, MenuModel};
use render::{DockTheme, IconCache};
use text::TextRenderer;
use windows::{WindowManagementData, WindowSet};

/// Resting icon size. Fitts rewards the *final* size of a target, so a
/// generous resting size does more work than magnification does.
const BASE_ICON: f32 = 48.0;
const SLOT_WIDTH: f32 = 60.0;
const MAX_SCALE: f32 = 1.6;
const PADDING: f32 = 10.0;

/// The dock's surface is taller than the space it reserves, so that a magnified
/// icon can overflow upward over the wallpaper.
const RESTING_HEIGHT: f32 = BASE_ICON + PADDING * 2.0;
const SURFACE_HEIGHT: f32 = BASE_ICON * MAX_SCALE + PADDING * 3.0;

/// Resolve pin specs into launchers, dropping what cannot resolve. A pin that
/// cannot be resolved is not drawn -- silently showing something that does
/// nothing when clicked is worse than showing nothing -- but it stays in the
/// pins file, so a temporarily uninstalled app comes back on reinstall.
fn resolve_pins(specs: &[String]) -> Vec<Launcher> {
    specs
        .iter()
        .filter_map(|spec| match launchers::resolve(spec) {
            Ok(l) => Some(l),
            Err(e) => {
                eprintln!("lnp-dock: skipping {spec}: {e}");
                None
            }
        })
        .collect()
}

/// Run a command detached: setsid means applications outlive the dock, so a
/// dock restart never takes the user's windows with it.
fn spawn_detached(exec: &str, name: &str) {
    let result = Command::new("setsid")
        .arg("-f")
        .arg("sh")
        .arg("-c")
        .arg(exec)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    if let Err(e) = result {
        eprintln!("lnp-dock: failed to launch {name}: {e}");
    }
}

fn main() -> Result<()> {
    let pin_specs = pins::load();
    let items = resolve_pins(&pin_specs);

    // An empty pin list is valid: Apps and the live window list still work.

    let conn = Connection::connect_to_env().context("connecting to the Wayland compositor")?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh)
        .context("wl_compositor unavailable")?;
    let layer_shell = LayerShell::bind(&globals, &qh)
        .context("zwlr_layer_shell_v1 unavailable -- this compositor cannot host a dock")?;
    let shm = Shm::bind(&globals, &qh).context("wl_shm unavailable")?;
    // xdg_wm_base is only used to parent the right-click menu popups.
    let xdg_shell = XdgShell::bind(&globals, &qh).context("xdg_wm_base unavailable")?;

    // KWin only reveals this global to clients whose binary matches the Exec
    // of a desktop file declaring X-KDE-Wayland-Interfaces. If it is absent we
    // still run -- launchers work, the window list does not -- and we say so
    // once, in the log, with the actual fix.
    let window_manager = match globals.bind::<OrgKdePlasmaWindowManagement, _, _>(
        &qh,
        13..=18,
        WindowManagementData,
    ) {
        Ok(m) => {
            eprintln!("lnp-dock: window management granted; live window list enabled");
            Some(m)
        }
        Err(e) => {
            eprintln!(
                "lnp-dock: no window list ({e}); running launchers-only. \
                 The compositor grants this only if lnp-dock.desktop is installed \
                 and its Exec matches this binary."
            );
            None
        }
    };

    // +1 slot for the Apps button.
    let width = ((items.len() + 1) as f32 * SLOT_WIDTH + PADDING * 2.0).ceil() as u32;
    let height = SURFACE_HEIGHT.ceil() as u32;

    let surface = compositor.create_surface(&qh);
    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("lnp-dock"), None);

    layer.set_anchor(Anchor::BOTTOM);
    layer.set_size(width, height);
    // Reserve only the resting height. The extra surface above it is where
    // magnified icons go, and must not push windows around.
    layer.set_exclusive_zone(RESTING_HEIGHT as i32);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.commit();

    let pool = SlotPool::new(width as usize * height as usize * 4, &shm)
        .context("creating shm pool")?;

    let mut dock = Dock {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        pool,
        layer,
        exit: false,
        first_configure: true,
        width,
        height,
        scale: 1,
        pointer: None,
        keyboard: None,
        pointer_x: None,
        last_motion: Instant::now(),
        animating: false,
        launchers: items,
        pin_specs,
        windows: WindowSet::default(),
        window_manager,
        slots: Vec::new(),
        compositor,
        xdg_shell,
        seat: None,
        text: TextRenderer::new(),
        menu: None,
        tooltip: None,
        drag: None,
        apps_menu: None,
        apps_catalog: None,
        apps_icon: ["start-here-kde", "applications-all", "start-here"]
            .iter()
            .find_map(|n| launchers::lookup_icon(n)),
        cache: IconCache::default(),
        params: MagnifyParams {
            slot_width: SLOT_WIDTH,
            max_scale: MAX_SCALE,
            radius_slots: 2.0,
        },
        damping: SpeedDamping::default(),
        theme: DockTheme::default(),
    };

    dock.rebuild_slots();

    while !dock.exit {
        event_queue.blocking_dispatch(&mut dock)?;
    }

    Ok(())
}

struct Dock {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    pool: SlotPool,
    layer: LayerSurface,

    exit: bool,
    first_configure: bool,
    width: u32,
    height: u32,
    scale: i32,

    pointer: Option<wl_pointer::WlPointer>,
    keyboard: Option<smithay_client_toolkit::reexports::client::protocol::wl_keyboard::WlKeyboard>,
    /// Pointer x in surface-local logical pixels, if it is over the dock.
    pointer_x: Option<f32>,
    last_motion: Instant,
    /// Whether we are driving frame callbacks to animate magnification.
    animating: bool,

    launchers: Vec<Launcher>,
    /// The pin specs as stored on disk; `launchers` is what resolved of them.
    pin_specs: Vec<String>,
    /// Live windows, fed by the plasma-window-management events. Empty forever
    /// if the compositor did not grant us the interface.
    windows: WindowSet,
    window_manager: Option<OrgKdePlasmaWindowManagement>,
    /// What the dock actually shows: pins first, then running apps that match
    /// no pin, grouped by app id. Rebuilt on every model change.
    slots: Vec<Slot>,
    compositor: CompositorState,
    xdg_shell: XdgShell,
    seat: Option<wl_seat::WlSeat>,
    text: Option<TextRenderer>,
    /// The open right-click menu, if any.
    menu: Option<OpenMenu>,
    /// The visible hover tooltip, if any.
    tooltip: Option<OpenTooltip>,
    /// Left button currently held on a slot.
    drag: Option<DragState>,
    /// The open Apps menu, if any.
    apps_menu: Option<OpenAppsMenu>,
    /// Scanned application catalogue, filled on first menu open. Fresh per
    /// dock process; newly installed apps appear after the dock restarts or
    /// on the next session at the latest.
    apps_catalog: Option<Vec<apps::AppEntry>>,
    /// Icon for the Apps button, resolved once.
    apps_icon: Option<PathBuf>,
    cache: IconCache,
    params: MagnifyParams,
    damping: SpeedDamping,
    theme: DockTheme,
}

/// One position in the dock.
struct Slot {
    kind: SlotKind,
    icon: Option<PathBuf>,
    /// Uuids of this slot's windows, in mapping order.
    windows: Vec<String>,
}

enum SlotKind {
    /// The Apps button: always slot 0, opens the application menu. This is
    /// the door to everything not pinned -- the job the old panel's Kickoff
    /// did.
    AppsButton,
    /// A pinned launcher (index into `Dock::launchers`). Windows whose app id
    /// matches the pin attach here, so pin and app are one icon -- the
    /// launcher/window unification every dock since NeXTSTEP has aimed for.
    Launcher(usize),
    /// A running application nobody pinned.
    App(String),
}

/// Pinned slots start after the Apps button.
const PIN_SLOT_OFFSET: usize = 1;

/// The open Apps menu: popup surface, layout model, and the scanned catalogue
/// snapshot it is showing.
struct OpenAppsMenu {
    popup: Popup,
    model: AppsMenuModel,
}

/// An open right-click menu: the popup surface plus the testable model.
struct OpenMenu {
    popup: Popup,
    model: MenuModel,
}

/// A visible tooltip: input-transparent popup plus its composed lines.
struct OpenTooltip {
    popup: Popup,
    model: tooltip::TooltipModel,
    slot: usize,
}

/// Movement past this many logical pixels turns a press into a drag; under
/// it, release is a click. 8px sits above pointer jitter and below the
/// smallest intentional drag.
const DRAG_THRESHOLD: f32 = 8.0;

/// A left button held on a slot: a click until proven a drag.
struct DragState {
    /// Slot index the press landed on. Pinned slots sit at
    /// `PIN_SLOT_OFFSET..PIN_SLOT_OFFSET + launchers.len()`, so the launcher
    /// index is `slot_idx - PIN_SLOT_OFFSET`.
    slot_idx: usize,
    press_x: f32,
    current_x: f32,
    /// True once movement exceeded DRAG_THRESHOLD (pinned slots only).
    active: bool,
}

impl Dock {
    /// Centre of slot `index` in surface-local logical pixels.
    fn slot_center(&self, index: usize) -> f32 {
        self.params.slot_center(PADDING, index)
    }

    /// Which slot, if any, sits under a surface-local x coordinate.
    fn slot_at(&self, x: f32) -> Option<usize> {
        let rel = x - PADDING;
        if rel < 0.0 {
            return None;
        }
        let idx = (rel / self.params.slot_width) as usize;
        (idx < self.slots.len()).then_some(idx)
    }

    /// Recompute the slot list from pins plus live windows.
    fn rebuild_slots(&mut self) {
        let launchers = &self.launchers;

        let mut slots: Vec<Slot> = vec![Slot {
            kind: SlotKind::AppsButton,
            icon: self.apps_icon.clone(),
            windows: Vec::new(),
        }];
        slots.extend(launchers.iter().enumerate().map(|(i, l)| Slot {
            kind: SlotKind::Launcher(i),
            icon: l.icon.clone(),
            windows: Vec::new(),
        }));

        // Running apps that match no pin get their own slots, appended in
        // first-seen order so the layout stays stable while windows churn.
        let mut extras: Vec<Slot> = Vec::new();

        for (uuid, w) in self.windows.visible() {
            let attached = slots.iter_mut().find(|s| {
                matches!(&s.kind, SlotKind::Launcher(i)
                    if launchers::app_id_matches(&launchers[*i], &w.app_id))
            });

            if let Some(slot) = attached {
                slot.windows.push(uuid.clone());
            } else if let Some(slot) = extras
                .iter_mut()
                .find(|s| matches!(&s.kind, SlotKind::App(a) if *a == w.app_id))
            {
                slot.windows.push(uuid.clone());
            } else {
                extras.push(Slot {
                    kind: SlotKind::App(w.app_id.clone()),
                    icon: launchers::icon_for_app_id(&w.app_id, w.themed_icon.as_deref()),
                    windows: vec![uuid.clone()],
                });
            }
        }

        slots.extend(extras);
        self.slots = slots;
    }

    /// Called by the window event handlers whenever the model shifts.
    fn window_model_changed(&mut self, qh: &QueueHandle<Self>) {
        // Slot indices are about to shift under any visible tooltip.
        self.hide_tooltip();
        self.rebuild_slots();

        if std::env::var_os("LNP_DOCK_DEBUG").is_some() {
            for (i, s) in self.slots.iter().enumerate() {
                let label = match &s.kind {
                    SlotKind::AppsButton => "apps-button".to_string(),
                    SlotKind::Launcher(l) => format!("pin:{}", self.launchers[*l].name),
                    SlotKind::App(a) => format!("app:{a}"),
                };
                eprintln!("lnp-dock: slot {i} {label} windows={}", s.windows.len());
            }
        }

        let want = (self.slots.len() as f32 * SLOT_WIDTH + PADDING * 2.0).ceil() as u32;
        if want != self.width {
            self.width = want;
            self.layer.set_size(want, self.height);
            // The compositor answers with a configure, and that drives the
            // redraw at the agreed size; drawing now would race it.
            self.layer.commit();
        } else {
            self.draw(qh);
        }
    }

    /// Click semantics: launch when nothing runs; focus-or-minimize a single
    /// window; cycle through several. One button does the obvious thing --
    /// nobody should need to learn middle-click chords to use a dock.
    fn activate_slot(&self, index: usize) {
        let Some(slot) = self.slots.get(index) else {
            return;
        };

        let wins: Vec<&windows::WindowInfo> = slot
            .windows
            .iter()
            .filter_map(|u| self.windows.get(u))
            .collect();

        match wins.len() {
            0 => {
                match slot.kind {
                    SlotKind::Launcher(i) => self.launch(i),
                    // The Apps menu opens on press, not on release-click.
                    SlotKind::AppsButton | SlotKind::App(_) => {}
                }
            }
            1 => {
                if wins[0].is_active() {
                    wins[0].minimize();
                } else {
                    wins[0].activate();
                }
            }
            _ => {
                let next = match wins.iter().position(|w| w.is_active()) {
                    Some(p) => (p + 1) % wins.len(),
                    None => 0,
                };
                wins[next].activate();
            }
        }
    }

    /// Open the right-click menu for a slot, anchored above it, with a proper
    /// pointer grab so it dismisses the way every menu the user has ever used
    /// dismisses: click elsewhere, it goes away.
    fn open_menu(&mut self, qh: &QueueHandle<Self>, slot_idx: usize, serial: u32) {
        // Tear down whatever is open BEFORE building anything new.
        //
        // Assigning to self.menu at the end of this function would also drop
        // the old popup -- but only after the new one exists and has taken its
        // grab. That leaves two popups alive at once and destroys the older
        // one out of order, which xdg_shell forbids (popups are strictly
        // last-in-first-out). The compositor's response is to keep the stale
        // grab, and a stale grab routes all input into our popup chain: every
        // other application's context menu is dismissed the instant it opens.
        // It presents as "right-click stopped working everywhere except the
        // desktop", because the desktop's menu is drawn by plasmashell rather
        // than through a grabbing popup.
        self.hide_tooltip();
        self.apps_menu = None;
        self.menu = None;

        let Some(slot) = self.slots.get(slot_idx) else {
            return;
        };
        let Some(text) = self.text.as_ref() else {
            eprintln!("lnp-dock: menu: no font loaded");
            return;
        };
        let Some(seat) = self.seat.clone() else {
            eprintln!("lnp-dock: menu: no seat to grab with");
            return;
        };

        let items = match &slot.kind {
            SlotKind::AppsButton => Vec::new(),
            SlotKind::Launcher(i) => {
                menu::items_for_slot(Some(&self.launchers[*i].spec), None, None)
            }
            SlotKind::App(app_id) => {
                let spec = launchers::spec_for_app_id(app_id);
                let name = spec
                    .as_deref()
                    .and_then(|s| launchers::resolve(s).ok())
                    .map(|l| l.name);
                menu::items_for_slot(None, name.as_deref(), spec)
            }
        };
        if items.is_empty() {
            eprintln!("lnp-dock: menu: nothing to offer for this slot");
            return;
        }

        let model = MenuModel::new(items, |s| text.measure(s, menu::FONT_PX));

        let positioner = match XdgPositioner::new(&self.xdg_shell) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("lnp-dock: positioner: {e}");
                return;
            }
        };
        positioner.set_size(model.width as i32, model.height as i32);
        // Anchor to the slot's rectangle on the dock plate, opening upward;
        // the compositor slides it back on-screen at the dock's ends.
        let slot_left = (self.slot_center(slot_idx) - self.params.slot_width / 2.0) as i32;
        let plate_top = (self.height as f32 - RESTING_HEIGHT) as i32;
        positioner.set_anchor_rect(
            slot_left,
            plate_top,
            self.params.slot_width as i32,
            RESTING_HEIGHT as i32,
        );
        use smithay_client_toolkit::reexports::protocols::xdg::shell::client::xdg_positioner::{
            Anchor as PAnchor, ConstraintAdjustment, Gravity,
        };
        positioner.set_anchor(PAnchor::Top);
        positioner.set_gravity(Gravity::Top);
        positioner.set_constraint_adjustment(ConstraintAdjustment::SlideX);

        let surface = match Surface::new(&self.compositor, qh) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("lnp-dock: menu surface: {e}");
                return;
            }
        };
        let popup = match Popup::from_surface(None, &positioner, qh, surface, &self.xdg_shell) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("lnp-dock: menu popup: {e}");
                return;
            }
        };

        // Parent to the layer surface, take the grab with the click's serial,
        // then the initial commit; the compositor answers with a configure and
        // that is when the menu gets drawn.
        self.layer.get_popup(popup.xdg_popup());
        popup.xdg_popup().grab(&seat, serial);
        popup.wl_surface().commit();

        eprintln!("lnp-dock: menu opened for slot {slot_idx}");
        self.menu = Some(OpenMenu { popup, model });
    }

    fn close_menu(&mut self) {
        self.menu = None;
    }

    fn toggle_apps_menu(&mut self, qh: &QueueHandle<Self>, serial: u32) {
        if self.apps_menu.take().is_some() {
            return; // was open: the take closed it
        }
        self.open_apps_menu(qh, serial);
    }

    fn open_apps_menu(&mut self, qh: &QueueHandle<Self>, serial: u32) {
        // Same rule as open_menu: destroy before create, never overlap grabs.
        self.hide_tooltip();
        self.menu = None;
        self.apps_menu = None;

        if self.text.is_none() {
            eprintln!("lnp-dock: apps menu: no font loaded");
            return;
        }
        let Some(seat) = self.seat.clone() else {
            eprintln!("lnp-dock: apps menu: no seat to grab with");
            return;
        };

        // First open scans the system; after that the catalogue is reused for
        // the life of the process.
        if self.apps_catalog.is_none() {
            let catalog = apps::scan();
            eprintln!("lnp-dock: scanned {} applications", catalog.len());
            self.apps_catalog = Some(catalog);
        }
        let cats = apps::present_categories(self.apps_catalog.as_ref().unwrap());
        let model = AppsMenuModel::new(cats);

        let Ok(positioner) = XdgPositioner::new(&self.xdg_shell) else {
            return;
        };
        positioner.set_size(appsmenu::MENU_W as i32, appsmenu::MENU_H as i32);
        // Anchored over the Apps button (slot 0), sliding right as needed.
        let slot_left = (self.slot_center(0) - self.params.slot_width / 2.0) as i32;
        positioner.set_anchor_rect(slot_left, 0, self.params.slot_width as i32, self.height as i32);
        use smithay_client_toolkit::reexports::protocols::xdg::shell::client::xdg_positioner::{
            Anchor as PAnchor, ConstraintAdjustment, Gravity,
        };
        positioner.set_anchor(PAnchor::Top);
        positioner.set_gravity(Gravity::Top);
        positioner.set_constraint_adjustment(ConstraintAdjustment::SlideX);

        let Ok(surface) = Surface::new(&self.compositor, qh) else {
            return;
        };
        let Ok(popup) = Popup::from_surface(None, &positioner, qh, surface, &self.xdg_shell)
        else {
            return;
        };

        self.layer.get_popup(popup.xdg_popup());
        popup.xdg_popup().grab(&seat, serial);
        popup.wl_surface().commit();

        eprintln!("lnp-dock: apps menu opened");
        self.apps_menu = Some(OpenAppsMenu { popup, model });
    }

    fn draw_apps_menu(&mut self) {
        let started = std::time::Instant::now();
        let Some(am) = self.apps_menu.as_ref() else {
            return;
        };
        let Some(catalog) = self.apps_catalog.as_ref() else {
            return;
        };
        let Some(text) = self.text.as_ref() else {
            return;
        };

        let scale = self.scale.max(1);
        let sf = scale as f32;
        let pw = (appsmenu::MENU_W * sf) as i32;
        let ph = (appsmenu::MENU_H * sf) as i32;

        // Precompute all draw operations before touching the shm pool: the
        // canvas borrow and the icon cache borrow must not overlap the
        // apps_menu borrow.
        let filtered = apps::filtered(catalog, am.model.selected_name());

        struct TileOp {
            x: f32,
            y: f32,
            icon: Option<PathBuf>,
            label: String,
            hovered: bool,
        }
        let mut tiles: Vec<TileOp> = Vec::new();
        for (i, app) in filtered.iter().enumerate() {
            let (x, y) = am.model.tile_origin(i);
            if y + appsmenu::TILE_H < 0.0 || y > appsmenu::MENU_H {
                continue; // culled by scroll
            }
            tiles.push(TileOp {
                x,
                y,
                icon: app.icon.clone(),
                label: tooltip::ellipsize(&app.name, appsmenu::TILE_W - 8.0, |s| {
                    text.measure(s, appsmenu::TILE_FONT)
                }),
                hovered: am.model.hover == Hover::App(i),
            });
        }

        struct CatOp {
            y: f32,
            label: &'static str,
            selected: bool,
            hovered: bool,
        }
        let cats: Vec<CatOp> = am
            .model
            .cats
            .iter()
            .enumerate()
            .map(|(i, c)| CatOp {
                y: appsmenu::RAIL_TOP + i as f32 * appsmenu::RAIL_LINE_H,
                label: c,
                selected: i == am.model.selected_cat,
                hovered: am.model.hover == Hover::Cat(i),
            })
            .collect();

        let Ok((buffer, canvas)) =
            self.pool
                .create_buffer(pw, ph, pw * 4, wl_shm::Format::Argb8888)
        else {
            return;
        };
        let Some(mut pixmap) = Pixmap::new(pw as u32, ph as u32) else {
            return;
        };

        let plate = DockTheme {
            background: tiny_skia::Color::from_rgba8(30, 30, 36, 248),
            corner_radius: 12.0,
            ..self.theme
        };
        if let Some(rect) = Rect::from_xywh(0.0, 0.0, pw as f32, ph as f32) {
            render::draw_background(&mut pixmap, rect, &plate);
        }
        // A faint rail separation.
        if let Some(rect) = Rect::from_xywh(0.0, 0.0, appsmenu::RAIL_W * sf, ph as f32) {
            let rail_bg = DockTheme {
                background: tiny_skia::Color::from_rgba8(255, 255, 255, 10),
                corner_radius: 12.0,
                ..self.theme
            };
            render::draw_background(&mut pixmap, rect, &rail_bg);
        }

        let text = self.text.as_ref().unwrap();
        for cat in &cats {
            if cat.selected || cat.hovered {
                if let Some(rect) = Rect::from_xywh(
                    4.0 * sf,
                    cat.y * sf,
                    (appsmenu::RAIL_W - 8.0) * sf,
                    appsmenu::RAIL_LINE_H * sf,
                ) {
                    let hl = DockTheme {
                        background: tiny_skia::Color::from_rgba8(
                            255,
                            255,
                            255,
                            if cat.selected { 42 } else { 20 },
                        ),
                        corner_radius: 7.0,
                        ..self.theme
                    };
                    render::draw_background(&mut pixmap, rect, &hl);
                }
            }
            text.draw(
                &mut pixmap,
                cat.label,
                14.0 * sf,
                (cat.y + appsmenu::RAIL_LINE_H * 0.68) * sf,
                appsmenu::RAIL_FONT * sf,
                [230, 230, 236, 255],
            );
        }

        for tile in &tiles {
            if tile.hovered {
                if let Some(rect) = Rect::from_xywh(
                    tile.x * sf,
                    tile.y * sf,
                    appsmenu::TILE_W * sf,
                    appsmenu::TILE_H * sf,
                ) {
                    let hl = DockTheme {
                        background: tiny_skia::Color::from_rgba8(255, 255, 255, 26),
                        corner_radius: 9.0,
                        ..self.theme
                    };
                    render::draw_background(&mut pixmap, rect, &hl);
                }
            }
            if let Some(icon) = &tile.icon {
                render::draw_icon(
                    &mut pixmap,
                    &mut self.cache,
                    icon,
                    (tile.x + appsmenu::TILE_W / 2.0) * sf,
                    (tile.y + 8.0 + appsmenu::TILE_ICON) * sf,
                    appsmenu::TILE_ICON * sf,
                );
            }
            let text = self.text.as_ref().unwrap();
            let label_w = text.measure(&tile.label, appsmenu::TILE_FONT);
            text.draw(
                &mut pixmap,
                &tile.label,
                (tile.x + (appsmenu::TILE_W - label_w) / 2.0) * sf,
                (tile.y + appsmenu::TILE_H - 12.0) * sf,
                appsmenu::TILE_FONT * sf,
                [220, 220, 226, 255],
            );
        }

        for (dst, src) in canvas.chunks_exact_mut(4).zip(pixmap.pixels()) {
            dst[0] = src.blue();
            dst[1] = src.green();
            dst[2] = src.red();
            dst[3] = src.alpha();
        }

        let am = self.apps_menu.as_ref().unwrap();
        am.popup.wl_surface().set_buffer_scale(scale);
        am.popup.wl_surface().damage_buffer(0, 0, pw, ph);
        if buffer.attach_to(am.popup.wl_surface()).is_ok() {
            am.popup.wl_surface().commit();
        }

        if std::env::var_os("LNP_DOCK_DEBUG").is_some() {
            // A scroll produces events at the pointer's report rate, so any
            // per-frame cost above a few milliseconds turns into visible lag.
            eprintln!(
                "lnp-dock: apps menu draw took {:.1} ms",
                started.elapsed().as_secs_f32() * 1000.0
            );
        }
    }

    /// The tooltip's text for a slot: display name, then window titles.
    fn tooltip_lines(&self, slot: &Slot) -> Vec<String> {
        let name = match &slot.kind {
            SlotKind::AppsButton => "Applications".to_string(),
            SlotKind::Launcher(i) => self.launchers[*i].name.clone(),
            SlotKind::App(app_id) => launchers::spec_for_app_id(app_id)
                .and_then(|s| launchers::resolve(&s).ok())
                .map(|l| l.name)
                .unwrap_or_else(|| app_id.clone()),
        };
        let titles: Vec<String> = slot
            .windows
            .iter()
            .filter_map(|u| self.windows.get(u))
            .map(|w| w.title.clone())
            .collect();
        tooltip::compose(&name, &titles)
    }

    fn show_tooltip(&mut self, qh: &QueueHandle<Self>, slot_idx: usize) {
        // A grab-holding menu outranks a tooltip.
        if self.menu.is_some() {
            return;
        }
        let Some(slot) = self.slots.get(slot_idx) else {
            return;
        };
        let Some(text) = self.text.as_ref() else {
            return;
        };

        let model = tooltip::TooltipModel::new(self.tooltip_lines(slot), |s| {
            text.measure(s, tooltip::FONT_PX)
        });

        let Ok(positioner) = XdgPositioner::new(&self.xdg_shell) else {
            return;
        };
        positioner.set_size(model.width as i32, model.height as i32);
        // Anchor the full surface column of the slot, so the tooltip clears
        // even a fully magnified icon.
        let slot_left = (self.slot_center(slot_idx) - self.params.slot_width / 2.0) as i32;
        positioner.set_anchor_rect(
            slot_left,
            0,
            self.params.slot_width as i32,
            self.height as i32,
        );
        use smithay_client_toolkit::reexports::protocols::xdg::shell::client::xdg_positioner::{
            Anchor as PAnchor, ConstraintAdjustment, Gravity,
        };
        positioner.set_anchor(PAnchor::Top);
        positioner.set_gravity(Gravity::Top);
        positioner.set_constraint_adjustment(ConstraintAdjustment::SlideX);

        let Ok(surface) = Surface::new(&self.compositor, qh) else {
            return;
        };
        let Ok(popup) = Popup::from_surface(None, &positioner, qh, surface, &self.xdg_shell)
        else {
            return;
        };

        // A tooltip must never take input: an empty input region makes the
        // pointer see straight through it.
        if let Ok(region) = Region::new(&self.compositor) {
            popup.wl_surface().set_input_region(Some(region.wl_region()));
        }

        self.layer.get_popup(popup.xdg_popup());
        popup.wl_surface().commit();

        self.tooltip = Some(OpenTooltip {
            popup,
            model,
            slot: slot_idx,
        });
    }

    fn hide_tooltip(&mut self) {
        // Dropping the popup destroys its protocol objects (SCTK Drop impls),
        // which unmaps the surface.
        self.tooltip = None;
    }

    fn draw_tooltip(&mut self) {
        let Some(tip) = self.tooltip.as_ref() else {
            return;
        };
        let Some(text) = self.text.as_ref() else {
            return;
        };

        let scale = self.scale.max(1);
        let sf = scale as f32;
        let pw = (tip.model.width * sf) as i32;
        let ph = (tip.model.height * sf) as i32;

        let Ok((buffer, canvas)) =
            self.pool
                .create_buffer(pw, ph, pw * 4, wl_shm::Format::Argb8888)
        else {
            return;
        };
        let Some(mut pixmap) = Pixmap::new(pw as u32, ph as u32) else {
            return;
        };

        let plate = DockTheme {
            background: tiny_skia::Color::from_rgba8(32, 32, 38, 245),
            corner_radius: 9.0,
            ..self.theme
        };
        if let Some(rect) = Rect::from_xywh(0.0, 0.0, pw as f32, ph as f32) {
            render::draw_background(&mut pixmap, rect, &plate);
        }

        for (i, line) in tip.model.lines.iter().enumerate() {
            let baseline =
                (tooltip::VPAD + i as f32 * tooltip::LINE_H + tooltip::LINE_H * 0.72) * sf;
            // The name line full-bright, title lines slightly dimmed.
            let color = if i == 0 {
                [235, 235, 240, 255]
            } else {
                [200, 200, 208, 255]
            };
            text.draw(
                &mut pixmap,
                line,
                tooltip::HPAD * sf,
                baseline,
                tooltip::FONT_PX * sf,
                color,
            );
        }

        for (dst, src) in canvas.chunks_exact_mut(4).zip(pixmap.pixels()) {
            dst[0] = src.blue();
            dst[1] = src.green();
            dst[2] = src.red();
            dst[3] = src.alpha();
        }

        let tip = self.tooltip.as_ref().unwrap();
        tip.popup.wl_surface().set_buffer_scale(scale);
        tip.popup.wl_surface().damage_buffer(0, 0, pw, ph);
        if buffer.attach_to(tip.popup.wl_surface()).is_ok() {
            tip.popup.wl_surface().commit();
        }
    }

    fn draw_menu(&mut self) {
        let Some(menu_ref) = self.menu.as_ref() else {
            return;
        };
        let Some(text) = self.text.as_ref() else {
            return;
        };

        let scale = self.scale.max(1);
        let sf = scale as f32;
        let pw = (menu_ref.model.width * sf) as i32;
        let ph = (menu_ref.model.height * sf) as i32;

        let Ok((buffer, canvas)) =
            self.pool
                .create_buffer(pw, ph, pw * 4, wl_shm::Format::Argb8888)
        else {
            return;
        };
        let Some(mut pixmap) = Pixmap::new(pw as u32, ph as u32) else {
            return;
        };

        // Opaque-ish plate, same family as the dock but readable at any
        // wallpaper.
        let plate = DockTheme {
            background: tiny_skia::Color::from_rgba8(32, 32, 38, 245),
            corner_radius: 10.0,
            ..self.theme
        };
        if let Some(rect) = Rect::from_xywh(0.0, 0.0, pw as f32, ph as f32) {
            render::draw_background(&mut pixmap, rect, &plate);
        }

        for (i, item) in menu_ref.model.items.iter().enumerate() {
            let row_top = (menu::VPAD + i as f32 * menu::ROW_H) * sf;

            if menu_ref.model.hover == Some(i) {
                if let Some(rect) =
                    Rect::from_xywh(3.0 * sf, row_top, pw as f32 - 6.0 * sf, menu::ROW_H * sf)
                {
                    let hl = DockTheme {
                        background: tiny_skia::Color::from_rgba8(255, 255, 255, 34),
                        corner_radius: 7.0,
                        ..self.theme
                    };
                    render::draw_background(&mut pixmap, rect, &hl);
                }
            }

            let baseline = row_top + menu::ROW_H * 0.68 * sf;
            text.draw(
                &mut pixmap,
                &item.label,
                menu::HPAD * sf,
                baseline,
                menu::FONT_PX * sf,
                [235, 235, 240, 255],
            );
        }

        for (dst, src) in canvas.chunks_exact_mut(4).zip(pixmap.pixels()) {
            dst[0] = src.blue();
            dst[1] = src.green();
            dst[2] = src.red();
            dst[3] = src.alpha();
        }

        let menu_ref = self.menu.as_ref().unwrap();
        menu_ref.popup.wl_surface().set_buffer_scale(scale);
        menu_ref.popup.wl_surface().damage_buffer(0, 0, pw, ph);
        if let Err(e) = buffer.attach_to(menu_ref.popup.wl_surface()) {
            eprintln!("lnp-dock: menu attach: {e}");
            return;
        }
        menu_ref.popup.wl_surface().commit();
    }

    /// Which pinned (launcher) position a drag at `x` points into.
    fn drag_target(&self, x: f32) -> usize {
        let count = self.launchers.len().max(1);
        (((x - PADDING) / self.params.slot_width).floor() as isize - PIN_SLOT_OFFSET as isize)
            .clamp(0, count as isize - 1) as usize
    }

    /// Finish a drag: persist the new order and rebuild from it. `from_slot`
    /// is the slot index the drag started on.
    fn commit_reorder(&mut self, qh: &QueueHandle<Self>, from_slot: usize, x: f32) {
        let count = self.launchers.len();
        let Some(from) = from_slot.checked_sub(PIN_SLOT_OFFSET) else {
            return;
        };
        if from >= count {
            return;
        }
        let to = self.drag_target(x);
        if to == from {
            return;
        }

        let order = pins::preview_order(count, from, to);
        let displayed: Vec<String> = self.launchers.iter().map(|l| l.spec.clone()).collect();
        let new_displayed: Vec<String> = order.iter().map(|&i| displayed[i].clone()).collect();

        self.pin_specs = pins::reorder_interleaved(&self.pin_specs, &displayed, &new_displayed);
        self.reload_pins(qh);
    }

    /// Re-resolve launchers from the pin list and refresh everything.
    fn reload_pins(&mut self, qh: &QueueHandle<Self>) {
        pins::save(&self.pin_specs);
        self.launchers = resolve_pins(&self.pin_specs);
        self.window_model_changed(qh);
    }

    fn apply_menu_action(&mut self, qh: &QueueHandle<Self>, action: MenuAction) {
        match action {
            MenuAction::Pin(spec) => {
                if !self.pin_specs.contains(&spec) {
                    self.pin_specs.push(spec);
                    self.reload_pins(qh);
                }
            }
            MenuAction::Unpin(spec) => {
                self.pin_specs.retain(|s| *s != spec);
                self.reload_pins(qh);
            }
        }
    }

    fn launch(&self, index: usize) {
        if let Some(item) = self.launchers.get(index) {
            spawn_detached(&item.exec, &item.name);
        }
    }

    fn draw(&mut self, qh: &QueueHandle<Self>) {
        let scale = self.scale.max(1);
        let pw = self.width as i32 * scale;
        let ph = self.height as i32 * scale;
        let stride = pw * 4;
        let sf = scale as f32;

        let baseline = (self.height as f32 - PADDING) * sf;
        let damping = self.damping.damping();

        // During an active drag the pinned slots render in preview order --
        // the same preview_order() the commit uses, so what the user sees is
        // exactly what release will save -- and magnification rests, because
        // targets must not move while one of them is in the user's hand.
        let dragging: Option<(usize, f32)> = self
            .drag
            .as_ref()
            .filter(|d| d.active)
            .map(|d| (d.slot_idx, d.current_x));

        let npinned = self.launchers.len();
        let (display, gap_pos): (Vec<usize>, Option<usize>) = match dragging {
            Some((from_slot, x)) => {
                // Drags only ever arm on Launcher slots, so this cannot wrap.
                let from = from_slot - PIN_SLOT_OFFSET;
                let to = self.drag_target(x);
                let mut order = vec![0usize]; // the Apps button stays put
                order.extend(
                    pins::preview_order(npinned, from, to)
                        .into_iter()
                        .map(|i| i + PIN_SLOT_OFFSET),
                );
                order.extend(PIN_SLOT_OFFSET + npinned..self.slots.len());
                (order, Some(to + PIN_SLOT_OFFSET))
            }
            None => ((0..self.slots.len()).collect(), None),
        };

        let mag_pointer = if dragging.is_some() { None } else { self.pointer_x };

        // Work out where every icon goes *before* borrowing the shm pool. The
        // canvas borrow lives until commit, and the layout needs &self.
        // (position, size, icon, window count, leave-a-gap)
        let layout: Vec<(f32, f32, Option<PathBuf>, usize, bool)> = display
            .iter()
            .enumerate()
            .map(|(pos, &si)| {
                let center = self.slot_center(pos);
                let s = self.params.scale_at(center, mag_pointer, damping);
                (
                    center * sf,
                    BASE_ICON * s * sf,
                    self.slots[si].icon.clone(),
                    self.slots[si].windows.len(),
                    gap_pos == Some(pos),
                )
            })
            .collect();

        // The icon in hand, drawn on top at the pointer.
        let held: Option<(f32, Option<PathBuf>)> =
            dragging.map(|(from, x)| (x * sf, self.slots[from].icon.clone()));

        let bg_rect = Rect::from_xywh(
            0.0,
            (self.height as f32 - RESTING_HEIGHT) * sf,
            self.width as f32 * sf,
            RESTING_HEIGHT * sf,
        );

        let Ok((buffer, canvas)) =
            self.pool
                .create_buffer(pw, ph, stride, wl_shm::Format::Argb8888)
        else {
            eprintln!("lnp-dock: could not allocate a buffer");
            return;
        };

        // Paint into a tiny-skia pixmap, then blit into the shm canvas.
        let Some(mut pixmap) = Pixmap::new(pw as u32, ph as u32) else {
            return;
        };

        // Background plate covers only the resting height, sitting at the
        // bottom of the taller surface.
        if let Some(rect) = bg_rect {
            render::draw_background(&mut pixmap, rect, &self.theme);
        }

        let dot_y = (self.height as f32 - PADDING * 0.45) * sf;
        for (center, drawn, icon, nwin, is_gap) in &layout {
            if *is_gap {
                continue; // the dragged icon's landing gap
            }
            if let Some(icon) = icon {
                render::draw_icon(&mut pixmap, &mut self.cache, icon, *center, baseline, *drawn);
            }
            // Running indicator: one dot per window, capped at three so a
            // browser with forty windows does not grow a caterpillar.
            let dots = (*nwin).min(3);
            for d in 0..dots {
                let offset = (d as f32 - (dots as f32 - 1.0) / 2.0) * 7.0 * sf;
                render::draw_running_indicator(&mut pixmap, center + offset, dot_y, &self.theme);
            }
        }

        // The held icon rides the pointer, slightly enlarged and lifted.
        if let Some((hx, Some(icon))) = held {
            render::draw_icon(
                &mut pixmap,
                &mut self.cache,
                &icon,
                hx,
                baseline - 4.0 * sf,
                BASE_ICON * 1.12 * sf,
            );
        }

        // tiny-skia is RGBA premultiplied; wl_shm Argb8888 on little-endian is
        // BGRA in memory. Swap R and B on the way out.
        for (dst, src) in canvas.chunks_exact_mut(4).zip(pixmap.pixels()) {
            dst[0] = src.blue();
            dst[1] = src.green();
            dst[2] = src.red();
            dst[3] = src.alpha();
        }

        self.layer.wl_surface().set_buffer_scale(scale);
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, pw, ph);

        // Keep animating only while the pointer is over us.
        if self.animating {
            self.layer
                .wl_surface()
                .frame(qh, FrameCallbackData(self.layer.wl_surface().clone()));
        }

        if let Err(e) = buffer.attach_to(self.layer.wl_surface()) {
            eprintln!("lnp-dock: attach failed: {e}");
            return;
        }
        self.layer.commit();
    }
}

impl CompositorHandler for Dock {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        new_factor: i32,
    ) {
        if new_factor != self.scale {
            self.scale = new_factor.max(1);
            self.draw(qh);
        }
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        // No motion for a moment means the pointer has arrived. This is the
        // case where the user most wants the target large, so let damping
        // recover toward full magnification.
        if self.pointer_x.is_some() && self.last_motion.elapsed() >= SETTLE_AFTER {
            self.damping.settle(Instant::now());
        }

        // A pointer that has rested on a slot long enough earns a tooltip.
        // The frame callbacks that drive magnification double as the timer.
        if self.menu.is_none()
            && self.drag.is_none()
            && self.last_motion.elapsed() >= tooltip::SHOW_DELAY
        {
            if let Some(idx) = self.pointer_x.and_then(|x| self.slot_at(x)) {
                if self.tooltip.as_ref().map(|t| t.slot) != Some(idx) {
                    self.show_tooltip(qh, idx);
                }
            }
        }

        self.draw(qh);
    }

    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform,
    ) {
    }

    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for Dock {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        // A zero dimension means "you choose", so keep ours.
        if configure.new_size.0 != 0 {
            self.width = configure.new_size.0;
        }
        if configure.new_size.1 != 0 {
            self.height = configure.new_size.1;
        }

        if self.first_configure {
            self.first_configure = false;
        }
        self.draw(qh);
    }
}

impl SeatHandler for Dock {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, seat: wl_seat::WlSeat) {
        // Kept for the menu popup grab.
        self.seat = Some(seat);
    }

    // (new_capability below also stores the seat: for seats that already
    // existed at startup, new_capability is the callback that reliably fires.)

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_none() {
            match self.seat_state.get_pointer(qh, &seat) {
                Ok(p) => self.pointer = Some(p),
                Err(e) => eprintln!("lnp-dock: no pointer: {e}"),
            }
        }
        // A grabbing popup receives keyboard focus even though the layer
        // surface itself asks for none, so we need a keyboard object to be
        // told when that focus goes away.
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            match self.seat_state.get_keyboard(qh, &seat, None) {
                Ok(k) => self.keyboard = Some(k),
                Err(e) => eprintln!("lnp-dock: no keyboard: {e}"),
            }
        }
        if self.seat.is_none() {
            self.seat = Some(seat);
        }
    }

    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer {
            if let Some(p) = self.pointer.take() {
                p.release();
            }
        }
        if capability == Capability::Keyboard {
            if let Some(k) = self.keyboard.take() {
                k.release();
            }
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl PointerHandler for Dock {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            // Apps menu events: rail selection, tile hover/launch, scrolling.
            if self
                .apps_menu
                .as_ref()
                .is_some_and(|am| &event.surface == am.popup.wl_surface())
            {
                // Decide with borrows, act after releasing them.
                enum Act {
                    Nothing,
                    Redraw,
                    Launch(String, String),
                }
                let mut act = Act::Nothing;

                {
                    let catalog = self.apps_catalog.as_deref().unwrap_or(&[]);
                    let am = self.apps_menu.as_mut().unwrap();
                    let count = apps::filtered(catalog, am.model.selected_name()).len();
                    let (x, y) = (event.position.0 as f32, event.position.1 as f32);

                    match event.kind {
                        PointerEventKind::Motion { .. } | PointerEventKind::Enter { .. } => {
                            let hover = if let Some(c) = am.model.cat_at(x, y) {
                                Hover::Cat(c)
                            } else if let Some(t) = am.model.tile_at(x, y, count) {
                                Hover::App(t)
                            } else {
                                Hover::None
                            };
                            if hover != am.model.hover {
                                am.model.hover = hover;
                                act = Act::Redraw;
                            }
                        }
                        PointerEventKind::Leave { .. } => {
                            if am.model.hover != Hover::None {
                                am.model.hover = Hover::None;
                                act = Act::Redraw;
                            }
                        }
                        PointerEventKind::Axis { vertical, .. } => {
                            // Scroll arrives three different ways depending on
                            // the device and the compositor:
                            //
                            //   absolute  pixels -- touchpads, and wheels on
                            //             compositors that translate for us
                            //   value120  high-resolution wheels, where 120
                            //             is one logical notch
                            //   discrete  the deprecated step count, older
                            //             compositors only
                            //
                            // Reading `absolute` alone silently does nothing
                            // on hardware that only reports one of the others,
                            // which is a scroll that looks broken rather than
                            // one that errors. Take whichever is populated.
                            const NOTCH_PX: f32 = 60.0;
                            let delta = if vertical.absolute != 0.0 {
                                vertical.absolute as f32
                            } else if vertical.value120 != 0 {
                                vertical.value120 as f32 / 120.0 * NOTCH_PX
                            } else {
                                vertical.discrete as f32 * NOTCH_PX
                            };

                            if std::env::var_os("LNP_DOCK_DEBUG").is_some() {
                                eprintln!(
                                    "lnp-dock: axis abs={} v120={} disc={} -> delta={:.1} \
                                     scroll={:.1} max={:.1} count={}",
                                    vertical.absolute,
                                    vertical.value120,
                                    vertical.discrete,
                                    delta,
                                    am.model.scroll,
                                    am.model.max_scroll(count),
                                    count
                                );
                            }

                            let before = am.model.scroll;
                            am.model.scroll_by(delta * appsmenu::SCROLL_MULTIPLIER, count);
                            if am.model.scroll != before {
                                act = Act::Redraw;
                            }
                        }
                        PointerEventKind::Press { button, .. } if button == 0x110 => {
                            if let Some(c) = am.model.cat_at(x, y) {
                                am.model.select_cat(c);
                                act = Act::Redraw;
                            } else if let Some(t) = am.model.tile_at(x, y, count) {
                                let filtered =
                                    apps::filtered(catalog, am.model.selected_name());
                                if let Some(app) = filtered.get(t) {
                                    act = Act::Launch(app.exec.clone(), app.name.clone());
                                }
                            }
                        }
                        _ => {}
                    }
                }

                match act {
                    Act::Nothing => {}
                    Act::Redraw => self.draw_apps_menu(),
                    Act::Launch(exec, name) => {
                        spawn_detached(&exec, &name);
                        self.apps_menu = None;
                    }
                }
                continue;
            }

            // Menu popup events: hover tracking and item activation.
            if let Some(menu_open) = self.menu.as_mut() {
                if &event.surface == menu_open.popup.wl_surface() {
                    match event.kind {
                        PointerEventKind::Motion { .. } | PointerEventKind::Enter { .. } => {
                            let row = menu_open
                                .model
                                .row_at(event.position.0 as f32, event.position.1 as f32);
                            if row != menu_open.model.hover {
                                menu_open.model.hover = row;
                                self.draw_menu();
                            }
                        }
                        PointerEventKind::Leave { .. } => {
                            if menu_open.model.hover.is_some() {
                                menu_open.model.hover = None;
                                self.draw_menu();
                            }
                        }
                        PointerEventKind::Press { button, .. } if button == 0x110 => {
                            let action = menu_open
                                .model
                                .row_at(event.position.0 as f32, event.position.1 as f32)
                                .map(|row| menu_open.model.items[row].action.clone());
                            self.close_menu();
                            if let Some(action) = action {
                                self.apply_menu_action(qh, action);
                            }
                        }
                        _ => {}
                    }
                    continue;
                }
            }

            if &event.surface != self.layer.wl_surface() {
                continue;
            }

            match event.kind {
                PointerEventKind::Enter { .. } => {
                    self.damping.reset();
                    self.pointer_x = Some(event.position.0 as f32);
                    self.last_motion = Instant::now();
                    if !self.animating {
                        self.animating = true;
                        self.draw(qh);
                    }
                }
                PointerEventKind::Motion { .. } => {
                    let x = event.position.0 as f32;
                    let now = Instant::now();
                    self.damping.update(x, now);
                    self.pointer_x = Some(x);
                    self.last_motion = now;

                    // A held button turns into a drag once it has clearly
                    // moved; below the threshold it is still a click. Only
                    // pinned (Launcher) slots are draggable.
                    let held_is_pinned = self
                        .drag
                        .as_ref()
                        .map(|d| {
                            matches!(
                                self.slots.get(d.slot_idx).map(|s| &s.kind),
                                Some(SlotKind::Launcher(_))
                            )
                        })
                        .unwrap_or(false);
                    let mut drag_became_active = false;
                    if let Some(d) = self.drag.as_mut() {
                        d.current_x = x;
                        if !d.active
                            && (x - d.press_x).abs() > DRAG_THRESHOLD
                            && held_is_pinned
                        {
                            d.active = true;
                            drag_became_active = true;
                        }
                    }
                    if drag_became_active {
                        self.hide_tooltip();
                    }

                    // Hot tracking: while a tooltip is up, sliding along the
                    // dock switches it instantly rather than re-waiting.
                    if self.tooltip.is_some() && self.drag.as_ref().is_none_or(|d| !d.active) {
                        match self.slot_at(x) {
                            Some(idx) if self.tooltip.as_ref().map(|t| t.slot) != Some(idx) => {
                                self.hide_tooltip();
                                self.show_tooltip(qh, idx);
                            }
                            None => self.hide_tooltip(),
                            _ => {}
                        }
                    }
                }
                PointerEventKind::Leave { .. } => {
                    self.pointer_x = None;
                    self.damping.reset();
                    self.animating = false;
                    self.hide_tooltip();
                    // The implicit grab keeps a held drag alive off-surface;
                    // an actual Leave means the button is up. Abandon, don't
                    // guess an order the user never confirmed.
                    self.drag = None;
                    // One last paint to settle everything back to rest.
                    self.draw(qh);
                }
                PointerEventKind::Press { button, serial, .. } => {
                    self.hide_tooltip();
                    match button {
                        // BTN_LEFT: the Apps button opens its menu on press
                        // (menus open on press everywhere); other slots arm a
                        // click-or-drag resolved on release.
                        0x110 => {
                            let x = event.position.0 as f32;
                            if let Some(idx) = self.slot_at(x) {
                                if matches!(self.slots[idx].kind, SlotKind::AppsButton) {
                                    self.toggle_apps_menu(qh, serial);
                                } else {
                                    self.drag = Some(DragState {
                                        slot_idx: idx,
                                        press_x: x,
                                        current_x: x,
                                        active: false,
                                    });
                                }
                            }
                        }
                        // BTN_RIGHT
                        0x111 => {
                            match self.slot_at(event.position.0 as f32) {
                                Some(idx) => self.open_menu(qh, idx, serial),
                                None => eprintln!(
                                    "lnp-dock: right-click at x={} hit no slot",
                                    event.position.0
                                ),
                            }
                        }
                        _ => {}
                    }
                }
                PointerEventKind::Release { button, .. } => {
                    if button == 0x110 {
                        if let Some(d) = self.drag.take() {
                            if d.active {
                                self.commit_reorder(qh, d.slot_idx, d.current_x);
                            } else {
                                self.activate_slot(d.slot_idx);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

// Required by XdgShell::bind even though the dock never creates xdg windows;
// the shell is bound purely to parent the right-click menu popups.
impl smithay_client_toolkit::shell::xdg::window::WindowHandler for Dock {
    fn request_close(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &smithay_client_toolkit::shell::xdg::window::Window,
    ) {
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &smithay_client_toolkit::shell::xdg::window::Window,
        _: smithay_client_toolkit::shell::xdg::window::WindowConfigure,
        _: u32,
    ) {
    }
}

impl PopupHandler for Dock {
    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        popup: &Popup,
        _config: PopupConfigure,
    ) {
        let is_menu = self
            .menu
            .as_ref()
            .is_some_and(|m| m.popup.wl_surface() == popup.wl_surface());
        if is_menu {
            // First configure: the compositor has agreed on placement, now the
            // menu may draw.
            eprintln!("lnp-dock: menu configured; drawing");
            self.draw_menu();
            return;
        }

        let is_tooltip = self
            .tooltip
            .as_ref()
            .is_some_and(|t| t.popup.wl_surface() == popup.wl_surface());
        if is_tooltip {
            self.draw_tooltip();
            return;
        }

        if self
            .apps_menu
            .as_ref()
            .is_some_and(|a| a.popup.wl_surface() == popup.wl_surface())
        {
            eprintln!("lnp-dock: apps menu configured; drawing");
            self.draw_apps_menu();
        }
    }

    fn done(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, popup: &Popup) {
        // Compositor dismissed the grab (clicked elsewhere, Escape, ...).
        let is_menu = self
            .menu
            .as_ref()
            .is_some_and(|m| m.popup.wl_surface() == popup.wl_surface());
        if is_menu {
            eprintln!("lnp-dock: menu dismissed by compositor");
            self.close_menu();
            return;
        }

        if self
            .tooltip
            .as_ref()
            .is_some_and(|t| t.popup.wl_surface() == popup.wl_surface())
        {
            self.hide_tooltip();
            return;
        }

        if self
            .apps_menu
            .as_ref()
            .is_some_and(|a| a.popup.wl_surface() == popup.wl_surface())
        {
            eprintln!("lnp-dock: apps menu dismissed by compositor");
            self.apps_menu = None;
        }
    }
}

/// Menus close when they stop being the focused thing.
///
/// A popup with a grab takes keyboard focus, and the compositor moves that
/// focus away when something else needs the input -- which is exactly what
/// happens when a screenshot tool opens its region-select overlay. Without
/// this, the dock kept its grab, the overlay never received the drag, and
/// selecting a region silently did nothing while a dock menu was open.
///
/// Holding a grab until the user explicitly dismisses it is antisocial: a
/// menu is not more important than whatever the user just reached for.
impl KeyboardHandler for Dock {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &smithay_client_toolkit::reexports::client::protocol::wl_keyboard::WlKeyboard,
        _: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _: &[Keysym],
    ) {
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &smithay_client_toolkit::reexports::client::protocol::wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        let ours = self
            .menu
            .as_ref()
            .is_some_and(|m| m.popup.wl_surface() == surface)
            || self
                .apps_menu
                .as_ref()
                .is_some_and(|a| a.popup.wl_surface() == surface);

        if ours {
            eprintln!("lnp-dock: menu lost keyboard focus; releasing the grab");
            self.menu = None;
            self.apps_menu = None;
        }
    }

    fn press_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &smithay_client_toolkit::reexports::client::protocol::wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        // Escape closes the menu, as it does everywhere else.
        if event.keysym == Keysym::Escape {
            self.menu = None;
            self.apps_menu = None;
        }
    }

    fn repeat_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &smithay_client_toolkit::reexports::client::protocol::wl_keyboard::WlKeyboard,
        _: u32,
        _: KeyEvent,
    ) {
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &smithay_client_toolkit::reexports::client::protocol::wl_keyboard::WlKeyboard,
        _: u32,
        _: KeyEvent,
    ) {
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &smithay_client_toolkit::reexports::client::protocol::wl_keyboard::WlKeyboard,
        _: u32,
        _: Modifiers,
        _: RawModifiers,
        _: u32,
    ) {
    }
}

impl ShmHandler for Dock {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl OutputHandler for Dock {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl ProvidesRegistryState for Dock {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

delegate_registry!(Dock);
// SCTK 0.21 collapsed the per-protocol delegate_* macros into one.
delegate_dispatch2!(Dock);
