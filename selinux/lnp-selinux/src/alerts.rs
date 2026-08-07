//! Reading setroubleshootd's alert database over D-Bus.
//!
//! setroubleshootd already does the hard part: it watches the audit log,
//! analyses every denial with its plugin set, and produces human-oriented
//! explanations plus suggested remedies. Duplicating that would be foolish.
//! This module just reads what it knows.

use crate::fixes::{self, Fix};

/// The tuple shape of `get_alert`, from the interface's own signature
/// `ssiasa(ssssbbi)tts`: uuid, summary, report count, audit events, plugin
/// analyses, first seen, last seen, level.
type PluginTuple = (String, String, String, String, bool, bool, i32);
type AlertTuple = (
    String,
    String,
    i32,
    Vec<String>,
    Vec<PluginTuple>,
    u64,
    u64,
    String,
);

#[zbus::proxy(
    interface = "org.fedoraproject.SetroubleshootdIface",
    default_service = "org.fedoraproject.Setroubleshootd",
    default_path = "/org/fedoraproject/Setroubleshootd"
)]
/// Every member carries an explicit `name`.
///
/// zbus would otherwise PascalCase them (`get_all_alerts` -> `GetAllAlerts`),
/// and setroubleshootd's D-Bus policy allowlists the snake_case names one by
/// one. A renamed member matches no allow rule, so the bus rejects the call
/// with "Sender is not authorized to send message" -- which reads like a
/// permissions problem and is really a spelling one.
pub trait Setroubleshootd {
    #[zbus(name = "get_all_alerts")]
    fn get_all_alerts(&self) -> zbus::Result<Vec<(String, String, i32)>>;

    #[zbus(name = "get_alert")]
    fn get_alert(&self, uuid: &str) -> zbus::Result<AlertTuple>;

    #[zbus(name = "delete_alert")]
    fn delete_alert(&self, uuid: &str) -> zbus::Result<bool>;

    /// Begin emitting `alert` signals.
    ///
    /// Not optional, and not documented anywhere obvious: without this call
    /// setroubleshootd records alerts but stays silent, which looks exactly
    /// like a broken subscription. Verified with `gdbus monitor` -- no
    /// signals before `start()`, signals immediately after.
    #[zbus(name = "start")]
    fn start(&self) -> zbus::Result<String>;

    #[zbus(signal, name = "alert")]
    fn alert(&self, level: String, uuid: String) -> zbus::Result<()>;
}

/// One suggested remedy, already parsed into validated actions.
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub if_text: String,
    pub then_text: String,
    pub raw_text: String,
    pub analysis_id: String,
    pub fixes: Vec<Fix>,
}

#[derive(Debug, Clone)]
pub struct SecurityAlert {
    pub uuid: String,
    pub summary: String,
    pub count: i32,
    pub level: String,
    pub last_seen: u64,
    pub suggestions: Vec<Suggestion>,
    /// The raw audit lines, for the "technical details" disclosure.
    pub audit: Vec<String>,
}

impl SecurityAlert {
    /// The plainest sentence we can manage about what happened.
    ///
    /// setroubleshoot's summaries read "SELinux is preventing X from Y" --
    /// accurate, but it leads with a product name nobody outside this world
    /// recognises. Rephrase around what the user cares about.
    pub fn plain_summary(&self) -> String {
        let s = self.summary.trim().trim_end_matches('.');
        if let Some(rest) = s.strip_prefix("SELinux is preventing ") {
            format!("The security system blocked {rest}.")
        } else {
            format!("{s}.")
        }
    }

    /// Best available action, preferring the safest.
    pub fn best_fix(&self) -> Option<&Fix> {
        self.suggestions
            .iter()
            .flat_map(|s| s.fixes.iter())
            .min_by_key(|f| f.risk())
    }
}

pub fn connect() -> zbus::Result<SetroubleshootdProxyBlocking<'static>> {
    let conn = zbus::blocking::Connection::system()?;
    SetroubleshootdProxyBlocking::new(&conn)
}

pub fn load_all(proxy: &SetroubleshootdProxyBlocking<'_>) -> zbus::Result<Vec<SecurityAlert>> {
    let mut alerts = Vec::new();

    for (uuid, _summary, _count) in proxy.get_all_alerts()? {
        match load_one(proxy, &uuid) {
            Ok(a) => alerts.push(a),
            Err(e) => eprintln!("lnp-selinux: could not read alert {uuid}: {e}"),
        }
    }

    // Most recent first: the thing that just happened is the thing they came
    // to look at.
    alerts.sort_by(|a, b| b.last_seen.cmp(&a.last_seen));
    Ok(alerts)
}

pub fn load_one(
    proxy: &SetroubleshootdProxyBlocking<'_>,
    uuid: &str,
) -> zbus::Result<SecurityAlert> {
    let (uuid, summary, count, audit, plugins, _first, last_seen, level) =
        proxy.get_alert(uuid)?;

    let suggestions = plugins
        .into_iter()
        .map(
            |(if_text, then_text, do_text, analysis_id, _fixable, _bug, _prio)| Suggestion {
                fixes: fixes::parse_do_text(&do_text),
                if_text,
                then_text,
                raw_text: do_text,
                analysis_id,
            },
        )
        .collect();

    Ok(SecurityAlert {
        uuid,
        summary,
        count,
        level,
        last_seen,
        suggestions,
        audit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alert(summary: &str) -> SecurityAlert {
        SecurityAlert {
            uuid: "u".into(),
            summary: summary.into(),
            count: 1,
            level: "yellow".into(),
            last_seen: 0,
            suggestions: Vec::new(),
            audit: Vec::new(),
        }
    }

    #[test]
    fn summaries_lose_the_product_name() {
        let a = alert("SELinux is preventing httpd from write access on the directory /var/www.");
        assert_eq!(
            a.plain_summary(),
            "The security system blocked httpd from write access on the directory /var/www."
        );
    }

    #[test]
    fn unusual_summaries_still_read_as_sentences() {
        let a = alert("Something else entirely");
        assert_eq!(a.plain_summary(), "Something else entirely.");
    }

    #[test]
    fn best_fix_prefers_the_safest() {
        let mut a = alert("x");
        a.suggestions = vec![Suggestion {
            if_text: String::new(),
            then_text: String::new(),
            raw_text: String::new(),
            analysis_id: "t".into(),
            fixes: vec![
                Fix::CustomModule { name: "m".into() },
                Fix::Restorecon { path: "/x".into(), recursive: false },
            ],
        }];
        assert!(matches!(a.best_fix(), Some(Fix::Restorecon { .. })));
    }
}
