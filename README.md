<div align="center">

# 🛡️ VulnRadar

**Enterprise-Grade Desktop Web Security Posture & Passive Vulnerability Scanner**

*Perform non-intrusive security audits, front-end software composition analysis (SCA), port discovery, email/DNS anti-spoofing verification, and continuous posture monitoring.*

[![Release](https://img.shields.io/github/v/release/manuja-me/vuln-radar?style=for-the-badge&color=06B6D4&label=Latest%20Release)](https://github.com/manuja-me/vuln-radar/releases)
[![Build Status](https://img.shields.io/github/actions/workflow/status/manuja-me/vuln-radar/release.yml?style=for-the-badge&label=Build)](https://github.com/manuja-me/vuln-radar/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=for-the-badge)](https://github.com/manuja-me/vuln-radar/releases)

[![Tauri v2](https://img.shields.io/badge/Tauri-v2.0-24C8D8?logo=tauri&logoColor=white&style=flat-square)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.80+-black?logo=rust&logoColor=white&style=flat-square)](https://www.rust-lang.org)
[![Svelte 5](https://img.shields.io/badge/Svelte-5.0-FF3E00?logo=svelte&logoColor=white&style=flat-square)](https://svelte.dev)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind-v4.0-38B2AC?logo=tailwindcss&logoColor=white&style=flat-square)](https://tailwindcss.com)
[![SQLite](https://img.shields.io/badge/SQLite-WAL%20Mode-003B57?logo=sqlite&logoColor=white&style=flat-square)](https://sqlite.org)

[Key Capabilities](#-key-value-propositions) • [Feature Matrix](#-feature-matrix) • [Keyboard Shortcuts](#-keyboard-shortcuts) • [Architecture](#-architecture) • [Browser Mode](#-running-in-browser-localhost-mode) • [Installation](#-installation--downloads) • [Building from Source](#-building-from-source) • [Ethics & Security](#-security--ethics-disclaimer)

</div>

---

## 🚀 Executive Summary

**VulnRadar** is a high-performance, native desktop application engineered for security teams, DevSecOps practitioners, and web developers to evaluate organizational security posture in real time.

Unlike intrusive, active vulnerability scanners that can trigger Web Application Firewall (WAF) blocks, destabilize production infrastructure, or require complex testing agreements, VulnRadar conducts **intelligent, purely passive audits**. It synthesizes HTTP header verification, SSL/TLS cryptographic parameters, cookie hygiene, client-side Software Composition Analysis (SCA for CVEs), Certificate Transparency subdomain reconnaissance, DNS-over-HTTPS (DoH) email validation, and network port analysis into a unified **Security Posture Score (0–100)** with actionable, OWASP-aligned remediation steps.

---

## ⚡ Key Value Propositions

- 🛡️ **Safe & Passive Auditing**: Evaluates security misconfigurations and exposure surfaces without sending dangerous exploit payloads or disrupting target services.
- ⚡ **Native Performance & Minimal Footprint**: Built with a multi-threaded asynchronous Rust core and Tauri v2 native Webview bindings, providing sub-second scan execution with ultra-low memory overhead.
- 🔒 **Zero Telemetry & 100% Local Privacy**: All scan histories, monitored domains, and audit reports are strictly stored on your local machine in an embedded SQLite database (`WAL` mode). No external telemetry or cloud dependencies.
- 🎯 **Actionable Scoring & OWASP Mapping**: Every vulnerability finding is graded by severity (Critical, High, Medium, Low, Info) and paired with exploit impact analysis, standard references, and copy-paste remediation code.
- ⏰ **Automated Continuous Watchdog**: Background daemon continuously monitors scheduled targets (hourly, daily, weekly) and delivers native desktop notifications upon security grade degradation or newly introduced critical flaws.

---

## 🔍 Feature Matrix

| Security Module | Technical Inspection Details | Standard / Reference |
|---|---|---|
| **🛡️ HTTP Security Headers** | Audits Content-Security-Policy (CSP `unsafe-inline`, `unsafe-eval`, wildcards), HSTS, Clickjacking (`X-Frame-Options` / `frame-ancestors`), MIME sniffing (`X-Content-Type-Options`), Referrer-Policy, Permissions-Policy, COOP, and COEP. | OWASP A05:2021 (Security Misconfiguration) |
| **🍪 Cookie Hardening** | Inspects all `Set-Cookie` directives for missing `HttpOnly`, `Secure`, and strict `SameSite` (`Lax`/`Strict`/`None`) flags. | OWASP A01:2021 (Broken Access Control) |
| **🌐 Subdomain Reconnaissance** | Multi-source subdomain discovery querying Certificate Transparency logs (`crt.sh`) and HackerTarget with active DNS resolution. | Surface Attack Management & OSINT |
| **📧 DNS & Anti-Spoofing** | Queries DNS-over-HTTPS (DoH) to inspect SPF records (`v=spf1`), DMARC enforcement policies (`p=reject`/`p=quarantine`), and DNSSEC integrity. | RFC 7208 / RFC 7489 / RFC 4033 |
| **🤖 Endpoint & Policy Hunter** | Analyzes `/robots.txt` for disclosed administrative routes and checks RFC 9116 `/.well-known/security.txt` vulnerability disclosure compliance. | RFC 9116 / OWASP WSTG-INFO-003 |
| **📦 Front-End SCA & CVE Audit** | Fingerprints client-side JavaScript frameworks and libraries (jQuery, Angular, Bootstrap, Lodash, Moment.js) with known public CVE listings. | OWASP A06:2021 (Vulnerable Components) |
| **🔑 Secret & Leak Detection** | Scans client payloads for leaked cloud API keys (AWS, Google Cloud, Slack), JWT tokens, cleartext password forms, and developer comments. | OWASP A04:2021 (Insecure Design) |
| **🔌 Network Port Discovery** | High-speed TCP port scanner with service banner grabbing, preset profiles (`Top 20`, `Databases`, `Top 100`, `Custom`), and risk classification (e.g. exposed Telnet, SMB, RDP, Docker, Database engines). | Network Perimeter Hardening |
| **📑 Executive PDF Reports** | Generates executive audit summaries with visual score gauges, severity breakdowns, and one-click print / PDF export. | Compliance & Stakeholder Reporting |
| **📁 Fleet Batch Scanner** | Concurrently audits multiple target URLs from a list with real-time progress, individual report drill-downs, and aggregate score reporting. | Fleet & Infrastructure Auditing |
| **⚙️ Custom Parameters & Auth** | Configure custom request headers (`Authorization: Bearer <token>`, custom session cookies), User-Agent signatures, and timeouts. | Authenticated & Staging Audits |
| **⏰ Continuous Watchdog** | Background worker scans targets on schedule (1h, 6h, 12h, 24h, 7d) and triggers desktop notifications upon score drops. | Continuous Threat Monitoring |

---

## ⌨️ Keyboard Shortcuts

VulnRadar features power-user keyboard navigation for seamless operational workflows:

| Shortcut | Action | Scope |
|---|---|---|
| <kbd>Ctrl</kbd> + <kbd>K</kbd> / <kbd>Cmd</kbd> + <kbd>K</kbd> | Focus Target URL Bar | Global |
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

## 🌐 Running in Browser (Localhost / Headless CLI Mode)

If you prefer to access VulnRadar through your standard web browser (Chrome, Edge, Firefox, Brave, Safari) without launching the native desktop window, you can run the local web server directly via CLI:

```bash
# 1. Install frontend dependencies (if not already installed)
npm install

# 2. Spin up local development server and open in default browser
npm run web

# Alternatively, using standard npm/vite flags:
npm run dev -- --open
```

The interactive dashboard will be served locally at **`http://localhost:5173`**.

> [!TIP]
> **LAN / Remote Network Access**:
> To access the dashboard from other machines or mobile devices across your local network:
> ```bash
> npm run dev -- --host --open
> ```

---

## 🔒 Security & Ethics Disclaimer

VulnRadar is designed specifically for **defensive posture evaluation, DevSecOps compliance audits, and authorized penetration testing**. 

The scanning engine conducts purely passive inspection against publicly accessible HTTP, TLS, and DNS endpoints and does not transmit destructive payloads. Users are responsible for ensuring appropriate testing authorization when evaluating target domains and network assets.

---

## 📜 License

This project is open-source software licensed under the **[MIT License](LICENSE)**.
