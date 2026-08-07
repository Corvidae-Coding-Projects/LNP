//! Pinned launchers: resolving desktop entries and their icons.
//!
//! The .desktop parsing here is deliberately hand-rolled rather than pulled
//! from a crate. The format is a small INI dialect, we need four keys from it,
//! and owning the parser means the dock cannot break because a dependency
//! changed its API between releases. For a project whose whole premise is that
//! the user should never have to open a terminal, "it still builds and runs in
//! three years" is worth more than saving forty lines.

use anyhow::{Context, Result, anyhow};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A resolved launcher, ready to draw and to run.
#[derive(Debug, Clone)]
pub struct Launcher {
    /// The spec this launcher came from, verbatim -- the unit of pin
    /// management, since it is what the pins file stores.
    pub spec: String,
    pub name: String,
    pub icon: Option<PathBuf>,
    pub exec: String,
    pub desktop_file: PathBuf,
    pub terminal: bool,
}

/// The `[Desktop Entry]` group of a .desktop file, as a flat map.
pub(crate) fn parse_desktop_entry(path: &Path) -> Result<HashMap<String, String>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;

    let mut map = HashMap::new();
    let mut in_entry = false;

    for line in text.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') {
            // Only the main group interests us; actions and other groups can
            // repeat keys and would otherwise clobber what we already read.
            in_entry = line == "[Desktop Entry]";
            continue;
        }

        if !in_entry {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            // Skip localised variants like Name[de]; we want the default.
            if key.contains('[') {
                continue;
            }
            map.insert(key.to_string(), value.trim().to_string());
        }
    }

    Ok(map)
}

/// Directories that may contain .desktop files, most specific first.
pub(crate) fn application_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(home) = std::env::var_os("HOME") {
        let data_home = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(&home).join(".local/share"));
        dirs.push(data_home.join("applications"));
    }

    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
    for d in data_dirs.split(':').filter(|s| !s.is_empty()) {
        dirs.push(PathBuf::from(d).join("applications"));
    }

    dirs
}

/// Find a .desktop file by its id, e.g. `org.kde.dolphin.desktop`.
fn find_desktop_file(id: &str) -> Option<PathBuf> {
    let id = if id.ends_with(".desktop") {
        id.to_string()
    } else {
        format!("{id}.desktop")
    };

    for dir in application_dirs() {
        let candidate = dir.join(&id);
        if candidate.is_file() {
            return Some(candidate);
        }
        // Some entries live in subdirectories with a dash-mangled id.
        let flattened = id.replace('-', "/");
        let candidate = dir.join(&flattened);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

/// Strip the field codes (%u, %F, %i, ...) that a desktop Exec line may carry.
///
/// We launch with no arguments, so every field code should simply vanish. %% is
/// a literal percent and must survive.
pub fn strip_field_codes(exec: &str) -> String {
    let mut out = String::with_capacity(exec.len());
    let mut chars = exec.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('%') => out.push('%'),
            // Any other field code expands to nothing when we have no file or
            // URL arguments to pass.
            Some(_) => {}
            None => {}
        }
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Resolve a launcher spec into something runnable.
///
/// Accepts the same forms the Plasma task manager uses, so pinned lists can be
/// carried over unchanged:
///   * `applications:org.kde.dolphin.desktop`
///   * `org.kde.dolphin.desktop`
///   * `preferred://browser` / `preferred://filemanager`
pub fn resolve(spec: &str) -> Result<Launcher> {
    let id = if let Some(rest) = spec.strip_prefix("applications:") {
        rest.to_string()
    } else if let Some(kind) = spec.strip_prefix("preferred://") {
        preferred_application(kind)
            .ok_or_else(|| anyhow!("no application registered for preferred://{kind}"))?
    } else {
        spec.to_string()
    };

    let path = find_desktop_file(&id)
        .ok_or_else(|| anyhow!("no desktop file found for {id}"))?;

    let entry = parse_desktop_entry(&path)?;

    let exec = entry
        .get("Exec")
        .ok_or_else(|| anyhow!("{} has no Exec key", path.display()))?;

    let icon_name = entry.get("Icon").cloned();

    Ok(Launcher {
        spec: spec.to_string(),
        name: entry
            .get("Name")
            .cloned()
            .unwrap_or_else(|| id.trim_end_matches(".desktop").to_string()),
        icon: icon_name.as_deref().and_then(lookup_icon),
        exec: strip_field_codes(exec),
        desktop_file: path,
        terminal: entry.get("Terminal").map(|v| v == "true").unwrap_or(false),
    })
}

/// The pin spec for a running window's app id, if the app id resolves to an
/// installed desktop file. Windows whose app id resolves to nothing cannot be
/// pinned -- there would be nothing to launch later -- so their menu simply
/// omits the option.
pub fn spec_for_app_id(app_id: &str) -> Option<String> {
    let path = find_desktop_file(app_id)?;
    let file = path.file_name()?.to_str()?;
    Some(format!("applications:{file}"))
}

/// Resolve `preferred://browser` and friends via the XDG MIME associations.
///
/// This is why the pinned defaults use preferred:// -- pinning a hardcoded
/// browser is how you end up shipping a dock with a dead Firefox icon on a
/// machine that has Chrome.
fn preferred_application(kind: &str) -> Option<String> {
    let mime = match kind {
        "browser" => "x-scheme-handler/http",
        "filemanager" => "inode/directory",
        "mail" => "x-scheme-handler/mailto",
        "terminal" => return preferred_terminal(),
        _ => return None,
    };

    // The registered default wins, but only if it is actually installed.
    //
    // Dangling defaults are common and are exactly the sort of thing that
    // leaves a normal person staring at a dock icon that does nothing. This
    // machine, for instance, had inode/directory pointing at Nautilus on a
    // system with no Nautilus. Verify before trusting it.
    if let Some(id) = query_default_application(mime) {
        if find_desktop_file(&id).is_some() {
            return Some(id);
        }
    }

    fallback_candidates(kind)
        .iter()
        .find(|id| find_desktop_file(id).is_some())
        .map(|id| id.to_string())
}

/// Known-good alternatives, used only when the registered default is missing
/// or dangling. Ordered by how well they fit a Plasma desktop.
fn fallback_candidates(kind: &str) -> &'static [&'static str] {
    match kind {
        "browser" => &[
            "firefox.desktop",
            "google-chrome.desktop",
            "chromium-browser.desktop",
            "org.kde.falkon.desktop",
        ],
        "filemanager" => &[
            "org.kde.dolphin.desktop",
            "org.gnome.Nautilus.desktop",
            "nemo.desktop",
            "thunar.desktop",
        ],
        "mail" => &["org.kde.kmail2.desktop", "thunderbird.desktop"],
        _ => &[],
    }
}

fn query_default_application(mime: &str) -> Option<String> {
    // mimeapps.list, in XDG precedence order.
    let mut candidates = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        let config_home = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(&home).join(".config"));
        candidates.push(config_home.join("mimeapps.list"));
    }
    candidates.push(PathBuf::from("/etc/xdg/mimeapps.list"));
    for dir in application_dirs() {
        candidates.push(dir.join("mimeapps.list"));
    }

    for file in candidates {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let mut in_defaults = false;
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_defaults = line == "[Default Applications]";
                continue;
            }
            if !in_defaults {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                if key.trim() == mime {
                    // The value may be a semicolon-separated preference list.
                    if let Some(first) = value.split(';').map(str::trim).find(|s| !s.is_empty()) {
                        return Some(first.to_string());
                    }
                }
            }
        }
    }

    None
}

