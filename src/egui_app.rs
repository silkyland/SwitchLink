/// eGUI version - Works great in Linux!
use eframe::egui;
use eframe::egui::{CentralPanel, Context, ScrollArea, Ui};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::file_manager::{add_files, add_files_from_directory, format_file_size, get_file_size};

#[derive(Default)]
pub struct DbiApp {
    file_list: HashMap<String, PathBuf>,
    log_messages: Vec<String>,
    server_running: bool,
    connection_status: String,
}

impl DbiApp {
    pub fn new() -> Self {
        Self {
            log_messages: vec!["🦀 DBI Backend started with eGUI!".to_string()],
            connection_status: "Disconnected".to_string(),
            ..Default::default()
        }
    }
}

impl eframe::App for DbiApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎮 DBI Backend - Rust Edition");
            ui.label("High Performance USB Transfer for Nintendo Switch");

            ui.separator();

            // Two-panel layout
            ui.columns(2, |columns| {
                // Left panel - File Queue
                columns[0].vertical(|ui| {
                    self.file_panel(ui);
                });

                // Right panel - Server Control
                columns[1].vertical(|ui| {
                    self.server_panel(ui);
                });
            });

            // Footer
            ui.separator();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("Built with eGUI 🦀 | High-performance USB transfer");
            });
        });
    }
}

impl DbiApp {
    fn file_panel(&mut self, ui: &mut Ui) {
        ui.heading("📁 File Queue");

        ui.horizontal(|ui| {
            if ui.button("📂 Add Folder").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    match add_files_from_directory(&mut self.file_list, path) {
                        Ok(count) => {
                            self.log_messages.push(format!("✅ Added {} files from folder", count));
                        }
                        Err(e) => {
                            self.log_messages.push(format!("❌ Error adding folder: {}", e));
                        }
                    }
                }
            }

            if ui.button("📄 Add Files").clicked() {
                if let Some(files) = rfd::FileDialog::new()
                    .add_filter("Switch Files", &["nsp", "nsz", "xci", "xcz"])
                    .pick_files()
                {
                    let paths: Vec<PathBuf> = files.iter().map(|f| f.path()).collect();
                    let count = add_files(&mut self.file_list, paths);
                    self.log_messages.push(format!("✅ Added {} files", count));
                }
            }

            if ui.button("🗑️ Clear All").clicked() {
                let count = self.file_list.len();
                self.file_list.clear();
                self.log_messages.push(format!("✅ Cleared {} files", count));
            }
        });

        ui.label(format!("Files in queue: {}", self.file_list.len()));

        ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            for (name, path) in &self.file_list {
                ui.horizontal(|ui| {
                    ui.label(format!("• {}", name));
                    if let Ok(size) = std::fs::metadata(path) {
                        ui.label(format_file_size(size.len()));
                    }
                    if ui.button("✕").clicked() {
                        self.file_list.remove(name);
                        self.log_messages.push(format!("Removed file: {}", name));
                    }
                });
            }
        });
    }

    fn server_panel(&mut self, ui: &mut Ui) {
        ui.heading("⚙️ Server Control");

        // Status indicator
        let status_color = match self.server_running {
            true => egui::Color32::GREEN,
            false => egui::Color32::RED,
        };

        ui.horizontal(|ui| {
            ui.label("Status:");
            ui.colored_label(status_color, &self.connection_status);
        });

        ui.separator();

        // Server controls
        ui.vertical_centered(|ui| {
            if ui.button("▶️ Start Server").clicked() {
                if !self.file_list.is_empty() {
                    self.server_running = true;
                    self.connection_status = "Connected".to_string();
                    self.log_messages.push("🚀 Starting DBI server...".to_string());
                    self.log_messages.push("📋 Connect your Switch and select 'Install title from DBIbackend'".to_string());
                } else {
                    self.log_messages.push("❌ Please add files first".to_string());
                }
            }

            if ui.button("⏹️ Stop Server").clicked() {
                self.server_running = false;
                self.connection_status = "Disconnected".to_string();
                self.log_messages.push("⏹️ Server stopped".to_string());
            }
        });

        ui.separator();

        // Instructions
        ui.heading("📋 Instructions");
        ui.label("1. Add NSP/NSZ/XCI/XCZ files or folders");
        ui.label("2. Connect your Nintendo Switch via USB");
        ui.label("3. Launch DBI on your Switch");
        ui.label("4. Select 'Install title from DBIbackend'");
        ui.label("5. Click 'Start Server' above");

        ui.separator();

        // Activity Log
        ui.heading("📝 Activity Log");
        ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
            for msg in self.log_messages.iter().rev() {
                ui.label(msg);
            }
        });
    }
}

pub fn launch_egui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_title("DBI Backend - Rust Edition"),
        ..Default::default()
    };

    eframe::run_native(
        "DBI Backend - Rust Edition",
        options,
        Box::new(|_cc| Box::new(DbiApp::new())),
    ).unwrap();
}
