use backend::{list_processes, ProcessInfo};
use eframe::{egui, App};
use std::sync::{Arc, Mutex};
use ui::header::Header;
use ui::status_bar::StatusBar;

mod ui;
use ui::process_table::ProcessTable;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Trash Manager",
        native_options,
        Box::new(|cc| {
            // Global black theme
            let mut visuals = egui::Visuals::dark();
            visuals.override_text_color = Some(egui::Color32::WHITE);
            visuals.panel_fill = egui::Color32::BLACK;
            visuals.window_fill = egui::Color32::BLACK;
            cc.egui_ctx.set_visuals(visuals);

            // Slightly larger default spacing
            let mut style = (*cc.egui_ctx.style()).clone();
            style.spacing.item_spacing = egui::vec2(10.0, 8.0);
            style.spacing.button_padding = egui::vec2(12.0, 8.0);
            cc.egui_ctx.set_style(style);

            Box::new(ProcessManagerApp::default())
        }),
    )
}

struct ProcessManagerApp {
    processes: Arc<Mutex<Vec<ProcessInfo>>>,
    process_table: ProcessTable,
    header: Header,
}

impl Default for ProcessManagerApp {
    fn default() -> Self {
        // Load processes once at startup
        let processes = Arc::new(Mutex::new(Vec::new()));
        if let Ok(list) = list_processes() {
            if let Ok(mut proc_lock) = processes.lock() {
                *proc_lock = list;
            }
        }

        Self {
            processes,
            process_table: ProcessTable::default(),
            header: Header::default(),
        }
    }
}

impl App for ProcessManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show header with search and get refresh request
            let (search_changed, refresh_requested) = self.header.show(ui, &mut self.process_table);
            ui.add_space(6.0);

            // Handle refresh request
            if refresh_requested {
                if let Ok(list) = list_processes() {
                    if let Ok(mut proc_lock) = self.processes.lock() {
                        *proc_lock = list;
                    }
                }
            }

            // Take a snapshot for rendering
            let processes = {
                let lock = self.processes.lock().unwrap();
                lock.clone()
            };

            // Show process table with search filter
            let filtered_count = self
                .process_table
                .show(ui, &processes, &self.header.search_text);

            ui.add_space(6.0);

            // Show status bar
            StatusBar::show(ui, &processes, filtered_count);

            // Request repaint if search changed for immediate filtering
            if search_changed {
                ctx.request_repaint();
            }
        });
    }
}
