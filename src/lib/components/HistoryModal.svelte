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
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 animate-fade-in"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
    onkeydown={(e) => {
      if (e.key === "Escape") onClose();
    }}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div class="bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none w-full max-w-2xl max-h-[85vh] flex flex-col shadow-2xl overflow-hidden text-[var(--color-text-body)]">
      <!-- Header -->
      <div class="p-4 border-b border-[var(--color-hairline)] flex items-center justify-between bg-[var(--color-surface)]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-none bg-[var(--color-canvas)] border border-[var(--color-hairline)] flex items-center justify-center text-[var(--color-signal-red)]">
            <Clock class="w-4 h-4" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-xs font-black text-[var(--color-text-headline)] font-mono uppercase tracking-tight">Audit Archive & History</h2>
              <span class="px-1.5 py-0.2 text-[10px] bg-[var(--color-canvas)] text-[var(--color-text-muted)] rounded-none font-mono border border-[var(--color-hairline)] uppercase">
                {history.length} SAVED
              </span>
            </div>
            <p class="text-[11px] text-[var(--color-text-muted)] font-mono uppercase">Locally persisted SQLite audit snapshots</p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          {#if history.length > 0}
            <button
              type="button"
              onclick={onClearAll}
              class="px-2.5 py-1 text-xs font-mono font-bold text-red-600 dark:text-red-400 hover:bg-red-500/10 border border-transparent hover:border-red-500/30 rounded-none transition-colors cursor-pointer uppercase"
            >
              Clear Archive
            </button>
          {/if}
          <button
            type="button"
            onclick={onClose}
            class="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] rounded-none transition-colors cursor-pointer"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Search Bar -->
      {#if history.length > 0}
        <div class="p-3 border-b border-[var(--color-hairline)] bg-[var(--color-surface)]">
          <div class="relative">
            <Search class="w-3.5 h-3.5 text-[var(--color-text-muted)] absolute inset-y-0 left-3 my-auto pointer-events-none" />
            <input
              type="text"
              bind:value={searchQuery}
              placeholder="SEARCH PREVIOUS SCANS BY URL..."
              class="w-full pl-8 pr-3 py-1.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs font-mono text-[var(--color-text-headline)] placeholder-[var(--color-text-muted)] uppercase focus:outline-none"
            />
          </div>
        </div>
      {/if}

      <!-- List -->
      <div class="flex-1 overflow-y-auto p-4 space-y-2">
        {#if filteredHistory.length === 0}
          <div class="py-16 text-center text-[var(--color-text-muted)]">
            <Globe class="w-10 h-10 mx-auto mb-2 opacity-30 text-[var(--color-text-muted)]" />
            <p class="text-xs font-mono font-bold text-[var(--color-text-headline)] uppercase">No matching scan snapshots</p>
            <p class="text-[11px] text-[var(--color-text-muted)] mt-0.5 font-mono">Audit any website from the top bar to record it here.</p>
          </div>
        {:else}
          {#each filteredHistory as item}
            <div class="p-3 bg-[var(--color-surface)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] rounded-none transition-colors flex items-center justify-between gap-4">
              <div class="flex items-center gap-3 min-w-0 flex-1">
                <!-- Score badge -->
                <div class="w-9 h-9 rounded-none border flex flex-col items-center justify-center font-mono font-bold {getScoreColor(item.security_score)} flex-shrink-0">
                  <span class="text-xs leading-none">{item.security_score}</span>
                  <span class="text-[8px] uppercase tracking-tighter opacity-80 mt-0.5 font-mono">pts</span>
                </div>

                <div class="min-w-0 flex-1">
                  <div class="text-xs font-bold text-[var(--color-text-headline)] truncate font-mono">{item.target_url}</div>
                  <div class="flex items-center gap-2 text-[11px] text-[var(--color-text-muted)] mt-0.5 font-mono uppercase">
                    <span>{formatDate(item.scanned_at)}</span>
                    <span>•</span>
                    <span class="text-[var(--color-text-body)]">{item.total_findings} findings</span>
                    {#if item.critical_count > 0}
                      <span class="text-red-500 font-bold">({item.critical_count} critical)</span>
                    {/if}
                  </div>
                </div>
              </div>

              <div class="flex items-center gap-2 flex-shrink-0">
                <button
                  type="button"
                  onclick={() => onSelect(item.id)}
                  class="px-2.5 py-1 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs font-mono font-bold uppercase flex items-center gap-1 transition-colors cursor-pointer"
                >
                  <span>LOAD</span>
                  <ArrowRight class="w-3 h-3" />
                </button>
                <button
                  type="button"
                  onclick={() => onDelete(item.id)}
                  class="p-1 text-[var(--color-text-muted)] hover:text-red-500 rounded-none transition-colors cursor-pointer"
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

