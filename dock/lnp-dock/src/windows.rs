//! Live window tracking over org_kde_plasma_window_management.
//!
//! KWin implements this protocol but only reveals it to clients it trusts: the
//! client's binary must match the Exec of an installed desktop file carrying
//! X-KDE-Wayland-Interfaces=org_kde_plasma_window_management. That is why the
//! package ships lnp-dock.desktop -- it is not a menu entry (NoDisplay=true),
//! it is the dock's credential. Without it the registry simply never contains
//! the global, and the dock degrades to launchers-only.
//!
//! This is the same protocol Plasma's own task manager uses, so we get exactly
//! what it gets: mapped/unmapped events, app ids, titles, state bits, and the
//! right to activate, minimize and close windows -- no polling, no bridge
//! process, no scraping.

use std::collections::HashMap;

use smithay_client_toolkit::dispatch2::Dispatch2;
use smithay_client_toolkit::reexports::client::{Connection, QueueHandle};
use wayland_protocols_plasma::plasma_window_management::client::{
    org_kde_plasma_window::{self, OrgKdePlasmaWindow},
    org_kde_plasma_window_management::{self, OrgKdePlasmaWindowManagement},
};

use crate::Dock;

// State bits from the protocol's org_kde_plasma_window_management.state enum.
pub const STATE_ACTIVE: u32 = 0x1;
pub const STATE_MINIMIZED: u32 = 0x2;
pub const STATE_SKIPTASKBAR: u32 = 0x1000;

/// One mapped window, as the compositor describes it.
#[derive(Debug)]
pub struct WindowInfo {
    pub proxy: OrgKdePlasmaWindow,
    pub app_id: String,
    pub title: String,
    pub state: u32,
    pub themed_icon: Option<String>,
    /// Set once initial_state arrives. Until then the window is half-described
    /// (often with an empty app_id) and must not be shown.
    pub ready: bool,
}

impl WindowInfo {
    pub fn is_active(&self) -> bool {
        self.state & STATE_ACTIVE != 0
    }

    pub fn skips_taskbar(&self) -> bool {
        self.state & STATE_SKIPTASKBAR != 0
    }

    /// Raise and focus. The compositor unminimizes as part of activation.
    pub fn activate(&self) {
        self.proxy.set_state(STATE_ACTIVE, STATE_ACTIVE);
    }

    pub fn minimize(&self) {
        self.proxy.set_state(STATE_MINIMIZED, STATE_MINIMIZED);
    }
}

/// Windows keyed by their compositor-assigned uuid, in mapping order.
///
/// Mapping order (not a HashMap's arbitrary order) is what keeps slot layout
/// stable: a dock whose icons trade places between redraws would be the moving
/// target problem all over again, this time without even hovering.
#[derive(Default)]
pub struct WindowSet {
    by_uuid: HashMap<String, WindowInfo>,
    order: Vec<String>,
}

impl WindowSet {
    pub fn insert(&mut self, uuid: String, info: WindowInfo) {
        if !self.by_uuid.contains_key(&uuid) {
            self.order.push(uuid.clone());
        }
        self.by_uuid.insert(uuid, info);
    }

    pub fn remove(&mut self, uuid: &str) -> Option<WindowInfo> {
        self.order.retain(|u| u != uuid);
        self.by_uuid.remove(uuid)
    }

    pub fn get_mut(&mut self, uuid: &str) -> Option<&mut WindowInfo> {
        self.by_uuid.get_mut(uuid)
    }

    pub fn get(&self, uuid: &str) -> Option<&WindowInfo> {
        self.by_uuid.get(uuid)
    }

    /// Ready, taskbar-visible windows in mapping order.
    pub fn visible(&self) -> impl Iterator<Item = (&String, &WindowInfo)> {
        self.order.iter().filter_map(|u| {
            let w = self.by_uuid.get(u)?;
            (w.ready && !w.skips_taskbar()).then_some((u, w))
        })
    }
}

/// User data for the manager global.
pub struct WindowManagementData;

impl Dispatch2<OrgKdePlasmaWindowManagement, Dock> for WindowManagementData {
    fn event(
        &self,
        state: &mut Dock,
        _proxy: &OrgKdePlasmaWindowManagement,
        event: org_kde_plasma_window_management::Event,
        _conn: &Connection,
        qh: &QueueHandle<Dock>,
    ) {
        // Only the uuid variant creates windows. The numeric-id `Window` event
        // is emitted alongside it for backwards compatibility; handling both
        // would double every window.
        if let org_kde_plasma_window_management::Event::WindowWithUuid { uuid, .. } = event {
            let proxy = state.window_manager.as_ref().expect("manager set before events");
            let window = proxy.get_window_by_uuid(
                uuid.clone(),
                qh,
                PlasmaWindowData { uuid: uuid.clone() },
            );
            state.windows.insert(
                uuid,
                WindowInfo {
                    proxy: window,
                    app_id: String::new(),
                    title: String::new(),
                    state: 0,
                    themed_icon: None,
                    ready: false,
                },
            );
        }
    }
}

/// User data for each org_kde_plasma_window: remembers which uuid it is, since
/// the event handler is otherwise only handed the proxy.
pub struct PlasmaWindowData {
    pub uuid: String,
}

impl Dispatch2<OrgKdePlasmaWindow, Dock> for PlasmaWindowData {
    fn event(
        &self,
        state: &mut Dock,
        _proxy: &OrgKdePlasmaWindow,
        event: org_kde_plasma_window::Event,
        _conn: &Connection,
        qh: &QueueHandle<Dock>,
    ) {
        use org_kde_plasma_window::Event;

        let uuid = self.uuid.as_str();

        match event {
            Event::AppIdChanged { app_id } => {
                if let Some(w) = state.windows.get_mut(uuid) {
                    w.app_id = app_id;
                }
                state.window_model_changed(qh);
            }
            Event::TitleChanged { title } => {
                if let Some(w) = state.windows.get_mut(uuid) {
                    w.title = title;
                }
            }
            Event::StateChanged { flags } => {
                if let Some(w) = state.windows.get_mut(uuid) {
                    w.state = flags;
                }
                // Active/minimized drive the indicators; skiptaskbar drives
                // whether the window appears at all.
                state.window_model_changed(qh);
            }
            Event::ThemedIconNameChanged { name } => {
                if let Some(w) = state.windows.get_mut(uuid) {
                    w.themed_icon = (!name.is_empty()).then_some(name);
                }
            }
            Event::InitialState => {
                if let Some(w) = state.windows.get_mut(uuid) {
                    w.ready = true;
                }
                state.window_model_changed(qh);
            }
            Event::Unmapped => {
                if let Some(w) = state.windows.remove(uuid) {
                    // The protocol requires the destructor after unmap.
                    w.proxy.destroy();
                }
                state.window_model_changed(qh);
            }
            _ => {}
        }
    }
}
