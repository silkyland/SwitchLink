/// SwitchLink Protocol constants and structures
use std::io::{self, Read};
use bytes::{Buf, BufMut, BytesMut};

// Command IDs
pub const CMD_ID_EXIT: u32 = 0;
pub const CMD_ID_LIST_OLD: u32 = 1;
pub const CMD_ID_FILE_RANGE: u32 = 2;
pub const CMD_ID_LIST: u32 = 3;

// Command Types
pub const CMD_TYPE_REQUEST: u32 = 0;
pub const CMD_TYPE_RESPONSE: u32 = 1;
pub const CMD_TYPE_ACK: u32 = 2;

// Buffer size
pub const BUFFER_SEGMENT_DATA_SIZE: usize = 0x100000; // 1MB

// Magic bytes
pub const MAGIC: &[u8; 4] = b"DBI0";

#[derive(Debug, Clone)]
pub struct CommandHeader {
    pub magic: [u8; 4],
    pub cmd_type: u32,
    pub cmd_id: u32,
    pub data_size: u32,
}

impl CommandHeader {
    pub fn new(cmd_type: u32, cmd_id: u32, data_size: u32) -> Self {
        Self {
            magic: *MAGIC,
            cmd_type,
            cmd_id,
            data_size,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(16);
        buf.put_slice(&self.magic);
        buf.put_u32_le(self.cmd_type);
        buf.put_u32_le(self.cmd_id);
        buf.put_u32_le(self.data_size);
        buf.to_vec()
    }

    pub fn from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() < 16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid header size",
            ));
        }

        let mut buf = &data[..];
        let mut magic = [0u8; 4];
        buf.read_exact(&mut magic)?;

        if &magic != MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid magic bytes",
            ));
        }

        let cmd_type = buf.get_u32_le();
        let cmd_id = buf.get_u32_le();
        let data_size = buf.get_u32_le();

        Ok(Self {
            magic,
            cmd_type,
            cmd_id,
            data_size,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileRangeHeader {
    pub range_size: u32,
    pub range_offset: u64, // Keep as u64 for compatibility, but parse differently
    pub nsp_name_len: u32,
    pub nsp_name: String,
}

impl FileRangeHeader {
    pub fn from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() < 16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid file range header size",
            ));
        }

        // Parse header: range_size(4) + range_offset(8) + nsp_name_len(4) = 16 bytes
        let range_size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        
        // Try parsing offset as u64 first (Python way)
        let range_offset_u64 = u64::from_le_bytes([
            data[4], data[5], data[6], data[7],
            data[8], data[9], data[10], data[11],
        ]);
        
        // If offset seems unreasonably large (> 100GB), try u32 instead
        // This might be a protocol quirk or endianness issue
        let range_offset = if range_offset_u64 > 100_000_000_000 {
            let offset_u32 = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
            eprintln!("WARNING: offset {} seems too large, using u32 value {} instead", 
                     range_offset_u64, offset_u32);
            offset_u32 as u64
        } else {
            range_offset_u64
        };
        
        let nsp_name_len = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);

        // Filename starts at byte 16
        if data.len() < 16 + nsp_name_len as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Filename length exceeds available data",
            ));
        }

        let nsp_name_bytes = &data[16..16 + nsp_name_len as usize];
        let nsp_name = String::from_utf8(nsp_name_bytes.to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(Self {
            range_size,
            range_offset,
            nsp_name_len,
            nsp_name,
        })
    }
}
