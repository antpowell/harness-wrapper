# DSH WebView Wrapper - Build Guide

## Understanding the Build Process

### 1. `cargo build --release` vs `npm run tauri build`

| Command | What It Does | Output | Use Case |
|---------|-------------|--------|----------|
| `cargo build --release` | Compiles Rust backend only | Single `.exe` file | Development, testing, quick builds |
| `npm run tauri build` | Full production build + bundling | `.exe` + MSI installer + updater | Distribution, installation |

#### `cargo build --release`
```
Frontend (already built) ──┐
                            ├──► Rust Compiler ──► dsh-webview-wrapper.exe
Rust backend code ──────────┘
```

**Pros:**
- Fast (2-3 minutes)
- No additional tools needed
- Single file output
- Good for testing

**Cons:**
- No installer
- No auto-updater
- No code signing
- User must manually place files

#### `npm run tauri build`
```
Frontend Source ──► Vite Build ──► dist/ ──┐
                                           ├──► Tauri Bundler ──► MSI + EXE + Updater
Rust Source ──────► Cargo Build ──► EXE ───┘
```

**Pros:**
- Professional MSI installer
- Auto-updater support
- Code signing ready
- Start menu shortcuts
- Uninstaller included

**Cons:**
- Slower (5-10 minutes)
- Requires WiX toolset (Windows)
- More complex configuration

---

## How to Run the Built Executable

### Method 1: Direct Execution
```bash
# Navigate to project folder
cd C:\Engineering\deepSeekHarness\dsh-webview-wrapper

# Run the release executable
.\src-tauri\target\release\dsh-webview-wrapper.exe
```

### Method 2: Using the Batch Script
```bash
# Simply double-click or run:
.\run-release.bat
```

### Method 3: From Command Line with Options
```bash
# Run with console output visible
cd src-tauri\target\release
dsh-webview-wrapper.exe

# Run with logging
set RUST_LOG=debug
dsh-webview-wrapper.exe
```

---

## What Happens When You Run

### Step-by-Step Execution Flow

```
1. You run dsh-webview-wrapper.exe
        │
        ▼
2. Rust backend initializes
   - Loads configuration
   - Sets up state management
   - Initializes plugins
        │
        ▼
3. Window creation attempt
   - Creates native OS window
   - Initializes WebView2 control
   - Loads embedded frontend files
        │
        ▼
4. Frontend loads
   - HTML/CSS/JS rendered in WebView
   - Shows loading screen
   - Attempts to connect to DSH
        │
        ▼
5. Connection to DSH
   - Checks http://127.0.0.1:3080
   - Retries if not available
   - Loads DSH interface in iframe
```

---

## Troubleshooting

### "Access is denied" Error
**Cause:** Sandbox restrictions prevent window creation
**Solution:** Run outside sandbox or with elevated permissions

### "Failed to setup app" Error
**Cause:** WebView2 initialization blocked
**Solution:** Ensure WebView2 runtime is installed

### Frontend Not Loading
**Cause:** `dist/` folder missing or empty
**Solution:** Run `npm run build` first

### Port Already in Use
**Cause:** Previous instance still running
**Solution:** Kill node processes: `Get-Process node | Stop-Process`

---

## File Structure After Build

```
dsh-webview-wrapper/
├── src-tauri/
│   ├── target/
│   │   ├── debug/              # Development build
│   │   │   └── dsh-webview-wrapper.exe  (larger, slower)
│   │   └── release/            # Production build
│   │       └── dsh-webview-wrapper.exe  (smaller, faster) ← THIS ONE
│   └── ...
├── dist/                       # Built frontend files
├── src/                        # Frontend source
└── ...
```

---

## Quick Reference

### Build Commands
```bash
# Development build (fast, unoptimized)
cargo build

# Release build (slow, optimized)
cargo build --release

# Full production build with installer
npm run tauri build

# Development with hot reload
npm run tauri dev
```

### Run Commands
```bash
# Run release build
.\src-tauri\target\release\dsh-webview-wrapper.exe

# Run with debug logging
$env:RUST_LOG="debug"; .\src-tauri\target\release\dsh-webview-wrapper.exe

# Run development version
.\src-tauri\target\debug\dsh-webview-wrapper.exe
```

---

## Next Steps

1. **Test the executable** - Try running it on an unrestricted system
2. **Customize the UI** - Modify files in `src/` and rebuild
3. **Add features** - Edit `src-tauri/src/main.rs` for new commands
4. **Create installer** - Run `npm run tauri build` on a system with WiX
5. **Distribute** - Share the `.exe` or MSI with users