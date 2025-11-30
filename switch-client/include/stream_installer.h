#pragma once

#include <switch.h>
#include <cstdint>
#include <string>
#include <vector>
#include <functional>
#include <memory>
#include "pfs0.h"
#include "usb_client.h"

// Forward declaration
class ContentMeta;

// Progress callback: (bytesInstalled, totalBytes) -> continue?
using InstallProgressCallback = std::function<bool(uint64_t, uint64_t)>;

// Content info for installation
struct ContentInfo {
    NcmContentId contentId;
    uint64_t size;
    NcmContentType type;
    std::string filename;
};

// Streaming NSP Installer
// Installs NSP directly from USB without saving to SD card first
class StreamInstaller {
public:
    StreamInstaller(USBClient& client, NcmStorageId destStorage = NcmStorageId_SdCard);
    ~StreamInstaller();
    
    // Install NSP from USB
    // nspName: filename of NSP on PC
    // nspSize: total size of NSP file
    // progressCallback: called during installation
    bool install(const std::string& nspName, uint64_t nspSize, InstallProgressCallback progressCallback = nullptr);
    
    // Get last error message
    const std::string& getLastError() const { return m_lastError; }
    
private:
    // Initialize NCM services
    bool initializeServices();
    void closeServices();
    
    // Parse NSP structure
    bool parseNSP();
    
    // Install ticket and certificate
    bool installTicketCert();
    
    // Install NCA file
    bool installNCA(const ContentInfo& content);
    
    // Read CNMT (Content Meta) and get content list
    bool readCNMT();
    
    // Read CNMT from mounted filesystem
    bool readCNMTFromFS(FsFileSystem* fs);
    
    // Read CNMT directly from NSP (fallback)
    bool readCNMTFromNSP(const PFS0FileInfo* cnmtNcaFile);
    
    // Parse CNMT data
    bool parseCNMTData(const uint8_t* data, size_t size);
    
    // Register content meta with system
    bool registerContentMeta(const NcmContentInfo& cnmtContentInfo);
    
    // USB data read callback for PFS0
    bool readNSPData(uint64_t offset, uint64_t size, void* buffer);
    
    // Parse NCA ID from filename
    static bool parseNcaId(const std::string& filename, NcmContentId& outId);
    
    // Convert content ID to string
    static std::string contentIdToString(const NcmContentId& id);
    
    USBClient& m_client;
    NcmStorageId m_destStorage;
    std::string m_nspName;
    uint64_t m_nspSize;
    
    // PFS0 parser
    std::unique_ptr<PFS0> m_pfs0;
    
    // Parsed content meta
    std::unique_ptr<ContentMeta> m_parsedMeta;
    
    // Content list from CNMT
    std::vector<ContentInfo> m_contents;
    
    // NCM handles
    NcmContentStorage m_contentStorage;
    NcmContentMetaDatabase m_contentMetaDb;
    bool m_servicesInitialized;
    
    // Progress tracking
    InstallProgressCallback m_progressCallback;
    uint64_t m_totalInstallSize;
    uint64_t m_installedSize;
    
    // Error handling
    std::string m_lastError;
    
    // Buffer for USB reads
    static constexpr size_t READ_BUFFER_SIZE = 1024 * 1024; // 1MB
};
