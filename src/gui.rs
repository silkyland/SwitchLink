/// eGUI version - Modern, Beautiful UI for SwitchLink
use eframe::egui;
use eframe::egui::{CentralPanel, Context, ProgressBar, ScrollArea, Ui, Color32, Stroke, Rounding, Vec2};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use crate::database::Database;
use crate::file_manager::{add_files, add_files_from_directory, format_file_size};
use crate::usb::{DbiServer, TransferProgress};

// Modern Color Palette
pub struct ColorTheme {
    // Primary colors
    primary: Color32,
    primary_hover: Color32,
    primary_dark: Color32,
    
    // Accent colors
    accent: Color32,
    accent_hover: Color32,
    
    // Status colors
    success: Color32,
    warning: Color32,
    error: Color32,
    info: Color32,
    
    // Background colors
    bg_primary: Color32,
    bg_secondary: Color32,
    bg_tertiary: Color32,
    
    // Text colors
    text_primary: Color32,
    text_secondary: Color32,
    text_muted: Color32,
    
    // Border colors
    border: Color32,
    border_hover: Color32,
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            // Vibrant purple-blue gradient theme
            primary: Color32::from_rgb(99, 102, 241),        // Indigo
            primary_hover: Color32::from_rgb(79, 70, 229),   // Darker indigo
            primary_dark: Color32::from_rgb(67, 56, 202),    // Deep indigo
            
            accent: Color32::from_rgb(236, 72, 153),         // Pink
            accent_hover: Color32::from_rgb(219, 39, 119),   // Darker pink
            
            success: Color32::from_rgb(34, 197, 94),         // Green
            warning: Color32::from_rgb(251, 191, 36),        // Amber
            error: Color32::from_rgb(239, 68, 68),           // Red
            info: Color32::from_rgb(59, 130, 246),           // Blue
            
            bg_primary: Color32::from_rgb(15, 23, 42),       // Slate 900
            bg_secondary: Color32::from_rgb(30, 41, 59),     // Slate 800
            bg_tertiary: Color32::from_rgb(51, 65, 85),      // Slate 700
            
            text_primary: Color32::from_rgb(248, 250, 252),  // Slate 50
            text_secondary: Color32::from_rgb(203, 213, 225), // Slate 300
            text_muted: Color32::from_rgb(148, 163, 184),    // Slate 400
            
            border: Color32::from_rgba_premultiplied(71, 85, 105, 100), // Slate 600 with alpha
            border_hover: Color32::from_rgb(100, 116, 139),  // Slate 500
        }
    }
}

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
    theme: ColorTheme,
    animation_time: f32,
}

