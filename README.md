<div align="center">

# 🛡️ VulnRadar

**Enterprise-Grade Desktop Web Security Posture & Vulnerability Scanner**

*Perform real-time passive reconnaissance, software composition audits, email/DNS security checks, and continuous posture monitoring.*

[![Release](https://img.shields.io/github/v/release/manuja-me/vuln-radar?style=for-the-badge&color=06B6D4&label=Latest%20Release)](https://github.com/manuja-me/vuln-radar/releases)
[![Build Status](https://img.shields.io/github/actions/workflow/status/manuja-me/vuln-radar/release.yml?style=for-the-badge&label=Build)](https://github.com/manuja-me/vuln-radar/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=for-the-badge)](https://github.com/manuja-me/vuln-radar/releases)

[![Tauri v2](https://img.shields.io/badge/Tauri-v2.0-24C8D8?logo=tauri&logoColor=white&style=flat-square)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.80+-black?logo=rust&logoColor=white&style=flat-square)](https://www.rust-lang.org)
[![Svelte 5](https://img.shields.io/badge/Svelte-5.0-FF3E00?logo=svelte&logoColor=white&style=flat-square)](https://svelte.dev)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind-v4.0-38B2AC?logo=tailwindcss&logoColor=white&style=flat-square)](https://tailwindcss.com)

[Download Release](https://github.com/manuja-me/vuln-radar/releases) • [Features](#-core-capabilities) • [Architecture](#-architecture) • [Getting Started](#-getting-started) • [Security Policy](#-security--ethics-disclaimer)

</div>

---

## 🚀 Overview

**VulnRadar** is a high-performance, cross-platform desktop application built for security engineers, DevSecOps teams, and developers to perform non-intrusive, real-time security posture assessments against web applications and organizational domains.

Unlike intrusive, active penetration scanners that trigger WAF blocks and incident alarms, VulnRadar conducts **intelligent passive audits** — combining HTTP header verification, SSL/TLS checks, cookie hygiene, client-side Software Composition Analysis (SCA for CVEs), Certificate Transparency subdomain reconnaissance, and DNS-over-HTTPS (DoH) email spoofing validation into a single **Security Posture Score (0–100)** with OWASP-aligned remediation steps.

---

## ✨ Key Value Propositions

- ⚡ **Blazing Fast & Lightweight**: Powered by a multi-threaded Rust backend and Tauri v2 native bindings for near-zero memory footprint.
- 🔒 **Zero Telemetry & Local Privacy**: All scan histories, monitored targets, and configurations are stored purely on your local machine via an embedded SQLite database.
- 🎯 **Actionable Scoring & OWASP Mapping**: Every finding contains vulnerability severity ratings, real-world exploit impact explanations, and copy-paste remediation guidance.
- ⏰ **Automated Watchdog**: Built-in background service that automatically re-evaluates monitored assets and triggers desktop notifications if security grades degrade.

---

## 🔍 Feature Matrix

| Security Module | Description | Standard / Standard Mappings |
|---|---|---|
| **🛡️ HTTP Security Headers** | Audits CSP, HSTS, X-Frame-Options (Clickjacking), X-Content-Type-Options, Referrer-Policy, and Permissions-Policy. | OWASP A05:2021 (Security Misconfiguration) |
| **🍪 Cookie Hardening** | Inspects `Set-Cookie` directives for missing `HttpOnly`, `Secure`, and `SameSite` flags. | OWASP A01:2021 (Broken Access Control) |
| **🌐 Subdomain Reconnaissance** | Queries public Certificate Transparency (`crt.sh`) logs to map organizational subdomains and expanded attack surfaces. | OSINT & Surface Management |
| **📧 DNS & Email Security** | Performs DoH queries to evaluate SPF records (`v=spf1`), DMARC enforcement policies (`p=reject`), and DNSSEC status. | RFC 7208 / RFC 7489 (Anti-Spoofing) |
| **🤖 Endpoint & Policy Hunter** | Analyzes `/robots.txt` for disclosed sensitive admin routes and verifies RFC 9116 `/.well-known/security.txt` disclosure compliance. | RFC 9116 / OWASP WSTG |
| **📦 Dependency CVE Audit (SCA)** | Detects outdated front-end JavaScript libraries (jQuery, Angular 1.x, Bootstrap, Lodash, Moment.js) with known public CVEs. | OWASP A06:2021 (Vulnerable Components) |
| **🔑 Secret & Leak Detection** | Scans client payloads for leaked cloud API keys (AWS, Google, Slack), JWT tokens, and exposed developer HTML comments. | OWASP A04:2021 (Insecure Design) |
| **📑 Executive PDF Reports** | Generates executive audit summaries with visual score gauges, risk distributions, and one-click print/PDF export. | Client & Compliance Reporting |
| **📁 Batch Fleet Scanner** | Concurrently audits multiple target URLs from a single list with real-time status and aggregate score reporting. | Fleet & Infrastructure Auditing |
| **⚙️ Custom Headers & Auth** | Configure custom request headers (`Authorization: Bearer <token>`, custom cookies), User-Agents, and timeouts. | Authenticated & Staging Audits |
| **⏰ Continuous Watchdog** | Background daemon that scans targets on schedule (1h, 6h, 12h, 24h, 7d) and triggers desktop alerts on score drops. | Continuous Threat Monitoring |

---

## 🛠️ Architecture

```
vuln-radar/
├── src/                                  # Frontend UI (Svelte 5 + TypeScript + Tailwind)
│   ├── lib/
│   │   ├── components/
│   │   │   ├── ExecutiveReportModal.svelte  # Printable / PDF audit report generator
│   │   │   ├── BatchScanModal.svelte        # Fleet / multi-target scanner modal
│   │   │   ├── ScanOptionsModal.svelte      # Custom HTTP headers & auth configuration
│   │   │   ├── MonitorModal.svelte          # Background watchdog & continuous scheduler
│   │   │   ├── HistoryModal.svelte          # Local SQLite scan history drawer
│   │   │   ├── ScoreGauge.svelte            # Dynamic security health radial gauge
│   │   │   ├── FindingCard.svelte           # Finding card with OWASP & CVE tags
│   │   │   └── Navbar.svelte                # Top navigation & target input bar
│   │   └── types.ts                      # Core TypeScript definitions
│   └── routes/+page.svelte               # Primary reactive dashboard
└── src-tauri/                            # Backend Core (Rust + Tauri v2)
    ├── Cargo.toml
    └── src/
        ├── models.rs                     # Security findings, report models & DTOs
        ├── scanner/                      # Modular analysis engine
        │   ├── headers.rs                # HTTP security header verification
        │   ├── cookies.rs                # Cookie policy inspection
        │   ├── dependencies.rs           # Front-end CVE detection & SCA
        │   ├── leaks.rs                  # Secret pattern matching & comment leaks
        │   ├── subdomains.rs             # Certificate Transparency recon (crt.sh)
        │   ├── dns.rs                    # DoH SPF/DMARC/DNSSEC verification
        │   └── endpoints.rs              # robots.txt & security.txt inspection
        ├── db/                           # Embedded SQLite persistence (scans & monitors)
        └── lib.rs                        # IPC command handlers & Tokio background worker
```

---

## ⚡ Getting Started

### 📥 Download Pre-built Binaries

Pre-compiled production binaries are available on the [Releases](https://github.com/manuja-me/vuln-radar/releases) page:

| Operating System | Package Format | Download Link |
|---|---|---|
| **Windows (x64)** | Setup Installer (`.exe`) / MSI (`.msi`) | [Download Windows](https://github.com/manuja-me/vuln-radar/releases) |
| **macOS (Universal / Apple Silicon & Intel)** | Disk Image (`.dmg`) | [Download macOS](https://github.com/manuja-me/vuln-radar/releases) |
| **Linux (x64)** | AppImage / Debian (`.deb`) | [Download Linux](https://github.com/manuja-me/vuln-radar/releases) |

> [!NOTE]
> **Windows Defender SmartScreen Notice**:
> Because VulnRadar is an open-source project without a paid EV code-signing certificate, Windows SmartScreen may show an *"Unknown publisher"* prompt on first launch.
> 1. Click **More info**
> 2. Click **Run anyway**

---

### 🔨 Building from Source

#### Prerequisites
- [Node.js](https://nodejs.org/) (v18+) & `npm`
- [Rust & Cargo](https://rustup.rs/) (v1.80+)
- [Tauri Prerequisites](https://tauri.app/start/prerequisites/) for your operating system

```bash
# 1. Clone the repository
git clone https://github.com/manuja-me/vuln-radar.git
cd vuln-radar

# 2. Install frontend dependencies
npm install

# 3. Launch in development mode with live-reload
npm run tauri dev

# 4. Compile optimized production installer
npm run tauri build
```

Production installers will be generated under `src-tauri/target/release/bundle/`.

---

## 🔒 Security & Ethics Disclaimer

VulnRadar is created for **defensive posture evaluation, DevSecOps compliance verification, and authorized penetration testing**. 
The scanning engine executes purely passive analysis against publicly accessible HTTP/DNS endpoints without injecting malicious exploit payloads. Always verify that you have proper authorization before conducting security testing against networks and applications you do not own.

---

## 📜 License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.

