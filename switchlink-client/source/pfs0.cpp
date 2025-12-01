#include "pfs0.h"
#include <cstring>
#include <cstdio>
#include <algorithm>

PFS0::PFS0() : m_totalSize(0), m_dataOffset(0), m_initialized(false) {
}

PFS0::~PFS0() {
}

bool PFS0::initialize(DataReadCallback readCallback, uint64_t totalSize) {
    m_readCallback = readCallback;
    m_totalSize = totalSize;
    m_files.clear();
    m_initialized = false;
    
    // Read PFS0 header
    PFS0Header header;
    if (!m_readCallback(0, sizeof(header), &header)) {
        return false;
    }
    
    // Verify magic
    if (header.magic != PFS0_MAGIC) {
        return false;
    }
    

    
    // Calculate offsets
    uint64_t fileEntriesOffset = sizeof(PFS0Header);
    uint64_t stringTableOffset = fileEntriesOffset + (header.numFiles * sizeof(PFS0FileEntry));
    m_dataOffset = stringTableOffset + header.stringTableSize;
    
    // Read file entries
    std::vector<PFS0FileEntry> entries(header.numFiles);
    if (header.numFiles > 0) {
        if (!m_readCallback(fileEntriesOffset, header.numFiles * sizeof(PFS0FileEntry), entries.data())) {
            return false;
        }
    }
    
    // Read string table
    std::vector<char> stringTable(header.stringTableSize + 1);
    if (header.stringTableSize > 0) {
        if (!m_readCallback(stringTableOffset, header.stringTableSize, stringTable.data())) {
            return false;
        }
    }
    stringTable[header.stringTableSize] = '\0';
    
    // Parse file entries
    for (uint32_t i = 0; i < header.numFiles; i++) {
        PFS0FileInfo info;
        info.name = &stringTable[entries[i].stringTableOffset];
        info.offset = entries[i].dataOffset;
        info.size = entries[i].dataSize;
        m_files.push_back(info);
        

    }
    
    m_initialized = true;
    return true;
}

const PFS0FileInfo* PFS0::getFileByName(const std::string& name) const {
    for (const auto& file : m_files) {
        if (file.name == name) {
            return &file;
        }
    }
    return nullptr;
}

std::vector<const PFS0FileInfo*> PFS0::getFilesByExtension(const std::string& ext) const {
    std::vector<const PFS0FileInfo*> result;
    
    for (const auto& file : m_files) {
        if (file.name.length() >= ext.length()) {
            std::string fileExt = file.name.substr(file.name.length() - ext.length());
            // Case-insensitive comparison
            std::transform(fileExt.begin(), fileExt.end(), fileExt.begin(), ::tolower);
            std::string extLower = ext;
            std::transform(extLower.begin(), extLower.end(), extLower.begin(), ::tolower);
            
            if (fileExt == extLower) {
                result.push_back(&file);
            }
        }
    }
    
    return result;
}

bool PFS0::readFileData(const PFS0FileInfo& file, uint64_t offset, uint64_t size, void* buffer) {
    if (!m_initialized || !m_readCallback) {
        return false;
    }
    
    // Calculate absolute offset within NSP
    uint64_t absoluteOffset = m_dataOffset + file.offset + offset;
    
    return m_readCallback(absoluteOffset, size, buffer);
}
