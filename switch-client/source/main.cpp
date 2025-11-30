#include <switch.h>
#include <cstdio>
#include <cstring>
#include <sys/stat.h>
#include <algorithm>
#include <vector>
#include <string>
#include "usb_client.h"
#include "nsp_installer.h"

// DBI Style Colors
#define BG_BLUE "\x1b[44m"
#define BG_BLACK "\x1b[40m"
#define TXT_WHITE "\x1b[37m"
#define TXT_RED "\x1b[31m"
#define TXT_YELLOW "\x1b[33m"
#define TXT_GREEN "\x1b[32m"
#define TXT_CYAN "\x1b[36m"
#define RESET "\x1b[0m"

// Global PadState
PadState pad;

struct FileEntry {
    FileInfo info;
    bool installed;
};

void clearScreenBlue() {
    // Fill screen with blue background
    printf(BG_BLUE);
    consoleClear();
}

void drawHeader(size_t fileCount) {
    printf(BG_BLUE TXT_WHITE);
    printf("                                                                                \n");
    printf("           " TXT_WHITE "SwitchLink Installer" TXT_WHITE " (" TXT_YELLOW "%zu" TXT_WHITE ") files available                          \n", fileCount);
    printf("                                                                                \n");
    printf(TXT_WHITE "────────────────────────────────────────────────────────────────────────────────\n");
}

void drawFooter() {
    // Move to bottom (approximate line 40)
    printf("\x1b[40;1H"); // Move cursor to line 40
    printf(TXT_WHITE "────────────────────────────────────────────────────────────────────────────────\n");
    printf(BG_BLUE TXT_WHITE "  " TXT_YELLOW "A" TXT_WHITE " - Install   " TXT_YELLOW "UP/DOWN" TXT_WHITE " - Navigate   " TXT_YELLOW "+" TXT_WHITE " - Exit                                \n");
    printf("                                                                                \n");
}

std::string formatSize(uint64_t bytes) {
    char buf[32];
    if (bytes == 0) return "Unknown";
    
    double size = (double)bytes;
    const char* unit = "B";
    if (size >= 1024) { size /= 1024; unit = "KB"; }
    if (size >= 1024) { size /= 1024; unit = "MB"; }
    if (size >= 1024) { size /= 1024; unit = "GB"; }
    
    snprintf(buf, sizeof(buf), "%.2f %s", size, unit);
    return std::string(buf);
}

void printProgress(uint64_t current, uint64_t total) {
    // DBI Style Progress
    // [################====] 45.5% (1.2/2.5 GB) @ 25.0 MB/s
    
    printf(BG_BLUE TXT_WHITE);
    printf("\r  ");
    
    int barWidth = 30;
    int filled = 0;
    float percent = 0.0f;
    
    if (total > 0 && total > current) {
        percent = (float)current / (float)total * 100.0f;
        filled = (int)(percent / 100.0f * barWidth);
    } else if (total > 0 && current >= total) {
        percent = 100.0f;
        filled = barWidth;
    } else {
        filled = (int)((current / (1024 * 1024)) % barWidth);
    }
    
    printf("[");
    for (int i = 0; i < barWidth; i++) {
        if (i < filled) printf(TXT_YELLOW "#" TXT_WHITE);
        else printf(TXT_WHITE "=");
    }
    printf("] ");
    
    if (total > 0) {
        printf(TXT_YELLOW "%5.1f%%" TXT_WHITE, percent);
    }
    
    std::string curStr = formatSize(current);
    std::string totStr = formatSize(total);
    
    printf(" (%s/%s) ", curStr.c_str(), totStr.c_str());
    
    // Speed
    static uint64_t lastBytes = 0;
    static uint64_t lastTime = 0;
    uint64_t currentTime = armGetSystemTick();
    
    if (lastTime > 0) {
        uint64_t timeDiff = (currentTime - lastTime) / 19200000; // ms
        if (timeDiff > 200) {
            uint64_t bytesDiff = current - lastBytes;
            float speed = (float)bytesDiff / (float)timeDiff * 1000.0f / 1024.0f / 1024.0f;
            printf("@ " TXT_GREEN "%.1f MB/s" TXT_WHITE, speed);
            lastBytes = current;
            lastTime = currentTime;
        }
    } else {
        lastBytes = current;
        lastTime = currentTime;
    }
    
    fflush(stdout);
}

