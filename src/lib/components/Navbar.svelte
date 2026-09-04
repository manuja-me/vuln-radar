<script lang="ts">
  import { onMount } from "svelte";
  import {
    ShieldCheck,
    Search,
    Loader2,
    History,
    FileDown,
    Settings,
    RotateCw,
    Sun,
    Moon,
  } from "lucide-svelte";
  import type { SwissTheme } from "$lib/types";

  let {
    targetUrl = $bindable(""),
    isScanning = false,
    hasReport = false,
    hasCustomOptions = false,
    activeMonitorsCount = 0,
    onScan,
    onOpenHistory,
    onOpenSettings,
    onOpenExport,
  }: {
    targetUrl: string;
    isScanning: boolean;
    hasReport: boolean;
    hasCustomOptions?: boolean;
    activeMonitorsCount?: number;
    onScan: () => void;
    onOpenHistory: () => void;
    onOpenSettings: (tab?: "params" | "ports" | "watchdog" | "batch" | "shortcuts" | "data") => void;
    onOpenExport: () => void;
  } = $props();

  let currentTheme = $state<SwissTheme>("swiss-dark");

  onMount(() => {
    try {
      const saved = localStorage.getItem("vulnradar_theme") as SwissTheme;
      if (saved === "swiss-light" || saved === "swiss-dark") {
        currentTheme = saved;
      } else {
        currentTheme = (document.documentElement.getAttribute("data-theme") as SwissTheme) || "swiss-dark";
      }
    } catch {}
  });

  function toggleTheme() {
    currentTheme = currentTheme === "swiss-dark" ? "swiss-light" : "swiss-dark";
    try {
      localStorage.setItem("vulnradar_theme", currentTheme);
      document.documentElement.setAttribute("data-theme", currentTheme);
    } catch {}
  }

  function handleSubmit(e: Event) {
    e.preventDefault();
    if (!targetUrl.trim() || isScanning) return;
    onScan();
  }
</script>

<!-- Swiss Window Toolbar & Integrated Command Bar -->
<header
  data-tauri-drag-region
  class="bg-[var(--color-surface)] border-b border-[var(--color-hairline)] sticky top-0 z-40 px-4 py-2.5 flex items-center justify-between gap-4 titlebar-drag desktop-select-none print:hidden flex-shrink-0 transition-colors"
