//! Visão "Ralos": Defender, serviços dispensáveis e apps de sistema.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

use egui::{Color32, RichText};

use crate::app::{fmt_bytes, fmt_bytes_short, LINE, MUTED, SURFACE, SURFACE_HI};
use crate::procs::ProcInfo;
use crate::sys::{self, DefenderStatus, SvcStart, SvcState, SvcStatus, SysResult};

/// Eventos que a visão devolve para o App tratar.
pub enum DrainOut {
    Toast(String, bool),
    Kill(Vec<u32>),
}

enum Action {
    SvcStop(&'static str),
    SvcDisable(&'static str),
    SvcEnable(&'static str),
    DefenderExclude(Vec<String>),
    DefenderCpu(u32),
    DefenderRealtime(bool),
    AppxRemove(&'static str),
}

struct Pending {
    title: String,
    lines: Vec<String>,
    action: Action,
}

pub struct Drains {
    svc: Vec<SvcStatus>,
    protected: Vec<SvcStatus>,
    defender: DefenderStatus,
    appx: HashSet<String>,
    last_refresh: Option<Instant>,
    tx: Sender<SysResult>,
    rx: Receiver<SysResult>,
    busy: usize,
    pending: Option<Pending>,
    exclusions_text: String,
    exclusions_seeded: bool,
}

fn service_state(state: SvcState) -> &'static str {
    match state {
        SvcState::Running => "running",
        SvcState::Stopped => "stopped",
        SvcState::Pending => "pending",
        SvcState::Missing => "missing",
    }
}

fn service_start(start: SvcStart) -> &'static str {
    match start {
        SvcStart::Auto => "automatic",
        SvcStart::Manual => "manual",
        SvcStart::Disabled => "disabled",
        SvcStart::Unknown => "unknown",
    }
}

impl Drains {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            svc: Vec::new(),
            protected: Vec::new(),
            defender: DefenderStatus::default(),
            appx: HashSet::new(),
            last_refresh: None,
            tx,
            rx,
            busy: 0,
            pending: None,
            exclusions_text: String::new(),
            exclusions_seeded: false,
        }
    }

    pub fn refresh(&mut self) {
        self.svc = sys::SERVICES
            .iter()
            .map(|e| sys::query_service(e.name))
            .collect();
        self.protected = sys::PROTECTED_SERVICES
            .iter()
            .map(|(n, _)| sys::query_service(n))
            .collect();
        self.defender = sys::defender_status();
        self.appx = sys::installed_appx_families().into_iter().collect();
        self.last_refresh = Some(Instant::now());
    }

    pub fn snapshot_json(&mut self) -> serde_json::Value {
        self.refresh();
        let services = sys::SERVICES
            .iter()
            .zip(&self.svc)
            .map(|(entry, status)| {
                serde_json::json!({
                    "name": entry.name,
                    "label": entry.label,
                    "why": entry.why,
                    "process_hint": entry.proc_hint,
                    "stop_only": entry.stop_only,
                    "state": service_state(status.state),
                    "start": service_start(status.start),
                })
            })
            .collect::<Vec<_>>();
        let protected = sys::PROTECTED_SERVICES
            .iter()
            .zip(&self.protected)
            .map(|((name, label), status)| {
                serde_json::json!({
                    "name": name,
                    "label": label,
                    "state": service_state(status.state),
                    "start": service_start(status.start),
                })
            })
            .collect::<Vec<_>>();
        let apps = sys::APPX
            .iter()
            .map(|app| serde_json::json!({
                "label": app.label,
                "package": app.pkg_name,
                "why": app.why,
                "installed": self.appx.iter().any(|family| family.starts_with(app.family_prefix)),
            }))
            .collect::<Vec<_>>();
        serde_json::json!({
            "supported": true,
            "defender": {
                "realtime_disabled": self.defender.realtime_disabled,
                "tamper_protection": self.defender.tamper_protection,
                "scan_cpu_factor": self.defender.scan_cpu_factor,
            },
            "services": services,
            "protected_services": protected,
            "system_apps": apps,
        })
    }

