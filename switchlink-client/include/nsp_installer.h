#pragma once

#include <switch.h>
#include <string>
#include <cstdint>

// NSP/NSZ Installer class
class NSPInstaller {
public:
    NSPInstaller();
    ~NSPInstaller();
    
    // Initialize NCM (Nintendo Content Manager)
    bool initialize();
    
    // Close NCM
    void close();
    
    // Install NSP file
    // destPath: path to NSP file on SD card (e.g., "/switch/downloads/game.nsp")
    // installToNand: true = install to NAND, false = install to SD card
    bool installNSP(const std::string& destPath, bool installToNand = false);
    
    // Get installation progress
    struct InstallProgress {
        uint64_t bytesInstalled;
        uint64_t totalBytes;
        float percentage;
        std::string currentFile;
    };
    
    InstallProgress getProgress() const { return m_progress; }
    
private:
    bool m_initialized;
    NcmContentStorage m_contentStorage;
    NcmContentMetaDatabase m_metaDb;
    FsFileSystem m_fileSystem;
    NsApplicationRecord m_appRecord;
    InstallProgress m_progress;
    
    // Helper functions
    bool openNSP(const std::string& path);
    bool extractAndInstall();
    bool verifyInstallation();
};
