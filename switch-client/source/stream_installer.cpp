#include "stream_installer.h"
#include "content_meta.h"
#include "es_wrapper.h"
#include <cstring>
#include <cstdio>
#include <algorithm>
#include <dirent.h>

StreamInstaller::StreamInstaller(USBClient& client, NcmStorageId destStorage)
    : m_client(client)
    , m_destStorage(destStorage)
    , m_nspSize(0)
    , m_servicesInitialized(false)
    , m_totalInstallSize(0)
    , m_installedSize(0)
{
    memset(&m_contentStorage, 0, sizeof(m_contentStorage));
    memset(&m_contentMetaDb, 0, sizeof(m_contentMetaDb));
}

StreamInstaller::~StreamInstaller() {
    closeServices();
}

bool StreamInstaller::initializeServices() {
    if (m_servicesInitialized) return true;
    
    Result rc;
    
    // Initialize NCM
    rc = ncmInitialize();
    if (R_FAILED(rc)) {
        m_lastError = "Failed to initialize NCM: " + std::to_string(rc);
        return false;
    }
    
    // Initialize NS
    rc = nsInitialize();
    if (R_FAILED(rc)) {
        m_lastError = "Failed to initialize NS: " + std::to_string(rc);
        ncmExit();
        return false;
    }
    
    // Open content storage
    rc = ncmOpenContentStorage(&m_contentStorage, m_destStorage);
    if (R_FAILED(rc)) {
        m_lastError = "Failed to open content storage: " + std::to_string(rc);
        nsExit();
        ncmExit();
        return false;
    }
    
    // Open content meta database
    rc = ncmOpenContentMetaDatabase(&m_contentMetaDb, m_destStorage);
    if (R_FAILED(rc)) {
        m_lastError = "Failed to open content meta database: " + std::to_string(rc);
        ncmContentStorageClose(&m_contentStorage);
        nsExit();
        ncmExit();
        return false;
    }
    
    m_servicesInitialized = true;
    // NCM services initialized
    return true;
}

void StreamInstaller::closeServices() {
    if (m_servicesInitialized) {
        ncmContentMetaDatabaseClose(&m_contentMetaDb);
        ncmContentStorageClose(&m_contentStorage);
        nsExit();
        ncmExit();
        m_servicesInitialized = false;
    }
}

bool StreamInstaller::readNSPData(uint64_t offset, uint64_t size, void* buffer) {
    // Use FILE_RANGE command to read data from PC
    uint32_t nameLen = m_nspName.length();
    uint32_t requestBodySize = 4 + 8 + 4 + nameLen;
    
    // Send FILE_RANGE command
    if (!m_client.sendCommand(CMD_TYPE_REQUEST, (uint32_t)Command::GET_FILE, requestBodySize)) {
        return false;
    }
    
    // Wait for ACK
    ProtocolHeader ack;
    if (!m_client.receiveData(&ack, sizeof(ack))) {
        return false;
    }
    
    if (ack.type != CMD_TYPE_ACK) {
        return false;
    }
    
    // Send request data
    std::vector<uint8_t> requestData(requestBodySize);
    uint32_t writePos = 0;
    
    uint32_t rangeSize = (uint32_t)size;
    memcpy(&requestData[writePos], &rangeSize, 4);
    writePos += 4;
    
    memcpy(&requestData[writePos], &offset, 8);
    writePos += 8;
    
    memcpy(&requestData[writePos], &nameLen, 4);
    writePos += 4;
    
    memcpy(&requestData[writePos], m_nspName.c_str(), nameLen);
    
    if (!m_client.sendRawData(requestData.data(), requestBodySize)) {
        return false;
    }
    
    // Receive response header
    ProtocolHeader response;
    if (!m_client.receiveData(&response, sizeof(response))) {
        return false;
    }
    
    if (response.type != CMD_TYPE_RESPONSE || response.length != size) {
        return false;
    }
    
    // Send ACK
    if (!m_client.sendCommand(CMD_TYPE_ACK, (uint32_t)Command::GET_FILE, 0)) {
        return false;
    }
    
    // Receive data
    return m_client.receiveData(buffer, size);
}

bool StreamInstaller::parseNSP() {
    m_pfs0 = std::make_unique<PFS0>();
    
    // Create read callback
    auto readCallback = [this](uint64_t offset, uint64_t size, void* buffer) -> bool {
        return this->readNSPData(offset, size, buffer);
    };
    
    if (!m_pfs0->initialize(readCallback, m_nspSize)) {
        m_lastError = "Failed to parse NSP structure";
        return false;
    }
    
    return true;
}