fn preferred_terminal() -> Option<String> {
    for id in [
        "org.kde.konsole.desktop",
        "org.gnome.Terminal.desktop",
        "xterm.desktop",
    ] {
        if find_desktop_file(id).is_some() {
            return Some(id.to_string());
        }
    }
    None
}

/// Best-effort icon for a running window that is not one of our pins.
///
/// Try the window's app_id as a desktop file id first (the normal case:
/// "org.kde.konsole" -> org.kde.konsole.desktop -> its Icon key), then the
/// themed icon name the compositor supplied, then the app_id as a raw icon
/// name, and finally a generic fallback so an exotic window still gets
/// *something* clickable rather than an invisible slot.
pub fn icon_for_app_id(app_id: &str, themed: Option<&str>) -> Option<PathBuf> {
    if let Some(path) = find_desktop_file(app_id) {
        if let Ok(entry) = parse_desktop_entry(&path) {
            if let Some(icon) = entry.get("Icon") {
                if let Some(p) = lookup_icon(icon) {
                    return Some(p);
                }
            }
        }
    }

    if let Some(name) = themed {
        if let Some(p) = lookup_icon(name) {
            return Some(p);
        }
    }

    if let Some(p) = lookup_icon(app_id) {
        return Some(p);
    }
    if let Some(p) = lookup_icon(&app_id.to_lowercase()) {
        return Some(p);
    }

    lookup_icon("application-x-executable")
}

/// Does this window belong to this launcher? Matches the desktop file id
/// against the window's app_id, which is how Wayland app_ids are defined to
/// work; case-insensitive because reality is sloppier than the spec.
pub fn app_id_matches(launcher: &Launcher, app_id: &str) -> bool {
    if app_id.is_empty() {
        return false;
    }
    let stem = launcher
        .desktop_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    stem.eq_ignore_ascii_case(app_id)
}

/// Look an icon name up in the icon theme.
///
/// Absolute paths in a desktop file's Icon key are used as-is, which the spec
/// permits and some applications rely on.
pub fn lookup_icon(name: &str) -> Option<PathBuf> {
    let as_path = Path::new(name);
    if as_path.is_absolute() && as_path.is_file() {
        return Some(as_path.to_path_buf());
    }

    // Ask for a large nominal size: the dock magnifies, so the icon has to
    // survive being drawn well above its resting size without going soft.
    for theme in ["breeze", "Adwaita", "hicolor"] {
        if let Some(p) = freedesktop_icons::lookup(name)
            .with_theme(theme)
            .with_size(128)
            .with_cache()
            .find()
        {
            return Some(p);
        }
    }

    freedesktop_icons::lookup(name).with_size(128).with_cache().find()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_codes_are_stripped() {
        assert_eq!(strip_field_codes("dolphin %u"), "dolphin");
        assert_eq!(strip_field_codes("app %F --flag %i"), "app --flag");
        assert_eq!(strip_field_codes("konsole"), "konsole");
    }

    #[test]
    fn literal_percent_survives() {
        assert_eq!(strip_field_codes("thing --fmt 100%%"), "thing --fmt 100%");
    }

    #[test]
    fn parses_a_minimal_desktop_entry() {
        let dir = std::env::temp_dir().join("lnp-dock-test-entry");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.desktop");
        std::fs::write(
            &path,
            "[Desktop Entry]\nName=Thing\nName[de]=Ding\nExec=thing %U\nIcon=thing\n\n[Desktop Action New]\nName=Other\n",
        )
        .unwrap();

        let e = parse_desktop_entry(&path).unwrap();
        assert_eq!(e.get("Name").unwrap(), "Thing", "localised key must not win");
        assert_eq!(e.get("Exec").unwrap(), "thing %U");
        // The action group's Name must not leak into the main group.
        assert_ne!(e.get("Name").unwrap(), "Other");

        std::fs::remove_dir_all(&dir).ok();
    }
}
