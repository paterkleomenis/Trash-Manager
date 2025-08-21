//! Status bar component showing process counts and statistics.

use backend::ProcessInfo;
use eframe::egui;

pub struct StatusBar;

impl StatusBar {
    pub fn show(ui: &mut egui::Ui, processes: &[ProcessInfo], filtered_count: usize) {
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!("Total processes: {}", processes.len()));

            if filtered_count != processes.len() {
                ui.separator();
                ui.label(format!("Filtered: {}", filtered_count));
            }

            ui.separator();
            let total_memory: u64 = processes.iter().map(|p| p.memory_bytes).sum();
            ui.label(format!(
                "Total memory: {:.1} GB",
                total_memory as f64 / (1024.0 * 1024.0 * 1024.0)
            ));

            ui.separator();
            let running_count = processes.iter().filter(|p| p.state == "R").count();
            ui.label(format!("Running: {}", running_count));
        });
    }
}
