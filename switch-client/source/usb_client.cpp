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
    

    
    // Initialize USB communications
    Result rc = usbCommsInitialize();
    if (R_FAILED(rc)) {

        return false;
    }
    

    
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

        return files;
    }
    

    
    // Flush any pending input
    // char tmp[64];
    // while (usbCommsRead(tmp, sizeof(tmp)) > 0);
    
    // Send LIST command
    if (!sendCommand(CMD_TYPE_REQUEST, (uint32_t)Command::LIST, 0)) {

        return files;
    }
    

    
    // Receive response header
    ProtocolHeader header;
    if (!receiveData(&header, sizeof(header))) {

        return files;
    }
    

    
    if (header.magic != PROTOCOL_MAGIC_DBI && header.magic != PROTOCOL_MAGIC_SWLK) {

        return files;
    }
    
    if (header.command != static_cast<uint32_t>(Command::LIST)) {

        return files;
    }
    
    uint32_t list_len = header.length;

    
    if (list_len == 0) {

        return files;
    }
    
    // Send ACK

    ProtocolHeader ack;
    ack.magic = PROTOCOL_MAGIC_DBI;
    ack.type = CMD_TYPE_ACK;
    ack.command = static_cast<uint32_t>(Command::LIST);
    ack.length = list_len;
    
    if (!sendRawData(&ack, sizeof(ack))) {

        return files;
    }
    
    // Receive file list data

    
    // Use a large buffer to avoid alignment/size issues
    // Allocate 4KB aligned buffer
    static char listBuf[4096] __attribute__((aligned(4096)));
    memset(listBuf, 0, sizeof(listBuf));
    
    // We read exactly list_len bytes
    // Note: usbCommsRead might return more if the packet is larger, but we only care about list_len
    if (!receiveData(listBuf, list_len)) {

        return files;
    }
    listBuf[list_len] = '\0';
    

    
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
    

    
    return files;
}

bool USBClient::downloadFile(const std::string& filename, const std::string& destPath, std::function<bool(uint64_t, uint64_t)> progressCallback, uint64_t fileSize) {
    if (!m_connected) return false;
    
    FILE* fp = fopen(destPath.c_str(), "wb");
    if (!fp) {

        return false;
    }
    
    // Use static buffer to avoid allocation overhead
    static uint8_t chunkBuffer[CHUNK_SIZE] __attribute__((aligned(4096)));
    
    uint64_t offset = 0;
    bool success = true;
    bool cancelled = false;
    
    while (true) {
        // Check if we've received all data
        if (fileSize > 0 && offset >= fileSize) {
            // Transfer complete
            break;
        }
        
        // Calculate chunk size
        uint32_t requestSize = CHUNK_SIZE;
        if (fileSize > 0 && offset + requestSize > fileSize) {
            requestSize = (uint32_t)(fileSize - offset);
        }
        
        // 1. Send FILE_RANGE command header
        uint32_t nameLen = filename.length();
        uint32_t requestBodySize = 4 + 8 + 4 + nameLen;
        
        if (!sendCommand(CMD_TYPE_REQUEST, (uint32_t)Command::GET_FILE, requestBodySize)) {

            success = false;
            break;
        }
        
        // 2. Wait for ACK
        ProtocolHeader ack;
        if (!receiveData(&ack, sizeof(ack))) {

            success = false;
            break;
        }
        
        if (ack.magic != PROTOCOL_MAGIC_DBI && ack.magic != PROTOCOL_MAGIC_SWLK) {

            success = false;
            break;
        }
        
        if (ack.type != CMD_TYPE_ACK || ack.command != (uint32_t)Command::GET_FILE) {

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

            success = false;
            break;
        }
        
        // 4. Receive Response Header
        ProtocolHeader response;
        if (!receiveData(&response, sizeof(response))) {

            success = false;
            break;
        }
        
        if (response.type != CMD_TYPE_RESPONSE) {

            success = false;
            break;
        }
        
        // 6. Receive File Data
        // Server now sends actual size in response.length (may be less than requested)
        uint32_t expectedSize = response.length;
        
        // 5. Send ACK for the response (must send even if expectedSize is 0)
        if (!sendCommand(CMD_TYPE_ACK, (uint32_t)Command::GET_FILE, 0)) {

            success = false;
            break;
        }
        
        // Check if transfer is complete (server sent 0 bytes)
        if (expectedSize == 0) {
            // Transfer complete - no more data from server
            break;
        }
        
        if (!receiveData(chunkBuffer, expectedSize)) {

            success = false;
            break;
        }
        
        // Write to file
        size_t written = fwrite(chunkBuffer, 1, expectedSize, fp);
        if (written != expectedSize) {

            success = false;
            break;
        }
        
        offset += written;
        
        // Call progress callback - returns false if user wants to cancel
        if (progressCallback) {
            if (!progressCallback(offset, fileSize)) {
                cancelled = true;
                break;
            }

        }
        
        if (fileSize == 0 && expectedSize < CHUNK_SIZE) {
            break;
        }
    }
    
    fclose(fp);
    
    // Delete incomplete file on failure or cancel
    if (!success || cancelled) {
        remove(destPath.c_str());
    }
    
    return success && !cancelled;
}