int main(int argc, char* argv[]) {
    consoleInit(NULL);
    padConfigureInput(1, HidNpadStyleSet_NpadStandard);
    padInitializeDefault(&pad);
    
    // Initial Blue Screen
    clearScreenBlue();
    
    printf("\n  " TXT_WHITE "Connecting to PC..." RESET "\n");
    consoleUpdate(NULL);
    
    USBClient client;
    bool usbConnected = client.initialize();
    
    if (!usbConnected) {
        clearScreenBlue();
        printf("\n  " TXT_RED "Failed to connect to PC!" TXT_WHITE "\n");
        printf("  Please check USB connection and run SwitchLink Backend.\n\n");
        printf("  Press + to exit.\n");
        while (appletMainLoop()) {
            padUpdate(&pad);
            if (padGetButtonsDown(&pad) & HidNpadButton_Plus) break;
        }
        consoleExit(NULL);
        return 0;
    }
    
    // Fetch list
    clearScreenBlue();
    printf("\n  " TXT_WHITE "Fetching file list..." RESET "\n");
    consoleUpdate(NULL);
    
    auto files = client.listFiles();
    std::vector<FileEntry> entries;
    for (const auto& f : files) {
        entries.push_back({f, false});
    }
    
    if (files.empty()) {
        clearScreenBlue();
        printf("\n  " TXT_RED "No files available!" TXT_WHITE "\n");
        printf("  Add files in SwitchLink Backend on your PC.\n\n");
        printf("  Press + to exit.\n");
        while (appletMainLoop()) {
            padUpdate(&pad);
            if (padGetButtonsDown(&pad) & HidNpadButton_Plus) break;
        }
        client.close();
        consoleExit(NULL);
        return 0;
    }
    
    int selectedIdx = 0;
    bool needsRedraw = true;
    
    while (appletMainLoop()) {
        padUpdate(&pad);
        u64 kDown = padGetButtonsDown(&pad);
        
        if (kDown & HidNpadButton_Plus) break;
        
        if (kDown & HidNpadButton_Down) {
            if (selectedIdx < (int)entries.size() - 1) {
                selectedIdx++;
                needsRedraw = true;
            }
        }
        
        if (kDown & HidNpadButton_Up) {
            if (selectedIdx > 0) {
                selectedIdx--;
                needsRedraw = true;
            }
        }
        
        if (kDown & HidNpadButton_A) {
            // Install Mode
            clearScreenBlue();
            drawHeader(entries.size());
            
            FileEntry& entry = entries[selectedIdx];
            
            printf("\n\n");
            printf("  " TXT_WHITE "Installing: " TXT_YELLOW "%s" TXT_WHITE "\n", entry.info.filename.c_str());
            printf("  " TXT_WHITE "Size: " TXT_CYAN "%s" TXT_WHITE "\n", formatSize(entry.info.size).c_str());
            printf("\n\n");
            
            mkdir("/switch/downloads", 0777);
            std::string destPath = "/switch/downloads/" + entry.info.filename;
            uint64_t fileSize = entry.info.size;
            
            bool success = client.downloadFile(entry.info.filename, destPath, 
                [fileSize](uint64_t current, uint64_t total) {
                    printProgress(current, fileSize > 0 ? fileSize : total);
                });
            
            printf("\n\n");
            if (success) {
                printf("  " TXT_GREEN "Installation Complete!" TXT_WHITE "\n");
                entry.installed = true;
                consoleUpdate(NULL);
                
                // Auto return immediately (just a small delay to see the message)
                svcSleepThread(500000000ULL); // 0.5 seconds
            } else {
                printf("  " TXT_RED "Installation Failed!" TXT_WHITE "\n");
                printf("\n  " TXT_WHITE "Press any key to return..." RESET);
                consoleUpdate(NULL);
                
                // Wait for key press only on failure
                while (appletMainLoop()) {
                    padUpdate(&pad);
                    if (padGetButtonsDown(&pad)) break;
                }
            }
            
            needsRedraw = true;
        }
        
        if (needsRedraw) {
            clearScreenBlue();
            drawHeader(entries.size());
            
            int maxVisible = 25; // More items visible
            int startIdx = 0;
            if (selectedIdx >= maxVisible) {
                startIdx = selectedIdx - maxVisible + 1;
            }
            int endIdx = std::min((int)entries.size(), startIdx + maxVisible);
            
            for (int i = startIdx; i < endIdx; i++) {
                bool isSelected = (i == selectedIdx);
                const auto& entry = entries[i];
                
                if (isSelected) {
                    // Selected: Black BG, Red Text (DBI Style)
                    printf(BG_BLACK TXT_RED);
                    printf("  [M] %s ", entry.info.filename.c_str());
                    
                    // Padding to fill line
                    int len = entry.info.filename.length() + 6;
                    for(int k=len; k<60; k++) printf(" ");
                    
                    printf("%10s  ", formatSize(entry.info.size).c_str());
                    printf(BG_BLUE TXT_WHITE "\n"); // Reset to blue
                } else {
                    // Normal: Blue BG, White Text
                    printf(BG_BLUE TXT_WHITE);
                    if (entry.installed) {
                        printf("  " TXT_GREEN "[I]" TXT_WHITE " %s ", entry.info.filename.c_str());
                    } else {
                        printf("  [ ] %s ", entry.info.filename.c_str());
                    }
                    
                    // Padding
                    int len = entry.info.filename.length() + 6;
                    for(int k=len; k<60; k++) printf(" ");
                    
                    printf("%10s  \n", formatSize(entry.info.size).c_str());
                }
            }
            
            drawFooter();
            consoleUpdate(NULL);
            needsRedraw = false;
        }
    }
    
    client.close();
    consoleExit(NULL);
    return 0;
}
