//! Minimal text rendering for the dock's menus.
//!
//! One font, one size at a time, greyscale coverage blended straight onto the
//! tiny-skia pixmap. A context menu needs exactly this much typography and no
//! more; shaping engines can arrive when the dock grows labels in scripts that
//! need them.

use fontdue::Font;
use tiny_skia::Pixmap;

pub struct TextRenderer {
    font: Font,
}

/// Where to look for a UI font, most preferred first. fc-match answers for the
/// system; the literal paths cover a machine where fontconfig is broken,
/// because a dock that panics over fonts helps nobody.
fn font_candidates() -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();

    if let Ok(res) = std::process::Command::new("fc-match")
        .args(["-f", "%{file}", "sans-serif"])
        .output()
    {
        if res.status.success() {
            let path = String::from_utf8_lossy(&res.stdout).trim().to_string();
            if !path.is_empty() {
                out.push(path.into());
            }
        }
    }

    out.push("/usr/share/fonts/google-noto/NotoSans-Regular.ttf".into());
    out.push("/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf".into());
    out
}

impl TextRenderer {
    pub fn new() -> Option<Self> {
        for path in font_candidates() {
            if let Ok(data) = std::fs::read(&path) {
                if let Ok(font) = Font::from_bytes(data, fontdue::FontSettings::default()) {
                    return Some(Self { font });
                }
            }
        }
        eprintln!("lnp-dock: no usable font found; menus will not be shown");
        None
    }

    /// Width of `text` at `px` in pixels.
    pub fn measure(&self, text: &str, px: f32) -> f32 {
        text.chars()
            .map(|c| self.font.metrics(c, px).advance_width)
            .sum()
    }

    /// Draw `text` with its baseline at (`x`, `baseline_y`). Returns the
    /// advance. `color` is straight-alpha RGBA.
    pub fn draw(
        &self,
        pixmap: &mut Pixmap,
        text: &str,
        x: f32,
        baseline_y: f32,
        px: f32,
        color: [u8; 4],
    ) -> f32 {
        let width = pixmap.width() as i32;
        let height = pixmap.height() as i32;
        let mut pen_x = x;

        for ch in text.chars() {
            let (metrics, coverage) = self.font.rasterize(ch, px);

            let gx = (pen_x + metrics.xmin as f32).round() as i32;
            // fontdue's ymin is the bitmap's bottom relative to the baseline,
            // y-up; pixmap is y-down.
            let gy = baseline_y.round() as i32 - metrics.height as i32 - metrics.ymin;

            for row in 0..metrics.height {
                let py = gy + row as i32;
                if py < 0 || py >= height {
                    continue;
                }
                for col in 0..metrics.width {
                    let px_x = gx + col as i32;
                    if px_x < 0 || px_x >= width {
                        continue;
                    }
                    let cov = coverage[row * metrics.width + col];
                    if cov == 0 {
                        continue;
                    }
                    blend(pixmap, px_x as u32, py as u32, color, cov);
                }
            }

            pen_x += metrics.advance_width;
        }

        pen_x - x
    }
}

/// Source-over blend of a straight-alpha colour with per-pixel coverage onto
/// the premultiplied pixmap.
fn blend(pixmap: &mut Pixmap, x: u32, y: u32, color: [u8; 4], coverage: u8) {
    let idx = (y * pixmap.width() + x) as usize;
    let Some(dst) = pixmap.pixels_mut().get_mut(idx) else {
        return;
    };

    let a = (color[3] as u32 * coverage as u32) / 255; // effective alpha 0..255
    let inv = 255 - a;

    let sr = color[0] as u32 * a / 255;
    let sg = color[1] as u32 * a / 255;
    let sb = color[2] as u32 * a / 255;

    let nr = (sr + dst.red() as u32 * inv / 255).min(255) as u8;
    let ng = (sg + dst.green() as u32 * inv / 255).min(255) as u8;
    let nb = (sb + dst.blue() as u32 * inv / 255).min(255) as u8;
    let na = (a + dst.alpha() as u32 * inv / 255).min(255) as u8;

    if let Some(px) = tiny_skia::PremultipliedColorU8::from_rgba(nr, ng, nb, na) {
        *dst = px;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_a_font_and_measures() {
        let tr = TextRenderer::new().expect("a system font should exist");
        let w = tr.measure("Unpin from dock", 15.0);
        assert!(w > 40.0 && w < 400.0, "implausible width {w}");
    }

    #[test]
    fn drawing_marks_pixels() {
        let tr = TextRenderer::new().unwrap();
        let mut pm = Pixmap::new(200, 40).unwrap();
        let advance = tr.draw(&mut pm, "Pin", 4.0, 30.0, 18.0, [255, 255, 255, 255]);
        assert!(advance > 5.0);
        let touched = pm.pixels().iter().filter(|p| p.alpha() > 0).count();
        assert!(touched > 20, "only {touched} pixels touched");
    }
}
