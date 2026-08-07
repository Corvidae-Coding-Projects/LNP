//! Geometry and state of the Apps menu: a category rail on the left, a
//! scrollable icon grid on the right. Pure layout math, testable without a
//! compositor; the surface plumbing lives in main.rs like the other popups.

pub const MENU_W: f32 = 640.0;
pub const MENU_H: f32 = 520.0;

pub const RAIL_W: f32 = 150.0;
pub const RAIL_TOP: f32 = 12.0;
pub const RAIL_LINE_H: f32 = 28.0;
pub const RAIL_FONT: f32 = 14.0;

pub const GRID_PAD: f32 = 10.0;
pub const COLS: usize = 5;
pub const TILE_W: f32 = (MENU_W - RAIL_W - GRID_PAD * 2.0) / COLS as f32;
pub const TILE_H: f32 = 92.0;
pub const TILE_ICON: f32 = 44.0;
pub const TILE_FONT: f32 = 12.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Hover {
    None,
    Cat(usize),
    /// Index into the currently filtered app list.
    App(usize),
}

#[derive(Debug)]
pub struct AppsMenuModel {
    pub cats: Vec<&'static str>,
    pub selected_cat: usize,
    pub scroll: f32,
    pub hover: Hover,
}

impl AppsMenuModel {
    pub fn new(cats: Vec<&'static str>) -> Self {
        Self {
            cats,
            selected_cat: 0,
            scroll: 0.0,
            hover: Hover::None,
        }
    }

    pub fn selected_name(&self) -> &'static str {
        self.cats.get(self.selected_cat).copied().unwrap_or("All")
    }

    /// Which category line sits at a surface-local point, if any.
    pub fn cat_at(&self, x: f32, y: f32) -> Option<usize> {
        if x < 0.0 || x >= RAIL_W {
            return None;
        }
        let rel = y - RAIL_TOP;
        if rel < 0.0 {
            return None;
        }
        let idx = (rel / RAIL_LINE_H) as usize;
        (idx < self.cats.len()).then_some(idx)
    }

    /// Which app tile sits at a surface-local point, given `count` apps in the
    /// current filter. Accounts for scroll.
    pub fn tile_at(&self, x: f32, y: f32, count: usize) -> Option<usize> {
        let gx = x - RAIL_W - GRID_PAD;
        let gy = y - GRID_PAD + self.scroll;
        if gx < 0.0 || gy < 0.0 || x >= MENU_W - GRID_PAD {
            return None;
        }
        let col = (gx / TILE_W) as usize;
        let row = (gy / TILE_H) as usize;
        if col >= COLS {
            return None;
        }
        let idx = row * COLS + col;
        (idx < count).then_some(idx)
    }

    /// Top-left of tile `idx` in surface coordinates (may be off-screen; the
    /// draw loop culls).
    pub fn tile_origin(&self, idx: usize) -> (f32, f32) {
        let col = idx % COLS;
        let row = idx / COLS;
        (
            RAIL_W + GRID_PAD + col as f32 * TILE_W,
            GRID_PAD + row as f32 * TILE_H - self.scroll,
        )
    }

    pub fn max_scroll(&self, count: usize) -> f32 {
        let rows = count.div_ceil(COLS);
        (rows as f32 * TILE_H + GRID_PAD * 2.0 - MENU_H).max(0.0)
    }

    pub fn scroll_by(&mut self, delta: f32, count: usize) {
        self.scroll = (self.scroll + delta).clamp(0.0, self.max_scroll(count));
    }

    pub fn select_cat(&mut self, idx: usize) {
        if idx < self.cats.len() && idx != self.selected_cat {
            self.selected_cat = idx;
            self.scroll = 0.0;
            self.hover = Hover::Cat(idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> AppsMenuModel {
        AppsMenuModel::new(vec!["All", "Internet", "System"])
    }

    #[test]
    fn rail_hit_testing() {
        let m = model();
        assert_eq!(m.cat_at(10.0, RAIL_TOP + 1.0), Some(0));
        assert_eq!(m.cat_at(10.0, RAIL_TOP + RAIL_LINE_H * 2.5), Some(2));
        assert_eq!(m.cat_at(RAIL_W + 1.0, RAIL_TOP + 1.0), None);
        assert_eq!(m.cat_at(10.0, RAIL_TOP + RAIL_LINE_H * 3.5), None);
    }

    #[test]
    fn grid_hit_testing_respects_scroll() {
        let mut m = model();
        let x0 = RAIL_W + GRID_PAD + 1.0;
        assert_eq!(m.tile_at(x0, GRID_PAD + 1.0, 30), Some(0));
        assert_eq!(m.tile_at(x0 + TILE_W, GRID_PAD + 1.0, 30), Some(1));
        assert_eq!(m.tile_at(x0, GRID_PAD + TILE_H + 1.0, 30), Some(COLS));

        m.scroll = TILE_H; // scrolled one row down
        assert_eq!(m.tile_at(x0, GRID_PAD + 1.0, 30), Some(COLS));

        // Beyond the app count is dead space, not a phantom tile.
        assert_eq!(m.tile_at(x0, GRID_PAD + 1.0, 3), None);
    }

    #[test]
    fn scroll_clamps_to_content() {
        let mut m = model();
        m.scroll_by(-100.0, 100);
        assert_eq!(m.scroll, 0.0);
        m.scroll_by(1e6, 100);
        assert_eq!(m.scroll, m.max_scroll(100));
        // A short list cannot scroll at all.
        let mut m2 = model();
        m2.scroll_by(50.0, 4);
        assert_eq!(m2.scroll, 0.0);
    }

    #[test]
    fn selecting_category_resets_scroll() {
        let mut m = model();
        m.scroll_by(300.0, 200);
        assert!(m.scroll > 0.0);
        m.select_cat(1);
        assert_eq!(m.scroll, 0.0);
        assert_eq!(m.selected_cat, 1);
    }
}
