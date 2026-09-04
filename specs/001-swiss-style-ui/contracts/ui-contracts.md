# UI Contracts: Swiss Style Design System

**Feature**: `001-swiss-style-ui`
**Status**: Completed

## 1. Global CSS Theme Contract (`app.css`)

The core design system contract is exposed via CSS custom properties and utility classes:

```css
:root {
  /* Default: Swiss Dark */
  --color-canvas: #09090b;
  --color-surface: #121215;
  --color-surface-hover: #18181b;
  --color-hairline: #27272a;
  --color-hairline-strong: #3f3f46;
  --color-text-headline: #ffffff;
  --color-text-body: #d4d4d8;
  --color-text-muted: #71717a;
  --color-signal-red: #ef4444;
  --color-signal-amber: #f59e0b;
  --color-signal-green: #10b981;
}

[data-theme="swiss-light"] {
  /* Swiss Light (Poster / Print style) */
  --color-canvas: #ffffff;
  --color-surface: #f4f4f5;
  --color-surface-hover: #e4e4e7;
  --color-hairline: #18181b;
  --color-hairline-strong: #09090b;
  --color-text-headline: #09090b;
  --color-text-body: #27272a;
  --color-text-muted: #52525b;
  --color-signal-red: #dc2626;
  --color-signal-amber: #d97706;
  --color-signal-green: #059669;
}
```

---

## 2. Component Interface Contracts

### `ScoreGauge.svelte`
- **Props**:
  - `score: number` (0 to 100)
  - `grade?: string` (Optional override)
  - `compact?: boolean` (For drawer/modal headers)
- **Visual Output**:
  - Hero display numeral in `font-black` (e.g. `85`) with adjacent `/100` meta label.
  - Grade badge in monospace (`[GRADE: B+]`).
  - Segmented linear status bar calibrated in 10-point ticks.

### `SeverityBadge.svelte`
- **Props**:
  - `severity: "critical" | "high" | "medium" | "low" | "info" | "pass"`
  - `count?: number` (Optional counter tag)
  - `interactive?: boolean` (For filter pills)
  - `active?: boolean` (Active filter state)
- **Visual Output**:
  - Sharp rectangle with 1px border.
  - Monospaced, uppercase label (`CRITICAL`).
  - Solid signal color dot or background highlight on active state.

### `FindingCard.svelte`
- **Props**:
  - `finding: Finding`
  - `expanded?: boolean`
  - `onToggle?: () => void`
- **Visual Output**:
  - Flush-left card anchored by a 4px left-border signal strip corresponding to severity.
  - Two-tier header: Title in `font-bold` and OWASP category in uppercase tracking.
  - Tabular metadata drawer for CVE ID, CVSS vector, and remediation code blocks.

### `Navbar.svelte`
- **Props**:
  - `targetUrl: string`
  - `isScanning: boolean`
  - `currentWorkspace: string`
  - `currentTheme: "swiss-dark" | "swiss-light"`
  - `onScan: (url: string) => void`
  - `onWorkspaceChange: (ws: string) => void`
  - `onThemeToggle: () => void`
- **Visual Output**:
  - Integrated top grid bar with high-contrast workspace selector.
  - Command-line style target input with sharp rectangular scan action button (`[RUN SCAN]`).
  - Tactile theme switch pill showing active state (`● DARK` / `○ LIGHT`).
