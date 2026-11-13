# ☕ WinCaffeine

<p align="center">
  <a href="https://github.com/TowelDude/wincaffeine/releases"><img src="https://img.shields.io/github/v/release/TowelDude/wincaffeine?color=orange" alt="Release" /></a>
  <a href="https://github.com/TowelDude/wincaffeine/blob/master/LICENSE"><img src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue" alt="License" /></a>
  <img src="https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-0078D6?logo=windows" alt="Platform" />
  <img src="https://img.shields.io/badge/arch-x64%20%7C%20x86-blueviolet" alt="Arch" />
  <a href="https://www.virustotal.com/gui/file/355d4727a0de9f6ff5662b84fe5609d4a867f2e813f4460c3f98ac0c2379f9af"><img src="https://img.shields.io/badge/VirusTotal%20(x64)-Clean-brightgreen?logo=virustotal" alt="VirusTotal x64" /></a>
  <a href="https://www.virustotal.com/gui/file/8dec2866b62f479d33affe1cf542436db6b02798f9b6837a9c3fac576c33446c"><img src="https://img.shields.io/badge/VirusTotal%20(x86)-Clean-brightgreen?logo=virustotal" alt="VirusTotal x86" /></a>
</p>

**The ultra-lightweight macOS Caffeine & Amphetamine alternative for Windows, built in pure Rust.**

WinCaffeine lives unobtrusively in your system tray and prevents your display, lock screen, and screensaver from turning on while you work, read, render, or download.

Zero runtime dependencies, handcrafted pixel art, native Win32 APIs, and an ultra-compact binary footprint (~**250 KB**).

---

## 🎨 Visual Preview

<p align="center">
  <img src="assets/hero_active.png" width="160" alt="WinCaffeine Active" />
  &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
  <img src="assets/hero_inactive.png" width="160" alt="WinCaffeine Inactive" />
  <br/>
  <b>Active (Awake)</b> &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; <b>Inactive (Normal Sleep)</b>
</p>

| State | Tray Icon | Behavior |
| :--- | :---: | :--- |
| **Active** | ☕ Steaming Mug | Keeps display & system awake continuously via `SetThreadExecutionState`. |
| **Inactive** | 💤 Sleepy Mug | Lets Windows sleep naturally according to your system power plan. |

> **Note for Windows 10 / 11:** New system tray icons may initially appear inside the overflow chevron menu (`^`) next to the clock. You can drag the coffee mug icon out onto the taskbar so it is always visible.

---

## ⚡ Features

- **The Windows Equivalent of macOS Caffeine / Amphetamine**:
  - Simple, fast display inhibitor that keeps your PC awake on demand without changing Windows power plans.
- **Ultra-Compact & Fast**:
  - Pure native Win32 API (`windows-sys`).
  - No Electron, no WebView, no bulky runtime.
  - Compiles to a single standalone `.exe` under 300 KB.
  - Idle resource consumption: **0% CPU**, **~2 MB RAM**.
- **System Tray Presets**:
  - `Allow Sleep (Turn Off)` (manually deactivate anytime)
  - `15 Minutes`
  - `30 Minutes`
  - `60 Minutes`
  - `Until Turned Off` (Indefinite keep-awake)
- **Custom Timer Dialog**:
  - **Duration Mode**: Enter custom minutes or hours with quick chips (`+15m`, `+30m`, `+1h`, `+2h`, `+4h`).
  - **Specific Time Mode**: Specify an exact clock time (e.g. `18:30` or `23:00`) when Caffeine will turn off.
  - Real-time countdown calculation preview.
- **Robust Startup & Recovery**:
  - Listens to `TaskbarCreated` broadcast messages to automatically restore the tray icon whenever Windows Explorer boots or restarts.
  - Self-healing tray icon state.
- **Background Startup & Installation**:
  - Run portably without installation, or install with one click.
  - First launch provides an interactive setup dialog:
    - Copies to `%LOCALAPPDATA%\Caffeine\caffeine.exe`.
    - Configures Windows Startup (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`).
  - Context menu toggle: `[✓] Start with Windows (Service)`.
- **Single Instance Guard**:
  - Launching another instance focuses the existing instance's menu without duplicating tray icons.
- **Dual Architecture Support**:
  - Native standalone binaries for both **x64** (64-bit) and **x86** (32-bit).

---

## 🛡️ Security & Integrity (VirusTotal)

WinCaffeine binaries are 100% open-source, stripped of build environment metadata, and free of telemetry or external network calls:

| Binary | Arch | Size | SHA-256 Checksum | VirusTotal Scan |
| :--- | :---: | :---: | :--- | :---: |
| [`caffeine-x64.exe`](https://github.com/TowelDude/wincaffeine/releases/latest/download/caffeine-x64.exe) | 64-bit | 272 KB | `355d4727a0de9f6ff5662b84fe5609d4a867f2e813f4460c3f98ac0c2379f9af` | [![VirusTotal](https://img.shields.io/badge/VirusTotal-Scan%20Report-blue?logo=virustotal)](https://www.virustotal.com/gui/file/355d4727a0de9f6ff5662b84fe5609d4a867f2e813f4460c3f98ac0c2379f9af) |
| [`caffeine-x86.exe`](https://github.com/TowelDude/wincaffeine/releases/latest/download/caffeine-x86.exe) | 32-bit | 236 KB | `8dec2866b62f479d33affe1cf542436db6b02798f9b6837a9c3fac576c33446c` | [![VirusTotal](https://img.shields.io/badge/VirusTotal-Scan%20Report-blue?logo=virustotal)](https://www.virustotal.com/gui/file/8dec2866b62f479d33affe1cf542436db6b02798f9b6837a9c3fac576c33446c) |

> **Note:** VirusTotal indexes files by SHA-256 hash upon their first upload. When downloading a freshly released binary, you can verify it by dropping the `.exe` onto [virustotal.com](https://www.virustotal.com/) to trigger its first scan report.

---

## 🚀 Installation & Usage

### 1. Interactive Setup
1. Download `caffeine-x64.exe` (or `caffeine-x86.exe`) from [Releases](https://github.com/TowelDude/wincaffeine/releases).
2. Double-click the executable.
3. If not yet installed, the setup dialog appears:
   - Click **"Install as Background Service"** to install into `%LOCALAPPDATA%\Caffeine` and enable startup on login.
   - Or click **"Run Portably Once"** to run immediately without touching the registry.
4. The coffee cup icon appears in your notification area (system tray).

### 2. Command Line Options

```cmd
# Run silently in tray (used by Windows startup)
caffeine.exe --tray

# Install to AppData and enable Windows startup
caffeine.exe --install

# Unregister startup and uninstall
caffeine.exe --uninstall

# Check installation and autostart status
caffeine.exe --status

# Show help
caffeine.exe --help
```

---

## 🛠️ Building from Source

### Prerequisites
- [Rust](https://rustup.rs/) (1.80+ recommended)
- `cargo-xwin` (for cross-compilation from Linux/macOS)

### Windows Native Build
```cmd
cargo build --release
```

### Cross-Compiling (Linux / macOS)
```bash
# 64-bit (x64)
cargo xwin build --release --target x86_64-pc-windows-msvc

# 32-bit (x86)
cargo xwin build --release --target i686-pc-windows-msvc
```

Compiled binaries will be located at:
- `target/x86_64-pc-windows-msvc/release/caffeine.exe`
- `target/i686-pc-windows-msvc/release/caffeine.exe`

### Pixel Art Regeneration
The pixel art generator is located in `assets/generate_assets.py`:
```bash
python3 assets/generate_assets.py
```

---

## 📜 License
MIT / Apache-2.0
