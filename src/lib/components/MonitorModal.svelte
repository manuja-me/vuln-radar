<script lang="ts">
  import type { MonitorTarget } from "$lib/types";
  import {
    X,
    Activity,
    Plus,
    Trash2,
    Play,
    Pause,
    Clock,
    RotateCw,
    Shield,
    Globe,
    BellRing,
  } from "lucide-svelte";

  let {
    isOpen = false,
    monitors = [],
    onAddMonitor,
    onDeleteMonitor,
    onToggleMonitor,
    onScanNow,
    onClose,
  }: {
    isOpen: boolean;
    monitors: MonitorTarget[];
    onAddMonitor: (url: string, intervalHours: number) => Promise<void>;
    onDeleteMonitor: (id: string) => Promise<void>;
    onToggleMonitor: (id: string) => Promise<void>;
    onScanNow: (url: string) => void;
    onClose: () => void;
  } = $props();

  let newUrl = $state("");
  let selectedInterval = $state(24);
  let isAdding = $state(false);

  function formatDate(iso?: string | null) {
    if (!iso) return "Never";
    try {
      const d = new Date(iso);
      return d.toLocaleDateString() + " " + d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    } catch {
      return iso;
    }
  }

  async function handleAdd() {
    const url = newUrl.trim();
    if (!url) return;
    isAdding = true;
    try {
      await onAddMonitor(url, selectedInterval);
      newUrl = "";
    } finally {
      isAdding = false;
    }
  }

  function getScoreBadge(score?: number | null) {
    if (score === undefined || score === null) return "bg-[var(--color-canvas)] text-[var(--color-text-muted)] border border-[var(--color-hairline)] rounded-none";
    if (score >= 85) return "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30 rounded-none";
    if (score >= 70) return "bg-blue-500/10 text-blue-600 dark:text-blue-400 border border-blue-500/30 rounded-none";
    if (score >= 50) return "bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/30 rounded-none";
    return "bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/30 rounded-none";
  }
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
    <div
      class="bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none w-full max-w-3xl max-h-[85vh] flex flex-col shadow-2xl overflow-hidden text-[var(--color-text-body)]"
    >
      <!-- Header -->
      <div class="p-4 border-b border-[var(--color-hairline)] flex items-center justify-between bg-[var(--color-surface)]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-none bg-[var(--color-canvas)] border border-[var(--color-hairline)] flex items-center justify-center text-[var(--color-signal-red)]">
            <Activity class="w-4 h-4" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-xs font-black text-[var(--color-text-headline)] uppercase tracking-tight font-mono">Continuous Security Watchdog</h2>
              <span class="px-1.5 py-0.2 text-[10px] bg-[var(--color-canvas)] text-[var(--color-text-muted)] rounded-none font-mono border border-[var(--color-hairline)] uppercase font-bold">
                {monitors.length} ACTIVE
              </span>
            </div>
            <p class="text-[11px] text-[var(--color-text-muted)] font-mono uppercase">Automated background re-scanning with degradation alerts</p>
          </div>
        </div>
        <button
          type="button"
          onclick={onClose}
          class="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] rounded-none transition-colors cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-5 space-y-5">
        <!-- Add Monitor Form -->
        <div class="p-3.5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-2.5">
          <div class="flex items-center gap-1.5 text-xs font-bold text-[var(--color-text-headline)] uppercase tracking-wider font-mono">
            <BellRing class="w-3.5 h-3.5 text-[var(--color-signal-red)]" />
            <span>Schedule Asset Watchdog</span>
          </div>
          <div class="flex flex-col sm:flex-row items-center gap-2">
            <input
              type="text"
              bind:value={newUrl}
              placeholder="HTTPS://DOMAIN-TO-WATCH.COM"
              class="w-full sm:flex-1 px-3 py-1.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs font-mono text-[var(--color-text-headline)] placeholder-[var(--color-text-muted)] uppercase focus:outline-none"
            />
            <select
              bind:value={selectedInterval}
              class="w-full sm:w-auto px-3 py-1.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs font-mono uppercase font-bold text-[var(--color-text-headline)] focus:outline-none cursor-pointer"
            >
              <option value={1}>Every 1 hour</option>
              <option value={6}>Every 6 hours</option>
              <option value={12}>Every 12 hours</option>
              <option value={24}>Every 24 hours</option>
              <option value={168}>Every 7 days</option>
            </select>
            <button
              type="button"
              onclick={handleAdd}
              disabled={!newUrl.trim() || isAdding}
              class="w-full sm:w-auto px-3.5 py-1.5 bg-[var(--color-text-headline)] hover:opacity-90 disabled:opacity-50 text-[var(--color-canvas)] font-mono font-bold uppercase rounded-none text-xs flex items-center justify-center gap-1.5 transition-opacity cursor-pointer flex-shrink-0"
            >
              <Plus class="w-3.5 h-3.5" />
              <span>Add Watchdog</span>
            </button>
          </div>
        </div>

        <!-- Monitor List -->
        <div class="space-y-2.5">
          <div class="flex items-center justify-between">
            <h3 class="text-xs font-bold text-[var(--color-text-headline)] uppercase tracking-wider font-mono">
              Configured Domains ({monitors.length})
            </h3>
            <span class="text-[11px] font-mono text-[var(--color-text-muted)] uppercase">Auto-polls every 60s</span>
          </div>

          {#if monitors.length === 0}
            <div class="py-12 text-center text-[var(--color-text-muted)] bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none font-mono">
              <Activity class="w-8 h-8 mx-auto mb-2 opacity-30 text-[var(--color-text-muted)]" />
              <p class="text-xs font-bold text-[var(--color-text-headline)] uppercase">No continuous monitors configured</p>
              <p class="text-[11px] text-[var(--color-text-muted)] mt-0.5 max-w-sm mx-auto">
                Add your critical web properties above to track and alert on configuration changes.
              </p>
            </div>
          {:else}
            <div class="space-y-2">
              {#each monitors as item}
                <div
                  class="p-3 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none flex items-center justify-between gap-4"
                >
                  <div class="flex items-center gap-3 min-w-0 flex-1">
                    <!-- Status dot -->
                    <div
                      class="w-2 h-2 rounded-none flex-shrink-0 {item.is_active ? 'bg-emerald-500' : 'bg-[var(--color-text-muted)]'}"
                      title={item.is_active ? 'Active Watchdog' : 'Paused Watchdog'}
                    ></div>

                    <div class="min-w-0 flex-1">
                      <div class="flex items-center gap-2">
                        <span class="text-xs font-bold text-[var(--color-text-headline)] font-mono truncate">
                          {item.target_url}
                        </span>
                        <span class="px-1.5 py-0.2 text-[10px] font-mono bg-[var(--color-canvas)] text-[var(--color-text-muted)] rounded-none border border-[var(--color-hairline)] uppercase font-bold">
                          Every {item.interval_hours}h
                        </span>
                      </div>

                      <div class="flex flex-wrap items-center gap-2 text-[11px] text-[var(--color-text-muted)] font-mono mt-0.5 uppercase">
                        <span>Last: {formatDate(item.last_scanned_at)}</span>
                        <span>•</span>
                        <span>Next: {formatDate(item.next_scan_at)}</span>
                      </div>
                    </div>
                  </div>

                  <!-- Right actions -->
                  <div class="flex items-center gap-1.5 flex-shrink-0">
                    {#if item.last_score !== null && item.last_score !== undefined}
                      <span
                        class="px-2 py-0.5 text-xs font-mono font-bold uppercase rounded-none border {getScoreBadge(item.last_score)}"
                      >
                        {item.last_score} pts
                      </span>
                    {/if}

                    <button
                      type="button"
                      onclick={() => {
                        onScanNow(item.target_url);
                        onClose();
                      }}
                      class="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] rounded-none transition-colors cursor-pointer"
                      title="Run Audit Now"
                    >
                      <RotateCw class="w-3.5 h-3.5" />
                    </button>

                    <button
                      type="button"
                      onclick={() => onToggleMonitor(item.id)}
                      class="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] rounded-none transition-colors cursor-pointer"
                      title={item.is_active ? 'Pause Watchdog' : 'Resume Watchdog'}
                    >
                      {#if item.is_active}
                        <Pause class="w-3.5 h-3.5" />
                      {:else}
                        <Play class="w-3.5 h-3.5" />
                      {/if}
                    </button>

                    <button
                      type="button"
                      onclick={() => onDeleteMonitor(item.id)}
                      class="p-1.5 text-[var(--color-text-muted)] hover:text-red-500 rounded-none transition-colors cursor-pointer"
                      title="Delete Watchdog"
                    >
                      <Trash2 class="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

