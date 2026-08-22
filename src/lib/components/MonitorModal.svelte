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
            <Activity class="w-4 h-4" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-base font-black text-white tracking-tight">Continuous Security Watchdog</h2>
              <span class="px-2 py-0.2 text-[10px] bg-slate-800 text-cyan-400 rounded-full font-mono font-bold border border-slate-700">
                {monitors.length} Active
              </span>
            </div>
            <p class="text-[11px] text-slate-400">Automated background re-scanning with degradation alerts</p>
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
        <!-- Add Monitor Form -->
        <div class="p-4 bg-slate-950/60 border border-slate-800 rounded-xl space-y-3 shadow-inner">
          <div class="flex items-center gap-1.5 text-xs font-mono font-bold text-slate-300 uppercase tracking-wider">
            <BellRing class="w-3.5 h-3.5 text-cyan-400" />
            <span>Schedule Asset Watchdog</span>
          </div>
          <div class="flex flex-col sm:flex-row items-center gap-2">
            <input
              type="text"
              bind:value={newUrl}
              placeholder="https://domain-to-watch.com"
              class="w-full sm:flex-1 px-3.5 py-2.5 bg-slate-900 border border-slate-800 focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500/40 rounded-xl text-xs font-mono text-slate-200 placeholder-slate-600 focus:outline-none"
            />
            <select
              bind:value={selectedInterval}
              class="w-full sm:w-auto px-3.5 py-2.5 bg-slate-900 border border-slate-800 focus:border-cyan-500 rounded-xl text-xs font-semibold text-slate-300 focus:outline-none cursor-pointer"
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
              class="w-full sm:w-auto px-4 py-2.5 bg-gradient-to-r from-cyan-500 to-cyan-400 hover:from-cyan-400 hover:to-cyan-300 disabled:opacity-50 text-slate-950 font-bold rounded-xl text-xs flex items-center justify-center gap-1.5 transition-all shadow-lg shadow-cyan-500/20 cursor-pointer flex-shrink-0"
            >
              <Plus class="w-4 h-4" />
              <span>Add Watchdog</span>
            </button>
          </div>
        </div>

        <!-- Monitor List -->
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <h3 class="text-xs font-mono font-bold text-slate-400 uppercase tracking-wider">
              Configured Domains ({monitors.length})
            </h3>
            <span class="text-[11px] font-mono text-slate-500">Auto-polls every 60s</span>
          </div>

          {#if monitors.length === 0}
            <div class="py-14 text-center text-slate-500 bg-slate-950/40 border border-slate-800/80 rounded-xl">
              <Activity class="w-10 h-10 mx-auto mb-2 opacity-30 text-cyan-400" />
              <p class="text-xs font-bold text-slate-300">No continuous monitors configured</p>
              <p class="text-[11px] text-slate-500 mt-1 max-w-sm mx-auto">
                Add your critical web properties above. VulnRadar will passively inspect them on schedule and alert you when scores drop.
              </p>
            </div>
          {:else}
            <div class="space-y-2.5">
              {#each monitors as item}
                <div
                  class="p-4 bg-slate-950/80 border border-slate-800/90 rounded-xl flex items-center justify-between gap-4 shadow-sm"
                >
                  <div class="flex items-center gap-3.5 min-w-0 flex-1">
                    <!-- Animated Pulse Indicator -->
                    <div
                      class="w-2.5 h-2.5 rounded-full flex-shrink-0 {item.is_active ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)] animate-pulse' : 'bg-slate-600'}"
                      title={item.is_active ? 'Active Watchdog' : 'Paused Watchdog'}
                    ></div>

                    <div class="min-w-0 flex-1">
                      <div class="flex items-center gap-2">
                        <span class="text-xs font-bold text-slate-200 font-mono truncate">
                          {item.target_url}
                        </span>
                        <span class="px-2 py-0.5 text-[10px] font-mono bg-slate-800/80 text-slate-400 rounded-md border border-slate-700/60">
                          Every {item.interval_hours}h
                        </span>
                      </div>

                      <div class="flex flex-wrap items-center gap-3 text-[11px] text-slate-400 font-mono mt-1">
                        <span>Last: {formatDate(item.last_scanned_at)}</span>
                        <span>•</span>
                        <span>Next: {formatDate(item.next_scan_at)}</span>
                      </div>
                    </div>
                  </div>

                  <!-- Right actions -->
                  <div class="flex items-center gap-2 flex-shrink-0">
                    {#if item.last_score !== null && item.last_score !== undefined}
                      <span
                        class="px-2.5 py-0.5 text-xs font-mono font-bold rounded-md border {getScoreBadge(item.last_score)}"
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
                      class="p-2 text-cyan-400 hover:bg-cyan-500/10 rounded-lg transition-colors cursor-pointer"
                      title="Run Audit Now"
                    >
                      <RotateCw class="w-4 h-4" />
                    </button>

                    <button
                      type="button"
                      onclick={() => onToggleMonitor(item.id)}
                      class="p-2 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-lg transition-colors cursor-pointer"
                      title={item.is_active ? 'Pause Watchdog' : 'Resume Watchdog'}
                    >
                      {#if item.is_active}
                        <Pause class="w-4 h-4" />
                      {:else}
                        <Play class="w-4 h-4" />
                      {/if}
                    </button>

                    <button
                      type="button"
                      onclick={() => onDeleteMonitor(item.id)}
                      class="p-2 text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 rounded-lg transition-colors cursor-pointer"
                      title="Delete Watchdog"
                    >
                      <Trash2 class="w-4 h-4" />
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

