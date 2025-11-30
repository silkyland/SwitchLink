#include "usb_client.h"
#include <cstring>
#include <cstdio>

USBClient::USBClient() : m_connected(false) {
}

USBClient::~USBClient() {
    close();
}

bool USBClient::initialize() {
    // Initialize pad for input handling
    PadState pad;
    padInitializeDefault(&pad);
    
    printf("Initializing USB Comms...\n");
    consoleUpdate(NULL);
    
    // Initialize USB communications
    Result rc = usbCommsInitialize();
    if (R_FAILED(rc)) {
        printf("Failed to initialize usbComms: 0x%x\n", rc);
        consoleUpdate(NULL);
        return false;
    }
    
    printf("USB Comms initialized!\n");
    printf("Waiting for PC connection...\n");
    consoleUpdate(NULL);
    
    // We can't easily check connection state with simple usbComms
    // So we just assume it's ready and let the first read/write handle waiting
    // But we'll give user a chance to cancel if it hangs
    
    m_connected = true;
    return true;
}

void USBClient::close() {
    if (m_connected) {
        usbCommsExit();
        m_connected = false;
    }
}

bool USBClient::sendCommand(uint32_t type, uint32_t cmdId, uint32_t length) {
    if (!m_connected) return false;
    
    ProtocolHeader header;
    header.magic = PROTOCOL_MAGIC_DBI;
    header.type = type;
    header.command = cmdId;
    header.length = length;
    
    // Retry sending header up to 3 times
    for (int i = 0; i < 3; i++) {
        size_t transferred = usbCommsWrite(&header, sizeof(header));
        if (transferred == sizeof(header)) {
            return true;
        }
        // Wait a bit before retry
        svcSleepThread(100000000ULL); // 100ms
    }
    
    printf("Failed to send command header\n");
    return false;
}

bool USBClient::receiveData(void* buffer, uint32_t length) {
    if (!m_connected) return false;
    
    uint32_t totalReceived = 0;
    uint8_t* buf = (uint8_t*)buffer;
    
    // Loop until we receive all requested bytes
    while (totalReceived < length) {
        size_t transferred = usbCommsRead(buf + totalReceived, length - totalReceived);
        
        if (transferred == 0) {
            printf("Failed to receive data: got %u/%u bytes (connection lost)\n", totalReceived, length);
            consoleUpdate(NULL);
            return false;
        }
        
        totalReceived += transferred;
    }
    
    return true;
}

bool USBClient::sendRawData(const void* data, uint32_t length) {
    if (!m_connected || !data || length == 0) return false;
    size_t transferred = usbCommsWrite(data, length);
    return transferred == length;
}

std::vector<FileInfo> USBClient::listFiles() {
    std::vector<FileInfo> files;
    
    if (!m_connected) {
        printf("Not connected!\n");
        return files;
    }
    
    printf("Sending LIST command...\n");
    consoleUpdate(NULL);
    
    // Flush any pending input
    // char tmp[64];
    // while (usbCommsRead(tmp, sizeof(tmp)) > 0);
    
    // Send LIST command
    if (!sendCommand(CMD_TYPE_REQUEST, (uint32_t)Command::LIST, 0)) {
        printf("Failed to send LIST command\n");
        consoleUpdate(NULL);
        return files;
    }
    
    printf("Waiting for response...\n");
    consoleUpdate(NULL);
    
    // Receive response header
    ProtocolHeader header;
    if (!receiveData(&header, sizeof(header))) {
        printf("Failed to receive response header\n");
        consoleUpdate(NULL);
        return files;
    }
    
    printf("Received header: magic=0x%x, cmd=%d, len=%u\n", 
           header.magic, header.command, header.length);
    
    if (header.magic != PROTOCOL_MAGIC_DBI && header.magic != PROTOCOL_MAGIC_SWLK) {
        printf("Invalid response header (magic: 0x%x)\n", header.magic);
        return files;
    }
    
    if (header.command != static_cast<uint32_t>(Command::LIST)) {
        printf("Invalid response command: %u\n", header.command);
        return files;
    }
    
    uint32_t list_len = header.length;
    printf("File list size: %u bytes\n", list_len);
    consoleUpdate(NULL);
    
    if (list_len == 0) {
        printf("No files available\n");
        consoleUpdate(NULL);
        return files;
    }
    
    // Send ACK
    printf("Sending ACK...\n");
    ProtocolHeader ack;
    ack.magic = PROTOCOL_MAGIC_DBI;
    ack.type = CMD_TYPE_ACK;
    ack.command = static_cast<uint32_t>(Command::LIST);
    ack.length = list_len;
    
    if (!sendRawData(&ack, sizeof(ack))) {
        printf("Failed to send ACK\n");
        return files;
    }
    
    // Receive file list data
    printf("Receiving file list data (%u bytes)...\n", list_len);
    consoleUpdate(NULL);
    
    // Use a large buffer to avoid alignment/size issues
    // Allocate 4KB aligned buffer
    static char listBuf[4096] __attribute__((aligned(4096)));
    memset(listBuf, 0, sizeof(listBuf));
    
    // We read exactly list_len bytes
    // Note: usbCommsRead might return more if the packet is larger, but we only care about list_len
    if (!receiveData(listBuf, list_len)) {
        printf("Failed to receive file list data\n");
        consoleUpdate(NULL);
        return files;
    }
    listBuf[list_len] = '\0';
    
    printf("Received data successfully!\n");
    consoleUpdate(NULL);
    
    // Parse file list (format: filename|size\n)
    std::string listStr(listBuf);
    
    size_t start = 0;
    size_t end = 0;
    
    while ((end = listStr.find('\n', start)) != std::string::npos) {
        std::string line = listStr.substr(start, end - start);
        if (!line.empty()) {
            FileInfo info;
            
            // Parse filename|size
            size_t pipePos = line.find('|');
            if (pipePos != std::string::npos) {
                info.filename = line.substr(0, pipePos);
                std::string sizeStr = line.substr(pipePos + 1);
                info.size = std::stoull(sizeStr);
            } else {
                // Backward compatibility: no size info
                info.filename = line;
                info.size = 0;
            }
            
            memset(info.sha256, 0, sizeof(info.sha256));
            files.push_back(info);
        }
        start = end + 1;
    }
    
    // Handle last line if no newline at end
    if (start < listStr.length()) {
        std::string line = listStr.substr(start);
        if (!line.empty()) {
            FileInfo info;
            
            // Parse filename|size
            size_t pipePos = line.find('|');
            if (pipePos != std::string::npos) {
                info.filename = line.substr(0, pipePos);
                std::string sizeStr = line.substr(pipePos + 1);
                info.size = std::stoull(sizeStr);
            } else {
                // Backward compatibility: no size info
                info.filename = line;
                info.size = 0;
            }
            
            memset(info.sha256, 0, sizeof(info.sha256));
            files.push_back(info);
        }
    }
    
    consoleUpdate(NULL);
    
    return files;
}

