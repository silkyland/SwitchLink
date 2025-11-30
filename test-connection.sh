#!/bin/bash

echo "🔍 Testing Nintendo Switch Connection..."
echo ""

# 1. Check if Switch is connected
echo "1. Checking USB connection..."
if lsusb | grep -q "057e:3000"; then
    echo "✅ Nintendo Switch found!"
    lsusb | grep "057e:3000"
else
    echo "❌ Nintendo Switch NOT found!"
    echo "   Please connect your Switch and open DBI"
    exit 1
fi

echo ""

# 2. Check permissions
echo "2. Checking USB permissions..."
BUS=$(lsusb | grep "057e:3000" | awk '{print $2}')
DEV=$(lsusb | grep "057e:3000" | awk '{print $4}' | sed 's/://')
DEVICE="/dev/bus/usb/$BUS/$DEV"

if [ -r "$DEVICE" ] && [ -w "$DEVICE" ]; then
    echo "✅ Permissions OK!"
    ls -l "$DEVICE"
else
    echo "❌ Permission denied!"
    echo "   Device: $DEVICE"
    ls -l "$DEVICE" 2>/dev/null || echo "   Device not found!"
    echo ""
    echo "   Fix: sudo cp 99-nintendo-switch.rules /etc/udev/rules.d/"
    echo "        sudo udevadm control --reload-rules && sudo udevadm trigger"
    echo "        Then unplug and replug your Switch"
    exit 1
fi

echo ""

# 3. Check udev rules
echo "3. Checking udev rules..."
if [ -f "/etc/udev/rules.d/99-nintendo-switch.rules" ]; then
    echo "✅ udev rules installed!"
    echo "   Content:"
    cat /etc/udev/rules.d/99-nintendo-switch.rules | grep -v "^#" | grep -v "^$"
else
    echo "⚠️  udev rules NOT found!"
    echo "   Run: sudo cp 99-nintendo-switch.rules /etc/udev/rules.d/"
fi

echo ""

# 4. Check if DBI Backend is built
echo "4. Checking DBI Backend binary..."
if [ -f "./target/release/dbi-backend-rust" ]; then
    echo "✅ Binary found!"
    ls -lh ./target/release/dbi-backend-rust
else
    echo "❌ Binary NOT found!"
    echo "   Run: cargo build --release"
    exit 1
fi

echo ""

# 5. Check libusb
echo "5. Checking libusb..."
if ldconfig -p | grep -q libusb; then
    echo "✅ libusb installed!"
else
    echo "⚠️  libusb might not be installed!"
    echo "   Run: sudo apt-get install libusb-1.0-0-dev"
fi

echo ""
echo "🎉 All checks passed! Ready to use DBI Backend!"
echo ""
echo "Next steps:"
echo "1. Open DBI on your Switch"
echo "2. Go to 'Install title from DBIbackend'"
echo "3. Run: ./target/release/dbi-backend-rust"
echo "4. Add files and click 'Start Server'"
