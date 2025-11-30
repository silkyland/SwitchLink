# 🔧 Nintendo Switch Connection Troubleshooting Guide

## ✅ การแก้ปัญหาที่ทำไปแล้ว

### 1. ตรวจสอบว่า Switch เชื่อมต่ออยู่

```bash
lsusb | grep -i nintendo
```

**ผลลัพธ์ที่ควรเห็น:**

```
Bus 001 Device 010: ID 057e:3000 Nintendo Co., Ltd SDK Debugger
```

### 2. ตั้งค่า USB Permissions

สร้างไฟล์ `/etc/udev/rules.d/99-nintendo-switch.rules`:

```bash
# Nintendo Switch USB Rules for DBI Backend
SUBSYSTEM=="usb", ATTR{idVendor}=="057e", ATTR{idProduct}=="3000", MODE="0666"
SUBSYSTEM=="usb", ATTR{idVendor}=="057e", MODE="0666"
```

**วิธีติดตั้ง:**

```bash
sudo cp 99-nintendo-switch.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### 3. ตรวจสอบ Permissions

```bash
ls -l /dev/bus/usb/001/010
```

**ผลลัพธ์ที่ถูกต้อง:**

```
crw-rw-rw-. 1 root root 189, 9 Nov 30 08:38 /dev/bus/usb/001/010
```

- ต้องเป็น `rw-rw-rw-` (0666) เพื่อให้ทุกคนเข้าถึงได้

## 🔍 การตรวจสอบปัญหา

### ขั้นตอนที่ 1: ตรวจสอบ USB Connection

```bash
# ดู USB devices ทั้งหมด
lsusb

# ดูเฉพาะ Nintendo Switch
lsusb | grep -i nintendo

# ดู USB tree
lsusb -t
```

### ขั้นตอนที่ 2: ตรวจสอบ Logs

```bash
# รันโปรแกรมพร้อม logging
RUST_LOG=info ./target/release/dbi-backend-rust 2>&1 | tee dbi.log

# ดู logs แบบ real-time
tail -f dbi.log

# ค้นหา errors
grep -i error dbi.log
```

### ขั้นตอนที่ 3: ตรวจสอบ DBI บน Switch

1. เปิด DBI บน Switch
2. ไปที่ **"Run MTP responder"** หรือ **"Install title from DBIbackend"**
3. ตรวจสอบว่า Switch แสดง "Waiting for connection..."

## 🐛 ปัญหาที่พบบ่อยและวิธีแก้

### ❌ ปัญหา: "Nintendo Switch not found"

**สาเหตุ:**

- Switch ไม่ได้เชื่อมต่อผ่าน USB
- DBI ไม่ได้เปิดบน Switch
- USB cable ไม่รองรับ data transfer

**วิธีแก้:**

1. ตรวจสอบ USB cable (ต้องเป็น cable ที่รองรับ data)
2. ลอง USB port อื่น
3. เปิด DBI บน Switch ใหม่
4. Unplug และ plug Switch กลับเข้าไปใหม่
5. รัน `lsusb | grep -i nintendo` เพื่อยืนยันว่าเจอ Switch

### ❌ ปัญหา: "Permission denied" หรือ "Access denied"

**สาเหตุ:**

- USB permissions ไม่ถูกต้อง
- udev rules ยังไม่ได้ติดตั้ง

**วิธีแก้:**

```bash
# 1. ติดตั้ง udev rules
sudo cp 99-nintendo-switch.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger

# 2. Unplug และ plug Switch กลับเข้าไปใหม่

# 3. ตรวจสอบ permissions
ls -l /dev/bus/usb/001/010  # เปลี่ยน 001/010 ตาม lsusb

# 4. ถ้ายังไม่ได้ ลองรัน sudo
sudo ./target/release/dbi-backend-rust
```

### ❌ ปัญหา: "Connection timeout"

**สาเหตุ:**

- Switch ไม่ได้อยู่ในโหมด DBI
- USB cable มีปัญหา
- Timeout สั้นเกินไป

**วิธีแก้:**

1. ตรวจสอบว่า DBI เปิดอยู่บน Switch
2. ลอง USB cable อื่น
3. ลอง USB port อื่น (USB 3.0 ดีกว่า USB 2.0)
4. รีสตาร์ท DBI บน Switch

### ❌ ปัญหา: "USB reset failed"

**สาเหตุ:**

- ปกติ ไม่ต้องกังวล (โค้ดจัดการให้แล้ว)

**วิธีแก้:**

- ไม่ต้องทำอะไร โปรแกรมจะ retry อัตโนมัติ

### ❌ ปัญหา: "Transfer speed very slow"

**สาเหตุ:**

- USB 2.0 port (ช้ากว่า USB 3.0)
- USB cable คุณภาพต่ำ
- Background processes ใช้ CPU มาก

**วิธีแก้:**

1. ใช้ USB 3.0 port (สีน้ำเงิน)
2. ใช้ USB cable คุณภาพดี (สาย original ของ Switch)
3. ปิด background applications
4. ตรวจสอบ CPU usage: `htop`

## 📋 Checklist ก่อนเริ่มใช้งาน

- [ ] ติดตั้ง libusb แล้ว (`sudo apt-get install libusb-1.0-0-dev`)
- [ ] ติดตั้ง udev rules แล้ว
- [ ] Reload udev rules แล้ว
- [ ] Switch เชื่อมต่อผ่าน USB
- [ ] DBI เปิดอยู่บน Switch
- [ ] USB cable รองรับ data transfer
- [ ] ใช้ USB 3.0 port (ถ้ามี)
- [ ] เพิ่มไฟล์เข้า queue แล้ว

## 🔬 Advanced Debugging

### ดู USB Traffic

```bash
# ติดตั้ง usbmon
sudo modprobe usbmon

