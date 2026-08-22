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
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-fade-in"
  >
    <div
      class="bg-[#202020] border border-[#333333] rounded-xl w-full max-w-3xl max-h-[85vh] flex flex-col shadow-xl overflow-hidden text-[#e3e2e0]"
    >
      <!-- Header -->
      <div class="p-4 border-b border-[#2e2e2e] flex items-center justify-between bg-[#191919]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-[#282828] border border-[#383838] flex items-center justify-center text-neutral-300">
            <Activity class="w-4 h-4" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-sm font-semibold text-white tracking-tight">Continuous Security Watchdog</h2>
              <span class="px-1.5 py-0.2 text-[10px] bg-[#282828] text-neutral-400 rounded font-mono border border-[#383838]">
                {monitors.length} Active
              </span>
            </div>
            <p class="text-[11px] text-neutral-400">Automated background re-scanning with degradation alerts</p>
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
        <!-- Add Monitor Form -->
        <div class="p-3.5 bg-[#191919] border border-[#2e2e2e] rounded-lg space-y-2.5">
          <div class="flex items-center gap-1.5 text-xs font-medium text-neutral-400 uppercase tracking-wider">
            <BellRing class="w-3.5 h-3.5 text-neutral-400" />
            <span>Schedule Asset Watchdog</span>
          </div>
          <div class="flex flex-col sm:flex-row items-center gap-2">
            <input
              type="text"
              bind:value={newUrl}
              placeholder="https://domain-to-watch.com"
              class="w-full sm:flex-1 px-3 py-1.5 bg-[#202020] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs font-mono text-neutral-200 placeholder-neutral-500 focus:outline-none"
            />
            <select
              bind:value={selectedInterval}
              class="w-full sm:w-auto px-3 py-1.5 bg-[#202020] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs font-medium text-neutral-300 focus:outline-none cursor-pointer"
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
              class="w-full sm:w-auto px-3.5 py-1.5 bg-white hover:bg-neutral-200 disabled:opacity-50 text-neutral-950 font-semibold rounded-lg text-xs flex items-center justify-center gap-1.5 transition-colors cursor-pointer flex-shrink-0 shadow-sm"
            >
              <Plus class="w-3.5 h-3.5" />
              <span>Add Watchdog</span>
            </button>
          </div>
        </div>

        <!-- Monitor List -->
        <div class="space-y-2.5">
          <div class="flex items-center justify-between">
            <h3 class="text-xs font-medium text-neutral-400 uppercase tracking-wider">
              Configured Domains ({monitors.length})
            </h3>
            <span class="text-[11px] font-mono text-neutral-500">Auto-polls every 60s</span>
          </div>

          {#if monitors.length === 0}
            <div class="py-12 text-center text-neutral-500 bg-[#191919] border border-[#2e2e2e] rounded-lg">
              <Activity class="w-8 h-8 mx-auto mb-2 opacity-30 text-neutral-400" />
              <p class="text-xs font-medium text-neutral-300">No continuous monitors configured</p>
              <p class="text-[11px] text-neutral-500 mt-0.5 max-w-sm mx-auto">
                Add your critical web properties above to track and alert on configuration changes.
              </p>
            </div>
          {:else}
            <div class="space-y-2">
              {#each monitors as item}
                <div
                  class="p-3 bg-[#191919] border border-[#2e2e2e] rounded-lg flex items-center justify-between gap-4"
                >
                  <div class="flex items-center gap-3 min-w-0 flex-1">
                    <!-- Status dot -->
                    <div
                      class="w-2 h-2 rounded-full flex-shrink-0 {item.is_active ? 'bg-emerald-400' : 'bg-neutral-600'}"
                      title={item.is_active ? 'Active Watchdog' : 'Paused Watchdog'}
                    ></div>

                    <div class="min-w-0 flex-1">
                      <div class="flex items-center gap-2">
                        <span class="text-xs font-medium text-neutral-200 font-mono truncate">
                          {item.target_url}
                        </span>
                        <span class="px-1.5 py-0.2 text-[10px] font-mono bg-[#282828] text-neutral-400 rounded border border-[#383838]">
                          Every {item.interval_hours}h
                        </span>
                      </div>

                      <div class="flex flex-wrap items-center gap-2 text-[11px] text-neutral-400 font-mono mt-0.5">
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
                        class="px-2 py-0.5 text-xs font-mono rounded border {getScoreBadge(item.last_score)}"
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
                      class="p-1.5 text-neutral-300 hover:text-white hover:bg-[#282828] rounded-md transition-colors cursor-pointer"
                      title="Run Audit Now"
                    >
                      <RotateCw class="w-3.5 h-3.5" />
                    </button>

                    <button
                      type="button"
                      onclick={() => onToggleMonitor(item.id)}
                      class="p-1.5 text-neutral-400 hover:text-white hover:bg-[#282828] rounded-md transition-colors cursor-pointer"
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
                      class="p-1.5 text-neutral-500 hover:text-red-400 rounded-md transition-colors cursor-pointer"
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

