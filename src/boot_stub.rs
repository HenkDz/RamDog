//! Partida (tudo que sobe com o PC) é Windows. No macOS a aba existe e explica.

use crate::procs::ProcInfo;

pub enum BootOut {
    Toast(String, bool),
    Kill(Vec<u32>),
}

pub struct Boot;

impl Boot {
    pub fn new() -> Self {
        Self
    }

    pub fn snapshot_json(&mut self) -> serde_json::Value {
        serde_json::json!({ "supported": false, "reason": "Startup inventory is Windows-only" })
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _procs: &[ProcInfo],
        _search: &str,
        _is_admin: bool,
    ) -> Vec<BootOut> {
        ui.add_space(16.0);
        ui.label("The Startup view lists what Windows launches at boot and logon.");
        ui.add_space(8.0);
        ui.label("On macOS the equivalent is LaunchAgents/LaunchDaemons — this view does not support them yet.");
        Vec::new()
    }
}
