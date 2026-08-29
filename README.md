<div align="center">

# 🛡️ VulnRadar

**Enterprise-grade desktop security workstation & non-intrusive vulnerability scanner.**

[![GitHub Release](https://img.shields.io/github/v/release/manuja-me/vuln-radar?style=flat&color=3b82f6)](https://github.com/manuja-me/vuln-radar/releases)
[![Build Status](https://img.shields.io/github/actions/workflow/status/manuja-me/vuln-radar/release.yml?branch=main&style=flat)](https://github.com/manuja-me/vuln-radar/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg?style=flat)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat)](https://github.com/manuja-me/vuln-radar/releases)

[![Tauri](https://img.shields.io/badge/Tauri-v2-24C8D8?logo=tauri&logoColor=white&style=flat-square)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.80+-black?logo=rust&logoColor=white&style=flat-square)](https://www.rust-lang.org)
[![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte&logoColor=white&style=flat-square)](https://svelte.dev)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind-v4-38B2AC?logo=tailwindcss&logoColor=white&style=flat-square)](https://tailwindcss.com)
[![SQLite](https://img.shields.io/badge/SQLite-WAL-003B57?logo=sqlite&logoColor=white&style=flat-square)](https://sqlite.org)

[Features](#-features) • [Installation](#-installation) • [Workspaces](#-workspaces) • [Building from Source](#-building-from-source) • [Architecture](#-architecture) • [Security & Ethics](#-security--ethics-disclaimer)

</div>

---

## 🚀 Overview

**VulnRadar** is a native, privacy-first desktop security workstation engineered for DevSecOps engineers, security researchers, and developers to evaluate web application attack surfaces in real time.

Built on **Rust** and **Tauri v2**, VulnRadar performs sub-second, **non-intrusive audits** without disruptive payloads, rate-limit bans, or service degradation. It correlates HTTP security headers, dangerous URL parameter patterns, SSL/TLS cryptographic posture, cookie hygiene, known CVEs, Certificate Transparency logs, email security (SPF/DMARC/DNSSEC), and TCP port discovery into an actionable **Security Posture Score (0–100)** with OWASP remediation blueprints.

---

## ✨ Features

- 🛡️ **Safe & Non-Destructive Scanning** — Audits misconfigurations, dangerous parameters, and exposure surfaces without sending disruptive payloads or destabilizing production databases.
- 🎯 **RCE & Attack Surface Heuristics** — Inspects query strings for command execution (`?cmd=`, `?exec=`) and dynamic template inclusion (`?tpl=`, `?include=`), correlates server banners with known CVEs, and safely probes unauthenticated debug consoles.
- 🔒 **Zero Telemetry & 100% Local Privacy** — All scan histories, monitored targets, and executive audit reports are stored locally in an embedded SQLite database (`WAL` mode). No external analytics or cloud dependencies.
- 🔌 **High-Speed TCP Port Discovery** — Built-in asynchronous TCP scanner with service banner grabbing and preset profiles (`Top 20`, `Databases`, `Top 100`, `Custom`).
- 📧 **DNS & Anti-Spoofing Verification** — Queries DNS-over-HTTPS (DoH) to inspect SPF records (`v=spf1`), DMARC enforcement policies (`p=reject`/`p=quarantine`), and DNSSEC integrity.
- 🌐 **Subdomain Reconnaissance** — Multi-source discovery aggregating Certificate Transparency logs (`crt.sh`) and passive DNS resolution.
- 🤖 **1-Click AI Remediation Prompts** — Generates context-rich remediation prompts tailored for AI coding assistants (Antigravity, Claude, ChatGPT) with stack presets (*Node.js*, *Next.js*, *FastAPI*, *Django*, *Nginx*, *SvelteKit*).
- ⏰ **Continuous Watchdog Daemon** — Background monitor tracks target endpoints on custom intervals (1h, 6h, 12h, 24h, 7d) and triggers desktop notifications upon security grade degradation.
- 📊 **Executive & Multi-Format Exports** — Export compliance audits instantly to PDF/Print, Markdown, JSON, CSV (formula injection safe), or reproducible cURL commands.

---

## 📥 Installation

Pre-compiled production binaries for all major platforms are available on the **[Releases](https://github.com/manuja-me/vuln-radar/releases)** page.

| Platform | Architecture | Package Format |
|---|---|---|
| **Windows** | `x86_64` (64-bit) | Installer (`.exe`), Windows Installer (`.msi`) |
| **macOS** | Universal (`Apple Silicon` & `Intel`) | Disk Image (`.dmg`) |
| **Linux (Universal)** | `x86_64` (64-bit) | AppImage (`.AppImage`) |
| **Linux (Debian / Ubuntu)** | `x86_64` (64-bit) | Debian Package (`.deb`) |
| **Linux (Arch Linux / Manjaro)** | `x86_64` (64-bit) | Pacman Package (`.pkg.tar.zst`), AUR (`vuln-radar-bin`) |

### Linux Quick Install

#### Arch Linux (`.pkg.tar.zst` / AUR)
Download the `.pkg.tar.zst` package from GitHub Releases and install with `pacman`:

```bash
sudo pacman -U vuln-radar-bin-<version>-1-x86_64.pkg.tar.zst
```

Or build and install locally with `makepkg`:

```bash
cd packaging/aur/vuln-radar-bin
makepkg -si
```

#### Debian / Ubuntu (`.deb`)
```bash
sudo dpkg -i VulnRadar_<version>_amd64.deb
# or
sudo apt install ./VulnRadar_<version>_amd64.deb
```

#### Standalone Linux (`.AppImage`)
```bash
chmod +x VulnRadar_<version>_amd64.AppImage
./VulnRadar_<version>_amd64.AppImage
```

> [!NOTE]
> **Windows Defender Notice**:
> Because VulnRadar is a community-driven open-source project without a paid EV code-signing certificate, Windows SmartScreen may show an *"Unknown publisher"* dialog on initial launch. Click **More info** → **Run anyway**.

---

## 🖥️ Workspaces

VulnRadar provides 8 dedicated desktop workspaces designed for security auditing:

| Workspace | Description |
|---|---|
| **🛡️ Posture Audit** | Core interactive dashboard with dynamic security score gauge, severity breakdown, finding filters, and AI remediation action bar. |
| **🔌 Port Matrix** | Visual TCP port grid detailing open ports, service banners, protocols, and exposure risk ratings. |
| **📧 DNS & Anti-Spoof** | Real-time inspection of SPF alignment, DMARC quarantine/reject policies, and DNSSEC validation. |
| **🌐 Surface Recon** | Subdomain enumeration feed correlating Certificate Transparency log history and DNS records. |
| **📁 Fleet Batch** | Concurrent bulk scanner for auditing fleets of domain URLs with CSV import/export. |
| **⏰ Watchdog Daemon** | Scheduled background threat monitor with configurable scan intervals and system notifications. |
| **📜 Scan History** | Local SQLite scan history drawer with one-click report reloading, deletion, and comparison. |
| **⚙️ Settings Hub** | Centralized configuration for custom HTTP headers, auth tokens, port profiles, and database storage. |

### ⌨️ Keyboard Shortcuts

| Shortcut | Action | Scope |
|---|---|---|
| <kbd>Ctrl</kbd> / <kbd>Cmd</kbd> + <kbd>K</kbd> | Focus Target URL Input Bar | Global |
| <kbd>Ctrl</kbd> / <kbd>Cmd</kbd> + <kbd>,</kbd> | Open Unified Settings Hub | Global |
| <kbd>Ctrl</kbd> / <kbd>Cmd</kbd> + <kbd>H</kbd> | Open Scan History Drawer | Global |
| <kbd>Ctrl</kbd> / <kbd>Cmd</kbd> + <kbd>B</kbd> | Open Fleet Batch Scanner | Global |
| <kbd>Ctrl</kbd> / <kbd>Cmd</kbd> + <kbd>M</kbd> | Open Watchdog Monitors | Global |
| <kbd>Ctrl</kbd> / <kbd>Cmd</kbd> + <kbd>O</kbd> | Open Scan Parameters & Auth | Global |
| <kbd>Ctrl</kbd> / <kbd>Cmd</kbd> + <kbd>E</kbd> | Export Report (Markdown, JSON, CSV, cURL) | Active Scan |
| <kbd>Ctrl</kbd> / <kbd>Cmd</kbd> + <kbd>P</kbd> | Generate Executive Print / PDF Report | Active Scan |
| <kbd>?</kbd> | Display Keyboard Shortcuts Modal | Global |
| <kbd>Esc</kbd> | Dismiss Active Modal / Drawer | Global |

---

## 🛠️ Architecture & Tech Stack

```
vuln-radar/
├── packaging/
│   └── aur/
│       └── vuln-radar-bin/
│           └── PKGBUILD                  # Arch Linux AUR package recipe (.pkg.tar.zst)
├── src/                                  # Frontend UI (Svelte 5 Runes + TypeScript + Tailwind CSS)
│   ├── lib/
│   │   ├── components/                   # UI components, modals, gauges, finding cards
│   │   └── types.ts                      # Core TypeScript definitions & DTOs
│   └── routes/+page.svelte               # Primary reactive workstation dashboard
└── src-tauri/                            # Backend Core (Rust + Tauri v2 native runtime)
    ├── Cargo.toml                        # Rust dependencies & optimization profiles
    └── src/
        ├── models.rs                     # Security finding & report models
        ├── scanner/                      # Modular security audit modules (RCE, Headers, Ports, etc.)
        ├── db/                           # Embedded SQLite persistence (WAL mode)
        └── lib.rs                        # Tauri IPC command handlers & Tokio background worker
```

---

## 🔨 Building from Source

### Prerequisites
- **Node.js** (v18+) & `npm`
- **Rust & Cargo** (v1.80+)
- **OS Build Dependencies**: Follow the official [Tauri v2 Prerequisites](https://tauri.app/start/prerequisites/) for your operating system.

### Step-by-Step Setup

```bash
# 1. Clone the repository
git clone https://github.com/manuja-me/vuln-radar.git
cd vuln-radar

# 2. Install frontend dependencies
npm install

# 3. Launch development server with hot-reloading
npm run tauri dev

# 4. Compile optimized production release package
npm run tauri build
```

Production installers will be output to `src-tauri/target/release/bundle/`.

### Packaging for Arch Linux (`.pkg.tar.zst`)

```bash
cd packaging/aur/vuln-radar-bin
updpkgsums   # Updates SHA256 checksums if testing a new asset
makepkg -si  # Builds package and installs dependencies
```

---

## 🔒 Security & Ethics Disclaimer

VulnRadar is designed specifically for **defensive posture evaluation, DevSecOps compliance audits, and authorized penetration testing**. 

The scanning engine conducts purely non-destructive inspection against publicly accessible HTTP, TLS, and DNS endpoints and does not transmit destructive exploit payloads. Users are responsible for ensuring appropriate testing authorization when evaluating target domains and network assets.

---

## 📜 License

This project is open-source software licensed under the **[MIT License](LICENSE)**.
