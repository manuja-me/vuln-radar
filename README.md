# 🛡️ VulnRadar

> **Desktop Web Vulnerability & Security Posture Scanner**  
> Built with **Rust**, **Tauri v2**, **Svelte 5**, **TypeScript**, and **Tailwind CSS**.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24C8D8?logo=tauri&logoColor=white)](https://tauri.app)
[![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte&logoColor=white)](https://svelte.dev)
[![Rust](https://img.shields.io/badge/Rust-1.80+-000000?logo=rust&logoColor=white)](https://www.rust-lang.org)

---

## 🚀 Overview

**VulnRadar** is a lightweight, cross-platform desktop application designed to perform **real-time passive security audits** and vulnerability assessments against target web applications.

When a URL is inspected, VulnRadar evaluates HTTP headers, SSL/TLS enforcement, cookie security attributes, client-side JavaScript dependencies (against known CVE databases), and client-side sensitive data disclosures to compute an instant **Security Health Score (0–100)** and actionable remediation guide.

---

## ✨ Features

- 🔍 **Security Header Inspection**: Flags missing or weakly configured `Content-Security-Policy`, `Strict-Transport-Security`, `X-Frame-Options` (Clickjacking), `X-Content-Type-Options`, `Referrer-Policy`, and `Permissions-Policy`.
- 🍪 **Cookie Security Audit**: Analyzes `Set-Cookie` directives for missing `HttpOnly`, `Secure`, and `SameSite` flags.
- 📦 **Software Composition Analysis (SCA)**: Identifies outdated front-end libraries (jQuery, Angular 1.x, Bootstrap, Lodash, Moment.js) and maps them to known **CVEs**.
- 🔑 **Information Disclosure & Leak Detection**: Regex pattern matching for exposed cloud credentials (AWS keys, Google API keys, Slack tokens, JWTs) and sensitive HTML developer comments.
- 🎯 **Risk Health Score**: Automated 0–100 rating with grade classifications (Grade A to F) and OWASP Top 10 mappings.
- 📊 **Local SQLite Scan History**: Persists all scan records locally for retrospective analysis.
- 📄 **Export Reports**: Generate and export assessment reports to structured Markdown or JSON.

---

## 🛠️ Architecture & Tech Stack

```
vuln-radar/
├── src/                          # Svelte 5 + TypeScript Frontend
│   ├── lib/
│   │   ├── components/           # ScoreGauge, FindingCard, HistoryModal, ExportModal, Navbar
│   │   └── types.ts              # Security & Report Data Types
│   ├── routes/                   # SvelteKit App Shell & Dashboard
│   └── app.css                   # Tailwind CSS Theme
└── src-tauri/                    # Rust Core Backend
    ├── Cargo.toml
    └── src/
        ├── models.rs             # Vulnerability, Severity, and Report structures
        ├── scanner/              # Analysis modules (headers, cookies, dependencies, leaks)
        ├── db/                   # Embedded SQLite persistence
        └── lib.rs                # Tauri command handlers & IPC
```

---

## ⚡ Getting Started

### 📥 Download Pre-built Release

Download the latest installer or portable executable for your operating system from the [Releases](https://github.com/manuja-me/vuln-radar/releases) page:

- **Windows**: `VulnRadar_<version>_x64-setup.exe` or `.msi`
- **macOS**: `VulnRadar_<version>_universal.dmg`
- **Linux**: `VulnRadar_<version>_amd64.AppImage` or `.deb`

> [!NOTE]
> **Windows Defender SmartScreen Notice**:
> Because VulnRadar is an open-source tool distributed without a costly commercial Extended Validation (EV) certificate, Windows may show a *"Windows protected your PC (Unknown publisher)"* popup on first launch.
> 1. Click **More info**
> 2. Click **Run anyway**

---

### Prerequisites (For Building from Source)

- [Node.js](https://nodejs.org/) (v18+) & `npm`
- [Rust & Cargo](https://rustup.rs/) (v1.80+)
- [Tauri Prerequisites](https://tauri.app/start/prerequisites/) for your operating system

### Local Development

```bash
# 1. Clone the repository
git clone https://github.com/manuja-me/vuln-radar.git
cd vuln-radar

# 2. Install dependencies
npm install

# 3. Run the desktop application in development mode
npm run tauri dev
```

### Production Build

```bash
npm run tauri build
```

The compiled standalone installer and executable will be generated under `src-tauri/target/release/bundle/`.


---

## 🔒 Security & Ethics Disclaimer

VulnRadar is designed for **defensive security auditing, development compliance, and authorized penetration testing**. Passive analysis inspects standard client-accessible resources. Always obtain explicit authorization before executing active security testing against systems you do not own.

---

## 📜 License

Distributed under the [MIT License](LICENSE).
