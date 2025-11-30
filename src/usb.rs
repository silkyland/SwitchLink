/// USB communication module for Nintendo Switch
use anyhow::{anyhow, Result};
use rusb::{Context, Device, DeviceHandle, Direction, UsbContext};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use crate::protocol::*;

#[derive(Debug, Clone, Default)]
pub struct TransferProgress {
    pub current_file: String,
    pub bytes_sent: u64,
    pub total_size: u64,
    pub speed_mbps: f64,
    pub logs: Vec<String>,
    last_update: Option<Instant>,
    bytes_at_last_update: u64,
}

impl TransferProgress {
    pub fn update_speed(&mut self) {
        if let Some(last_time) = self.last_update {
            let elapsed = last_time.elapsed().as_secs_f64();
            if elapsed >= 0.5 {
                let bytes_diff = self.bytes_sent - self.bytes_at_last_update;
                self.speed_mbps = (bytes_diff as f64 / elapsed) / 1_000_000.0;
                self.last_update = Some(Instant::now());
                self.bytes_at_last_update = self.bytes_sent;
            }
        } else {
            self.last_update = Some(Instant::now());
            self.bytes_at_last_update = self.bytes_sent;
        }
    }
    
    pub fn add_log(&mut self, message: String) {
        self.logs.push(message);
        // Keep only last 50 logs
        if self.logs.len() > 50 {
            self.logs.remove(0);
        }
    }
}

// Nintendo Switch USB IDs
const SWITCH_VENDOR_ID: u16 = 0x057E;
const SWITCH_PRODUCT_ID: u16 = 0x3000;

const TIMEOUT: Duration = Duration::from_millis(100);
const TIMEOUT_LONG: Duration = Duration::from_secs(30); // For waiting ACK from Switch

pub struct UsbConnection {
    handle: DeviceHandle<Context>,
    in_endpoint: u8,
    out_endpoint: u8,
}

impl UsbConnection {
    pub fn connect() -> Result<Self> {
        info!("Searching for Nintendo Switch...");
        
        let context = Context::new()?;
        let device = Self::find_switch(&context)?;
        
        info!("Switch found, opening device...");
        let mut handle = device.open()?;
        
        // Reset device only on first connection
        // Don't reset on reconnect as it causes I/O errors
        if let Err(e) = handle.reset() {
            debug!("Reset failed (might be already reset): {}", e);
        }
        std::thread::sleep(Duration::from_millis(500));
        
        // Reopen after reset
        drop(handle);
        std::thread::sleep(Duration::from_millis(500));
        handle = device.open()?;
        
        // Set configuration
        if let Err(e) = handle.set_active_configuration(1) {
            debug!("Set configuration failed: {}", e);
        }
        
        // Claim interface 0
        handle.claim_interface(0)?;
        info!("Claimed interface 0");
        
        // Find endpoints
        let config_desc = device.config_descriptor(0)?;
        
        let mut in_endpoint = None;
        let mut out_endpoint = None;
        let mut interface_number = 0;
        
        for interface in config_desc.interfaces() {
            interface_number = interface.number();
            for descriptor in interface.descriptors() {
                for endpoint in descriptor.endpoint_descriptors() {
                    match endpoint.direction() {
                        Direction::In => in_endpoint = Some(endpoint.address()),
                        Direction::Out => out_endpoint = Some(endpoint.address()),
                    }
                }
            }
        }
        
        let in_endpoint = in_endpoint.ok_or_else(|| anyhow!("IN endpoint not found"))?;
        let out_endpoint = out_endpoint.ok_or_else(|| anyhow!("OUT endpoint not found"))?;
        
        info!("Connected to Switch (Interface: {}, IN: 0x{:02X}, OUT: 0x{:02X})", interface_number, in_endpoint, out_endpoint);
        
        Ok(Self {
            handle,
            in_endpoint,
            out_endpoint,
        })
    }
    
    fn find_switch(context: &Context) -> Result<Device<Context>> {
        for device in context.devices()?.iter() {
            let device_desc = device.device_descriptor()?;
            if device_desc.vendor_id() == SWITCH_VENDOR_ID
                && device_desc.product_id() == SWITCH_PRODUCT_ID
            {
                return Ok(device);
            }
        }
        Err(anyhow!("Nintendo Switch not found"))
    }
    
