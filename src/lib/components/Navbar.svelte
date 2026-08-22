<script lang="ts">
  import { ShieldAlert, Search, Loader2, History, FileDown, RefreshCw } from "lucide-svelte";

  let {
    targetUrl = $bindable(""),
    isScanning = false,
    hasReport = false,
    onScan,
    onOpenHistory,
    onOpenExport,
  }: {
    targetUrl: string;
    isScanning: boolean;
    hasReport: boolean;
    onScan: () => void;
    onOpenHistory: () => void;
    onOpenExport: () => void;
  } = $props();

  function handleSubmit(e: Event) {
    e.preventDefault();
    if (!targetUrl.trim() || isScanning) return;
    onScan();
  }

  function setPreset(url: string) {
    targetUrl = url;
    onScan();
  }
</script>

<header class="bg-slate-900/80 backdrop-blur-md border-b border-slate-800/80 sticky top-0 z-40 px-6 py-4">
  <div class="max-w-7xl mx-auto flex flex-col md:flex-row items-center justify-between gap-4">
    <!-- Brand -->
    <div class="flex items-center gap-3 self-start md:self-auto">
      <div class="w-10 h-10 rounded-xl bg-gradient-to-tr from-cyan-500 to-blue-600 flex items-center justify-center shadow-lg shadow-cyan-500/20 text-slate-950 font-black">
        <ShieldAlert class="w-6 h-6 text-slate-950" />
      </div>
      <div>
        <div class="flex items-center gap-2">
          <span class="text-base font-extrabold tracking-tight text-white">VulnRadar</span>
          <span class="px-1.5 py-0.2 text-[10px] uppercase font-bold tracking-wider bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 rounded">v1.0</span>
        </div>
        <p class="text-[11px] text-slate-400">Desktop Web Vulnerability & Security Scanner</p>
      </div>
    </div>

    <!-- URL Input & Scan trigger -->
    <form onsubmit={handleSubmit} class="w-full md:max-w-xl flex items-center gap-2">
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

    <!-- History & Export Actions -->
    <div class="flex items-center gap-2 self-end md:self-auto">
      <button
        type="button"
        onclick={onOpenHistory}
        class="px-3.5 py-2 bg-slate-800/80 hover:bg-slate-800 text-slate-300 hover:text-slate-100 border border-slate-700/60 rounded-xl text-xs font-semibold flex items-center gap-2 transition-colors cursor-pointer"
        title="View Scan History"
      >
        <History class="w-4 h-4 text-cyan-400" />
        <span>History</span>
      </button>

      {#if hasReport}
        <button
          type="button"
          onclick={onOpenExport}
          class="px-3.5 py-2 bg-slate-800/80 hover:bg-slate-800 text-slate-300 hover:text-slate-100 border border-slate-700/60 rounded-xl text-xs font-semibold flex items-center gap-2 transition-colors cursor-pointer"
          title="Export Report"
        >
          <FileDown class="w-4 h-4 text-cyan-400" />
          <span>Export</span>
        </button>
      {/if}
    </div>
  </div>
</header>
