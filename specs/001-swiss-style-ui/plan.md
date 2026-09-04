# Implementation Plan: Swiss Style UI Redesign

**Branch**: `001-swiss-style-ui` | **Date**: 2026-09-04 | **Spec**: [specs/001-swiss-style-ui/spec.md](spec.md)

**Input**: Feature specification from `specs/001-swiss-style-ui/spec.md`

## Summary

Redesign the VulnRadar desktop security scanner user interface following the International Typographic (Swiss) Style. The overhaul introduces a strict modular grid system, high-contrast grotesque typography, flush-left alignments, crisp 1px dividing hairlines, and an unembellished signal color palette. It implements a user-toggleable dual-theme system supporting both **Swiss Dark** (deep pitch canvas for SOC/workstation environments) and **Swiss Light** (stark white/cream poster aesthetic with jet-black typography and vivid red signal accents). All changes preserve existing Svelte 5 state models and Rust IPC backend interfaces.

## Technical Context

**Language/Version**: TypeScript ~5.6, Svelte 5.0 (Runes), Rust 1.80+ (Tauri v2)

**Primary Dependencies**: Svelte 5, Tailwind CSS v4, `@lucide/svelte`, `lucide-svelte`, `@tauri-apps/api`

**Storage**: Local embedded SQLite via `rusqlite` (WAL mode) and browser `localStorage` for theme preference (`vulnradar_theme`)

**Testing**: `npm run check` (`svelte-check`), `cargo check`

**Target Platform**: Cross-platform Desktop (Windows, macOS, Linux via Tauri v2)

**Project Type**: Desktop Security Workstation Application

**Performance Goals**: Fluid 60 FPS transitions, zero UI thread blocking, instant theme switching without window reloads

**Constraints**: Zero external Google Font CDN dependencies (100% offline-first & zero-telemetry per Constitution Principle II), WCAG 2.1 AA contrast ratios (>4.5:1) for all typography and status signals

**Scale/Scope**: 8 Workspaces, 13+ Modals/Drawers, finding cards, gauges, tables in `src/routes/+page.svelte` and `src/lib/components/`

## Constitution Check

*GATE: Passed before Phase 0 research. Re-evaluated post Phase 1 design.*

- **Principle I: Safe & Non-Destructive Auditing** — **PASS**: UI transformation only affects visual rendering and presentation of audit data; scanning logic remains untouched.
- **Principle II: Zero-Telemetry & Local-First Privacy** — **PASS**: Theme preferences stored locally in browser `localStorage`; typography uses local system font stacks without remote CDN calls.
- **Principle III: Asynchronous Rust Core & Zero UI Blocking** — **PASS**: High-performance CSS transforms and Svelte 5 runes guarantee 60 FPS rendering without taxing the main thread.
- **Principle IV: Type-Safe Tauri IPC & Shared Schemas** — **PASS**: Retains all existing TypeScript DTO interfaces in `src/lib/types.ts` without modifying backend payload schemas.
- **Principle V: Standardized Scoring & OWASP-Mapped Findings** — **PASS**: Redesigned score presentation and severity badges map directly to existing 0-100 scores and OWASP categories with high-contrast signal colors.

## Project Structure

### Documentation (this feature)

```text
specs/001-swiss-style-ui/
├── spec.md              # Feature specification
├── plan.md              # This implementation plan
├── research.md          # Technical research & design decisions
├── data-model.md        # Theme, signal palette, and layout models
├── contracts/           # Component & CSS design system contracts
│   └── ui-contracts.md
├── checklists/
│   └── requirements.md  # Specification quality checklist
└── quickstart.md        # Validation & testing scenarios
```

### Source Code (affected components)

```text
src/
├── app.css                                  # Core design system tokens, themes & fonts
├── routes/
│   └── +page.svelte                         # 8-workspace modular grid layouts & navigation
└── lib/
    ├── types.ts                             # Theme type extensions
    └── components/
        ├── Navbar.svelte                    # Asymmetrical Swiss top bar & theme toggle
        ├── ScoreGauge.svelte                # Typographic score block & calibrated meter
        ├── SeverityBadge.svelte             # High-contrast signal indicator tags
        ├── FindingCard.svelte               # Tabular modular finding inspector cards
        ├── BatchScanModal.svelte            # Structured data fleet grid
        ├── ExecutiveReportModal.svelte      # Typographic print/PDF report styling
        ├── ExportModal.svelte               # Clean export action drawer
        ├── HistoryModal.svelte              # Tabular scan history drawer
        ├── MonitorModal.svelte              # Watchdog target matrix
        ├── ScanOptionsModal.svelte          # Scan parameter controls
        ├── SettingsModal.svelte             # Settings hub with theme selector
        ├── ShortcutsModal.svelte            # Keyboard shortcuts grid
        └── Toast.svelte                     # Sharp signal notification banner
```

**Structure Decision**: Component-level modernization within existing SvelteKit project structure. No directory reorganization required.

## Complexity Tracking

*No constitutional gate violations.*
