<script lang="ts">
  import { onMount } from "svelte";
  import {
    Settings,
    FileDown,
    Sun,
    Moon,
    HelpCircle,
    Sparkles,
  } from "lucide-svelte";
  import type { SwissTheme } from "$lib/types";

  let {
    isScanning = false,
    hasReport = false,
    hasCustomOptions = false,
    activeMonitorsCount = 0,
    hasUpdateAvailable = false,
    updateVersion = "",
    onOpenSettings,
    onOpenExport,
    onOpenShortcuts,
    onCheckUpdates,
  }: {
    isScanning: boolean;
    hasReport: boolean;
    hasCustomOptions?: boolean;
    activeMonitorsCount?: number;
    hasUpdateAvailable?: boolean;
    updateVersion?: string;
    onOpenSettings: (tab?: "params" | "ports" | "watchdog" | "batch" | "wordlists" | "shortcuts" | "data") => void;
    onOpenExport: () => void;
    onOpenShortcuts?: () => void;
    onCheckUpdates?: () => void;
  } = $props();

  let currentTheme = $state<SwissTheme>("swiss-dark");
  let isMaximized = $state(false);

  async function getAppWindow() {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      return getCurrentWindow();
    } catch {
      return null;
    }
  }

  async function checkMaximized() {
    const win = await getAppWindow();
    if (win) {
      try {
        isMaximized = await win.isMaximized();
      } catch {}
    }
  }

  async function minimizeWindow() {
    const win = await getAppWindow();
    if (win) {
      try {
        await win.minimize();
      } catch {}
    }
  }

  async function toggleMaximizeWindow() {
    const win = await getAppWindow();
    if (win) {
      try {
        await win.toggleMaximize();
        await checkMaximized();
      } catch {}
    }
  }

  async function closeWindow() {
    const win = await getAppWindow();
    if (win) {
      try {
        await win.close();
      } catch {}
    }
  }

  onMount(() => {
    try {
      const saved = localStorage.getItem("vulnradar_theme") as SwissTheme;
      if (saved === "swiss-light" || saved === "swiss-dark") {
        currentTheme = saved;
      } else {
        currentTheme = (document.documentElement.getAttribute("data-theme") as SwissTheme) || "swiss-dark";
      }
      document.documentElement.classList.toggle("dark", currentTheme === "swiss-dark");
    } catch {}

    checkMaximized();
    let unlistenResize: (() => void) | null = null;
    getAppWindow().then((win) => {
      if (win) {
        win.onResized(() => {
          checkMaximized();
        }).then((unsub) => {
          unlistenResize = unsub;
        }).catch(() => {});
      }
    }).catch(() => {});

    return () => {
      if (unlistenResize) unlistenResize();
    };
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

<!-- Clean Standard Tactical Desktop Window Titlebar -->
<header
  data-tauri-drag-region
  class="h-10 pl-3 pr-0 bg-surface-container border-b border-surface-container-high sticky top-0 z-40 flex items-center justify-between titlebar-drag desktop-select-none print:hidden flex-shrink-0 transition-colors"
>
  <!-- Left: App Brand & Version -->
  <div class="flex items-center gap-2 no-drag">
    <img src="/favicon.png" alt="VulnRadar" class="h-5 w-5 object-contain" />
    <span class="font-mono text-xs font-black uppercase text-on-surface tracking-wider">VulnRadar</span>
    <button
      type="button"
      onclick={() => onCheckUpdates?.()}
      class="font-mono text-[10px] text-outline hover:text-primary transition-colors cursor-pointer px-1 py-0.5 rounded hover:bg-surface-container-high flex items-center gap-1"
      title="Check for software updates"
    >
      <span>v1.2.0</span>
      {#if hasUpdateAvailable}
        <span class="w-1.5 h-1.5 rounded-full bg-primary animate-pulse" title={`Update v${updateVersion} available`}></span>
      {/if}
    </button>
  </div>

  <!-- Center: Engine Status Pulse -->
  <div class="hidden sm:flex items-center gap-1.5 text-[11px] font-mono text-outline">
    <span class="w-1.5 h-1.5 rounded-full {isScanning ? 'bg-primary animate-ping' : 'bg-tertiary'}"></span>
    <span class="{isScanning ? 'text-primary font-bold' : 'text-on-surface-variant'}">
      {isScanning ? 'Scanning...' : 'Ready'}
    </span>
  </div>

  <!-- Right: Standard Control Actions & Windows Caption Buttons -->
  <div class="flex items-center no-drag">
    <!-- Utility actions -->
    <div class="flex items-center gap-1.5 pr-2">
      <!-- Shortcuts Help Button -->
      {#if onOpenShortcuts}
        <button
          type="button"
          onclick={onOpenShortcuts}
          class="w-7 h-7 bg-surface-container-lowest hover:bg-surface-container border border-surface-container-high text-outline hover:text-on-surface rounded flex items-center justify-center cursor-pointer transition-colors"
          title="Keyboard Shortcuts (?)"
        >
          <HelpCircle class="w-3.5 h-3.5" />
        </button>
      {/if}

      <!-- Theme Switcher Icon -->
      <button
        type="button"
        onclick={toggleTheme}
        class="w-7 h-7 bg-surface-container-lowest hover:bg-surface-container border border-surface-container-high text-outline hover:text-on-surface rounded flex items-center justify-center cursor-pointer transition-colors"
        title="Toggle Dark / Light Theme"
      >
        {#if currentTheme === "swiss-dark"}
          <Moon class="w-3.5 h-3.5 text-secondary" />
        {:else}
          <Sun class="w-3.5 h-3.5 text-amber-500" />
        {/if}
      </button>

      <!-- Export Report Button (Visible when report exists) -->
      {#if hasReport}
        <button
          type="button"
          onclick={onOpenExport}
          class="h-7 px-2.5 bg-surface-container-lowest hover:bg-surface-container border border-surface-container-high text-outline hover:text-on-surface text-[10px] font-mono font-bold uppercase rounded cursor-pointer flex items-center gap-1.5 transition-colors"
          title="Export Reports"
        >
          <FileDown class="w-3.5 h-3.5 text-secondary" />
          <span>EXPORT</span>
        </button>
      {/if}

      <!-- Update Alert Button (Visible when update is available) -->
      {#if hasUpdateAvailable && onCheckUpdates}
        <button
          type="button"
          onclick={onCheckUpdates}
          class="h-7 px-2 bg-primary/10 hover:bg-primary/20 border border-primary/40 text-primary text-[10px] font-mono font-bold uppercase rounded cursor-pointer flex items-center gap-1.5 transition-colors animate-pulse"
          title={`Update v${updateVersion || ''} Available! Click to review & install`}
        >
          <Sparkles class="w-3.5 h-3.5" />
          <span>UPDATE</span>
        </button>
      {/if}

      <!-- Settings Icon Button -->
      <button
        type="button"
        onclick={() => onOpenSettings()}
        class="w-7 h-7 bg-surface-container-lowest hover:bg-surface-container border border-surface-container-high text-outline hover:text-on-surface rounded flex items-center justify-center cursor-pointer transition-colors relative"
        title="Settings & Audit Engine (⌘,)"
      >
        <Settings class="w-3.5 h-3.5" />
        {#if hasCustomOptions || activeMonitorsCount > 0}
          <span class="absolute top-1 right-1 w-1.5 h-1.5 bg-primary rounded-full"></span>
        {/if}
      </button>
    </div>

    <!-- Windows Style Window Caption Controls -->
    <div class="flex items-center h-10 border-l border-surface-container-high">
      <button
        type="button"
        onclick={minimizeWindow}
        class="w-11 h-10 flex items-center justify-center text-outline hover:text-on-surface hover:bg-surface-container-high transition-colors cursor-pointer"
        title="Minimize"
        aria-label="Minimize Window"
      >
        <svg class="w-3 h-3" viewBox="0 0 10 10" stroke="currentColor" stroke-width="1" fill="none">
          <path d="M1 5h8" />
        </svg>
      </button>

      <button
        type="button"
        onclick={toggleMaximizeWindow}
        class="w-11 h-10 flex items-center justify-center text-outline hover:text-on-surface hover:bg-surface-container-high transition-colors cursor-pointer"
        title={isMaximized ? "Restore" : "Maximize"}
        aria-label={isMaximized ? "Restore Window" : "Maximize Window"}
      >
        {#if isMaximized}
          <svg class="w-3 h-3" viewBox="0 0 10 10" stroke="currentColor" stroke-width="1" fill="none">
            <rect x="2.5" y="1.5" width="6" height="6" />
            <path d="M1.5 3.5v5h5" />
          </svg>
        {:else}
          <svg class="w-3 h-3" viewBox="0 0 10 10" stroke="currentColor" stroke-width="1" fill="none">
            <rect x="1.5" y="1.5" width="7" height="7" />
          </svg>
        {/if}
      </button>

      <button
        type="button"
        onclick={closeWindow}
        class="w-11 h-10 flex items-center justify-center text-outline hover:text-white hover:bg-red-600 transition-colors cursor-pointer"
        title="Close"
        aria-label="Close Window"
      >
        <svg class="w-3 h-3" viewBox="0 0 10 10" stroke="currentColor" stroke-width="1.1" fill="none">
          <path d="M1.5 1.5l7 7M8.5 1.5l-7 7" />
        </svg>
      </button>
    </div>
  </div>
</header>
