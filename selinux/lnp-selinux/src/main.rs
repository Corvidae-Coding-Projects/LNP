//! lnp-selinux -- Security Alerts.
//!
//! When the built-in security system blocks something, this explains what
//! happened in ordinary words and offers the fix as a button. Two modes:
//!
//!   --watch   background service: notifies when a new alert appears, with a
//!             button that opens this window
//!   (default) the window: every current alert, with its fixes
//!
//! Nothing here runs setroubleshoot's suggested shell text. Suggestions are
//! parsed into a closed set of validated actions (see `fixes`), and applied
//! by a privileged helper that validates them all over again.

mod alerts;
mod fixes;

use std::collections::HashMap;
use std::process::Command;
use std::sync::mpsc::{Receiver, Sender, channel};

use alerts::SecurityAlert;
use eframe::egui;
use fixes::{Fix, Risk};

const APP_ID: &str = "org.lnp.selinux";

fn main() -> eframe::Result<()> {
    if std::env::args().any(|a| a == "--watch") {
        watch_mode();
        return Ok(());
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([760.0, 620.0])
            .with_min_inner_size([560.0, 380.0])
            .with_app_id(APP_ID)
            .with_title("Security Alerts"),
        ..Default::default()
    };

    eframe::run_native(
        "Security Alerts",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(App::new()))
        }),
    )
}

// ------------------------------------------------------------------- watch

#[zbus::proxy(
    interface = "org.freedesktop.Notifications",
    default_service = "org.freedesktop.Notifications",
    default_path = "/org/freedesktop/Notifications"
)]
trait Notifications {
    #[allow(clippy::too_many_arguments)]
    fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<&str>,
        hints: HashMap<&str, zbus::zvariant::Value<'_>>,
        timeout: i32,
    ) -> zbus::Result<u32>;

    #[zbus(signal)]
    fn action_invoked(&self, id: u32, action_key: String) -> zbus::Result<()>;
}

/// How often to look for new alerts. Polling rather than subscribing is a
/// deliberate choice, forced by how setroubleshootd actually behaves:
///
/// It is D-Bus activated and **exits when idle** -- `gdbus monitor` shows
/// "does not have an owner" moments after each alert. A signal subscription
/// is bound to one instance's unique bus name, so it goes deaf the first time
/// the daemon recycles, and the `start()` handshake that enables signals has
/// to be redone by every new instance. Polling sidesteps the whole lifecycle:
/// each call simply activates the daemon again if needed.
///
/// Half a minute is imperceptible for something whose message is "nothing is
/// broken, look when convenient".
const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

/// Background mode: one notification per new alert, with a button that opens
/// the window. Deliberately calm -- it says what was blocked and that nothing
/// is broken, because that is true nearly every time.
fn watch_mode() {
    let session = match zbus::blocking::Connection::session() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("lnp-selinux: no session bus: {e}");
            return;
        }
    };
    let notifier = match NotificationsProxyBlocking::new(&session) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("lnp-selinux: no notification service: {e}");
            return;
        }
    };

    // Track which notification ids are ours, so a click on somebody else's
    // notification never opens our window.
    let ours: std::sync::Arc<std::sync::Mutex<Vec<u32>>> = Default::default();

    {
        let ours = ours.clone();
        let session2 = session.clone();
        std::thread::spawn(move || {
            let Ok(n) = NotificationsProxyBlocking::new(&session2) else {
                return;
            };
            let Ok(signals) = n.receive_action_invoked() else {
                return;
            };
            for sig in signals {
                let Ok(args) = sig.args() else { continue };
                let mine = ours.lock().map(|v| v.contains(&args.id)).unwrap_or(false);
                if args.action_key == "show" && mine {
                    let _ = Command::new("setsid")
                        .arg("-f")
                        .arg(current_exe_path())
                        .spawn();
                }
            }
        });
    }

    // Everything already on record at startup is history, not news. Seeding
    // from it means a login never dumps a backlog of old notifications.
    let mut seen: std::collections::HashSet<String> = match alerts::connect()
        .and_then(|p| alerts::load_all(&p))
    {
        Ok(list) => list.into_iter().map(|a| a.uuid).collect(),
        Err(e) => {
            eprintln!("lnp-selinux: cannot reach the security service yet ({e}); will retry");
            Default::default()
        }
    };

    eprintln!(
        "lnp-selinux: watching for security alerts ({} already on record)",
        seen.len()
    );

    loop {
        std::thread::sleep(POLL_INTERVAL);

        // Reconnect every round: the daemon comes and goes, and a proxy that
        // outlived its instance is worse than a fresh one.
        let proxy = match alerts::connect() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("lnp-selinux: security service unreachable: {e}");
                continue;
            }
        };
        let current = match alerts::load_all(&proxy) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("lnp-selinux: could not read alerts: {e}");
                continue;
            }
        };

        for alert in &current {
            if !seen.insert(alert.uuid.clone()) {
                continue;
            }

            let body = format!(
                "{}\n\nNothing is broken and your files are safe. \
                 Open Security Alerts if something you were using stopped working.",
                alert.plain_summary()
            );

            let mut hints: HashMap<&str, zbus::zvariant::Value<'_>> = HashMap::new();
            hints.insert("desktop-entry", zbus::zvariant::Value::from(APP_ID));

            match notifier.notify(
                "Security Alerts",
                0,
                "security-high",
                "Something was blocked for your protection",
                &body,
                vec!["show", "Show me"],
                hints,
                20_000,
            ) {
                Ok(id) => {
                    eprintln!("lnp-selinux: notified about {}", alert.uuid);
                    if let Ok(mut v) = ours.lock() {
                        v.push(id);
                    }
                }
                Err(e) => eprintln!("lnp-selinux: could not notify: {e}"),
            }
        }
    }
}

