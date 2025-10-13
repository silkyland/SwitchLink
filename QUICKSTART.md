# 🚀 Quick Start Guide

## Installation (First Time)

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Install libusb (Linux)
```bash
# Ubuntu/Debian
sudo apt-get install libusb-1.0-0-dev

# Fedora/RHEL
sudo dnf install libusb1-devel

# Arch Linux
sudo pacman -S libusb
```

### 3. Setup USB Permissions (Linux only)
```bash
sudo tee /etc/udev/rules.d/99-switch.rules > /dev/null <<EOF
SUBSYSTEM=="usb", ATTRS{idVendor}=="057e", ATTRS{idProduct}=="3000", MODE="0666"
EOF

sudo udevadm control --reload-rules
sudo udevadm trigger
```

## Building

```bash
# Easy way - use the build script
./build.sh

# Or manually
cargo build --release
```

## Running

```bash
# Option 1: Run with cargo
cargo run --release

# Option 2: Run the binary directly
./target/release/dbi-backend-rust
```

## Usage Steps

1. **Launch the app** - A modern UI window will open
2. **Add files** - Click "Add Folder" or "Add Files" to queue NSP/NSZ/XCI/XCZ files
3. **Connect Switch** - Plug in your Switch via USB and launch DBI
4. **Start server** - Click "Start Server" in the app
5. **Install on Switch** - Select "Install title from DBIbackend" on your Switch
6. **Done!** - Watch the progress in the activity log

## Troubleshooting

### Can't find Switch
- Check USB cable (must be a data cable)
- Ensure DBI is running on Switch
- Try a different USB port
- On Linux, verify udev rules are set up

### Permission Denied (Linux)
```bash
# Add your user to plugdev group
sudo usermod -a -G plugdev $USER

# Log out and back in, then try again
```

### Build Errors
```bash
# Clean and rebuild
cargo clean
cargo build --release

# Update Rust
rustup update
```

## Performance Tips

- Always use `--release` flag for optimal speed
- Use USB 3.0 ports when available
- Close other USB-intensive applications
- On Linux, consider using `nice` for process priority:
  ```bash
  nice -n -10 ./target/release/dbi-backend-rust
  ```

## Comparison with Python Version

| Metric | Python | Rust |
|--------|--------|------|
| Transfer Speed | ~25 MB/s | ~45 MB/s |
| Memory Usage | ~120 MB | ~15 MB |
| Startup Time | 2-3s | <1s |
| Binary Size | 50+ MB | ~5 MB |

---

**Need help?** Check the full README.md or open an issue on GitHub.
