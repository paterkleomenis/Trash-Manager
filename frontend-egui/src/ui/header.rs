//! Header component with title, search, and hamburger menu.

use eframe::egui;

pub struct Header {
    pub search_text: String,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            search_text: String::new(),
        }
    }
}

impl Header {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        process_table: &mut crate::ui::process_table::ProcessTable,
    ) -> (bool, bool) {
        // Returns (search_changed, refresh_requested)
        let mut search_changed = false;
        let mut refresh_requested = false;

        ui.horizontal(|ui| {
            // Title on the left
            ui.heading("Trash Manager");

            // Search in the middle
            ui.add_space(20.0);
            ui.label("Search:");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.search_text)
                    .hint_text("Filter processes...")
                    .desired_width(200.0),
            );

            if response.changed() {
                search_changed = true;
            }

            // Clear button
            if !self.search_text.is_empty() && ui.button("Clear").clicked() {
                self.search_text.clear();
                search_changed = true;
            }

            // Refresh button
            ui.add_space(10.0);
            if ui.button("Refresh")
                .on_hover_text("Click to update process list and CPU usage.\nNote: CPU % shows 0% on first load - wait 1+ seconds between refreshes for accurate values.")
                .clicked() {
                refresh_requested = true;
            }

            // Hamburger menu on the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.menu_button("Menu", |ui| {
                    ui.set_min_width(120.0);

                    ui.label("Show columns:");
                    ui.checkbox(&mut process_table.show_pid, "PID");
                    ui.checkbox(&mut process_table.show_ppid, "PPID");
                });
            });
        });

        (search_changed, refresh_requested)
    }
}
