# 🔑 Keys และการติดตั้งเกม - คำอธิบายโดยละเอียด

## ❓ คำถามที่พบบ่อย

### 1. มันไม่ใช่แค่ transfer file ใช่ไหม?

**ใช่ครับ!** การทำงานมี 2 ขั้นตอน:

```
┌─────────────────────────────────────────────────────────┐
│  Step 1: Transfer File (USB)                            │
│  PC Backend ──USB──> Switch Client ──> /switch/downloads│
│                                                          │
│  Step 2: Install Game (NCM API)                         │
│  /switch/downloads/game.nsp ──NCM──> NAND/SD (installed)│
└─────────────────────────────────────────────────────────┘
```

#### Step 1: Transfer (ที่เราทำแล้ว)

- รับไฟล์ NSP/NSZ จาก PC ผ่าน USB
- เขียนไฟล์ลง SD card ที่ `/switch/downloads/`
- ความเร็ว 40-50 MB/s

#### Step 2: Install (ต้องเพิ่ม)

- เปิดไฟล์ NSP ด้วย `fs` API
- แตก NCA (Nintendo Content Archive) files
- ติดตั้งด้วย NCM (Nintendo Content Manager) API
- Verify signatures
- Register กับ system

### 2. ต้องมี prod.keys หรือ title.keys ไหม?

**ไม่ต้องครับ!** เพราะ:

#### Keys อยู่ที่ไหน?

```
SD Card Structure:
/atmosphere/
├── prod.keys          ← System keys (จำเป็น)
├── title.keys         ← Title-specific keys (optional)
└── contents/          ← Installed games
```

#### ใครจัดการ Keys?

```
┌──────────────┐
│ Your Client  │ ──calls──> ┌──────────┐
│ (SwitchLink) │            │ libnx    │
└──────────────┘            └──────────┘
                                  │
                            calls │
                                  ▼
                            ┌──────────┐
                            │ NCM API  │ ──reads──> ┌─────────────┐
                            │ (System) │            │ prod.keys   │
                            └──────────┘            │ (Atmosphère)│
                                                    └─────────────┘
```

**สรุป**: Atmosphère จัดการ keys ให้อัตโนมัติ เราไม่ต้องยุ่ง!

## 🛠️ การติดตั้ง NSP - Technical Details

### วิธีการติดตั้งที่ถูกต้อง

```cpp
// 1. Initialize NCM
ncmInitialize();
nsInitialize();

// 2. Open content storage
NcmContentStorage storage;
NcmStorageId storageId = NcmStorageId_SdCard; // หรือ NcmStorageId_BuiltInUser
ncmOpenContentStorage(&storage, storageId);

// 3. Open NSP file
FsFileSystem fs;
fsOpenFileSystemWithId(&fs, 0, FsFileSystemType_ApplicationPackage, path);

// 4. Read and install NCAs
// - Extract NCA files from NSP
// - Verify signatures (ใช้ keys จาก Atmosphère)
// - Install to content storage
ncmContentStorageRegister(&storage, &contentId, &placeholderId);

// 5. Register with system
nsCountApplicationContentMeta(&count);
nsListApplicationRecordContentMeta(&record);

// 6. Cleanup
ncmContentStorageClose(&storage);
ncmExit();
nsExit();
```

### ไฟล์ที่ต้องการ

#### บน Switch (SD Card):

```
/atmosphere/
├── prod.keys          ✅ จำเป็น (มีอยู่แล้วถ้าใช้ Atmosphère)
├── title.keys         ⚠️  Optional (สำหรับ encrypted titles)
└── contents/          ✅ จำเป็น (Atmosphère สร้างให้)

/switch/
├── switchlink-client.nro  ✅ Client ของเรา
└── downloads/             ✅ Temp folder สำหรับไฟล์ที่ download
```

#### บน PC:

