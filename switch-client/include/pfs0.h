#pragma once

#include <switch.h>
#include <cstdint>
#include <string>
#include <vector>
#include <functional>

// PFS0 (Partition FS) structure - used by NSP files
// NSP is essentially a PFS0 container with NCAs inside

#pragma pack(push, 1)

struct PFS0Header {
    uint32_t magic;      // "PFS0" = 0x30534650
    uint32_t numFiles;
    uint32_t stringTableSize;
    uint32_t reserved;
};

struct PFS0FileEntry {
    uint64_t dataOffset;
    uint64_t dataSize;
    uint32_t stringTableOffset;
    uint32_t reserved;
};

// NCA Header (first 0x400 bytes, encrypted with header key)
struct NcaHeader {
    uint8_t fixed_key_sig[0x100];
    uint8_t npdm_key_sig[0x100];
    uint32_t magic;              // "NCA3" = 0x3341434E
    uint8_t distribution;        // 0 = download, 1 = gamecard
    uint8_t contentType;         // 0=Program, 1=Meta, 2=Control, 3=Manual, 4=Data, 5=PublicData
    uint8_t keyGeneration;
    uint8_t kaekIndex;
    uint64_t nca_size;
    uint64_t titleId;
    uint32_t contentIndex;
    uint32_t sdkVersion;
    uint8_t keyGeneration2;
    uint8_t fixedKeyGeneration;
    uint8_t padding[0xE];
    uint8_t rightsId[0x10];
    // ... more fields follow
};

#pragma pack(pop)

constexpr uint32_t PFS0_MAGIC = 0x30534650; // "PFS0"
constexpr uint32_t NCA3_MAGIC = 0x3341434E; // "NCA3"
constexpr size_t NCA_HEADER_SIZE = 0x400;

// File entry info parsed from PFS0
struct PFS0FileInfo {
    std::string name;
    uint64_t offset;      // Offset within NSP data section
    uint64_t size;
};

// Callback for reading data from USB
// offset: offset within the NSP file
// size: number of bytes to read
// buffer: destination buffer
// Returns: true on success
using DataReadCallback = std::function<bool(uint64_t offset, uint64_t size, void* buffer)>;

// PFS0 Parser class
class PFS0 {
public:
    PFS0();
    ~PFS0();
    
    // Initialize with data read callback
    bool initialize(DataReadCallback readCallback, uint64_t totalSize);
    
    // Get all file entries
    const std::vector<PFS0FileInfo>& getFiles() const { return m_files; }
    
    // Get file by name
    const PFS0FileInfo* getFileByName(const std::string& name) const;
    
    // Get files by extension (e.g., ".nca", ".cnmt.nca", ".tik", ".cert")
    std::vector<const PFS0FileInfo*> getFilesByExtension(const std::string& ext) const;
    
    // Get data offset (where file data starts after header + string table)
    uint64_t getDataOffset() const { return m_dataOffset; }
    
    // Read file data
    bool readFileData(const PFS0FileInfo& file, uint64_t offset, uint64_t size, void* buffer);
    
private:
    DataReadCallback m_readCallback;
    uint64_t m_totalSize;
    uint64_t m_dataOffset;
    std::vector<PFS0FileInfo> m_files;
    bool m_initialized;
};