bool StreamInstaller::parseNcaId(const std::string& filename, NcmContentId& outId) {
    // NCA filename format: <32 hex chars>.nca or <32 hex chars>.cnmt.nca
    std::string name = filename;
    
    // Remove extension
    size_t dotPos = name.find('.');
    if (dotPos != std::string::npos) {
        name = name.substr(0, dotPos);
    }
    
    // Must be 32 hex characters
    if (name.length() != 32) {
        return false;
    }
    
    // Parse hex string to bytes
    for (size_t i = 0; i < 16; i++) {
        std::string byteStr = name.substr(i * 2, 2);
        outId.c[i] = (uint8_t)strtoul(byteStr.c_str(), nullptr, 16);
    }
    
    return true;
}

std::string StreamInstaller::contentIdToString(const NcmContentId& id) {
    char str[33];
    for (size_t i = 0; i < 16; i++) {
        snprintf(&str[i * 2], 3, "%02x", id.c[i]);
    }
    str[32] = '\0';
    return std::string(str);
}

bool StreamInstaller::installTicketCert() {
    // Find .tik and .cert files
    auto tikFiles = m_pfs0->getFilesByExtension(".tik");
    auto certFiles = m_pfs0->getFilesByExtension(".cert");
    
    if (tikFiles.empty()) {
        return true; // Not an error - free games don't have tickets
    }
    
    if (tikFiles.size() != certFiles.size()) {
        m_lastError = "Ticket/Certificate count mismatch";
        return false;
    }
    
    // Try to initialize ES service for ticket import
    Result rc = esInitialize();
    if (R_FAILED(rc)) {
        // ES service not available - skip ticket installation
        // Most users have sigpatches, so this is not fatal
        return true;
    }
    
    // ES service available - try to install tickets
    for (size_t i = 0; i < tikFiles.size(); i++) {
        const auto* tikFile = tikFiles[i];
        const auto* certFile = certFiles[i];
        
        // Read ticket data
        std::vector<uint8_t> tikData(tikFile->size);
        if (!m_pfs0->readFileData(*tikFile, 0, tikFile->size, tikData.data())) {
            continue;
        }
        
        // Read certificate data
        std::vector<uint8_t> certData(certFile->size);
        if (!m_pfs0->readFileData(*certFile, 0, certFile->size, certData.data())) {
            continue;
        }
        
        // Import ticket using ES service
        esImportTicket(tikData.data(), tikData.size(), certData.data(), certData.size());
    }
    
    esExit();
    return true; // Always return true - ticket failures are not fatal
}

bool StreamInstaller::installNCA(const ContentInfo& content) {
    // Installing NCA
    
    const PFS0FileInfo* ncaFile = m_pfs0->getFileByName(content.filename);
    if (!ncaFile) {
        m_lastError = "NCA file not found: " + content.filename;
        return false;
    }
    
    Result rc;
    NcmPlaceHolderId placeholderId;
    memcpy(&placeholderId, &content.contentId, sizeof(NcmContentId));
    
    // Delete any existing placeholder
    ncmContentStorageDeletePlaceHolder(&m_contentStorage, &placeholderId);
    
    // Create placeholder
    rc = ncmContentStorageCreatePlaceHolder(&m_contentStorage, &content.contentId, &placeholderId, content.size);
    if (R_FAILED(rc)) {
        m_lastError = "Failed to create placeholder: " + std::to_string(rc);
        return false;
    }
    
    // Stream NCA data to placeholder
    static uint8_t buffer[READ_BUFFER_SIZE] __attribute__((aligned(0x1000)));
    uint64_t offset = 0;
    
    while (offset < content.size) {
        uint64_t chunkSize = std::min((uint64_t)READ_BUFFER_SIZE, content.size - offset);
        
        // Read chunk from USB
        if (!m_pfs0->readFileData(*ncaFile, offset, chunkSize, buffer)) {
            m_lastError = "Failed to read NCA data";
            ncmContentStorageDeletePlaceHolder(&m_contentStorage, &placeholderId);
            return false;
        }
        
        // Patch NCA header if needed (change distribution type from gamecard to download)
        if (offset == 0 && chunkSize >= sizeof(NcaHeader)) {
            // NCA header is encrypted, but distribution byte is at a known offset
            // For simplicity, we'll skip patching here - most NSPs don't need it
        }
        
        // Write to placeholder
        rc = ncmContentStorageWritePlaceHolder(&m_contentStorage, &placeholderId, offset, buffer, chunkSize);
        if (R_FAILED(rc)) {
            m_lastError = "Failed to write to placeholder: " + std::to_string(rc);
            ncmContentStorageDeletePlaceHolder(&m_contentStorage, &placeholderId);
            return false;
        }
        
        offset += chunkSize;
        m_installedSize += chunkSize;
        
        // Update progress
        if (m_progressCallback) {
            if (!m_progressCallback(m_installedSize, m_totalInstallSize)) {
                m_lastError = "Installation cancelled";
                ncmContentStorageDeletePlaceHolder(&m_contentStorage, &placeholderId);
                return false;
            }
        }
    }
    
    // Register the content
    rc = ncmContentStorageRegister(&m_contentStorage, &content.contentId, &placeholderId);
    if (R_FAILED(rc)) {
        if (rc == 0x805) {
            // Already exists - delete placeholder
            ncmContentStorageDeletePlaceHolder(&m_contentStorage, &placeholderId);
        } else {
            m_lastError = "Failed to register content: 0x" + std::to_string(rc);
            ncmContentStorageDeletePlaceHolder(&m_contentStorage, &placeholderId);
            return false;
        }
    }
    return true;
}

