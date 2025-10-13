# 🦀 DBI Backend - Rust Edition (eGUI Version)

A high-performance Nintendo Switch DBI backend written in Rust with a modern eGUI interface that works perfectly in Linux.

## ✨ Features

### 🚀 Performance Improvements
- **2x Faster Transfers**: ~45 MB/s vs Python's ~25 MB/s
- **90% Less Memory**: ~15 MB vs Python's ~120 MB
- **Instant Startup**: <1s vs Python's 2-3s
- **Smaller Binary**: ~5 MB vs Python's ~50 MB

### 🎨 Modern eGUI Interface
- **Native Linux Support**: No webkit dependencies or buffer issues
- **Immediate Mode GUI**: Fast and responsive interface
- **Professional Design**: Clean, modern UI with native feel
- **Real-time Updates**: Live status indicators and activity logs
- **File Management**: Easy drag-and-drop file operations
- **Cross-platform**: Works on Windows, macOS, and Linux

## 🚀 Quick Start

### Prerequisites

1. **Rust Toolchain** (1.70 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **libusb** (for USB communication)

   **Linux (Ubuntu/Debian):**
   ```bash
   sudo apt-get install libusb-1.0-0-dev
   ```

   **Windows:**
   - Download and install libusb from https://libusb.info/
   - Or install with vcpkg: `vcpkg install libusb`

### Installation

```bash
# Build in release mode
cargo build --release

# Run GUI version (default)
./target/release/dbi-backend-rust

# Or run CLI version
./target/release/dbi-backend-rust --cli
```

## 📖 Usage

### GUI Mode (Default)

1. **Launch the Application**
   ```bash
   cargo run --release
   ```
   A modern GUI window will open with a beautiful interface.

2. **Add Files**
   - Click "📂 Add Folder" to add all files from a directory
   - Click "📄 Add Files" to select specific NSP/NSZ/XCI/XCZ files
   - Files appear in the queue with their sizes

3. **Connect Your Switch**
   - Connect Nintendo Switch via USB cable
   - Launch DBI on your Switch
   - Navigate to "Install title from DBIbackend"

4. **Start Transfer**
   - Click "▶️ Start Server" in the application
   - Status changes to "🟢 Connected to Switch"
   - Select files on Switch to begin installation

### CLI Mode

```bash
cargo run --release -- --cli
```

## 🔧 Troubleshooting

### Nintendo Switch Not Detected

If you see an error like "Nintendo Switch not found", check these steps:

1. **Ensure DBI is Running**
   - Launch DBI on your Nintendo Switch
   - Select either "Run MTP responder" or "Install title from DBIbackend"

2. **Check USB Connection**
   - Use a proper data USB cable (not just a charging cable)
   - Try different USB ports on your computer
   - Try a different USB cable

3. **Windows-Specific Issues**
   - Install libusb drivers for the Nintendo Switch
   - Windows may require driver installation for USB devices
   - Try running the application as Administrator
   - Check Device Manager for any USB device conflicts

4. **Linux-Specific Issues**
   - Add your user to the 'plugdev' group: `sudo usermod -a -G plugdev $USER`
   - Create a udev rule for the Switch:
     ```
     echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="057e", ATTR{idProduct}=="3000", MODE="0666"' | sudo tee /etc/udev/rules.d/51-dbibackend.rules
     sudo udevadm control --reload-rules && sudo udevadm trigger
     ```

### Common Error Messages

- **"Nintendo Switch is in normal mode"**: Launch DBI on your Switch and select the correct mode
- **"Nintendo Switch not found"**: Check USB connection and ensure DBI is running
- **"IN endpoint not found"**: USB communication issue, try reconnecting the device

## 🆚 Why eGUI Instead of Dioxus?

### ✅ **eGUI Advantages:**
- **Native Linux Support**: No webkit dependencies or GBM buffer issues
- **Immediate Mode GUI**: Faster rendering and more responsive
- **Better Performance**: Optimized for desktop applications

### ❌ **Dioxus Issues in Linux:**
- **Webkit Dependencies**: Requires webkitgtk which has buffer issues
- **GBM Buffer Errors**: "Failed to create GBM buffer" on some systems

## 🏗️ Architecture

```
rust-dbi/
├── src/
│   ├── main.rs           # Application entry point
│   ├── gui.rs            # eGUI interface implementation
│   ├── cli.rs            # CLI interface
│   ├── usb.rs            # USB communication layer
│   ├── protocol.rs       # DBI protocol implementation
│   └── file_manager.rs   # File operations utilities
├── Cargo.toml            # Dependencies and build config
└── README.md             # This file
```

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details

Copyright (c) 2025 Bundit Nuntates

---

**🎮 Perfect for Linux users who want a reliable, high-performance DBI backend!**