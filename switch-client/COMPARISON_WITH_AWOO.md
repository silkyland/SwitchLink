# 📊 เปรียบเทียบ SwitchLink Client กับ Awoo Installer

## 🎯 สรุปการเปรียบเทียบ

### ✅ **สิ่งที่เรามีแล้ว (ใช้งานได้)**

| Feature                       | SwitchLink | Awoo Installer | Status                          |
| ----------------------------- | ---------- | -------------- | ------------------------------- |
| **NSP Installation**          | ✅         | ✅             | เหมือนกัน                       |
| **USB Streaming**             | ✅         | ✅             | เหมือนกัน                       |
| **PFS0 Parsing**              | ✅         | ✅             | เหมือนกัน                       |
| **CNMT Reading**              | ✅         | ✅             | เหมือนกัน                       |
| **NCA Installation**          | ✅         | ✅             | เหมือนกัน                       |
| **Content Meta Registration** | ✅         | ✅             | เหมือนกัน                       |
| **Application Record Push**   | ✅         | ✅             | **แก้ไขแล้ว** (ไม่ลบ base game) |
| **Progress Callback**         | ✅         | ✅             | เหมือนกัน                       |
| **Error Handling**            | ✅         | ✅             | เหมือนกัน                       |

---

### ⚠️ **สิ่งที่เราขาด (แต่ไม่จำเป็น)**

| Feature                        | SwitchLink | Awoo Installer | ความสำคัญ  | หมายเหตุ                      |
| ------------------------------ | ---------- | -------------- | ---------- | ----------------------------- |
| **Ticket Installation**        | ❌         | ✅             | 🟡 ปานกลาง | ต้องการ custom ES IPC wrapper |
| **XCI Support**                | ❌         | ✅             | 🟢 ต่ำ     | NSP เพียงพอ                   |
| **NSZ Support**                | ❌         | ✅             | 🟢 ต่ำ     | ต้องการ decompression         |
| **Network Install**            | ❌         | ✅             | 🟢 ต่ำ     | มี USB แล้ว                   |
| **SD Card Install**            | ❌         | ✅             | 🟢 ต่ำ     | มี USB แล้ว                   |
| **NCA Signature Verification** | ❌         | ✅             | 🟡 ปานกลาง | ต้องการ crypto library        |
| **Signature Patches Install**  | ❌         | ✅             | 🟢 ต่ำ     | ผู้ใช้ติดตั้งเอง              |
| **GUI (Plutonium)**            | ❌         | ✅             | 🟢 ต่ำ     | เรามี console UI              |
| **Multi-language**             | ❌         | ✅             | 🟢 ต่ำ     | ภาษาอังกฤษเพียงพอ             |

---

## 🔍 รายละเอียดการเปรียบเทียบ

### **1. Ticket Installation**

#### **Awoo Installer:**

```c
// include/nx/ipc/es.h
Result esInitialize(void);
void esExit(void);
Result esImportTicket(void const *tikBuf, size_t tikSize,
                      void const *certBuf, size_t certSize);

// source/nx/ipc/es.c
static Service g_esSrv;

Result esInitialize(void) {
    return smGetService(&g_esSrv, "es");
}

Result esImportTicket(void const *tikBuf, size_t tikSize,
                      void const *certBuf, size_t certSize) {
    return serviceDispatch(&g_esSrv, 1,
        .buffer_attrs = {
            SfBufferAttr_HipcMapAlias | SfBufferAttr_In,
            SfBufferAttr_HipcMapAlias | SfBufferAttr_In,
        },
        .buffers = {
            { tikBuf,   tikSize },
            { certBuf,  certSize },
        },
    );
}
```

#### **SwitchLink:**

```cpp
// ปัจจุบัน: ข้าม ticket installation
// เหตุผล: ผู้ใช้ส่วนใหญ่มี sigpatches อยู่แล้ว
printf("NOTE: Ticket installation is skipped.\n");
printf("This installer assumes you have sigpatches installed.\n");
```

**ข้อดี:**

- ✅ ง่ายกว่า ไม่ต้องจัดการ ES service
- ✅ ทำงานได้กับ sigpatches (ผู้ใช้ส่วนใหญ่)

**ข้อเสีย:**

- ❌ เกมที่ซื้อมาอาจเล่นไม่ได้ถ้าไม่มี sigpatches
- ❌ ไม่สมบูรณ์เท่า Awoo

---

### **2. Application Record Registration**

#### **Awoo Installer:**

```c
// include/nx/ipc/ns_ext.h
typedef enum {
    NsApplicationRecordType_Installed       = 0x3,
    NsApplicationRecordType_GamecardMissing = 0x5,
    NsApplicationRecordType_Archived        = 0xB,
} NsApplicationRecordType;

typedef struct {
    NcmContentMetaKey metaRecord;
    u64 storageId;
} ContentStorageRecord;

Result nsPushApplicationRecord(u64 application_id,
                                NsApplicationRecordType last_modified_event,
                                ContentStorageRecord *content_records,
                                u32 count);
```

