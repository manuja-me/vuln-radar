<script lang="ts">
  import {
    ShieldCheck,
    Search,
    Loader2,
    History,
    FileDown,
    Layers,
    Activity,
    Sliders,
    Printer,
    Keyboard,
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
    onOpenShortcuts,
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
    onOpenShortcuts?: () => void;
  } = $props();

  function handleSubmit(e: Event) {
    e.preventDefault();
    if (!targetUrl.trim() || isScanning) return;
    onScan();
  }
</script>

<header
  class="bg-[#191919] border-b border-[#2e2e2e] sticky top-0 z-40 px-6 py-3 print:hidden"
>
  <div
    class="max-w-7xl mx-auto flex flex-col lg:flex-row items-center justify-between gap-3.5"
  >
    <!-- Brand -->
    <div class="flex items-center gap-2.5 self-start lg:self-auto">
      <div
        class="w-8 h-8 rounded-lg bg-[#262626] border border-[#333333] flex items-center justify-center text-neutral-200"
      >
        <ShieldCheck class="w-4 h-4 text-white" />
      </div>
      <div>
        <div class="flex items-center gap-2">
          <span class="text-sm font-semibold tracking-tight text-white">
            VulnRadar
          </span>
          <span
            class="px-1.5 py-0.2 text-[10px] font-mono bg-neutral-800 text-neutral-400 border border-neutral-700/60 rounded"
          >
            v0.3.0
          </span>
        </div>
        <p class="text-[11px] text-neutral-400">
          Web Security Posture & Vulnerability Scanner
        </p>
      </div>
    </div>

    <!-- URL Input & Scan trigger -->
    <form
      onsubmit={handleSubmit}
      class="w-full lg:max-w-xl flex items-center gap-2"
    >
      <div class="relative flex-1 group">
        <div
          class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-neutral-500 group-focus-within:text-neutral-300 transition-colors"
        >
          <Search class="w-3.5 h-3.5" />
        </div>
        <input
          type="text"
          bind:value={targetUrl}
          placeholder="Enter target domain or URL (e.g. example.com)..."
          disabled={isScanning}
          class="w-full pl-9 pr-10 py-1.5 bg-[#202020] hover:bg-[#252525] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs text-[#e3e2e0] placeholder-neutral-500 font-mono transition-all disabled:opacity-60 focus:outline-none"
        />
        <div
          class="absolute inset-y-0 right-0 pr-2.5 flex items-center pointer-events-none text-[10px] font-mono text-neutral-400"
        >
          ⏎
        </div>
      </div>

      <!-- Scan Configuration / Options Trigger -->
      <button
        type="button"
        onclick={onOpenOptions}
        class="p-2 bg-[#202020] hover:bg-[#2a2a2a] text-neutral-300 hover:text-white border border-[#2e2e2e] rounded-lg transition-colors cursor-pointer relative"
        title="Scan Options & Parameters"
      >
        <Sliders class="w-3.5 h-3.5 text-neutral-300" />
        {#if hasCustomOptions}
          <span
            class="absolute top-1.5 right-1.5 w-1.5 h-1.5 bg-blue-400 rounded-full"
          ></span>
        {/if}
      </button>

      <!-- Scan Button -->
      <button
        type="submit"
        disabled={!targetUrl.trim() || isScanning}
        class="px-3.5 py-1.5 bg-white hover:bg-neutral-200 disabled:opacity-50 text-neutral-950 font-semibold text-xs rounded-lg flex items-center gap-1.5 transition-colors cursor-pointer disabled:cursor-not-allowed flex-shrink-0 shadow-sm"
      >
        {#if isScanning}
          <Loader2 class="w-3.5 h-3.5 animate-spin" />
          <span>Auditing...</span>
        {:else}
          <span>Audit</span>
        {/if}
      </button>
    </form>

    <!-- Navigation & Feature Actions -->
    <div class="flex flex-wrap items-center gap-1.5 self-end lg:self-auto">
      <button
        type="button"
        onclick={onOpenBatch}
        class="px-2.5 py-1.5 bg-[#202020] hover:bg-[#2a2a2a] text-neutral-300 hover:text-white border border-[#2e2e2e] rounded-lg text-xs font-medium flex items-center gap-1.5 transition-colors cursor-pointer"
        title="Batch Fleet Scanner"
      >
        <Layers class="w-3.5 h-3.5 text-neutral-400" />
        <span>Batch</span>
      </button>

      <button
        type="button"
        onclick={onOpenMonitors}
        class="px-2.5 py-1.5 bg-[#202020] hover:bg-[#2a2a2a] text-neutral-300 hover:text-white border border-[#2e2e2e] rounded-lg text-xs font-medium flex items-center gap-1.5 transition-colors cursor-pointer"
        title="Continuous Monitoring Watchdog"
      >
        <Activity class="w-3.5 h-3.5 text-neutral-400" />
        <span>Watchdog</span>
      </button>

      <button
        type="button"
        onclick={onOpenHistory}
        class="px-2.5 py-1.5 bg-[#202020] hover:bg-[#2a2a2a] text-neutral-300 hover:text-white border border-[#2e2e2e] rounded-lg text-xs font-medium flex items-center gap-1.5 transition-colors cursor-pointer"
        title="View Scan History (Ctrl+H)"
      >
        <History class="w-3.5 h-3.5 text-neutral-400" />
        <span>History</span>
      </button>

      {#if onOpenShortcuts}
        <button
          type="button"
          onclick={onOpenShortcuts}
          class="p-1.5 bg-[#202020] hover:bg-[#2a2a2a] text-neutral-400 hover:text-white border border-[#2e2e2e] rounded-lg transition-colors cursor-pointer"
          title="Keyboard Shortcuts (⌘K / ?)"
        >
          <Keyboard class="w-3.5 h-3.5" />
        </button>
      {/if}

      {#if hasReport}
        <button
          type="button"
          onclick={onOpenExecutiveReport}
          class="px-2.5 py-1.5 bg-[#262626] hover:bg-[#303030] text-neutral-200 border border-[#383838] rounded-lg text-xs font-medium flex items-center gap-1.5 transition-colors cursor-pointer"
          title="Executive PDF Report"
        >
          <Printer class="w-3.5 h-3.5 text-neutral-300" />
          <span>Report</span>
        </button>

        <button
          type="button"
          onclick={onOpenExport}
          class="p-1.5 bg-[#202020] hover:bg-[#2a2a2a] text-neutral-300 hover:text-white border border-[#2e2e2e] rounded-lg transition-colors cursor-pointer"
          title="Export Markdown/JSON"
        >
          <FileDown class="w-3.5 h-3.5 text-neutral-400" />
        </button>
      {/if}
    </div>
  </div>
</header>


