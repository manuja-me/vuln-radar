<script lang="ts">
  import {
    ShieldCheck,
    Search,
    Loader2,
    History,
    FileDown,
    Settings,
    RotateCw,
    Sliders,
    Zap,
  } from "lucide-svelte";

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

  function handleSubmit(e: Event) {
    e.preventDefault();
    if (!targetUrl.trim() || isScanning) return;
    onScan();
  }
</script>

<!-- Native Desktop App Window Titlebar & Toolbar -->
<header
  data-tauri-drag-region
  class="bg-[#121318]/95 backdrop-blur-md border-b border-white/[0.08] sticky top-0 z-40 px-4 py-2 flex items-center justify-between gap-4 titlebar-drag desktop-select-none print:hidden flex-shrink-0"
>
  <!-- Window Drag / App Title & Status -->
  <div class="flex items-center gap-3 no-drag flex-shrink-0">
    <div class="flex items-center gap-2">
      <div
        class="w-7 h-7 rounded-lg bg-gradient-to-b from-neutral-800 to-neutral-900 border border-white/[0.12] flex items-center justify-center text-cyan-400 shadow-sm"
      >
        <ShieldCheck class="w-4 h-4" />
      </div>
      <div class="flex flex-col">
        <div class="flex items-center gap-1.5 leading-none">
          <span class="text-xs font-bold tracking-tight text-white font-mono">
            VulnRadar
          </span>
          <span
            class="px-1 py-0.5 text-[9px] font-mono bg-white/[0.06] text-neutral-400 border border-white/[0.08] rounded"
          >
            v0.6.3
          </span>
        </div>
        <span class="text-[10px] text-neutral-400 font-mono tracking-tight mt-0.5">
          Native Security Workstation
        </span>
      </div>
    </div>

    <!-- Active Port Scan Default Badge -->
    <div class="hidden md:flex items-center gap-1.5 px-2 py-0.5 rounded bg-cyan-950/30 border border-cyan-800/40 text-cyan-300 text-[10px] font-mono">
      <Zap class="w-2.5 h-2.5 text-cyan-400" />
      <span>Port Engine: Auto (Top 20)</span>
    </div>
  </div>

  <!-- Central Desktop Command Palette / Target URL Bar -->
  <form
    onsubmit={handleSubmit}
    class="w-full max-w-xl flex items-center gap-1.5 no-drag"
  >
    <div class="relative flex-1 group">
      <div
        class="absolute inset-y-0 left-0 pl-2.5 flex items-center pointer-events-none text-neutral-500 group-focus-within:text-cyan-400 transition-colors"
      >
        <Search class="w-3.5 h-3.5" />
      </div>
      <input
        type="text"
        bind:value={targetUrl}
        placeholder="Enter domain or backend URL (e.g. example.com or http://localhost:8000)..."
        disabled={isScanning}
        class="w-full pl-8 pr-12 py-1.5 bg-[#1a1b22] hover:bg-[#20222a] border border-white/[0.1] focus:border-cyan-500/80 rounded-md text-xs text-neutral-100 placeholder-neutral-500 font-mono transition-all disabled:opacity-60 focus:outline-none shadow-inner"
      />
      <div
        class="absolute inset-y-0 right-0 pr-2 flex items-center pointer-events-none gap-1"
      >
        <kbd class="px-1.5 py-0.5 text-[9px] font-mono bg-[#14151b] border border-white/[0.1] rounded text-neutral-400">
          ⌘K
        </kbd>
      </div>
    </div>

    <!-- Audit Action Button -->
    <button
      type="submit"
      disabled={!targetUrl.trim() || isScanning}
      class="px-3 py-1.5 bg-cyan-500 hover:bg-cyan-400 disabled:opacity-40 text-slate-950 font-bold text-xs rounded-md flex items-center gap-1.5 transition-all cursor-pointer disabled:cursor-not-allowed flex-shrink-0 shadow-sm shadow-cyan-500/20 active:scale-95"
    >
      {#if isScanning}
        <Loader2 class="w-3.5 h-3.5 animate-spin" />
        <span>Auditing...</span>
      {:else}
        <span>Audit</span>
        <span class="text-[10px] font-mono opacity-70">⏎</span>
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
        class="p-1.5 bg-[#1a1b22] hover:bg-[#22242e] text-neutral-300 hover:text-white border border-white/[0.08] rounded-md text-xs font-medium transition-colors cursor-pointer"
        title="Re-run Security Audit (Ctrl+R)"
      >
        <RotateCw class="w-3.5 h-3.5 text-neutral-400" />
      </button>
    {/if}

    <!-- Scan History Drawer -->
    <button
      type="button"
      onclick={onOpenHistory}
      class="px-2.5 py-1.5 bg-[#1a1b22] hover:bg-[#22242e] text-neutral-300 hover:text-white border border-white/[0.08] rounded-md text-xs font-medium flex items-center gap-1.5 transition-colors cursor-pointer"
      title="Open Local SQLite History (Ctrl+H)"
    >
      <History class="w-3.5 h-3.5 text-neutral-400" />
      <span class="hidden sm:inline">History</span>
    </button>

    <!-- Export & Share -->
    {#if hasReport}
      <button
        type="button"
        onclick={onOpenExport}
        class="px-2.5 py-1.5 bg-[#1a1b22] hover:bg-[#22242e] text-neutral-300 hover:text-white border border-white/[0.08] rounded-md text-xs font-medium flex items-center gap-1.5 transition-colors cursor-pointer"
        title="Export Report & Print PDF (Ctrl+E)"
      >
        <FileDown class="w-3.5 h-3.5 text-neutral-400" />
        <span class="hidden sm:inline">Export</span>
      </button>
    {/if}

    <!-- Unified Settings Hub -->
    <button
      type="button"
      onclick={() => onOpenSettings()}
      class="p-1.5 sm:px-2.5 sm:py-1.5 bg-[#1a1b22] hover:bg-[#22242e] text-neutral-300 hover:text-white border border-white/[0.08] rounded-md text-xs font-medium flex items-center gap-1.5 transition-colors cursor-pointer relative"
      title="Settings & Preferences (⌘,)"
    >
      <Settings class="w-3.5 h-3.5 text-neutral-400" />
      <span class="hidden sm:inline">Settings</span>
      {#if hasCustomOptions || activeMonitorsCount > 0}
        <span
          class="absolute top-1 right-1 w-1.5 h-1.5 bg-cyan-400 rounded-full animate-pulse"
          title="Active customizations or scheduled monitors"
        ></span>
      {/if}
    </button>
  </div>
</header>