#### **SwitchLink (แก้ไขแล้ว):**

```cpp
// ใช้ low-level IPC call โดยตรง
struct ContentStorageRecord {
    NcmContentMetaKey key;
    u8 storage_id;
    u8 padding[7];
} __attribute__((packed)) storageRecord;

// CRITICAL: ไม่ลบ application record เดิม!
rc = serviceDispatchIn(&appManSrv, 16, pushIn,
    .buffer_attrs = { SfBufferAttr_HipcMapAlias | SfBufferAttr_In },
    .buffers = { { &storageRecord, sizeof(storageRecord) } },
);
```

**สถานะ:** ✅ **แก้ไขแล้ว** - ทำงานเหมือน Awoo

---

### **3. XCI Support**

#### **Awoo Installer:**

- รองรับ XCI (Cartridge dumps)
- มี `install_xci.cpp`, `xci.cpp`, `hfs0.hpp`
- ซับซ้อนกว่า NSP

#### **SwitchLink:**

- ❌ ไม่รองรับ XCI
- เน้น NSP เท่านั้น

**ความจำเป็น:** 🟢 **ต่ำ**

- NSP เป็น format มาตรฐานสำหรับ digital games
- XCI ใช้สำหรับ cartridge dumps (น้อยกว่า)

---

### **4. NSZ Support (Compressed NSP)**

#### **Awoo Installer:**

- รองรับ NSZ (NSP ที่บีบอัด)
- ใช้ zstd decompression
- ติดตั้งได้เร็วกว่า (ไฟล์เล็กกว่า)

#### **SwitchLink:**

- ❌ ไม่รองรับ NSZ
- รองรับแค่ NSP ปกติ

**ความจำเป็น:** 🟢 **ต่ำ**

- NSP ปกติใช้งานได้ดี
- NSZ ต้องการ decompression library เพิ่ม

---

### **5. NCA Signature Verification**

#### **Awoo Installer:**

```cpp
// util/crypto.hpp
class Crypto {
public:
    static bool rsa2048PssVerify(const void* data, size_t size,
                                  const void* signature,
                                  const void* modulus);
};

// ตรวจสอบ NCA header signature
if (!Crypto::rsa2048PssVerify(&header->magic, 0x200,
                               header->fixed_key_sig,
                               Crypto::NCAHeaderSignature)) {
    // แจ้งเตือนผู้ใช้ว่า NCA ถูกแก้ไข
}
```

#### **SwitchLink:**

```cpp
// ไม่มีการตรวจสอบ signature
// ติดตั้งโดยตรง
```

**ความจำเป็น:** 🟡 **ปานกลาง**

- ✅ ช่วยตรวจสอบความถูกต้องของไฟล์
- ❌ ต้องการ crypto library (mbedtls)
- ❌ ซับซ้อน

---

### **6. Installation Sources**

| Source             | SwitchLink | Awoo Installer |
| ------------------ | ---------- | -------------- |
| **USB**            | ✅         | ✅             |
| **SD Card**        | ❌         | ✅             |
| **Network (HTTP)** | ❌         | ✅             |
| **Google Drive**   | ❌         | ✅             |

**ความจำเป็น:** 🟢 **ต่ำ**

- USB เพียงพอสำหรับ use case ของเรา
- Network install ช้ากว่า USB
- SD card install ไม่จำเป็น (มี USB แล้ว)

---

### **7. User Interface**

#### **Awoo Installer:**

- ใช้ Plutonium (GUI framework)
- มี touch support
- มี icons, images, animations
- Multi-language support

#### **SwitchLink:**

- Console-based UI (ANSI colors)
- Keyboard/controller only
- ภาษาอังกฤษเท่านั้น

**ความจำเป็น:** 🟢 **ต่ำ**

- Console UI ทำงานได้ดี
- ไม่ต้องการ GUI framework ซับซ้อน

---

## 🛠️ สิ่งที่ควรเพิ่ม (ถ้ามีเวลา)

### **1. Ticket Installation (Priority: Medium)**

**ทำไมควรเพิ่ม:**

- ✅ ทำให้สมบูรณ์กว่า
- ✅ รองรับเกมที่ซื้อมา (ไม่มี sigpatches)

**วิธีเพิ่ม:**

1. Copy `include/nx/ipc/es.h` และ `source/nx/ipc/es.c` จาก Awoo
2. เพิ่มใน Makefile
3. แก้ไข `installTicketCert()` ให้ใช้ `esImportTicket()`

**โค้ดตัวอย่าง:**

