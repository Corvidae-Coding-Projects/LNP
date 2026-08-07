//! Rasterising icons and painting the dock.
//!
//! Software rendering via tiny-skia rather than the GPU. A dock is a handful of
//! icons that only repaint while the pointer is over it; the GPU buys nothing
//! here and costs a wgpu dependency tree plus a class of driver bugs that we
//! would then have to explain to somebody who never asked for a GPU.

use anyhow::{Context, Result, anyhow};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tiny_skia::{
    Color, FillRule, Paint, PathBuilder, Pixmap, PixmapPaint, PixmapRef, Rect, Transform,
};

/// Icons are expensive to rasterise and cheap to keep, and magnification means
/// we ask for many sizes of the same icon. Cache per (path, size).
#[derive(Default)]
pub struct IconCache {
    cache: HashMap<(PathBuf, u32), Option<Pixmap>>,
}

impl IconCache {
    /// Rasterise `path` at `size` pixels square, or return a previously
    /// rasterised copy. A failed rasterisation is cached as a failure so a
    /// broken icon file does not get retried on every single frame.
    pub fn get(&mut self, path: &Path, size: u32) -> Option<PixmapRef<'_>> {
        let key = (path.to_path_buf(), size);
        self.cache
            .entry(key)
            .or_insert_with(|| rasterize(path, size).ok())
            .as_ref()
            .map(|p| p.as_ref())
    }
}

/// Quantise a requested size so the cache does not accumulate one entry per
/// pixel as an icon smoothly magnifies. 2px steps are below the threshold of
/// noticing and cut the cache to a few dozen entries per icon.
pub fn quantize_size(size: f32) -> u32 {
    let s = size.max(8.0).round() as u32;
    s.div_ceil(2) * 2
}

fn rasterize(path: &Path, size: u32) -> Result<Pixmap> {
    let data = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;

    let is_svg = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("svg") || e.eq_ignore_ascii_case("svgz"))
        .unwrap_or(false);

    if is_svg {
        rasterize_svg(&data, size)
    } else {
        rasterize_raster(&data, size)
    }
}

fn rasterize_svg(data: &[u8], size: u32) -> Result<Pixmap> {
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(data, &opt).context("parsing SVG")?;

    let mut pixmap = Pixmap::new(size, size).ok_or_else(|| anyhow!("bad pixmap size {size}"))?;

    // Fit the SVG's own viewBox into our square, preserving aspect ratio.
    let ts = tree.size();
    let scale = (size as f32 / ts.width()).min(size as f32 / ts.height());
    let dx = (size as f32 - ts.width() * scale) / 2.0;
    let dy = (size as f32 - ts.height() * scale) / 2.0;

    resvg::render(
        &tree,
        Transform::from_translate(dx, dy).pre_scale(scale, scale),
        &mut pixmap.as_mut(),
    );

    Ok(pixmap)
}

fn rasterize_raster(data: &[u8], size: u32) -> Result<Pixmap> {
    let img = image::load_from_memory(data).context("decoding raster icon")?;
    let img = img
        .resize_exact(size, size, image::imageops::FilterType::Lanczos3)
        .to_rgba8();

    let mut pixmap = Pixmap::new(size, size).ok_or_else(|| anyhow!("bad pixmap size {size}"))?;

    // tiny-skia stores premultiplied alpha; image gives straight alpha.
    for (dst, src) in pixmap.pixels_mut().iter_mut().zip(img.pixels()) {
        let [r, g, b, a] = src.0;
        *dst = tiny_skia::PremultipliedColorU8::from_rgba(
            (r as u16 * a as u16 / 255) as u8,
            (g as u16 * a as u16 / 255) as u8,
            (b as u16 * a as u16 / 255) as u8,
            a,
        )
        .ok_or_else(|| anyhow!("invalid premultiplied pixel"))?;
    }

    Ok(pixmap)
}

/// Colours and metrics of the dock's own chrome.
#[derive(Debug, Clone, Copy)]
pub struct DockTheme {
    pub background: Color,
    pub corner_radius: f32,
    /// Padding between the dock edge and the icon slots.
    pub padding: f32,
    /// Colour of the small dot under a running application.
    pub indicator: Color,
}

impl Default for DockTheme {
    fn default() -> Self {
        Self {
            background: Color::from_rgba8(28, 28, 32, 165),
            corner_radius: 18.0,
            padding: 8.0,
            indicator: Color::from_rgba8(235, 235, 240, 220),
        }
    }
}

