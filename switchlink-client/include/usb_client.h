#pragma once

#include <switch.h>
#include <cstdint>
#include <vector>
#include <string>
#include <functional>

// Protocol constants
// Protocol Magic Numbers
constexpr uint32_t PROTOCOL_MAGIC_SWLK = 0x4B4C5753; // "SWLK" (SwitchLink)
constexpr uint32_t PROTOCOL_MAGIC_LEGACY = 0x30494244;  // "DBI0" (Legacy support)
constexpr uint16_t PROTOCOL_VERSION = 1;

// Command types
// Command types (from Rust backend)
constexpr uint32_t CMD_TYPE_REQUEST = 0;
constexpr uint32_t CMD_TYPE_RESPONSE = 1;
constexpr uint32_t CMD_TYPE_ACK = 2;

enum class Command : uint32_t {
    EXIT = 0,
    LIST_OLD = 1,
    GET_FILE = 2, // CMD_ID_FILE_RANGE
    LIST = 3
};

// Protocol header
struct __attribute__((packed)) ProtocolHeader {
    uint32_t magic;
    uint32_t type;
    uint32_t command;
    uint32_t length;
};

// File information
struct FileInfo {
    std::string filename;
    uint64_t size;
    uint8_t sha256[32];
};

// USB Client class
class USBClient {
public:
    USBClient();
    ~USBClient();
    
    // Initialize USB connection
    bool initialize();
    
    // Close connection
    void close();
    
    // Check if connected
    bool isConnected() const { return m_connected; }
    
    // Send command
    bool sendCommand(uint32_t type, uint32_t cmdId, uint32_t length);
    
    // Send raw data
    bool sendRawData(const void* data, uint32_t length);
    
    // Receive data
    bool receiveData(void* buffer, uint32_t length);
    
    // List available files
    std::vector<FileInfo> listFiles();
    
    // Download a file
    // If fileSize is provided (> 0), it helps handling the last chunk correctly
    // progressCallback returns true to continue, false to cancel
    bool downloadFile(const std::string& filename, const std::string& destPath, std::function<bool(uint64_t, uint64_t)> progressCallback = nullptr, uint64_t fileSize = 0);
    
    static constexpr size_t CHUNK_SIZE = 1024 * 1024; // 1MB chunks for maximum performance
    static constexpr uint32_t USB_TIMEOUT = 5000; // 5 seconds
    
private:
    bool m_connected;
    // usbComms uses global state, no need for member variables
};
