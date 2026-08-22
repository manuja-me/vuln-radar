---
name: frontend-design
description: >-
  Expert guide for designing elegant, modern, enterprise-grade cybersecurity UIs and components in VulnRadar.
  Use this skill whenever designing or modifying UI components, layout structures, modal dialogs, color systems, typography, or micro-interactions in Svelte 5 and Tailwind CSS.
---

# 🎨 VulnRadar Frontend Design System & UI/UX Guidelines

This skill provides an authoritative design specification for building **elegant, ultra-modern, professional cybersecurity interfaces** for VulnRadar.

---

## 💎 Design Philosophy: Enterprise Dark Security

VulnRadar follows the **High-End Security & Developer Tool Aesthetic** (inspired by Linear, Vercel, Raycast, and modern SIEM dashboards):

1. **Depth via Subtle Layering**: Layer dark background surfaces (`slate-950` → `slate-900` → `slate-800`) with semi-transparent backdrops (`bg-slate-900/60`, `backdrop-blur-md`).
2. **Neon Precision Accents**: Use high-contrast cyber accents (`cyan-400`, `emerald-400`, `rose-400`) intentionally against deep obsidian/slate dark backgrounds.
3. **Information Density with Breathing Room**: Technical telemetry, CVE badges, and metrics must be scannable with consistent padding (`p-4` to `p-6`) and crisp typography.
4. **Zero Generic "AI Slop"**: Avoid unstyled default inputs, raw unrounded boxes, jarring full-opacity solid alert boxes, or mismatched font weights.

---

## 🎨 Color Palette & Design Tokens

### 1. Base Surfaces
- **App Canvas**: `bg-slate-950` (or `bg-[#0B0F19]`)
- **Card Background**: `bg-slate-900/60` with `border border-slate-800`
- **Hover Surface**: `hover:bg-slate-900 hover:border-cyan-500/30 transition-all`
- **Subtle Inset / Code**: `bg-slate-950/80`

### 2. Semantic Security Severities
Always use translucent tint backgrounds paired with colored borders and text:
- **Critical Risk**: `text-rose-400 bg-rose-500/10 border border-rose-500/30`
- **High Risk**: `text-orange-400 bg-orange-500/10 border border-orange-500/30`
- **Medium Risk**: `text-amber-400 bg-amber-500/10 border border-amber-500/30`
- **Low Risk**: `text-blue-400 bg-blue-500/10 border border-blue-500/30`
- **Info / Low Priority**: `text-slate-300 bg-slate-800/80 border border-slate-700/60`
- **Secure / Passed**: `text-emerald-400 bg-emerald-500/10 border border-emerald-500/30`

### 3. Brand & Action Accent
- **Primary Cyan**: `bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold shadow-lg shadow-cyan-500/20`
- **Secondary Glass Action**: `bg-slate-800/80 hover:bg-slate-800 text-slate-300 hover:text-white border border-slate-700/60`

---

## 🔤 Typography Guidelines

- **Technical Data & Values**: Always use `font-mono` for URLs, HTTP codes, CVE IDs, IPs, hashes, latency (ms), and timestamps.
- **Headings**: Use `font-black` or `font-extrabold tracking-tight text-white` for primary dashboard titles.
- **Section Labels**: Use `text-xs font-bold uppercase tracking-wider text-slate-400` for section headers and metric captions.
- **Body & Descriptions**: Use `text-xs text-slate-300 leading-relaxed` for vulnerability narratives and remediation text.

---

## 🧩 Svelte 5 Component Best Practices

1. **State & Reactivity (Runes)**:
   ```svelte
   <script lang="ts">
     let { isOpen = false, data = [], onClose }: Props = $props();
     let activeTab = $state("all");
     let filtered = $derived(data.filter(...));
   </script>
   ```
2. **Accessibility (A11y)**:
   - Always accompany interactive elements with accessible `<button type="button">` wrappers rather than bare `<div onclick=...>`.
   - Associate form controls with explicit IDs and matching labels.
3. **Modals & Overlays**:
   - Backdrop: `fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-fade-in`
   - Modal Container: `bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-3xl max-h-[85vh] flex flex-col shadow-2xl overflow-hidden`

---

## 🖨️ Print & PDF Export Rules

When designing printable or exportable views:
- Add `print:hidden` to navbars, close buttons, action buttons, and backdrops.
- Add `print:bg-white print:text-black print:border-slate-300` to containers so exported PDFs render with high-contrast, paper-friendly clarity.

---

## 📋 Quality & Polish Checklist

Before shipping any UI changes:
- [ ] Run `npm run check` and ensure **0 errors and 0 warnings**.
- [ ] Run `npm run build` to verify Vite bundle compilation.
- [ ] Check responsive breakpoints (`sm:`, `md:`, `lg:`) for mobile and desktop window resizing.
- [ ] Verify Lucide icons have consistent size (`w-4 h-4` or `w-5 h-5`) and semantic color tokens.
