# Cross-Compilation Guide

Build DBI Backend for different platforms from Linux.

## 📦 Prerequisites

### Method 1: Using `cross` (Recommended)

```bash
# Install cross
cargo install cross

# Install Docker (required by cross)
# Ubuntu/Debian:
sudo apt-get install docker.io
sudo usermod -aG docker $USER
# Log out and log back in
```

### Method 2: Using `cargo` directly

```bash
# Install MinGW for Windows cross-compilation
sudo apt-get install mingw-w64

# Add Windows target
rustup target add x86_64-pc-windows-gnu
```

---

## 🪟 Build for Windows

### Using cross (Easy):
```bash
# Windows 64-bit (GNU)
cross build --release --target x86_64-pc-windows-gnu

# Output: target/x86_64-pc-windows-gnu/release/dbi-backend-rust.exe
```

### Using cargo (Manual):
```bash
# Add target
rustup target add x86_64-pc-windows-gnu

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

### Using build script:
```bash
./build-windows.sh
```

---

## 🍎 Build for macOS

### Intel (x86_64):
```bash
rustup target add x86_64-apple-darwin
cross build --release --target x86_64-apple-darwin
```

### Apple Silicon (ARM64):
```bash
rustup target add aarch64-apple-darwin
cross build --release --target aarch64-apple-darwin
```

---

## 🐧 Build for Linux (Other Architectures)

### ARM64 (Raspberry Pi, etc.):
```bash
cross build --release --target aarch64-unknown-linux-gnu
```

### ARMv7 (Older Raspberry Pi):
```bash
cross build --release --target armv7-unknown-linux-gnueabihf
```

---

## 📊 All Supported Targets

| Platform | Target | Command |
|----------|--------|---------|
| Linux x64 | `x86_64-unknown-linux-gnu` | `cargo build --release` |
| Windows x64 | `x86_64-pc-windows-gnu` | `cross build --release --target x86_64-pc-windows-gnu` |
| Windows x64 (MSVC) | `x86_64-pc-windows-msvc` | `cross build --release --target x86_64-pc-windows-msvc` |
| macOS x64 | `x86_64-apple-darwin` | `cross build --release --target x86_64-apple-darwin` |
| macOS ARM64 | `aarch64-apple-darwin` | `cross build --release --target aarch64-apple-darwin` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `cross build --release --target aarch64-unknown-linux-gnu` |
| Linux ARMv7 | `armv7-unknown-linux-gnueabihf` | `cross build --release --target armv7-unknown-linux-gnueabihf` |

---

## 🚀 Quick Start

### 1. Install cross:
```bash
cargo install cross
```

### 2. Build for Windows:
```bash
./build-windows.sh
```

### 3. Find your binary:
```bash
ls -lh target/x86_64-pc-windows-gnu/release/dbi-backend-rust.exe
```

---

## 🔧 Troubleshooting

### Docker permission denied:
```bash
sudo usermod -aG docker $USER
# Log out and log back in
```

### Cross build fails:
```bash
# Update cross
cargo install cross --force

# Update Docker images
docker pull ghcr.io/cross-rs/x86_64-pc-windows-gnu:latest
```

### Missing dependencies:
```bash
# Linux
sudo apt-get install libusb-1.0-0-dev pkg-config

# Windows cross-compile
sudo apt-get install mingw-w64
```

---

## 📦 GitHub Actions

Automatic builds for all platforms on every release:

1. Create a tag:
```bash
git tag v0.1.0
git push origin v0.1.0
```

2. GitHub Actions will automatically:
   - Build for Linux, Windows, macOS (x64 + ARM64)
   - Create release with all binaries
   - Upload artifacts

See `.github/workflows/release.yml` for details.

---

## 🎯 Testing Windows Build on Linux

### Using Wine:
```bash
# Install Wine
sudo apt-get install wine64

# Run Windows binary
wine target/x86_64-pc-windows-gnu/release/dbi-backend-rust.exe
```

**Note**: GUI may not work properly in Wine. Best to test on real Windows.

---

## 📝 Notes

- **Windows builds** require `libusb` DLL (included in release)
- **macOS builds** require code signing for distribution
- **ARM builds** may have performance differences
- **Cross-compilation** is faster than building on each platform

---

## 🔗 Resources

- [cross documentation](https://github.com/cross-rs/cross)
- [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
- [rusb Windows Guide](https://github.com/a1ien/rusb#windows)