# ดู USB traffic
sudo cat /sys/kernel/debug/usb/usbmon/1u

# หรือใช้ wireshark
sudo wireshark
# เลือก usbmon1 interface
```

### ดู Detailed Logs

```bash
# เปิด debug logging
RUST_LOG=debug ./target/release/dbi-backend-rust 2>&1 | tee dbi-debug.log

# ดู trace logging (มาก)
RUST_LOG=trace ./target/release/dbi-backend-rust 2>&1 | tee dbi-trace.log
```

### ตรวจสอบ USB Endpoints

```bash
# ดู USB device details
lsusb -v -d 057e:3000

# ดู endpoints
lsusb -v -d 057e:3000 | grep -A 5 "Endpoint"
```

## 🚀 การทดสอบ Connection

### Test Script

```bash
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
DEVICE=$(lsusb | grep "057e:3000" | awk '{print "/dev/bus/usb/"$2"/"$4}' | sed 's/://')
if [ -r "$DEVICE" ] && [ -w "$DEVICE" ]; then
    echo "✅ Permissions OK!"
    ls -l "$DEVICE"
else
    echo "❌ Permission denied!"
    echo "   Run: sudo cp 99-nintendo-switch.rules /etc/udev/rules.d/"
    echo "   Then: sudo udevadm control --reload-rules && sudo udevadm trigger"
    exit 1
fi

echo ""

# 3. Check udev rules
echo "3. Checking udev rules..."
if [ -f "/etc/udev/rules.d/99-nintendo-switch.rules" ]; then
    echo "✅ udev rules installed!"
else
    echo "⚠️  udev rules NOT found!"
    echo "   Run: sudo cp 99-nintendo-switch.rules /etc/udev/rules.d/"
fi

echo ""
echo "🎉 All checks passed! Ready to use DBI Backend!"
```

บันทึกเป็น `test-connection.sh` และรัน:

```bash
chmod +x test-connection.sh
./test-connection.sh
```

## 📞 ขอความช่วยเหลือ

ถ้ายังแก้ไม่ได้ ให้รวบรวมข้อมูลเหล่านี้:

1. **System Info:**

   ```bash
   uname -a
   lsb_release -a
   ```

2. **USB Info:**

   ```bash
   lsusb | grep -i nintendo
   lsusb -v -d 057e:3000
   ```

3. **Permissions:**

   ```bash
   ls -l /dev/bus/usb/001/010  # เปลี่ยนตาม lsusb
   cat /etc/udev/rules.d/99-nintendo-switch.rules
   ```

4. **Logs:**
   ```bash
   RUST_LOG=debug ./target/release/dbi-backend-rust 2>&1 | tee dbi-debug.log
   # รอ error เกิดขึ้น แล้วส่ง dbi-debug.log
   ```

## 🎯 Quick Fix Commands

```bash
# แก้ปัญหาทั่วไป (รันทีละบรรทัด)
sudo cp 99-nintendo-switch.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger

# Unplug และ plug Switch กลับเข้าไปใหม่

# ตรวจสอบว่าเจอ Switch
lsusb | grep -i nintendo

# ตรวจสอบ permissions
ls -l /dev/bus/usb/001/010  # เปลี่ยนตาม lsusb

# รันโปรแกรมพร้อม logging
RUST_LOG=info ./target/release/dbi-backend-rust 2>&1 | tee dbi.log
```

---

**หมายเหตุ:** ถ้าทำตาม guide นี้แล้วยังไม่ได้ ให้ลอง:

1. รีสตาร์ทคอมพิวเตอร์
2. ใช้ USB cable อื่น
3. ลอง USB port อื่น
4. อัปเดต DBI บน Switch เป็นเวอร์ชันล่าสุด
5. รัน `sudo ./target/release/dbi-backend-rust` (ชั่วคราว)
