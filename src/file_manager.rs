/// File management utilities
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;

pub fn add_files_from_directory(
    file_list: &mut HashMap<String, PathBuf>,
    dir_path: PathBuf,
) -> Result<usize> {
    let mut count = 0;
    
    if !dir_path.is_dir() {
        return Ok(0);
    }
    
    for entry in std::fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(file_name) = path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    file_list.insert(name_str.to_string(), path.clone());
                    count += 1;
                }
            }
        }
    }
    
    Ok(count)
}

pub fn add_files(
    file_list: &mut HashMap<String, PathBuf>,
    file_paths: Vec<PathBuf>,
) -> usize {
    let mut count = 0;
    
    for path in file_paths {
        if path.is_file() {
            if let Some(file_name) = path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    file_list.insert(name_str.to_string(), path.clone());
                    count += 1;
                }
            }
        }
    }
    
    count
}

pub fn format_file_size(size: u64) -> String {
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

pub fn get_file_size(path: &PathBuf) -> u64 {
    std::fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(0)
}
