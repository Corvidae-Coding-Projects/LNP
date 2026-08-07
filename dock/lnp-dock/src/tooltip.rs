//! Tooltip content and geometry.
//!
//! A dock tooltip answers two questions: "what is this icon?" and "which
//! windows does it hold?" -- the second being the one the running indicator
//! dots cannot answer. Line one is the app's name; the window titles follow,
//! each ellipsized, capped so a browser with forty windows produces a tooltip
//! and not a bedsheet.

use std::time::Duration;

pub const FONT_PX: f32 = 14.0;
pub const HPAD: f32 = 12.0;
pub const VPAD: f32 = 8.0;
pub const LINE_H: f32 = 20.0;
/// Longest a line may render, in logical pixels, before ellipsis.
pub const MAX_LINE_W: f32 = 380.0;
/// Window titles shown before collapsing into "and N more".
const MAX_TITLES: usize = 4;

/// How long the pointer must rest on a slot before the tooltip appears. Once
/// one is up, sliding to a neighbour switches instantly ("hot" tracking) --
/// the delay exists to keep tooltips out of a pointer that is just passing
/// through, not to slow down someone actively reading the dock.
pub const SHOW_DELAY: Duration = Duration::from_millis(450);

#[derive(Debug)]
pub struct TooltipModel {
    pub lines: Vec<String>,
    pub width: f32,
    pub height: f32,
}

impl TooltipModel {
    /// `measure` maps a string to its rendered width at FONT_PX.
    pub fn new(lines: Vec<String>, measure: impl Fn(&str) -> f32) -> Self {
        let lines: Vec<String> = lines
            .into_iter()
            .map(|l| ellipsize(&l, MAX_LINE_W, &measure))
            .collect();
        let widest = lines.iter().map(|l| measure(l)).fold(0.0_f32, f32::max);
        Self {
            width: (widest + HPAD * 2.0).ceil(),
            height: (lines.len() as f32 * LINE_H + VPAD * 2.0).ceil(),
            lines,
        }
    }
}

/// App name first, then window titles, capped.
pub fn compose(name: &str, titles: &[String]) -> Vec<String> {
    let mut lines = vec![name.to_string()];

    let real: Vec<&String> = titles.iter().filter(|t| !t.trim().is_empty()).collect();

    // One window whose title is just the app's name adds nothing.
    if real.len() == 1 && real[0].as_str() == name {
        return lines;
    }

    for t in real.iter().take(MAX_TITLES) {
        lines.push(format!("  {t}"));
    }
    if real.len() > MAX_TITLES {
        lines.push(format!("  … and {} more", real.len() - MAX_TITLES));
    }

    lines
}

/// Cut `text` to fit `max_w`, appending an ellipsis. Works on characters, not
/// bytes, so multi-byte titles do not get sliced mid-codepoint.
pub fn ellipsize(text: &str, max_w: f32, measure: impl Fn(&str) -> f32) -> String {
    if measure(text) <= max_w {
        return text.to_string();
    }

    let chars: Vec<char> = text.chars().collect();
    let mut keep = chars.len();
    while keep > 0 {
        let candidate: String = chars[..keep].iter().collect::<String>() + "…";
        if measure(&candidate) <= max_w {
            return candidate;
        }
        keep -= 1;
    }
    "…".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(s: &str) -> f32 {
        s.chars().count() as f32 * 10.0
    }

    #[test]
    fn compose_name_only_when_no_windows() {
        assert_eq!(compose("Konsole", &[]), vec!["Konsole"]);
    }

    #[test]
    fn compose_lists_titles() {
        let titles = vec!["a — vim".to_string(), "b — logs".to_string()];
        let lines = compose("Konsole", &titles);
        assert_eq!(lines.len(), 3);
        assert!(lines[1].contains("vim"));
    }

    #[test]
    fn compose_caps_and_counts_overflow() {
        let titles: Vec<String> = (0..7).map(|i| format!("win {i}")).collect();
        let lines = compose("App", &titles);
        // name + 4 titles + "and 3 more"
        assert_eq!(lines.len(), 6);
        assert!(lines[5].contains("3 more"));
    }

    #[test]
    fn compose_skips_redundant_single_title() {
        let lines = compose("Dolphin", &["Dolphin".to_string()]);
        assert_eq!(lines, vec!["Dolphin"]);
    }

    #[test]
    fn ellipsize_respects_width_and_codepoints() {
        let out = ellipsize("héllo wörld wide", 80.0, m);
        assert!(m(&out) <= 80.0);
        assert!(out.ends_with('…'));
        // Untouched when it fits.
        assert_eq!(ellipsize("short", 200.0, m), "short");
    }

    #[test]
    fn model_sizes_from_widest_line() {
        let model = TooltipModel::new(vec!["ab".into(), "abcd".into()], m);
        assert_eq!(model.width, (40.0 + HPAD * 2.0).ceil());
        assert_eq!(model.height, (2.0 * LINE_H + VPAD * 2.0).ceil());
    }
}
