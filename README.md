# 🦐 MuxPortal

**Termius with tmux** — An out-of-the-box SSH client with native tmux session management.

![Desktop](docs/screenshots/desktop.png)
![Mobile](docs/screenshots/mobile.png)

## ✨ Features

- **SSH Connection Management** — Store and organize your hosts with password or SSH key authentication
- **Native tmux Integration** — Automatically detects remote tmux, lists sessions as tabs
- **Session Management** — Create, attach, switch, and kill tmux sessions with GUI buttons
- **Terminal Emulation** — Full terminal via xterm.js with 256-color support
- **GUI Split Panes** — Map tmux `split-window` to GUI split view (coming soon)
- **Cross-Platform** — macOS, Windows, Linux, Android, iOS

## 📦 Install

### Desktop

Download from [GitHub Releases](https://github.com/cctice/muxportal/releases/latest):

| Platform | File |
|----------|------|
| Windows | `.msi` / `.exe` |
| macOS | `.dmg` |
| Linux | `.AppImage` / `.deb` |

### Mobile

| Platform | File |
|----------|------|
| Android | `.apk` |
| iOS | `.ipa` |

## 🚀 Quick Start

1. Add a host connection (host, port, username, auth)
2. Click a host to connect
3. MuxPortal auto-detects tmux sessions and shows them as tabs
4. Click a tab to attach, or create a new session

## 🛠 Build from Source

### Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) >= 1.70
- Platform-specific: Xcode (macOS/iOS), Android SDK + NDK (Android)

### Desktop

```bash
git clone https://github.com/cctice/muxportal.git
cd muxportal
npm install
npm run tauri dev     # Development
npm run tauri build   # Production build
```

### Android

```bash
npm run tauri android init
npm run tauri android dev
```

### iOS

```bash
npm run tauri ios init
npm run tauri ios dev
```

## 🏗 Tech Stack

- **Framework**: [Tauri 2.0](https://tauri.app/) — Rust backend + Web frontend
- **Frontend**: React + TypeScript + Vite
- **Terminal**: [xterm.js](https://github.com/xtermjs/xterm.js)
- **SSH**: [ssh2-rs](https://github.com/alexcrichton/ssh2-rs) (Rust native SSH)
- **Mobile**: Tauri 2.0 mobile (Kotlin for Android, Swift for iOS)

## 📄 License

[MIT](LICENSE)
