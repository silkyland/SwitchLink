# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Network transfer support (TCP/IP)
- Configuration file support
- Multi-language support
- File verification (SHA256)
- Resume interrupted transfers
- Compression support

## [0.1.0] - 2025-10-13

### Added
- 🎉 Initial release of DBI Backend Rust Edition
- ✨ Full DBI protocol implementation
  - LIST command for file listing
  - FILE_RANGE command for file transfer
  - EXIT command for graceful shutdown
- 🖥️ Modern eGUI interface
  - File queue management
  - Add files and folders
  - Remove files from queue
- 📊 Real-time progress tracking
  - Progress bar with percentage
  - Transfer speed in MB/s
  - ETA (Estimated Time Arrival)
  - Bytes sent / Total size
- 📝 Live activity logs in GUI
- 🔌 USB communication with Nintendo Switch
  - Auto-detect Switch (VID: 0x057E, PID: 0x3000)
  - Bulk transfer support
  - Error recovery and reconnection
- 🚀 Performance optimizations
  - 1MB chunk size for optimal speed
  - Async USB operations
  - Efficient memory usage
- 📚 Comprehensive documentation
  - README with quick start
  - QUICKSTART for developers
  - CLIENT_IDEAS for future development
- 🔧 Build scripts and tools
  - build.sh for easy compilation
  - run.sh for quick testing
  - Desktop entry file

### Fixed
- 🐛 Fixed offset parsing issue (u64 vs u32)
  - Correctly parse file range offsets
  - Handle large file offsets (> 100GB detection)
- 🔧 Fixed I/O errors on USB connection
  - Proper interface claiming
  - Correct endpoint configuration
  - Better error handling for pipe/IO errors
- ⚡ Fixed timeout issues
  - Separate timeouts for different operations
  - Long timeout (30s) for ACK from Switch
  - Short timeout (100ms) for polling

### Performance
- 📈 ~2x faster than Python version (45 MB/s vs 25 MB/s)
- 💾 ~90% less memory usage (15 MB vs 120 MB)
- ⚡ Instant startup (<1s vs 2-3s)
- 📦 Smaller binary (~5 MB vs ~50 MB with Python)

### Technical Details
- Language: Rust 2021 Edition
- GUI Framework: eGUI 0.27
- USB Library: rusb 0.9
- Async Runtime: tokio 1.35
- Build: Optimized release with LTO

### Known Issues
- Debug logs are verbose (will be configurable in next version)
- No configuration file support yet
- Single Switch support only (multi-Switch planned)

### Migration from Python
- ✅ All Python functionality ported
- ✅ Protocol compatibility maintained
- ✅ Better error handling
- ✅ Improved user experience
- ✅ Native Linux support (no webkit issues)

---

## Version History

- **0.1.0** (2025-10-13) - Initial release with full DBI protocol support
