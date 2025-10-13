#!/bin/bash
# Build DBI Backend for Windows on Linux

set -e

echo "🪟 Building DBI Backend for Windows..."
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if cross is installed
if ! command -v cross &> /dev/null; then
    echo -e "${YELLOW}Installing cross...${NC}"
    cargo install cross
fi

# Build for Windows 64-bit
echo -e "${BLUE}📦 Building for Windows x64 (GNU)...${NC}"
cross build --release --target x86_64-pc-windows-gnu

# Check if build was successful
if [ -f "target/x86_64-pc-windows-gnu/release/dbi-backend-rust.exe" ]; then
    SIZE=$(du -h target/x86_64-pc-windows-gnu/release/dbi-backend-rust.exe | cut -f1)
    
    echo ""
    echo -e "${GREEN}✅ Windows build complete!${NC}"
    echo ""
    echo "📊 Details:"
    echo "   File: target/x86_64-pc-windows-gnu/release/dbi-backend-rust.exe"
    echo "   Size: $SIZE"
    echo ""
    echo "🚀 To test on Windows:"
    echo "   1. Copy dbi-backend-rust.exe to Windows PC"
    echo "   2. Run: dbi-backend-rust.exe"
    echo ""
else
    echo -e "${YELLOW}❌ Build failed!${NC}"
    exit 1
fi
