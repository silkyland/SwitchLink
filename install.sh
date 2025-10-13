#!/bin/bash
# DBI Backend Rust - Installation Script

set -e

echo "🦀 Installing DBI Backend (Rust Edition)..."
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}⚠️  Cargo not found. Please install Rust first:${NC}"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build release version
echo -e "${BLUE}📦 Building release version...${NC}"
cargo build --release

# Check if build was successful
if [ ! -f "target/release/dbi-backend-rust" ]; then
    echo -e "${YELLOW}❌ Build failed!${NC}"
    exit 1
fi

# Create local bin directory if it doesn't exist
mkdir -p ~/.local/bin

# Copy executable
echo -e "${BLUE}📋 Copying executable to ~/.local/bin/dbi${NC}"
cp target/release/dbi-backend-rust ~/.local/bin/dbi
chmod +x ~/.local/bin/dbi

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo ""
    echo -e "${YELLOW}⚠️  ~/.local/bin is not in your PATH${NC}"
    echo "   Add this line to your ~/.bashrc or ~/.zshrc:"
    echo ""
    echo "   export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
fi

# Get file size
SIZE=$(du -h ~/.local/bin/dbi | cut -f1)

echo ""
echo -e "${GREEN}✅ Installation complete!${NC}"
echo ""
echo "📊 Details:"
echo "   Location: ~/.local/bin/dbi"
echo "   Size: $SIZE"
echo ""
echo "🚀 Usage:"
echo "   dbi              # Launch GUI"
echo "   dbi --cli        # Launch CLI"
echo ""
echo "🎮 Ready to transfer games to your Nintendo Switch!"
