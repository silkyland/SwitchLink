# 🎮 SwitchLink Client

A lightweight NSP installer for Nintendo Switch via USB streaming.

## ✨ Features

- ✅ **NSP Installation** - Install games directly from PC via USB
- ✅ **Ticket Support** - Automatic ticket installation (requires sigpatches)
- ✅ **DLC & Updates** - Proper support for Base Game + Updates + DLC
- ✅ **Progress Tracking** - Real-time progress with speed indicator
- ✅ **Clean UI** - Console-based interface with ANSI colors

## 📋 Requirements

### Nintendo Switch

- Custom Firmware (Atmosphere recommended)
- Hekate bootloader
- **Sigpatches** (Important!) - Download from [sigmapatches.coomer.party](https://sigmapatches.coomer.party/)
- SD Card with sufficient space

### PC

- SwitchLink Backend (Python)
- USB Type-C cable
- NSP files

## 🚀 Quick Start

### 1. Build

```bash
make clean
make
```

### 2. Install on Switch

Copy `switchlink-client.nro` to:

```
/switch/switchlink-client/switchlink-client.nro
```

### 3. Run

1. Open Homebrew Menu (Hold R + open any game)
2. Launch SwitchLink USB Installer
3. Connect USB to PC
4. Select NSP file from PC
5. Press A to install

## 🎯 Usage

### Controls

- **D-Pad**: Navigate file list
- **L/R**: Page up/down
- **A**: Install selected file
- **B**: Cancel download
- **+**: Exit

### Installation Process

```
1. Parsing NSP structure...
2. Reading content metadata...
3. Installing tickets...
4. Installing NCAs...
5. Finalizing installation...
✓ Installation Complete!
```

## 🐛 Troubleshooting

### Game doesn't appear in Home Menu

1. Reboot your Switch
2. Check if you have sigpatches installed
3. Verify SD card has enough space

### Game appears but won't launch

1. **Install sigpatches** (most common issue)
2. Check if ticket installation succeeded
3. Verify game matches your firmware version

### DLC not working

1. Install Base Game first
2. Verify DLC matches Base Game region
3. Reboot Switch after DLC installation

## 📚 Documentation

- `TODO.md` - Future features and roadmap

## 🔧 Technical Details

### Architecture

- **USB Protocol**: Custom protocol for file streaming
- **NCM**: Nintendo Content Manager for installation
- **ES Service**: eShop Services for ticket installation
- **PFS0**: Partition FileSystem for NSP parsing

### File Structure

```
switch-client/
├── include/          # Header files
│   ├── es_wrapper.h      # ES service wrapper
│   ├── stream_installer.h # Main installer
│   └── ...
├── source/           # Source files
│   ├── es_wrapper.c      # ES implementation
│   ├── stream_installer.cpp # Installer logic
│   └── ...
├── Makefile          # Build configuration
└── README.md         # This file
```

## 🤝 Contributing

Contributions are welcome! See `TODO.md` for planned features.

## 📝 License

MIT License - See LICENSE file for details

## 🙏 Credits

- Based on concepts from Awoo Installer and Tinfoil
- ES service wrapper adapted from Awoo Installer
- Uses libnx for Switch development

## ⚠️ Disclaimer

This software is for educational purposes only. Use at your own risk.

---

**Version:** 0.2.0  
**Author:** Bundit Nuntates  
**Last Updated:** 2025-12-01