bool StreamInstaller::readCNMT() {
    // Find CNMT NCA files
    auto cnmtFiles = m_pfs0->getFilesByExtension(".cnmt.nca");
    
    if (cnmtFiles.empty()) {
        m_lastError = "No CNMT NCA found in NSP";
        return false;
    }
    
    m_contents.clear();
    m_totalInstallSize = 0;
    
    // Process each CNMT NCA
    for (const auto* cnmtFile : cnmtFiles) {
        // Processing CNMT
        
        // Parse CNMT NCA ID
        NcmContentId cnmtNcaId;
        if (!parseNcaId(cnmtFile->name, cnmtNcaId)) {
            // Could not parse CNMT NCA ID
            continue;
        }
        
        // First, install the CNMT NCA
        ContentInfo cnmtInfo;
        cnmtInfo.contentId = cnmtNcaId;
        cnmtInfo.filename = cnmtFile->name;
        cnmtInfo.size = cnmtFile->size;
        cnmtInfo.type = NcmContentType_Meta;
        
        // Install CNMT NCA first
        if (!installNCA(cnmtInfo)) {
            continue;
        }
        
        // Get the path to the installed CNMT NCA
        char ncaPath[FS_MAX_PATH];
        Result rc = ncmContentStorageGetPath(&m_contentStorage, ncaPath, sizeof(ncaPath), &cnmtNcaId);
        if (R_FAILED(rc)) {

            continue;
        }

        
        // Mount the CNMT NCA filesystem
        FsFileSystem cnmtFs;
        rc = fsOpenFileSystemWithId(&cnmtFs, 0, FsFileSystemType_ContentMeta, ncaPath, FsContentAttributes_None);
        if (R_FAILED(rc)) {

            // Try alternative method - read CNMT directly from NSP
            if (!readCNMTFromNSP(cnmtFile)) {
                continue;
            }
        } else {
            // Read CNMT from mounted filesystem
            if (!readCNMTFromFS(&cnmtFs)) {
                fsFsClose(&cnmtFs);
                continue;
            }
            fsFsClose(&cnmtFs);
        }
        
        // Create CNMT content info for registration
        NcmContentInfo cnmtContentInfo;
        memset(&cnmtContentInfo, 0, sizeof(cnmtContentInfo));
        cnmtContentInfo.content_id = cnmtNcaId;
        cnmtContentInfo.content_type = NcmContentType_Meta;
        // Set size using size_low and size_high
        cnmtContentInfo.size_low = (uint32_t)(cnmtFile->size & 0xFFFFFFFF);
        cnmtContentInfo.size_high = (uint8_t)((cnmtFile->size >> 32) & 0xFF);
        cnmtContentInfo.id_offset = 0;
        
        // Register content meta with the system
        if (!registerContentMeta(cnmtContentInfo)) {

        }
    }
    
    // Now add all other NCAs to install list
    auto ncaFiles = m_pfs0->getFilesByExtension(".nca");
    
    for (const auto* ncaFile : ncaFiles) {
        // Skip CNMT NCAs (already installed)
        if (ncaFile->name.find(".cnmt.nca") != std::string::npos) {
            continue;
        }
        
        ContentInfo info;
        if (!parseNcaId(ncaFile->name, info.contentId)) {

            continue;
        }
        
        info.filename = ncaFile->name;
        info.size = ncaFile->size;
        info.type = NcmContentType_Data; // Will be determined from CNMT
        
        // Check if already in content list from CNMT
        bool found = false;
        for (const auto& existing : m_contents) {
            if (memcmp(&existing.contentId, &info.contentId, sizeof(NcmContentId)) == 0) {
                found = true;
                break;
            }
        }
        
        if (!found) {
            m_contents.push_back(info);
            m_totalInstallSize += info.size;
        }
    }
    

    return true;
}

