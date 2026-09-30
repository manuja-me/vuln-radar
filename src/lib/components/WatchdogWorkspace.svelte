<script lang="ts">
  import type { MonitorTarget } from "$lib/types";
  import {
    Activity,
    Plus,
    X,
    Play,
    Pause,
    Clock,
    RotateCw,
    Shield,
    Globe,
    BellRing,
    Trash2,
  } from "lucide-svelte";

  let {
    monitors = [],
    onAddMonitor,
    onDeleteMonitor,
    onToggleMonitor,
    onScanTarget,
  }: {
    monitors: MonitorTarget[];
    onAddMonitor?: (url: string, intervalHours: number) => Promise<void>;
    onDeleteMonitor?: (id: string) => Promise<void>;
    onToggleMonitor?: (id: string) => Promise<void>;
    onScanTarget?: (url: string) => void;
  } = $props();

  let newUrl = $state("");
  let selectedInterval = $state(24);
  let isAdding = $state(false);
  let showAddForm = $state(false);

  async function handleAdd() {
    const url = newUrl.trim();
    if (!url || !onAddMonitor) return;
    isAdding = true;
    try {
      await onAddMonitor(url, selectedInterval);
      newUrl = "";
      showAddForm = false;
    } finally {
      isAdding = false;
    }
  }

  function formatTime(iso: string) {
    try {
      return new Date(iso).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    } catch {
      return iso;
    }
  }
</script>

<div class="space-y-4 max-w-6xl w-full mx-auto animate-fade-in pb-10 flex-1">
  <!-- Header Bar -->
  <div class="flex items-center justify-between pb-3 border-b border-surface-container-high">
    <div class="flex items-center gap-2.5">
      <Activity class="w-4 h-4 text-primary" />
      <div>
        <h2 class="text-sm font-bold text-on-surface font-mono uppercase tracking-tight">
          Automated Watchdog Daemon ({monitors.length})
        </h2>
        <p class="text-[11px] text-outline font-mono mt-0.5">
          Continuous background vulnerability tracking with degradation alerting
        </p>
      </div>
    </div>

    <button
      type="button"
      onclick={() => (showAddForm = !showAddForm)}
      class="px-3 py-1.5 bg-primary text-on-primary font-bold rounded text-xs font-mono uppercase transition-opacity cursor-pointer hover:opacity-90 flex items-center gap-1.5 shadow-xs"
    >
      <Plus class="w-3.5 h-3.5" />
      <span>{showAddForm ? "Cancel" : "Add Target"}</span>
    </button>
  </div>

  <!-- Inline Add Target Form -->
  {#if showAddForm}
    <div class="p-4 bg-surface-container-low border border-surface-container-high rounded-lg space-y-3 animate-fade-in font-mono">
      <div class="text-xs font-bold uppercase tracking-wider text-on-surface">
        New Watchdog Target
      </div>
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
        <div class="sm:col-span-2">
          <input
            type="text"
            bind:value={newUrl}
            placeholder="https://example.com"
            class="w-full h-9 px-3 text-xs bg-surface-container-lowest rounded border border-surface-container-high font-mono text-on-surface outline-none focus:border-outline"
          />
        </div>
        <div class="flex items-center gap-2">
          <select
            bind:value={selectedInterval}
            class="h-9 px-3 text-xs bg-surface-container-lowest rounded border border-surface-container-high font-mono text-on-surface outline-none cursor-pointer flex-1"
          >
            <option value={1}>Every 1 Hour</option>
            <option value={6}>Every 6 Hours</option>
            <option value={12}>Every 12 Hours</option>
            <option value={24}>Every 24 Hours</option>
          </select>
          <button
            type="button"
            disabled={isAdding || !newUrl.trim()}
            onclick={handleAdd}
            class="h-9 px-4 bg-primary text-on-primary font-bold text-xs uppercase rounded cursor-pointer hover:bg-primary/90 disabled:opacity-50 transition-colors shrink-0"
          >
            {isAdding ? "Adding..." : "Save"}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Targets List -->
  {#if monitors.length === 0}
    <div class="py-16 text-center bg-surface-container-low border border-surface-container-high rounded space-y-3 font-mono">
      <Activity class="w-10 h-10 text-primary mx-auto opacity-70" />
      <h3 class="text-sm font-bold text-on-surface uppercase">No Monitored Targets Active</h3>
      <p class="text-xs text-outline max-w-sm mx-auto leading-relaxed">
        Schedule targets for continuous re-auditing with native desktop alerts upon score degradation.
      </p>
      <button
        type="button"
        onclick={() => (showAddForm = true)}
        class="mt-2 px-3 py-1.5 bg-surface-container hover:bg-surface-container-high border border-surface-container-high rounded text-xs font-bold uppercase text-on-surface transition-colors cursor-pointer"
      >
        + Add Your First Target
      </button>
    </div>
  {:else}
    <div class="space-y-2">
      {#each monitors as m (m.id)}
        <div class="p-3.5 bg-surface-container-low border border-surface-container-high rounded flex items-center justify-between gap-4 font-mono hover:border-surface-container-highest transition-colors">
          <div class="space-y-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="font-bold text-on-surface text-xs truncate">{m.target_url}</span>
              <span class="px-1.5 py-0.2 text-[10px] rounded bg-surface-container text-outline uppercase border border-surface-container-high">
                Every {m.interval_hours}h
              </span>
            </div>
            <div class="text-[11px] text-outline uppercase">
              Next Run: {formatTime(m.next_scan_at)} • Last Score: {m.last_score ?? "Pending"}
            </div>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            {#if onScanTarget}
              <button
                type="button"
                onclick={() => onScanTarget(m.target_url)}
                class="px-2.5 py-1 bg-surface-container hover:bg-surface-bright text-on-surface border border-surface-container-high rounded text-xs font-bold uppercase transition-colors cursor-pointer"
                title="Trigger immediate scan now"
              >
                Scan Now
              </button>
            {/if}
            {#if onToggleMonitor}
              <button
                type="button"
                onclick={() => onToggleMonitor(m.id)}
                class="px-2.5 py-1 rounded text-xs font-bold uppercase transition-colors cursor-pointer {m.is_active ? 'bg-tertiary/20 text-tertiary border border-tertiary/30' : 'bg-surface-container text-outline border border-surface-container-high'}"
              >
                {m.is_active ? "Active" : "Paused"}
              </button>
            {/if}
            {#if onDeleteMonitor}
              <button
                type="button"
                onclick={() => onDeleteMonitor(m.id)}
                class="p-1 text-outline hover:text-error rounded cursor-pointer"
                title="Delete monitor"
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
