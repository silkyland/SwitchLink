/// CLI version with basic functionality
use std::collections::HashMap;
use std::path::PathBuf;
use std::io::{self, Write};

fn main() {
    println!("🦀 DBI Backend - Rust Edition v0.1.0");
    println!("=====================================");
    println!();

    let mut file_list: HashMap<String, PathBuf> = HashMap::new();

    loop {
        println!("\n📋 Menu:");
        println!("1. 📂 Add folder");
        println!("2. 📄 Add files");
        println!("3. 📋 List files");
        println!("4. 🗑️ Clear list");
        println!("5. 🚀 Start server (demo)");
        println!("6. ❌ Exit");
        print!("\nChoose option: ");

        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();

        match choice {
            "1" => {
                print!("Enter folder path: ");
                io::stdout().flush().unwrap();
                let mut path = String::new();
                io::stdin().read_line(&mut path).unwrap();

                let path = path.trim();
                if let Ok(metadata) = std::fs::metadata(path) {
                    if metadata.is_dir() {
                        let count = add_folder_files(&mut file_list, PathBuf::from(path));
                        println!("✅ Added {} files from folder", count);
                    } else {
                        println!("❌ Path is not a directory");
                    }
                } else {
                    println!("❌ Directory not found");
                }
            }
            "2" => {
                print!("Enter file path: ");
                io::stdout().flush().unwrap();
                let mut path = String::new();
                io::stdin().read_line(&mut path).unwrap();

                let path = path.trim();
                if let Ok(metadata) = std::fs::metadata(path) {
                    if metadata.is_file() {
                        let file_name = PathBuf::from(path).file_name()
                            .unwrap().to_string_lossy().to_string();
                        file_list.insert(file_name, PathBuf::from(path));
                        println!("✅ Added file: {}", file_name);
                    } else {
                        println!("❌ Path is not a file");
                    }
                } else {
                    println!("❌ File not found");
                }
            }
            "3" => {
                println!("\n📁 File List ({} files):", file_list.len());
                if file_list.is_empty() {
                    println!("  No files in queue");
                } else {
                    for (name, path) in &file_list {
                        let size = std::fs::metadata(path).map(|m| format_file_size(m.len())).unwrap_or_default();
                        println!("  • {} ({})", name, size);
                    }
                }
            }
            "4" => {
                let count = file_list.len();
                file_list.clear();
                println!("✅ Cleared {} files", count);
            }
            "5" => {
                println!("\n🚀 Starting DBI Server Demo...");
                println!("📋 Files in queue: {}", file_list.len());
                println!();
                println!("💡 Instructions:");
                println!("  1. Connect Nintendo Switch via USB");
                println!("  2. Launch DBI on your Switch");
                println!("  3. Select 'Install title from DBIbackend'");
                println!();
                println!("🔄 Server demo mode - Press Enter to stop...");
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                println!("⏹️ Server stopped");
            }
            "6" => {
                println!("👋 Goodbye!");
                break;
            }
            _ => {
                println!("❌ Invalid choice. Please select 1-6.");
            }
        }
    }
}

fn add_folder_files(file_list: &mut HashMap<String, PathBuf>, folder_path: PathBuf) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(folder_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name() {
                        if let Some(name_str) = file_name.to_str() {
                            file_list.insert(name_str.to_string(), path);
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}
