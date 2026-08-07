//! Persistent pins.
//!
//! One launcher spec per line in ~/.config/lnp-dock/pins, same spec syntax the
//! dock has always used (preferred://…, applications:….desktop). Plain text on
//! purpose: a normal person never touches this file, but the person helping
//! them over the phone can read it aloud.
//!
//! Specs that fail to resolve (uninstalled apps) are kept in the file and
//! simply not shown -- unpinning someone's app because it was briefly
//! uninstalled would be data loss.

use std::path::PathBuf;

pub const DEFAULT_PINS: &[&str] = &[
    "preferred://browser",
    "preferred://filemanager",
    "applications:org.kde.konsole.desktop",
    "applications:systemsettings.desktop",
    "applications:org.kde.discover.desktop",
];

fn pins_path() -> PathBuf {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME").unwrap_or_default();
            PathBuf::from(home).join(".config")
        });
    config.join("lnp-dock").join("pins")
}

pub fn load() -> Vec<String> {
    let path = pins_path();

    match std::fs::read_to_string(&path) {
        Ok(text) => parse(&text),
        Err(_) => {
            // First run: materialise the defaults so the file the user's
            // helper finds is the file that is actually in effect.
            let defaults: Vec<String> = DEFAULT_PINS.iter().map(|s| s.to_string()).collect();
            save(&defaults);
            defaults
        }
    }
}

pub fn save(pins: &[String]) {
    let path = pins_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }

    let mut text = String::from(
        "# Apps pinned to the dock, one per line, in order.\n\
         # Managed by right-clicking dock icons; hand-editing works too.\n",
    );
    for p in pins {
        text.push_str(p);
        text.push('\n');
    }

    // Write-then-rename so a crash mid-write cannot truncate the pin list.
    let tmp = path.with_extension("tmp");
    if std::fs::write(&tmp, &text).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}

fn parse(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// Display order after dragging item `from` to position `to`: the classic
/// remove-and-reinsert. Pure so it can drive both the live drag preview and
/// the final commit -- one function, so what you see is what gets saved.
pub fn preview_order(count: usize, from: usize, to: usize) -> Vec<usize> {
    let mut order: Vec<usize> = (0..count).collect();
    if from < count {
        let item = order.remove(from);
        order.insert(to.min(count - 1), item);
    }
    order
}

/// Rewrite `all` (the pins file content) so its resolvable entries take the
/// order of `new_displayed`, while unresolvable entries -- pins whose app is
/// not currently installed, which the dock keeps but does not show -- stay in
/// their file positions rather than being shuffled or lost.
pub fn reorder_interleaved(
    all: &[String],
    displayed: &[String],
    new_displayed: &[String],
) -> Vec<String> {
    let mut replacement = new_displayed.iter();
    all.iter()
        .map(|spec| {
            if displayed.contains(spec) {
                replacement
                    .next()
                    .cloned()
                    .unwrap_or_else(|| spec.clone())
            } else {
                spec.clone()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn preview_moves_forward_and_back() {
        assert_eq!(preview_order(5, 1, 3), vec![0, 2, 3, 1, 4]);
        assert_eq!(preview_order(5, 3, 1), vec![0, 3, 1, 2, 4]);
        assert_eq!(preview_order(5, 2, 2), vec![0, 1, 2, 3, 4]);
        assert_eq!(preview_order(3, 0, 2), vec![1, 2, 0]);
        // Out-of-range targets clamp instead of panicking.
        assert_eq!(preview_order(3, 0, 99), vec![1, 2, 0]);
    }

    #[test]
    fn reorder_keeps_unresolvable_pins_in_place() {
        // "ghost" is an uninstalled app's pin: present in the file, not shown.
        let all = v(&["a", "ghost", "b", "c"]);
        let displayed = v(&["a", "b", "c"]);
        let new_displayed = v(&["c", "a", "b"]);
        assert_eq!(
            reorder_interleaved(&all, &displayed, &new_displayed),
            v(&["c", "ghost", "a", "b"])
        );
    }

    #[test]
    fn reorder_without_ghosts_is_plain_replacement() {
        let all = v(&["a", "b", "c"]);
        let new = v(&["b", "c", "a"]);
        assert_eq!(reorder_interleaved(&all, &all.clone(), &new), new);
    }

    #[test]
    fn parse_skips_comments_and_blanks() {
        let text = "# header\n\napplications:a.desktop\n  preferred://browser  \n#x\n";
        assert_eq!(
            parse(text),
            vec!["applications:a.desktop", "preferred://browser"]
        );
    }

    #[test]
    fn roundtrip_preserves_order_and_content() {
        let pins: Vec<String> = vec![
            "preferred://browser".into(),
            "applications:org.kde.konsole.desktop".into(),
        ];
        let mut text = String::new();
        for p in &pins {
            text.push_str(p);
            text.push('\n');
        }
        assert_eq!(parse(&text), pins);
    }
}