bool USBClient::downloadFile(const std::string& filename, const std::string& destPath, std::function<void(uint64_t, uint64_t)> progressCallback, uint64_t fileSize) {
    if (!m_connected) return false;
    
    FILE* fp = fopen(destPath.c_str(), "wb");
    if (!fp) {
        printf("Failed to open destination file: %s\n", destPath.c_str());
        return false;
    }
    
    // Use static buffer to avoid allocation overhead
    static uint8_t chunkBuffer[CHUNK_SIZE] __attribute__((aligned(4096)));
    
    uint64_t offset = 0;
    bool success = true;
    
    while (true) {
        // Calculate chunk size
        uint32_t requestSize = CHUNK_SIZE;
        if (fileSize > 0) {
            if (offset >= fileSize) break; // Done
            if (offset + requestSize > fileSize) {
                requestSize = (uint32_t)(fileSize - offset);
            }
        }
        
        // 1. Send FILE_RANGE command header
        uint32_t nameLen = filename.length();
        uint32_t requestBodySize = 4 + 8 + 4 + nameLen;
        
        if (!sendCommand(CMD_TYPE_REQUEST, (uint32_t)Command::GET_FILE, requestBodySize)) {
            printf("Failed to send FILE_RANGE command\n");
            success = false;
            break;
        }
        
        // 2. Wait for ACK
        ProtocolHeader ack;
        if (!receiveData(&ack, sizeof(ack))) {
            printf("Failed to receive ACK\n");
            success = false;
            break;
        }
        
        if (ack.magic != PROTOCOL_MAGIC_DBI && ack.magic != PROTOCOL_MAGIC_SWLK) {
            printf("Invalid ACK magic\n");
            success = false;
            break;
        }
        
        if (ack.type != CMD_TYPE_ACK || ack.command != (uint32_t)Command::GET_FILE) {
            printf("Unexpected response type: %u, cmd: %u\n", ack.type, ack.command);
            success = false;
            break;
        }
        
        // 3. Send FILE_RANGE request data
        std::vector<uint8_t> requestData(requestBodySize);
        uint32_t writePos = 0;
        
        // range_size (4 bytes)
        memcpy(&requestData[writePos], &requestSize, 4);
        writePos += 4;
        
        // range_offset (8 bytes)
        memcpy(&requestData[writePos], &offset, 8);
        writePos += 8;
        
        // nsp_name_len (4 bytes)
        memcpy(&requestData[writePos], &nameLen, 4);
        writePos += 4;
        
        // nsp_name
        memcpy(&requestData[writePos], filename.c_str(), nameLen);
        
        if (!sendRawData(requestData.data(), requestBodySize)) {
            printf("Failed to send request data\n");
            success = false;
            break;
        }
        
        // 4. Receive Response Header
        ProtocolHeader response;
        if (!receiveData(&response, sizeof(response))) {
            printf("Failed to receive response header\n");
            success = false;
            break;
        }
        
        if (response.type != CMD_TYPE_RESPONSE) {
            printf("Unexpected response type: %u\n", response.type);
            success = false;
            break;
        }
        
        // 5. Send ACK for the response
        if (!sendCommand(CMD_TYPE_ACK, (uint32_t)Command::GET_FILE, 0)) {
            printf("Failed to send ACK for response\n");
            success = false;
            break;
        }
        
        // 6. Receive File Data
        uint32_t expectedSize = requestSize;
        if (response.length != expectedSize) {
            expectedSize = response.length;
        }
        
        if (expectedSize == 0) break;
        
        if (!receiveData(chunkBuffer, expectedSize)) {
            printf("Failed to receive chunk data\n");
            success = false;
            break;
        }
        
        // Write to file
        size_t written = fwrite(chunkBuffer, 1, expectedSize, fp);
        if (written != expectedSize) {
            printf("Failed to write to file\n");
            success = false;
            break;
        }
        
        offset += written;
        
        if (progressCallback) {
            progressCallback(offset, fileSize);
            consoleUpdate(NULL);
        }
        
        if (fileSize == 0 && expectedSize < CHUNK_SIZE) {
            break;
        }
    }
    
    fclose(fp);
    
    if (!success) {
        remove(destPath.c_str());
    }
    
    return success;
}
