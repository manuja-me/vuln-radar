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
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-fade-in">
    <div class="bg-[#202020] border border-[#333333] rounded-xl w-full max-w-2xl max-h-[85vh] flex flex-col shadow-xl overflow-hidden text-[#e3e2e0]">
      <!-- Header -->
      <div class="p-4 border-b border-[#2e2e2e] flex items-center justify-between bg-[#191919]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-[#282828] border border-[#383838] flex items-center justify-center text-neutral-300">
            <Clock class="w-4 h-4" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-sm font-semibold text-white tracking-tight">Audit Archive & History</h2>
              <span class="px-1.5 py-0.2 text-[10px] bg-[#282828] text-neutral-400 rounded font-mono border border-[#383838]">
                {history.length} Saved
              </span>
            </div>
            <p class="text-[11px] text-neutral-400">Locally persisted SQLite audit snapshots</p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          {#if history.length > 0}
            <button
              type="button"
              onclick={onClearAll}
              class="px-2.5 py-1 text-xs font-medium text-red-400 hover:bg-red-950/30 rounded-lg transition-colors cursor-pointer"
            >
              Clear Archive
            </button>
          {/if}
          <button
            type="button"
            onclick={onClose}
            class="p-1.5 text-neutral-400 hover:text-white hover:bg-[#282828] rounded-lg transition-colors cursor-pointer"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Search Bar -->
      {#if history.length > 0}
        <div class="p-3 border-b border-[#2e2e2e] bg-[#1a1a1a]">
          <div class="relative">
            <Search class="w-3.5 h-3.5 text-neutral-500 absolute inset-y-0 left-3 my-auto pointer-events-none" />
            <input
              type="text"
              bind:value={searchQuery}
              placeholder="Search previous scans by URL..."
              class="w-full pl-8 pr-3 py-1.5 bg-[#141414] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs font-mono text-neutral-200 placeholder-neutral-500 focus:outline-none"
            />
          </div>
        </div>
      {/if}

      <!-- List -->
      <div class="flex-1 overflow-y-auto p-4 space-y-2">
        {#if filteredHistory.length === 0}
          <div class="py-16 text-center text-neutral-500">
            <Globe class="w-10 h-10 mx-auto mb-2 opacity-30 text-neutral-400" />
            <p class="text-xs font-medium text-neutral-300">No matching scan snapshots</p>
            <p class="text-[11px] text-neutral-500 mt-0.5">Audit any website from the top bar to record it here.</p>
          </div>
        {:else}
          {#each filteredHistory as item}
            <div class="p-3 bg-[#191919] hover:bg-[#232323] border border-[#2e2e2e] rounded-lg transition-colors flex items-center justify-between gap-4">
              <div class="flex items-center gap-3 min-w-0 flex-1">
                <!-- Score badge -->
                <div class="w-9 h-9 rounded-md border flex flex-col items-center justify-center font-mono font-medium {getScoreColor(item.security_score)} flex-shrink-0">
                  <span class="text-xs leading-none">{item.security_score}</span>
                  <span class="text-[8px] uppercase tracking-tighter opacity-80 mt-0.5">pts</span>
                </div>

                <div class="min-w-0 flex-1">
                  <div class="text-xs font-medium text-neutral-200 truncate font-mono">{item.target_url}</div>
                  <div class="flex items-center gap-2 text-[11px] text-neutral-400 mt-0.5 font-mono">
                    <span>{formatDate(item.scanned_at)}</span>
                    <span>•</span>
                    <span class="text-neutral-300">{item.total_findings} findings</span>
                    {#if item.critical_count > 0}
                      <span class="text-red-400 font-medium">({item.critical_count} critical)</span>
                    {/if}
                  </div>
                </div>
              </div>

              <div class="flex items-center gap-2 flex-shrink-0">
                <button
                  type="button"
                  onclick={() => onSelect(item.id)}
                  class="px-2.5 py-1 bg-[#262626] hover:bg-[#303030] text-neutral-200 border border-[#383838] rounded-md text-xs font-medium flex items-center gap-1 transition-colors cursor-pointer"
                >
                  <span>Load</span>
                  <ArrowRight class="w-3 h-3" />
                </button>
                <button
                  type="button"
                  onclick={() => onDelete(item.id)}
                  class="p-1 text-neutral-500 hover:text-red-400 rounded transition-colors cursor-pointer"
                  title="Delete scan"
                >
                  <Trash2 class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}

