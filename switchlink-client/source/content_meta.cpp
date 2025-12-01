#include "content_meta.h"
#include <cstdio>
#include <algorithm>

ContentMeta::ContentMeta() : m_parsed(false) {
    memset(&m_header, 0, sizeof(m_header));
}

ContentMeta::ContentMeta(const uint8_t* data, size_t size) : m_parsed(false) {
    memset(&m_header, 0, sizeof(m_header));
    parse(data, size);
}

bool ContentMeta::parse(const uint8_t* data, size_t size) {
    m_parsed = false;
    m_contentInfos.clear();
    m_extendedHeader.clear();
    m_rawData.clear();
    
    if (size < sizeof(PackagedContentMetaHeader)) {
        return false;
    }
    
    // Store raw data
    m_rawData.assign(data, data + size);
    
    // Read header
    memcpy(&m_header, data, sizeof(PackagedContentMetaHeader));
    
    // CNMT parsed successfully
    
    // Read extended header
    size_t offset = sizeof(PackagedContentMetaHeader);
    if (m_header.extendedHeaderSize > 0) {
        if (offset + m_header.extendedHeaderSize > size) {
            return false;
        }
        m_extendedHeader.assign(data + offset, data + offset + m_header.extendedHeaderSize);
        offset += m_header.extendedHeaderSize;
    }
    
    // Read content infos
    for (uint16_t i = 0; i < m_header.contentCount; i++) {
        if (offset + sizeof(NcmPackagedContentInfo) > size) {
            return false;
        }
        
        NcmPackagedContentInfo packagedInfo;
        memcpy(&packagedInfo, data + offset, sizeof(NcmPackagedContentInfo));
        offset += sizeof(NcmPackagedContentInfo);
        
        // Only include valid content types (0-5)
        uint8_t contentType = static_cast<uint8_t>(packagedInfo.info.content_type);
        if (contentType <= 5) {
            m_contentInfos.push_back(packagedInfo.info);
            // Content parsed
        }
    }
    
    m_parsed = true;
    return true;
}

NcmContentMetaKey ContentMeta::getContentMetaKey() const {
    NcmContentMetaKey key;
    memset(&key, 0, sizeof(key));
    
    key.id = m_header.titleId;
    key.version = m_header.version;
    key.type = static_cast<NcmContentMetaType>(m_header.type);
    
    return key;
}

bool ContentMeta::createInstallContentMeta(std::vector<uint8_t>& outBuffer,
                                            const NcmContentInfo& cnmtContentInfo,
                                            bool ignoreReqFirmVersion) {
    if (!m_parsed) {
        return false;
    }
    
    outBuffer.clear();
    
    // Create NcmContentMetaHeader
    NcmContentMetaHeader installHeader;
    memset(&installHeader, 0, sizeof(installHeader));
    installHeader.extended_header_size = m_header.extendedHeaderSize;
    installHeader.content_count = m_contentInfos.size() + 1; // +1 for CNMT itself
    installHeader.content_meta_count = m_header.contentMetaCount;
    installHeader.attributes = m_header.attributes;
    installHeader.storage_id = 0;
    
    // Append header
    size_t headerOffset = outBuffer.size();
    outBuffer.resize(outBuffer.size() + sizeof(NcmContentMetaHeader));
    memcpy(outBuffer.data() + headerOffset, &installHeader, sizeof(NcmContentMetaHeader));
    
    // Append extended header
    if (!m_extendedHeader.empty()) {
        size_t extOffset = outBuffer.size();
        outBuffer.resize(outBuffer.size() + m_extendedHeader.size());
        memcpy(outBuffer.data() + extOffset, m_extendedHeader.data(), m_extendedHeader.size());
        
        // Optionally zero out required firmware version
        if (ignoreReqFirmVersion && 
            (m_header.type == NcmContentMetaType_Application || 
             m_header.type == NcmContentMetaType_Patch)) {
            // Required system version is at offset 8 in extended header
            if (m_extendedHeader.size() >= 12) {
                uint32_t* reqVer = (uint32_t*)(outBuffer.data() + sizeof(NcmContentMetaHeader) + 8);
                *reqVer = 0;
            }
        }
    }
    
    // Append CNMT content info first
    size_t cnmtOffset = outBuffer.size();
    outBuffer.resize(outBuffer.size() + sizeof(NcmContentInfo));
    memcpy(outBuffer.data() + cnmtOffset, &cnmtContentInfo, sizeof(NcmContentInfo));
    
    // Append other content infos
    for (const auto& info : m_contentInfos) {
        size_t infoOffset = outBuffer.size();
        outBuffer.resize(outBuffer.size() + sizeof(NcmContentInfo));
        memcpy(outBuffer.data() + infoOffset, &info, sizeof(NcmContentInfo));
    }
    
    // For patches, append extended data
    if (m_header.type == NcmContentMetaType_Patch && m_extendedHeader.size() >= sizeof(PatchMetaExtendedHeader)) {
        PatchMetaExtendedHeader* patchHeader = (PatchMetaExtendedHeader*)m_extendedHeader.data();
        if (patchHeader->extendedDataSize > 0) {
            // Extended data follows content meta entries in raw data
            size_t extDataOffset = sizeof(PackagedContentMetaHeader) + 
                                   m_header.extendedHeaderSize + 
                                   (m_header.contentCount * sizeof(NcmPackagedContentInfo)) +
                                   (m_header.contentMetaCount * sizeof(NcmContentMetaKey));
            
            if (extDataOffset + patchHeader->extendedDataSize <= m_rawData.size()) {
                size_t appendOffset = outBuffer.size();
                outBuffer.resize(outBuffer.size() + patchHeader->extendedDataSize);
                memcpy(outBuffer.data() + appendOffset, 
                       m_rawData.data() + extDataOffset, 
                       patchHeader->extendedDataSize);
            }
        }
    }
    
    // Install content meta created
    return true;
}

namespace ContentMetaUtil {

uint64_t getBaseTitleId(uint64_t titleId, NcmContentMetaType type) {
    switch (type) {
        case NcmContentMetaType_Patch:
            return titleId ^ 0x800;
        case NcmContentMetaType_AddOnContent:
            return (titleId ^ 0x1000) & ~0xFFFULL;
        default:
            return titleId;
    }
}

std::string contentIdToString(const NcmContentId& id) {
    char str[33];
    for (size_t i = 0; i < 16; i++) {
        snprintf(&str[i * 2], 3, "%02x", id.c[i]);
    }
    str[32] = '\0';
    return std::string(str);
}

bool parseContentId(const std::string& str, NcmContentId& outId) {
    if (str.length() < 32) {
        return false;
    }
    
    memset(&outId, 0, sizeof(outId));
    for (size_t i = 0; i < 16; i++) {
        std::string byteStr = str.substr(i * 2, 2);
        outId.c[i] = (uint8_t)strtoul(byteStr.c_str(), nullptr, 16);
    }
    
    return true;
}

} // namespace ContentMetaUtil
