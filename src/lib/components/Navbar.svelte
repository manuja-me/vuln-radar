<script lang="ts">
  import {
    ShieldAlert,
    Search,
    Loader2,
    History,
    FileDown,
    RefreshCw,
    Layers,
    Activity,
    Settings2,
    Printer,
  } from "lucide-svelte";

  let {
    targetUrl = $bindable(""),
    isScanning = false,
    hasReport = false,
    hasCustomOptions = false,
    onScan,
    onOpenOptions,
    onOpenBatch,
    onOpenMonitors,
    onOpenHistory,
    onOpenExport,
    onOpenExecutiveReport,
  }: {
    targetUrl: string;
    isScanning: boolean;
    hasReport: boolean;
    hasCustomOptions?: boolean;
    onScan: () => void;
    onOpenOptions: () => void;
    onOpenBatch: () => void;
    onOpenMonitors: () => void;
    onOpenHistory: () => void;
    onOpenExport: () => void;
    onOpenExecutiveReport: () => void;
  } = $props();

  function handleSubmit(e: Event) {
    e.preventDefault();
    if (!targetUrl.trim() || isScanning) return;
    onScan();
  }
</script>

<header class="bg-slate-900/80 backdrop-blur-md border-b border-slate-800/80 sticky top-0 z-40 px-6 py-4 print:hidden">
  <div class="max-w-7xl mx-auto flex flex-col lg:flex-row items-center justify-between gap-4">
    <!-- Brand -->
    <div class="flex items-center gap-3 self-start lg:self-auto">
      <div class="w-10 h-10 rounded-xl bg-gradient-to-tr from-cyan-500 to-blue-600 flex items-center justify-center shadow-lg shadow-cyan-500/20 text-slate-950 font-black">
        <ShieldAlert class="w-6 h-6 text-slate-950" />
      </div>
      <div>
        <div class="flex items-center gap-2">
          <span class="text-base font-extrabold tracking-tight text-white">VulnRadar</span>
          <span class="px-1.5 py-0.2 text-[10px] uppercase font-bold tracking-wider bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 rounded">v0.1.0</span>
        </div>
        <p class="text-[11px] text-slate-400">Desktop Web Vulnerability & Security Scanner</p>
      </div>
    </div>

    <!-- URL Input & Scan trigger -->
    <form onsubmit={handleSubmit} class="w-full lg:max-w-xl flex items-center gap-2">
      <div class="relative flex-1">
        <div class="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none text-slate-400">
          <Search class="w-4 h-4" />
        </div>
        <input
          type="text"
          bind:value={targetUrl}
          placeholder="Enter website URL (e.g. example.com, testphp.vulnweb.com)..."
          disabled={isScanning}
          class="w-full pl-10 pr-4 py-2.5 bg-slate-950/80 border border-slate-800 focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500 rounded-xl text-sm text-slate-100 placeholder-slate-500 font-mono transition-all disabled:opacity-60"
        />
      </div>

      <button
        type="button"
        onclick={onOpenOptions}
        class="p-2.5 bg-slate-800/80 hover:bg-slate-800 text-slate-300 hover:text-slate-100 border border-slate-700/60 rounded-xl transition-colors cursor-pointer relative"
        title="Scan Options & Custom Headers"
      >
        <Settings2 class="w-4 h-4 text-cyan-400" />
        {#if hasCustomOptions}
          <span class="absolute top-1.5 right-1.5 w-2 h-2 bg-cyan-400 rounded-full"></span>
        {/if}
      </button>

      <button
        type="submit"
        disabled={!targetUrl.trim() || isScanning}
        class="px-5 py-2.5 bg-cyan-500 hover:bg-cyan-400 disabled:opacity-50 text-slate-950 font-bold text-sm rounded-xl flex items-center gap-2 transition-all shadow-lg shadow-cyan-500/20 cursor-pointer disabled:cursor-not-allowed flex-shrink-0"
      >
        {#if isScanning}
          <Loader2 class="w-4 h-4 animate-spin" />
          <span>Scanning...</span>
        {:else}
          <RefreshCw class="w-4 h-4" />
          <span>Scan</span>
        {/if}
      </button>
    </form>

    <!-- Navigation & Feature Action Badges -->
    <div class="flex flex-wrap items-center gap-2 self-end lg:self-auto">
      <button
        type="button"
        onclick={onOpenBatch}
        class="px-3 py-2 bg-slate-800/80 hover:bg-slate-800 text-slate-300 hover:text-slate-100 border border-slate-700/60 rounded-xl text-xs font-semibold flex items-center gap-1.5 transition-colors cursor-pointer"
        title="Batch Fleet Scanner"
      >
        <Layers class="w-3.5 h-3.5 text-cyan-400" />
        <span>Batch</span>
      </button>

      <button
        type="button"
        onclick={onOpenMonitors}
        class="px-3 py-2 bg-slate-800/80 hover:bg-slate-800 text-slate-300 hover:text-slate-100 border border-slate-700/60 rounded-xl text-xs font-semibold flex items-center gap-1.5 transition-colors cursor-pointer"
        title="Continuous Monitoring Watchdogs"
      >
        <Activity class="w-3.5 h-3.5 text-cyan-400" />
        <span>Monitors</span>
      </button>

      <button
        type="button"
        onclick={onOpenHistory}
        class="px-3 py-2 bg-slate-800/80 hover:bg-slate-800 text-slate-300 hover:text-slate-100 border border-slate-700/60 rounded-xl text-xs font-semibold flex items-center gap-1.5 transition-colors cursor-pointer"
        title="View Scan History"
      >
        <History class="w-3.5 h-3.5 text-cyan-400" />
        <span>History</span>
      </button>

      {#if hasReport}
        <button
          type="button"
          onclick={onOpenExecutiveReport}
          class="px-3 py-2 bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-400 border border-cyan-500/30 rounded-xl text-xs font-bold flex items-center gap-1.5 transition-colors cursor-pointer"
          title="Executive PDF Report"
        >
          <Printer class="w-3.5 h-3.5" />
          <span>PDF Report</span>
        </button>

        <button
          type="button"
          onclick={onOpenExport}
          class="p-2 bg-slate-800/80 hover:bg-slate-800 text-slate-300 hover:text-slate-100 border border-slate-700/60 rounded-xl transition-colors cursor-pointer"
          title="Export Markdown"
        >
          <FileDown class="w-4 h-4 text-cyan-400" />
        </button>
      {/if}
    </div>
  </div>
</header>

