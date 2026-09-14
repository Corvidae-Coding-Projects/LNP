//! Turning setroubleshoot's suggested commands into validated actions.
//!
//! setroubleshoot hands us a `do_text` blob -- shell commands meant for a
//! human to read and retype. We never run that text. It is parsed into a
//! small closed set of actions with strictly validated arguments, and the
//! privileged helper accepts only those, by name, with no shell involved.
//! Anything unrecognised is shown to the user as text and left unrunnable.
//!
//! The risk grading matters as much as the parsing:
//!
//! * `Restorecon` puts a file's label back to what policy already says it
//!   should be. It grants nothing new -- it repairs mislabelling, which is
//!   the single most common cause of these alerts.
//! * `SetBool` flips a switch the policy authors shipped for exactly this
//!   purpose. Documented, reversible, bounded.
//! * `Fcontext` records a new expected label for a path, then applies it.
//!   Narrow, but it does change policy expectations.
//! * `CustomModule` generates policy from the denial displayed in one alert.
//!   The privileged helper re-fetches that alert and verifies the exact record.

#[derive(Debug, Clone, PartialEq)]
pub enum Fix {
    /// setsebool -P <name> on|off
    SetBool { name: String, value: bool },
    /// restorecon [-R] -v <path>
    Restorecon { path: String, recursive: bool },
    /// semanage fcontext -a -t <setype> '<path>' ; restorecon -v '<path>'
    Fcontext { setype: String, path: String },
    CustomModule { alert_id: String, last_seen: u64, audit_record: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Risk {
    /// Repairs to match existing policy. Grants nothing new.
    Safe,
    /// A supported, reversible policy switch or narrow labelling rule.
    Moderate,
    /// A persistent type-based permission rule. Requires explicit confirmation.
    High,
}

impl Fix {
    pub fn risk(&self) -> Risk {
        match self {
            Fix::Restorecon { .. } => Risk::Safe,
            Fix::SetBool { .. } | Fix::Fcontext { .. } => Risk::Moderate,
            Fix::CustomModule { .. } => Risk::High,
        }
    }

    /// The button's words. No jargon, and it says what will actually happen.
    pub fn button_label(&self) -> String {
        match self {
            Fix::Restorecon { .. } => "Repair this file's security label".into(),
            Fix::SetBool { name, value } => {
                let verb = if *value { "Turn on" } else { "Turn off" };
                format!("{verb} the “{}” permission", humanize_bool(name))
            }
            Fix::Fcontext { .. } => "Correct this file's security type".into(),
            Fix::CustomModule { .. } => "Allow this permanently…".into(),
        }
    }

    /// What the user is agreeing to, in a sentence.
    pub fn explanation(&self) -> String {
        match self {
            Fix::Restorecon { path, .. } => format!(
                "Puts the security label on {path} back to what the system \
                 expects. This grants no new permissions -- it repairs a \
                 file that was labelled wrongly, which is the usual cause of \
                 these messages."
            ),
            Fix::SetBool { name, value } => format!(
                "Switches the built-in “{}” permission {}. This is a setting \
                 the system was designed to have changed, and it can be \
                 changed back.",
                humanize_bool(name),
                if *value { "on" } else { "off" }
            ),
            Fix::Fcontext { setype, path } => format!(
                "Records that {path} should be treated as “{setype}”, and \
                 applies it. Affects only that location."
            ),
            Fix::CustomModule { .. } =>
                "Creates a permanent permission rule from the denial shown in this alert. \
                 SELinux rules apply to security types, so other programs and files with \
                 those same types may also gain this access. Only continue if this was \
                 something you intended to allow.".into(),
        }
    }

    /// Arguments handed to the privileged helper. Every element is validated;
    /// nothing is ever concatenated into a shell command.
    pub fn helper_args(&self) -> Vec<String> {
        match self {
            Fix::SetBool { name, value } => vec![
                "setbool".into(),
                name.clone(),
                if *value { "on".into() } else { "off".into() },
            ],
            Fix::Restorecon { path, recursive } => {
                let mut a = vec!["relabel".to_string()];
                if *recursive {
                    a.push("--recursive".into());
                }
                a.push(path.clone());
                a
            }
            Fix::Fcontext { setype, path } => {
                vec!["fcontext".into(), setype.clone(), path.clone()]
            }
            Fix::CustomModule { alert_id, last_seen, audit_record } => vec![
                "allow-module".into(), alert_id.clone(), last_seen.to_string(), audit_record.clone(),
            ],
        }
    }
}

/// "httpd_can_network_connect" -> "httpd can network connect"
fn humanize_bool(name: &str) -> String {
    name.replace('_', " ")
}

fn valid_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Paths must be absolute and free of anything that could matter to a shell,
/// even though the helper never uses one. Defence in depth: if this function
/// is ever wrong, the helper's own validation still stands.
fn valid_path(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 4096
        && s.starts_with('/')
        && !s.contains("..")
        && !s.chars().any(|c| {
            matches!(
                c,
                ';' | '&' | '|' | '$' | '`' | '\n' | '\r' | '<' | '>' | '(' | ')' | '"' | '\\'
            )
        })
}

fn unquote(s: &str) -> &str {
    s.trim().trim_matches('\'').trim_matches('"')
}

/// Custom policy needs trusted alert context, not merely a suggested module
/// name. Multiple denial records are ambiguous in this API: get_alert returns
/// the entire audit event, which can contain denials for different alerts.
pub fn parse_alert_fixes(text: &str, alert_id: &str, last_seen: u64, audit: &[String]) -> Vec<Fix> {
    let mut fixes = parse_do_text(text);
    let suggests_module = text.lines().any(|line| {
        let tokens: Vec<_> = line.split_whitespace().collect();
        tokens.windows(3).any(|parts| {
            parts[0] == "audit2allow" && parts[1] == "-M" && valid_identifier(unquote(parts[2]))
        })
    });
    let denials: Vec<_> = audit.iter().filter(|record| {
        ["type=AVC ", "type=USER_AVC ", "type=1400 ", "type=1107 "]
            .iter().any(|prefix| record.starts_with(prefix))
    }).collect();
    if suggests_module && denials.len() == 1 && !alert_id.is_empty() {
        fixes.push(Fix::CustomModule {
            alert_id: alert_id.into(), last_seen, audit_record: denials[0].clone(),
        });
    }
    fixes
}

/// Extract every action we recognise from a suggestion blob.
pub fn parse_do_text(text: &str) -> Vec<Fix> {
    let mut out: Vec<Fix> = Vec::new();
    let mut pending_fcontext: Option<(String, String)> = None;

    for raw in text.lines() {
        // Suggestion lines are shown as shell prompts.
        let line = raw.trim().trim_start_matches('#').trim();
        if line.is_empty() {
            continue;
        }

        let tokens: Vec<&str> = line.split_whitespace().collect();

        if tokens.first() == Some(&"setsebool") {
            // setsebool -P name on
            let args: Vec<&str> = tokens.iter().skip(1).copied().filter(|t| *t != "-P").collect();
            if args.len() >= 2 {
                let name = unquote(args[0]);
                let value = match unquote(args[1]) {
                    "on" | "1" | "true" => Some(true),
                    "off" | "0" | "false" => Some(false),
                    _ => None,
                };
                if let (true, Some(value)) = (valid_identifier(name), value) {
                    out.push(Fix::SetBool { name: name.to_string(), value });
                }
            }
            continue;
        }

        if tokens.iter().any(|t| t.ends_with("restorecon")) {
            let recursive = tokens.contains(&"-R");
            if let Some(path) = tokens.iter().rev().find(|t| t.starts_with('/') || t.starts_with('\'') || t.starts_with('"')) {
                let path = unquote(path);
                if valid_path(path) {
                    // A restorecon that follows a semanage fcontext is the
                    // second half of one action, not a separate offer.
                    if let Some((setype, fpath)) = pending_fcontext.take() {
                        if fpath == path {
                            out.push(Fix::Fcontext { setype, path: path.to_string() });
                            continue;
                        }
                        pending_fcontext = None;
                    }
                    out.push(Fix::Restorecon { path: path.to_string(), recursive });
                }
            }
            continue;
        }

        if tokens.first() == Some(&"semanage") && tokens.get(1) == Some(&"fcontext") {
            let mut setype = None;
            let mut path = None;
            let mut it = tokens.iter().skip(2).peekable();
            while let Some(t) = it.next() {
                match *t {
                    "-t" => setype = it.next().map(|s| unquote(s).to_string()),
                    other if other.starts_with('/') || other.starts_with('\'') => {
                        path = Some(unquote(other).to_string())
                    }
                    _ => {}
                }
            }
            if let (Some(setype), Some(path)) = (setype, path) {
                if valid_identifier(&setype) && valid_path(&path) {
                    pending_fcontext = Some((setype, path));
                }
            }
            continue;
        }

    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_boolean_suggestion() {
        let t = "Allow this by executing:\n# setsebool -P httpd_can_network_connect 1\n";
        assert_eq!(
            parse_do_text(t),
            vec![Fix::SetBool { name: "httpd_can_network_connect".into(), value: true }]
        );
        assert_eq!(parse_do_text(t)[0].risk(), Risk::Moderate);
    }

    #[test]
    fn parses_restorecon() {
        let t = "# /sbin/restorecon -v /srv/www/index.html";
        let fixes = parse_do_text(t);
        assert_eq!(
            fixes,
            vec![Fix::Restorecon { path: "/srv/www/index.html".into(), recursive: false }]
        );
        assert_eq!(fixes[0].risk(), Risk::Safe, "relabelling grants nothing new");
    }

    #[test]
    fn fcontext_and_its_restorecon_are_one_action() {
        let t = "# semanage fcontext -a -t httpd_sys_content_t '/srv/web'\n\
                 # restorecon -v '/srv/web'";
        assert_eq!(
            parse_do_text(t),
            vec![Fix::Fcontext { setype: "httpd_sys_content_t".into(), path: "/srv/web".into() }]
        );
    }

    #[test]
    fn custom_policy_requires_selected_alert_context() {
        let t = "Allow this access for now by executing:\n\
                 # ausearch -c '(cat)' --raw | audit2allow -M my-cat\n\
                 # semodule -X 300 -i my-cat.pp\n";
        assert!(parse_do_text(t).is_empty());
        assert!(parse_do_text("# audit2allow -M local-policy").is_empty());
        assert!(parse_do_text("# semodule -i local-policy.pp").is_empty());
        let audit = vec!["type=AVC msg=audit(1.2:3): avc: denied { read } for test\n".into()];
        let fixes = parse_alert_fixes(t, "selected-alert", 42, &audit);
        assert_eq!(fixes, vec![Fix::CustomModule {
            alert_id: "selected-alert".into(), last_seen: 42, audit_record: audit[0].clone(),
        }]);
        assert_eq!(fixes[0].risk(), Risk::High);
        assert_eq!(fixes[0].helper_args(), vec!["allow-module", "selected-alert", "42", &audit[0]]);
        assert!(parse_alert_fixes(t, "selected-alert", 42, &[]).is_empty());
        assert!(parse_alert_fixes(t, "selected-alert", 42, &[audit[0].clone(), audit[0].clone()]).is_empty());
    }

    #[test]
    fn rejects_injection_attempts() {
        // Nothing here should survive validation.
        for evil in [
            "# setsebool -P httpd;rm -rf / on",
            "# restorecon -v /tmp/x;reboot",
            "# restorecon -v '/tmp/$(id)'",
            "# restorecon -v /../../etc/shadow",
            "# audit2allow -M ../../evil",
            "# restorecon -v relative/path",
        ] {
            let fixes = parse_do_text(evil);
            for f in &fixes {
                match f {
                    Fix::SetBool { name, .. } => assert!(valid_identifier(name), "{evil}"),
                    Fix::Restorecon { path, .. } => assert!(valid_path(path), "{evil}"),
                    Fix::Fcontext { setype, path } => {
                        assert!(valid_identifier(setype) && valid_path(path), "{evil}")
                    }
                    Fix::CustomModule { .. } => panic!("custom policy requires alert context"),
                }
            }
        }
    }

    #[test]
    fn unknown_text_yields_nothing_runnable() {
        assert!(parse_do_text("You should report this as a bug.").is_empty());
        assert!(parse_do_text("").is_empty());
    }

    #[test]
    fn helper_args_are_separate_tokens() {
        let f = Fix::Restorecon { path: "/a b/c".into(), recursive: true };
        assert_eq!(f.helper_args(), vec!["relabel", "--recursive", "/a b/c"]);
    }
}
