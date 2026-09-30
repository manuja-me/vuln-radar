<script lang="ts">
  import type { ScanSummary } from "$lib/types";
  import { HardDrive, X, Trash2 } from "lucide-svelte";

  let {
    history = [],
    onSelectScan,
    onDeleteScan,
    onClearHistory,
  }: {
    history: ScanSummary[];
    onSelectScan?: (id: string) => void;
    onDeleteScan?: (id: string) => void;
    onClearHistory?: () => void;
  } = $props();

  function getScoreBadge(score?: number) {
    if (score === undefined || score === null) return "bg-surface-container text-outline border border-surface-container-high";
    if (score >= 85) return "bg-emerald-500/10 text-emerald-400 border border-emerald-500/30";
    if (score >= 70) return "bg-blue-500/10 text-blue-400 border border-blue-500/30";
    if (score >= 50) return "bg-amber-500/10 text-amber-400 border border-amber-500/30";
    return "bg-error/10 text-error border border-error/30";
  }
</script>

<div class="space-y-4 max-w-6xl w-full mx-auto animate-fade-in pb-10 flex-1 font-mono">
  <!-- Header Bar -->
  <div class="flex items-center justify-between pb-3 border-b border-surface-container-high">
    <div class="flex items-center gap-2.5">
      <HardDrive class="w-4 h-4 text-primary" />
      <div>
        <h2 class="text-sm font-bold text-on-surface uppercase tracking-tight">
          Local SQLite Audit Logs ({history.length})
        </h2>
        <p class="text-[11px] text-outline mt-0.5">
          Persisted assessment snapshots with historical vulnerability scores
        </p>
      </div>
    </div>

    {#if history.length > 0 && onClearHistory}
      <button
        type="button"
        onclick={onClearHistory}
        class="px-3 py-1.5 bg-surface-container-low hover:bg-surface-container text-error border border-error/30 rounded text-xs uppercase font-bold transition-colors cursor-pointer flex items-center gap-1.5"
      >
        <Trash2 class="w-3.5 h-3.5" />
        <span>Clear History</span>
      </button>
    {/if}
  </div>

  <!-- Historical Log Items -->
  {#if history.length === 0}
    <div class="py-16 text-center bg-surface-container-low border border-surface-container-high rounded text-outline text-xs uppercase space-y-2">
      <HardDrive class="w-8 h-8 mx-auto text-outline opacity-50" />
      <div>No historical scan logs recorded yet.</div>
    </div>
  {:else}
    <div class="space-y-2">
      {#each history as item (item.id)}
        <div class="p-3 bg-surface-container-low border border-surface-container-high rounded flex items-center justify-between gap-4 hover:border-surface-container-highest transition-colors">
          <div class="space-y-1 min-w-0 flex-1">
            <div class="flex items-center gap-2">
              <span class="font-bold text-on-surface text-xs truncate">{item.target_url}</span>
              <span class="px-2 py-0.5 text-[10px] rounded font-bold {getScoreBadge(item.security_score)}">
                SCORE: {item.security_score}/100
              </span>
            </div>
            <div class="text-[11px] text-outline uppercase">
              {new Date(item.scanned_at).toLocaleString()} • {item.total_findings} FINDINGS ({item.critical_count} CRITICAL)
            </div>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            {#if onSelectScan}
              <button
                type="button"
                onclick={() => onSelectScan(item.id)}
                class="px-3 py-1 bg-on-surface text-surface-container-lowest font-bold rounded text-xs uppercase transition-opacity cursor-pointer hover:opacity-90 shadow-xs"
              >
                Load
              </button>
            {/if}
            {#if onDeleteScan}
              <button
                type="button"
                onclick={() => onDeleteScan(item.id)}
                class="p-1 text-outline hover:text-error rounded cursor-pointer"
                title="Delete scan"
              >
                <X class="w-4 h-4" />
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
