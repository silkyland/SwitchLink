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

pub struct DbiServer {
    connection: Option<UsbConnection>,
    file_list: Arc<Mutex<HashMap<String, PathBuf>>>,
    running: Arc<Mutex<bool>>,
    progress: Option<Arc<Mutex<TransferProgress>>>,
}

impl DbiServer {
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
        info!("DBI Server started, entering command loop");
        
        self.poll_commands()
    }
    
    pub fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
        self.connection = None;
        info!("DBI Server stopped");
    }
    
    fn poll_commands(&mut self) -> Result<()> {
        while *self.running.lock().unwrap() {
            let header_result = {
                let conn = self.connection.as_ref().unwrap();
                conn.read_command_header()
            };
            
            match header_result {
                Ok(header) => {
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
                                error!("Switch disconnected");
                                return Err(anyhow!("Switch disconnected"));
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
        for (name, _) in file_list.iter() {
            nsp_path_list.push_str(name);
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
        
        // Send response
        let response = CommandHeader::new(
            CMD_TYPE_RESPONSE,
            CMD_ID_FILE_RANGE,
            file_range.range_size,
        );
        conn.write_command_header(&response)?;
        info!("Sent FILE_RANGE response header");
        
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
                    
                    // Get file size for total
                    let file_list = self.file_list.lock().unwrap();
                    if let Some(file_path) = file_list.get(&file_range.nsp_name) {
                        if let Ok(metadata) = std::fs::metadata(file_path) {
                            p.total_size = metadata.len();
                        }
                    }
                    
                    p.add_log(format!("[>] Transferring: {}", file_range.nsp_name));
                }
            }
        }
        
        // Send file data
        let file_list = self.file_list.lock().unwrap();
        if let Some(file_path) = file_list.get(&file_range.nsp_name) {
            self.send_file_range(
                file_path,
                file_range.range_offset,
                file_range.range_size as usize,
            )?;
        } else {
            error!("File not found: {}", file_range.nsp_name);
            return Err(anyhow!("File not found"));
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
        
        // Get file metadata to check size
        let metadata = std::fs::metadata(file_path)?;
        let file_size = metadata.len();
        
        debug!("File size: {}, requested offset: {}, requested size: {}", 
               file_size, offset, size);
        
        // Check if offset is beyond file size
        if offset >= file_size {
            warn!("Offset {} is beyond file size {}, sending 0 bytes", offset, file_size);
            info!("File transfer complete: 0 bytes sent (requested: {}, available: 0)", size);
            return Ok(());
        }
        
        let mut file = File::open(file_path)?;
        file.seek(SeekFrom::Start(offset))?;
        
        // Adjust size if it would read beyond file end
        let available = (file_size - offset) as usize;
        let actual_size = std::cmp::min(size, available);
        
        if actual_size < size {
            warn!("Requested {} bytes but only {} bytes available from offset {}", 
                  size, actual_size, offset);
        }
        
        let mut curr_off = 0;
        let mut buffer = vec![0u8; BUFFER_SEGMENT_DATA_SIZE];
        
        while curr_off < actual_size {
            let read_size = std::cmp::min(BUFFER_SEGMENT_DATA_SIZE, actual_size - curr_off);
            let bytes_read = file.read(&mut buffer[..read_size])?;
            
            if bytes_read == 0 {
                warn!("Unexpected EOF at offset {}, sent {} / {} bytes", 
                      offset + curr_off as u64, curr_off, actual_size);
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
                debug!("Sent {} / {} bytes", curr_off, actual_size);
            }
        }
        
        info!("File transfer complete: {} bytes sent (requested: {}, available: {})", 
              curr_off, size, actual_size);
        Ok(())
    }
}
