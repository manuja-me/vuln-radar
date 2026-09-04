# Tasks: Swiss Style UI Redesign

**Feature**: `001-swiss-style-ui`
**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish core design tokens, CSS custom properties, and theme types

- [X] T001 Configure Swiss typographic scale, signal color tokens, and dual-theme variables in `src/app.css`
- [X] T002 Add `SwissTheme` state types and theme definitions in `src/lib/types.ts`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core visual foundation and theme switching infrastructure that all user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T003 Implement global theme initialization and `localStorage` persistence in `src/app.html` and `src/routes/+layout.svelte`
- [X] T004 [P] Redesign `src/lib/components/SeverityBadge.svelte` using rectangular geometry, uppercase monospace labels, and Swiss signal color tokens

**Checkpoint**: Foundation ready — theme switching active and foundational badges available for user stories.

---

## Phase 3: User Story 1 - Structured Posture Evaluation & Score Hierarchy (Priority: P1) 🎯 MVP

**Goal**: Deliver the primary audit dashboard featuring stark typographic score presentation, modular asymmetric grid layout, and high-contrast severity counters.

**Independent Test**: Run a scan against any target URL and verify that the security posture score displays as bold, oversized typography with calibrated geometric meter, clear letter grade, and modular severity summaries across both Swiss Dark and Swiss Light themes.

### Implementation for User Story 1

- [X] T005 [P] [US1] Redesign `src/lib/components/ScoreGauge.svelte` to implement typographic score presentation (oversized numeral, `/100` meta, calibrated tick bar)
- [X] T006 [US1] Redesign Posture Audit workspace layout in `src/routes/+page.svelte` to implement the asymmetric modular grid (target summary, score card, severity counters)
- [X] T007 [US1] Validate User Story 1 independently in browser for visual hierarchy and score rendering in both themes

**Checkpoint**: At this point, User Story 1 is fully functional and delivers a viable, high-contrast MVP audit dashboard.

---

## Phase 4: User Story 2 - Finding Inspector & Actionable Evidence Clarity (Priority: P2)

**Goal**: Present vulnerability findings, HTTP headers, CVE details, and AI remediation actions with high legibility, clean tabular boundaries, and monospace code blocks.

**Independent Test**: Expand finding cards in the audit list; verify that technical descriptions, CVE tags, parameter values, and remediation prompt pills render with flush-left alignment, sharp hairlines, and clear copy affordances.

### Implementation for User Story 2

- [X] T008 [P] [US2] Redesign `src/lib/components/FindingCard.svelte` with flush-left severity signal strips, tabular CVE/CVSS metadata, and clean monospace evidence disclosure
- [X] T009 [US2] Update finding filtering pills, search input, and AI remediation tech selection grid in `src/routes/+page.svelte` to match Swiss typographic rules
- [X] T010 [US2] Validate User Story 2 independently in browser by expanding findings and verifying copyable evidence blocks

**Checkpoint**: User Stories 1 AND 2 operate together as a complete posture audit and vulnerability inspection workstation.

---

## Phase 5: User Story 3 - Clean Modular Navigation & Workspace Switching (Priority: P3)

**Goal**: Harmonize global navigation, workspace toolbars, and overlay dialogs with the Swiss modular grid and typographic scale.

**Independent Test**: Navigate across all 8 workspaces (Port Matrix, DNS, Surface Recon, Watchdog, Fleet Batch, History, Settings); verify that active tab indicators, data grids, and modals use crisp 1px hairlines and consistent typography.

### Implementation for User Story 3

- [X] T011 [P] [US3] Redesign `src/lib/components/Navbar.svelte` with asymmetrical grid, integrated command bar, numbered workspace tabs (`01/AUDIT`), and `[DARK]` / `[LIGHT]` theme toggle
- [X] T012 [US3] Restructure remaining workspaces (Port Matrix, DNS, Surface Recon, Fleet Batch, Watchdog, History, Settings) in `src/routes/+page.svelte` with tabular modular grid layouts
- [X] T013 [P] [US3] Refactor modal dialogs and drawers (`src/lib/components/SettingsModal.svelte`, `src/lib/components/HistoryModal.svelte`, `src/lib/components/ExportModal.svelte`, `src/lib/components/ShortcutsModal.svelte`, `src/lib/components/Toast.svelte`) with sharp hairlines and high-contrast scrims
- [X] T014 [US3] Validate User Story 3 independently by cycling through all 8 workspaces and testing modal drawers

**Checkpoint**: Complete Swiss Style Design transformation realized across the entire application.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Quality gates, type checking, accessibility validation, and documentation

- [X] T015 [P] Run compiler and type checks via `npm run check`
- [X] T016 [P] Execute end-to-end verification following `specs/001-swiss-style-ui/quickstart.md`
- [X] T017 Update documentation in `README.md` highlighting the Swiss Style UI design and theme toggle

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories.
- **User Stories (Phase 3+)**: All depend on Foundational phase completion.
  - **User Story 1 (P1)**: Independent MVP slice.
  - **User Story 2 (P2)**: Integrates with US1 audit list, independently testable.
  - **User Story 3 (P3)**: Builds on global layout and navigation.
- **Polish (Phase 6)**: Runs after desired user stories are completed.

### Parallel Opportunities

- Within Phase 1: `T001` and `T002` can be developed concurrently.
- Within Phase 2: `T004` (`SeverityBadge.svelte`) can be developed in parallel with `T003`.
- Within Phase 3: `T005` (`ScoreGauge.svelte`) can be developed in parallel with `T006` (`+page.svelte` audit layout).
- Within Phase 4: `T008` (`FindingCard.svelte`) can be developed in parallel with `T009`.
- Within Phase 5: `T011` (`Navbar.svelte`) and `T013` (Modal dialogs) can be developed in parallel.
- Polish phase: `T015` and `T016` can run in parallel.

---

## Implementation Strategy

### MVP First (User Story 1 Only)
1. Complete Setup (T001, T002).
2. Complete Foundational (T003, T004).
3. Complete User Story 1 (T005, T006, T007).
4. **STOP & VALIDATE**: Verify that the primary posture score and audit summary demonstrate the Swiss aesthetic cleanly.

### Incremental Delivery
1. Add User Story 2 (T008, T009, T010) → Refined finding inspection and evidence clarity.
2. Add User Story 3 (T011, T012, T013, T014) → Full 8-workspace navigation and modal styling.
3. Run Polish (T015, T016, T017) → Type safety, verification, and documentation.
