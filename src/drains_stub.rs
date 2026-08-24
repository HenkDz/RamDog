//! Ralos são Windows (Defender, SCM, Appx). No macOS a aba existe e explica.

use crate::procs::ProcInfo;

pub enum DrainOut {
    Toast(String, bool),
    Kill(Vec<u32>),
}

pub struct Drains;

impl Drains {
    pub fn new() -> Self {
        Self
    }

    pub fn snapshot_json(&mut self) -> serde_json::Value {
        serde_json::json!({ "supported": false, "reason": "Defender, services, and Appx are Windows-only" })
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, _procs: &[ProcInfo], _is_admin: bool) -> Vec<DrainOut> {
        ui.add_space(16.0);
        ui.label("Drains (Defender, services, Appx) are Windows-specific.");
        ui.add_space(8.0);
        ui.label("On macOS RamDog lists, categorizes, and terminates processes — this view has no equivalent.");
        Vec::new()
    }
}
