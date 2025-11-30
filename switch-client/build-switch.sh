#!/bin/bash
# SwitchLink Client Build Script
# Builds the Switch homebrew client using Docker

set -e

cd "$(dirname "$0")"

echo "🎮 SwitchLink Client Builder"
echo "=============================="
echo ""

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed!"
    echo "Please install Docker first:"
    echo "  sudo dnf install -y docker"
    echo "  sudo systemctl start docker"
    echo "  sudo usermod -aG docker \$USER"
    exit 1
fi

# Check if Docker is running
if ! docker info &> /dev/null; then
    echo "❌ Docker is not running!"
    echo "Please start Docker:"
    echo "  sudo systemctl start docker"
    exit 1
fi

# Pull latest devkitPro image if needed
echo "📥 Checking for devkitPro Docker image..."
if ! docker images | grep -q "devkitpro/devkita64"; then
    echo "Pulling devkitPro image (this may take a while)..."
    docker pull devkitpro/devkita64
fi

# Get absolute path
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Clean previous build
echo ""
echo "🧹 Cleaning previous build..."
docker run --rm -v "${SCRIPT_DIR}:/project" -w /project devkitpro/devkita64 make clean 2>/dev/null || true

# Build
echo ""
echo "🔨 Building SwitchLink Client..."
echo ""
docker run --rm -v "${SCRIPT_DIR}:/project" -w /project devkitpro/devkita64 make

# Check if build was successful
if [ -f "switchlink-client.nro" ]; then
    echo ""
    echo "✅ Build successful!"
    echo ""
    echo "📦 Output file:"
    ls -lh switchlink-client.nro
    echo ""
    echo "📋 Next steps:"
    echo "  1. Copy switchlink-client.nro to /switch/ folder on your SD card"
    echo "  2. Launch from Homebrew Menu on your Switch"
    echo "  3. Connect Switch to PC via USB"
    echo "  4. Start SwitchLink Backend on PC"
    echo ""
else
    echo ""
    echo "❌ Build failed!"
    exit 1
fi
