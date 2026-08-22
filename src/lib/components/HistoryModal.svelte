<script lang="ts">
  import type { ScanSummary } from "$lib/types";
  import { X, Clock, Trash2, Globe, ArrowRight, Shield } from "lucide-svelte";

  let {
    isOpen = false,
    history = [],
    onSelect,
    onDelete,
    onClearAll,
    onClose,
  }: {
    isOpen: boolean;
    history: ScanSummary[];
    onSelect: (id: string) => void;
    onDelete: (id: string) => void;
    onClearAll: () => void;
    onClose: () => void;
  } = $props();

  function formatDate(iso: string) {
    try {
      const d = new Date(iso);
      return d.toLocaleDateString() + " " + d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    } catch {
      return iso;
    }
  }

  function getScoreColor(score: number) {
    if (score >= 85) return "text-emerald-400 border-emerald-500/30 bg-emerald-500/10";
    if (score >= 70) return "text-cyan-400 border-cyan-500/30 bg-cyan-500/10";
    if (score >= 50) return "text-amber-400 border-amber-500/30 bg-amber-500/10";
    return "text-rose-400 border-rose-500/30 bg-rose-500/10";
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4 animate-fade-in">
    <div class="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-2xl max-h-[85vh] flex flex-col shadow-2xl overflow-hidden">
      <!-- Header -->
      <div class="p-5 border-b border-slate-800 flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <Clock class="w-5 h-5 text-cyan-400" />
          <h2 class="text-lg font-bold text-slate-100">Scan History</h2>
          <span class="px-2 py-0.5 text-xs bg-slate-800 text-slate-300 rounded-full font-mono font-semibold">
            {history.length}
          </span>
        </div>

        <div class="flex items-center gap-2">
          {#if history.length > 0}
            <button
              type="button"
              onclick={onClearAll}
              class="px-3 py-1.5 text-xs font-semibold text-rose-400 hover:bg-rose-500/10 rounded-lg transition-colors cursor-pointer border border-transparent hover:border-rose-500/20"
            >
              Clear All
            </button>
          {/if}
          <button
            type="button"
            onclick={onClose}
            class="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-lg transition-colors cursor-pointer"
          >
            <X class="w-5 h-5" />
          </button>
        </div>
      </div>

      <!-- List -->
      <div class="flex-1 overflow-y-auto p-4 space-y-3">
        {#if history.length === 0}
          <div class="py-16 text-center text-slate-500">
            <Globe class="w-12 h-12 mx-auto mb-3 opacity-30" />
            <p class="text-sm font-medium">No previous scans found.</p>
            <p class="text-xs text-slate-600 mt-1">Scanned websites will be automatically recorded here.</p>
          </div>
        {:else}
          {#each history as item}
            <div class="p-4 bg-slate-950/60 hover:bg-slate-950 border border-slate-800/80 hover:border-cyan-500/30 rounded-xl transition-all flex items-center justify-between gap-4">
              <div class="flex items-center gap-3.5 min-w-0 flex-1">
                <!-- Score badge -->
                <div class="w-11 h-11 rounded-lg border flex flex-col items-center justify-center font-mono font-bold {getScoreColor(item.security_score)} flex-shrink-0">
                  <span class="text-sm leading-none">{item.security_score}</span>
                  <span class="text-[9px] uppercase tracking-tighter opacity-80 mt-0.5">pts</span>
                </div>

                <div class="min-w-0 flex-1">
                  <div class="text-sm font-bold text-slate-200 truncate">{item.target_url}</div>
                  <div class="flex items-center gap-3 text-xs text-slate-400 mt-1 font-mono">
                    <span>{formatDate(item.scanned_at)}</span>
                    <span>•</span>
                    <span class="text-slate-300">{item.total_findings} findings</span>
                    {#if item.critical_count > 0}
                      <span class="text-rose-400 font-bold">({item.critical_count} critical)</span>
                    {/if}
                  </div>
                </div>
              </div>

              <div class="flex items-center gap-2 flex-shrink-0">
                <button
                  type="button"
                  onclick={() => onSelect(item.id)}
                  class="px-3 py-1.5 bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-400 border border-cyan-500/30 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-colors cursor-pointer"
                >
                  <span>Load</span>
                  <ArrowRight class="w-3.5 h-3.5" />
                </button>
                <button
                  type="button"
                  onclick={() => onDelete(item.id)}
                  class="p-2 text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 rounded-lg transition-colors cursor-pointer"
                  title="Delete scan"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}
