use backend::{kill_pid, list_processes, ProcessInfo};
use eframe::{egui, App};
use egui_extras::{Column, TableBuilder};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
    last_refresh: Instant,
    sort_column: SortColumn,
    sort_descending: bool,
    killing: Arc<Mutex<HashSet<i32>>>,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum SortColumn {
    #[default]
    PID,
    Name,
    CPU,
    Memory,
    State,
    PPID,
}

impl Default for ProcessManagerApp {
    fn default() -> Self {
        Self {
            processes: Arc::new(Mutex::new(Vec::new())),
            last_refresh: Instant::now(),
            sort_column: SortColumn::PID,
            sort_descending: false,
            killing: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

impl App for ProcessManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Refresh ~5 times/second (smooth but not wasteful)
        if self.last_refresh.elapsed() > Duration::from_millis(200) {
            if let Ok(list) = list_processes() {
                if let Ok(mut proc_lock) = self.processes.lock() {
                    *proc_lock = list;
                }
            }
            self.last_refresh = Instant::now();
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Trash Manager");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.small(format!(
                        "refreshed: {} ms ago",
                        self.last_refresh.elapsed().as_millis()
                    ));
                });
            });
            ui.add_space(6.0);

            // Take a snapshot for sorting/rendering
            let mut processes = {
                let lock = self.processes.lock().unwrap();
                lock.clone()
            };

            // Sort once
            processes.sort_by(|a, b| {
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

            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(80.0)) // PID
                .column(Column::remainder()) // Name (takes remaining space)
                .column(Column::exact(80.0)) // CPU
                .column(Column::exact(110.0)) // Memory
                .column(Column::exact(90.0)) // State
                .column(Column::exact(80.0)) // PPID
                .column(Column::exact(90.0)) // Action
                .header(row_height, |mut header| {
                    header.col(|ui| {
                        sort_header(
                            ui,
                            "PID",
                            SortColumn::PID,
                            &mut self.sort_column,
                            &mut self.sort_descending,
                        )
                    });
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
                    header.col(|ui| {
                        sort_header(
                            ui,
                            "PPID",
                            SortColumn::PPID,
                            &mut self.sort_column,
                            &mut self.sort_descending,
                        )
                    });
                    header.col(|ui| {
                        ui.label("");
                    });
                })
                .body(|mut body| {
                    body.rows(row_height, processes.len(), |mut row| {
                        let idx = row.index();
                        let p = &processes[idx];

                        row.col(|ui| {
                            ui.label(
                                egui::RichText::new(p.pid.to_string())
                                    .size(text_sz)
                                    .monospace(),
                            );
                        });

                        row.col(|ui| {
                            let mut lbl =
                                egui::Label::new(egui::RichText::new(&p.name).size(text_sz));
                            lbl = lbl.sense(egui::Sense::hover());
                            let resp = ui.add(lbl);
                            resp.on_hover_text(format!("{}\nPID: {}", p.name, p.pid));
                        });

                        row.col(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{:.1}", p.cpu_percent)).size(text_sz),
                            );
                        });

                        row.col(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{:.1} MB",
                                    p.memory_bytes as f32 / (1024.0 * 1024.0)
                                ))
                                .size(text_sz),
                            );
                        });

                        row.col(|ui| {
                            ui.label(egui::RichText::new(&p.state).size(text_sz));
                        });

                        row.col(|ui| {
                            ui.label(
                                egui::RichText::new(p.ppid.to_string())
                                    .size(text_sz)
                                    .monospace(),
                            );
                        });

                        row.col(|ui| {
                            let is_killing = self.killing.lock().unwrap().contains(&p.pid);
                            let btn = egui::Button::new(
                                egui::RichText::new(if is_killing { "..." } else { "KILL" })
                                    .size(14.0)
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(200, 40, 40))
                            .min_size(egui::vec2(70.0, 26.0));

                            if ui.add_enabled(!is_killing, btn).clicked() {
                                self.killing.lock().unwrap().insert(p.pid);
                                let pid = p.pid;
                                let killing = self.killing.clone();

                                // If kill_pid blocks, run it off the UI thread:
                                tokio::task::spawn_blocking(move || {
                                    let _ = kill_pid(pid);
                                    killing.lock().unwrap().remove(&pid);
                                });
                            }
                        });
                    });
                });
        });
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
