<script lang="ts">
  import {
    Shield,
    Radar,
    Globe,
    FolderSearch,
    Layers,
    Activity,
    History,
    Settings,
  } from "lucide-svelte";

  let {
    currentWorkspace = "audit",
    activeMonitorsCount = 0,
    historyCount = 0,
    onSelectWorkspace,
  }: {
    currentWorkspace: string;
    activeMonitorsCount?: number;
    historyCount?: number;
    onSelectWorkspace: (ws: string) => void;
  } = $props();

  const workspaces = [
    { id: "audit", num: "01", label: "AUDIT", icon: Shield },
    { id: "ports", num: "02", label: "PORTS", icon: Radar },
    { id: "dns", num: "03", label: "DNS", icon: Globe },
    { id: "paths", num: "04", label: "PATHS", icon: FolderSearch },
    { id: "batch", num: "05", label: "FLEET", icon: Layers },
    { id: "watchdog", num: "06", label: "WATCHDOG", icon: Activity },
    { id: "history", num: "07", label: "LOGS", icon: History },
    { id: "settings", num: "08", label: "SETTINGS", icon: Settings },
  ];
</script>

<aside
  class="w-52 sm:w-56 bg-surface-container border-r border-surface-container-high flex flex-col justify-between shrink-0 select-none overflow-y-auto print:hidden"
>
  <div class="flex flex-col p-3 gap-1.5">
    <div class="px-3 py-1.5 text-[10px] font-mono uppercase tracking-wider text-outline">
      Workspaces
    </div>
    {#each workspaces as ws}
      {@const active = currentWorkspace === ws.id}
      {@const Icon = ws.icon}
      <button
        type="button"
        onclick={() => onSelectWorkspace(ws.id)}
        class="h-10 px-3 rounded font-mono text-xs flex items-center justify-between transition-colors cursor-pointer {active
          ? 'bg-surface-container-highest text-on-surface font-bold border-l-2 border-primary'
          : 'text-outline hover:text-on-surface hover:bg-surface-container-low'}"
      >
        <div class="flex items-center gap-2.5 min-w-0">
          <Icon class="w-4 h-4 shrink-0 {active ? 'text-primary' : 'text-outline'}" />
          <span class="tracking-wider truncate">{ws.num}/{ws.label}</span>
        </div>
        {#if ws.id === "watchdog" && activeMonitorsCount > 0}
          <span class="px-2 py-0.5 text-[10px] bg-tertiary/20 text-tertiary font-bold rounded">
            {activeMonitorsCount}
          </span>
        {:else if ws.id === "history" && historyCount > 0}
          <span class="px-2 py-0.5 text-[10px] bg-surface-container-lowest text-outline font-bold rounded">
            {historyCount}
          </span>
        {/if}
      </button>
    {/each}
  </div>

  <!-- Bottom Workspace Status Footer -->
  <div class="p-3.5 px-4 border-t border-surface-container-high text-[11px] font-mono text-outline flex items-center justify-between">
    <span>VulnRadar Engine</span>
    <span class="text-tertiary">Active</span>
  </div>
</aside>
