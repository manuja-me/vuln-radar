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

- 🇨🇭 **Swiss Style Design System & Dual Themes**: Stark, objective International Typographic Style with zero-radius geometry, 1px hairlines, asymmetrical modular grid, and seamless live switching between **Swiss Dark** and **Swiss Light** with persistent local state.
- 🛡️ **Safe & Non-Destructive**: Audits misconfigurations, dangerous parameters, and exposure surfaces without sending destructive exploit payloads or degrading target services.
- ⚡ **Multi-Threaded Rust Core**: Native asynchronous Tokio runtime paired with lightweight Tauri v2 Webview bindings delivers sub-second scan execution with minimal memory footprint.
- 🎯 **RCE Risk & Parameter Heuristics**: Inspects target query strings for command execution (`?cmd=`, `?exec=`) and file/template inclusion (`?tpl=`, `?include=`) entry points, correlates server banners with known CVEs, and safely probes unauthenticated debug consoles.
- 🤖 **AI-Powered 1-Click Remediation**: Generates structured, copy-paste prompts tailored for AI coding assistants (Antigravity, Claude, ChatGPT) containing full finding context, OWASP definitions, and tech-stack-specific patches.
- 🔒 **Zero Telemetry & 100% Local Privacy**: All scan histories, monitored targets, and executive audit reports are stored entirely on your local machine in an embedded SQLite database (`WAL` mode). Zero external CDN fonts or cloud dependencies.
- ⏰ **Continuous Watchdog Daemon**: Background daemon monitors target URLs on custom intervals (1h, 6h, 12h, 24h, 7d) and fires native OS notifications upon security grade degradation or newly detected flaws.
- 📊 **Executive & Multi-Format Reports**: Export compliance audits instantly to PDF/Print, Markdown, JSON, CSV (formula-injection safe), or reproducible cURL command strings.

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

## 📦 Release Changelog

### 🇨🇭 [v0.8.0] — Swiss Style UI Redesign & Dual-Theme Engine
- 🎨 **Swiss Style Design System (International Typographic Style)**: Ground-up visual transformation prioritizing clarity, asymmetric modular grids, high information density, and sharp zero-radius geometry (`rounded-none`).
- 🌗 **Dual-Theme Engine (Swiss Dark & Swiss Light)**:
  - **Swiss Dark**: Pitch-black canvas (`#09090b`), sharp hairline dividers (`#27272a`), crisp white headline typography, and signal red accents.
  - **Swiss Light**: Stark white poster aesthetic (`#ffffff`), jet-black high-contrast text, razor-sharp hairlines, and vivid signal indicators.
  - **Live Nav Toggle**: Persistent instant toggle in the global navigation bar with zero flash of unstyled content (FOUC) and `localStorage` persistence.
- 📊 **Typographic Posture Score & Calibrated Meter**: Replaced blurred circular gauges with an oversized, bold score numeral, `/100` meta label, and 10-tick geometric calibrated bar.
- 📋 **Tabular Finding Inspector & Clean Evidence**: Findings render with flush-left severity strips, high-contrast monospace parameters, and clean tabular borders.
- 🧭 **Unified Numbered Workspaces**: Restructured navigation tabs (`01/AUDIT`, `02/PORTS`, `03/DNS`, `04/RECON`, `05/FLEET`, `06/WATCHDOG`, `07/HISTORY`, `08/SETTINGS`) with consistent tabular metrics.
- 🔒 **100% Offline / Local-First**: Zero external CDN font requests; powered purely by modern local system font stacks and CSS custom properties.

### 🛡️ [v0.7.0] — RCE Risk Assessment & Parameter Heuristic Scanner
- ⚡ **RCE (Remote Code Execution) Risk Engine**: Added a dedicated, non-intrusive RCE risk assessment scanner module (`rce.rs`) running concurrently in the Rust auditing core.
- 🎯 **URL Parameter Attack Surface Heuristics**: Automatically flags dangerous query parameters commonly abused for command injection (`?cmd=`, `?exec=`, `?run=`, `?eval=`) and dynamic template / file inclusion (`?tpl=`, `?template=`, `?include=`, `?page=`).
- 🔍 **Software & Framework CVE Correlation**: Passively correlates response banners and technology stacks against critical RCE CVEs (Apache 2.4.49/50 `CVE-2021-41773`, PHP 8.1.0-dev backdoor, EOL PHP CGI `CVE-2024-4577`, Jenkins `CVE-2024-23897`, Webmin `CVE-2019-15107`, Spring4Shell).
- 🚨 **High-Risk Management Endpoint Probing**: Safely audits unauthenticated management endpoints including Spring Boot Actuators (`/actuator/env`, `/actuator/gateway/routes`, `/actuator/jolokia`), Jenkins script console (`/script`), Apache Solr (`/solr/admin/cores`), and server status consoles.
- 🏷️ **UI & Reporting Integration**: Tagged with OWASP `A03:2021-Injection` and `A06:2021-Vulnerable Components`, CVE badges, filterable under `"RCE & Injection Risks"` across UI dashboards and exported audit reports.

### 🚀 [v0.6.3] — Streamlined Native Workspace & UI Refinements
- 🧹 **Streamlined Interface**: Cleaned up the Posture Audit view and findings dashboard to maintain a distraction-free, focused security auditing workspace.
- ⚡ **Performance & Core Stability**: Refined component tree and optimized telemetry rendering for desktop builds.

