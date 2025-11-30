# 🛠️ SwitchLink Development Setup Guide

Complete guide to set up Nintendo Switch homebrew development environment on Fedora Linux.

## 📋 Prerequisites

- Fedora Linux (tested on Fedora 43)
- Internet connection
- At least 2GB free disk space

## 🚀 Installation Steps

### Step 1: Install devkitPro (Manual Method for Fedora)

Since devkitPro doesn't have official Fedora packages, we'll install it manually:

```bash
# Create installation directory
sudo mkdir -p /opt/devkitpro
sudo chown $USER:$USER /opt/devkitpro

# Download devkitPro pacman
cd /tmp
wget https://github.com/devkitPro/pacman/releases/download/v1.0.2/devkitpro-pacman-1.0.2-1.pkg.tar.xz

# Extract
tar -xf devkitpro-pacman-1.0.2-1.pkg.tar.xz -C /

# Or use alternative method - install from source
git clone https://github.com/devkitPro/pacman.git
cd pacman
# Follow build instructions
```

### Step 2: Alternative - Use Docker (Recommended for Fedora)

This is the easiest method for Fedora users:

```bash
# Install Docker if not already installed
sudo dnf install -y docker
sudo systemctl start docker
sudo systemctl enable docker
sudo usermod -aG docker $USER

# Log out and log back in for group changes to take effect

# Pull devkitPro Docker image
docker pull devkitpro/devkita64

# Build SwitchLink client
cd /home/dit/Sites/dbi-backend-gui/switch-client
docker run --rm -v $(pwd):/src devkitpro/devkita64 make

# Output will be: switchlink-client.nro
```

### Step 3: Install Dependencies (If building natively)

```bash
# Install build tools
sudo dnf groupinstall "Development Tools"
sudo dnf install -y cmake git wget curl

# Install Switch-specific tools
export DEVKITPRO=/opt/devkitpro
export DEVKITARM=/opt/devkitpro/devkitARM
export DEVKITPPC=/opt/devkitpro/devkitPPC

# Add to ~/.bashrc or ~/.zshrc
echo 'export DEVKITPRO=/opt/devkitpro' >> ~/.bashrc
echo 'export DEVKITARM=/opt/devkitpro/devkitARM' >> ~/.bashrc
echo 'export DEVKITPPC=/opt/devkitpro/devkitPPC' >> ~/.bashrc
echo 'export PATH=$DEVKITPRO/tools/bin:$PATH' >> ~/.bashrc

source ~/.bashrc
```

### Step 4: Install devkitA64 and libnx

Using dkp-pacman (if installed):

```bash
sudo dkp-pacman -S switch-dev
sudo dkp-pacman -S devkitA64
sudo dkp-pacman -S libnx
sudo dkp-pacman -S switch-tools
```

Or using Docker (easier):

```bash
# Everything is included in the Docker image!
# Just use: docker run --rm -v $(pwd):/src devkitpro/devkita64 make
```

## 🔨 Building SwitchLink Client

### Method 1: Using Docker (Recommended)

```bash
cd /home/dit/Sites/dbi-backend-gui/switch-client

# Build
docker run --rm -v $(pwd):/src devkitpro/devkita64 make

# Clean
docker run --rm -v $(pwd):/src devkitpro/devkita64 make clean

# Output: switchlink-client.nro
```

### Method 2: Native Build (If devkitPro is installed)

```bash
cd /home/dit/Sites/dbi-backend-gui/switch-client

# Build
make

# Clean
make clean

# Output: switchlink-client.nro
```

## 📦 Testing on Switch

1. **Copy to SD Card**:

   ```bash
   # Mount your Switch SD card
   sudo mkdir -p /mnt/switch
   sudo mount /dev/sdX1 /mnt/switch  # Replace sdX1 with your SD card device

   # Copy the .nro file
   sudo cp switchlink-client.nro /mnt/switch/switch/

   # Unmount
   sudo umount /mnt/switch
   ```

2. **Launch on Switch**:
   - Insert SD card into Switch
   - Boot into CFW (Atmosphère)
   - Open Homebrew Menu
   - Find "SwitchLink USB Installer"
   - Launch it!

## 🐛 Troubleshooting

### Docker permission denied

```bash
sudo usermod -aG docker $USER
# Log out and log back in
```

### devkitPro not found

```bash
# Use Docker method instead - it's easier!
docker pull devkitpro/devkita64
```

### Build fails with missing headers

```bash
# Make sure you're using the Docker image
# It has everything pre-installed
docker run --rm -v $(pwd):/src devkitpro/devkita64 make clean
docker run --rm -v $(pwd):/src devkitpro/devkita64 make
```

### Switch doesn't detect USB

- Use a good quality USB-C cable (data transfer capable)
- Try different USB ports on your PC
- Make sure Switch is in homebrew mode
- Check that SwitchLink Backend is running on PC

## 📝 Quick Build Script

Create a build script for convenience:

```bash
#!/bin/bash
# build-switch.sh

cd "$(dirname "$0")"

echo "🔨 Building SwitchLink Client..."
docker run --rm -v $(pwd):/src devkitpro/devkita64 make

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo "📦 Output: switchlink-client.nro"
    ls -lh switchlink-client.nro
else
    echo "❌ Build failed!"
    exit 1
fi
```

Make it executable:

```bash
chmod +x build-switch.sh
./build-switch.sh
```

## 🎯 Next Steps

After building successfully:

1. ✅ Copy `switchlink-client.nro` to Switch SD card
2. ✅ Update PC backend to support new protocol (SWLK magic)
3. ✅ Test USB connection
4. ✅ Benchmark transfer speeds
5. ✅ Add more features (WiFi, cloud storage, etc.)

## 📚 Resources

- [devkitPro Documentation](https://devkitpro.org/wiki/Getting_Started)
- [libnx Documentation](https://switchbrew.org/wiki/Homebrew_Development)
- [Switch Homebrew Guide](https://switch.homebrew.guide/)
- [Atmosphère CFW](https://github.com/Atmosphere-NX/Atmosphere)

---

**Happy Hacking! 🎮**
