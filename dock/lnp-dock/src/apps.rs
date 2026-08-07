//! The application catalogue behind the dock's Apps button.
//!
//! Scans the XDG application directories once per menu-opening session,
//! filters to what a person can actually launch, and sorts everything into
//! the familiar categories. This is the "rest of the system" door: anything
//! installed is reachable from here, so nothing ever requires a terminal or
//! a file manager safari through /usr/share/applications.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::launchers;

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub name: String,
    pub icon: Option<PathBuf>,
    pub exec: String,
    pub category: &'static str,
}

/// Category display order. Only non-empty ones are shown; "All" is synthetic.
pub const CATEGORIES: &[&str] = &[
    "All",
    "Internet",
    "Multimedia",
    "Graphics",
    "Office",
    "Games",
    "Development",
    "Education",
    "Accessories",
    "System",
    "Settings",
    "Other",
];

/// Map a desktop file's Categories field to one display category. First match
/// in freedesktop main-category order wins.
fn categorize(categories: &str) -> &'static str {
    let cats: Vec<&str> = categories.split(';').collect();
    let has = |c: &str| cats.contains(&c);

    if has("Network") {
        "Internet"
    } else if has("AudioVideo") || has("Audio") || has("Video") {
        "Multimedia"
    } else if has("Graphics") {
        "Graphics"
    } else if has("Office") {
        "Office"
    } else if has("Game") {
        "Games"
    } else if has("Development") {
        "Development"
    } else if has("Education") || has("Science") {
        "Education"
    } else if has("Settings") {
        "Settings"
    } else if has("System") {
        "System"
    } else if has("Utility") {
        "Accessories"
    } else {
        "Other"
    }
}

/// Should this entry appear in a launcher menu at all?
fn is_visible(entry: &std::collections::HashMap<String, String>) -> bool {
    let flag = |k: &str| entry.get(k).map(|v| v == "true").unwrap_or(false);

    if flag("NoDisplay") || flag("Hidden") {
        return false;
    }
    // Terminal=true entries would silently run without their terminal here --
    // a button that does nothing visible. Leave them to actual terminals.
    if flag("Terminal") {
        return false;
    }
    if entry.get("Type").map(|t| t != "Application").unwrap_or(false) {
        return false;
    }
    // We are a KDE session; respect OnlyShowIn/NotShowIn.
    if let Some(only) = entry.get("OnlyShowIn") {
        if !only.split(';').any(|d| d == "KDE") {
            return false;
        }
    }
    if let Some(not) = entry.get("NotShowIn") {
        if not.split(';').any(|d| d == "KDE") {
            return false;
        }
    }
    entry.contains_key("Exec")
}

/// Scan every application directory, first-in-precedence wins per desktop id.
pub fn scan() -> Vec<AppEntry> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut apps = Vec::new();

    for dir in launchers::application_dirs() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for file in entries.flatten() {
            let path = file.path();
            let Some(fname) = path.file_name().and_then(|f| f.to_str()) else {
                continue;
            };
            if !fname.ends_with(".desktop") {
                continue;
            }
            // XDG precedence: an id already seen in an earlier (more
            // specific) directory shadows this one entirely.
            if !seen.insert(fname.to_string()) {
                continue;
            }

            let Ok(entry) = launchers::parse_desktop_entry(&path) else {
                continue;
            };
            if !is_visible(&entry) {
                continue;
            }
            let (Some(name), Some(exec)) = (entry.get("Name"), entry.get("Exec")) else {
                continue;
            };

            apps.push(AppEntry {
                name: name.clone(),
                icon: entry
                    .get("Icon")
                    .and_then(|i| launchers::lookup_icon(i)),
                exec: launchers::strip_field_codes(exec),
                category: categorize(entry.get("Categories").map(String::as_str).unwrap_or("")),
            });
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

/// The categories that actually have apps, in display order, "All" first.
pub fn present_categories(apps: &[AppEntry]) -> Vec<&'static str> {
    CATEGORIES
        .iter()
        .copied()
        .filter(|c| *c == "All" || apps.iter().any(|a| a.category == *c))
        .collect()
}

pub fn filtered<'a>(apps: &'a [AppEntry], category: &str) -> Vec<&'a AppEntry> {
    apps.iter()
        .filter(|a| category == "All" || a.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categorize_prefers_specific_over_other() {
        assert_eq!(categorize("Qt;KDE;Network;WebBrowser;"), "Internet");
        assert_eq!(categorize("AudioVideo;Player;"), "Multimedia");
        assert_eq!(categorize("Utility;TextEditor;"), "Accessories");
        assert_eq!(categorize(""), "Other");
    }

    #[test]
    fn visibility_rules() {
        use std::collections::HashMap;
        let mut e = HashMap::new();
        e.insert("Exec".to_string(), "x".to_string());
        assert!(is_visible(&e));

        e.insert("NoDisplay".to_string(), "true".to_string());
        assert!(!is_visible(&e));
        e.remove("NoDisplay");

        e.insert("Terminal".to_string(), "true".to_string());
        assert!(!is_visible(&e), "terminal apps would launch invisibly");
        e.remove("Terminal");

        e.insert("OnlyShowIn".to_string(), "GNOME;".to_string());
        assert!(!is_visible(&e));
        e.insert("OnlyShowIn".to_string(), "GNOME;KDE;".to_string());
        assert!(is_visible(&e));
    }

    #[test]
    fn scan_finds_real_apps_on_this_machine() {
        let apps = scan();
        assert!(apps.len() > 10, "only {} apps found", apps.len());
        assert!(apps.iter().any(|a| a.name.contains("Dolphin")));
        // Sorted case-insensitively.
        let names: Vec<String> = apps.iter().map(|a| a.name.to_lowercase()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn category_filtering() {
        let apps = scan();
        let cats = present_categories(&apps);
        assert_eq!(cats[0], "All");
        assert_eq!(filtered(&apps, "All").len(), apps.len());
        for c in cats.iter().skip(1) {
            assert!(!filtered(&apps, c).is_empty());
        }
    }
}