### 🚀 [v0.6.2] — Antigravity AI Remediation Engine & 1-Click Prompt Generator
- ✨ **"Fix These with AI" Action Bar**: Added a dynamic, dedicated AI remediation action button directly on the Severity Breakdown card that automatically stays hidden/disabled when 0 findings exist.
- 🤖 **Interactive AI Remediation Modal (`AiFixModal`)**: Generates structured, Antigravity/LLM-ready prompts incorporating full finding contexts (OWASP, CVE, evidence, HTTP telemetry, remediation diffs) with stack selection pills (*Auto-Detect*, *Node/Express*, *Next.js*, *Python/FastAPI*, *Django*, *Nginx*, *SvelteKit*).
- 📋 **1-Click AI Prompt Copy**: Added instant clipboard copying with visual feedback so developers can paste full remediation tasks directly into Antigravity or AI coding assistants.
- 🔍 **Per-Card "Fix with AI" Button**: Added individual AI prompt generators directly within each finding card's remediation section for targeted, single-issue fixes.

### 🚀 [v0.6.1] — Backend API Auditing & Smart Protocol Fallback
- 🌐 **Backend & Local API Support**: Seamlessly audit local development servers (`localhost:8000`, `127.0.0.1:5000`), microservices, Docker containers, and internal LAN targets.
- ⚡ **Smart Protocol Selection & Auto-Fallback**: Automatically defaults to `http://` for local/port-based targets and retries on `http://` if initial `https://` handshake fails.
- 🔒 **Self-Signed SSL/TLS Certificate Support**: Enabled `.danger_accept_invalid_certs(true)` to inspect staging servers and internal APIs with self-signed TLS certificates without connection crashes.
- 🎯 **Private Host Optimization**: Skips public DoH and Certificate Transparency lookups for private IP addresses (`10.x`, `192.168.x`, `172.16-31.x`, `.local`, `.internal`) to prevent unnecessary latency.
- 💡 **Actionable Diagnostics & Quick Presets**: Added `localhost:8000` quick target button and clear connection error guidance for offline backend services.

### 🚀 [v0.6.0] — Native Desktop Workstation & Auto Port Scanner
- 🖥️ **Native Desktop Workstation Architecture**: Replaced generic web styling with an ultra-sleek, acrylic left activity sidebar and dedicated workspaces (`Posture Audit`, `Port Matrix`, `DNS & Anti-Spoof`, `Surface Recon`, `Fleet Batch`, `Watchdog Daemon`, `Scan Logs`, and `Settings Hub`).
- 🪟 **Native Window Titlebar & Drag Region**: Added native header toolbar with `-webkit-app-region: drag` (`data-tauri-drag-region`), application badge, active port status pill, and instant command palette (`⌘K`).
- 🔌 **Automatic Default Port Scanning**: TCP Port Scanner is now enabled by default with the lowest/fastest preset (`Top 20`, 600ms timeout) without requiring manual configuration.
- 📊 **Native Live Status Bar**: Bottom 24px desktop status bar providing real-time telemetry (Rust Core version, SQLite WAL mode, Port Engine profile, Active monitors, Response latency, Keyboard shortcut hints).
- 🎨 **Desktop Density & Window Containment**: Viewport locked to `h-screen w-screen overflow-hidden` with zero browser scrolling, system native fonts, hairline borders, and disabled chrome text selection (`desktop-select-none`).

### ⚡ [v0.5.0] — Unified Settings Hub & Multi-Format Export
- ⚙️ **Unified Settings Hub (`⌘,`)**: Centralized scan parameters, custom HTTP headers, auth tokens, port scan profiles, scheduled monitors, and SQLite storage management into a single modal drawer.
- ⌨️ **Comprehensive Keyboard Navigation**: Added global shortcuts (`Ctrl+K`, `Ctrl+B`, `Ctrl+M`, `Ctrl+H`, `Ctrl+O`, `Ctrl+E`, `Ctrl+P`, `?`).
- 📄 **Multi-Format Export & Print**: Added JSON, CSV (formula injection-safe), Markdown, and cURL export capabilities.

### 🌐 [v0.4.1] — Multi-Source Subdomain Reconnaissance
- 🔍 Integrated Certificate Transparency logs (`crt.sh`) and HackerTarget with active DNS resolver verification.

### 🔌 [v0.4.0] — High-Speed TCP Port Discovery
- ⚡ Asynchronous TCP port scanner with service banner grabbing, preset profiles (`Top 20`, `Databases`, `Top 100`, `Custom`), and risk classification.

---

## 📥 Installation & Downloads

Pre-compiled production binaries are available for major operating systems on the **[Releases](https://github.com/manuja-me/vuln-radar/releases)** page:

| Operating System | Package Architecture | Formats |
|---|---|---|
| **Windows** | x86_64 (64-bit) | Installer (`.exe`), Windows Installer (`.msi`) |
| **macOS** | Universal Binary (Apple Silicon & Intel) | Disk Image (`.dmg`) |
| **Linux** | x86_64 (64-bit) | AppImage (`.AppImage`), Debian Package (`.deb`) |

### 🪟 Windows Package Manager (WinGet)

You can install and update VulnRadar on Windows 10 & 11 with a single command using `winget`:

```powershell
# Install VulnRadar
winget install manuja-me.VulnRadar

# Or using the friendly moniker
winget install vulnradar

# Upgrade to the latest version
winget upgrade manuja-me.VulnRadar
```

> [!NOTE]
> **Windows Defender Notice**:
> Because VulnRadar is a community-driven open-source project without a paid EV code-signing certificate, Windows SmartScreen may show an *"Unknown publisher"* dialog on initial launch.
> 1. Click **More info**
> 2. Click **Run anyway**

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
