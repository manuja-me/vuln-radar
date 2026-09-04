# Data Model: Swiss Style UI Redesign

**Feature**: `001-swiss-style-ui`
**Status**: Completed

## 1. Theme Configuration Entity

Represents the active visual theme state across the application.

```typescript
export type SwissTheme = "swiss-dark" | "swiss-light";

export interface ThemeState {
  current: SwissTheme;
  persistedKey: "vulnradar_theme";
}
```

### Validation & Rules
- Default theme is `"swiss-dark"`.
- Valid values are strictly `"swiss-dark"` and `"swiss-light"`.
- Persisted to browser `localStorage` on change and synced with `document.documentElement.dataset.theme`.

---

## 2. Signal Color & Severity Model

Maps backend security severity enumerations to the Swiss graphic palette.

```typescript
export type SeverityLevel = "critical" | "high" | "medium" | "low" | "info" | "pass";

export interface SeverityVisualToken {
  level: SeverityLevel;
  label: string;             // Uppercase display label (e.g., "CRITICAL", "HIGH")
  textColor: string;         // CSS variable reference for text
  borderColor: string;       // Hairline border accent
  badgeBg: string;           // Badge fill
  signalDot: string;         // Geometric indicator dot color
}
```

### Mapping Matrix

| Severity | Label | Dark Token | Light Token | Border Accent |
| :--- | :--- | :--- | :--- | :--- |
| `critical` | `CRITICAL` | `text-red-400` | `text-red-600` | `border-l-4 border-l-red-500` |
| `high` | `HIGH` | `text-orange-400` | `text-orange-600` | `border-l-4 border-l-orange-500` |
| `medium` | `MEDIUM` | `text-amber-400` | `text-amber-600` | `border-l-4 border-l-amber-500` |
| `low` | `LOW` | `text-blue-400` | `text-blue-600` | `border-l-4 border-l-blue-500` |
| `info` | `INFO` | `text-zinc-400` | `text-zinc-600` | `border-l-4 border-l-zinc-500` |
| `pass` | `PASS` | `text-emerald-400`| `text-emerald-600`| `border-l-4 border-l-emerald-500`|

---

## 3. Posture Score Display Entity

Represents the mathematical breakdown rendered by the redesigned `ScoreGauge.svelte`.

```typescript
export interface PostureScorePresentation {
  score: number;             // Integer 0 - 100
  grade: string;             // "A+", "A", "B", "C", "D", "F"
  benchmarkLabel: string;    // "EXCELLENT", "ACCEPTABLE", "DEGRADED", "CRITICAL RISK"
  colorClass: string;        // Dynamic signal color token based on score
  deductions: {
    criticalCount: number;
    highCount: number;
    mediumCount: number;
    lowCount: number;
  };
}
```

---

## 4. Modular Workspace Grid Layout

Defines the spatial geometry for the 8 workstations in `+page.svelte`.

```typescript
export interface WorkspaceGridDefinition {
  id: "audit" | "ports" | "dns" | "recon" | "batch" | "watchdog" | "history" | "settings";
  title: string;
  code: string;              // e.g. "01/AUDIT", "02/PORTS", "03/DNS"
  layoutType: "asymmetric-split" | "full-data-grid" | "centered-dialog";
  primaryColumnSpan: number; // e.g. 4 of 12 cols
  secondaryColumnSpan: number; // e.g. 8 of 12 cols
}
```
