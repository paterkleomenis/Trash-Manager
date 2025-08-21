//! Process table component with sorting and kill functionality.

use backend::{kill_pid, ProcessInfo};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

// Filter processes based on search text
fn filter_processes(processes: &[ProcessInfo], search_text: &str) -> Vec<ProcessInfo> {
    if search_text.is_empty() {
        return processes.to_vec();
    }

    let search_lower = search_text.to_lowercase();
    processes
        .iter()
        .filter(|p| {
            p.name.to_lowercase().contains(&search_lower)
                || p.pid.to_string().contains(&search_lower)
        })
        .cloned()
        .collect()
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum SortColumn {
    #[default]
    PID,
    Name,
    CPU,
    Memory,
    State,
    PPID,
}

pub struct ProcessTable {
    pub sort_column: SortColumn,
    pub sort_descending: bool,
    pub killing: Arc<Mutex<HashSet<i32>>>,
    pub show_pid: bool,
    pub show_ppid: bool,
}

impl Default for ProcessTable {
    fn default() -> Self {
        Self {
            sort_column: SortColumn::PID,
            sort_descending: false,
            killing: Arc::new(Mutex::new(HashSet::new())),
            show_pid: false,
            show_ppid: false,
        }
    }
}

impl ProcessTable {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        processes: &[ProcessInfo],
        search_text: &str,
    ) -> usize {
        // Filter processes first
        let filtered_processes = filter_processes(processes, search_text);

        // Sort filtered processes
        let mut sorted_processes = filtered_processes;
        sorted_processes.sort_by(|a, b| {
            let ord = match self.sort_column {
                SortColumn::PID => a.pid.cmp(&b.pid),
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::CPU => ord_f32(a.cpu_percent, b.cpu_percent),
                SortColumn::Memory => a.memory_bytes.cmp(&b.memory_bytes),
                SortColumn::State => a.state.cmp(&b.state),
                SortColumn::PPID => a.ppid.cmp(&b.ppid),
            };
            if self.sort_descending {
                ord.reverse()
            } else {
                ord
            }
        });

        // Build a real table: fixed columns, striped rows, consistent layout
        let text_sz = 16.0;
        let row_height = 30.0;

        let mut table_builder = TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center));

        // Add columns conditionally
        if self.show_pid {
            table_builder = table_builder.column(Column::exact(80.0)); // PID
        }
        table_builder = table_builder
            .column(Column::remainder()) // Name (takes remaining space)
            .column(Column::exact(80.0)) // CPU
            .column(Column::exact(110.0)) // Memory
            .column(Column::exact(90.0)); // State
        if self.show_ppid {
            table_builder = table_builder.column(Column::exact(80.0)); // PPID
        }

        table_builder
            .header(row_height, |mut header| {
                if self.show_pid {
                    header.col(|ui| {
                        sort_header(
                            ui,
                            "PID",
                            SortColumn::PID,
                            &mut self.sort_column,
                            &mut self.sort_descending,
                        )
                    });
                }
                header.col(|ui| {
                    sort_header(
                        ui,
                        "Name",
                        SortColumn::Name,
                        &mut self.sort_column,
                        &mut self.sort_descending,
                    )
                });
                header.col(|ui| {
                    sort_header(
                        ui,
                        "CPU %",
                        SortColumn::CPU,
                        &mut self.sort_column,
                        &mut self.sort_descending,
                    )
                });
                header.col(|ui| {
                    sort_header(
                        ui,
                        "Memory",
                        SortColumn::Memory,
                        &mut self.sort_column,
                        &mut self.sort_descending,
                    )
                });
                header.col(|ui| {
                    sort_header(
                        ui,
                        "State",
                        SortColumn::State,
                        &mut self.sort_column,
                        &mut self.sort_descending,
                    )
                });
                if self.show_ppid {
                    header.col(|ui| {
                        sort_header(
                            ui,
                            "PPID",
                            SortColumn::PPID,
                            &mut self.sort_column,
                            &mut self.sort_descending,
                        )
                    });
                }
            })
            .body(|body| {
                body.rows(row_height, sorted_processes.len(), |mut row| {
                    let idx = row.index();
                    let p = &sorted_processes[idx];

                    // PID column - conditionally shown, NO right-click menu
                    if self.show_pid {
                        row.col(|ui| {
                            ui.label(
                                egui::RichText::new(p.pid.to_string())
                                    .size(text_sz)
                                    .monospace(),
                            );
                        });
                    }

                    // Name column - WITH right-click menu
                    row.col(|ui| {
                        let response = ui.add(
                            egui::Label::new(egui::RichText::new(&p.name).size(text_sz))
                                .sense(egui::Sense::click()),
                        );
                        response.clone().on_hover_text(format!(
                            "{}\nPID: {}\nRight-click for options",
                            p.name, p.pid
                        ));

                        response.context_menu(|ui| {
                            self.show_context_menu(ui, p);
                        });
                    });

                    // CPU column - WITH right-click menu
                    row.col(|ui| {
                        let response = ui.add(
                            egui::Label::new(
                                egui::RichText::new(format!("{:.1}", p.cpu_percent)).size(text_sz),
                            )
                            .sense(egui::Sense::click()),
                        );

                        response.context_menu(|ui| {
                            self.show_context_menu(ui, p);
                        });
                    });

                    // Memory column - WITH right-click menu
                    row.col(|ui| {
                        let response = ui.add(
                            egui::Label::new(
                                egui::RichText::new(format!(
                                    "{:.1} MB",
                                    p.memory_bytes as f32 / (1024.0 * 1024.0)
                                ))
                                .size(text_sz),
                            )
                            .sense(egui::Sense::click()),
                        );

                        response.context_menu(|ui| {
                            self.show_context_menu(ui, p);
                        });
                    });

                    // State column - WITH right-click menu
                    row.col(|ui| {
                        let response = ui.add(
                            egui::Label::new(egui::RichText::new(&p.state).size(text_sz))
                                .sense(egui::Sense::click()),
                        );

                        response.context_menu(|ui| {
                            self.show_context_menu(ui, p);
                        });
                    });

                    // PPID column - conditionally shown, WITH right-click menu
                    if self.show_ppid {
                        row.col(|ui| {
                            let response = ui.add(
                                egui::Label::new(
                                    egui::RichText::new(p.ppid.to_string())
                                        .size(text_sz)
                                        .monospace(),
                                )
                                .sense(egui::Sense::click()),
                            );

                            response.context_menu(|ui| {
                                self.show_context_menu(ui, p);
                            });
                        });
                    }
                });
            });

        // Return the count of filtered processes
        sorted_processes.len()
    }

    fn show_context_menu(&mut self, ui: &mut egui::Ui, p: &ProcessInfo) {
        ui.set_min_width(200.0);

        let is_killing = self.killing.lock().unwrap().contains(&p.pid);

        let kill_text = if is_killing {
            "Killing..."
        } else {
            "Kill Process"
        };
        let kill_button = ui.add_enabled(
            !is_killing,
            egui::Button::new(kill_text)
                .fill(egui::Color32::from_rgb(200, 40, 40))
                .min_size(egui::vec2(180.0, 25.0)),
        );

        if kill_button.clicked() {
            self.killing.lock().unwrap().insert(p.pid);
            let pid = p.pid;
            let killing = self.killing.clone();

            tokio::task::spawn_blocking(move || {
                let _ = kill_pid(pid);
                killing.lock().unwrap().remove(&pid);
            });

            ui.close_menu();
        }

        ui.separator();
        ui.label(format!("PID: {}", p.pid));
        ui.label(format!("Name: {}", p.name));
        ui.label(format!("State: {}", p.state));
        ui.label(format!("Parent PID: {}", p.ppid));
        ui.label(format!(
            "Memory: {:.1} MB",
            p.memory_bytes as f32 / (1024.0 * 1024.0)
        ));
    }
}

fn sort_header(
    ui: &mut egui::Ui,
    title: &str,
    col: SortColumn,
    sort_col: &mut SortColumn,
    descending: &mut bool,
) {
    let active = *sort_col == col;
    let arrow = if !active {
        ""
    } else if *descending {
        " ↓"
    } else {
        " ↑"
    };
    let btn = egui::Button::new(
        egui::RichText::new(format!("{title}{arrow}"))
            .strong()
            .size(15.0),
    )
    .frame(false);

    if ui.add(btn).clicked() {
        if active {
            *descending = !*descending;
        } else {
            *sort_col = col;
            *descending = false;
        }
    }
}

// Safe f32 ordering (handles NaN)
fn ord_f32(a: f32, b: f32) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}
