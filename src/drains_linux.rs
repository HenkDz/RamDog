use egui::{Align, Color32, Layout, RichText};

use crate::app::MUTED;
use crate::{
    linux::Job,
    procs::ProcInfo,
    startup_linux::{self, Inventory, Source},
};
pub enum DrainOut {
    Toast(String, bool),
    Kill(Vec<u32>),
}
#[derive(Default)]
pub struct Drains {
    scan: Job<Inventory>,
    action: Job<()>,
    only_active: bool,
}
impl Drains {
    pub fn new() -> Self {
        Self {
            only_active: true,
            ..Default::default()
        }
    }
    pub fn ui(&mut self, ui: &mut egui::Ui, _procs: &[ProcInfo], _admin: bool) -> Vec<DrainOut> {
        use crate::kit;
        self.scan.poll();
        if self.action.poll() {
            self.scan.start(startup_linux::scan);
        }
        if self.scan.due(10) {
            self.scan.start(startup_linux::scan);
        }
        kit::intro(ui, "Serviços em segundo plano, do maior pro menor em RAM. Revise o consumo e a finalidade antes de parar um; os essenciais estão protegidos.");
        ui.add_space(6.0);
        kit::toolbar(ui, |ui| {
            ui.checkbox(&mut self.only_active, RichText::new("Somente em execução").size(12.5));
            if ui.add(kit::button("Atualizar")).clicked() {
                self.scan.start(startup_linux::scan);
            }
            self.scan.status(ui);
            self.action.status(ui);
        });
        for warning in &self.scan.value.warnings {
            ui.colored_label(egui::Color32::YELLOW, warning);
        }
        let mut entries: Vec<_> = self
            .scan
            .value
            .entries
            .iter()
            .filter(|e| matches!(e.source, Source::Unit { .. }) && (!self.only_active || e.active))
            .collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.memory.unwrap_or(0)));
        ui.add_space(6.0);
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            for e in entries {
                kit::row(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut sub = e.kind.clone();
                        if e.pid > 0 {
                            sub.push_str(&format!(" · PID {}", e.pid));
                        }
                        if !e.description.is_empty() {
                            sub.push_str(" · ");
                            sub.push_str(&e.description);
                        }
                        ui.scope(|ui| {
                            ui.set_max_width((ui.available_width() - 610.0).max(160.0));
                            kit::two_lines(ui, RichText::new(&e.name), &sub);
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if let Source::Unit { user, unit } = &e.source {
                                if ui
                                    .add_enabled(
                                        e.can_toggle && !self.action.busy(),
                                        kit::button(if e.enabled { "Não iniciar no boot" } else { "Iniciar no boot" }),
                                    )
                                    .clicked()
                                {
                                    let e = e.clone();
                                    self.action.start(move || startup_linux::toggle(&e, !e.enabled));
                                }
                                for (label, action, enabled) in [("Iniciar", "start", !e.active), ("Reiniciar", "restart", e.active), ("Parar", "stop", e.active)] {
                                    if ui.add_enabled(enabled && !e.protected && !self.action.busy(), kit::button(label)).clicked() {
                                        let (user, unit) = (*user, unit.clone());
                                        self.action.start(move || startup_linux::unit_action(user, action, &unit));
                                    }
                                }
                            }
                            if e.protected {
                                kit::badge(ui, "protegido", MUTED);
                            }
                            if e.active {
                                kit::badge(ui, "em execução", Color32::from_rgb(120, 200, 140));
                            }
                            ui.label(
                                RichText::new(e.memory.map(|m| format!("{:.1} MiB", m as f64 / 1048576.0)).unwrap_or_else(|| "RAM indisponível".into()))
                                    .monospace()
                                    .size(12.5),
                            );
                        });
                    });
                });
            }
        });
        ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));
        Vec::new()
    }
}