>
  <!-- Window Drag / App Title & Status -->
  <div class="flex items-center gap-3 no-drag flex-shrink-0">
    <div class="flex items-center gap-2">
      <div
        class="w-7 h-7 rounded-none bg-[var(--color-surface)] border border-[var(--color-hairline)] border-l-2 border-l-[var(--color-signal-red)] flex items-center justify-center text-[var(--color-text-headline)]"
      >
        <ShieldCheck class="w-4 h-4 text-[var(--color-signal-red)]" />
      </div>
      <div class="flex flex-col">
        <div class="flex items-center gap-1.5 leading-none">
          <span class="text-xs font-black tracking-tight text-[var(--color-text-headline)] font-mono uppercase">
            VULNRADAR
          </span>
          <span
            class="px-1 py-0.2 text-[9px] font-mono font-bold bg-[var(--color-canvas)] text-[var(--color-text-muted)] border border-[var(--color-hairline)] rounded-none"
          >
            v0.7.0
          </span>
        </div>
        <span class="text-[9px] text-[var(--color-text-muted)] font-mono uppercase tracking-widest mt-0.5 font-semibold">
          SECURITY WORKSTATION
        </span>
      </div>
    </div>
  </div>

  <!-- Central Command Bar / Target URL Input -->
  <form
    onsubmit={handleSubmit}
    class="w-full max-w-xl flex items-center gap-1.5 no-drag"
  >
    <div class="relative flex-1 group">
      <div
        class="absolute inset-y-0 left-0 pl-2.5 flex items-center pointer-events-none text-[var(--color-text-muted)] group-focus-within:text-[var(--color-text-headline)] transition-colors"
      >
        <Search class="w-3.5 h-3.5" />
      </div>
      <input
        type="text"
        bind:value={targetUrl}
        placeholder="TARGET URL (E.G. EXAMPLE.COM OR HTTP://LOCALHOST:8000)..."
        disabled={isScanning}
        class="w-full pl-8 pr-12 py-1.5 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs text-[var(--color-text-headline)] placeholder-[var(--color-text-muted)] font-mono uppercase transition-all disabled:opacity-60 focus:outline-none"
      />
      <div
        class="absolute inset-y-0 right-0 pr-2 flex items-center pointer-events-none gap-1"
      >
        <kbd class="px-1.5 py-0.2 text-[9px] font-mono bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none text-[var(--color-text-muted)]">
          ⌘K
        </kbd>
      </div>
    </div>

    <!-- Audit Action Button -->
    <button
      type="submit"
      disabled={!targetUrl.trim() || isScanning}
      class="px-3.5 py-1.5 bg-[var(--color-signal-red)] hover:opacity-90 disabled:opacity-40 text-white font-bold font-mono text-xs rounded-none flex items-center gap-1.5 transition-all cursor-pointer disabled:cursor-not-allowed flex-shrink-0 uppercase tracking-wider"
    >
      {#if isScanning}
        <Loader2 class="w-3.5 h-3.5 animate-spin" />
        <span>SCANNING...</span>
      {:else}
        <span>AUDIT</span>
        <span class="text-[10px] font-mono opacity-80">⏎</span>
      {/if}
    </button>
  </form>

  <!-- Desktop Quick Utility Tools -->
  <div class="flex items-center gap-1.5 no-drag flex-shrink-0">
    <!-- Re-scan current target -->
    {#if hasReport && !isScanning}
      <button
        type="button"
        onclick={onScan}
        class="p-1.5 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs font-mono transition-colors cursor-pointer"
        title="Re-run Security Audit (Ctrl+R)"
      >
        <RotateCw class="w-3.5 h-3.5" />
      </button>
    {/if}

    <!-- Scan History Drawer -->
    <button
      type="button"
      onclick={onOpenHistory}
      class="px-2.5 py-1.5 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs font-mono font-bold uppercase tracking-wider flex items-center gap-1.5 transition-colors cursor-pointer"
      title="Open Local SQLite History (Ctrl+H)"
    >
      <History class="w-3.5 h-3.5" />
      <span class="hidden sm:inline">HISTORY</span>
    </button>

    <!-- Export & Share -->
    {#if hasReport}
      <button
        type="button"
        onclick={onOpenExport}
        class="px-2.5 py-1.5 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs font-mono font-bold uppercase tracking-wider flex items-center gap-1.5 transition-colors cursor-pointer"
        title="Export Report & Print PDF (Ctrl+E)"
      >
        <FileDown class="w-3.5 h-3.5" />
        <span class="hidden sm:inline">EXPORT</span>
      </button>
    {/if}

    <!-- Dual-Theme Toggle (Swiss Dark ↔ Swiss Light) -->
    <button
      type="button"
      onclick={toggleTheme}
      class="px-2.5 py-1.5 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs font-mono font-bold uppercase tracking-wider flex items-center gap-1.5 transition-colors cursor-pointer"
      title="Toggle Swiss Theme Mode (Dark / Light)"
    >
      {#if currentTheme === "swiss-dark"}
        <Sun class="w-3.5 h-3.5 text-amber-400" />
        <span class="hidden sm:inline">LIGHT</span>
      {:else}
        <Moon class="w-3.5 h-3.5 text-zinc-600" />
        <span class="hidden sm:inline">DARK</span>
      {/if}
    </button>

    <!-- Unified Settings Hub -->
    <button
      type="button"
      onclick={() => onOpenSettings()}
      class="p-1.5 sm:px-2.5 sm:py-1.5 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs font-mono font-bold uppercase tracking-wider flex items-center gap-1.5 transition-colors cursor-pointer relative"
      title="Settings & Preferences (⌘,)"
    >
      <Settings class="w-3.5 h-3.5" />
      <span class="hidden sm:inline">SETTINGS</span>
      {#if hasCustomOptions || activeMonitorsCount > 0}
        <span
          class="absolute top-1 right-1 w-1.5 h-1.5 bg-[var(--color-signal-red)] rounded-none"
          title="Active customizations or scheduled monitors"
        ></span>
      {/if}
    </button>
  </div>
</header>