fn current_exe_path() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "/usr/bin/lnp-selinux".into())
}

// --------------------------------------------------------------------- gui

enum FixOutcome {
    Ok(String),
    Failed(String),
    Cancelled,
}

struct App {
    alerts: Vec<SecurityAlert>,
    error: Option<String>,
    expanded: Option<String>,
    /// A high-risk fix awaiting explicit confirmation.
    confirming: Option<Fix>,
    busy: bool,
    outcome: Option<FixOutcome>,
    tx: Sender<FixOutcome>,
    rx: Receiver<FixOutcome>,
}

impl App {
    fn new() -> Self {
        let (tx, rx) = channel();
        let mut app = Self {
            alerts: Vec::new(),
            error: None,
            expanded: None,
            confirming: None,
            busy: false,
            outcome: None,
            tx,
            rx,
        };
        app.reload();
        app
    }

    fn reload(&mut self) {
        match alerts::connect().and_then(|p| alerts::load_all(&p)) {
            Ok(a) => {
                self.alerts = a;
                self.error = None;
            }
            Err(e) => {
                self.error = Some(format!(
                    "The security service is not answering ({e}). It may not \
                     be installed yet -- open the Welcome app and run the \
                     security checkup."
                ));
            }
        }
    }

    /// Apply a fix on a worker thread; pkexec shows the system's own password
    /// dialog and the window must stay responsive behind it.
    fn apply(&mut self, fix: &Fix) {
        self.busy = true;
        self.outcome = None;
        let args = fix.helper_args();
        let tx = self.tx.clone();

        std::thread::spawn(move || {
            let out = Command::new("pkexec")
                .arg("--disable-internal-agent")
                .arg("/usr/libexec/lnp-selinux-fix")
                .args(&args)
                .output();

            let result = match out {
                Ok(o) if o.status.success() => {
                    FixOutcome::Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
                }
                Ok(o) if matches!(o.status.code(), Some(126) | Some(127)) => {
                    FixOutcome::Cancelled
                }
                Ok(o) => {
                    let mut msg = String::from_utf8_lossy(&o.stderr).trim().to_string();
                    if msg.is_empty() {
                        msg = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    }
                    FixOutcome::Failed(msg)
                }
                Err(e) => FixOutcome::Failed(e.to_string()),
            };
            let _ = tx.send(result);
        });
    }
}

fn risk_colour(risk: Risk) -> egui::Color32 {
    match risk {
        Risk::Safe => egui::Color32::from_rgb(120, 200, 130),
        Risk::Moderate => egui::Color32::from_rgb(225, 195, 90),
        Risk::High => egui::Color32::from_rgb(230, 130, 120),
    }
}

