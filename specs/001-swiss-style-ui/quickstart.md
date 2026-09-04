# Quickstart & Validation Guide: Swiss Style UI Redesign

**Feature**: `001-swiss-style-ui`
**Status**: Completed

## Prerequisites

- Node.js (v18+) and `npm` installed.
- Rust toolchain (1.80+) installed with `cargo`.

---

## Validation Scenarios

### Scenario 1: Verify Dual-Theme Toggle (Swiss Dark ↔ Swiss Light)
1. Launch the local development frontend:
   ```bash
   npm run dev
   ```
2. Navigate to `http://localhost:5173`.
3. Locate the theme toggle pill in the top navigation bar.
4. Click the theme toggle to activate **Swiss Light**:
   - Canvas background shifts to stark white `#ffffff`.
   - Typography shifts to high-contrast jet-black `#09090b`.
   - Grid dividers render as crisp dark hairlines.
   - Severity badges render with solid signal-colored borders and tags.
5. Click again to return to **Swiss Dark**:
   - Canvas background returns to deep pitch `#09090b`.
   - Typography shifts to clean white `#ffffff`.
6. Reload the browser page:
   - Verify that the chosen theme state persisted across reload via `localStorage`.

---

### Scenario 2: Audit Scan Grid & Typographic Scale
1. Enter `https://example.com` into the URL command bar and trigger a scan.
2. In the **Posture Audit** workspace:
   - Verify the **Posture Score block** displays bold, oversized numerals (`font-black`) with clean `/100` meta text rather than a blurry circular gradient.
   - Verify that the finding list presents sharp rectangular cards with vertical left-side severity signal strips (Red for Critical/High, Amber for Medium, Blue for Low, Gray for Info).
   - Verify that expanding a finding reveals monospace technical details, HTTP headers, and remediation blocks aligned with crisp cell borders.

---

### Scenario 3: Modular Workspace Navigation & Responsive Grid
1. Use keyboard shortcuts or click workspace tabs to navigate between:
   - **01/AUDIT** (Posture Audit)
   - **02/PORTS** (Port Matrix Grid)
   - **03/DNS** (Anti-Spoofing & DNSSEC)
   - **04/RECON** (Surface Reconnaissance)
2. Verify that workspace headings render in uppercase tracking with bold numerical prefixes (`01/`, `02/`, etc.).
3. Resize the browser window from 1280px down to 1024px and up to 1920px:
   - Verify that columns align along consistent vertical margins without overlapping or clipped labels.

---

### Scenario 4: Automated Verification & Type Checks
Run the automated compiler and type checks to ensure zero regressions:
```bash
# Verify Svelte 5 and TypeScript compilation
npm run check

# Verify Rust backend build
cd src-tauri && cargo check
```