impl DbiApp {
    pub fn new() -> Self {
        // Initialize database
        let db_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("switchlink")
            .join("games.db");
        
        // Create directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        
        let database = Database::new(&db_path).ok();
        
        Self {
            log_messages: vec!["🚀 SwitchLink started - Ready to transfer!".to_string()],
            connection_status: "Disconnected".to_string(),
            progress: Arc::new(Mutex::new(TransferProgress::default())),
            database,
            search_query: String::new(),
            theme: ColorTheme::default(),
            animation_time: 0.0,
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
        self.log_messages.push("[>] Starting SwitchLink server...".to_string());
        self.log_messages.push("[i] Connect your Switch and select 'Install title from SwitchLink'".to_string());
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
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // Update animation time
        self.animation_time += ctx.input(|i| i.stable_dt);
        
        // Request repaint continuously when server is running for smooth progress updates
        if self.server_running {
            ctx.request_repaint();
        }
        
        // Apply custom theme
        self.apply_custom_theme(ctx);
        
        // Check for close request and show confirmation if transfer is in progress
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.server_running {
                // Show confirmation dialog
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                
                egui::Window::new("⚠️ Confirm Close")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(10.0);
                            ui.heading("Transfer in Progress");
                            ui.add_space(10.0);
                            ui.label("Are you sure you want to close?");
                            ui.label("This will interrupt the current transfer.");
                            ui.add_space(20.0);
                            
                            ui.horizontal(|ui| {
                                if self.danger_button(ui, "Yes, Close").clicked() {
                                    self.stop_server();
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                                ui.add_space(10.0);
                                if self.secondary_button(ui, "Cancel").clicked() {
                                    // Just close the dialog
                                }
                            });
                        });
                    });
            }
        }
        
        // Top panel - Header with gradient
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::none()
                .fill(self.theme.bg_secondary)
                .inner_margin(egui::Margin::symmetric(20.0, 15.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // App title with gradient effect
                    ui.heading(egui::RichText::new("🎮 SwitchLink")
                        .size(24.0)
                        .color(self.theme.text_primary));
                    
                    ui.add_space(20.0);
                    
                    // Server status badge
                    let (status_text, status_color) = if self.server_running {
                        ("● Running", self.theme.success)
                    } else {
                        ("○ Stopped", self.theme.text_muted)
                    };
                    
                    egui::Frame::none()
                        .fill(if self.server_running { 
                            Color32::from_rgba_premultiplied(34, 197, 94, 30) 
                        } else { 
                            self.theme.bg_tertiary 
                        })
                        .rounding(Rounding::same(12.0))
                        .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(status_text)
                                .color(status_color)
                                .size(13.0));
                        });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Control buttons
                        if self.server_running {
                            if self.danger_button(ui, "■ Stop Server").clicked() {
                                self.stop_server();
                            }
                        } else {
                            if self.primary_button(ui, "▶ Start Server").clicked() {
                                self.start_server();
                            }
                        }
                    });
                });
            });
        
        // Bottom panel - Status bar with gradient
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::none()
                .fill(self.theme.bg_secondary)
                .inner_margin(egui::Margin::symmetric(20.0, 10.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("v0.1.0")
                        .color(self.theme.text_muted)
                        .size(12.0));
                    
                    ui.separator();
                    
                    ui.label(egui::RichText::new("Built with Rust 🦀")
                        .color(self.theme.text_muted)
                        .size(12.0));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(
                            egui::RichText::new("☕ Buy me a coffee")
                                .color(self.theme.text_primary)
                                .size(12.0))
                            .fill(self.theme.accent)
                            .rounding(Rounding::same(8.0)))
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked() 
                        {
                            let _ = open::that("https://buymeacoffee.com/silkyland");
                        }
                    });
                });
            });
        
        // Bottom panel for Activity Log and Transfer Progress
        egui::TopBottomPanel::bottom("activity_panel")
            .min_height(250.0)
            .frame(egui::Frame::none()
                .fill(self.theme.bg_primary)
                .inner_margin(egui::Margin::same(20.0)))
            .show(ctx, |ui| {
                ui.columns(2, |columns| {
                    // Left - Activity Log
                    columns[0].vertical(|ui| {
                        self.activity_log_panel(ui);
                    });
                    
                    // Right - Instructions and Progress
                    columns[1].vertical(|ui| {
                        if self.server_running {
                            self.transfer_progress_panel(ui);
                        } else {
                            self.instructions_panel(ui);
                        }
                    });
                });
            });
        
        // Central panel - File Library with gradient background
        CentralPanel::default()
            .frame(egui::Frame::none()
                .fill(self.theme.bg_primary)
                .inner_margin(egui::Margin::same(20.0)))
            .show(ctx, |ui| {
                self.file_panel(ui);
            });
    }
}

impl DbiApp {
    // Apply custom theme to the context
    fn apply_custom_theme(&self, ctx: &Context) {
        let mut style = (*ctx.style()).clone();
        
        // Set dark background
        style.visuals.window_fill = self.theme.bg_primary;
        style.visuals.panel_fill = self.theme.bg_secondary;
        style.visuals.extreme_bg_color = self.theme.bg_tertiary;
        
        // Set text colors
        style.visuals.override_text_color = Some(self.theme.text_primary);
        
        // Set widget colors
        style.visuals.widgets.noninteractive.bg_fill = self.theme.bg_tertiary;
        style.visuals.widgets.inactive.bg_fill = self.theme.bg_tertiary;
        style.visuals.widgets.hovered.bg_fill = self.theme.primary_hover;
        style.visuals.widgets.active.bg_fill = self.theme.primary;
        
        // Set rounding
        style.visuals.widgets.noninteractive.rounding = Rounding::same(8.0);
        style.visuals.widgets.inactive.rounding = Rounding::same(8.0);
        style.visuals.widgets.hovered.rounding = Rounding::same(8.0);
        style.visuals.widgets.active.rounding = Rounding::same(8.0);
        
        // Set spacing
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 8.0);
        
