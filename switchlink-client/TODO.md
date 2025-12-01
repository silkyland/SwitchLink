# 🚀 SwitchLink Client - Future Features & Roadmap

## ✅ Current Features (v0.2.0)

- ✅ NSP Installation via USB
- ✅ Ticket Installation (ES service)
- ✅ Base Game + Update + DLC support
- ✅ Progress tracking with speed indicator
- ✅ Console-based UI with ANSI colors
- ✅ Error handling and recovery
- ✅ Application record registration

---

## 🎯 Planned Features (Priority Order)

### **High Priority** 🔴

#### 1. **Installation Verification**

- [ ] Verify installed content after installation
- [ ] Check NCA integrity
- [ ] Validate application records
- [ ] Show installed game information

**Why:** Ensure installation success and detect corrupted files

**Estimated Time:** 2-4 hours

---

#### 2. **Better Error Messages**

- [ ] User-friendly error messages (Thai + English)
- [ ] Error code lookup table
- [ ] Troubleshooting suggestions
- [ ] Log file generation

**Why:** Help users understand and fix problems

**Estimated Time:** 2-3 hours

---

#### 3. **Installation Queue**

- [ ] Select multiple NSP files
- [ ] Queue management (add/remove/reorder)
- [ ] Batch installation
- [ ] Auto-continue after completion

**Why:** Install multiple games without manual intervention

**Estimated Time:** 4-6 hours

---

### **Medium Priority** 🟡

#### 4. **SD Card Installation Source**

- [ ] Browse NSP files on SD card
- [ ] Install from SD card (no USB needed)
- [ ] File browser UI

**Why:** Alternative installation method when USB is not available

**Estimated Time:** 6-8 hours

---

#### 5. **Installation History**

- [ ] Track installed games
- [ ] Installation date/time
- [ ] Installation status (success/failed)
- [ ] Reinstall detection

**Why:** Keep track of what's been installed

**Estimated Time:** 3-4 hours

---

#### 6. **NSZ Support (Compressed NSP)**

- [ ] Decompress NSZ files
- [ ] On-the-fly decompression during installation
- [ ] Progress tracking for decompression

**Why:** Smaller file sizes, faster transfers

**Estimated Time:** 8-12 hours

**Dependencies:** zstd library

---

#### 7. **NCA Signature Verification**

- [ ] Verify NCA header signatures
- [ ] Warn about modified/corrupted files
- [ ] Optional verification (can be disabled)

**Why:** Detect corrupted or modified files

**Estimated Time:** 6-8 hours

**Dependencies:** mbedtls library

---

### **Low Priority** 🟢

#### 8. **XCI Support (Cartridge Dumps)**

- [ ] Parse XCI files
- [ ] Extract and install from XCI
- [ ] HFS0 filesystem support

**Why:** Support cartridge dumps (less common than NSP)

**Estimated Time:** 12-16 hours

---

#### 9. **Network Installation**

- [ ] HTTP/HTTPS download
- [ ] Install directly from URL
- [ ] Resume support for interrupted downloads

**Why:** Install without PC (direct from internet)

**Estimated Time:** 10-14 hours

**Dependencies:** curl library, SSL support

---

#### 10. **Multi-language Support**

- [ ] Thai language
- [ ] English language
- [ ] Language selection in settings
- [ ] Localized error messages

**Why:** Better user experience for non-English speakers

**Estimated Time:** 4-6 hours

---

#### 11. **Settings/Configuration**

- [ ] Default installation location (SD/NAND)
- [ ] Enable/disable ticket installation
- [ ] Enable/disable NCA verification
- [ ] Color scheme selection
- [ ] Save settings to config file

**Why:** Customize user experience

**Estimated Time:** 4-6 hours

---

#### 12. **Game Management**

- [ ] List installed games
- [ ] Uninstall games
- [ ] View game information
- [ ] Manage DLC and updates

**Why:** Complete game management solution

**Estimated Time:** 16-20 hours

**Note:** This might be better as a separate app (like Goldleaf)

---

## 🔧 Technical Improvements

### **Code Quality**

- [ ] Add unit tests
- [ ] Add integration tests
- [ ] Improve error handling
- [ ] Code documentation (Doxygen)
- [ ] Performance profiling

### **Build System**

- [ ] CI/CD pipeline
- [ ] Automated builds
- [ ] Release automation
- [ ] Version management

### **UI/UX**

- [ ] Touch screen support
- [ ] Better progress indicators
- [ ] Confirmation dialogs
- [ ] Help/About screen

---

## 📋 Feature Request Template

If you want to request a new feature, please provide:

1. **Feature Name:** What is it?
2. **Description:** What does it do?
3. **Use Case:** Why is it needed?
4. **Priority:** High/Medium/Low
5. **Estimated Complexity:** Easy/Medium/Hard

---

## 🎯 Next Release Goals (v0.3.0)

**Target Features:**

1. Installation Verification
2. Better Error Messages
3. Installation Queue

**Timeline:** 2-3 weeks

---

## 🤝 Contributing

Want to help implement these features? Here's how:

1. Pick a feature from the list
2. Create a branch: `feature/feature-name`
3. Implement the feature
4. Test thoroughly
5. Submit a pull request

---

## 📝 Notes

- Features are listed in priority order
- Estimated times are approximate
- Some features may require external libraries
- User feedback will influence priority

---

**Last Updated:** 2025-12-01
**Version:** 0.2.0
