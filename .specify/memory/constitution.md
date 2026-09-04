<!--
Sync Impact Report
- Version change: Initial Draft (Template) → 1.0.0
- List of modified principles:
  - [PRINCIPLE_1_NAME] → I. Safe & Non-Destructive Auditing
  - [PRINCIPLE_2_NAME] → II. Zero-Telemetry & Local-First Privacy
  - [PRINCIPLE_3_NAME] → III. Asynchronous Rust Core & Zero UI Blocking
  - [PRINCIPLE_4_NAME] → IV. Type-Safe Tauri IPC & Shared Schemas
  - [PRINCIPLE_5_NAME] → V. Standardized Scoring & OWASP-Mapped Findings
- Added sections:
  - Technical Constraints & Security Standards
  - Development Workflow & Quality Gates
- Removed sections: None
- Follow-up TODOs: None
-->

# VulnRadar Constitution

## Core Principles

### I. Safe & Non-Destructive Auditing
VulnRadar audits MUST NOT degrade target performance, execute destructive exploit payloads, or risk data corruption on target web applications. All automated checks MUST focus on passive inspection (HTTP response headers, cookie directives, TLS handshakes, DNS records, public OSINT/Certificate Transparency) and safe, non-intrusive heuristics (parameter pattern discovery, CVE banner correlation, unauthenticated status probes). Active intrusive probes or brute-force mechanisms are strictly forbidden unless explicitly authorized and bounded.

### II. Zero-Telemetry & Local-First Privacy
User confidentiality is paramount. All scan targets, session cookies, authorization headers, scan histories, and generated reports MUST remain 100% local on the user's workstation. No analytics, telemetry, or external reporting services may be integrated without explicit, opt-in consent. Persistence MUST use the embedded local SQLite database running in Write-Ahead Logging (`WAL`) mode with strict local filesystem permissions.

### III. Asynchronous Rust Core & Zero UI Blocking
All computationally intensive tasks, network requests, TCP socket connections, and database transactions MUST execute asynchronously within the native Rust/Tokio runtime. The desktop frontend MUST remain fluid, responsive, and render at 60 FPS without ever being blocked by running audits, background watchdog schedules, or batch fleet scans. Long-running scans MUST stream progress events to the frontend via Tauri event channels.

### IV. Type-Safe Tauri IPC & Shared Schemas
Communication across the Tauri IPC boundary MUST be strictly typed and validated. Every Rust command parameter and return payload MUST serialize to and deserialize from strictly typed Serde structures that mirror TypeScript interfaces on the frontend. Untyped `any` or ambiguous dictionary payloads across the IPC bridge are prohibited.

### V. Standardized Scoring & OWASP-Mapped Findings
Every security finding MUST map to an established industry classification (OWASP Top 10, CWE, or RFC standard) with a deterministic severity rating (Critical, High, Medium, Low, Info). The overall Security Posture Score (0–100) MUST follow an explainable, weighted deduction model. Each reported vulnerability MUST provide concise, actionable remediation advice, concrete code patches, and reproducible validation evidence (such as sanitized cURL strings).

## Technical Constraints & Security Standards

### Desktop Architecture & Tech Stack
- **Native Backend**: Rust 1.80+, Tauri v2 desktop framework with minimal external dependencies.
- **Asynchronous Runtime**: `tokio` with full features, `reqwest` for HTTP client interactions with safe timeout ceilings (default 10s per request).
- **Embedded Database**: Local `rusqlite` with bundled SQLite in `WAL` mode; all queries MUST use parameterized statements to prevent SQL injection.
- **Frontend Stack**: Svelte 5 utilizing modern Runes (`$state`, `$derived`, `$effect`), SvelteKit with `@sveltejs/adapter-static`, TypeScript (strict mode), and Tailwind CSS v4.
- **Icons & UI Assets**: `lucide-svelte` / `@lucide/svelte` for consistent iconography.

### Security Standards & Data Sanitization
- **Export Sanitization**: All tabular exports (CSV) MUST sanitize cell prefixes (`=`, `+`, `-`, `@`) to prevent CSV/formula injection in spreadsheet applications.
- **Header & Token Security**: Custom authorization headers and cookie values configured by users MUST be masked in default UI views and omitted from non-essential logging.
- **Report Rendering**: PDF and print rendering MUST run within an isolated webview context with strict Content Security Policy (`CSP`) preventing remote script execution.

## Development Workflow & Quality Gates

### Spec-Driven Development (SDD)
- All new features, security scanning heuristics, or architectural modifications MUST start with a specification (`/speckit-specify`) and technical plan (`/speckit-plan`) before writing implementation code.
- Requirements and acceptance criteria MUST be validated against this Constitution.

### Verification & Quality Gates
- **Rust Verification**: `cargo check` and `cargo clippy -- -D warnings` MUST complete cleanly before committing Rust changes.
- **Frontend Verification**: `npm run check` (`svelte-check`) MUST pass with zero TypeScript and Svelte compilation errors.
- **Heuristic Integrity**: Every new vulnerability detector or parser MUST include unit tests utilizing mock HTTP responses, sample HTML/headers, or synthetic test fixtures.

## Governance

This Constitution supersedes informal guidelines and ad-hoc practices. All pull requests, code reviews, and automated agent operations MUST verify compliance with these principles.

- **Amendments**: Amending this constitution requires documented justification, an evaluation of backwards compatibility, and a version increment.
- **Versioning Policy**:
  - **MAJOR**: Changes that redefine, remove, or contradict existing core principles.
  - **MINOR**: Adding new principles, expanding security standards, or introducing new tooling constraints.
  - **PATCH**: Non-semantic clarifications, typographical corrections, or formatting adjustments.
- **Compliance Reviews**: Antigravity agents and contributors MUST reference `.specify/memory/constitution.md` during feature planning and verification phases.

**Version**: 1.0.0 | **Ratified**: 2026-09-04 | **Last Amended**: 2026-09-04
