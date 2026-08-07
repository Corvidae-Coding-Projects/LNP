//! The right-click menu model: items, geometry, hit-testing.
//!
//! Kept free of Wayland types so the layout math is testable. The surface
//! plumbing (xdg_popup, grab, drawing) lives with the rest of it in main.rs.

/// Logical metrics. Row height comfortably above the 24px minimum hit target;
/// the font sits at 15px inside it.
pub const ROW_H: f32 = 32.0;
pub const HPAD: f32 = 16.0;
pub const VPAD: f32 = 6.0;
pub const FONT_PX: f32 = 15.0;

#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    /// Append this spec to the pins.
    Pin(String),
    /// Remove this spec from the pins.
    Unpin(String),
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: MenuAction,
}

/// Everything about an open menu except its Wayland surface.
#[derive(Debug)]
pub struct MenuModel {
    pub items: Vec<MenuItem>,
    /// Logical size, computed from the widest label.
    pub width: f32,
    pub height: f32,
    pub hover: Option<usize>,
}

impl MenuModel {
    /// `measure` maps a label to its rendered width at FONT_PX.
    pub fn new(items: Vec<MenuItem>, measure: impl Fn(&str) -> f32) -> Self {
        let widest = items
            .iter()
            .map(|i| measure(&i.label))
            .fold(0.0_f32, f32::max);
        Self {
            width: (widest + HPAD * 2.0).ceil(),
            height: (items.len() as f32 * ROW_H + VPAD * 2.0).ceil(),
            items,
            hover: None,
        }
    }

    /// Which row a surface-local point falls in.
    pub fn row_at(&self, x: f32, y: f32) -> Option<usize> {
        if x < 0.0 || x >= self.width {
            return None;
        }
        let rel = y - VPAD;
        if rel < 0.0 {
            return None;
        }
        let row = (rel / ROW_H) as usize;
        (row < self.items.len()).then_some(row)
    }
}

/// Build the items for a slot. Pinned slots offer unpin; running unpinned apps
/// offer pin when their app id resolves to something launchable.
pub fn items_for_slot(
    pinned_spec: Option<&str>,
    app_name: Option<&str>,
    pin_spec_for_app: Option<String>,
) -> Vec<MenuItem> {
    let mut items = Vec::new();

    if let Some(spec) = pinned_spec {
        items.push(MenuItem {
            label: "Unpin from dock".into(),
            action: MenuAction::Unpin(spec.to_string()),
        });
    } else if let Some(spec) = pin_spec_for_app {
        let label = match app_name {
            Some(name) if !name.is_empty() => format!("Pin “{name}” to dock"),
            _ => "Pin to dock".into(),
        };
        items.push(MenuItem {
            label,
            action: MenuAction::Pin(spec),
        });
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(n: usize) -> MenuModel {
        let items = (0..n)
            .map(|i| MenuItem {
                label: format!("Item {i}"),
                action: MenuAction::Pin(format!("spec{i}")),
            })
            .collect();
        MenuModel::new(items, |s| s.len() as f32 * 8.0)
    }

    #[test]
    fn size_fits_widest_label() {
        let m = model(2);
        assert_eq!(m.width, ("Item 0".len() as f32 * 8.0 + HPAD * 2.0).ceil());
        assert_eq!(m.height, (2.0 * ROW_H + VPAD * 2.0).ceil());
    }

    #[test]
    fn row_hit_testing() {
        let m = model(3);
        assert_eq!(m.row_at(10.0, VPAD + 1.0), Some(0));
        assert_eq!(m.row_at(10.0, VPAD + ROW_H + 1.0), Some(1));
        assert_eq!(m.row_at(10.0, VPAD + 2.5 * ROW_H), Some(2));
        assert_eq!(m.row_at(10.0, 1.0), None, "top padding is not a row");
        assert_eq!(m.row_at(-1.0, VPAD + 1.0), None);
        assert_eq!(m.row_at(10.0, VPAD + 3.0 * ROW_H + 1.0), None);
    }

    #[test]
    fn pinned_slot_offers_unpin() {
        let items = items_for_slot(Some("applications:a.desktop"), None, None);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].action, MenuAction::Unpin("applications:a.desktop".into()));
    }

    #[test]
    fn running_app_offers_pin_only_when_launchable() {
        let items = items_for_slot(None, Some("KWrite"), Some("applications:kw.desktop".into()));
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].action, MenuAction::Pin("applications:kw.desktop".into()));
        assert!(items[0].label.contains("KWrite"));

        let items = items_for_slot(None, Some("mystery"), None);
        assert!(items.is_empty(), "unlaunchable app must offer nothing");
    }
}
