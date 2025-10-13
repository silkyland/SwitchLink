/// eGUI version - Works perfectly in Linux!
use eframe::egui;
use eframe::egui::{CentralPanel, Context, ProgressBar, ScrollArea, Ui};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use crate::database::Database;
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
    database: Option<Database>,
    search_query: String,
}

impl DbiApp {
    pub fn new() -> Self {
        // Initialize database
        let db_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("dbi-backend")
            .join("games.db");
        
        // Create directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        
        let database = Database::new(&db_path).ok();
        
        Self {
            log_messages: vec!["[*] DBI Backend started with eGUI!".to_string()],
            connection_status: "Disconnected".to_string(),
            progress: Arc::new(Mutex::new(TransferProgress::default())),
            database,
            search_query: String::new(),
            ..Default::default()
        }
    }

    fn start_server(&mut self) {
        if self.file_list.is_empty() {
            self.log_messages.push("[!] Please add files first".to_string());
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
        self.log_messages.push("[>] Starting DBI server...".to_string());
        self.log_messages.push("[i] Connect your Switch and select 'Install title from DBIbackend'".to_string());
    }

    fn stop_server(&mut self) {
        if let Some(server) = &self.server_instance {
            if let Ok(mut server) = server.lock() {
                server.stop();
            }
        }
        
        self.server_running = false;
        self.connection_status = "Disconnected".to_string();
        self.log_messages.push("[x] Server stopped".to_string());
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
        
        // Top panel - Controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎮 DBI Backend - Rust Edition");
                ui.separator();
                
                // Server controls
                if ui.button("▶ Start Server").clicked() {
                    self.start_server();
                }
                if ui.button("■ Stop Server").clicked() {
                    self.stop_server();
                }
            });
        });
        
        // Bottom panel - Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("v0.1.0");
                ui.separator();
                
                // Connection status
                let status_color = match self.server_running {
                    true => egui::Color32::GREEN,
                    false => egui::Color32::RED,
                };
                ui.colored_label(status_color, &self.connection_status);
                
                ui.separator();
                ui.label("Built with Rust");
            });
        });
        
        // Bottom panel for Activity Log and Instructions
        egui::TopBottomPanel::bottom("activity_panel").min_height(200.0).show(ctx, |ui| {
            ui.columns(2, |columns| {
                // Left - Activity Log
                columns[0].vertical(|ui| {
                    self.activity_log_panel(ui);
                });
                
                // Right - Instructions
                columns[1].vertical(|ui| {
                    ui.heading("ℹ Instructions");
                    ui.label("1. Add NSP/NSZ/XCI/XCZ files or folders");
                    ui.label("2. Connect your Nintendo Switch via USB");
                    ui.label("3. Launch DBI on your Switch");
                    ui.label("4. Select 'Install title from DBIbackend'");
                    ui.label("5. Click 'Start Server' above");
                    
                    ui.separator();
                    
                    // Transfer Progress
                    if self.server_running {
                        if let Ok(progress) = self.progress.lock() {
                            ui.heading("▶ Transfer Progress");
                            
                            if !progress.current_file.is_empty() {
                                ui.label(format!("• File: {}", progress.current_file));
                            }
                            
                            let progress_ratio = if progress.total_size > 0 {
                                progress.bytes_sent as f32 / progress.total_size as f32
                            } else {
                                0.0
                            };
                            
                            ui.add(ProgressBar::new(progress_ratio)
                                .text(format!("{:.1}%", progress_ratio * 100.0)));
                            
                            ui.horizontal(|ui| {
                                ui.label(format!("↑ Sent: {}", format_file_size(progress.bytes_sent)));
                                ui.label(format!("/ {}", format_file_size(progress.total_size)));
                            });
                            
                            if progress.speed_mbps > 0.0 {
                                ui.label(format!("⚡ Speed: {:.2} MB/s", progress.speed_mbps));
                                
                                if progress.total_size > progress.bytes_sent && progress.speed_mbps > 0.0 {
                                    let remaining_bytes = progress.total_size - progress.bytes_sent;
                                    let remaining_seconds = (remaining_bytes as f64 / (progress.speed_mbps * 1_000_000.0)) as u64;
                                    let minutes = remaining_seconds / 60;
                                    let seconds = remaining_seconds % 60;
                                    ui.label(format!("⏱ ETA: {}m {}s", minutes, seconds));
                                }
                            }
                        }
                    }
                });
            });
        });
        
        // Central panel - File Library (full width)
        CentralPanel::default().show(ctx, |ui| {
            self.file_panel(ui);
        });
    }
}

