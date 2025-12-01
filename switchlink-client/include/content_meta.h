#pragma once

#include <switch.h>
#include <cstdint>
#include <vector>
#include <string>
#include <cstring>

// Content Meta structures for NSP installation
// Based on Awoo-Installer's implementation

#pragma pack(push, 1)

// Packaged Content Meta Header (from NSP's CNMT file)
struct PackagedContentMetaHeader {
    uint64_t titleId;
    uint32_t version;
    uint8_t type;               // NcmContentMetaType
    uint8_t _0xd;
    uint16_t extendedHeaderSize;
    uint16_t contentCount;
    uint16_t contentMetaCount;
    uint8_t attributes;
    uint8_t storageId;
    uint8_t installType;
    uint8_t committed;
    uint32_t requiredSystemVersion;
    uint32_t _0x1c;
};
static_assert(sizeof(PackagedContentMetaHeader) == 0x20, "PackagedContentMetaHeader must be 0x20!");

// Use libnx's NcmPackagedContentInfo instead of custom struct
// struct PackagedContentInfo is already defined in ncm_types.h as NcmPackagedContentInfo

// Application extended header
struct ApplicationMetaExtendedHeader {
    uint64_t patchTitleId;
    uint32_t requiredSystemVersion;
    uint32_t requiredApplicationVersion;
};

// Patch extended header
struct PatchMetaExtendedHeader {
    uint64_t applicationTitleId;
    uint32_t requiredSystemVersion;
    uint32_t extendedDataSize;
    uint8_t reserved[0x8];
};

// AddOnContent extended header
struct AddOnContentMetaExtendedHeader {
    uint64_t applicationTitleId;
    uint32_t requiredApplicationVersion;
    uint8_t reserved[0x4];
};

#pragma pack(pop)

// Content Meta class for parsing and creating install meta
class ContentMeta {
public:
    ContentMeta();
    ContentMeta(const uint8_t* data, size_t size);
    
    // Parse from raw CNMT data
    bool parse(const uint8_t* data, size_t size);
    
    // Get header info
    const PackagedContentMetaHeader& getHeader() const { return m_header; }
    uint64_t getTitleId() const { return m_header.titleId; }
    uint32_t getVersion() const { return m_header.version; }
    NcmContentMetaType getType() const { return static_cast<NcmContentMetaType>(m_header.type); }
    
    // Get content infos (NCAs to install)
    const std::vector<NcmContentInfo>& getContentInfos() const { return m_contentInfos; }
    
    // Get NcmContentMetaKey for registration
    NcmContentMetaKey getContentMetaKey() const;
    
    // Create install content meta buffer for ncmContentMetaDatabaseSet
    bool createInstallContentMeta(std::vector<uint8_t>& outBuffer, 
                                   const NcmContentInfo& cnmtContentInfo,
                                   bool ignoreReqFirmVersion = true);
    
private:
    PackagedContentMetaHeader m_header;
    std::vector<uint8_t> m_extendedHeader;
    std::vector<NcmContentInfo> m_contentInfos;
    std::vector<uint8_t> m_rawData;
    bool m_parsed;
};

// Utility functions
namespace ContentMetaUtil {
    // Get base title ID (for patches/DLC)
    uint64_t getBaseTitleId(uint64_t titleId, NcmContentMetaType type);
    
    // Convert NcmContentId to hex string
    std::string contentIdToString(const NcmContentId& id);
    
    // Parse NcmContentId from hex string
    bool parseContentId(const std::string& str, NcmContentId& outId);
}
