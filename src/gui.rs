/// eGUI version - Works perfectly in Linux!
use eframe::egui;
use eframe::egui::{CentralPanel, Context, ProgressBar, ScrollArea, Ui};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use crate::file_manager::{add_files, add_files_from_directory, format_file_size};
use crate::usb::{DbiServer, TransferProgress};

#[derive(Default)]
pub struct DbiApp {
    file_list: HashMap<String, PathBuf>,
    log_messages: Vec<String>,
    server_running: bool,
    connection_status: String,
    server_thread: Option<thread::JoinHandle<()>>,
    server_instance: Option<Arc<Mutex<DbiServer>>>,
    progress: Arc<Mutex<TransferProgress>>,
}

impl DbiApp {
    pub fn new() -> Self {
        Self {
            log_messages: vec!["🦀 DBI Backend started with eGUI!".to_string()],
            connection_status: "Disconnected".to_string(),
            progress: Arc::new(Mutex::new(TransferProgress::default())),
            ..Default::default()
        }
    }

    fn start_server(&mut self) {
        if self.file_list.is_empty() {
            self.log_messages.push("❌ Please add files first".to_string());
            return;
        }

        // Reset progress
        if let Ok(mut progress) = self.progress.lock() {
            *progress = TransferProgress::default();
        }

        // Create shared file list
        let file_list = Arc::new(Mutex::new(self.file_list.clone()));
        
        // Create server instance with progress tracking
        let server = DbiServer::new_with_progress(file_list, self.progress.clone());
        let server_arc = Arc::new(Mutex::new(server));
        self.server_instance = Some(server_arc.clone());

        // Start server in background thread
        let server_clone = server_arc.clone();
        let handle = thread::spawn(move || {
            if let Ok(mut server) = server_clone.lock() {
                if let Err(e) = server.start() {
                    eprintln!("Server error: {}", e);
                }
            }
        });

        self.server_thread = Some(handle);
        self.server_running = true;
        self.connection_status = "Connected".to_string();
        self.log_messages.push("🚀 Starting DBI server...".to_string());
        self.log_messages.push("📋 Connect your Switch and select 'Install title from DBIbackend'".to_string());
    }

    fn stop_server(&mut self) {
        if let Some(server) = &self.server_instance {
            if let Ok(mut server) = server.lock() {
                server.stop();
            }
        }
        
        self.server_running = false;
        self.connection_status = "Disconnected".to_string();
        self.log_messages.push("⏹️ Server stopped".to_string());
        self.server_thread = None;
        self.server_instance = None;
    }
}

impl eframe::App for DbiApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Request repaint continuously when server is running for smooth progress updates
        if self.server_running {
            ctx.request_repaint();
        }
        
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎮 DBI Backend - Rust Edition");
            ui.label("🚀 eGUI Version - Works perfectly in Linux!");

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
                ui.label("Built with eGUI 🦀 | Native Linux performance");
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
                    let paths: Vec<PathBuf> = files.iter().map(|f| f.to_path_buf()).collect();
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
            let mut to_remove = Vec::new();
            
            for (name, path) in &self.file_list {
                ui.horizontal(|ui| {
                    ui.label(format!("• {}", name));
                    if let Ok(size) = std::fs::metadata(path) {
                        ui.label(format_file_size(size.len()));
                    }
                    if ui.button("✕").clicked() {
                        to_remove.push(name.clone());
                    }
                });
            }
            
            // Remove files after iteration
            for name in to_remove {
                self.file_list.remove(&name);
                self.log_messages.push(format!("Removed file: {}", name));
            }
        });
    }

    fn server_panel(&mut self, ui: &mut Ui) {
        ui.heading("⚙ Server Control");

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

        // Progress bar and transfer stats
        if self.server_running {
            if let Ok(progress) = self.progress.lock() {
                ui.heading("▶ Transfer Progress");
                
                // Current file being transferred
                if !progress.current_file.is_empty() {
                    ui.label(format!("• File: {}", progress.current_file));
                }
                
                // Progress bar
                let progress_ratio = if progress.total_size > 0 {
                    progress.bytes_sent as f32 / progress.total_size as f32
                } else {
                    0.0
                };
                
                ui.add(ProgressBar::new(progress_ratio)
                    .text(format!("{:.1}%", progress_ratio * 100.0)));
                
                // Transfer stats
                ui.horizontal(|ui| {
                    ui.label(format!("↑ Sent: {}", format_file_size(progress.bytes_sent)));
                    ui.label(format!("/ {}", format_file_size(progress.total_size)));
                });
                
                // Speed
                if progress.speed_mbps > 0.0 {
                    ui.label(format!("⚡ Speed: {:.2} MB/s", progress.speed_mbps));
                    
                    // ETA
                    if progress.total_size > progress.bytes_sent && progress.speed_mbps > 0.0 {
                        let remaining_bytes = progress.total_size - progress.bytes_sent;
                        let remaining_seconds = (remaining_bytes as f64 / (progress.speed_mbps * 1_000_000.0)) as u64;
                        let minutes = remaining_seconds / 60;
                        let seconds = remaining_seconds % 60;
                        ui.label(format!("⏱ ETA: {}m {}s", minutes, seconds));
                    }
                }
                
                ui.separator();
            }
        }

        // Server controls
        ui.vertical_centered(|ui| {
            if ui.button("▶️ Start Server").clicked() {
                self.start_server();
            }

            if ui.button("⏹️ Stop Server").clicked() {
                self.stop_server();
            }
        });

        ui.separator();

        // Instructions
        ui.heading("ℹ Instructions");
        ui.label("1. Add NSP/NSZ/XCI/XCZ files or folders");
        ui.label("2. Connect your Nintendo Switch via USB");
        ui.label("3. Launch DBI on your Switch");
        ui.label("4. Select 'Install title from DBIbackend'");
        ui.label("5. Click 'Start Server' above");

        ui.separator();

        // Activity Log
        ui.heading("≡ Activity Log");
        
        // Get logs from progress
        if let Ok(progress) = self.progress.lock() {
            if !progress.logs.is_empty() {
                // Merge server logs with app logs
                for log in progress.logs.iter().rev().take(10) {
                    if !self.log_messages.contains(log) {
                        self.log_messages.push(log.clone());
                    }
                }
            }
        }
        
        ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
            for msg in self.log_messages.iter().rev().take(20) {
                ui.label(msg);
            }
        });
    }
}

pub fn launch_gui() {
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