    pub fn read(&self, size: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; size];
        let bytes_read = self.handle.read_bulk(self.in_endpoint, &mut buf, TIMEOUT)?;
        buf.truncate(bytes_read);
        Ok(buf)
    }
    
    pub fn read_with_long_timeout(&self, size: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; size];
        let bytes_read = self.handle.read_bulk(self.in_endpoint, &mut buf, TIMEOUT_LONG)?;
        buf.truncate(bytes_read);
        Ok(buf)
    }
    
    pub fn write(&self, data: &[u8]) -> Result<usize> {
        Ok(self.handle.write_bulk(self.out_endpoint, data, TIMEOUT)?)
    }
    
    pub fn read_command_header(&self) -> Result<CommandHeader> {
        let data = self.read(16)?;
        Ok(CommandHeader::from_bytes(&data)?)
    }
    
    pub fn read_command_header_with_long_timeout(&self) -> Result<CommandHeader> {
        let data = self.read_with_long_timeout(16)?;
        Ok(CommandHeader::from_bytes(&data)?)
    }
    
    pub fn write_command_header(&self, header: &CommandHeader) -> Result<()> {
        self.write(&header.to_bytes())?;
        Ok(())
    }
}

pub struct SwitchLinkServer {
    connection: Option<UsbConnection>,
    file_list: Arc<Mutex<HashMap<String, PathBuf>>>,
    running: Arc<Mutex<bool>>,
    progress: Option<Arc<Mutex<TransferProgress>>>,
}

impl SwitchLinkServer {
    pub fn new(file_list: Arc<Mutex<HashMap<String, PathBuf>>>) -> Self {
        Self {
            connection: None,
            file_list,
            running: Arc::new(Mutex::new(false)),
            progress: None,
        }
    }
    
    pub fn new_with_progress(
        file_list: Arc<Mutex<HashMap<String, PathBuf>>>,
        progress: Arc<Mutex<TransferProgress>>,
    ) -> Self {
        Self {
            connection: None,
            file_list,
            running: Arc::new(Mutex::new(false)),
            progress: Some(progress),
        }
    }
    