impl DbiApp {
    fn file_panel(&mut self, ui: &mut Ui) {
        ui.heading("📁 File Library");

        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("+ Add Folder").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    if let Some(db) = &self.database {
                        match db.add_directory(&path, &["nsp", "nsz", "xci", "xcz"]) {
                            Ok(count) => {
                                self.log_messages.push(format!("[+] Added {} files from folder", count));
                                self.reload_file_list();
                            }
                            Err(e) => {
                                self.log_messages.push(format!("[!] Error: {}", e));
                            }
                        }
                    }
                }
            }

            if ui.button("+ Add Files").clicked() {
                if let Some(files) = rfd::FileDialog::new()
                    .add_filter("Switch Files", &["nsp", "nsz", "xci", "xcz"])
                    .pick_files()
                {
                    if let Some(db) = &self.database {
                        let mut count = 0;
                        for file in files {
                            if db.add_file(&file).is_ok() {
                                count += 1;
                            }
                        }
                        self.log_messages.push(format!("[+] Added {} files", count));
                        self.reload_file_list();
                    }
                }
            }

            if ui.button("🗑️ Clear All").clicked() {
                self.file_list.clear();
                self.log_messages.push("[x] Cleared file queue".to_string());
            }
            
            if ui.button("🔄 Refresh").clicked() {
                self.reload_file_list();
                self.log_messages.push("[*] Refreshed file list".to_string());
            }
        });

        ui.separator();

        // Search bar
        ui.horizontal(|ui| {
            ui.label("🔍 Search:");
            let response = ui.text_edit_singleline(&mut self.search_query);
            if response.changed() {
                self.reload_file_list();
            }
            if ui.button("✕").clicked() {
                self.search_query.clear();
                self.reload_file_list();
            }
        });

        ui.separator();

        // Statistics
        if let Some(db) = &self.database {
            if let Ok((count, total_size, installs)) = db.get_stats() {
                ui.horizontal(|ui| {
                    ui.label(format!("📦 {} files", count));
                    ui.label("|");
                    ui.label(format!("💾 {}", format_file_size(total_size)));
                    ui.label("|");
                    ui.label(format!("📥 {} installs", installs));
                });
            }
        }
        
        ui.label(format!("Queue: {}", self.file_list.len()));

        ui.separator();

        // File table
        use egui_extras::{TableBuilder, Column};
        
        let files_to_display = if let Some(db) = &self.database {
            if self.search_query.is_empty() {
                db.get_files().unwrap_or_default()
            } else {
                db.search(&self.search_query).unwrap_or_default()
            }
        } else {
            vec![]
        };

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(30.0)) // Favorite
            .column(Column::remainder().at_least(200.0)) // Filename
            .column(Column::auto().at_least(80.0)) // Size
            .column(Column::auto().at_least(60.0)) // Installs
            .column(Column::auto().at_least(100.0)) // Actions
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("⭐");
                });
                header.col(|ui| {
                    ui.strong("Filename");
                });
                header.col(|ui| {
                    ui.strong("Size");
                });
                header.col(|ui| {
                    ui.strong("Installs");
                });
                header.col(|ui| {
                    ui.strong("Actions");
                });
            })
            .body(|mut body| {
                for file in &files_to_display {
                    body.row(18.0, |mut row| {
                        // Favorite column
                        row.col(|ui| {
                            let star = if file.favorite { "⭐" } else { "☆" };
                            if ui.button(star).clicked() {
                                if let Some(db) = &self.database {
                                    let _ = db.toggle_favorite(file.id);
                                    self.reload_file_list();
                                }
                            }
                        });
                        
                        // Filename column with truncation
                        row.col(|ui| {
                            ui.label(&file.filename)
                                .on_hover_text(&file.filename); // Show full name on hover
                        });
                        
                        // Size column
                        row.col(|ui| {
                            ui.label(format_file_size(file.size));
                        });
                        
                        // Install count column
                        row.col(|ui| {
                            ui.label(format!("{}", file.install_count));
                        });
                        
                        // Actions column
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                // Add to queue button
                                if ui.small_button("+").on_hover_text("Add to queue").clicked() {
                                    let path = PathBuf::from(&file.path);
                                    if path.exists() {
                                        self.file_list.insert(file.filename.clone(), path);
                                        self.log_messages.push(format!("[+] Added to queue: {}", file.filename));
                                    } else {
                                        self.log_messages.push(format!("[!] File not found: {}", file.filename));
                                    }
                                }
                                
                                // Delete button
                                if ui.small_button("✕").on_hover_text("Remove from library").clicked() {
                                    if let Some(db) = &self.database {
                                        let _ = db.remove_file(file.id);
                                        self.reload_file_list();
                                        self.log_messages.push(format!("[x] Removed: {}", file.filename));
                                    }
                                }
                            });
                        });
                    });
                }
            });
    }
    
    fn reload_file_list(&mut self) {
        // Reload file list from database
        // This will be called after any database operation
    }
    
    fn activity_log_panel(&mut self, ui: &mut Ui) {
        ui.heading("Activity Log");
        
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
        
        // Terminal-style log box - fill available space
        let available_height = ui.available_height();
        
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(20, 20, 20))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)))
            .inner_margin(8.0)
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .max_height(available_height)
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                        
                        for msg in self.log_messages.iter().rev().take(50) {
                            ui.colored_label(egui::Color32::from_rgb(200, 200, 200), msg);
                        }
                    });
            });
    }
}

pub fn launch_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0]) // 16:9 aspect ratio
            .with_title("DBI Backend - Rust Edition"),
        ..Default::default()
    };

    eframe::run_native(
        "DBI Backend - Rust Edition",
        options,
        Box::new(|_cc| Box::new(DbiApp::new())),
    ).unwrap();
}
