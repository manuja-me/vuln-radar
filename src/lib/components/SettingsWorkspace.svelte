<script lang="ts">
  import type {
    MonitorTarget,
    ScanOptions,
    ScanReport,
    WordlistConfig,
  } from "$lib/types";
  import WordlistSelector from "./WordlistSelector.svelte";
  import {
    Sliders,
    Server,
    Activity,
    Layers,
    ListFilter,
    Keyboard,
    Database,
    Globe,
    Plus,
    Trash2,
    Play,
    Pause,
    RotateCw,
    BellRing,
    Loader2,
    CheckCircle2,
    AlertOctagon,
    AlertTriangle,
    ArrowRight,
    Check,
    Info,
    Terminal,
    ShieldCheck,
    Sparkles,
    Download,
    ExternalLink,
    ArrowLeft,
    X,
  } from "lucide-svelte";

  let {
    activeTab = "params",
    options,
    monitors = [],
    historyCount = 0,
    hasUpdateAvailable = false,
    updateVersion = "",
    currentVersion = "1.4.0",
    onCheckUpdates,
    onApplyOptions,
    onAddMonitor,
    onDeleteMonitor,
    onToggleMonitor,
    onScanTarget,
    onClearHistory,
    onOpenHistory,
    onClose,
  }: {
    activeTab?: "params" | "ports" | "watchdog" | "wordlists" | "shortcuts" | "data" | "updates";
    options: ScanOptions;
    monitors: MonitorTarget[];
    historyCount?: number;
    hasUpdateAvailable?: boolean;
    updateVersion?: string;
    currentVersion?: string;
    onCheckUpdates?: () => void;
    onApplyOptions?: (newOptions: ScanOptions) => void;
    onAddMonitor?: (url: string, intervalHours: number) => Promise<void>;
    onDeleteMonitor?: (id: string) => Promise<void>;
    onToggleMonitor?: (id: string) => Promise<void>;
    onScanTarget?: (url: string) => void;
    onClearHistory?: () => Promise<void>;
    onOpenHistory?: () => void;
    onClose?: () => void;
  } = $props();

  let currentTab = $state<"params" | "ports" | "watchdog" | "wordlists" | "shortcuts" | "data" | "updates">("params");

  // Sync activeTab when changed from parent
  $effect(() => {
    if (activeTab) {
      currentTab = activeTab;
    }
  });

  // --- Scan Parameters & Headers Local State ---
  let headerRows = $state<{ key: string; value: string }[]>([{ key: "", value: "" }]);
  let userAgentInput = $state("");
  let timeoutSeconds = $state(15);
  let includeSubdomains = $state<boolean>(true);

  // --- Port Scanner Local State ---
  let enablePortScan = $state<boolean>(true);
  let portScanProfile = $state("top20");
  let customPortsInput = $state("21, 22, 80, 443, 3000-3005, 8080, 8443");
  let portTimeoutMs = $state(600);

  // --- Wordlist Engine Local State ---
  let wordlistConfig = $state<WordlistConfig>({
    selectedIds: ["common_paths", "sensitive_files", "api_documentation"],
    customPaths: [],
    activePreset: "balanced",
  });

  // Initialize and synchronize options state
  let initialized = false;
  $effect(() => {
    if (options && !initialized) {
      headerRows =
        options.custom_headers && options.custom_headers.length > 0
          ? options.custom_headers.map(([key, value]) => ({ key, value }))
          : [{ key: "", value: "" }];
      userAgentInput = options?.user_agent || "";
      timeoutSeconds = options?.timeout_seconds || 15;
      includeSubdomains = options?.include_subdomains ?? true;

      enablePortScan = options?.enable_port_scan ?? true;
      portScanProfile = options?.port_scan_profile || "top20";
      customPortsInput = options?.custom_ports || "21, 22, 80, 443, 3000-3005, 8080, 8443";
      portTimeoutMs = options?.port_timeout_ms || 600;

      wordlistConfig = options?.wordlist_config
        ? {
            selectedIds: [...options.wordlist_config.selectedIds],
            customPaths: [...options.wordlist_config.customPaths],
            activePreset: options.wordlist_config.activePreset,
          }
        : {
            selectedIds: ["common_paths", "sensitive_files", "api_documentation"],
            customPaths: [],
            activePreset: "balanced",
          };
      initialized = true;
    }
  });

  function addHeaderRow() {
    headerRows = [...headerRows, { key: "", value: "" }];
  }

  function removeHeaderRow(index: number) {
    headerRows = headerRows.filter((_, i) => i !== index);
    if (headerRows.length === 0) {
      headerRows = [{ key: "", value: "" }];
    }
  }

  function handleSaveParameters() {
    const validHeaders = headerRows
      .filter((r) => r.key.trim().length > 0)
      .map((r) => [r.key.trim(), r.value.trim()] as [string, string]);

    const newOptions: ScanOptions = {
      custom_headers: validHeaders.length > 0 ? validHeaders : undefined,
      user_agent: userAgentInput.trim() ? userAgentInput.trim() : undefined,
      timeout_seconds: Number(timeoutSeconds) || 15,
      include_subdomains: includeSubdomains,
      enable_port_scan: enablePortScan,
      port_scan_profile: portScanProfile,
      custom_ports: customPortsInput.trim() ? customPortsInput.trim() : undefined,
      port_timeout_ms: Number(portTimeoutMs) || 800,
      wordlist_config: wordlistConfig,
    };

    if (onApplyOptions) {
      onApplyOptions(newOptions);
    }
  }

  // --- Watchdog Local State ---
  let newWatchdogUrl = $state("");
  let selectedInterval = $state(24);
  let isAddingWatchdog = $state(false);

  async function handleAddWatchdog() {
    const url = newWatchdogUrl.trim();
    if (!url || !onAddMonitor) return;
    isAddingWatchdog = true;
    try {
      await onAddMonitor(url, selectedInterval);
      newWatchdogUrl = "";
    } finally {
      isAddingWatchdog = false;
    }
  }

  function formatWatchdogDate(iso?: string | null) {
    if (!iso) return "Never";
    try {
      const d = new Date(iso);
      return d.toLocaleDateString() + " " + d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    } catch {
      return iso;
    }
  }

  function getScoreBadge(score?: number | null) {
    if (score === undefined || score === null) return "bg-[var(--color-canvas)] text-[var(--color-text-muted)] border border-[var(--color-hairline)] rounded-none";
    if (score >= 85) return "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30 rounded-none";
    if (score >= 70) return "bg-blue-500/10 text-blue-600 dark:text-blue-400 border border-blue-500/30 rounded-none";
    if (score >= 50) return "bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/30 rounded-none";
    return "bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/30 rounded-none";
  }

  // --- Keyboard Shortcuts Reference ---
  const shortcutsList = [
    { key: "⌘ / Ctrl + ,", description: "Open Settings Workspace" },
    { key: "⌘ / Ctrl + K", description: "Focus target domain input" },
    { key: "⌘ / Ctrl + H", description: "Open Scan History Archive" },
    { key: "⌘ / Ctrl + O", description: "Open Scan & Recon Parameters" },
    { key: "⌘ / Ctrl + M", description: "Open Continuous Watchdog" },
    { key: "⌘ / Ctrl + B", description: "Open Fleet Batch Scanner" },
    { key: "⌘ / Ctrl + E", description: "Export Report (Markdown, JSON)" },
    { key: "⌘ / Ctrl + P", description: "Print Executive PDF Report" },
    { key: "Esc", description: "Close active modal or subdialog" },
    { key: "?", description: "Open keyboard shortcuts guide" },
  ];

  // Derived indicators
  const hasCustomParams = $derived(
    !!(
      (headerRows.filter((r) => r.key.trim().length > 0).length > 0) ||
      userAgentInput.trim() ||
      timeoutSeconds !== 15 ||
      includeSubdomains === false
    )
  );

  const activeMonitorsCount = $derived(
    monitors.filter((m) => m.is_active).length
  );