    fn maybe_refresh(&mut self) {
        let due = self
            .last_refresh
            .map(|t| t.elapsed() > Duration::from_secs(5))
            .unwrap_or(true);
        if due {
            self.refresh();
        }
    }

    /// Sugere pastas de projeto / agentes vistas nos processos atuais.
    fn seed_exclusions(&mut self, procs: &[ProcInfo]) {
        if self.exclusions_seeded {
            return;
        }
        self.exclusions_seeded = true;
        let home = std::env::var("USERPROFILE").unwrap_or_default();
        let mut set: BTreeSet<String> = BTreeSet::new();
        for p in procs {
            if let Some(cwd) = &p.launcher.init_cwd {
                if !cwd.is_empty() {
                    set.insert(cwd.clone());
                }
            }
        }
        for d in [
            ".claude",
            ".codex",
            ".cargo",
            ".rustup",
            ".grok",
            "AppData\\Roaming\\npm",
            "AppData\\Local\\hermes",
            ".buzz",
        ] {
            let path = format!("{home}\\{d}");
            if std::path::Path::new(&path).is_dir() {
                set.insert(path);
            }
        }
        self.exclusions_text = set.into_iter().collect::<Vec<_>>().join("\n");
    }

    fn run(&mut self, action: Action, is_admin: bool, out: &mut Vec<DrainOut>) {
        // Ações diretas quando dá (sem UAC); senão PowerShell elevado.
        let elevated = |label: &str, script: String, this: &mut Self| {
            this.busy += 1;
            sys::run_elevated_ps(label.to_string(), script, this.tx.clone());
        };
        match action {
            Action::SvcStop(name) => {
                if is_admin {
                    match sys::stop_service(name) {
                        Ok(()) => out.push(DrainOut::Toast(format!("{name}: stopped"), false)),
                        Err(e) => out.push(DrainOut::Toast(format!("{name}: {e}"), true)),
                    }
                    self.last_refresh = None;
                } else {
                    elevated(
                        &format!("{name}: stop"),
                        format!("Stop-Service -Name {} -Force", sys::ps_quote(name)),
                        self,
                    );
                }
            }
            Action::SvcDisable(name) => {
                if is_admin {
                    let r = sys::set_start_type(name, SvcStart::Disabled).and_then(|_| {
                        match sys::stop_service(name) {
                            Err(e) if e != "already stopped" => Err(e),
                            _ => Ok(()),
                        }
                    });
                    match r {
                        Ok(()) => out.push(DrainOut::Toast(
                            format!("{name}: disabled (will not start again)"),
                            false,
                        )),
                        Err(e) => out.push(DrainOut::Toast(format!("{name}: {e}"), true)),
                    }
                    self.last_refresh = None;
                } else {
                    elevated(
                        &format!("{name}: disable"),
                        format!("Set-Service -Name {0} -StartupType Disabled; Stop-Service -Name {0} -Force -ErrorAction SilentlyContinue", sys::ps_quote(name)),
                        self,
                    );
                }
            }
            Action::SvcEnable(name) => {
                if is_admin {
                    let r = sys::set_start_type(name, SvcStart::Auto)
                        .and_then(|_| sys::start_service(name));
                    match r {
                        Ok(()) => out.push(DrainOut::Toast(format!("{name}: re-enabled"), false)),
                        Err(e) => out.push(DrainOut::Toast(format!("{name}: {e}"), true)),
                    }
                    self.last_refresh = None;
                } else {
                    elevated(
                        &format!("{name}: re-enable"),
                        format!(
                            "Set-Service -Name {0} -StartupType Automatic; Start-Service -Name {0}",
                            sys::ps_quote(name)
                        ),
                        self,
                    );
                }
            }
            Action::DefenderExclude(paths) => {
                let list = paths
                    .iter()
                    .map(|p| sys::ps_quote(p))
                    .collect::<Vec<_>>()
                    .join(",");
                elevated(
                    "Defender: exclusions",
                    format!("Add-MpPreference -ExclusionPath {list}"),
                    self,
                );
            }
            Action::DefenderCpu(f) => {
                elevated(
                    "Defender: scan CPU",
                    format!("Set-MpPreference -ScanAvgCPULoadFactor {f}"),
                    self,
                );
            }
            Action::DefenderRealtime(disable) => {
                elevated(
                    if disable {
                        "Defender: pause real-time"
                    } else {
                        "Defender: re-enable real-time"
                    },
                    format!(
                        "Set-MpPreference -DisableRealtimeMonitoring ${}",
                        if disable { "true" } else { "false" }
                    ),
                    self,
                );
            }
            Action::AppxRemove(pkg) => {
                // Remove-AppxPackage do usuário atual não exige admin, mas rodamos elevado para
                // cobrir pacotes provisionados (-AllUsers) e ter um único caminho de erro.
                elevated(
                    &format!("uninstall {pkg}"),
                    format!(
                        "Get-AppxPackage -Name {0} -AllUsers | Remove-AppxPackage -AllUsers",
                        sys::ps_quote(pkg)
                    ),
                    self,
                );
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, procs: &[ProcInfo], is_admin: bool) -> Vec<DrainOut> {
        let mut out = Vec::new();
        self.maybe_refresh();
        self.seed_exclusions(procs);
        while let Ok(r) = self.rx.try_recv() {
            self.busy = self.busy.saturating_sub(1);
            match r.result {
                Ok(()) => out.push(DrainOut::Toast(format!("{}: ok", r.label), false)),
                Err(e) => out.push(DrainOut::Toast(format!("{}: {e}", r.label), true)),
            }
            self.last_refresh = None;
        }

        // índice por nome de processo → (RAM, CPU, pids)
        let mut by_name: HashMap<String, (u64, f32, Vec<u32>)> = HashMap::new();
        for p in procs {
            let e = by_name.entry(p.name_lower.clone()).or_default();
            e.0 += p.private_ws;
            e.1 += p.cpu_pct;
            e.2.push(p.pid);
        }
        let mut svchost_hint = HashMap::new();
        for p in procs {
            if p.name_lower == "svchost.exe" {
                let cl = p.cmdline.to_lowercase();
                for e in sys::SERVICES {
                    if cl.contains(&format!("-s {}", e.name.to_lowercase())) {
                        let x = svchost_hint
                            .entry(e.name)
                            .or_insert((0u64, 0f32, Vec::new()));
                        x.0 += p.private_ws;
                        x.1 += p.cpu_pct;
                        x.2.push(p.pid);
                    }
                }
            }
        }

        let muted = MUTED;
        let accent = Color32::from_rgb(232, 178, 92);
        let ok_c = Color32::from_rgb(120, 200, 140);
        let warn_c = Color32::from_rgb(232, 120, 100);
        let mut queued: Vec<Action> = Vec::new();
        let mut confirm: Option<Pending> = None;

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Windows drains").strong().size(16.0));
                ui.label(RichText::new("— what uses RAM/CPU without being asked, and what you can do about it").color(muted));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Refresh").clicked() {
                        self.last_refresh = None;
                    }
                    if self.busy > 0 {
                        ui.spinner();
                        ui.label(RichText::new(format!("{} action(s) waiting for UAC/PowerShell", self.busy)).color(muted).small());
                    }
                    if !is_admin {
                        ui.label(RichText::new("not elevated: each action opens a UAC prompt").color(muted).small());
                    }
                });
            });
            ui.add_space(8.0);

            // ---------- Defender ----------
            let (mp_ram, mp_cpu, _) = by_name.get("msmpeng.exe").cloned().unwrap_or((0, 0.0, Vec::new()));
            section(ui, "Microsoft Defender", &format!("MsMpEng.exe {} · CPU {:.1}%", fmt_bytes(mp_ram), mp_cpu), |ui| {
                ui.label(RichText::new("Kernel-protected process: even an administrator cannot terminate it, and the WinDefend service cannot be stopped. What works is reducing its workload:").color(muted));
                ui.add_space(4.0);
                let d = self.defender.clone();
                ui.horizontal(|ui| {
                    pill(ui, "real-time", match d.realtime_disabled { Some(true) => ("paused", warn_c), Some(false) => ("active", ok_c), None => ("?", muted) });
                    pill(ui, "tamper protection", match d.tamper_protection { Some(true) => ("on", accent), Some(false) => ("off", muted), None => ("?", muted) });
                    pill(ui, "scheduled scan CPU", (&format!("{}%", d.scan_cpu_factor.map(|v| v.to_string()).unwrap_or_else(|| "50 (default)".into())), muted));
                });
                ui.add_space(6.0);

                ui.label(RichText::new("1. Exclude project/agent folders from real-time scanning").strong());
                ui.label(RichText::new("This is where Defender spends CPU/RAM: every file node/cargo/git touches is scanned. One folder per line; edit freely.").color(muted).small());
                ui.add(egui::TextEdit::multiline(&mut self.exclusions_text).desired_rows(4).desired_width(f32::INFINITY).font(egui::TextStyle::Monospace));
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(RichText::new("Add exclusions").strong())).clicked() {
                        let paths: Vec<String> = self.exclusions_text.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
                        if paths.is_empty() {
                            out.push(DrainOut::Toast("No folders provided".into(), true));
                        } else {
                            confirm = Some(Pending {
                                title: "Exclude folders from Defender scanning".into(),
                                lines: paths.clone(),
                                action: Action::DefenderExclude(paths),
                            });
                        }
                    }
                    ui.label(RichText::new("Add-MpPreference -ExclusionPath · files in these folders will not be scanned").color(muted).small());
                });
                ui.add_space(6.0);

                ui.label(RichText::new("2. Limit scheduled scan CPU").strong());
                ui.horizontal(|ui| {
                    for f in [5u32, 10, 20] {
                        if ui.button(format!("{f}%")).clicked() {
                            queued.push(Action::DefenderCpu(f));
                        }
                    }
                    ui.label(RichText::new("Set-MpPreference -ScanAvgCPULoadFactor · applies to full/scheduled scans").color(muted).small());
                });
                ui.add_space(6.0);

                ui.label(RichText::new("3. Pause real-time protection").strong());
                ui.horizontal(|ui| {
                    let tp_on = d.tamper_protection == Some(true);
                    let paused = d.realtime_disabled == Some(true);
                    if paused {
                        if ui.button("Re-enable real-time").clicked() {
                            queued.push(Action::DefenderRealtime(false));
                        }
                    } else {
                        let b = ui.add_enabled(!tp_on, egui::Button::new(RichText::new("Pause real-time").color(warn_c)));
                        if b.clicked() {
                            confirm = Some(Pending {
                                title: "Pause Defender real-time protection".into(),
                                lines: vec!["Files and downloads will not be scanned until you re-enable it (Windows often turns it back on after a while or on reboot).".into()],
                                action: Action::DefenderRealtime(true),
                            });
                        }
                    }
                    if tp_on {
                        ui.label(RichText::new("blocked by Tamper Protection — turn it off in Windows Security › Virus & threat protection › Manage settings").color(muted).small());
                        if ui.small_button("Open Windows Security").clicked() {
                            crate::app::open_url("windowsdefender://threatsettings");
                        }
                    }
                });
            });

            // ---------- Serviços ----------
            section(ui, "Optional services", "stop now or disable permanently (they will not start again)", |ui| {
                egui::Grid::new("svc_grid").num_columns(5).spacing([14.0, 6.0]).striped(true).show(ui, |ui| {
                    ui.label(RichText::new("Service").strong());
                    ui.label(RichText::new("What it is").strong());
                    ui.label(RichText::new("Status").strong());
                    ui.label(RichText::new("RAM").strong());
                    ui.label(RichText::new("Actions").strong());
                    ui.end_row();
                    for (i, e) in sys::SERVICES.iter().enumerate() {
                        let st = self.svc.get(i).cloned().unwrap_or(SvcStatus { state: SvcState::Missing, start: SvcStart::Unknown });
                        if st.state == SvcState::Missing {
                            continue;
                        }
                        ui.vertical(|ui| {
                            ui.set_width(230.0);
                            ui.label(RichText::new(e.label).strong());
                            ui.label(RichText::new(e.name).monospace().small().color(muted));
                        });
                        ui.vertical(|ui| {
                            ui.set_max_width(440.0);
                            ui.add(egui::Label::new(RichText::new(e.why).color(muted).small()).wrap());
                        });
                        ui.vertical(|ui| {
                            let (s, c) = match st.state {
                                SvcState::Running => ("running", accent),
                                SvcState::Stopped => ("stopped", muted),
                                SvcState::Pending => ("changing…", muted),
                                SvcState::Missing => ("—", muted),
                            };
                            ui.label(RichText::new(s).color(c));
                            let start = match st.start {
                                SvcStart::Auto => "automatic start",
                                SvcStart::Manual => "manual start",
                                SvcStart::Disabled => "disabled",
                                SvcStart::Unknown => "",
                            };
                            ui.label(RichText::new(start).small().color(if st.start == SvcStart::Disabled { ok_c } else { muted }));
                        });
                        let ram = if st.state != SvcState::Running {
                            0
                        } else if let Some(x) = svchost_hint.get(e.name) {
                            x.0
                        } else if e.proc_hint != "svchost.exe" {
                            by_name.get(e.proc_hint).map(|x| x.0).unwrap_or(0)
                        } else {
                            0
                        };
                        ui.label(RichText::new(if ram > 0 { fmt_bytes_short(ram) } else { "–".into() }).monospace());
                        ui.horizontal(|ui| {
                            if st.state == SvcState::Running && ui.small_button("Stop").on_hover_text("Stops it now; it returns on the next boot (or when something requests it)").clicked() {
                                queued.push(Action::SvcStop(e.name));
                            }
                            if !e.stop_only {
                                if st.start != SvcStart::Disabled {
                                    if ui.add(egui::Button::new(RichText::new("Disable").color(warn_c)).small()).on_hover_text("Stops it and prevents it from starting again").clicked() {
                                        confirm = Some(Pending {
                                            title: format!("Disable {}", e.label),
                                            lines: vec![e.why.to_string(), format!("Service {} → StartupType Disabled. Reversible here (Re-enable).", e.name)],
                                            action: Action::SvcDisable(e.name),
                                        });
                                    }
                                } else if ui.small_button("Re-enable").clicked() {
                                    queued.push(Action::SvcEnable(e.name));
                                }
                            }
                        });
                        ui.end_row();
                    }
                    for (i, (name, label)) in sys::PROTECTED_SERVICES.iter().enumerate() {
                        let st = self.protected.get(i).cloned();
                        if st.as_ref().map(|s| s.state == SvcState::Missing).unwrap_or(true) {
                            continue;
                        }
                        ui.vertical(|ui| {
                            ui.set_width(230.0);
                            ui.label(RichText::new(*label).color(muted));
                            ui.label(RichText::new(*name).monospace().small().color(muted));
                        });
                        ui.vertical(|ui| {
                            ui.set_max_width(440.0);
                            ui.add(egui::Label::new(RichText::new("Protected by Windows — cannot be stopped or terminated. Use the Defender actions above.").color(muted).small()).wrap());
                        });
                        ui.label(RichText::new("running").color(muted));
                        let pn = match *name { "WinDefend" => "msmpeng.exe", "WdNisSvc" => "nissrv.exe", _ => "mpdefendercoreservice.exe" };
                        ui.label(RichText::new(by_name.get(pn).map(|x| fmt_bytes_short(x.0)).unwrap_or_else(|| "–".into())).monospace());
                        ui.label(RichText::new("🔒").color(muted));
                        ui.end_row();
                    }
                });
            });

            // ---------- Apps de sistema ----------
            section(ui, "Optional system apps", "installed for this user — terminate now or uninstall", |ui| {
                let mut any = false;
                egui::Grid::new("appx_grid").num_columns(4).spacing([14.0, 6.0]).striped(true).show(ui, |ui| {
                    for a in sys::APPX {
                        let installed = self.appx.iter().any(|f| f.starts_with(a.family_prefix));
                        if !installed {
                            continue;
                        }
                        any = true;
                        let mut ram = 0u64;
                        let mut pids = Vec::new();
                        for pn in a.procs {
                            if let Some((r, _, ps)) = by_name.get(*pn) {
                                ram += r;
                                pids.extend(ps.iter().copied());
                            }
                        }
                        ui.vertical(|ui| { ui.set_width(230.0); ui.label(RichText::new(a.label).strong()); });
                        ui.vertical(|ui| { ui.set_max_width(440.0); ui.add(egui::Label::new(RichText::new(a.why).color(muted).small()).wrap()); });
                        ui.label(RichText::new(if ram > 0 { format!("{} · {} proc.", fmt_bytes_short(ram), pids.len()) } else { "not running".into() }).color(if ram > 0 { accent } else { muted }).monospace());
                        ui.horizontal(|ui| {
                            if !pids.is_empty() && ui.small_button("Terminate").on_hover_text("Terminates the processes now (the app may return on its own)").clicked() {
                                out.push(DrainOut::Kill(pids.clone()));
                            }
                            if ui.add(egui::Button::new(RichText::new("Uninstall").color(warn_c)).small()).clicked() {
                                confirm = Some(Pending {
                                    title: format!("Uninstall {}", a.label),
                                    lines: vec![a.why.to_string(), format!("Get-AppxPackage {} | Remove-AppxPackage. It can be reinstalled from the Microsoft Store.", a.pkg_name)],
                                    action: Action::AppxRemove(a.pkg_name),
                                });
                            }
                        });
                        ui.end_row();
                    }
                });
                if !any {
                    ui.label(RichText::new("None of the cataloged apps are installed.").color(muted));
                }
            });

            ui.add_space(8.0);
            ui.label(RichText::new("Everything that starts with the PC (registry, startup folder, tasks, services) is in the Startup view — beyond Task Manager's limited list.").color(muted).small());
            ui.add_space(12.0);
        });

        if let Some(p) = confirm {
            self.pending = Some(p);
        }
        for a in queued {
            self.run(a, is_admin, &mut out);
        }
        // modal de confirmação
        if self.pending.is_some() {
            let mut go = false;
            let mut cancel = false;
            let title = self
                .pending
                .as_ref()
                .map(|p| p.title.clone())
                .unwrap_or_default();
            let lines = self
                .pending
                .as_ref()
                .map(|p| p.lines.clone())
                .unwrap_or_default();
            let modal = egui::Modal::new(egui::Id::new("drain_confirm")).show(ui.ctx(), |ui| {
                ui.set_width(520.0);
                ui.heading(&title);
                ui.add_space(6.0);
                for l in &lines {
                    ui.add(egui::Label::new(RichText::new(l).color(muted)).wrap());
                }
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(RichText::new("Confirm").strong())
                                .fill(Color32::from_rgb(160, 60, 55)),
                        )
                        .clicked()
                    {
                        go = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                    ui.label(RichText::new("Enter confirms · Esc cancels").weak().small());
                });
            });
            let (enter, esc) = ui.ctx().input(|i| {
                (
                    i.key_pressed(egui::Key::Enter),
                    i.key_pressed(egui::Key::Escape),
                )
            });
            if enter {
                go = true;
            }
            if esc || modal.should_close() {
                cancel = true;
            }
            if go {
                if let Some(p) = self.pending.take() {
                    self.run(p.action, is_admin, &mut out);
                }
            } else if cancel {
                self.pending = None;
            }
        }
        out
    }
}

fn section(ui: &mut egui::Ui, title: &str, sub: &str, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(SURFACE)
        .stroke(egui::Stroke::new(1.0_f32, LINE))
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(title).strong().size(14.5));
                ui.label(RichText::new(sub).color(MUTED).small());
            });
            ui.add_space(6.0);
            add(ui);
        });
    ui.add_space(10.0);
}

fn pill(ui: &mut egui::Ui, label: &str, (value, color): (&str, Color32)) {
    egui::Frame::new()
        .fill(SURFACE_HI)
        .corner_radius(4.0)
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).small().color(MUTED));
                ui.label(RichText::new(value).small().strong().color(color));
            });
        });
}
