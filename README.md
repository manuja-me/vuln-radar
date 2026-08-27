<div align="center">

# 🛡️ VulnRadar

**Enterprise-Grade Desktop Web Security Posture & Vulnerability Scanner**

*High-performance, non-intrusive security audits, RCE risk heuristics, front-end software composition analysis (SCA), TCP port discovery, email/DNS anti-spoofing verification, and continuous posture monitoring.*

[![Release](https://img.shields.io/github/v/release/manuja-me/vuln-radar?style=for-the-badge&color=06B6D4&label=Latest%20Release)](https://github.com/manuja-me/vuln-radar/releases)
[![Build Status](https://img.shields.io/github/actions/workflow/status/manuja-me/vuln-radar/release.yml?style=for-the-badge&label=Build)](https://github.com/manuja-me/vuln-radar/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=for-the-badge)](https://github.com/manuja-me/vuln-radar/releases)

[![Tauri v2](https://img.shields.io/badge/Tauri-v2.0-24C8D8?logo=tauri&logoColor=white&style=flat-square)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.80+-black?logo=rust&logoColor=white&style=flat-square)](https://www.rust-lang.org)
[![Svelte 5](https://img.shields.io/badge/Svelte-5.0-FF3E00?logo=svelte&logoColor=white&style=flat-square)](https://svelte.dev)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind-v4.0-38B2AC?logo=tailwindcss&logoColor=white&style=flat-square)](https://tailwindcss.com)
[![SQLite](https://img.shields.io/badge/SQLite-WAL%20Mode-003B57?logo=sqlite&logoColor=white&style=flat-square)](https://sqlite.org)

[Key Capabilities](#-key-value-propositions) • [Feature Matrix](#-feature-matrix) • [Architecture](#-architecture) • [Workspaces](#-workspaces--navigation) • [Keyboard Shortcuts](#-keyboard-shortcuts) • [Installation](#-installation--downloads) • [Building from Source](#-building-from-source) • [Changelog](#-release-changelog) • [Ethics & Security](#-security--ethics-disclaimer)

</div>

---

## 🚀 Executive Summary

**VulnRadar** is a native, privacy-first desktop security workstation engineered for DevSecOps engineers, security researchers, and developers to evaluate web application posture in real time.

Unlike invasive scanners that send disruptive payloads, trigger Web Application Firewall (WAF) IP bans, or destabilize production databases, VulnRadar conducts **intelligent, non-intrusive audits**. It synthesizes HTTP security header validation, RCE risk parameter heuristics, SSL/TLS cryptographic health, cookie hygiene, client-side Software Composition Analysis (SCA for known CVEs), Certificate Transparency subdomain enumeration, DNS-over-HTTPS (DoH) email validation, and network port analysis into an actionable **Security Posture Score (0–100)** with OWASP-mapped remediation guidelines.

---

## ⚡ Key Value Propositions

- 🛡️ **Safe & Non-Destructive**: Audits misconfigurations, dangerous parameters, and exposure surfaces without sending destructive exploit payloads or degrading target services.
- ⚡ **Multi-Threaded Rust Core**: Native asynchronous Tokio runtime paired with lightweight Tauri v2 Webview bindings delivers sub-second scan execution with minimal memory footprint.
- 🎯 **RCE Risk & Parameter Heuristics**: Inspects target query strings for command execution (`?cmd=`, `?exec=`) and file/template inclusion (`?tpl=`, `?include=`) entry points, correlates server banners with known CVEs, and safely probes unauthenticated debug consoles.
- 🤖 **AI-Powered 1-Click Remediation**: Generates structured, copy-paste prompts tailored for AI coding assistants (Antigravity, Claude, ChatGPT) containing full finding context, OWASP definitions, and tech-stack-specific patches.
- 🔒 **Zero Telemetry & 100% Local Privacy**: All scan histories, monitored targets, and executive audit reports are stored entirely on your local machine in an embedded SQLite database (`WAL` mode). No external analytics or cloud dependencies.
- ⏰ **Continuous Watchdog Daemon**: Background daemon monitors target URLs on custom intervals (1h, 6h, 12h, 24h, 7d) and fires native OS notifications upon security grade degradation or newly detected flaws.
- 📊 **Executive & Multi-Format Reports**: Export compliance audits instantly to PDF/Print, Markdown, JSON, CSV (formula-injection safe), or reproducible cURL command strings.

---

## 🔍 Feature Matrix

| Security Module | Technical Inspection Details | Standard / Reference |
|---|---|---|
| **💥 RCE Risk & Attack Surface** | Inspects URL query strings for command execution/evaluation keywords (`cmd`, `exec`, `eval`, `run`) and template/file inclusion (`tpl`, `include`, `page`); matches server banners against known critical RCE CVEs (Apache `CVE-2021-41773`, PHP `CVE-2024-4577`, PHP 8.1.0-dev backdoor, Jenkins `CVE-2024-23897`, Webmin `CVE-2019-15107`); probes unauthenticated Spring Actuators (`/actuator/env`), Jenkins Groovy console (`/script`), and Solr Admin. | OWASP A03:2021 (Injection) / OWASP A06:2021 (Vulnerable Components) |
| **🛡️ HTTP Security Headers** | Audits Content-Security-Policy (CSP `unsafe-inline`, `unsafe-eval`, wildcards), HSTS, Clickjacking (`X-Frame-Options` / `frame-ancestors`), MIME sniffing (`X-Content-Type-Options`), Referrer-Policy, Permissions-Policy, COOP, and COEP. | OWASP A05:2021 (Security Misconfiguration) |
| **🍪 Cookie Hardening** | Inspects all `Set-Cookie` directives for missing `HttpOnly`, `Secure`, and strict `SameSite` (`Lax`/`Strict`/`None`) flags. | OWASP A01:2021 (Broken Access Control) |
| **🌐 Subdomain Reconnaissance** | Multi-source subdomain discovery querying Certificate Transparency logs (`crt.sh`) and HackerTarget with active DNS resolution. | Surface Attack Management & OSINT |
| **📧 DNS & Anti-Spoofing** | Queries DNS-over-HTTPS (DoH) to inspect SPF records (`v=spf1`), DMARC enforcement policies (`p=reject`/`p=quarantine`), and DNSSEC integrity. | RFC 7208 / RFC 7489 / RFC 4033 |
| **🤖 Endpoint & Policy Hunter** | Analyzes `/robots.txt` for disclosed administrative routes and checks RFC 9116 `/.well-known/security.txt` vulnerability disclosure compliance. | RFC 9116 / OWASP WSTG-INFO-003 |
| **📦 Front-End SCA & CVE Audit** | Fingerprints client-side JavaScript frameworks and libraries (jQuery, Angular, Bootstrap, Lodash, Moment.js) with known public CVE listings. | OWASP A06:2021 (Vulnerable Components) |
| **🔑 Secret & Leak Detection** | Scans client payloads for leaked cloud API keys (AWS, Google Cloud, Slack), JWT tokens, cleartext password forms, and developer comments. | OWASP A04:2021 (Insecure Design) |
| **🔌 Network Port Discovery** | High-speed TCP port scanner with service banner grabbing, preset profiles (`Top 20`, `Databases`, `Top 100`, `Custom`), and risk classification (e.g. exposed Telnet, SMB, RDP, Docker, Database engines). | Network Perimeter Hardening |
| **🤖 AI Remediation Prompts** | Generates context-rich remediation prompts with stack selection pills (*Auto-Detect*, *Node/Express*, *Next.js*, *Python/FastAPI*, *Django*, *Nginx*, *SvelteKit*) for 1-click clipboard copying. | AI-Assisted DevSecOps |
| **📑 Executive PDF Reports** | Generates executive audit summaries with visual score gauges, severity breakdowns, and one-click print / PDF export. | Compliance & Stakeholder Reporting |
| **📁 Fleet Batch Scanner** | Concurrently audits multiple target URLs from a list with real-time progress, individual report drill-downs, and aggregate score reporting. | Fleet & Infrastructure Auditing |
| **⚙️ Custom Parameters & Auth** | Configure custom request headers (`Authorization: Bearer <token>`, custom session cookies), User-Agent signatures, and timeouts. | Authenticated & Staging Audits |
| **⏰ Continuous Watchdog** | Background worker scans targets on schedule (1h, 6h, 12h, 24h, 7d) and triggers desktop notifications upon score drops. | Continuous Threat Monitoring |

---

## 🖥️ Workspaces & Navigation

VulnRadar is designed as a native desktop security workstation featuring 8 specialized workspaces:

1. **🛡️ Posture Audit**: Primary reactive dashboard with the dynamic security score gauge, severity breakdown, finding filter pills, and AI remediation action bar.
2. **🔌 Port Matrix**: Visual TCP port grid detailing open ports, service banners, protocols, and exposure risk ratings.
3. **📧 DNS & Anti-Spoof**: Real-time inspection of SPF alignment, DMARC quarantine/reject policy enforcement, and DNSSEC validation.
4. **🌐 Surface Recon**: Subdomain enumeration feed correlating Certificate Transparency log history and DNS records.
5. **📁 Fleet Batch**: Concurrent bulk scanner for auditing fleets of domain URLs with CSV import/export.
6. **⏰ Watchdog Daemon**: Continuous background monitoring schedule manager with configurable scan frequencies.
7. **📜 Scan History**: Local SQLite scan history drawer with one-click report reloading, deletion, and comparison.
8. **⚙️ Settings Hub**: Centralized configuration drawer for custom HTTP headers, auth tokens, port profiles, and database maintenance.

---

## ⌨️ Keyboard Shortcuts

VulnRadar features power-user keyboard shortcuts for fluid navigation:

| Shortcut | Action | Scope |
|---|---|---|
| <kbd>Ctrl</kbd> + <kbd>K</kbd> / <kbd>Cmd</kbd> + <kbd>K</kbd> | Focus Target URL Input Bar | Global |
| <kbd>Ctrl</kbd> + <kbd>,</kbd> / <kbd>Cmd</kbd> + <kbd>,</kbd> | Open Unified Settings Hub | Global |
| <kbd>Ctrl</kbd> + <kbd>H</kbd> / <kbd>Cmd</kbd> + <kbd>H</kbd> | Open Scan History Drawer | Global |
| <kbd>Ctrl</kbd> + <kbd>B</kbd> / <kbd>Cmd</kbd> + <kbd>B</kbd> | Open Batch Fleet Scanner | Global |
| <kbd>Ctrl</kbd> + <kbd>M</kbd> / <kbd>Cmd</kbd> + <kbd>M</kbd> | Open Watchdog Monitors | Global |
| <kbd>Ctrl</kbd> + <kbd>O</kbd> / <kbd>Cmd</kbd> + <kbd>O</kbd> | Open Scan Parameters & Auth | Global |
| <kbd>Ctrl</kbd> + <kbd>E</kbd> / <kbd>Cmd</kbd> + <kbd>E</kbd> | Export Report (Markdown, JSON, CSV, cURL) | Active Scan |
| <kbd>Ctrl</kbd> + <kbd>P</kbd> / <kbd>Cmd</kbd> + <kbd>P</kbd> | Generate Executive Print / PDF Report | Active Scan |
| <kbd>?</kbd> | Display Keyboard Shortcuts Modal | Global |
| <kbd>Esc</kbd> | Dismiss Active Modal / Drawer | Global |

---

## 🛠️ Architecture

```
vuln-radar/
├── src/                                  # Frontend UI (Svelte 5 Runes + TypeScript + Tailwind CSS)
│   ├── lib/
│   │   ├── components/
│   │   │   ├── SettingsModal.svelte      # Unified configuration & preferences hub
│   │   │   ├── ExecutiveReportModal.svelte# Printable / PDF audit report generator
│   │   │   ├── ExportModal.svelte        # Multi-format export (JSON, CSV, Markdown, cURL)
│   │   │   ├── BatchScanModal.svelte     # Fleet multi-target scanner
│   │   │   ├── ScanOptionsModal.svelte   # Custom HTTP headers & auth configuration
│   │   │   ├── MonitorModal.svelte       # Continuous watchdog daemon manager
│   │   │   ├── HistoryModal.svelte       # Local SQLite scan history drawer
│   │   │   ├── ScoreGauge.svelte         # Dynamic radial security health gauge
│   │   │   ├── FindingCard.svelte        # Finding card with OWASP & CVE tags
│   │   │   ├── SeverityBadge.svelte      # Standardized severity badges
│   │   │   ├── Toast.svelte              # Reactive notification system
│   │   │   └── Navbar.svelte             # Top navigation & target input bar
│   │   └── types.ts                      # Core TypeScript definitions & DTOs
│   └── routes/+page.svelte               # Primary reactive dashboard
└── src-tauri/                            # Backend Core (Rust + Tauri v2 native runtime)
    ├── Cargo.toml                        # Rust dependencies & max release optimization profile
    └── src/
        ├── models.rs                     # Security findings, report models & DTOs
        ├── scanner/                      # Modular analysis engine
        │   ├── rce.rs                    # RCE risk heuristics & known CVE correlation
        │   ├── headers.rs                # HTTP security header verification
        │   ├── cookies.rs                # Cookie policy inspection
        │   ├── dependencies.rs           # Front-end CVE detection & SCA
        │   ├── leaks.rs                  # Secret pattern matching & comment leaks
        │   ├── subdomains.rs             # Multi-source subdomain reconnaissance
        │   ├── dns.rs                    # DoH SPF/DMARC/DNSSEC verification
        │   ├── endpoints.rs              # robots.txt & security.txt inspection
        │   ├── ports.rs                  # Asynchronous TCP port scanner & banner grabber
        │   └── mod.rs                    # Scanner orchestration & score calculation
        ├── db/                           # Embedded SQLite persistence (WAL mode & caching)
        └── lib.rs                        # Tauri IPC command handlers & Tokio background worker
```

---

## 📦 Release Changelog

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

---

## 🔒 Security & Ethics Disclaimer

VulnRadar is designed specifically for **defensive posture evaluation, DevSecOps compliance audits, and authorized penetration testing**. 

The scanning engine conducts purely non-destructive inspection against publicly accessible HTTP, TLS, and DNS endpoints and does not transmit destructive exploit payloads. Users are responsible for ensuring appropriate testing authorization when evaluating target domains and network assets.

---

## 📜 License

This project is open-source software licensed under the **[MIT License](LICENSE)**.