```
/home/dit/Sites/dbi-backend-gui/
├── target/release/dbi-backend-rust  ✅ Backend
└── games/                            ✅ NSP/NSZ files
```

## 📝 สิ่งที่ต้องเพิ่มใน Client

### 1. NSP Installer Module

```cpp
class NSPInstaller {
    // ใช้ NCM API ติดตั้ง NSP
    bool installNSP(const char* path);

    // ใช้ NS API register กับ system
    bool registerApplication(u64 titleId);

    // Verify installation
    bool verifyInstalled(u64 titleId);
};
```

### 2. Workflow ที่สมบูรณ์

```
1. USB Transfer
   ├─> Download NSP from PC
   ├─> Save to /switch/downloads/
   └─> Show progress (40-50 MB/s)

2. NSP Installation
   ├─> Open NSP file
   ├─> Extract NCAs
   ├─> Verify signatures (ใช้ prod.keys อัตโนมัติ)
   ├─> Install to NAND/SD
   └─> Register with system

3. Verification
   ├─> Check installation status
   ├─> Verify title ID
   └─> Show success message

4. Cleanup
   └─> Delete temp file (optional)
```

## 🔐 Security & Keys

### prod.keys คืออะไร?

```
# prod.keys format
master_key_00 = XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
master_key_01 = XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
...
header_key = XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

**ใช้สำหรับ**:

- Decrypt NCA headers
- Verify signatures
- Decrypt game content

### title.keys คืออะไร?

```
# title.keys format
TITLE_ID = TITLE_KEY
01234567890ABCDEF = XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

**ใช้สำหรับ**:

- Decrypt specific titles
- Optional (ไม่จำเป็นสำหรับ NSP ส่วนใหญ่)

### Client ต้องทำอะไร?

**ไม่ต้องทำอะไรเลย!** เพราะ:

1. Keys อยู่ใน `/atmosphere/prod.keys`
2. Atmosphère mount keys ให้ system
3. NCM API อ่าน keys อัตโนมัติ
4. เราแค่เรียก API ธรรมดา

## ⚠️ ข้อควรระวัง

### 1. Storage Space

```cpp
// ตรวจสอบพื้นที่ก่อนติดตั้ง
s64 freeSpace;
nsGetFreeSpaceSize(NcmStorageId_SdCard, &freeSpace);

if (freeSpace < fileSize * 2) {
    printf("Not enough space!\n");
    return false;
}
```

### 2. Corrupted NSP

```cpp
// Verify NSP integrity
bool verifyNSP(const char* path) {
    // Check file size
    // Verify PFS0 header
    // Check NCA signatures
}
```

### 3. Installation Errors

```cpp
// Handle common errors
switch (rc) {
    case 0x234C02:  // Insufficient space
    case 0x234E02:  // Invalid NCA
    case 0x235002:  // Signature verification failed
    // ...
}
```

## 🎯 สรุป

### คำตอบคำถาม:

1. **ต้องติดตั้งที่ฝั่ง client ไหม?**

   - ✅ **ใช่** - ต้องใช้ NCM API ติดตั้งจริงๆ
   - ❌ ไม่ใช่แค่ copy file

2. **ต้องมี prod.keys ไหม?**

   - ✅ **ต้องมี** - แต่อยู่ใน Atmosphère แล้ว
   - ❌ Client ไม่ต้องจัดการเอง

3. **ต้องมี title.keys ไหม?**
   - ⚠️ **Optional** - ขึ้นอยู่กับ title
   - ส่วนใหญ่ไม่จำเป็น

### Next Steps:

1. ✅ Transfer file ผ่าน USB (ทำแล้ว)
2. ⏳ เพิ่ม NSP installer (ต้องทำ)
3. ⏳ ใช้ NCM API (ต้องทำ)
4. ⏳ Error handling (ต้องทำ)

---

**ต้องการให้ผมเพิ่ม NSP installer module ให้ไหมครับ?** 🎮
