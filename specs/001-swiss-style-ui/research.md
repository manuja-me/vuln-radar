# Technical Research: Swiss Style UI Redesign

**Feature**: `001-swiss-style-ui`
**Status**: Completed

## 1. Typographic Architecture & Font Stack

### Decision
Adopt a strict grotesque sans-serif hierarchy paired with monospaced data figures, loaded with offline-safe local font fallbacks to comply with Constitution Principle II (Zero Telemetry & Local Privacy).

- **Primary Grotesque Sans**: `Inter, "Helvetica Neue", Arial, -apple-system, BlinkMacSystemFont, sans-serif`
- **Technical Monospace**: `"JetBrains Mono", ui-monospace, Menlo, Monaco, Consolas, monospace`
- **Typographic Scale & Leading**:
  - **Display / Hero**: `text-4xl` to `text-6xl`, `font-black` (900 weight), tight tracking (`tracking-tighter`), line-height 1.0.
  - **Workspace & Section Headers**: `text-xl` to `text-2xl`, `font-bold` (700 weight), uppercase with wide letter-spacing (`tracking-wider`).
  - **Meta Labels & Category Pills**: `text-xs`, `font-semibold` (600 weight), uppercase (`tracking-widest`).
  - **Body Copy**: `text-sm`, `font-normal` (400 weight), leading relaxed (1.6) for optimal scanning legibility.
  - **Technical Findings & Evidence**: `text-xs` to `text-sm`, `font-mono`, tabular numbers (`tabular-nums`).

### Rationale
Swiss typography (International Typographic Style) relies on stark hierarchy between colossal display weights and highly legible, unembellished body copy. Avoids remote Google Fonts CDN queries at runtime.

### Alternatives Considered
- *Skeuomorphic / Rounded Cyberpunk styling* (existing): Relied on cyan glows, rounded-xl cards, and subtle drop shadows. Rejected in favor of Swiss clarity.
- *Serif editorial style*: Considered for executive reports, but rejected for a desktop security workstation which demands industrial precision.

---

## 2. Grid System & Visual Geometry

### Decision
Replace rounded bubble containers (`rounded-xl`, `rounded-2xl`, drop shadows) with a rigid, asymmetrical **Modular Grid** utilizing crisp hairlines (`1px` borders), sharp corners (`rounded-none` / `rounded-sm`), and intentional asymmetric alignments.

- **Layout Structure**:
  - Global top bar anchored by high-contrast hairline borders (`border-b`).
  - Two-column asymmetric workspace layout on desktop: Left column (30-35% width) for target overview, Score Block, and severity counts; Right column (65-70% width) for the active finding inspector or data grid.
  - Rectangular modular cards with subtle hover invert or hairline shifts rather than glowing shadows.
- **Rhythm & Spacing**:
  - Base spacing rhythm in multiples of 4px/8px (`p-4`, `p-6`, `gap-4`, `gap-6`).
  - Strict horizontal and vertical grid alignment across all cards, inputs, and modals.

### Rationale
The hallmark of Swiss graphic design is the mathematical grid conceived by Josef Müller-Brockmann. It guarantees rapid visual indexing for security engineers scanning hundreds of parameters.

---

## 3. Dual-Theme Palette & Contrast System

### Decision
Implement a zero-runtime-overhead dual-theme system (`swiss-dark` and `swiss-light`) controlled via `data-theme` attribute on the root HTML element and persisted in `localStorage`.

### Palette Specification

| Token | Swiss Dark Mode | Swiss Light Mode (Poster) | Functional Usage |
| :--- | :--- | :--- | :--- |
| `--bg-canvas` | `#09090b` (Deep Pitch) | `#fcfcfc` (Stark White Paper) | Application background |
| `--bg-surface` | `#121215` (Graphite) | `#f4f4f5` (Neutral Gray) | Modular card surfaces |
| `--border-hairline` | `#27272a` (Subtle Zinc) | `#18181b` (Sharp Charcoal/Black) | Grid lines & card borders |
| `--text-primary` | `#f4f4f5` (High Contrast White) | `#09090b` (Jet Black) | Headings, scores, primary metrics |
| `--text-muted` | `#71717a` (Cool Gray) | `#52525b` (Subtle Slate) | Labels, timestamps, descriptions |
| `--signal-critical` | `#ef4444` (Swiss Signal Red) | `#dc2626` (Pure Red Ink) | Critical CVEs, RCE risk, fail status |
| `--signal-high` | `#f97316` (International Orange) | `#ea580c` (Bold Orange) | High severity alerts |
| `--signal-medium` | `#eab308` (Industrial Yellow) | `#ca8a04` (Warm Ochre) | Medium severity warnings |
| `--signal-low` | `#3b82f6` (Signal Blue) | `#2563eb` (Cobalt Blue) | Low severity notes |
| `--signal-pass` | `#10b981` (Signal Green) | `#059669` (Forest Green) | Hardened headers, 100% pass |

### Rationale
Allows security researchers working in dim SOC environments to use Swiss Dark, while providing the iconic stark white/black/red International Typographic poster aesthetic in Swiss Light for high ambient lighting and executive presentations.

---

## 4. Component Refactoring Strategy

### Decision
Refactor core components incrementally without modifying underlying Svelte 5 state models or Rust IPC data structures:

1. **`Navbar.svelte`**:
   - Asymmetrical layout: Brand mark with bold tracking, target URL input styled as an integrated command bar with sharp borders, workspace tabs rendered as crisp underlined items.
   - Theme toggle button with tactile `[DARK]` / `[LIGHT]` monospaced indicator.
2. **`ScoreGauge.svelte`**:
   - Replace generic curved SVG circular meter with a prominent typographic score block: huge bold score (`88`), maximum baseline (`/100`), grade letter (`B+`), and segmented geometric bar indicator.
3. **`SeverityBadge.svelte`**:
   - Sharp rectangular badge with uppercase tracking and solid signal color dot.
4. **`FindingCard.svelte`**:
   - Modular tabular format with stark severity border line, flush-left title, clear OWASP category badge, and clean monospace evidence disclosure.
5. **Modal Overlays (`SettingsModal`, `ExecutiveReportModal`, etc.)**:
   - Sharp rectangular dialogs with solid backdrop scrims and precise grid dividers.