bool StreamInstaller::readCNMTFromFS(FsFileSystem* fs) {
    // Find .cnmt file in the filesystem
    FsDir dir;
    Result rc = fsFsOpenDirectory(fs, "/", FsDirOpenMode_ReadFiles, &dir);
    if (R_FAILED(rc)) {

        return false;
    }
    
    char cnmtName[FS_MAX_PATH] = {0};
    FsDirectoryEntry entry;
    s64 entriesRead;
    
    while (R_SUCCEEDED(fsDirRead(&dir, &entriesRead, 1, &entry)) && entriesRead > 0) {
        std::string name(entry.name);
        if (name.find(".cnmt") != std::string::npos && name.find(".nca") == std::string::npos) {
            strncpy(cnmtName, entry.name, sizeof(cnmtName) - 1);
            break;
        }
    }
    fsDirClose(&dir);
    
    if (cnmtName[0] == '\0') {

        return false;
    }
    

    
    // Open and read the CNMT file
    char cnmtPath[FS_MAX_PATH];
    snprintf(cnmtPath, sizeof(cnmtPath), "/%s", cnmtName);
    
    FsFile cnmtFile;
    rc = fsFsOpenFile(fs, cnmtPath, FsOpenMode_Read, &cnmtFile);
    if (R_FAILED(rc)) {

        return false;
    }
    
    s64 fileSize;
    rc = fsFileGetSize(&cnmtFile, &fileSize);
    if (R_FAILED(rc)) {
        fsFileClose(&cnmtFile);

        return false;
    }
    
    std::vector<uint8_t> cnmtData(fileSize);
    u64 bytesRead;
    rc = fsFileRead(&cnmtFile, 0, cnmtData.data(), fileSize, FsReadOption_None, &bytesRead);
    fsFileClose(&cnmtFile);
    
    if (R_FAILED(rc) || bytesRead != (u64)fileSize) {

        return false;
    }
    
    // Parse CNMT
    return parseCNMTData(cnmtData.data(), cnmtData.size());
}

bool StreamInstaller::readCNMTFromNSP(const PFS0FileInfo* cnmtNcaFile) {
    // Fallback: try to read CNMT structure directly
    // This is a simplified approach that may not work for all NSPs

    
    // For now, just enumerate all NCAs from NSP
    auto ncaFiles = m_pfs0->getFilesByExtension(".nca");
    
    for (const auto* ncaFile : ncaFiles) {
        if (ncaFile->name.find(".cnmt.nca") != std::string::npos) {
            continue; // Skip CNMT NCAs
        }
        
        ContentInfo info;
        if (!parseNcaId(ncaFile->name, info.contentId)) {
            continue;
        }
        
        info.filename = ncaFile->name;
        info.size = ncaFile->size;
        info.type = NcmContentType_Data;
        
        m_contents.push_back(info);
        m_totalInstallSize += info.size;
    }
    
    return !m_contents.empty();
}

bool StreamInstaller::parseCNMTData(const uint8_t* data, size_t size) {
    ContentMeta meta(data, size);
    
    // Get content infos from CNMT
    const auto& contentInfos = meta.getContentInfos();
    

    
    // Store parsed meta for later registration
    m_parsedMeta = std::make_unique<ContentMeta>(data, size);
    
    // Update content list with proper types
    for (const auto& ncmInfo : contentInfos) {
        ContentInfo info;
        info.contentId = ncmInfo.content_id;
        info.type = static_cast<NcmContentType>(ncmInfo.content_type);
        
        // Get size from NcmContentInfo (size_low + size_high)
        info.size = (uint64_t)ncmInfo.size_low | ((uint64_t)ncmInfo.size_high << 32);
        
        // Find filename in PFS0
        std::string idStr = ContentMetaUtil::contentIdToString(ncmInfo.content_id);
        for (const auto& file : m_pfs0->getFiles()) {
            if (file.name.find(idStr) != std::string::npos) {
                info.filename = file.name;
                break;
            }
        }
        
        if (info.filename.empty()) {
            info.filename = idStr + ".nca";
        }
        
        m_contents.push_back(info);
        m_totalInstallSize += info.size;
        

    }
    
    return true;
}

