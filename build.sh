#!/bin/bash
# Build script for DBI Backend Rust Edition

set -e

echo "🦀 Building DBI Backend - Rust Edition"
echo "======================================"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install from https://rustup.rs/"
    exit 1
fi

# Check if libusb is available
if ! pkg-config --exists libusb-1.0; then
    echo "⚠️  Warning: libusb-1.0 not found via pkg-config"
    echo "   You may need to install libusb development files:"
    echo "   - Ubuntu/Debian: sudo apt-get install libusb-1.0-0-dev"
    echo "   - Fedora: sudo dnf install libusb1-devel"
    echo "   - macOS: brew install libusb"
fi

echo ""
echo "📦 Building in release mode..."
cargo build --release

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Build successful!"
    echo ""
    echo "📍 Binary location: target/release/dbi-backend-rust"
    echo "📊 Binary size: $(du -h target/release/dbi-backend-rust | cut -f1)"
    echo ""
    echo "🚀 To run: ./target/release/dbi-backend-rust"
    echo "   or:     cargo run --release"
else
    echo ""
    echo "❌ Build failed. Please check the errors above."
    exit 1
fi
