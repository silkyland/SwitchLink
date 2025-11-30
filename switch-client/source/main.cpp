#include <switch.h>
#include <cstdio>
#include <cstring>
#include <sys/stat.h>
#include <algorithm>
#include <vector>
#include <string>
#include "usb_client.h"
#include "nsp_installer.h"

// Console dimensions (Switch default: 80x45)
#define CONSOLE_WIDTH 80
#define CONSOLE_HEIGHT 45
#define HEADER_HEIGHT 4
#define FOOTER_HEIGHT 3
#define LIST_HEIGHT (CONSOLE_HEIGHT - HEADER_HEIGHT - FOOTER_HEIGHT - 2)

// ANSI escape codes
#define ESC "\x1b"
#define CSI ESC "["

// Cursor control
#define CURSOR_HOME CSI "H"
#define CLEAR_SCREEN CSI "2J"
#define CLEAR_LINE CSI "2K"

// Foreground colors
#define FG_BLACK CSI "30m"
#define FG_RED CSI "31m"
#define FG_GREEN CSI "32m"
#define FG_YELLOW CSI "33m"
#define FG_BLUE CSI "34m"
#define FG_CYAN CSI "36m"
#define FG_WHITE CSI "37m"
#define FG_BRIGHT_RED CSI "91m"
#define FG_BRIGHT_WHITE CSI "97m"
#define FG_BRIGHT_YELLOW CSI "93m"
#define FG_BRIGHT_GREEN CSI "92m"
#define FG_BRIGHT_CYAN CSI "96m"

// Background colors
#define BG_BLACK CSI "40m"
#define BG_BLUE CSI "44m"
#define BG_WHITE CSI "47m"

// Text styles
#define RESET_ALL CSI "0m"
#define BOLD CSI "1m"

// Global state
PadState pad;

struct FileEntry {
    FileInfo info;
    bool installed;
};

// Move cursor to specific position (1-indexed)
void moveCursor(int row, int col) {
    printf(CSI "%d;%dH", row, col);
}

// Clear entire screen and reset cursor
void clearScreen() {
    printf(RESET_ALL);
    printf(CLEAR_SCREEN);
    printf(CURSOR_HOME);
    consoleClear();
    consoleUpdate(NULL);
}

// Fill a line with spaces (for clean background)
void fillLine(int row, const char* bgColor) {
    moveCursor(row, 1);
    printf("%s", bgColor);
    for (int i = 0; i < CONSOLE_WIDTH; i++) printf(" ");
}

// Draw header with title
void drawHeader(size_t fileCount) {
    // Line 1: Empty blue line
    fillLine(1, BG_BLUE);
    
    // Line 2: Title
    moveCursor(2, 1);
    printf(BG_BLUE FG_BRIGHT_WHITE BOLD);
    printf("  SwitchLink Installer");
    printf(RESET_ALL BG_BLUE FG_WHITE);
    printf("  -  ");
    printf(FG_BRIGHT_YELLOW "%zu" FG_WHITE " files available", fileCount);
    // Fill rest of line
    for (int i = 40; i < CONSOLE_WIDTH; i++) printf(" ");
    
    // Line 3: Empty
    fillLine(3, BG_BLUE);
    
    // Line 4: Separator
    moveCursor(4, 1);
    printf(BG_BLUE FG_CYAN);
    for (int i = 0; i < CONSOLE_WIDTH; i++) printf("=");
}