</script>

<div class="flex flex-col w-full max-w-7xl mx-auto pb-10 animate-fade-in flex-1">
  <!-- Top Workspace Header -->
  <section class="bg-surface-container rounded-t-lg p-4 sm:p-5 flex items-center justify-between border border-surface-container-high border-b-0 shadow-xs">
    <div class="flex items-center gap-3">
      <div class="w-9 h-9 rounded bg-surface-container-lowest border border-surface-container-high flex items-center justify-center text-primary shrink-0">
        <Sliders class="w-4 h-4" />
      </div>
      <div>
        <div class="flex items-center gap-2">
          <h2 class="text-sm font-black text-on-surface uppercase tracking-tight font-mono">
            Settings & Engine Configuration
          </h2>
          <span class="px-1.5 py-0.2 text-[10px] bg-surface-container-lowest text-outline rounded font-mono border border-surface-container-high uppercase font-bold">
            VULNRADAR
          </span>
        </div>
        <p class="text-xs text-outline font-mono mt-0.5">
          Manage audit parameters, active watchdog, scanner profiles, wordlists, and preferences
        </p>
      </div>
    </div>

    {#if onClose}
      <button
        type="button"
        onclick={onClose}
        class="h-8 px-3 rounded bg-surface-container-lowest hover:bg-surface-container-high border border-surface-container-high text-xs font-mono font-bold uppercase text-outline hover:text-on-surface transition-colors cursor-pointer flex items-center gap-1.5 shrink-0"
        title="Return to primary audit workspace"
      >
        <ArrowLeft class="w-3.5 h-3.5" />
        <span class="hidden sm:inline">Back to Audit</span>
      </button>
    {/if}
  </section>

  <!-- Main Dual-Pane Container -->
  <div class="flex-1 flex flex-col md:flex-row min-h-[640px] bg-surface-container-low border border-surface-container-high rounded-b-lg overflow-hidden shadow-sm">
    <!-- Left Navigation Rail -->
    <nav class="w-full md:w-56 bg-surface-container border-b md:border-b-0 md:border-r border-surface-container-high p-3 flex md:flex-col gap-1 overflow-x-auto md:overflow-y-auto shrink-0 font-mono">
      <button
        type="button"
        onclick={() => (currentTab = "params")}
        class="px-3 py-2 rounded text-xs font-bold uppercase flex items-center justify-between transition-colors cursor-pointer {currentTab === 'params' ? 'bg-on-surface text-surface-container-lowest font-black' : 'text-outline hover:text-on-surface hover:bg-surface-container-highest'}"
      >
        <div class="flex items-center gap-2.5">
          <Sliders class="w-3.5 h-3.5" />
          <span>01/PARAMETERS</span>
        </div>
        {#if hasCustomParams}
          <span class="w-1.5 h-1.5 rounded-full bg-primary" title="Custom parameters active"></span>
        {/if}
      </button>

      <button
        type="button"
        onclick={() => (currentTab = "ports")}
        class="px-3 py-2 rounded text-xs font-bold uppercase flex items-center justify-between transition-colors cursor-pointer {currentTab === 'ports' ? 'bg-on-surface text-surface-container-lowest font-black' : 'text-outline hover:text-on-surface hover:bg-surface-container-highest'}"
      >
        <div class="flex items-center gap-2.5">
          <Server class="w-3.5 h-3.5" />
          <span>02/PORTS</span>
        </div>
        {#if enablePortScan}
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-500" title="Port scanner enabled"></span>
        {/if}
      </button>

      <button
        type="button"
        onclick={() => (currentTab = "watchdog")}
        class="px-3 py-2 rounded text-xs font-bold uppercase flex items-center justify-between transition-colors cursor-pointer {currentTab === 'watchdog' ? 'bg-on-surface text-surface-container-lowest font-black' : 'text-outline hover:text-on-surface hover:bg-surface-container-highest'}"
      >
        <div class="flex items-center gap-2.5">
          <Activity class="w-3.5 h-3.5" />
          <span>03/WATCHDOG</span>
        </div>
        {#if activeMonitorsCount > 0}
          <span class="px-1.5 py-0.2 text-[10px] font-mono rounded bg-surface-container-lowest text-on-surface border border-surface-container-high">
            {activeMonitorsCount}
          </span>
        {/if}
      </button>

      <button
        type="button"
        onclick={() => (currentTab = "wordlists")}
        class="px-3 py-2 rounded text-xs font-bold uppercase flex items-center justify-between transition-colors cursor-pointer {currentTab === 'wordlists' ? 'bg-on-surface text-surface-container-lowest font-black' : 'text-outline hover:text-on-surface hover:bg-surface-container-highest'}"
      >
        <div class="flex items-center gap-2.5">
          <ListFilter class="w-3.5 h-3.5" />
          <span>04/WORDLISTS</span>
        </div>
        {#if wordlistConfig.selectedIds.length > 0}
          <span class="px-1.5 py-0.2 text-[10px] font-mono rounded bg-surface-container-lowest text-on-surface border border-surface-container-high">
            {wordlistConfig.selectedIds.length}
          </span>
        {/if}
      </button>

      <button
        type="button"
        onclick={() => (currentTab = "shortcuts")}
        class="px-3 py-2 rounded text-xs font-bold uppercase flex items-center justify-between transition-colors cursor-pointer {currentTab === 'shortcuts' ? 'bg-on-surface text-surface-container-lowest font-black' : 'text-outline hover:text-on-surface hover:bg-surface-container-highest'}"
      >
        <div class="flex items-center gap-2.5">
          <Keyboard class="w-3.5 h-3.5" />
          <span>05/SHORTCUTS</span>
        </div>
        <kbd class="text-[10px] font-mono opacity-70">⌘K</kbd>
      </button>

      <div class="hidden md:block my-2 border-t border-surface-container-high"></div>

      <button
        type="button"
        onclick={() => (currentTab = "data")}
        class="px-3 py-2 rounded text-xs font-bold uppercase flex items-center justify-between transition-colors cursor-pointer {currentTab === 'data' ? 'bg-on-surface text-surface-container-lowest font-black' : 'text-outline hover:text-on-surface hover:bg-surface-container-highest'}"
      >
        <div class="flex items-center gap-2.5">
          <Database class="w-3.5 h-3.5" />
          <span>06/STORAGE</span>
        </div>
        {#if historyCount > 0}
          <span class="text-[10px] font-mono text-outline">{historyCount}</span>
        {/if}
      </button>

      <button
        type="button"
        onclick={() => (currentTab = "updates")}
        class="px-3 py-2 rounded text-xs font-bold uppercase flex items-center justify-between transition-colors cursor-pointer {currentTab === 'updates' ? 'bg-on-surface text-surface-container-lowest font-black' : 'text-outline hover:text-on-surface hover:bg-surface-container-highest'}"
      >
        <div class="flex items-center gap-2.5">
          <Sparkles class="w-3.5 h-3.5 text-primary" />
          <span>07/UPDATES</span>
        </div>
        {#if hasUpdateAvailable}
          <span class="w-2 h-2 rounded-full bg-primary animate-pulse" title="Update available"></span>
        {/if}
      </button>
    </nav>

    <!-- Right Expansive Content Workspace -->
    <div class="flex-1 min-w-0 flex flex-col bg-surface-container-lowest overflow-hidden">
      <div class="flex-1 overflow-y-auto p-5 sm:p-6 md:p-8 space-y-6">

        <!-- 01/PARAMETERS TAB -->
        {#if currentTab === "params"}
          <div class="space-y-6 animate-fade-in">
            <div>
              <h3 class="text-sm font-bold text-on-surface font-mono uppercase tracking-tight">
                Scan Reconnaissance & Request Parameters
              </h3>
              <p class="text-xs text-outline mt-0.5 font-mono">
                Configure HTTP injection headers, client identity, timeout limits, and domain scope.
              </p>
            </div>

            <!-- Custom Headers Section -->
            <div class="space-y-3">
              <div class="flex items-center justify-between">
                <span class="text-xs font-bold uppercase tracking-wider text-outline font-mono">
                  Custom Request Headers & Auth Tokens
                </span>
                <button
                  type="button"
                  onclick={addHeaderRow}
                  class="px-2.5 py-1 bg-surface-container hover:bg-surface-container-highest text-on-surface rounded text-xs font-mono font-bold uppercase flex items-center gap-1.5 transition-colors cursor-pointer border border-surface-container-high"
                >
                  <Plus class="w-3.5 h-3.5" />
                  <span>Add Header</span>
                </button>
              </div>

              <div class="space-y-2">
                {#each headerRows as row, i}
                  <div class="flex items-center gap-2.5">
                    <input
                      type="text"
                      bind:value={row.key}
                      placeholder="Header Name (e.g. Authorization, X-API-Key)"
                      class="flex-1 h-9 px-3 text-xs bg-surface-container rounded border border-surface-container-high font-mono text-on-surface placeholder:text-outline focus:border-outline outline-none"
                    />
                    <input
                      type="text"
                      bind:value={row.value}
                      placeholder="Value (e.g. Bearer eyJhbGciOi...)"
                      class="flex-1 h-9 px-3 text-xs bg-surface-container rounded border border-surface-container-high font-mono text-on-surface placeholder:text-outline focus:border-outline outline-none"
                    />
                    <button
                      type="button"
                      onclick={() => removeHeaderRow(i)}
                      class="w-9 h-9 flex items-center justify-center text-outline hover:text-error hover:bg-surface-container-highest rounded transition-colors cursor-pointer"
                      title="Remove header"
                    >
                      <Trash2 class="w-3.5 h-3.5" />
                    </button>
                  </div>
                {/each}
              </div>
            </div>

            <!-- User-Agent Input -->
            <div class="space-y-1.5">
              <label for="settings-user-agent" class="text-xs font-bold uppercase tracking-wider text-outline font-mono">
                Custom User-Agent Identifier
              </label>
              <input
                id="settings-user-agent"
                type="text"
                bind:value={userAgentInput}
                placeholder="Default: VulnRadar/1.x (+https://github.com/manuja-me/vuln-radar)"
                class="w-full h-9 px-3 text-xs bg-surface-container rounded border border-surface-container-high font-mono text-on-surface placeholder:text-outline focus:border-outline outline-none"
              />
            </div>

            <!-- Timeout & Scope Controls -->
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div class="space-y-1.5">
                <label for="settings-timeout" class="text-xs font-bold uppercase tracking-wider text-outline font-mono">
                  HTTP Request Timeout (Seconds)
                </label>
                <input
                  id="settings-timeout"
                  type="number"
                  min="3"
                  max="60"
                  bind:value={timeoutSeconds}
                  class="w-full h-9 px-3 text-xs bg-surface-container rounded border border-surface-container-high font-mono text-on-surface focus:border-outline outline-none"
                />
              </div>

              <div class="space-y-1.5 flex flex-col justify-end">
                <label class="flex items-center gap-2.5 p-2 bg-surface-container rounded border border-surface-container-high cursor-pointer hover:bg-surface-container-highest transition-colors">
                  <input
                    type="checkbox"
                    bind:checked={includeSubdomains}
                    class="rounded text-primary focus:ring-0 cursor-pointer"
                  />
                  <span class="text-xs font-mono font-bold uppercase text-on-surface">
                    Audit Related Subdomains
                  </span>
                </label>
              </div>
            </div>
          </div>

        <!-- 02/PORTS TAB -->
        {:else if currentTab === "ports"}
          <div class="space-y-6 animate-fade-in">
            <div>
              <h3 class="text-sm font-bold text-on-surface font-mono uppercase tracking-tight">
                Port Scanner & Network Reconnaissance
              </h3>
              <p class="text-xs text-outline mt-0.5 font-mono">
                Configure concurrent TCP connection probes, port lists, and socket timeouts.
              </p>
            </div>

            <div class="space-y-4">
              <label class="flex items-center gap-2.5 p-3 bg-surface-container rounded border border-surface-container-high cursor-pointer hover:bg-surface-container-highest transition-colors">
                <input
                  type="checkbox"
                  bind:checked={enablePortScan}
                  class="rounded text-primary focus:ring-0 cursor-pointer"
                />
                <div>
                  <span class="text-xs font-mono font-bold uppercase text-on-surface">
                    Enable Background TCP Port Scanner
                  </span>
                  <p class="text-[11px] font-mono text-outline mt-0.5">
                    Scans key server ports concurrently during primary target audits
                  </p>
                </div>
              </label>

              {#if enablePortScan}
                <div class="space-y-4 p-4 bg-surface-container rounded border border-surface-container-high animate-fade-in">
                  <div class="space-y-1.5">
                    <span class="text-xs font-bold uppercase tracking-wider text-outline font-mono">
                      Port Profile Preset
                    </span>
                    <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
                      {#each [
                        { id: "top20", label: "Top 20 Critical" },
                        { id: "top100", label: "Top 100 Common" },
                        { id: "databases", label: "Databases & Cache" },
                        { id: "custom", label: "Custom Ports" }
                      ] as profile}
                        <button
                          type="button"
                          onclick={() => (portScanProfile = profile.id)}
                          class="p-2.5 rounded text-xs font-mono font-bold uppercase text-center border transition-colors cursor-pointer {portScanProfile === profile.id ? 'bg-primary text-on-primary border-primary shadow-xs' : 'bg-surface-container-lowest text-outline hover:text-on-surface border-surface-container-high'}"
                        >
                          {profile.label}
                        </button>
                      {/each}
                    </div>
                  </div>

                  {#if portScanProfile === "custom"}
                    <div class="space-y-1.5 animate-fade-in">
                      <label for="settings-custom-ports" class="text-xs font-bold uppercase tracking-wider text-outline font-mono">
                        Custom Port Specification (Ranges & Lists)
                      </label>
                      <input
                        id="settings-custom-ports"
                        type="text"
                        bind:value={customPortsInput}
                        placeholder="e.g. 21, 22, 80, 443, 3000-3005, 8080"
                        class="w-full h-9 px-3 text-xs bg-surface-container-lowest rounded border border-surface-container-high font-mono text-on-surface focus:border-outline outline-none"
                      />
                    </div>
                  {/if}

                  <div class="space-y-1.5">
                    <label for="settings-port-timeout" class="text-xs font-bold uppercase tracking-wider text-outline font-mono">
                      Connection Timeout per Port ({portTimeoutMs}ms)
                    </label>
                    <input
                      id="settings-port-timeout"
                      type="range"
                      min="200"
                      max="3000"
                      step="100"
                      bind:value={portTimeoutMs}
                      class="w-full accent-primary cursor-pointer"
                    />
                    <div class="flex justify-between text-[10px] font-mono text-outline">
                      <span>200ms (Fast / Local)</span>
                      <span>800ms (Standard)</span>
                      <span>3000ms (High Latency)</span>
                    </div>
                  </div>
                </div>
              {/if}
            </div>
          </div>

        <!-- 03/WATCHDOG TAB -->
        {:else if currentTab === "watchdog"}
          <div class="space-y-6 animate-fade-in">
            <div>
              <h3 class="text-sm font-bold text-on-surface font-mono uppercase tracking-tight">
                Automated Background Watchdog Daemon
              </h3>
              <p class="text-xs text-outline mt-0.5 font-mono">
                Schedules periodic security health checks and triggers desktop alerts if scores decline.
              </p>
            </div>

            <!-- Add Monitor Target Form -->
            <div class="p-4 bg-surface-container rounded border border-surface-container-high space-y-3">
              <span class="text-xs font-bold uppercase tracking-wider text-outline font-mono">
                Schedule New Monitored Host
              </span>
              <div class="flex flex-col sm:flex-row gap-2.5">
                <input
                  type="text"
                  bind:value={newWatchdogUrl}
                  placeholder="https://app.example.com"
                  class="flex-1 h-9 px-3 text-xs bg-surface-container-lowest rounded border border-surface-container-high font-mono text-on-surface focus:border-outline outline-none"
                />
                <select
                  bind:value={selectedInterval}
                  class="h-9 px-3 text-xs bg-surface-container-lowest rounded border border-surface-container-high font-mono text-on-surface focus:border-outline outline-none cursor-pointer"
                >
                  <option value={1}>Every 1 Hour</option>
                  <option value={6}>Every 6 Hours</option>
                  <option value={12}>Every 12 Hours</option>
                  <option value={24}>Every 24 Hours</option>
                </select>
                <button
                  type="button"
                  disabled={isAddingWatchdog || !newWatchdogUrl.trim()}
                  onclick={handleAddWatchdog}
                  class="h-9 px-4 bg-primary text-on-primary font-bold uppercase font-mono text-xs rounded hover:bg-primary/90 transition-opacity cursor-pointer disabled:opacity-50 shrink-0"
                >
                  {isAddingWatchdog ? "Adding..." : "+ Add Target"}
                </button>
              </div>
            </div>

            <!-- Monitored Targets List -->
            <div class="space-y-2">
              {#if monitors.length === 0}
                <div class="py-12 text-center bg-surface-container rounded border border-surface-container-high font-mono text-xs text-outline space-y-2">
                  <Activity class="w-8 h-8 text-outline mx-auto opacity-50" />
                  <p>No active watchdog domains scheduled.</p>
                </div>
              {:else}
                {#each monitors as m}
                  <div class="p-3.5 bg-surface-container rounded border border-surface-container-high flex items-center justify-between gap-4 font-mono">
                    <div class="min-w-0 space-y-1">
                      <div class="flex items-center gap-2">
                        <span class="text-xs font-bold text-on-surface truncate">{m.target_url}</span>
                        <span class="px-2 py-0.5 text-[10px] rounded {getScoreBadge(m.last_score)} font-bold">
                          {m.last_score !== null && m.last_score !== undefined ? `${m.last_score}/100` : "PENDING"}
                        </span>
                      </div>
                      <div class="text-[11px] text-outline">
                        Interval: {m.interval_hours}h • Next check: {formatWatchdogDate(m.next_scan_at)}
                      </div>
                    </div>

                    <div class="flex items-center gap-2 shrink-0">
                      {#if onScanTarget}
                        <button
                          type="button"
                          onclick={() => onScanTarget(m.target_url)}
                          class="px-2.5 py-1 bg-surface-container-lowest hover:bg-surface-container-highest text-on-surface rounded text-xs font-bold uppercase border border-surface-container-high transition-colors cursor-pointer"
                        >
                          Audit Now
                        </button>
                      {/if}
                      {#if onToggleMonitor}
                        <button
                          type="button"
                          onclick={() => onToggleMonitor(m.id)}
                          class="p-1.5 rounded text-outline hover:text-on-surface hover:bg-surface-container-highest transition-colors cursor-pointer"
                          title={m.is_active ? "Pause watchdog" : "Resume watchdog"}
                        >
                          {#if m.is_active}
                            <Pause class="w-3.5 h-3.5" />
                          {:else}
                            <Play class="w-3.5 h-3.5" />
                          {/if}
                        </button>
                      {/if}
                      {#if onDeleteMonitor}
                        <button
                          type="button"
                          onclick={() => onDeleteMonitor(m.id)}
                          class="p-1.5 rounded text-outline hover:text-error hover:bg-surface-container-highest transition-colors cursor-pointer"
                          title="Remove watchdog"
                        >
                          <Trash2 class="w-3.5 h-3.5" />
                        </button>
                      {/if}
                    </div>
                  </div>
                {/each}
              {/if}
            </div>
          </div>

        <!-- 04/WORDLISTS TAB -->
        {:else if currentTab === "wordlists"}
          <div class="space-y-6 animate-fade-in w-full">
            <div>
              <h3 class="text-sm font-bold text-on-surface font-mono uppercase tracking-tight">
                Wordlist & Path Discovery Engine
              </h3>
              <p class="text-xs text-outline mt-0.5 font-mono">
                Select target path catalogs, sensitive file signatures, or create dynamic dictionary patterns.
              </p>
            </div>

            <!-- Full Width Wordlist Selector -->
            <WordlistSelector
              bind:config={wordlistConfig}
              onChange={(newCfg) => {
                wordlistConfig = newCfg;
              }}
            />
          </div>

        <!-- 06/SHORTCUTS TAB -->
        {:else if currentTab === "shortcuts"}
          <div class="space-y-6 animate-fade-in">
            <div>
              <h3 class="text-sm font-bold text-on-surface font-mono uppercase tracking-tight">
                Keyboard Shortcuts & Navigation
              </h3>
              <p class="text-xs text-outline mt-0.5 font-mono">
                Quick desktop hotkeys for rapid terminal operations and workspace switching.
              </p>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {#each shortcutsList as shortcut}
                <div class="p-3 bg-surface-container rounded border border-surface-container-high flex items-center justify-between gap-3 font-mono">
                  <span class="text-xs text-on-surface">{shortcut.description}</span>
                  <kbd class="px-2 py-1 bg-surface-container-lowest text-primary rounded border border-surface-container-high text-xs font-bold shrink-0">
                    {shortcut.key}
                  </kbd>
                </div>
              {/each}
            </div>
          </div>

        <!-- 07/STORAGE TAB -->
        {:else if currentTab === "data"}
          <div class="space-y-6 animate-fade-in">
            <div>
              <h3 class="text-sm font-bold text-on-surface font-mono uppercase tracking-tight">
                Storage, Diagnostics & About
              </h3>
              <p class="text-xs text-outline mt-0.5 font-mono">
                Local SQLite database state and system build telemetry.
              </p>
            </div>

            <!-- Database Metrics Cards -->
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div class="p-4 bg-surface-container rounded border border-surface-container-high space-y-1 font-mono">
                <span class="text-xs font-bold text-outline uppercase tracking-wider">Persisted Scans</span>
                <div class="text-xl font-bold text-on-surface flex items-center justify-between">
                  <span>{historyCount} Snapshots</span>
                  {#if onOpenHistory}
                    <button
                      type="button"
                      onclick={onOpenHistory}
                      class="text-xs text-primary hover:underline uppercase cursor-pointer"
                    >
                      View Archive
                    </button>
                  {/if}
                </div>
              </div>

              <div class="p-4 bg-surface-container rounded border border-surface-container-high space-y-1 font-mono">
                <span class="text-xs font-bold text-outline uppercase tracking-wider">Watchdog Domains</span>
                <div class="text-xl font-bold text-on-surface">
                  {monitors.length} Configured
                </div>
              </div>
            </div>

            <!-- Clear History Action -->
            {#if onClearHistory && historyCount > 0}
              <div class="p-4 bg-error/10 border border-error/30 rounded flex items-center justify-between gap-4 font-mono">
                <div>
                  <div class="text-xs font-bold text-error uppercase">Clear Local Scan Archive</div>
                  <p class="text-[11px] text-outline mt-0.5">
                    Permanently delete all historical audit reports stored in the local SQLite database.
                  </p>
                </div>
                <button
                  type="button"
                  onclick={onClearHistory}
                  class="px-3 py-1.5 bg-error text-white font-bold rounded text-xs uppercase cursor-pointer hover:opacity-90 shrink-0"
                >
                  Clear History
                </button>
              </div>
            {/if}

            <!-- About Card -->
            <div class="p-5 bg-surface-container rounded border border-surface-container-high space-y-3 font-mono">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 rounded bg-surface-container-lowest border border-surface-container-high flex items-center justify-center text-primary">
                  <ShieldCheck class="w-4 h-4" />
                </div>
                <div>
                  <div class="text-xs font-bold text-on-surface uppercase">VulnRadar Desktop v{currentVersion}</div>
                  <div class="text-[11px] text-outline uppercase">Tauri v2 • Svelte 5 • Rust 2021 Engine</div>
                </div>
              </div>
              <p class="text-xs text-outline leading-relaxed">
                Lightweight passive reconnaissance, HTTP security posture verification, TLS cipher audit, DNS SPF/DMARC alignment, and asynchronous TCP port scanner.
              </p>
            </div>
          </div>

        <!-- 08/UPDATES TAB -->
        {:else if currentTab === "updates"}
          <div class="space-y-6 animate-fade-in">
            <div>
              <h3 class="text-sm font-bold text-on-surface font-mono uppercase tracking-tight">
                Software Updates & Channel
              </h3>
              <p class="text-xs text-outline mt-0.5 font-mono">
                Manage application auto-updates, verify cryptographic signatures, and check for new releases.
              </p>
            </div>

            <!-- Update Status Card -->
            <div class="p-5 bg-surface-container rounded border border-surface-container-high space-y-4 font-mono">
              <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
                <div class="flex items-center gap-3">
                  <div class="w-10 h-10 rounded bg-primary/10 border border-primary/20 flex items-center justify-center text-primary shrink-0">
                    <Sparkles class="w-5 h-5" />
                  </div>
                  <div>
                    <div class="text-xs font-bold text-on-surface uppercase">
                      VulnRadar Desktop v{currentVersion}
                    </div>
                    <div class="text-[11px] text-outline mt-0.5">
                      Channel: <span class="text-primary font-bold">Stable (GitHub Releases)</span>
                    </div>
                  </div>
                </div>

                {#if onCheckUpdates}
                  <button
                    type="button"
                    onclick={onCheckUpdates}
                    class="px-4 py-2 bg-primary hover:bg-primary/90 text-on-primary font-bold text-xs uppercase rounded flex items-center gap-1.5 transition-colors cursor-pointer shrink-0 shadow-xs"
                  >
                    <RotateCw class="w-3.5 h-3.5" />
                    <span>Check for Updates</span>
                  </button>
                {/if}
              </div>

              {#if hasUpdateAvailable}
                <div class="p-3 bg-primary/10 border border-primary/30 rounded flex items-center justify-between">
                  <span class="text-xs font-mono font-bold text-primary">
                    Update v{updateVersion} is ready to download!
                  </span>
                  {#if onCheckUpdates}
                    <button
                      type="button"
                      onclick={onCheckUpdates}
                      class="px-3 py-1 bg-primary text-on-primary font-bold text-xs uppercase rounded hover:bg-primary/90 transition-colors cursor-pointer"
                    >
                      Install Update
                    </button>
                  {/if}
                </div>
              {/if}
            </div>

            <!-- Cryptographic Verification Notice -->
            <div class="p-4 bg-surface-container rounded border border-surface-container-high space-y-2 font-mono">
              <div class="flex items-center gap-2 text-xs font-bold text-on-surface uppercase">
                <ShieldCheck class="w-4 h-4 text-emerald-400" />
                <span>Cryptographic Signature Verification</span>
              </div>
              <p class="text-[11px] text-outline leading-relaxed">
                All updates are digitally signed with an Ed25519 Minisign private key during automated GitHub Actions releases. VulnRadar verifies the binary integrity and author signature before applying updates.
              </p>
            </div>

            <!-- Release Channel Link -->
            <div class="flex items-center justify-between p-3.5 bg-surface-container rounded border border-surface-container-high text-xs font-mono">
              <span class="text-outline uppercase">Official Release Repository</span>
              <a
                href="https://github.com/manuja-me/vuln-radar/releases"
                target="_blank"
                rel="noreferrer"
                class="text-primary hover:underline flex items-center gap-1 font-bold"
              >
                <span>github.com/manuja-me/vuln-radar/releases</span>
                <ExternalLink class="w-3.5 h-3.5" />
              </a>
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer for parameter save/apply -->
      {#if currentTab === "params" || currentTab === "ports" || currentTab === "wordlists"}
        <div class="px-6 py-3.5 border-t border-surface-container-high flex items-center justify-between bg-surface-container text-xs font-mono shrink-0">
          <span class="text-outline uppercase text-[11px]">
            Changes apply immediately to subsequent security audits.
          </span>
          <div class="flex items-center gap-2">
            <button
              type="button"
              onclick={handleSaveParameters}
              class="px-4 py-2 bg-primary hover:bg-primary/90 text-on-primary font-bold uppercase rounded text-xs transition-colors cursor-pointer shadow-sm"
            >
              Apply Parameters
            </button>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>
