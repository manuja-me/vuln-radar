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
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-fade-in"
  >
    <div
      class="bg-[#202020] border border-[#333333] rounded-xl w-full max-w-3xl max-h-[85vh] flex flex-col shadow-xl overflow-hidden text-[#e3e2e0]"
    >
      <!-- Header -->
      <div class="p-4 border-b border-[#2e2e2e] flex items-center justify-between bg-[#191919]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-[#282828] border border-[#383838] flex items-center justify-center text-neutral-300">
            <Layers class="w-4 h-4" />
          </div>
          <div>
            <h2 class="text-sm font-semibold text-white tracking-tight">Fleet & Batch Security Scanner</h2>
            <p class="text-[11px] text-neutral-400">Sequential multi-target automated surface assessment</p>
          </div>
        </div>
        <button
          type="button"
          onclick={onClose}
          class="p-1.5 text-neutral-400 hover:text-white hover:bg-[#282828] rounded-lg transition-colors cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-5 space-y-5">
        {#if !isRunning && batchItems.length === 0}
          <!-- Input Form -->
          <div class="space-y-4">
            <div>
              <div class="flex items-center justify-between mb-2">
                <label
                  for="batch-urls"
                  class="block text-xs font-medium text-neutral-400 uppercase tracking-wider"
                >
                  Target Inventory (One URL Per Line)
                </label>
                <span class="text-[11px] text-neutral-500">Supports HTTP & HTTPS</span>
              </div>
              <textarea
                id="batch-urls"
                bind:value={rawUrls}
                rows="6"
                placeholder="https://example.com&#10;https://api.example.com&#10;https://staging.example.com"
                class="w-full p-3 bg-[#191919] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs font-mono text-neutral-200 placeholder-neutral-500 focus:outline-none"
              ></textarea>
            </div>

            <div class="p-3.5 bg-[#191919] border border-[#2e2e2e] rounded-lg flex flex-col sm:flex-row sm:items-center justify-between gap-3">
              <div class="text-xs text-neutral-400">
                <span class="font-medium text-neutral-200">Sequential Audit Queue:</span>
                Targets are audited one by one to prevent socket saturation.
              </div>
              <button
                type="button"
                onclick={startBatchScan}
                class="px-4 py-1.5 bg-white hover:bg-neutral-200 text-neutral-950 font-semibold rounded-lg text-xs flex items-center gap-2 transition-colors cursor-pointer shadow-sm flex-shrink-0"
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
                <h3 class="text-sm font-semibold text-white tracking-tight">
                  {isRunning ? "Auditing Fleet Surface..." : "Fleet Assessment Completed"}
                </h3>
                <p class="text-xs text-neutral-400 font-mono mt-0.5">
                  Progress: <strong class="text-white">{completedCount}</strong> of {batchItems.length} audited
                </p>
              </div>

              {#if !isRunning}
                <button
                  type="button"
                  onclick={() => (batchItems = [])}
                  class="px-3 py-1.5 bg-[#262626] hover:bg-[#303030] text-neutral-300 rounded-lg text-xs font-medium cursor-pointer transition-colors"
                >
                  New Target Batch
                </button>
              {/if}
            </div>

            <!-- Progress Bar -->
            <div class="w-full bg-[#161616] border border-[#2e2e2e] rounded-full h-2 overflow-hidden p-0.5">
              <div
                class="bg-blue-500 h-full rounded-full transition-all duration-300"
                style="width: {batchItems.length > 0 ? (completedCount / batchItems.length) * 100 : 0}%"
              ></div>
            </div>

            <!-- Results List -->
            <div class="space-y-2 max-h-80 overflow-y-auto">
              {#each batchItems as item}
                <div
                  class="p-3 bg-[#191919] border border-[#2e2e2e] rounded-lg flex items-center justify-between gap-4"
                >
                  <div class="flex items-center gap-3 min-w-0 flex-1">
                    {#if item.status === "scanning"}
                      <Loader2 class="w-4 h-4 text-blue-400 animate-spin flex-shrink-0" />
                    {:else if item.status === "completed"}
                      <CheckCircle2 class="w-4 h-4 text-emerald-400 flex-shrink-0" />
                    {:else}
                      <AlertOctagon class="w-4 h-4 text-red-400 flex-shrink-0" />
                    {/if}

                    <div class="min-w-0 flex-1">
                      <div class="text-xs font-medium text-neutral-200 truncate font-mono">
                        {item.url}
                      </div>
                      {#if item.error}
                        <div class="text-[11px] text-red-400 font-mono mt-0.5 truncate">
                          {item.error}
                        </div>
                      {:else if item.report}
                        <div class="text-[11px] text-neutral-400 font-mono mt-0.5 flex items-center gap-2">
                          <span>{item.report.total_findings} findings</span>
                          {#if item.report.critical_count > 0}
                            <span class="text-red-400 font-medium">({item.report.critical_count} critical)</span>
                          {/if}
                        </div>
                      {/if}
                    </div>
                  </div>

                  {#if item.report}
                    <div class="flex items-center gap-2 flex-shrink-0">
                      <span
                        class="px-2 py-0.5 text-xs font-mono rounded border {getScoreBadge(item.report.security_score)}"
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
                        class="px-2.5 py-1 bg-[#262626] hover:bg-[#303030] text-neutral-200 border border-[#383838] rounded-md text-xs font-medium flex items-center gap-1 cursor-pointer transition-colors"
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