    pub fn connect(&mut self) -> Result<()> {
        self.connection = Some(UsbConnection::connect()?);
        Ok(())
    }
    
    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }
    
    pub fn start(&mut self) -> Result<()> {
        if self.connection.is_none() {
            self.connect()?;
        }
        
        *self.running.lock().unwrap() = true;
        info!("SwitchLink Server started, entering command loop");
        
        self.poll_commands()
    }
    
    pub fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
        self.connection = None;
        info!("SwitchLink Server stopped");
    }
    
    fn poll_commands(&mut self) -> Result<()> {
        let mut reconnect_attempts = 0;
        const MAX_RECONNECT_ATTEMPTS: u32 = 3;
        
        while *self.running.lock().unwrap() {
            // Check connection and try to reconnect if needed
            if self.connection.is_none() && reconnect_attempts < MAX_RECONNECT_ATTEMPTS {
                warn!("Connection lost, attempting to reconnect... (attempt {}/{})", 
                      reconnect_attempts + 1, MAX_RECONNECT_ATTEMPTS);
                
                match self.connect() {
                    Ok(_) => {
                        info!("Reconnected successfully!");
                        reconnect_attempts = 0;
                    }
                    Err(e) => {
                        reconnect_attempts += 1;
                        error!("Reconnect failed: {}", e);
                        if reconnect_attempts >= MAX_RECONNECT_ATTEMPTS {
                            return Err(anyhow!("Failed to reconnect after {} attempts", MAX_RECONNECT_ATTEMPTS));
                        }
                        std::thread::sleep(Duration::from_secs(2));
                        continue;
                    }
                }
            }
            
            let header_result = {
                let conn = self.connection.as_ref().unwrap();
                conn.read_command_header()
            };
            
            match header_result {
                Ok(header) => {
                    reconnect_attempts = 0; // Reset on successful read
                    debug!("Received command: type={}, id={}, size={}",
                           header.cmd_type, header.cmd_id, header.data_size);
                    
                    match header.cmd_id {
                        CMD_ID_EXIT => {
                            self.process_exit_command()?;
                            break;
                        }
                        CMD_ID_FILE_RANGE => {
                            self.process_file_range_command(header.data_size)?;
                        }
                        CMD_ID_LIST => {
                            self.process_list_command()?;
                        }
                        _ => {
                            warn!("Unknown command ID: {}", header.cmd_id);
                        }
                    }
                }
                Err(e) => {
                    if let Some(rusb_err) = e.downcast_ref::<rusb::Error>() {
                        match rusb_err {
                            rusb::Error::Timeout => {
                                // Timeout is normal - just continue polling
                                continue;
                            }
                            rusb::Error::Pipe | rusb::Error::Io => {
                                // Pipe/IO error might be normal during connection setup
                                // Just continue and wait for actual commands
                                debug!("USB pipe/IO error (might be normal): {}", e);
                                std::thread::sleep(Duration::from_millis(100));
                                continue;
                            }
                            rusb::Error::NoDevice => {
                                warn!("Switch disconnected, will try to reconnect...");
                                self.connection = None;
                                std::thread::sleep(Duration::from_secs(1));
                                continue;
                            }
                            _ => {
                                error!("USB error: {}", e);
                                // Don't reconnect immediately, just continue
                                std::thread::sleep(Duration::from_millis(100));
                                continue;
                            }
                        }
                    } else {
                        error!("Error reading command: {}", e);
                        std::thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn process_exit_command(&self) -> Result<()> {
        info!("Processing EXIT command");
        let conn = self.connection.as_ref().unwrap();
        
        let response = CommandHeader::new(CMD_TYPE_RESPONSE, CMD_ID_EXIT, 0);
        conn.write_command_header(&response)?;
        
        Ok(())
    }
    
    fn process_list_command(&self) -> Result<()> {
        info!("Processing LIST command");
        let conn = self.connection.as_ref().unwrap();
        let file_list = self.file_list.lock().unwrap();
        
        let mut nsp_path_list = String::new();
        for (name, path) in file_list.iter() {
            // Get file size
            let size = if let Ok(metadata) = std::fs::metadata(path) {
                metadata.len()
            } else {
                0
            };
            
            // Format: filename|size\n
            nsp_path_list.push_str(name);
            nsp_path_list.push('|');
            nsp_path_list.push_str(&size.to_string());
            nsp_path_list.push('\n');
        }
        
        let nsp_path_list_bytes = nsp_path_list.as_bytes();
        let list_len = nsp_path_list_bytes.len() as u32;
        
        info!("Sending file list response: {} files, {} bytes", file_list.len(), list_len);
        debug!("File list content: {}", nsp_path_list.trim());
        
        let response = CommandHeader::new(CMD_TYPE_RESPONSE, CMD_ID_LIST, list_len);
        conn.write_command_header(&response)?;
        info!("Sent LIST response header");
        
        if list_len > 0 {
            // Wait for ACK before sending file list (like Python version)
            // Use longer timeout for ACK from Switch
            info!("Waiting for ACK from Switch...");
            let _ack = conn.read_command_header_with_long_timeout()?;
            info!("Received ACK from Switch");
            
            conn.write(nsp_path_list_bytes)?;
            info!("Sent file list data ({} bytes)", list_len);
        } else {
            info!("File list is empty, no data to send");
        }
        
        Ok(())
    }
    
    fn process_file_range_command(&self, data_size: u32) -> Result<()> {
        info!("Processing FILE_RANGE command (data_size={})", data_size);
        let conn = self.connection.as_ref().unwrap();
        
        // Send ACK
        let ack = CommandHeader::new(CMD_TYPE_ACK, CMD_ID_FILE_RANGE, data_size);
        conn.write_command_header(&ack)?;
        info!("Sent ACK for FILE_RANGE");
        
        // Read file range header with longer timeout
        let header_data = conn.read_with_long_timeout(data_size as usize)?;
        
        // Debug: Show raw bytes
        if header_data.len() >= 16 {
            debug!("Raw header bytes: {:02X?}", &header_data[..16]);
            debug!("range_size bytes [0-4]: {:02X?}", &header_data[0..4]);
            debug!("range_offset bytes [4-12]: {:02X?}", &header_data[4..12]);
            debug!("nsp_name_len bytes [12-16]: {:02X?}", &header_data[12..16]);
        }
        
        let file_range = FileRangeHeader::from_bytes(&header_data)?;
        
        info!(
            "File range request: name={}, offset={}, size={}",
            file_range.nsp_name, file_range.range_offset, file_range.range_size
        );
        
        // Calculate actual size to send BEFORE sending response header
        let file_list = self.file_list.lock().unwrap();
        let file_path = match file_list.get(&file_range.nsp_name) {
            Some(path) => path.clone(),
            None => {
                error!("File not found: {}", file_range.nsp_name);
                // Send response with 0 bytes to signal error
                let response = CommandHeader::new(CMD_TYPE_RESPONSE, CMD_ID_FILE_RANGE, 0);
                conn.write_command_header(&response)?;
                return Err(anyhow!("File not found"));
            }
        };
        drop(file_list); // Release lock early
        
        // Get file size and calculate actual bytes to send
        let metadata = std::fs::metadata(&file_path)?;
        let file_size = metadata.len();
        let offset = file_range.range_offset;
        let requested_size = file_range.range_size as usize;
        
        let actual_size = if offset >= file_size {
            0 // No more data to send
        } else {
            let available = (file_size - offset) as usize;
            std::cmp::min(requested_size, available)
        };
        
        info!("Calculated actual_size={} (requested={}, available from offset {})", 
              actual_size, requested_size, offset);
        
        // Send response with ACTUAL size (not requested size)
        let response = CommandHeader::new(
            CMD_TYPE_RESPONSE,
            CMD_ID_FILE_RANGE,
            actual_size as u32,
        );
        conn.write_command_header(&response)?;
        info!("Sent FILE_RANGE response header with size={}", actual_size);
        
        // Wait for ACK with longer timeout
        info!("Waiting for ACK from Switch...");
        let _ack = conn.read_command_header_with_long_timeout()?;
        info!("Received ACK from Switch");
        
        // Update progress - set current file and total size
        if let Some(progress) = &self.progress {
            if let Ok(mut p) = progress.lock() {
                if p.current_file != file_range.nsp_name {
                    p.current_file = file_range.nsp_name.clone();
                    p.bytes_sent = 0;
                    p.total_size = file_size;
                    p.add_log(format!("[>] Transferring: {}", file_range.nsp_name));
                }
            }
        }
        
        // Send file data (only if there's data to send)
        if actual_size > 0 {
            self.send_file_range(&file_path, offset, actual_size)?;
        } else {
            info!("No data to send (offset {} >= file_size {})", offset, file_size);
        }
        
        Ok(())
    }
    
    fn send_file_range(
        &self,
        file_path: &PathBuf,
        offset: u64,
        size: usize,
    ) -> Result<()> {
        let conn = self.connection.as_ref().unwrap();
        
        // Size is already calculated correctly by process_file_range_command
        // Just open file and send the exact number of bytes
        let mut file = File::open(file_path)?;
        file.seek(SeekFrom::Start(offset))?;
        
        let mut curr_off = 0;
        let mut buffer = vec![0u8; BUFFER_SEGMENT_DATA_SIZE];
        
        while curr_off < size {
            let read_size = std::cmp::min(BUFFER_SEGMENT_DATA_SIZE, size - curr_off);
            let bytes_read = file.read(&mut buffer[..read_size])?;
            
            if bytes_read == 0 {
                warn!("Unexpected EOF at offset {}, sent {} / {} bytes", 
                      offset + curr_off as u64, curr_off, size);
                break;
            }
            
            conn.write(&buffer[..bytes_read])?;
            curr_off += bytes_read;
            
            // Update progress
            if let Some(progress) = &self.progress {
                if let Ok(mut p) = progress.lock() {
                    p.bytes_sent += bytes_read as u64;
                    p.update_speed();
                }
            }
            
            if curr_off % (10 * 1024 * 1024) == 0 {
                debug!("Sent {} / {} bytes", curr_off, size);
            }
        }
        
        info!("File transfer complete: {} bytes sent", curr_off);
        Ok(())
    }
}
