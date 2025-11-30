# 🎨 DBI Backend - Modern UI/UX Guide

## 🌟 Key Features

### 1. Modern Color Palette

- **Primary**: Indigo (#6366F1) - สำหรับ main actions
- **Accent**: Pink (#EC4899) - สำหรับ highlights
- **Success**: Green (#22C55E) - สำหรับ success states
- **Error**: Red (#EF4444) - สำหรับ errors
- **Dark Theme**: Slate 900/800/700 - สำหรับ backgrounds

### 2. Card-Based Components

ทุก section ใช้ card design พร้อม:

- Rounded corners (12px)
- Subtle borders
- Proper padding (15-20px)
- Shadow effects

### 3. Enhanced Buttons

```
📁 Add Folder    - Primary button (Indigo)
📄 Add Files     - Primary button (Indigo)
🗑️ Clear Queue   - Danger button (Red)
🔄 Refresh       - Secondary button (Gray)
▶ Start Server  - Primary button (Indigo)
■ Stop Server   - Danger button (Red)
```

### 4. Visual Feedback

- **Status Badge**: ● Running (Green) / ○ Stopped (Gray)
- **Queue Badge**: แสดงจำนวน files พร้อม highlight
- **Progress Bar**: Animated พร้อม percentage
- **Color-Coded Logs**: ✅ Success, ❌ Error, 🔄 Info

## 📱 Layout Structure

```
┌─────────────────────────────────────────────────────────┐
│  🎮 DBI Backend          ● Running     ▶ Start Server  │ <- Header
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────────────────────────────────────────┐  │
│  │ 📁 File Library              📦 Stats           │  │ <- File Library
│  └─────────────────────────────────────────────────┘  │    Header Card
│                                                         │
│  ┌─────────────────────────────────────────────────┐  │
│  │ 📁 Add  📄 Add  🗑️ Clear  🔄 Refresh  Queue: 0  │  │ <- Action
│  └─────────────────────────────────────────────────┘  │    Buttons Card
│                                                         │
│  ┌─────────────────────────────────────────────────┐  │
│  │ 🔍 Search files...                          ✕   │  │ <- Search Card
│  └─────────────────────────────────────────────────┘  │
│                                                         │
│  ┌─────────────────────────────────────────────────┐  │
│  │ [File Table]                                    │  │ <- File Table
│  └─────────────────────────────────────────────────┘  │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌──────────────────────────┐   │
│  │ 📋 Activity Log  │  │ 📖 Quick Start Guide     │   │ <- Bottom Panel
│  │                  │  │ or                       │   │   (2 columns)
│  │ [Terminal-style] │  │ 📊 Transfer Progress     │   │
│  └──────────────────┘  └──────────────────────────┘   │
├─────────────────────────────────────────────────────────┤
│  v0.1.0  |  Built with Rust 🦀      ☕ Buy me coffee  │ <- Footer
└─────────────────────────────────────────────────────────┘
```

## 🎯 User Flow

### Adding Files

1. Click **📁 Add Folder** or **📄 Add Files**
2. Select files/folder from dialog
3. See **✅ Added X files** in activity log
4. Files appear in table with stats

### Starting Transfer

1. Add files to library
2. Click **+** button to add to queue
3. See **Queue: X** badge update
4. Click **▶ Start Server**
5. See **● Running** status badge
6. View real-time progress in right panel

### Monitoring Progress

- **Current File**: Shows filename being transferred
- **Progress Bar**: Animated with percentage
- **Stats Cards**:
  - 📤 Transferred: Shows bytes sent
  - 💾 Total Size: Shows total size
  - ⚡ Speed: Shows MB/s
  - ⏱ ETA: Shows estimated time

### Activity Log

- **Color-Coded Messages**:
  - ✅ Green: Success operations
  - ❌ Red: Errors
  - 🔄 Blue: Info messages
  - Gray: General logs
- **Auto-Scroll**: Always shows latest message
- **History**: Keeps last 50 messages

## 🎨 Design Tokens

### Colors

```rust
Primary:        #6366F1  // Indigo
Primary Hover:  #4F46E5  // Darker Indigo
Accent:         #EC4899  // Pink
Success:        #22C55E  // Green
Warning:        #FBBF24  // Amber
Error:          #EF4444  // Red
Info:           #3B82F6  // Blue

BG Primary:     #0F172A  // Slate 900
BG Secondary:   #1E293B  // Slate 800
BG Tertiary:    #334155  // Slate 700

Text Primary:   #F8FAFC  // Slate 50
Text Secondary: #CBD5E1  // Slate 300
Text Muted:     #94A3B8  // Slate 400
```

### Spacing

```rust
Small:   8px
Medium:  12px
Large:   15px
XLarge:  20px
```

### Border Radius

```rust
Small:  6px
Medium: 8px
Large:  12px
```

### Typography

```rust
Heading:    18-24px, Bold
Body:       13-14px, Regular
Small:      11-12px, Regular
Monospace:  13px, Monospace (for logs)
```

## 🚀 Quick Start

### Build & Run

```bash
# Development
cargo run

# Release (optimized)
cargo build --release
./target/release/dbi-backend-rust
```

### First Time Setup

1. Launch application
2. Read **📖 Quick Start Guide** in bottom-right panel
3. Click **📁 Add Folder** to add your game files
4. Connect Nintendo Switch via USB
5. Launch DBI on Switch
6. Select "Install title from DBIbackend"
7. Click **▶ Start Server**
8. Monitor progress in **📊 Transfer Progress** panel

## 💡 Tips

### File Management

- Use **📁 Add Folder** for bulk imports
- Use **📄 Add Files** for selective imports
- Click **⭐** to favorite files
- Use **🔍 Search** to filter files
- Click **+** to add to queue
- Click **-** to remove from queue

### Transfer Optimization

- Add multiple files to queue for batch transfer
- Monitor **⚡ Speed** to check USB performance
- Check **⏱ ETA** for time estimation
- Watch **📋 Activity Log** for any errors

### Troubleshooting

- If transfer is slow, check USB cable quality
- If connection fails, restart DBI on Switch
- Check **📋 Activity Log** for error messages
- Use **🔄 Refresh** to reload file list

## 🎉 What's New

### UI Improvements

- ✅ Modern card-based design
- ✅ Vibrant color palette
- ✅ Large, accessible buttons
- ✅ Clear visual hierarchy
- ✅ Generous spacing
- ✅ Premium appearance

### UX Enhancements

- ✅ Quick Start Guide panel
- ✅ Real-time progress tracking
- ✅ Color-coded activity logs
- ✅ Status badges and indicators
- ✅ Hover effects and animations
- ✅ Better error messages

### Technical Improvements

- ✅ Larger window size (1400x900)
- ✅ Minimum window size (1024x768)
- ✅ Optimized rendering
- ✅ Better performance
- ✅ Cleaner code structure

---

**Enjoy the new modern UI! 🎨✨**
