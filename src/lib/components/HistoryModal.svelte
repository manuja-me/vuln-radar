<script lang="ts">
  import type { ScanSummary } from "$lib/types";
  import { X, Clock, Trash2, Globe, ArrowRight, Shield, Search } from "lucide-svelte";

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

  let searchQuery = $state("");

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

  const filteredHistory = $derived.by(() => {
    if (!searchQuery.trim()) return history;
    return history.filter((item) =>
      item.target_url.toLowerCase().includes(searchQuery.toLowerCase())
    );
  });
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-4 animate-fade-in">
    <div class="bg-slate-900/95 border border-slate-800 rounded-2xl w-full max-w-2xl max-h-[85vh] flex flex-col shadow-2xl overflow-hidden backdrop-blur-2xl">
      <!-- Header -->
      <div class="p-5 border-b border-slate-800/80 flex items-center justify-between bg-slate-950/40">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl bg-cyan-500/10 border border-cyan-500/30 flex items-center justify-center text-cyan-400 shadow-lg shadow-cyan-500/10">
            <Clock class="w-4 h-4" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-base font-black text-white tracking-tight">Audit Archive & History</h2>
              <span class="px-2 py-0.2 text-[10px] bg-slate-800 text-cyan-400 rounded-full font-mono font-bold border border-slate-700">
                {history.length} Saved
              </span>
            </div>
            <p class="text-[11px] text-slate-400">Locally persisted SQLite audit snapshots</p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          {#if history.length > 0}
            <button
              type="button"
              onclick={onClearAll}
              class="px-2.5 py-1 text-xs font-semibold text-rose-400 hover:bg-rose-500/10 rounded-lg transition-colors cursor-pointer border border-transparent hover:border-rose-500/20"
            >
              Clear Archive
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

      <!-- Search Bar -->
      {#if history.length > 0}
        <div class="p-3 border-b border-slate-800/60 bg-slate-950/20">
          <div class="relative">
            <Search class="w-4 h-4 text-slate-500 absolute inset-y-0 left-3 my-auto pointer-events-none" />
            <input
              type="text"
              bind:value={searchQuery}
              placeholder="Search previous scans by URL..."
              class="w-full pl-9 pr-3 py-1.5 bg-slate-950 border border-slate-800 focus:border-cyan-500 rounded-lg text-xs font-mono text-slate-200 placeholder-slate-600 focus:outline-none"
            />
          </div>
        </div>
      {/if}

      <!-- List -->
      <div class="flex-1 overflow-y-auto p-4 space-y-2.5">
        {#if filteredHistory.length === 0}
          <div class="py-16 text-center text-slate-500">
            <Globe class="w-12 h-12 mx-auto mb-3 opacity-30 text-cyan-400" />
            <p class="text-xs font-bold text-slate-300">No matching scan snapshots</p>
            <p class="text-[11px] text-slate-500 mt-1">Audit any website from the top bar to record it here.</p>
          </div>
        {:else}
          {#each filteredHistory as item}
            <div class="p-3.5 bg-slate-950/80 hover:bg-slate-950 border border-slate-800/90 hover:border-cyan-500/40 rounded-xl transition-all flex items-center justify-between gap-4 shadow-sm">
              <div class="flex items-center gap-3 min-w-0 flex-1">
                <!-- Score badge -->
                <div class="w-11 h-11 rounded-lg border flex flex-col items-center justify-center font-mono font-bold {getScoreColor(item.security_score)} flex-shrink-0 shadow-sm">
                  <span class="text-sm leading-none">{item.security_score}</span>
                  <span class="text-[9px] uppercase tracking-tighter opacity-80 mt-0.5">pts</span>
                </div>

                <div class="min-w-0 flex-1">
                  <div class="text-xs font-bold text-slate-200 truncate font-mono">{item.target_url}</div>
                  <div class="flex items-center gap-2.5 text-[11px] text-slate-400 mt-1 font-mono">
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
                  class="px-2.5 py-1.5 bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-400 border border-cyan-500/30 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-colors cursor-pointer"
                >
                  <span>Load</span>
                  <ArrowRight class="w-3.5 h-3.5" />
                </button>
                <button
                  type="button"
                  onclick={() => onDelete(item.id)}
                  class="p-1.5 text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 rounded-lg transition-colors cursor-pointer"
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

