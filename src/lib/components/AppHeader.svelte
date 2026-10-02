<script lang="ts">
  import { onMount } from "svelte";

  let {
    currentWorkspace = "audit",
    currentVersion = "1.4.1",
  }: {
    currentWorkspace?: string;
    currentVersion?: string;
  } = $props();

  let isMaximized = $state(false);

  const workspaceNames: Record<string, string> = {
    audit: "POSTURE AUDIT",
    ports: "PORT MATRIX",
    dns: "DNS SECURITY",
    paths: "PATH RADAR",
    batch: "FLEET RADAR",
    watchdog: "WATCHDOG MONITOR",
    history: "AUDIT LOGS",
    settings: "SETTINGS",
  };

  async function checkMaximized() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      isMaximized = await invoke<boolean>("is_window_maximized");
    } catch {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        isMaximized = await getCurrentWindow().isMaximized();
      } catch {}
    }
  }

  async function handleMinimize() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("minimize_window");
    } catch {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        await getCurrentWindow().minimize();
      } catch (err) {
        console.warn("Failed to minimize window:", err);
      }
    }
  }

  async function handleToggleMaximize() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      isMaximized = await invoke<boolean>("toggle_maximize_window");
    } catch {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const win = getCurrentWindow();
        await win.toggleMaximize();
        isMaximized = await win.isMaximized();
      } catch (err) {
        console.warn("Failed to toggle maximize window:", err);
      }
    }
  }

  async function handleHideToTray() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("hide_to_tray");
    } catch {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        await getCurrentWindow().hide();
      } catch (err) {
        console.warn("Failed to hide to tray:", err);
      }
    }
  }

  async function handleClose() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("close_window");
    } catch {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        await getCurrentWindow().close();
      } catch (err) {
        console.warn("Failed to close window:", err);
      }
    }
  }

  function handleDoubleClick(e: MouseEvent) {
    if ((e.target as HTMLElement).closest("button")) return;
    handleToggleMaximize();
  }

  onMount(() => {
    checkMaximized();
    let unlisten: (() => void) | undefined;
    import("@tauri-apps/api/window")
      .then(({ getCurrentWindow }) => {
        try {
          const win = getCurrentWindow();
          win.onResized(() => {
            checkMaximized();
          }).then((u) => {
            unlisten = u;
          });
        } catch {}
      })
      .catch(() => {});

    window.addEventListener("resize", checkMaximized);
    return () => {
      if (unlisten) unlisten();
      window.removeEventListener("resize", checkMaximized);
    };
  });
</script>

<header
  data-tauri-drag-region
  role="toolbar"
  tabindex="-1"
  aria-label="Application Title Bar"
  ondblclick={handleDoubleClick}
  class="h-9 w-full bg-surface-container border-b border-surface-container-high flex items-center justify-between select-none shrink-0 z-50 print:hidden text-on-surface"
>
  <!-- Left Side: Branding & Workspace Breadcrumb -->
  <div data-tauri-drag-region class="flex items-center gap-2.5 px-3 h-full min-w-0">
    <div class="flex items-center gap-2 pointer-events-none">
      <img src="/favicon.png" alt="VulnRadar" class="h-4 w-4 object-contain flex-shrink-0" />
      <span class="font-mono text-xs font-black uppercase tracking-wider text-on-surface">VulnRadar</span>
      <span class="font-mono text-[10px] text-outline px-1 py-0.2 bg-surface-container-high rounded hidden sm:inline">
        v{currentVersion}
      </span>
    </div>

    <!-- Active Workspace Breadcrumb -->
    <div class="h-3.5 w-px bg-surface-container-high hidden md:block"></div>
    <div class="hidden md:flex items-center gap-1.5 font-mono text-[11px] text-outline pointer-events-none">
      <span class="text-primary font-bold">//</span>
      <span class="tracking-wide uppercase text-on-surface/80">{workspaceNames[currentWorkspace] || "WORKSPACE"}</span>
    </div>
  </div>

  <!-- Center Area: Draggable Space & Window Title -->
  <div data-tauri-drag-region class="flex-1 h-full flex items-center justify-center px-4 min-w-0">
    <span data-tauri-drag-region class="font-mono text-[11px] text-outline/60 truncate pointer-events-none select-none">
      VulnRadar — Enterprise Security Posture & Vulnerability Scanner
    </span>
  </div>

  <!-- Right Side: Custom Windows Controls -->
  <div class="flex items-center h-full flex-shrink-0">
    <!-- 1. Hide to System Tray -->
    <button
      type="button"
      onclick={handleHideToTray}
      class="w-11 h-full flex items-center justify-center text-outline hover:text-on-surface hover:bg-surface-container-high/80 transition-colors cursor-pointer group"
      title="Hide to System Tray"
      aria-label="Hide to System Tray"
    >
      <svg
        class="w-3.5 h-3.5 group-hover:scale-105 transition-transform"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M12 3v12" />
        <path d="m8 11 4 4 4-4" />
        <path d="M3 15v4a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-4" />
      </svg>
    </button>

    <!-- 2. Minimize -->
    <button
      type="button"
      onclick={handleMinimize}
      class="w-11 h-full flex items-center justify-center text-outline hover:text-on-surface hover:bg-surface-container-high/80 transition-colors cursor-pointer"
      title="Minimize"
      aria-label="Minimize Window"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <line x1="4" y1="12" x2="20" y2="12" />
      </svg>
    </button>

    <!-- 3. Maximize / Restore -->
    <button
      type="button"
      onclick={handleToggleMaximize}
      class="w-11 h-full flex items-center justify-center text-outline hover:text-on-surface hover:bg-surface-container-high/80 transition-colors cursor-pointer"
      title={isMaximized ? "Restore Window" : "Maximize Window"}
      aria-label={isMaximized ? "Restore Window" : "Maximize Window"}
    >
      {#if isMaximized}
        <!-- Restore Icon (overlapping squares) -->
        <svg
          class="w-3.5 h-3.5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect x="7" y="7" width="13" height="13" rx="1.5" />
          <path d="M5 17H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h12a1 1 0 0 1 1 1v1" />
        </svg>
      {:else}
        <!-- Maximize Icon (single square) -->
        <svg
          class="w-3.5 h-3.5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect x="4" y="4" width="16" height="16" rx="2" />
        </svg>
      {/if}
    </button>

    <!-- 4. Close Window -->
    <button
      type="button"
      onclick={handleClose}
      class="w-11 h-full flex items-center justify-center text-outline hover:text-white hover:bg-[#e81123] transition-colors cursor-pointer"
      title="Close"
      aria-label="Close Window"
    >
      <svg
        class="w-3.5 h-3.5"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </button>
  </div>
</header>