// Draw footer with controls
void drawFooter() {
    int footerStart = CONSOLE_HEIGHT - FOOTER_HEIGHT + 1;
    
    // Separator line
    moveCursor(footerStart, 1);
    printf(BG_BLUE FG_CYAN);
    for (int i = 0; i < CONSOLE_WIDTH; i++) printf("=");
    
    // Controls line
    moveCursor(footerStart + 1, 1);
    printf(BG_BLUE FG_WHITE);
    printf(" ");
    printf(FG_BRIGHT_YELLOW "A" FG_WHITE ":Install ");
    printf(FG_BRIGHT_YELLOW "D-Pad" FG_WHITE ":Navigate ");
    printf(FG_BRIGHT_YELLOW "L/R" FG_WHITE ":Page ");
    printf(FG_BRIGHT_YELLOW "+" FG_WHITE ":Exit");
    // Fill rest
    for (int i = 55; i < CONSOLE_WIDTH; i++) printf(" ");
    
    // Bottom line
    fillLine(footerStart + 2, BG_BLUE);
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

// Progress bar state
static uint64_t g_lastBytes = 0;
static uint64_t g_lastTime = 0;
static float g_lastSpeed = 0.0f;
static bool g_cancelRequested = false;

void resetProgressState() {
    g_lastBytes = 0;
    g_lastTime = 0;
    g_lastSpeed = 0.0f;
    g_cancelRequested = false;
}

// Show cancel confirmation dialog
// Returns true if user confirms cancel, false to continue
bool showCancelConfirmation() {
    // Save current screen area (we'll just redraw after)
    int dialogRow = 20;
    
    // Draw dialog box
    moveCursor(dialogRow, 1);
    printf(BG_BLUE);
    printf(CLEAR_LINE);
    
    moveCursor(dialogRow + 1, 1);
    printf(BG_BLACK FG_BRIGHT_WHITE);
    printf("  +------------------------------------------+  ");
    
    moveCursor(dialogRow + 2, 1);
    printf(BG_BLACK FG_BRIGHT_WHITE);
    printf("  |                                          |  ");
    
    moveCursor(dialogRow + 3, 1);
    printf(BG_BLACK FG_BRIGHT_YELLOW);
    printf("  |       Cancel download?                   |  ");
    
    moveCursor(dialogRow + 4, 1);
    printf(BG_BLACK FG_WHITE);
    printf("  |                                          |  ");
    
    moveCursor(dialogRow + 5, 1);
    printf(BG_BLACK FG_WHITE);
    printf("  |   " FG_BRIGHT_GREEN "A" FG_WHITE ": Yes, Cancel    " FG_BRIGHT_RED "B" FG_WHITE ": No, Continue   |  ");
    
    moveCursor(dialogRow + 6, 1);
    printf(BG_BLACK FG_BRIGHT_WHITE);
    printf("  +------------------------------------------+  ");
    
    moveCursor(dialogRow + 7, 1);
    printf(BG_BLUE);
    printf(CLEAR_LINE);
    
    consoleUpdate(NULL);
    
    // Wait for user input
    while (appletMainLoop()) {
        padUpdate(&pad);
        u64 kDown = padGetButtonsDown(&pad);
        
        if (kDown & HidNpadButton_A) {
            return true;  // Confirm cancel
        }
        if (kDown & HidNpadButton_B) {
            return false; // Continue download
        }
    }
    
    return false;
}

// Draw progress bar and check for cancel request
// Returns true to continue, false to cancel
bool drawProgressBar(int row, uint64_t current, uint64_t total) {
    // Check for B button press (cancel request)
    padUpdate(&pad);
    u64 kDown = padGetButtonsDown(&pad);
    
    if (kDown & HidNpadButton_B) {
        if (showCancelConfirmation()) {
            g_cancelRequested = true;
            return false; // Cancel
        }
        // User chose to continue - redraw progress area
    }
    
    moveCursor(row, 1);
    printf(BG_BLUE);
    printf(CLEAR_LINE);
    
    int barWidth = 40;
    float percent = 0.0f;
    int filled = 0;
    
    if (total > 0) {
        percent = (float)current / (float)total * 100.0f;
        if (percent > 100.0f) percent = 100.0f;
        filled = (int)(percent / 100.0f * barWidth);
    }
    
    // Draw progress bar
    printf("  " FG_WHITE "[");
    for (int i = 0; i < barWidth; i++) {
        if (i < filled) {
            printf(FG_BRIGHT_GREEN "#");
        } else {
            printf(FG_WHITE "-");
        }
    }
    printf(FG_WHITE "] ");
    
    // Percentage
    printf(FG_BRIGHT_YELLOW "%5.1f%%" FG_WHITE, percent);
    
    // Size info
    moveCursor(row + 1, 1);
    printf(BG_BLUE);
    printf(CLEAR_LINE);
    printf("  " FG_WHITE "Progress: " FG_BRIGHT_CYAN "%s" FG_WHITE " / " FG_BRIGHT_CYAN "%s",
           formatSize(current).c_str(), formatSize(total).c_str());
    
    // Calculate speed
    uint64_t currentTime = armGetSystemTick();
    if (g_lastTime > 0) {
        uint64_t timeDiff = (currentTime - g_lastTime) / 19200000; // Convert to ms
        if (timeDiff > 300) { // Update every 300ms
            uint64_t bytesDiff = current - g_lastBytes;
            g_lastSpeed = (float)bytesDiff / (float)timeDiff * 1000.0f / 1024.0f / 1024.0f;
            g_lastBytes = current;
            g_lastTime = currentTime;
        }
    } else {
        g_lastBytes = current;
        g_lastTime = currentTime;
    }
    
    // Show speed
    printf("   " FG_BRIGHT_GREEN "%.1f MB/s" FG_WHITE, g_lastSpeed);
    
    // Show cancel hint
    moveCursor(row + 3, 1);
    printf(BG_BLUE FG_YELLOW);
    printf("  Press " FG_BRIGHT_YELLOW "B" FG_YELLOW " to cancel download");
    
    consoleUpdate(NULL);
    
    return true; // Continue
}

// Draw a simple message screen
void drawMessageScreen(const char* title, const char* message, const char* submessage = nullptr) {
    clearScreen();
    
    // Fill background
    for (int i = 1; i <= CONSOLE_HEIGHT; i++) {
        fillLine(i, BG_BLUE);
    }
    
    // Center the message
    int centerRow = CONSOLE_HEIGHT / 2 - 2;
    
    moveCursor(centerRow, 1);
    printf(BG_BLUE FG_BRIGHT_WHITE BOLD);
    printf("  %s", title);
    
    moveCursor(centerRow + 2, 1);
    printf(BG_BLUE FG_WHITE);
    printf("  %s", message);
    
    if (submessage) {
        moveCursor(centerRow + 4, 1);
        printf(BG_BLUE FG_YELLOW);
        printf("  %s", submessage);
    }
    
    consoleUpdate(NULL);
}

// Draw file list item
void drawFileItem(int row, int index, const FileEntry& entry, bool isSelected) {
    moveCursor(row, 1);
    
    if (isSelected) {
        // Selected item: highlighted
        printf(BG_WHITE FG_BLACK);
    } else {
        printf(BG_BLUE FG_WHITE);
    }
    
    // Status indicator
    if (entry.installed) {
        printf(isSelected ? FG_GREEN : FG_BRIGHT_GREEN);
        printf(" [OK] ");
    } else {
        printf(isSelected ? FG_BLACK : FG_WHITE);
        printf(" [ ]  ");
    }
    
    // Filename (truncate if too long)
    std::string name = entry.info.filename;
    int maxNameLen = 55;
    if ((int)name.length() > maxNameLen) {
        name = name.substr(0, maxNameLen - 3) + "...";
    }
    
    printf(isSelected ? FG_BLACK : FG_WHITE);
    printf("%-55s", name.c_str());
    
    // Size (right aligned)
    printf(isSelected ? FG_BLUE : FG_BRIGHT_CYAN);
    printf("%10s", formatSize(entry.info.size).c_str());
    
    // Fill rest of line
    printf(" ");
    
    printf(RESET_ALL);
}

// Draw the main file list screen
void drawFileListScreen(const std::vector<FileEntry>& entries, int selectedIdx, int scrollOffset) {
    // Clear and fill background
    clearScreen();
    for (int i = 1; i <= CONSOLE_HEIGHT; i++) {
        fillLine(i, BG_BLUE);
    }
    
    // Draw header
    drawHeader(entries.size());
    
    // Calculate visible range
    int listStartRow = HEADER_HEIGHT + 1;
    int maxVisible = LIST_HEIGHT;
    
    // Adjust scroll offset to keep selected item visible
    if (selectedIdx < scrollOffset) {
        scrollOffset = selectedIdx;
    } else if (selectedIdx >= scrollOffset + maxVisible) {
        scrollOffset = selectedIdx - maxVisible + 1;
    }
    
    int endIdx = std::min((int)entries.size(), scrollOffset + maxVisible);
    
    // Draw file list
    for (int i = scrollOffset; i < endIdx; i++) {
        int row = listStartRow + (i - scrollOffset);
        drawFileItem(row, i, entries[i], i == selectedIdx);
    }
    
    // Draw scroll indicator if needed
    if (entries.size() > (size_t)maxVisible) {
        int indicatorRow = listStartRow + maxVisible / 2;
        moveCursor(indicatorRow, CONSOLE_WIDTH - 3);
        printf(BG_BLUE FG_YELLOW);
        if (scrollOffset > 0) {
            moveCursor(listStartRow, CONSOLE_WIDTH - 3);
            printf("^");
        }
        if (endIdx < (int)entries.size()) {
            moveCursor(listStartRow + maxVisible - 1, CONSOLE_WIDTH - 3);
            printf("v");
        }
    }
    
    // Draw footer
    drawFooter();
    
    consoleUpdate(NULL);
}

// Draw download screen
void drawDownloadScreen(const std::string& filename, uint64_t fileSize) {
    clearScreen();
    for (int i = 1; i <= CONSOLE_HEIGHT; i++) {
        fillLine(i, BG_BLUE);
    }
    
    // Title
    moveCursor(3, 1);
    printf(BG_BLUE FG_BRIGHT_WHITE BOLD);
    printf("  Downloading File");
    
    // Separator
    moveCursor(5, 1);
    printf(BG_BLUE FG_CYAN);
    for (int i = 0; i < CONSOLE_WIDTH; i++) printf("=");
    
    // File info
    moveCursor(8, 1);
    printf(BG_BLUE FG_WHITE);
    printf("  File: " FG_BRIGHT_YELLOW "%s", filename.c_str());
    
    moveCursor(10, 1);
    printf(BG_BLUE FG_WHITE);
    printf("  Size: " FG_BRIGHT_CYAN "%s", formatSize(fileSize).c_str());
    
    consoleUpdate(NULL);
}

int main(int argc, char* argv[]) {
    consoleInit(NULL);
    padConfigureInput(1, HidNpadStyleSet_NpadStandard);
    padInitializeDefault(&pad);
    
    // Show connecting screen
    drawMessageScreen("SwitchLink Installer", "Connecting to PC...", "Please wait...");
    
    USBClient client;
    bool usbConnected = client.initialize();
    
    if (!usbConnected) {
        drawMessageScreen("Connection Failed", 
                         "Could not connect to PC!",
                         "Check USB cable and run SwitchLink Backend. Press [+] to exit.");
        while (appletMainLoop()) {
            padUpdate(&pad);
            if (padGetButtonsDown(&pad) & HidNpadButton_Plus) break;
        }
        consoleExit(NULL);
        return 0;
    }
    
    // Fetch file list
    drawMessageScreen("SwitchLink Installer", "Fetching file list...", "Please wait...");
    
    auto files = client.listFiles();
    std::vector<FileEntry> entries;
    for (const auto& f : files) {
        entries.push_back({f, false});
    }
    
    if (files.empty()) {
        drawMessageScreen("No Files Available", 
                         "Add files in SwitchLink Backend on your PC.",
                         "Press [+] to exit.");
        while (appletMainLoop()) {
            padUpdate(&pad);
            if (padGetButtonsDown(&pad) & HidNpadButton_Plus) break;
        }
        client.close();
        consoleExit(NULL);
        return 0;
    }
    
    int selectedIdx = 0;
    int scrollOffset = 0;
    bool needsRedraw = true;
    
    while (appletMainLoop()) {
        padUpdate(&pad);
        u64 kDown = padGetButtonsDown(&pad);
        
        if (kDown & HidNpadButton_Plus) break;
        
        // Navigation
        if (kDown & HidNpadButton_Down) {
            if (selectedIdx < (int)entries.size() - 1) {
                selectedIdx++;
                // Adjust scroll
                if (selectedIdx >= scrollOffset + LIST_HEIGHT) {
                    scrollOffset++;
                }
                needsRedraw = true;
            }
        }
        
        if (kDown & HidNpadButton_Up) {
            if (selectedIdx > 0) {
                selectedIdx--;
                // Adjust scroll
                if (selectedIdx < scrollOffset) {
                    scrollOffset--;
                }
                needsRedraw = true;
            }
        }
        
        // Page navigation
        if (kDown & HidNpadButton_R) {
            selectedIdx = std::min((int)entries.size() - 1, selectedIdx + LIST_HEIGHT);
            scrollOffset = std::max(0, selectedIdx - LIST_HEIGHT + 1);
            needsRedraw = true;
        }
        
        if (kDown & HidNpadButton_L) {
            selectedIdx = std::max(0, selectedIdx - LIST_HEIGHT);
            scrollOffset = std::max(0, scrollOffset - LIST_HEIGHT);
            needsRedraw = true;
        }
        
        // Install selected file
        if (kDown & HidNpadButton_A) {
            FileEntry& entry = entries[selectedIdx];
            uint64_t fileSize = entry.info.size;
            
            // Show download screen
            drawDownloadScreen(entry.info.filename, fileSize);
            resetProgressState();
            
            // Create download directory
            mkdir("/switch/downloads", 0777);
            std::string destPath = "/switch/downloads/" + entry.info.filename;
            
            // Download with progress (callback returns true to continue, false to cancel)
            bool success = client.downloadFile(entry.info.filename, destPath, 
                [fileSize](uint64_t current, uint64_t total) -> bool {
                    return drawProgressBar(14, current, fileSize > 0 ? fileSize : total);
                });
            
            // Clear the cancel hint and dialog area
            for (int i = 17; i <= 28; i++) {
                moveCursor(i, 1);
                printf(BG_BLUE);
                printf(CLEAR_LINE);
            }
            
            if (success) {
                entry.installed = true;
                
                // Show success message
                moveCursor(18, 1);
                printf(BG_BLUE FG_BRIGHT_GREEN BOLD);
                printf("  Download Complete!");
                consoleUpdate(NULL);
                
                svcSleepThread(1000000000ULL); // 1 second
            } else if (g_cancelRequested) {
                // Show cancelled message
                moveCursor(18, 1);
                printf(BG_BLUE FG_BRIGHT_YELLOW BOLD);
                printf("  Download Cancelled");
                consoleUpdate(NULL);
                
                svcSleepThread(1000000000ULL); // 1 second
            } else {
                // Show error message
                moveCursor(18, 1);
                printf(BG_BLUE FG_RED BOLD);
                printf("  Download Failed!");
                moveCursor(20, 1);
                printf(BG_BLUE FG_WHITE);
                printf("  Press any button to continue...");
                consoleUpdate(NULL);
                
                while (appletMainLoop()) {
                    padUpdate(&pad);
                    if (padGetButtonsDown(&pad)) break;
                }
            }
            
            needsRedraw = true;
        }
        
        // Redraw file list
        if (needsRedraw) {
            drawFileListScreen(entries, selectedIdx, scrollOffset);
            needsRedraw = false;
        }
    }
    
    client.close();
    consoleExit(NULL);
    return 0;
}
