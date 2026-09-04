<script lang="ts">
  import type {
    BatchScanItem,
    MonitorTarget,
    ScanOptions,
    ScanReport,
  } from "$lib/types";
  import {
    X,
    Sliders,
    Server,
    Activity,
    Layers,
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
  } from "lucide-svelte";

  let {
    isOpen = false,
    activeTab = "params",
    options,
    monitors = [],
    historyCount = 0,
    onApplyOptions,
    onAddMonitor,
    onDeleteMonitor,
    onToggleMonitor,
    onScanTarget,
    onClearHistory,
    onOpenHistory,
    onSelectBatchReport,
    onClose,
  }: {
    isOpen: boolean;
    activeTab?: "params" | "ports" | "watchdog" | "batch" | "shortcuts" | "data";
    options: ScanOptions;
    monitors: MonitorTarget[];
    historyCount?: number;
    onApplyOptions?: (newOptions: ScanOptions) => void;
    onAddMonitor?: (url: string, intervalHours: number) => Promise<void>;
    onDeleteMonitor?: (id: string) => Promise<void>;
    onToggleMonitor?: (id: string) => Promise<void>;
    onScanTarget?: (url: string) => void;
    onClearHistory?: () => Promise<void>;
    onOpenHistory?: () => void;
    onSelectBatchReport?: (report: ScanReport) => void;
    onClose: () => void;
  } = $props();

  let currentTab = $state<"params" | "ports" | "watchdog" | "batch" | "shortcuts" | "data">("params");

  // Sync activeTab when modal opens
  $effect(() => {
    if (isOpen && activeTab) {
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

  // Synchronize internal options state when opening modal
  let previousIsOpen = false;
  $effect(() => {
    if (isOpen && !previousIsOpen) {
      headerRows =
        options && options.custom_headers && options.custom_headers.length > 0
          ? options.custom_headers.map(([key, value]) => ({ key, value }))
          : [{ key: "", value: "" }];
      userAgentInput = options?.user_agent || "";
      timeoutSeconds = options?.timeout_seconds || 15;
      includeSubdomains = options?.include_subdomains ?? true;

      enablePortScan = options?.enable_port_scan ?? true;
      portScanProfile = options?.port_scan_profile || "top20";
      customPortsInput = options?.custom_ports || "21, 22, 80, 443, 3000-3005, 8080, 8443";
      portTimeoutMs = options?.port_timeout_ms || 600;
    }
    previousIsOpen = isOpen;
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
    };

    if (onApplyOptions) {
      onApplyOptions(newOptions);
    }
    onClose();
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

  // --- Batch Scanner Local State ---
  let batchRawUrls = $state(
    "https://example.com\nhttps://httpbin.org\nhttp://testphp.vulnweb.com"
  );
  let isBatchRunning = $state(false);
  let batchItems = $state<BatchScanItem[]>([]);
  let batchCompletedCount = $derived(
    batchItems.filter((i) => i.status === "completed" || i.status === "failed")
      .length
  );

  async function invokeTauri<T>(
    cmd: string,
    args: Record<string, unknown> = {}
  ): Promise<T> {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(cmd, args);
  }

  async function startBatchScan() {
    const lines = batchRawUrls
      .split("\n")
      .map((l) => l.trim())
      .filter((l) => l.length > 0);

    if (lines.length === 0) return;

    isBatchRunning = true;
    batchItems = lines.map((url) => ({
      url,
      status: "scanning",
      report: null,
      error: null,
    }));

    try {
      const results = await invokeTauri<BatchScanItem[]>("scan_batch", {
        urls: lines,
        options,
      });
      batchItems = results;
    } catch (e: any) {
      console.error("Batch scan error:", e);
    } finally {
      isBatchRunning = false;
    }
  }

  // --- Keyboard Shortcuts Reference ---
  const shortcutsList = [
    { key: "⌘ / Ctrl + ,", description: "Open Settings & Configuration" },
    { key: "⌘ / Ctrl + K", description: "Focus target domain input" },
    { key: "⌘ / Ctrl + H", description: "Open Scan History Archive" },
    { key: "⌘ / Ctrl + O", description: "Open Scan & Recon Parameters" },
    { key: "⌘ / Ctrl + M", description: "Open Continuous Watchdog" },
    { key: "⌘ / Ctrl + B", description: "Open Fleet Batch Scanner" },
    { key: "⌘ / Ctrl + E", description: "Export Report (Markdown, JSON)" },
    { key: "⌘ / Ctrl + P", description: "Print Executive PDF Report" },
    { key: "Esc", description: "Close active modal or settings" },
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

{#if isOpen}
  <!-- Backdrop -->
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
      class="bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none w-full max-w-4xl max-h-[88vh] flex flex-col shadow-2xl overflow-hidden text-[var(--color-text-body)]"
    >
      <!-- Modal Header -->
      <div class="px-5 py-3.5 border-b border-[var(--color-hairline)] flex items-center justify-between bg-[var(--color-surface)]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-none bg-[var(--color-canvas)] border border-[var(--color-hairline)] flex items-center justify-center text-[var(--color-signal-red)]">
            <Sliders class="w-4 h-4" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-xs font-black text-[var(--color-text-headline)] uppercase tracking-tight font-mono">Settings & Engine Configuration</h2>
              <span class="px-1.5 py-0.2 text-[10px] bg-[var(--color-canvas)] text-[var(--color-text-muted)] rounded-none font-mono border border-[var(--color-hairline)] uppercase font-bold">
                VULNRADAR
              </span>
            </div>
            <p class="text-[11px] text-[var(--color-text-muted)] font-mono uppercase">Manage audit parameters, active watchdog, scanner profiles, and preferences</p>
          </div>
        </div>

        <button
          type="button"
          onclick={onClose}
          class="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] rounded-none transition-colors cursor-pointer"
          aria-label="Close settings dialog"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Main Modal Layout: Left Sidebar Navigation + Right Content Area -->
      <div class="flex-1 flex flex-col md:flex-row min-h-0 overflow-hidden">
        
        <!-- Left Sidebar Tabs -->
        <nav class="w-full md:w-56 bg-[var(--color-surface)] border-b md:border-b-0 md:border-r border-[var(--color-hairline)] p-3 flex md:flex-col gap-1 overflow-x-auto md:overflow-y-auto flex-shrink-0 font-mono">
          <button
            type="button"
            onclick={() => (currentTab = "params")}
            class="px-3 py-2 rounded-none text-xs font-bold uppercase flex items-center justify-between transition-colors cursor-pointer {currentTab === 'params' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)]'}"
          >
            <div class="flex items-center gap-2.5">
              <Sliders class="w-3.5 h-3.5" />
              <span>01/PARAMETERS</span>
            </div>
            {#if hasCustomParams}
              <span class="w-1.5 h-1.5 rounded-none bg-[var(--color-signal-red)]" title="Custom parameters active"></span>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => (currentTab = "ports")}
            class="px-3 py-2 rounded-none text-xs font-bold uppercase flex items-center justify-between transition-colors cursor-pointer {currentTab === 'ports' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)]'}"
          >
            <div class="flex items-center gap-2.5">
              <Server class="w-3.5 h-3.5" />
              <span>02/PORTS</span>
            </div>
            {#if enablePortScan}
              <span class="w-1.5 h-1.5 rounded-none bg-emerald-500" title="Port scanner enabled"></span>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => (currentTab = "watchdog")}
            class="px-3 py-2 rounded-none text-xs font-bold uppercase flex items-center justify-between transition-colors cursor-pointer {currentTab === 'watchdog' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)]'}"
          >
            <div class="flex items-center gap-2.5">
              <Activity class="w-3.5 h-3.5" />
              <span>03/WATCHDOG</span>
            </div>
            {#if activeMonitorsCount > 0}
              <span class="px-1.5 py-0.2 text-[10px] font-mono rounded-none bg-[var(--color-canvas)] text-[var(--color-text-headline)] border border-[var(--color-hairline)]">
                {activeMonitorsCount}
              </span>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => (currentTab = "batch")}
            class="px-3 py-2 rounded-none text-xs font-bold uppercase flex items-center justify-between transition-colors cursor-pointer {currentTab === 'batch' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)]'}"
          >
            <div class="flex items-center gap-2.5">
              <Layers class="w-3.5 h-3.5" />
              <span>04/BATCH</span>
            </div>
          </button>

          <button
            type="button"
            onclick={() => (currentTab = "shortcuts")}
            class="px-3 py-2 rounded-none text-xs font-bold uppercase flex items-center justify-between transition-colors cursor-pointer {currentTab === 'shortcuts' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)]'}"
          >
            <div class="flex items-center gap-2.5">
              <Keyboard class="w-3.5 h-3.5" />
              <span>05/SHORTCUTS</span>
            </div>
            <kbd class="text-[10px] font-mono opacity-70">⌘K</kbd>
          </button>

          <div class="hidden md:block my-2 border-t border-[var(--color-hairline)]"></div>

          <button
            type="button"
            onclick={() => (currentTab = "data")}
            class="px-3 py-2 rounded-none text-xs font-bold uppercase flex items-center justify-between transition-colors cursor-pointer {currentTab === 'data' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)]'}"
          >
            <div class="flex items-center gap-2.5">
              <Database class="w-3.5 h-3.5" />
              <span>06/STORAGE</span>
            </div>
            {#if historyCount > 0}
              <span class="text-[10px] font-mono text-[var(--color-text-muted)]">{historyCount}</span>
            {/if}
          </button>
        </nav>

        <!-- Right Content Body -->
        <div class="flex-1 flex flex-col min-h-0 bg-[var(--color-surface)]">
          
          <div class="flex-1 overflow-y-auto p-5 md:p-6 space-y-6">

            <!-- TAB 1: Scan & Recon Parameters -->
            {#if currentTab === "params"}
              <div class="space-y-5 animate-fade-in">
                <div>
                  <h3 class="text-xs font-black text-[var(--color-text-headline)] font-mono uppercase tracking-tight">Scan & Request Parameters</h3>
                  <p class="text-xs text-[var(--color-text-muted)] mt-0.5 font-mono">Control HTTP timeouts, custom headers, tokens, and subdomain discovery</p>
                </div>

                <!-- Subdomain Recon Option -->
                <div class="p-4 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none flex items-center justify-between">
                  <div>
                    <div class="text-xs font-bold text-[var(--color-text-headline)] flex items-center gap-2 font-mono uppercase">
                      <Globe class="w-3.5 h-3.5 text-[var(--color-signal-red)]" />
                      <span>Subdomain Mapping (crt.sh Certificate Logs)</span>
                    </div>
                    <p class="text-[11px] text-[var(--color-text-muted)] mt-0.5 font-mono">
                      Query Certificate Transparency logs for public subdomains belonging to the target domain.
                    </p>
                  </div>
                  <label class="relative inline-flex items-center cursor-pointer ml-3 flex-shrink-0">
                    <input
                      type="checkbox"
                      bind:checked={includeSubdomains}
                      class="sr-only peer"
                    />
                    <div class="w-9 h-5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] peer-focus:outline-none rounded-none peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-[var(--color-text-muted)] after:rounded-none after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-[var(--color-text-headline)] peer-checked:after:bg-[var(--color-canvas)]"></div>
                  </label>
                </div>

                <!-- HTTP Timeout Selector -->
                <div class="space-y-2">
                  <label for="timeout-select" class="block text-xs font-bold text-[var(--color-text-headline)] uppercase tracking-wider font-mono">
                    HTTP Request Socket Timeout
                  </label>
                  <div class="grid grid-cols-4 gap-2" id="timeout-select">
                    {#each [5, 15, 30, 60] as sec}
                      <button
                        type="button"
                        onclick={() => (timeoutSeconds = sec)}
                        class="py-2 text-xs font-mono font-bold rounded-none border transition-colors cursor-pointer {timeoutSeconds === sec ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent' : 'bg-[var(--color-canvas)] text-[var(--color-text-muted)] border-[var(--color-hairline)] hover:text-[var(--color-text-headline)]'}"
                      >
                        {sec}S
                      </button>
                    {/each}
                  </div>
                </div>

                <!-- Custom User-Agent Signature -->
                <div class="space-y-1.5">
                  <label
                    for="settings-user-agent"
                    class="block text-xs font-bold text-[var(--color-text-headline)] uppercase tracking-wider font-mono"
                  >
                    Custom User-Agent Signature
                  </label>
                  <input
                    id="settings-user-agent"
                    type="text"
                    bind:value={userAgentInput}
                    placeholder="DEFAULT: MOZILLA/5.0 ... VULNRADAR/1.0"
                    class="w-full px-3 py-2 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs font-mono text-[var(--color-text-headline)] placeholder-[var(--color-text-muted)] uppercase focus:outline-none"
                  />
                </div>

                <!-- Custom Headers Table -->
                <div class="space-y-2.5">
                  <div class="flex items-center justify-between">
                    <div>
                      <span class="block text-xs font-bold text-[var(--color-text-headline)] uppercase tracking-wider font-mono">
                        Custom HTTP Headers & Auth Tokens
                      </span>
                      <p class="text-[11px] text-[var(--color-text-muted)] font-mono">Injected into all active and passive requests</p>
                    </div>
                    <button
                      type="button"
                      onclick={addHeaderRow}
                      class="px-2.5 py-1 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] rounded-none text-xs font-mono font-bold uppercase flex items-center gap-1 cursor-pointer transition-colors border border-[var(--color-hairline)]"
                    >
                      <Plus class="w-3.5 h-3.5" />
                      <span>Add Header</span>
                    </button>
                  </div>

                  <div class="space-y-2">
                    {#each headerRows as row, i}
                      <div class="flex items-center gap-2">
                        <input
                          type="text"
                          bind:value={row.key}
                          placeholder="HEADER (E.G. AUTHORIZATION)"
                          class="flex-1 px-3 py-1.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs font-mono text-[var(--color-text-headline)] placeholder-[var(--color-text-muted)] uppercase focus:outline-none"
                        />
                        <input
                          type="text"
                          bind:value={row.value}
                          placeholder="VALUE (E.G. BEARER EYJHBGC...)"
                          class="flex-1 px-3 py-1.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs font-mono text-[var(--color-text-headline)] placeholder-[var(--color-text-muted)] focus:outline-none"
                        />
                        <button
                          type="button"
                          onclick={() => removeHeaderRow(i)}
                          class="p-1.5 text-[var(--color-text-muted)] hover:text-red-500 rounded-none transition-colors cursor-pointer"
                          aria-label="Remove header row"
                        >
                          <Trash2 class="w-4 h-4" />
                        </button>
                      </div>
                    {/each}
                  </div>
                </div>
              </div>

            <!-- TAB 2: Network Port Scanner -->
            {:else if currentTab === "ports"}
              <div class="space-y-5 animate-fade-in">
                <div>
                  <h3 class="text-xs font-black text-[var(--color-text-headline)] font-mono uppercase tracking-tight">Network Port Scanner (Nmap Engine)</h3>
                  <p class="text-xs text-[var(--color-text-muted)] mt-0.5 font-mono">Asynchronous TCP port discovery, socket banner grabbing, and exposed database inspection</p>
                </div>

                <!-- Main Toggle -->
                <div class="p-4 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none flex items-center justify-between">
                  <div>
                    <div class="text-xs font-bold text-[var(--color-text-headline)] flex items-center gap-2 font-mono uppercase">
                      <Server class="w-3.5 h-3.5 text-[var(--color-signal-red)]" />
                      <span>Enable TCP Port Scanner Probes</span>
                    </div>
                    <p class="text-[11px] text-[var(--color-text-muted)] mt-0.5 font-mono">
                      Probes the resolved host IP address for listening services, sensitive administrative ports, and remote databases.
                    </p>
                  </div>
                  <label class="relative inline-flex items-center cursor-pointer ml-3 flex-shrink-0">
                    <input
                      type="checkbox"
                      bind:checked={enablePortScan}
                      class="sr-only peer"
                    />
                    <div class="w-9 h-5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] peer-focus:outline-none rounded-none peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-[var(--color-text-muted)] after:rounded-none after:h-3.5 after:w-3.5 after:transition-all peer-checked:bg-[var(--color-text-headline)] peer-checked:after:bg-[var(--color-canvas)]"></div>
                  </label>
                </div>

                {#if enablePortScan}
                  <div class="space-y-4 pt-1">
                    <!-- Port Profiles -->
                    <div class="space-y-2">
                      <span class="block text-xs font-bold text-[var(--color-text-headline)] uppercase tracking-wider font-mono">
                        Port Target Profile
                      </span>
                      <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
                        <button
                          type="button"
                          onclick={() => (portScanProfile = "top20")}
                          class="p-2.5 rounded-none border text-left transition-colors cursor-pointer {portScanProfile === 'top20' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent font-bold' : 'bg-[var(--color-canvas)] border-[var(--color-hairline)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)]'}"
                        >
                          <div class="text-xs font-mono font-bold uppercase">Top 20 Ports</div>
                          <div class="text-[10px] opacity-80 mt-0.5 font-mono uppercase">Fast web & core</div>
                        </button>
                        <button
                          type="button"
                          onclick={() => (portScanProfile = "top100")}
                          class="p-2.5 rounded-none border text-left transition-colors cursor-pointer {portScanProfile === 'top100' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent font-bold' : 'bg-[var(--color-canvas)] border-[var(--color-hairline)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)]'}"
                        >
                          <div class="text-xs font-mono font-bold uppercase">Top 100 Ports</div>
                          <div class="text-[10px] opacity-80 mt-0.5 font-mono uppercase">Standard Nmap set</div>
                        </button>
                        <button
                          type="button"
                          onclick={() => (portScanProfile = "databases")}
                          class="p-2.5 rounded-none border text-left transition-colors cursor-pointer {portScanProfile === 'databases' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent font-bold' : 'bg-[var(--color-canvas)] border-[var(--color-hairline)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)]'}"
                        >
                          <div class="text-xs font-mono font-bold uppercase">Databases</div>
                          <div class="text-[10px] opacity-80 mt-0.5 font-mono uppercase">MySQL, Redis, Mongo</div>
                        </button>
                        <button
                          type="button"
                          onclick={() => (portScanProfile = "custom")}
                          class="p-2.5 rounded-none border text-left transition-colors cursor-pointer {portScanProfile === 'custom' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent font-bold' : 'bg-[var(--color-canvas)] border-[var(--color-hairline)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)]'}"
                        >
                          <div class="text-xs font-mono font-bold uppercase">Custom List</div>
                          <div class="text-[10px] opacity-80 mt-0.5 font-mono uppercase">Specify ranges</div>
                        </button>
                      </div>
                    </div>

                    <!-- Custom Port Input -->
                    {#if portScanProfile === "custom"}
                      <div class="space-y-1.5">
                        <label for="settings-custom-ports" class="block text-xs font-bold text-[var(--color-text-headline)] uppercase tracking-wider font-mono">
                          Custom Ports & Ranges (e.g. 21, 22, 80, 443, 3000-3005, 8080)
                        </label>
                        <input
                          id="settings-custom-ports"
                          type="text"
                          bind:value={customPortsInput}
                          placeholder="80, 443, 3000-3005, 8080, 8443"
                          class="w-full px-3 py-2 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs font-mono text-[var(--color-text-headline)] placeholder-[var(--color-text-muted)] focus:outline-none"
                        />
                      </div>
                    {/if}

                    <!-- Socket Timeout Per Probe -->
                    <div class="space-y-1.5">
                      <span class="block text-xs font-bold text-[var(--color-text-headline)] uppercase tracking-wider font-mono">
                        Port Probe Socket Timeout
                      </span>
                      <div class="grid grid-cols-3 gap-2">
                        {#each [{ label: "FAST (500MS)", ms: 500 }, { label: "BALANCED (800MS)", ms: 800 }, { label: "THOROUGH (1500MS)", ms: 1500 }] as t}
                          <button
                            type="button"
                            onclick={() => (portTimeoutMs = t.ms)}
                            class="py-2 text-xs font-mono font-bold rounded-none border transition-colors cursor-pointer {portTimeoutMs === t.ms ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent' : 'bg-[var(--color-canvas)] text-[var(--color-text-muted)] border-[var(--color-hairline)] hover:text-[var(--color-text-headline)]'}"
                          >
                            {t.label}
                          </button>
                        {/each}
                      </div>
                    </div>
                  </div>
                {/if}
              </div>

            <!-- TAB 3: Continuous Watchdog -->
            {:else if currentTab === "watchdog"}
              <div class="space-y-5 animate-fade-in">
                <div class="flex items-center justify-between">
                  <div>
                    <h3 class="text-xs font-black text-[var(--color-text-headline)] font-mono uppercase tracking-tight">Continuous Security Watchdog</h3>
                    <p class="text-xs text-[var(--color-text-muted)] mt-0.5 font-mono">Automated background re-scanning with score degradation notifications</p>
                  </div>
                  <span class="px-2 py-0.5 text-xs font-mono bg-[var(--color-canvas)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] rounded-none font-bold uppercase">
                    {monitors.length} MONITORED DOMAINS
                  </span>
                </div>

                <!-- Add Watchdog Form -->
                <div class="p-4 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-3">
                  <div class="flex items-center gap-1.5 text-xs font-bold text-[var(--color-text-headline)] font-mono uppercase">
                    <BellRing class="w-3.5 h-3.5 text-[var(--color-signal-red)]" />
                    <span>Schedule New Domain Watchdog</span>
                  </div>
                  <div class="flex flex-col sm:flex-row items-center gap-2">
                    <input
                      type="text"
                      bind:value={newWatchdogUrl}
                      placeholder="HTTPS://EXAMPLE.COM"
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
                      onclick={handleAddWatchdog}
                      disabled={!newWatchdogUrl.trim() || isAddingWatchdog}
                      class="w-full sm:w-auto px-3.5 py-1.5 bg-[var(--color-text-headline)] hover:opacity-90 disabled:opacity-50 text-[var(--color-canvas)] font-mono font-bold uppercase rounded-none text-xs flex items-center justify-center gap-1.5 transition-opacity cursor-pointer flex-shrink-0"
                    >
                      <Plus class="w-3.5 h-3.5" />
                      <span>Add Target</span>
                    </button>
                  </div>
                </div>

                <!-- Configured Domains List -->
                <div class="space-y-2.5">
                  <div class="flex items-center justify-between">
                    <span class="text-xs font-bold text-[var(--color-text-headline)] uppercase tracking-wider font-mono">
                      Active Watchdog Fleet ({monitors.length})
                    </span>
                  </div>

                  {#if monitors.length === 0}
                    <div class="py-12 text-center text-[var(--color-text-muted)] bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none font-mono">
                      <Activity class="w-8 h-8 mx-auto mb-2 opacity-30 text-[var(--color-text-muted)]" />
                      <p class="text-xs font-bold text-[var(--color-text-headline)] uppercase">No continuous monitors configured</p>
                      <p class="text-[11px] text-[var(--color-text-muted)] mt-0.5 max-w-sm mx-auto">
                        Add web properties above to track and alert on score changes and new CVEs.
                      </p>
                    </div>
                  {:else}
                    <div class="space-y-2 max-h-72 overflow-y-auto">
                      {#each monitors as item}
                        <div
                          class="p-3 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none flex items-center justify-between gap-3"
                        >
                          <div class="flex items-center gap-3 min-w-0 flex-1">
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
                                <span>Last: {formatWatchdogDate(item.last_scanned_at)}</span>
                                <span>•</span>
                                <span>Next: {formatWatchdogDate(item.next_scan_at)}</span>
                              </div>
                            </div>
                          </div>

                          <div class="flex items-center gap-1.5 flex-shrink-0">
                            {#if item.last_score !== null && item.last_score !== undefined}
                              <span
                                class="px-2 py-0.5 text-xs font-mono font-bold uppercase rounded-none border {getScoreBadge(item.last_score)}"
                              >
                                {item.last_score} pts
                              </span>
                            {/if}

                            {#if onScanTarget}
                              <button
                                type="button"
                                onclick={() => {
                                  onScanTarget?.(item.target_url);
                                  onClose();
                                }}
                                class="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] rounded-none transition-colors cursor-pointer"
                                title="Run Audit Now"
                              >
                                <RotateCw class="w-3.5 h-3.5" />
                              </button>
                            {/if}

                            {#if onToggleMonitor}
                              <button
                                type="button"
                                onclick={() => onToggleMonitor?.(item.id)}
                                class="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] rounded-none transition-colors cursor-pointer"
                                title={item.is_active ? 'Pause Watchdog' : 'Resume Watchdog'}
                              >
                                {#if item.is_active}
                                  <Pause class="w-3.5 h-3.5" />
                                {:else}
                                  <Play class="w-3.5 h-3.5" />
                                {/if}
                              </button>
                            {/if}

                            {#if onDeleteMonitor}
                              <button
                                type="button"
                                onclick={() => onDeleteMonitor?.(item.id)}
                                class="p-1.5 text-[var(--color-text-muted)] hover:text-red-500 rounded-none transition-colors cursor-pointer"
                                title="Delete Watchdog"
                              >
                                <Trash2 class="w-3.5 h-3.5" />
                              </button>
                            {/if}
                          </div>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              </div>

            <!-- TAB 4: Batch Fleet Scanner -->
            {:else if currentTab === "batch"}
              <div class="space-y-5 animate-fade-in">
                <div>
                  <h3 class="text-xs font-black text-[var(--color-text-headline)] font-mono uppercase tracking-tight">Batch Fleet Security Scanner</h3>
                  <p class="text-xs text-[var(--color-text-muted)] mt-0.5 font-mono">Audit multiple endpoints in sequence to evaluate entire domain portfolios</p>
                </div>

                {#if !isBatchRunning && batchItems.length === 0}
                  <div class="space-y-3.5">
                    <div>
                      <div class="flex items-center justify-between mb-1.5">
                        <label
                          for="settings-batch-urls"
                          class="block text-xs font-bold text-[var(--color-text-headline)] uppercase tracking-wider font-mono"
                        >
                          Target Inventory (One URL Per Line)
                        </label>
                        <span class="text-[11px] text-[var(--color-text-muted)] font-mono uppercase">Supports HTTP & HTTPS</span>
                      </div>
                      <textarea
                        id="settings-batch-urls"
                        bind:value={batchRawUrls}
                        rows="5"
                        placeholder="https://example.com&#10;https://api.example.com&#10;https://staging.example.com"
                        class="w-full p-3 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs font-mono text-[var(--color-text-headline)] placeholder-[var(--color-text-muted)] focus:outline-none"
                      ></textarea>
                    </div>

                    <div class="p-3.5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                      <div class="text-xs text-[var(--color-text-muted)] font-mono">
                        <span class="font-bold text-[var(--color-text-headline)] uppercase">Sequential Fleet Queue:</span>
                        Targets are scanned sequentially using active audit parameters.
                      </div>
                      <button
                        type="button"
                        onclick={startBatchScan}
                        class="px-4 py-1.5 bg-[var(--color-signal-red)] hover:opacity-90 text-white font-mono font-bold uppercase rounded-none text-xs flex items-center gap-2 transition-opacity cursor-pointer flex-shrink-0"
                      >
                        <Play class="w-3.5 h-3.5 fill-current" />
                        <span>Launch Fleet Audit</span>
                      </button>
                    </div>
                  </div>
                {:else}
                  <!-- Batch Running / Completed View -->
                  <div class="space-y-4">
                    <div class="flex items-center justify-between">
                      <div>
                        <h4 class="text-xs font-black text-[var(--color-text-headline)] font-mono uppercase">
                          {isBatchRunning ? "Auditing Fleet Surface..." : "Fleet Assessment Completed"}
                        </h4>
                        <p class="text-xs text-[var(--color-text-muted)] font-mono mt-0.5 uppercase">
                          PROGRESS: <strong class="text-[var(--color-text-headline)] font-bold">{batchCompletedCount}</strong> OF {batchItems.length} AUDITED
                        </p>
                      </div>

                      {#if !isBatchRunning}
                        <button
                          type="button"
                          onclick={() => (batchItems = [])}
                          class="px-3 py-1 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs font-mono font-bold uppercase cursor-pointer transition-colors"
                        >
                          New Batch
                        </button>
                      {/if}
                    </div>

                    <!-- Progress Bar -->
                    <div class="w-full bg-[var(--color-canvas)] border border-[var(--color-hairline)] rounded-none h-2 overflow-hidden">
                      <div
                        class="bg-[var(--color-signal-red)] h-full transition-all duration-300 rounded-none"
                        style="width: {batchItems.length > 0 ? (batchCompletedCount / batchItems.length) * 100 : 0}%"
                      ></div>
                    </div>

                    <!-- Results -->
                    <div class="space-y-2 max-h-60 overflow-y-auto">
                      {#each batchItems as item}
                        <div
                          class="p-2.5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none flex items-center justify-between gap-3"
                        >
                          <div class="flex items-center gap-2.5 min-w-0 flex-1">
                            {#if item.status === "scanning"}
                              <Loader2 class="w-3.5 h-3.5 text-blue-500 animate-spin flex-shrink-0" />
                            {:else if item.status === "completed"}
                              <CheckCircle2 class="w-3.5 h-3.5 text-emerald-500 flex-shrink-0" />
                            {:else}
                              <AlertOctagon class="w-3.5 h-3.5 text-red-500 flex-shrink-0" />
                            {/if}

                            <div class="min-w-0 flex-1">
                              <div class="text-xs font-bold text-[var(--color-text-headline)] truncate font-mono">
                                {item.url}
                              </div>
                              {#if item.error}
                                <div class="text-[10px] text-red-500 font-mono mt-0.5 truncate">
                                  {item.error}
                                </div>
                              {:else if item.report}
                                <div class="text-[10px] text-[var(--color-text-muted)] font-mono mt-0.5 flex items-center gap-2 uppercase">
                                  <span>{item.report.total_findings} findings</span>
                                  {#if item.report.critical_count > 0}
                                    <span class="text-red-500 font-bold">({item.report.critical_count} critical)</span>
                                  {/if}
                                </div>
                              {/if}
                            </div>
                          </div>

                          {#if item.report}
                            <div class="flex items-center gap-2 flex-shrink-0">
                              <span
                                class="px-2 py-0.5 text-xs font-mono font-bold uppercase rounded-none border {getScoreBadge(item.report.security_score)}"
                              >
                                {item.report.security_score} pts
                              </span>
                              {#if onSelectBatchReport}
                                <button
                                  type="button"
                                  onclick={() => {
                                    if (item.report) {
                                      onSelectBatchReport?.(item.report);
                                      onClose();
                                    }
                                  }}
                                  class="px-2 py-1 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs font-mono font-bold uppercase flex items-center gap-1 cursor-pointer transition-colors"
                                >
                                  <span>Inspect</span>
                                  <ArrowRight class="w-3 h-3" />
                                </button>
                              {/if}
                            </div>
                          {/if}
                        </div>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>

            <!-- TAB 5: Keyboard Shortcuts Reference -->
            {:else if currentTab === "shortcuts"}
              <div class="space-y-4 animate-fade-in">
                <div>
                  <h3 class="text-xs font-black text-[var(--color-text-headline)] font-mono uppercase tracking-tight">Keyboard Shortcuts & Commands</h3>
                  <p class="text-xs text-[var(--color-text-muted)] mt-0.5 font-mono">Quick hotkeys for high-velocity navigation and scanning</p>
                </div>

                <div class="p-3.5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none divide-y divide-[var(--color-hairline)]">
                  {#each shortcutsList as sc}
                    <div class="flex items-center justify-between py-2 first:pt-0 last:pb-0">
                      <span class="text-xs font-mono text-[var(--color-text-body)]">{sc.description}</span>
                      <kbd class="px-2 py-0.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] rounded-none text-[11px] font-mono font-bold text-[var(--color-text-headline)]">
                        {sc.key}
                      </kbd>
                    </div>
                  {/each}
                </div>
              </div>

            <!-- TAB 6: Storage & About -->
            {:else if currentTab === "data"}
              <div class="space-y-5 animate-fade-in">
                <div>
                  <h3 class="text-xs font-black text-[var(--color-text-headline)] font-mono uppercase tracking-tight">Storage, Diagnostics & About</h3>
                  <p class="text-xs text-[var(--color-text-muted)] mt-0.5 font-mono">Local SQLite database state and system build information</p>
                </div>

                <!-- Database Metrics -->
                <div class="grid grid-cols-2 gap-3">
                  <div class="p-3.5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-1">
                    <span class="text-[11px] font-bold text-[var(--color-text-muted)] uppercase tracking-wider font-mono">Persisted Scans</span>
                    <div class="text-lg font-bold font-mono text-[var(--color-text-headline)] flex items-center justify-between">
                      <span>{historyCount} Snapshots</span>
                      {#if onOpenHistory}
                        <button
                          type="button"
                          onclick={() => {
                            onClose();
                            onOpenHistory?.();
                          }}
                          class="text-xs text-[var(--color-signal-red)] hover:underline font-mono uppercase cursor-pointer"
                        >
                          View Archive
                        </button>
                      {/if}
                    </div>
                  </div>

                  <div class="p-3.5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-1">
                    <span class="text-[11px] font-bold text-[var(--color-text-muted)] uppercase tracking-wider font-mono">Watchdog Domains</span>
                    <div class="text-lg font-bold font-mono text-[var(--color-text-headline)]">
                      {monitors.length} Configured
                    </div>
                  </div>
                </div>

                <!-- Clear History Action -->
                {#if onClearHistory && historyCount > 0}
                  <div class="p-4 bg-red-500/10 border border-red-500/30 rounded-none flex items-center justify-between gap-4">
                    <div>
                      <div class="text-xs font-bold text-red-600 dark:text-red-400 font-mono uppercase">Clear Local Scan Archive</div>
                      <p class="text-[11px] text-[var(--color-text-muted)] mt-0.5 font-mono">Permanently delete all historical audit reports stored in the local SQLite database.</p>
                    </div>
                    <button
                      type="button"
                      onclick={onClearHistory}
                      class="px-3 py-1.5 bg-[var(--color-signal-red)] hover:opacity-90 text-white rounded-none text-xs font-mono font-bold uppercase cursor-pointer transition-opacity flex-shrink-0"
                    >
                      Clear History
                    </button>
                  </div>
                {/if}

                <!-- About Card -->
                <div class="p-4 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-2.5">
                  <div class="flex items-center gap-2.5">
                    <div class="w-7 h-7 rounded-none bg-[var(--color-canvas)] border border-[var(--color-hairline)] flex items-center justify-center text-[var(--color-signal-red)]">
                      <ShieldCheck class="w-4 h-4" />
                    </div>
                    <div>
                      <div class="text-xs font-bold text-[var(--color-text-headline)] font-mono uppercase">VulnRadar Desktop v0.8.0</div>
                      <div class="text-[11px] text-[var(--color-text-muted)] font-mono uppercase">Tauri v2 • Svelte 5 • Rust 2021 Engine</div>
                    </div>
                  </div>
                  <p class="text-xs text-[var(--color-text-muted)] leading-relaxed font-mono">
                    Lightweight passive reconnaissance, HTTP security posture verification, TLS cipher audit, DNS SPF/DMARC alignment, and asynchronous TCP port scanner.
                  </p>
                </div>
              </div>
            {/if}

          </div>

          <!-- Modal Footer (Shown for editable parameter tabs) -->
          {#if currentTab === "params" || currentTab === "ports"}
            <div class="px-5 py-3 border-t border-[var(--color-hairline)] flex items-center justify-between bg-[var(--color-surface)]">
              <div class="text-[11px] text-[var(--color-text-muted)] font-mono uppercase">
                Changes apply immediately to subsequent security audits.
              </div>
              <div class="flex items-center gap-2">
                <button
                  type="button"
                  onclick={onClose}
                  class="px-3 py-1.5 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs font-mono font-bold uppercase cursor-pointer transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="button"
                  onclick={handleSaveParameters}
                  class="px-4 py-1.5 bg-[var(--color-text-headline)] hover:opacity-90 text-[var(--color-canvas)] font-mono font-bold uppercase rounded-none text-xs transition-opacity cursor-pointer"
                >
                  Apply Parameters
                </button>
              </div>
            </div>
          {/if}

        </div>

      </div>

    </div>
  </div>
{/if}