        ctx.set_style(style);
    }
    
    // Custom primary button
    fn primary_button(&self, ui: &mut Ui, text: &str) -> egui::Response {
        ui.add(egui::Button::new(
            egui::RichText::new(text)
                .color(Color32::WHITE)
                .size(14.0))
            .fill(self.theme.primary)
            .rounding(Rounding::same(8.0))
            .min_size(Vec2::new(120.0, 36.0)))
            .on_hover_cursor(egui::CursorIcon::PointingHand)
    }
    
    // Custom secondary button
    fn secondary_button(&self, ui: &mut Ui, text: &str) -> egui::Response {
        ui.add(egui::Button::new(
            egui::RichText::new(text)
                .color(self.theme.text_primary)
                .size(14.0))
            .fill(self.theme.bg_tertiary)
            .rounding(Rounding::same(8.0))
            .min_size(Vec2::new(120.0, 36.0)))
            .on_hover_cursor(egui::CursorIcon::PointingHand)
    }
    
    // Custom danger button
    fn danger_button(&self, ui: &mut Ui, text: &str) -> egui::Response {
        ui.add(egui::Button::new(
            egui::RichText::new(text)
                .color(Color32::WHITE)
                .size(14.0))
            .fill(self.theme.error)
            .rounding(Rounding::same(8.0))
            .min_size(Vec2::new(120.0, 36.0)))
            .on_hover_cursor(egui::CursorIcon::PointingHand)
    }
    
    // Instructions panel
    fn instructions_panel(&self, ui: &mut Ui) {
        egui::Frame::none()
            .fill(self.theme.bg_secondary)
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::Margin::same(20.0))
            .stroke(Stroke::new(1.0, self.theme.border))
            .show(ui, |ui| {
                ui.heading(egui::RichText::new("📖 Quick Start Guide")
                    .color(self.theme.text_primary)
                    .size(18.0));
                
                ui.add_space(15.0);
                
                let steps = [
                    ("1", "Add NSP/NSZ/XCI/XCZ files or folders", "📁"),
                    ("2", "Connect your Nintendo Switch via USB", "🔌"),
                    ("3", "Launch SwitchLink Client on your Switch", "🎮"),
                    ("4", "Select 'Install title from SwitchLink'", "📲"),
                    ("5", "Click 'Start Server' button above", "▶"),
                ];
                
                for (num, text, icon) in steps.iter() {
                    ui.horizontal(|ui| {
                        egui::Frame::none()
                            .fill(self.theme.primary)
                            .rounding(Rounding::same(6.0))
                            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(*num)
                                    .color(Color32::WHITE)
                                    .size(12.0)
                                    .strong());
                            });
                        
                        ui.label(egui::RichText::new(format!("{} {}", icon, text))
                            .color(self.theme.text_secondary)
                            .size(13.0));
                    });
                    ui.add_space(8.0);
                }
                
                ui.add_space(10.0);
                
                // Tips section
                egui::Frame::none()
                    .fill(Color32::from_rgba_premultiplied(59, 130, 246, 20))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("💡")
                                .size(16.0));
                            ui.label(egui::RichText::new("Tip: You can add multiple files at once and queue them for installation!")
                                .color(self.theme.info)
                                .size(12.0));
                        });
                    });
            });
    }
    
    // Transfer progress panel
    fn transfer_progress_panel(&self, ui: &mut Ui) {
        egui::Frame::none()
            .fill(self.theme.bg_secondary)
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::Margin::same(20.0))
            .stroke(Stroke::new(1.0, self.theme.border))
            .show(ui, |ui| {
                if let Ok(progress) = self.progress.lock() {
                    ui.heading(egui::RichText::new("📊 Transfer Progress")
                        .color(self.theme.text_primary)
                        .size(18.0));
                    
                    ui.add_space(15.0);
                    
                    // Current file
                    if !progress.current_file.is_empty() {
                        ui.label(egui::RichText::new("Current File")
                            .color(self.theme.text_muted)
                            .size(12.0));
                        ui.label(egui::RichText::new(&progress.current_file)
                            .color(self.theme.text_primary)
                            .size(14.0)
                            .strong());
                        ui.add_space(15.0);
                    }
                    
                    // Progress bar
                    let progress_ratio = if progress.total_size > 0 {
                        progress.bytes_sent as f32 / progress.total_size as f32
                    } else {
                        0.0
                    };
                    
                    ui.label(egui::RichText::new("Progress")
                        .color(self.theme.text_muted)
                        .size(12.0));
                    
                    let progress_bar = ProgressBar::new(progress_ratio)
                        .fill(self.theme.success)
                        .animate(true);
                    ui.add(progress_bar);
                    
                    ui.label(egui::RichText::new(format!("{:.1}%", progress_ratio * 100.0))
                        .color(self.theme.success)
                        .size(16.0)
                        .strong());
                    
                    ui.add_space(15.0);
                    
                    // Stats grid
                    ui.columns(2, |columns| {
                        // Left column
                        columns[0].vertical(|ui| {
                            self.stat_card(ui, "📤 Transferred", 
                                &format_file_size(progress.bytes_sent), 
                                self.theme.info);
                            
                            if progress.speed_mbps > 0.0 {
                                ui.add_space(10.0);
                                self.stat_card(ui, "⚡ Speed", 
                                    &format!("{:.2} MB/s", progress.speed_mbps), 
                                    self.theme.warning);
                            }
                        });
                        
                        // Right column
                        columns[1].vertical(|ui| {
                            self.stat_card(ui, "💾 Total Size", 
                                &format_file_size(progress.total_size), 
                                self.theme.text_muted);
                            
                            if progress.total_size > progress.bytes_sent && progress.speed_mbps > 0.0 {
                                ui.add_space(10.0);
                                let remaining_bytes = progress.total_size - progress.bytes_sent;
                                let remaining_seconds = (remaining_bytes as f64 / (progress.speed_mbps * 1_000_000.0)) as u64;
                                let minutes = remaining_seconds / 60;
                                let seconds = remaining_seconds % 60;
                                self.stat_card(ui, "⏱ ETA", 
                                    &format!("{}m {}s", minutes, seconds), 
                                    self.theme.accent);
                            }
                        });
                    });
                }
            });
    }
    
    // Stat card helper
    fn stat_card(&self, ui: &mut Ui, label: &str, value: &str, color: Color32) {
        egui::Frame::none()
            .fill(self.theme.bg_tertiary)
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new(label)
                    .color(self.theme.text_muted)
                    .size(11.0));
                ui.label(egui::RichText::new(value)
                    .color(color)
                    .size(14.0)
                    .strong());
            });
    }
    
    fn file_panel(&mut self, ui: &mut Ui) {
        // Header card
        egui::Frame::none()
            .fill(self.theme.bg_secondary)
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::Margin::same(20.0))
            .stroke(Stroke::new(1.0, self.theme.border))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("📁 File Library")
                        .color(self.theme.text_primary)
                        .size(20.0));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Statistics
                        if let Some(db) = &self.database {
                            if let Ok((count, total_size, installs)) = db.get_stats() {
                                egui::Frame::none()
                                    .fill(self.theme.bg_tertiary)
                                    .rounding(Rounding::same(8.0))
                                    .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(format!("📦 {} files", count))
                                                .color(self.theme.text_secondary)
                                                .size(12.0));
                                            ui.separator();
                                            ui.label(egui::RichText::new(format!("💾 {}", format_file_size(total_size)))
                                                .color(self.theme.text_secondary)
                                                .size(12.0));
                                            ui.separator();
                                            ui.label(egui::RichText::new(format!("📥 {} installs", installs))
                                                .color(self.theme.text_secondary)
                                                .size(12.0));
                                        });
                                    });
                            }
                        }
                    });
                });
            });
        
        ui.add_space(15.0);

        // Action buttons card
        egui::Frame::none()
            .fill(self.theme.bg_secondary)
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::Margin::same(15.0))
            .stroke(Stroke::new(1.0, self.theme.border))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Add Folder button
                    if ui.add(egui::Button::new(
                        egui::RichText::new("📁 Add Folder")
                            .color(self.theme.text_primary)
                            .size(13.0))
                        .fill(self.theme.primary)
                        .rounding(Rounding::same(8.0)))
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked() 
                    {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            if let Some(db) = &self.database {
                                match db.add_directory(&path, &["nsp", "nsz", "xci", "xcz"]) {
                                    Ok(count) => {
                                        self.log_messages.push(format!("✅ Added {} files from folder", count));
                                        self.reload_file_list();
                                    }
                                    Err(e) => {
                                        self.log_messages.push(format!("❌ Error: {}", e));
                                    }
                                }
                            }
                        }
                    }

                    // Add Files button
                    if ui.add(egui::Button::new(
                        egui::RichText::new("📄 Add Files")
                            .color(self.theme.text_primary)
                            .size(13.0))
                        .fill(self.theme.primary)
                        .rounding(Rounding::same(8.0)))
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked() 
                    {
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
                                self.log_messages.push(format!("✅ Added {} files", count));
                                self.reload_file_list();
                            }
                        }
                    }

                    ui.add_space(10.0);

                    // Clear All button
                    if ui.add(egui::Button::new(
                        egui::RichText::new("🗑️ Clear Queue")
                            .color(Color32::WHITE)
                            .size(13.0))
                        .fill(self.theme.error)
                        .rounding(Rounding::same(8.0)))
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked() 
                    {
                        self.file_list.clear();
                        self.log_messages.push("🗑️ Cleared file queue".to_string());
                    }
                    
                    // Refresh button
                    if ui.add(egui::Button::new(
                        egui::RichText::new("🔄 Refresh")
                            .color(self.theme.text_primary)
                            .size(13.0))
                        .fill(self.theme.bg_tertiary)
                        .rounding(Rounding::same(8.0)))
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked() 
                    {
                        self.reload_file_list();
                        self.log_messages.push("🔄 Refreshed file list".to_string());
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Queue badge
                        egui::Frame::none()
                            .fill(if self.file_list.is_empty() { 
                                self.theme.bg_tertiary 
                            } else { 
                                Color32::from_rgba_premultiplied(99, 102, 241, 40) 
                            })
                            .rounding(Rounding::same(8.0))
                            .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(format!("Queue: {}", self.file_list.len()))
                                    .color(if self.file_list.is_empty() { 
                                        self.theme.text_muted 
                                    } else { 
                                        self.theme.primary 
                                    })
                                    .size(13.0)
                                    .strong());
                            });
                    });
                });
            });

        ui.add_space(15.0);

        // Search bar card
        egui::Frame::none()
            .fill(self.theme.bg_secondary)
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::Margin::same(15.0))
            .stroke(Stroke::new(1.0, self.theme.border))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("�")
                        .size(16.0));
                    
                    let search_response = ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Search files...")
                            .desired_width(ui.available_width() - 40.0)
                    );
                    
                    if search_response.changed() {
                        self.reload_file_list();
                    }
                    
                    if !self.search_query.is_empty() {
                        if ui.add(egui::Button::new(
                            egui::RichText::new("✕")
                                .size(14.0))
                            .fill(self.theme.bg_tertiary)
                            .rounding(Rounding::same(6.0)))
                            .clicked() 
                        {
                            self.search_query.clear();
                            self.reload_file_list();
                        }
                    }
                });
            });

        ui.add_space(15.0);

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
                                // Add/Remove from queue button
                                let in_queue = self.file_list.contains_key(&file.filename);
                                let button_text = if in_queue { "-" } else { "+" };
                                let tooltip = if in_queue { "Remove from queue" } else { "Add to queue" };
                                
                                if ui.small_button(button_text).on_hover_text(tooltip).clicked() {
                                    if in_queue {
                                        self.file_list.remove(&file.filename);
                                        self.log_messages.push(format!("[-] Removed from queue: {}", file.filename));
                                    } else {
                                        let path = PathBuf::from(&file.path);
                                        if path.exists() {
                                            self.file_list.insert(file.filename.clone(), path);
                                            self.log_messages.push(format!("[+] Added to queue: {}", file.filename));
                                        } else {
                                            self.log_messages.push(format!("[!] File not found: {}", file.filename));
                                        }
                                    }
                                }
                                
                                // Delete button
                                if ui.small_button("Del").on_hover_text("Remove from library").clicked() {
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
        
        // Terminal-style log box with modern design
        egui::Frame::none()
            .fill(self.theme.bg_secondary)
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::Margin::same(20.0))
            .stroke(Stroke::new(1.0, self.theme.border))
            .show(ui, |ui| {
                ui.heading(egui::RichText::new("📋 Activity Log")
                    .color(self.theme.text_primary)
                    .size(18.0));
                
                ui.add_space(10.0);
                
                let available_height = ui.available_height() - 40.0;
                
                egui::Frame::none()
                    .fill(Color32::from_rgb(15, 23, 42))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(12.0))
                    .stroke(Stroke::new(1.0, Color32::from_rgba_premultiplied(71, 85, 105, 50)))
                    .show(ui, |ui| {
                        ScrollArea::vertical()
                            .max_height(available_height)
                            .stick_to_bottom(true)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                                
                                for msg in self.log_messages.iter().rev().take(50) {
                                    // Color code based on message type
                                    let color = if msg.contains("✅") || msg.contains("🚀") {
                                        self.theme.success
                                    } else if msg.contains("❌") || msg.contains("⚠️") {
                                        self.theme.error
                                    } else if msg.contains("🔄") {
                                        self.theme.info
                                    } else {
                                        self.theme.text_secondary
                                    };
                                    
                                    ui.colored_label(color, msg);
                                }
                            });
                    });
            });
    }
}

pub fn launch_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0]) // Larger window for better UX
            .with_min_inner_size([1024.0, 768.0]) // Minimum size
            .with_title("SwitchLink - Modern Edition"),
        ..Default::default()
    };

    eframe::run_native(
        "SwitchLink - Modern Edition",
        options,
        Box::new(|_cc| Box::new(DbiApp::new())),
    ).unwrap();
}
