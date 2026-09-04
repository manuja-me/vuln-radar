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
    if (score === undefined || score === null) return "bg-[var(--color-surface)] text-[var(--color-text-muted)] border-[var(--color-hairline)]";
    if (score >= 85) return "bg-[var(--color-signal-emerald)]/10 text-[var(--color-signal-emerald)] border-[var(--color-signal-emerald)]/30";
    if (score >= 70) return "bg-[var(--color-signal-blue)]/10 text-[var(--color-signal-blue)] border-[var(--color-signal-blue)]/30";
    if (score >= 50) return "bg-[var(--color-signal-amber)]/10 text-[var(--color-signal-amber)] border-[var(--color-signal-amber)]/30";
    return "bg-[var(--color-signal-red)]/10 text-[var(--color-signal-red)] border-[var(--color-signal-red)]/30";
  }
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-xs p-4 animate-fade-in"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
    onkeydown={(e) => {
      if (e.key === "Escape") onClose();
    }}
    tabindex="-1"
    role="dialog"
    aria-modal="true"
  >
    <div
      class="bg-[var(--color-surface)] border border-[var(--color-hairline-strong)] rounded-none w-full max-w-3xl max-h-[85vh] flex flex-col shadow-2xl overflow-hidden text-[var(--color-text-body)]"
    >
      <!-- Header -->
      <div class="p-4 border-b border-[var(--color-hairline)] flex items-center justify-between bg-[var(--color-canvas)]">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-none bg-[var(--color-surface)] border border-[var(--color-hairline)] flex items-center justify-center text-[var(--color-text-headline)]">
            <Layers class="w-4 h-4" />
          </div>
          <div>
            <div class="text-[10px] font-mono uppercase tracking-widest text-[var(--color-signal-red)]">05 / FLEET AUDIT</div>
            <h2 class="text-sm font-mono font-bold text-[var(--color-text-headline)] tracking-tight uppercase">Batch Security Scanner</h2>
          </div>
        </div>
        <button
          type="button"
          onclick={onClose}
          class="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface)] border border-transparent hover:border-[var(--color-hairline)] rounded-none transition-colors cursor-pointer"
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
                  class="block text-[10px] font-mono font-semibold text-[var(--color-text-muted)] uppercase tracking-widest"
                >
                  Target Inventory (One URL Per Line)
                </label>
                <span class="text-[10px] font-mono text-[var(--color-text-muted)]">HTTP & HTTPS SUPPORTED</span>
              </div>
              <textarea
                id="batch-urls"
                bind:value={rawUrls}
                rows="6"
                placeholder="https://example.com&#10;https://api.example.com&#10;https://staging.example.com"
                class="w-full p-3 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-text-headline)] rounded-none text-xs font-mono text-[var(--color-text-body)] placeholder-[var(--color-text-muted)] focus:outline-none"
              ></textarea>
            </div>

            <div class="p-3.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] rounded-none flex flex-col sm:flex-row sm:items-center justify-between gap-3">
              <div class="text-xs font-mono text-[var(--color-text-muted)]">
                <span class="font-bold text-[var(--color-text-headline)] uppercase">Sequential Audit Queue:</span>
                Targets are audited sequentially to prevent socket saturation.
              </div>
              <button
                type="button"
                onclick={startBatchScan}
                class="px-4 py-2 bg-[var(--color-signal-red)] hover:bg-[var(--color-signal-red-hover)] text-white font-mono uppercase tracking-widest text-xs font-bold rounded-none flex items-center gap-2 transition-colors cursor-pointer shadow-sm flex-shrink-0"
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
                <h3 class="text-xs font-mono font-bold text-[var(--color-text-headline)] uppercase tracking-wider">
                  {isRunning ? "Auditing Fleet Surface..." : "Fleet Assessment Completed"}
                </h3>
                <p class="text-xs text-[var(--color-text-muted)] font-mono mt-0.5">
                  Progress: <strong class="text-[var(--color-text-headline)]">{completedCount}</strong> of {batchItems.length} audited
                </p>
              </div>

              {#if !isRunning}
                <button
                  type="button"
                  onclick={() => (batchItems = [])}
                  class="px-3 py-1.5 bg-[var(--color-canvas)] hover:bg-[var(--color-surface)] text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs font-mono uppercase tracking-wider cursor-pointer transition-colors"
                >
                  New Target Batch
                </button>
              {/if}
            </div>

            <!-- Progress Bar -->
            <div class="w-full bg-[var(--color-canvas)] border border-[var(--color-hairline)] rounded-none h-2 overflow-hidden">
              <div
                class="bg-[var(--color-signal-red)] h-full rounded-none transition-all duration-300"
                style="width: {batchItems.length > 0 ? (completedCount / batchItems.length) * 100 : 0}%"
              ></div>
            </div>

            <!-- Results List -->
            <div class="space-y-2 max-h-80 overflow-y-auto">
              {#each batchItems as item}
                <div
                  class="p-3 bg-[var(--color-canvas)] border border-[var(--color-hairline)] rounded-none flex items-center justify-between gap-4"
                >
                  <div class="flex items-center gap-3 min-w-0 flex-1">
                    {#if item.status === "scanning"}
                      <Loader2 class="w-4 h-4 text-[var(--color-signal-blue)] animate-spin flex-shrink-0" />
                    {:else if item.status === "completed"}
                      <CheckCircle2 class="w-4 h-4 text-[var(--color-signal-emerald)] flex-shrink-0" />
                    {:else}
                      <AlertOctagon class="w-4 h-4 text-[var(--color-signal-red)] flex-shrink-0" />
                    {/if}

                    <div class="min-w-0 flex-1">
                      <div class="text-xs font-mono font-medium text-[var(--color-text-headline)] truncate">
                        {item.url}
                      </div>
                      {#if item.error}
                        <div class="text-[11px] text-[var(--color-signal-red)] font-mono mt-0.5 truncate">
                          {item.error}
                        </div>
                      {:else if item.report}
                        <div class="text-[11px] text-[var(--color-text-muted)] font-mono mt-0.5 flex items-center gap-2">
                          <span>{item.report.total_findings} findings</span>
                          {#if item.report.critical_count > 0}
                            <span class="text-[var(--color-signal-red)] font-bold">({item.report.critical_count} critical)</span>
                          {/if}
                        </div>
                      {/if}
                    </div>
                  </div>

                  {#if item.report}
                    <div class="flex items-center gap-2 flex-shrink-0">
                      <span
                        class="px-2 py-0.5 text-xs font-mono rounded-none border {getScoreBadge(item.report.security_score)}"
                      >
                        {item.report.security_score} PTS
                      </span>
                      <button
                        type="button"
                        onclick={() => {
                          if (item.report) {
                            onSelectReport(item.report);
                            onClose();
                          }
                        }}
                        class="px-2.5 py-1 bg-[var(--color-surface)] hover:bg-[var(--color-canvas)] text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs font-mono uppercase tracking-wider flex items-center gap-1 cursor-pointer transition-colors"
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