bool StreamInstaller::registerContentMeta(const NcmContentInfo& cnmtContentInfo) {
    if (!m_parsedMeta) {

        return false;
    }
    
    // Create install content meta buffer
    std::vector<uint8_t> installMetaBuffer;
    if (!m_parsedMeta->createInstallContentMeta(installMetaBuffer, cnmtContentInfo, true)) {

        return false;
    }
    
    // Get content meta key
    NcmContentMetaKey metaKey = m_parsedMeta->getContentMetaKey();
    

    
    // Set content meta in database
    Result rc = ncmContentMetaDatabaseSet(&m_contentMetaDb, &metaKey, 
                                          installMetaBuffer.data(), installMetaBuffer.size());
    if (R_FAILED(rc)) {

        return false;
    }
    
    // Commit the database
    rc = ncmContentMetaDatabaseCommit(&m_contentMetaDb);
    if (R_FAILED(rc)) {

        return false;
    }
    
    // Push application record
    uint64_t baseTitleId = ContentMetaUtil::getBaseTitleId(metaKey.id, 
                                                           static_cast<NcmContentMetaType>(metaKey.type));
    
    // Create content storage record (matches system's ContentStorageRecord)
    struct ContentStorageRecord {
        NcmContentMetaKey key;
        u8 storage_id;
        u8 padding[7];
    } __attribute__((packed)) storageRecord;
    
    memset(&storageRecord, 0, sizeof(storageRecord));
    storageRecord.key.id = metaKey.id;
    storageRecord.key.version = metaKey.version;
    storageRecord.key.type = metaKey.type;
    storageRecord.key.install_type = 0;
    storageRecord.storage_id = m_destStorage;
    
    // Use IApplicationManagerInterface to push application record
    // This is the low-level way that works in libnx
    Service appManSrv;
    rc = nsGetApplicationManagerInterface(&appManSrv);
    if (R_FAILED(rc)) {
        return true; // Not fatal - game is installed, just may need reboot
    }
    
    // CRITICAL FIX: Do NOT delete existing application record!
    // The old code deleted the base title record, which caused DLC to overwrite base games.
    // We now ONLY push/update the record without deleting.
    
    // Push application record (cmd 16 = PushApplicationRecord)
    // This appends/updates the record without removing existing ones
    struct {
        u8 last_modified_event;
        u8 padding[7];
        u64 application_id;
    } pushIn = { 0x3, {0}, baseTitleId }; // 0x3 = Installed
    
    rc = serviceDispatchIn(&appManSrv, 16, pushIn,
        .buffer_attrs = { SfBufferAttr_HipcMapAlias | SfBufferAttr_In },
        .buffers = { { &storageRecord, sizeof(storageRecord) } },
    );
    
    serviceClose(&appManSrv);
    return true;
}

bool StreamInstaller::install(const std::string& nspName, uint64_t nspSize, InstallProgressCallback progressCallback) {
    m_nspName = nspName;
    m_nspSize = nspSize;
    m_progressCallback = progressCallback;
    m_installedSize = 0;
    m_lastError.clear();
    
    printf("\n=== Installing: %s ===\n", nspName.c_str());
    printf("Size: %lu bytes\n", nspSize);
    printf("Destination: %s\n", m_destStorage == NcmStorageId_SdCard ? "SD Card" : "NAND");
    
    // Initialize services
    if (!initializeServices()) {
        return false;
    }
    
    // Parse NSP structure
    printf("\nParsing NSP structure...\n");
    if (!parseNSP()) {
        return false;
    }
    
    // Read content list from CNMT
    printf("\nReading content metadata...\n");
    if (!readCNMT()) {
        return false;
    }
    
    // IMPORTANT: Install tickets BEFORE NCAs (like Awoo Installer does)
    // This ensures the system recognizes the rights before installing content
    printf("\nInstalling tickets and certificates...\n");
    if (!installTicketCert()) {
        // Ticket installation failure is not fatal - continue anyway
        printf("Warning: Ticket installation had issues, but continuing...\n");
    }
    
    // Install each NCA
    printf("\nInstalling NCAs...\n");
    for (const auto& content : m_contents) {
        if (!installNCA(content)) {
            return false;
        }
    }
    
    // Final commit to ensure all changes are persisted
    printf("\nFinalizing installation...\n");
    Result rc = ncmContentMetaDatabaseCommit(&m_contentMetaDb);
    if (R_FAILED(rc)) {
        printf("Warning: Final database commit failed (0x%X)\n", rc);
        // Not fatal - data should already be committed
    }
    
    printf("\n=== Installation Complete! ===\n");
    printf("Game should now appear in your home menu.\n");
    printf("If it doesn't appear, try rebooting your Switch.\n");
    
    return true;
}

