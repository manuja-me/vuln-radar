<script lang="ts">
  import type { BatchScanItem, ScanOptions, ScanReport } from "$lib/types";
  import {
    Layers,
    Play,
    Loader2,
    CheckCircle2,
    AlertOctagon,
    AlertTriangle,
    ArrowRight,
    ArrowLeft,
    ExternalLink,
    Copy,
    Check,
    RotateCw,
    Globe,
    Shield,
    Terminal,
    Sliders,
    Sparkles,
    Trash2,
  } from "lucide-svelte";

  let {
    options,
    onSelectReport,
    onClose,
  }: {
    options: ScanOptions;
    onSelectReport?: (report: ScanReport) => void;
    onClose?: () => void;
  } = $props();

  let rawUrls = $state(
    "https://example.com\nhttps://httpbin.org\nhttp://testphp.vulnweb.com"
  );
  let isRunning = $state(false);
  let batchError = $state<string | null>(null);
  let batchItems = $state<BatchScanItem[]>([]);
  let copiedUrl = $state<string | null>(null);
  let activeFilter = $state<"all" | "completed" | "critical" | "failed">("all");

  const parsedUrls = $derived.by(() => {
    return rawUrls
      .split("\n")
      .map((l) => l.trim())
      .filter((l) => l.length > 0);
  });

  const completedCount = $derived(
    batchItems.filter((i) => i.status === "completed" || i.status === "failed")
      .length
  );

  const criticalTargetsCount = $derived(
    batchItems.filter((i) => (i.report?.critical_count ?? 0) > 0).length
  );

  const averageScore = $derived.by(() => {
    const scores = batchItems
      .filter((i) => i.report && typeof i.report.security_score === "number")
      .map((i) => i.report!.security_score);
    if (scores.length === 0) return null;
    const sum = scores.reduce((acc, s) => acc + s, 0);
    return Math.round(sum / scores.length);
  });

  const filteredItems = $derived.by(() => {
    if (activeFilter === "completed") {
      return batchItems.filter((i) => i.status === "completed");
    }
    if (activeFilter === "critical") {
      return batchItems.filter((i) => (i.report?.critical_count ?? 0) > 0);
    }
    if (activeFilter === "failed") {
      return batchItems.filter((i) => i.status === "failed");
    }
    return batchItems;
  });

  async function invokeTauri<T>(
    cmd: string,
    args: Record<string, unknown> = {}
  ): Promise<T> {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(cmd, args);
  }

  async function startBatchScan() {
    const lines = parsedUrls;
    if (lines.length === 0 || isRunning) return;

    isRunning = true;
    batchError = null;
    activeFilter = "all";
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
      batchError = e?.message || String(e);
      batchItems = batchItems.map((item) => ({
        ...item,
        status: "failed",
        error: batchError,
      }));
    } finally {
      isRunning = false;
    }
  }

  function handleLoadSamples() {
    rawUrls = [
      "https://example.com",
      "https://httpbin.org",
      "http://testphp.vulnweb.com",
      "https://scanme.nmap.org",
    ].join("\n");
  }

  function handleClearTargets() {
    rawUrls = "";
    batchItems = [];
    batchError = null;
  }

  async function copyUrl(url: string) {
    try {
      await navigator.clipboard.writeText(url);
      copiedUrl = url;
      setTimeout(() => {
        if (copiedUrl === url) copiedUrl = null;
      }, 1800);
    } catch {}
  }

  function getScoreBadge(score?: number | null) {
    if (score === undefined || score === null) {
      return "bg-surface-container text-outline border border-surface-container-high";
    }
    if (score >= 85) {
      return "bg-emerald-500/10 text-emerald-400 border border-emerald-500/30";
    }
    if (score >= 70) {
      return "bg-blue-500/10 text-blue-400 border border-blue-500/30";
    }
    if (score >= 50) {
      return "bg-amber-500/10 text-amber-400 border border-amber-500/30";
    }
    return "bg-error/10 text-error border border-error/30";
  }
</script>

