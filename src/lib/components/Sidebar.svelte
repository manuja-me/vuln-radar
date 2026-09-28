<script lang="ts">
  import { onMount } from "svelte";
  import {
    Shield,
    Radar,
    Globe,
    FolderSearch,
    Layers,
    Activity,
    History,
    Settings,
    Sun,
    Moon,
    Sparkles,
  } from "lucide-svelte";
  import type { SwissTheme } from "$lib/types";

  let {
    currentWorkspace = "audit",
    activeMonitorsCount = 0,
    historyCount = 0,
    currentVersion = "1.4.0",
    hasUpdateAvailable = false,
    updateVersion = "",
    isScanning = false,
    onSelectWorkspace,
    onCheckUpdates,
  }: {
    currentWorkspace: string;
    activeMonitorsCount?: number;
    historyCount?: number;
    currentVersion?: string;
    hasUpdateAvailable?: boolean;
    updateVersion?: string;
    isScanning?: boolean;
    onSelectWorkspace: (ws: string) => void;
    onCheckUpdates?: () => void;
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

  let currentTheme = $state<SwissTheme>("swiss-dark");

  onMount(() => {
    try {
      const saved = localStorage.getItem("vulnradar_theme") as SwissTheme;
      if (saved === "swiss-light" || saved === "swiss-dark") {
        currentTheme = saved;
      } else {
        currentTheme =
          (document.documentElement.getAttribute("data-theme") as SwissTheme) ||
          "swiss-dark";
      }
      document.documentElement.classList.toggle("dark", currentTheme === "swiss-dark");
      document.documentElement.setAttribute("data-theme", currentTheme);
    } catch {}
  });

  function toggleTheme() {
    currentTheme = currentTheme === "swiss-dark" ? "swiss-light" : "swiss-dark";
    try {
      localStorage.setItem("vulnradar_theme", currentTheme);
      document.documentElement.setAttribute("data-theme", currentTheme);
      document.documentElement.classList.toggle("dark", currentTheme === "swiss-dark");
    } catch {}
  }
</script>

<aside
  class="w-52 sm:w-56 bg-surface-container border-r border-surface-container-high flex flex-col justify-between shrink-0 select-none overflow-y-auto print:hidden"
>
  <div class="flex flex-col p-3 gap-1.5">
    <!-- Brand & Version Header -->
    <div class="flex items-center justify-between px-2.5 py-2 mb-1 border-b border-surface-container-high/60">
      <div class="flex items-center gap-2">
        <img src="/favicon.png" alt="VulnRadar" class="h-4 w-4 object-contain" />
        <span class="font-mono text-xs font-black uppercase tracking-wider text-on-surface">VulnRadar</span>
      </div>
      {#if onCheckUpdates}
        <button
          type="button"
          onclick={onCheckUpdates}
          class="font-mono text-[10px] text-outline hover:text-primary transition-colors cursor-pointer px-1 py-0.5 rounded hover:bg-surface-container-high flex items-center gap-1"
          title="Check for software updates"
        >
          <span>v{currentVersion}</span>
          {#if hasUpdateAvailable}
            <span class="w-1.5 h-1.5 rounded-full bg-primary animate-pulse" title={`Update v${updateVersion} available`}></span>
          {/if}
        </button>
      {:else}
        <span class="font-mono text-[10px] text-outline">v{currentVersion}</span>
      {/if}
    </div>

    <!-- Update Available Notification Banner -->
    {#if hasUpdateAvailable && onCheckUpdates}
      <button
        type="button"
        onclick={onCheckUpdates}
        class="mb-1 p-2 bg-primary/10 hover:bg-primary/20 border border-primary/40 text-primary text-[10px] font-mono font-bold uppercase rounded flex items-center justify-between transition-colors animate-pulse cursor-pointer"
        title="Software update ready"
      >
        <div class="flex items-center gap-1.5">
          <Sparkles class="w-3 h-3 shrink-0" />
          <span>Update Ready</span>
        </div>
        <span class="font-mono">{updateVersion ? `v${updateVersion}` : "New"}</span>
      </button>
    {/if}

    <div class="px-2.5 py-1 text-[10px] font-mono uppercase tracking-wider text-outline">
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

  <!-- Bottom Workspace Status & Theme Switcher -->
  <div class="p-3 px-3.5 border-t border-surface-container-high text-[11px] font-mono text-outline flex items-center justify-between">
    <div class="flex items-center gap-1.5">
      <span class="w-1.5 h-1.5 rounded-full {isScanning ? 'bg-primary animate-ping' : 'bg-tertiary'}"></span>
      <span class="text-xs">{isScanning ? "Scanning..." : "Engine Ready"}</span>
    </div>
    <button
      type="button"
      onclick={toggleTheme}
      class="p-1.5 text-outline hover:text-on-surface hover:bg-surface-container-highest rounded cursor-pointer transition-colors"
      title="Toggle Dark / Light Theme"
      aria-label="Toggle theme"
    >
      {#if currentTheme === "swiss-dark"}
        <Moon class="w-3.5 h-3.5 text-secondary" />
      {:else}
        <Sun class="w-3.5 h-3.5 text-amber-500" />
      {/if}
    </button>
  </div>
</aside>