```cpp
// เพิ่มไฟล์ใหม่: include/es_wrapper.h
#pragma once
#include <switch.h>

Result esInitialize(void);
void esExit(void);
Result esImportTicket(void const *tikBuf, size_t tikSize,
                      void const *certBuf, size_t certSize);

// เพิ่มไฟล์ใหม่: source/es_wrapper.c
#include "es_wrapper.h"
#include <string.h>

static Service g_esSrv;

Result esInitialize(void) {
    return smGetService(&g_esSrv, "es");
}

void esExit(void) {
    serviceClose(&g_esSrv);
}

Result esImportTicket(void const *tikBuf, size_t tikSize,
                      void const *certBuf, size_t certSize) {
    return serviceDispatch(&g_esSrv, 1,
        .buffer_attrs = {
            SfBufferAttr_HipcMapAlias | SfBufferAttr_In,
            SfBufferAttr_HipcMapAlias | SfBufferAttr_In,
        },
        .buffers = {
            { tikBuf,   tikSize },
            { certBuf,  certSize },
        },
    );
}
```

---

### **2. NCA Signature Verification (Priority: Low)**

**ทำไมควรเพิ่ม:**

- ✅ ตรวจสอบความถูกต้องของไฟล์
- ✅ ป้องกันไฟล์เสียหาย

**วิธีเพิ่ม:**

1. เพิ่ม mbedtls library
2. Implement RSA-2048 PSS verification
3. ตรวจสอบ NCA header ก่อนติดตั้ง

**ความซับซ้อน:** สูง (ต้องการ crypto library)

---

### **3. NSZ Support (Priority: Low)**

**ทำไมควรเพิ่ม:**

- ✅ ไฟล์เล็กกว่า (ประหยัด bandwidth)
- ✅ ติดตั้งเร็วกว่า

**วิธีเพิ่ม:**

1. เพิ่ม zstd decompression library
2. Decompress on-the-fly ระหว่างติดตั้ง

**ความซับซ้อน:** ปานกลาง

---

## 📋 สรุปสุดท้าย

### **สิ่งที่เราทำได้ดีแล้ว:**

✅ NSP installation ทำงานถูกต้อง
✅ USB streaming มีประสิทธิภาพ
✅ Application record registration แก้ไขแล้ว (ไม่ทับ base game)
✅ Progress tracking แม่นยำ
✅ Error handling ครบถ้วน

### **สิ่งที่ขาด (แต่ไม่จำเป็นมาก):**

🟡 Ticket installation (ทำงานได้ด้วย sigpatches)
🟢 XCI support (NSP เพียงพอ)
🟢 NSZ support (NSP ปกติใช้ได้)
🟢 Network install (USB เร็วกว่า)
🟢 NCA verification (ไม่จำเป็นมาก)

### **ข้อแนะนำ:**

**ถ้ามีเวลา 1-2 ชั่วโมง:**

- เพิ่ม Ticket Installation (เพิ่ม ES wrapper)

**ถ้ามีเวลา 4-6 ชั่วโมง:**

- เพิ่ม Ticket Installation
- เพิ่ม NCA Signature Verification

**ถ้ามีเวลามาก:**

- เพิ่มทุกอย่าง + NSZ support + XCI support

**แต่ปัจจุบัน:**
✅ **SwitchLink Client ใช้งานได้ดีแล้ว!**

- ติดตั้ง NSP ได้ถูกต้อง
- Base Game + Update + DLC อยู่ร่วมกันได้
- ทำงานกับ sigpatches (ผู้ใช้ส่วนใหญ่มี)
- UI ชัดเจน เข้าใจง่าย

---

## 🎮 การทดสอบที่แนะนำ

1. **ติดตั้ง Base Game** → ✅ ควรปรากฏใน Home Menu
2. **ติดตั้ง Update** → ✅ Base Game ยังอยู่ + แสดง version ใหม่
3. **ติดตั้ง DLC** → ✅ Base Game + Update ยังอยู่ + DLC ทำงาน
4. **ติดตั้ง Free Game** → ✅ ไม่มี ticket ก็เล่นได้
5. **ติดตั้ง Purchased Game (มี sigpatches)** → ✅ เล่นได้
6. **ติดตั้ง Purchased Game (ไม่มี sigpatches)** → ❌ อาจเล่นไม่ได้

---

## 📚 เอกสารอ้างอิง

- **Awoo Installer**: https://github.com/Huntereb/Awoo-Installer
- **Tinfoil (Base)**: https://github.com/Adubbz/Tinfoil
- **libnx**: https://switchbrew.github.io/libnx/
- **Switchbrew Wiki**: https://switchbrew.org/

---

**สรุป:** เราไม่ได้ขาดอะไรที่สำคัญมาก! ✨