<div class="flex flex-col w-full gap-4 max-w-7xl mx-auto pb-10 flex-1 animate-fade-in">
  <!-- Top Workspace Header -->
  <section class="bg-surface-container rounded-t-lg p-4 sm:p-5 flex items-center justify-between border border-surface-container-high border-b-0 shadow-xs">
    <div class="flex items-center gap-3">
      <div class="w-9 h-9 rounded bg-surface-container-lowest border border-surface-container-high flex items-center justify-center text-primary shrink-0">
        <Layers class="w-4 h-4" />
      </div>
      <div>
        <div class="flex items-center gap-2">
          <h2 class="text-sm font-black text-on-surface uppercase tracking-tight font-mono">
            Fleet Security Audit & Batch Surface Scanner
          </h2>
          <span class="px-1.5 py-0.2 text-[10px] bg-surface-container-lowest text-outline rounded font-mono border border-surface-container-high uppercase font-bold">
            05 / FLEET
          </span>
        </div>
        <p class="text-xs text-outline font-mono mt-0.5">
          Orchestrate sequential vulnerability reconnaissance across multiple web domains and services
        </p>
      </div>
    </div>

    {#if onClose}
      <button
        type="button"
        onclick={onClose}
        class="h-8 px-3 rounded bg-surface-container-lowest hover:bg-surface-container-high border border-surface-container-high text-xs font-mono font-bold uppercase text-outline hover:text-on-surface transition-colors cursor-pointer flex items-center gap-1.5 shrink-0"
        title="Return to primary audit workspace"
      >
        <ArrowLeft class="w-3.5 h-3.5" />
        <span class="hidden sm:inline">Back to Audit</span>
      </button>
    {/if}
  </section>

  <!-- Main Container -->
  <div class="flex-1 flex flex-col gap-4 bg-surface-container-low border border-surface-container-high rounded-b-lg p-4 sm:p-6 shadow-sm">
    <!-- Top Configuration & Launch Card -->
    <div class="bg-surface-container rounded-lg border border-surface-container-high p-4 sm:p-5 space-y-4">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 pb-3 border-b border-surface-container-high">
        <div>
          <h3 class="text-xs font-bold text-on-surface uppercase tracking-wider font-mono flex items-center gap-2">
            <Globe class="w-3.5 h-3.5 text-primary" />
            <span>Target Inventory Queue</span>
            <span class="px-1.5 py-0.2 text-[10px] bg-surface-container-lowest text-outline rounded border border-surface-container-high">
              {parsedUrls.length} {parsedUrls.length === 1 ? "Target" : "Targets"}
            </span>
          </h3>
          <p class="text-[11px] text-outline font-mono mt-0.5">
            Enter one domain or URL per line. Audits are processed sequentially to avoid socket exhaustion.
          </p>
        </div>

        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={handleLoadSamples}
            disabled={isRunning}
            class="px-2.5 py-1 bg-surface-container-lowest hover:bg-surface-container-highest border border-surface-container-high rounded text-[11px] font-mono font-bold uppercase text-outline hover:text-on-surface transition-colors cursor-pointer disabled:opacity-50 flex items-center gap-1"
          >
            <Sparkles class="w-3 h-3 text-primary" />
            <span>Load Samples</span>
          </button>
          <button
            type="button"
            onclick={handleClearTargets}
            disabled={isRunning || parsedUrls.length === 0}
            class="px-2.5 py-1 bg-surface-container-lowest hover:bg-surface-container-highest border border-surface-container-high rounded text-[11px] font-mono font-bold uppercase text-outline hover:text-error transition-colors cursor-pointer disabled:opacity-50 flex items-center gap-1"
          >
            <Trash2 class="w-3 h-3" />
            <span>Clear</span>
          </button>
        </div>
      </div>

      <!-- Target Textarea -->
      <div class="space-y-1.5">
        <textarea
          id="fleet-target-urls"
          rows="5"
          bind:value={rawUrls}
          disabled={isRunning}
          placeholder="https://example.com&#10;https://api.example.com&#10;https://staging.internal.net"
          class="w-full p-3 text-xs bg-surface-container-lowest rounded border border-surface-container-high font-mono text-on-surface placeholder:text-outline focus:border-outline outline-none leading-relaxed resize-y min-h-[110px]"
        ></textarea>
      </div>

      <!-- Launch Bar & Options Info -->
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-2">
        <div class="flex flex-wrap items-center gap-2 text-[11px] font-mono text-outline">
          <span class="flex items-center gap-1 bg-surface-container-lowest px-2 py-1 rounded border border-surface-container-high">
            <Sliders class="w-3 h-3 text-primary" />
            <span>Timeout: {options?.timeout_seconds ?? 15}s</span>
          </span>
          <span class="flex items-center gap-1 bg-surface-container-lowest px-2 py-1 rounded border border-surface-container-high">
            <span>Ports: {options?.enable_port_scan ? (options?.port_scan_profile || "top20") : "Disabled"}</span>
          </span>
          <span class="flex items-center gap-1 bg-surface-container-lowest px-2 py-1 rounded border border-surface-container-high">
            <span>Subdomains: {options?.include_subdomains ? "Enabled" : "Off"}</span>
          </span>
        </div>

        <button
          type="button"
          disabled={isRunning || parsedUrls.length === 0}
          onclick={startBatchScan}
          class="px-5 py-2.5 bg-primary text-on-primary font-bold uppercase font-mono text-xs rounded hover:bg-primary/90 transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2 shadow-sm shrink-0"
        >
          {#if isRunning}
            <Loader2 class="w-3.5 h-3.5 animate-spin" />
            <span>Auditing Fleet ({completedCount}/{batchItems.length})...</span>
          {:else}
            <Play class="w-3.5 h-3.5 fill-current" />
            <span>Launch Fleet Audit ({parsedUrls.length})</span>
          {/if}
        </button>
      </div>
    </div>

    <!-- Error Banner -->
    {#if batchError}
      <div class="p-3.5 bg-error/10 border border-error/30 rounded-lg text-error text-xs font-mono flex items-center gap-2.5 animate-fade-in">
        <AlertOctagon class="w-4 h-4 shrink-0" />
        <span class="flex-1">{batchError}</span>
      </div>
    {/if}

    <!-- Telemetry Cards & Progress (When Executing or Completed) -->
    {#if batchItems.length > 0}
      <div class="space-y-4 animate-fade-in">
        <!-- Progress Bar -->
        <div class="space-y-1.5">
          <div class="flex items-center justify-between text-xs font-mono text-outline">
            <span class="font-bold text-on-surface uppercase">
              {isRunning ? "Fleet Audit in Progress..." : "Fleet Assessment Completed"}
            </span>
            <span>
              {completedCount} / {batchItems.length} Audited ({batchItems.length > 0 ? Math.round((completedCount / batchItems.length) * 100) : 0}%)
            </span>
          </div>
          <div class="w-full bg-surface-container rounded h-2 overflow-hidden border border-surface-container-high">
            <div
              class="bg-primary h-full transition-all duration-300"
              style="width: {batchItems.length > 0 ? (completedCount / batchItems.length) * 100 : 0}%"
            ></div>
          </div>
        </div>

        <!-- Telemetry Stats Cards -->
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 font-mono">
          <div class="p-3 bg-surface-container rounded border border-surface-container-high">
            <div class="text-[10px] text-outline uppercase font-bold">Total Targets</div>
            <div class="text-xl font-black text-on-surface mt-1">{batchItems.length}</div>
          </div>
          <div class="p-3 bg-surface-container rounded border border-surface-container-high">
            <div class="text-[10px] text-outline uppercase font-bold">Completed</div>
            <div class="text-xl font-black text-emerald-400 mt-1">{completedCount}</div>
          </div>
          <div class="p-3 bg-surface-container rounded border border-surface-container-high">
            <div class="text-[10px] text-outline uppercase font-bold">Average Score</div>
            <div class="text-xl font-black text-on-surface mt-1">
              {averageScore !== null ? `${averageScore}/100` : "—"}
            </div>
          </div>
          <div class="p-3 bg-surface-container rounded border border-surface-container-high">
            <div class="text-[10px] text-outline uppercase font-bold">Critical Targets</div>
            <div class="text-xl font-black {criticalTargetsCount > 0 ? 'text-error' : 'text-on-surface'} mt-1">
              {criticalTargetsCount}
            </div>
          </div>
        </div>

        <!-- Filter Controls -->
        <div class="flex items-center justify-between gap-3 pt-2">
          <div class="flex items-center gap-1.5 overflow-x-auto font-mono text-xs">
            <button
              type="button"
              onclick={() => (activeFilter = "all")}
              class="px-2.5 py-1 rounded uppercase font-bold transition-colors cursor-pointer {activeFilter === 'all' ? 'bg-on-surface text-surface-container-lowest' : 'bg-surface-container text-outline hover:text-on-surface border border-surface-container-high'}"
            >
              All ({batchItems.length})
            </button>
            <button
              type="button"
              onclick={() => (activeFilter = "completed")}
              class="px-2.5 py-1 rounded uppercase font-bold transition-colors cursor-pointer {activeFilter === 'completed' ? 'bg-on-surface text-surface-container-lowest' : 'bg-surface-container text-outline hover:text-on-surface border border-surface-container-high'}"
            >
              Completed ({batchItems.filter(i => i.status === 'completed').length})
            </button>
            <button
              type="button"
              onclick={() => (activeFilter = "critical")}
              class="px-2.5 py-1 rounded uppercase font-bold transition-colors cursor-pointer {activeFilter === 'critical' ? 'bg-on-surface text-surface-container-lowest' : 'bg-surface-container text-outline hover:text-on-surface border border-surface-container-high'}"
            >
              Critical ({criticalTargetsCount})
            </button>
            <button
              type="button"
              onclick={() => (activeFilter = "failed")}
              class="px-2.5 py-1 rounded uppercase font-bold transition-colors cursor-pointer {activeFilter === 'failed' ? 'bg-on-surface text-surface-container-lowest' : 'bg-surface-container text-outline hover:text-on-surface border border-surface-container-high'}"
            >
              Failed ({batchItems.filter(i => i.status === 'failed').length})
            </button>
          </div>

          {#if !isRunning}
            <button
              type="button"
              onclick={startBatchScan}
              class="px-2.5 py-1 bg-surface-container hover:bg-surface-container-highest border border-surface-container-high text-xs font-mono font-bold uppercase text-on-surface rounded flex items-center gap-1.5 transition-colors cursor-pointer shrink-0"
            >
              <RotateCw class="w-3 h-3" />
              <span class="hidden sm:inline">Rerun Fleet Audit</span>
            </button>
          {/if}
        </div>

        <!-- Target Results List -->
        <div class="space-y-2 max-h-[500px] overflow-y-auto">
          {#if filteredItems.length === 0}
            <div class="p-8 text-center text-outline font-mono text-xs uppercase bg-surface-container rounded border border-surface-container-high">
              No targets match the selected filter.
            </div>
          {:else}
            {#each filteredItems as item}
              <div class="p-3.5 bg-surface-container rounded border border-surface-container-high flex flex-col md:flex-row md:items-center justify-between gap-3 font-mono hover:border-surface-container-highest transition-colors">
                <div class="flex items-center gap-3 min-w-0 flex-1">
                  {#if item.status === "scanning"}
                    <Loader2 class="w-4 h-4 text-primary animate-spin shrink-0" />
                  {:else if item.status === "completed"}
                    <CheckCircle2 class="w-4 h-4 text-emerald-400 shrink-0" />
                  {:else}
                    <AlertOctagon class="w-4 h-4 text-error shrink-0" />
                  {/if}

                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <span class="text-xs font-bold text-on-surface truncate">
                        {item.url}
                      </span>
                      <button
                        type="button"
                        onclick={() => copyUrl(item.url)}
                        class="text-outline hover:text-on-surface p-0.5 rounded cursor-pointer"
                        title="Copy target URL"
                      >
                        {#if copiedUrl === item.url}
                          <Check class="w-3 h-3 text-emerald-400" />
                        {:else}
                          <Copy class="w-3 h-3" />
                        {/if}
                      </button>
                    </div>

                    {#if item.error}
                      <div class="text-[11px] text-error mt-0.5 truncate">
                        {item.error}
                      </div>
                    {:else if item.report}
                      <div class="text-[11px] text-outline mt-0.5 flex flex-wrap items-center gap-2">
                        <span>{item.report.total_findings} findings</span>
                        {#if item.report.critical_count > 0}
                          <span class="text-error font-bold">({item.report.critical_count} critical)</span>
                        {/if}
                        {#if item.report.high_count > 0}
                          <span class="text-amber-400">({item.report.high_count} high)</span>
                        {/if}
                        <span>• {item.report.response_time_ms}ms</span>
                        {#if item.report.technologies_detected?.length > 0}
                          <span>• {item.report.technologies_detected.slice(0, 3).join(", ")}</span>
                        {/if}
                      </div>
                    {/if}
                  </div>
                </div>

                <!-- Score and Action Buttons -->
                <div class="flex items-center gap-2.5 shrink-0 self-end md:self-auto">
                  {#if item.report}
                    <span class="px-2 py-0.5 text-xs font-bold rounded {getScoreBadge(item.report.security_score)}">
                      SCORE: {item.report.security_score}/100
                    </span>

                    {#if onSelectReport}
                      <button
                        type="button"
                        onclick={() => onSelectReport(item.report!)}
                        class="px-3 py-1 bg-on-surface text-surface-container-lowest hover:opacity-90 font-bold rounded text-xs uppercase flex items-center gap-1.5 transition-opacity cursor-pointer shadow-xs"
                        title="Load target into primary audit workspace"
                      >
                        <span>Inspect</span>
                        <ArrowRight class="w-3 h-3" />
                      </button>
                    {/if}
                  {:else if item.status === "scanning"}
                    <span class="text-[11px] text-primary font-bold uppercase animate-pulse">
                      Analyzing...
                    </span>
                  {:else}
                    <span class="text-[11px] text-error font-bold uppercase">
                      Audit Failed
                    </span>
                  {/if}
                </div>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {:else}
      <!-- Empty State Guide -->
      <div class="p-8 sm:p-12 text-center bg-surface-container rounded-lg border border-surface-container-high space-y-3 font-mono">
        <div class="w-12 h-12 mx-auto rounded-full bg-surface-container-lowest border border-surface-container-high flex items-center justify-center text-primary">
          <Layers class="w-6 h-6" />
        </div>
        <h4 class="text-sm font-bold text-on-surface uppercase tracking-wider">
          No Active Fleet Queue
        </h4>
        <p class="text-xs text-outline max-w-md mx-auto leading-relaxed">
          Add target domains to the inventory above and click <strong class="text-on-surface">"Launch Fleet Audit"</strong> to begin comprehensive batch security analysis.
        </p>
      </div>
    {/if}
  </div>
</div>
