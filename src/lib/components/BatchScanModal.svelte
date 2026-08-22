<script lang="ts">
  import type { BatchScanItem, ScanOptions, ScanReport } from "$lib/types";
  import {
    X,
    Layers,
    Play,
    Loader2,
    CheckCircle2,
    AlertOctagon,
    ArrowRight,
    Globe,
    Shield,
    Terminal,
  } from "lucide-svelte";

  let {
    isOpen = false,
    options,
    onSelectReport,
    onClose,
  }: {
    isOpen: boolean;
    options: ScanOptions;
    onSelectReport: (report: ScanReport) => void;
    onClose: () => void;
  } = $props();

  let rawUrls = $state(
    "https://example.com\nhttps://httpbin.org\nhttp://testphp.vulnweb.com"
  );
  let isRunning = $state(false);
  let batchItems = $state<BatchScanItem[]>([]);
  let completedCount = $derived(
    batchItems.filter((i) => i.status === "completed" || i.status === "failed")
      .length
  );

  async function invokeTauri<T>(
    cmd: string,
    args: Record<string, unknown> = {}
  ): Promise<T> {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(cmd, args);
  }

  async function startBatchScan() {
    const lines = rawUrls
      .split("\n")
      .map((l) => l.trim())
      .filter((l) => l.length > 0);

    if (lines.length === 0) return;

    isRunning = true;
    batchItems = lines.map((url) => ({
      url,
      status: "scanning",
      report: null,
      error: null,
    }));

    try {
      const results = await invokeTauri<BatchScanItem[]>("scan_batch", {
        urls: lines,
        options,
      });
      batchItems = results;
    } catch (e: any) {
      console.error("Batch scan error:", e);
    } finally {
      isRunning = false;
    }
  }

  function getScoreBadge(score?: number) {
    if (score === undefined || score === null) return "bg-slate-800 text-slate-400";
    if (score >= 85) return "bg-emerald-500/10 text-emerald-400 border-emerald-500/30";
    if (score >= 70) return "bg-cyan-500/10 text-cyan-400 border-cyan-500/30";
    if (score >= 50) return "bg-amber-500/10 text-amber-400 border-amber-500/30";
    return "bg-rose-500/10 text-rose-400 border-rose-500/30";
  }
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-4 animate-fade-in"
  >
    <div
      class="bg-slate-900/95 border border-slate-800 rounded-2xl w-full max-w-3xl max-h-[85vh] flex flex-col shadow-2xl overflow-hidden backdrop-blur-2xl"
    >
      <!-- Header -->
      <div class="p-5 border-b border-slate-800/80 flex items-center justify-between bg-slate-950/40">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl bg-cyan-500/10 border border-cyan-500/30 flex items-center justify-center text-cyan-400 shadow-lg shadow-cyan-500/10">
            <Layers class="w-4 h-4" />
          </div>
          <div>
            <h2 class="text-base font-black text-white tracking-tight">Fleet & Batch Security Scanner</h2>
            <p class="text-[11px] text-slate-400">Sequential multi-target automated surface assessment</p>
          </div>
        </div>
        <button
          type="button"
          onclick={onClose}
          class="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-lg transition-colors cursor-pointer"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        {#if !isRunning && batchItems.length === 0}
          <!-- Input Form -->
          <div class="space-y-4">
            <div>
              <div class="flex items-center justify-between mb-2">
                <label
                  for="batch-urls"
                  class="block text-xs font-mono font-bold text-slate-300 uppercase tracking-wider"
                >
                  Target Inventory (One URL Per Line)
                </label>
                <span class="text-[11px] font-mono text-slate-500">Supports HTTP & HTTPS</span>
              </div>
              <textarea
                id="batch-urls"
                bind:value={rawUrls}
                rows="6"
                placeholder="https://example.com&#10;https://api.example.com&#10;https://staging.example.com"
                class="w-full p-3.5 bg-slate-950 border border-slate-800 focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500/40 rounded-xl text-xs font-mono text-slate-200 placeholder-slate-600 focus:outline-none shadow-inner"
              ></textarea>
            </div>

            <div class="p-4 bg-slate-950/60 border border-slate-800 rounded-xl flex flex-col sm:flex-row sm:items-center justify-between gap-3">
              <div class="text-xs text-slate-400">
                <span class="font-bold text-slate-200 font-mono">Sequential Audit Queue:</span>
                Targets are scanned one after another to prevent network congestion.
              </div>
              <button
                type="button"
                onclick={startBatchScan}
                class="px-5 py-2.5 bg-gradient-to-r from-cyan-500 to-cyan-400 hover:from-cyan-400 hover:to-cyan-300 text-slate-950 font-bold rounded-xl text-xs flex items-center gap-2 transition-all shadow-lg shadow-cyan-500/20 cursor-pointer flex-shrink-0"
              >
                <Play class="w-3.5 h-3.5 fill-current" />
                <span>Launch Fleet Audit</span>
              </button>
            </div>
          </div>
        {:else}
          <!-- Execution & Results Table -->
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <h3 class="text-sm font-black text-white tracking-tight">
                  {isRunning ? "Auditing Fleet Surface..." : "Fleet Assessment Completed"}
                </h3>
                <p class="text-xs text-slate-400 font-mono mt-0.5">
                  Progress: <strong class="text-cyan-400">{completedCount}</strong> of {batchItems.length} audited
                </p>
              </div>

              {#if !isRunning}
                <button
                  type="button"
                  onclick={() => (batchItems = [])}
                  class="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg text-xs font-semibold cursor-pointer transition-colors"
                >
                  New Target Batch
                </button>
              {/if}
            </div>

            <!-- Progress Bar -->
            <div class="w-full bg-slate-950 border border-slate-800 rounded-full h-2.5 overflow-hidden p-0.5">
              <div
                class="bg-gradient-to-r from-cyan-500 via-blue-500 to-emerald-400 h-full rounded-full transition-all duration-300"
                style="width: {batchItems.length > 0 ? (completedCount / batchItems.length) * 100 : 0}%"
              ></div>
            </div>

            <!-- Results List -->
            <div class="space-y-2 max-h-80 overflow-y-auto">
              {#each batchItems as item}
                <div
                  class="p-3.5 bg-slate-950/80 border border-slate-800/90 rounded-xl flex items-center justify-between gap-4 shadow-sm"
                >
                  <div class="flex items-center gap-3 min-w-0 flex-1">
                    {#if item.status === "scanning"}
                      <Loader2 class="w-4 h-4 text-cyan-400 animate-spin flex-shrink-0" />
                    {:else if item.status === "completed"}
                      <CheckCircle2 class="w-4 h-4 text-emerald-400 flex-shrink-0" />
                    {:else}
                      <AlertOctagon class="w-4 h-4 text-rose-400 flex-shrink-0" />
                    {/if}

                    <div class="min-w-0 flex-1">
                      <div class="text-xs font-bold text-slate-200 truncate font-mono">
                        {item.url}
                      </div>
                      {#if item.error}
                        <div class="text-[11px] text-rose-400 font-mono mt-0.5 truncate">
                          {item.error}
                        </div>
                      {:else if item.report}
                        <div class="text-[11px] text-slate-400 font-mono mt-0.5 flex items-center gap-2">
                          <span>{item.report.total_findings} findings</span>
                          {#if item.report.critical_count > 0}
                            <span class="text-rose-400 font-bold">({item.report.critical_count} critical)</span>
                          {/if}
                        </div>
                      {/if}
                    </div>
                  </div>

                  {#if item.report}
                    <div class="flex items-center gap-2 flex-shrink-0">
                      <span
                        class="px-2.5 py-0.5 text-xs font-mono font-bold rounded-md border {getScoreBadge(item.report.security_score)}"
                      >
                        {item.report.security_score} pts
                      </span>
                      <button
                        type="button"
                        onclick={() => {
                          if (item.report) {
                            onSelectReport(item.report);
                            onClose();
                          }
                        }}
                        class="px-2.5 py-1 bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-400 border border-cyan-500/30 rounded-lg text-xs font-semibold flex items-center gap-1 cursor-pointer transition-colors"
                      >
                        <span>Inspect</span>
                        <ArrowRight class="w-3 h-3" />
                      </button>
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

