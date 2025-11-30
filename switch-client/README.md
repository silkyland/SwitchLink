# 🎮 SwitchLink Client

Nintendo Switch client for SwitchLink file transfer system.

> **Inspired by [DBI](https://github.com/rashevskyv/dbi)**

## ✨ Features

- 🚀 **Fast USB transfers** - 1MB chunks, ~45 MB/s
- 📱 **Clean UI** - ANSI terminal interface
- ❌ **Cancel support** - Press B with confirmation
- 📜 **File browser** - D-Pad navigate, L/R page

## 🛠️ Building

```bash
# Requires devkitPro
make

# Or with Docker
docker run --rm -v $(pwd):/src devkitpro/devkita64 make
```

## 📦 Installation

1. Copy `switchlink-client.nro` to `/switch/` on SD card
2. Launch from Homebrew Menu
3. Connect USB to PC running SwitchLink Backend

## 🎮 Controls

| Button | Action |
|--------|--------|
| A | Download selected file |
| B | Cancel download |
| D-Pad | Navigate list |
| L/R | Page up/down |
| + | Exit |

## 📄 License

MIT License