fn first_line(s: &str) -> String {
    s.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim()
        .to_string()
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let ctx = &ctx;
        if let Ok(outcome) = self.rx.try_recv() {
            self.busy = false;
            self.outcome = Some(outcome);
            self.reload();
        }
        if self.busy {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }

        // Confirmation for the one fix that could matter.
        if let Some(fix) = self.confirming.clone() {
            egui::Window::new("Are you sure?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_max_width(460.0);
                    ui.label(fix.explanation());
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.confirming = None;
                        }
                        if ui
                            .button(
                                egui::RichText::new("Allow permanently")
                                    .color(risk_colour(Risk::High)),
                            )
                            .clicked()
                        {
                            let f = fix.clone();
                            self.confirming = None;
                            self.apply(&f);
                        }
                    });
                });
        }

        egui::Panel::top("head").show(ui, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("Security Alerts");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Refresh").clicked() {
                        self.reload();
                    }
                });
            });
            ui.label(
                egui::RichText::new(
                    "Your computer blocks actions that look out of place. Most \
                     of these are harmless mix-ups you can repair here.",
                )
                .weak(),
            );
            ui.add_space(8.0);
        });

        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(err) = self.error.clone() {
                ui.colored_label(risk_colour(Risk::High), err);
                return;
            }

            match &self.outcome {
                Some(FixOutcome::Ok(msg)) => {
                    ui.colored_label(
                        risk_colour(Risk::Safe),
                        format!("Done. {}", first_line(msg)),
                    );
                    ui.add_space(6.0);
                }
                Some(FixOutcome::Cancelled) => {
                    ui.label(
                        "That needs the administrator password, and it wasn't \
                         entered. Nothing was changed.",
                    );
                    ui.add_space(6.0);
                }
                Some(FixOutcome::Failed(msg)) => {
                    ui.colored_label(
                        risk_colour(Risk::High),
                        format!(
                            "That didn't work, and nothing was changed. {}",
                            first_line(msg)
                        ),
                    );
                    ui.add_space(6.0);
                }
                None => {}
            }

            if self.busy {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Applying the fix…");
                });
                ui.add_space(6.0);
            }

            if self.alerts.is_empty() {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("Nothing has been blocked.").size(18.0));
                    ui.label(egui::RichText::new("That is the good outcome.").weak());
                });
                return;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                let alerts = self.alerts.clone();
                for alert in &alerts {
                    self.alert_card(ui, alert);
                    ui.add_space(10.0);
                }
            });
        });
    }
}

impl App {
    fn alert_card(&mut self, ui: &mut egui::Ui, alert: &SecurityAlert) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label(egui::RichText::new(alert.plain_summary()).size(15.0).strong());
            if alert.count > 1 {
                ui.label(
                    egui::RichText::new(format!("Happened {} times.", alert.count)).weak(),
                );
            }
            ui.add_space(6.0);

            // Offer each distinct action once, safest first.
            let mut seen: Vec<Fix> = Vec::new();
            for suggestion in &alert.suggestions {
                for fix in &suggestion.fixes {
                    if !seen.contains(fix) {
                        seen.push(fix.clone());
                    }
                }
            }
            seen.sort_by_key(|f| f.risk());

            if seen.is_empty() {
                ui.label(
                    egui::RichText::new(
                        "There is no safe automatic fix for this one. If a \
                         program you were using stopped working, show this to \
                         whoever helps you with this computer.",
                    )
                    .weak(),
                );
            }

            for fix in &seen {
                let enabled = !self.busy;
                let button = egui::Button::new(
                    egui::RichText::new(fix.button_label()).color(risk_colour(fix.risk())),
                );
                if ui.add_enabled(enabled, button).clicked() {
                    if fix.risk() == Risk::High {
                        self.confirming = Some(fix.clone());
                    } else {
                        self.apply(fix);
                    }
                }
                ui.label(egui::RichText::new(fix.explanation()).weak().size(12.5));
                ui.add_space(4.0);
            }

            // Technical detail stays available and stays out of the way.
            let expanded = self.expanded.as_deref() == Some(alert.uuid.as_str());
            if ui
                .small_button(if expanded {
                    "Hide technical details"
                } else {
                    "Technical details"
                })
                .clicked()
            {
                self.expanded = if expanded { None } else { Some(alert.uuid.clone()) };
            }

            if expanded {
                for s in &alert.suggestions {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(&s.if_text).size(12.0).weak());
                    ui.label(egui::RichText::new(&s.then_text).size(12.0).weak());
                    ui.label(
                        egui::RichText::new(&s.raw_text).size(11.5).monospace().weak(),
                    );
                }
                for line in &alert.audit {
                    ui.label(egui::RichText::new(line).size(11.0).monospace().weak());
                }
            }
        });
    }
}