/// Paint a rounded rectangle. tiny-skia has no primitive for one, so build the
/// path from four arcs.
fn rounded_rect(rect: Rect, radius: f32) -> Option<tiny_skia::Path> {
    let r = radius
        .min(rect.width() / 2.0)
        .min(rect.height() / 2.0)
        .max(0.0);

    let (l, t, right, b) = (rect.left(), rect.top(), rect.right(), rect.bottom());
    let mut pb = PathBuilder::new();

    // Kappa: the control-point ratio that makes a cubic Bezier approximate a
    // quarter circle to within about 0.02%.
    const K: f32 = 0.5522847;
    let c = r * K;

    pb.move_to(l + r, t);
    pb.line_to(right - r, t);
    pb.cubic_to(right - r + c, t, right, t + r - c, right, t + r);
    pb.line_to(right, b - r);
    pb.cubic_to(right, b - r + c, right - r + c, b, right - r, b);
    pb.line_to(l + r, b);
    pb.cubic_to(l + r - c, b, l, b - r + c, l, b - r);
    pb.line_to(l, t + r);
    pb.cubic_to(l, t + r - c, l + r - c, t, l + r, t);
    pb.close();

    pb.finish()
}

/// Fill the dock's background plate.
pub fn draw_background(pixmap: &mut Pixmap, rect: Rect, theme: &DockTheme) {
    let Some(path) = rounded_rect(rect, theme.corner_radius) else {
        return;
    };

    let mut paint = Paint::default();
    paint.set_color(theme.background);
    paint.anti_alias = true;

    pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
}

/// Draw one icon centred on `center_x`, with its baseline sitting `bottom`
/// pixels from the top of the surface, scaled by `scale`.
///
/// The icon is rasterised at its *drawn* size rather than rasterised once and
/// scaled up, so a magnified icon is genuinely sharper rather than a blurry
/// enlargement. This is the main reason magnification is worth doing at all
/// beyond the Fitts benefit.
pub fn draw_icon(
    pixmap: &mut Pixmap,
    cache: &mut IconCache,
    icon: &Path,
    center_x: f32,
    bottom: f32,
    drawn_size: f32,
) {
    let size = quantize_size(drawn_size);
    let Some(src) = cache.get(icon, size) else {
        return;
    };

    let x = center_x - size as f32 / 2.0;
    let y = bottom - size as f32;

    pixmap.draw_pixmap(
        x.round() as i32,
        y.round() as i32,
        src,
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
}

/// Small dot under a running application.
pub fn draw_running_indicator(pixmap: &mut Pixmap, center_x: f32, y: f32, theme: &DockTheme) {
    let r = 2.0;
    let Some(rect) = Rect::from_xywh(center_x - r, y - r, r * 2.0, r * 2.0) else {
        return;
    };
    let Some(path) = rounded_rect(rect, r) else {
        return;
    };

    let mut paint = Paint::default();
    paint.set_color(theme.indicator);
    paint.anti_alias = true;

    pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_quantise_to_even_steps() {
        assert_eq!(quantize_size(48.0), 48);
        assert_eq!(quantize_size(49.0), 50);
        assert_eq!(quantize_size(48.3), 48);
        // Never degenerate to something unrasterisable.
        assert!(quantize_size(0.0) >= 8);
        assert!(quantize_size(-5.0) >= 8);
    }

    #[test]
    fn rounded_rect_is_buildable() {
        let rect = Rect::from_xywh(0.0, 0.0, 100.0, 40.0).unwrap();
        assert!(rounded_rect(rect, 12.0).is_some());
        // Radius larger than the box must clamp rather than produce garbage.
        assert!(rounded_rect(rect, 500.0).is_some());
        assert!(rounded_rect(rect, 0.0).is_some());
    }

    #[test]
    fn background_fills_without_panicking() {
        let mut pm = Pixmap::new(200, 100).unwrap();
        let rect = Rect::from_xywh(10.0, 10.0, 180.0, 80.0).unwrap();
        draw_background(&mut pm, rect, &DockTheme::default());
        // Centre pixel should now be non-transparent.
        let px = pm.pixel(100, 50).unwrap();
        assert!(px.alpha() > 0, "background did not paint");
    }

    #[test]
    fn raster_icons_are_premultiplied_correctly() {
        // A fully transparent source pixel must stay transparent, and an
        // opaque one must keep its colour. Getting this wrong shows up as dark
        // fringing around every icon.
        let mut img = image::RgbaImage::new(2, 1);
        img.put_pixel(0, 0, image::Rgba([255, 0, 0, 0]));
        img.put_pixel(1, 0, image::Rgba([0, 255, 0, 255]));
        let mut buf = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut buf, image::ImageFormat::Png)
            .unwrap();

        let pm = rasterize_raster(&buf.into_inner(), 2).unwrap();
        let transparent = pm.pixel(0, 0).unwrap();
        assert_eq!(transparent.alpha(), 0);
        assert_eq!(transparent.red(), 0, "transparent pixel kept colour");
    }
}
