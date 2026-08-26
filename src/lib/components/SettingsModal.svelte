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
    if (score === undefined || score === null) return "bg-neutral-800 text-neutral-400";
    if (score >= 85) return "bg-emerald-950/40 text-emerald-300 border-emerald-800/50";
    if (score >= 70) return "bg-blue-950/40 text-blue-300 border-blue-800/50";
    if (score >= 50) return "bg-amber-950/40 text-amber-300 border-amber-800/50";
    return "bg-rose-950/40 text-rose-300 border-rose-800/50";
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
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/65 backdrop-blur-xs p-4 animate-fade-in"
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
      class="bg-[#202020] border border-[#333333] rounded-2xl w-full max-w-4xl max-h-[88vh] flex flex-col shadow-2xl overflow-hidden text-[#e3e2e0]"
    >
      <!-- Modal Header -->
      <div class="px-5 py-3.5 border-b border-[#2e2e2e] flex items-center justify-between bg-[#191919]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-[#282828] border border-[#383838] flex items-center justify-center text-neutral-200">
            <Sliders class="w-4 h-4 text-white" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-sm font-semibold text-white tracking-tight">Settings & Engine Configuration</h2>
              <span class="px-1.5 py-0.2 text-[10px] bg-[#282828] text-neutral-400 rounded font-mono border border-[#383838]">
                VulnRadar
              </span>
            </div>
            <p class="text-[11px] text-neutral-400">Manage audit parameters, active watchdog, scanner profiles, and preferences</p>
          </div>
        </div>

        <button
          type="button"
          onclick={onClose}
          class="p-1.5 text-neutral-400 hover:text-white hover:bg-[#282828] rounded-lg transition-colors cursor-pointer"
          aria-label="Close settings dialog"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Main Modal Layout: Left Sidebar Navigation + Right Content Area -->
      <div class="flex-1 flex flex-col md:flex-row min-h-0 overflow-hidden">
        
        <!-- Left Sidebar Tabs -->
        <nav class="w-full md:w-56 bg-[#181818] border-b md:border-b-0 md:border-r border-[#2a2a2a] p-3 flex md:flex-col gap-1 overflow-x-auto md:overflow-y-auto flex-shrink-0">
          <button
            type="button"
            onclick={() => (currentTab = "params")}
            class="px-3 py-2 rounded-lg text-xs font-medium flex items-center justify-between transition-colors cursor-pointer {currentTab === 'params' ? 'bg-[#282828] text-white font-semibold shadow-xs' : 'text-neutral-400 hover:text-neutral-200 hover:bg-[#202020]'}"
          >
            <div class="flex items-center gap-2.5">
              <Sliders class="w-3.5 h-3.5 {currentTab === 'params' ? 'text-white' : 'text-neutral-400'}" />
              <span>Scan Parameters</span>
            </div>
            {#if hasCustomParams}
              <span class="w-1.5 h-1.5 rounded-full bg-blue-400" title="Custom parameters active"></span>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => (currentTab = "ports")}
            class="px-3 py-2 rounded-lg text-xs font-medium flex items-center justify-between transition-colors cursor-pointer {currentTab === 'ports' ? 'bg-[#282828] text-white font-semibold shadow-xs' : 'text-neutral-400 hover:text-neutral-200 hover:bg-[#202020]'}"
          >
            <div class="flex items-center gap-2.5">
              <Server class="w-3.5 h-3.5 {currentTab === 'ports' ? 'text-white' : 'text-neutral-400'}" />
              <span>Port Scanner</span>
            </div>
            {#if enablePortScan}
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-400" title="Port scanner enabled"></span>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => (currentTab = "watchdog")}
            class="px-3 py-2 rounded-lg text-xs font-medium flex items-center justify-between transition-colors cursor-pointer {currentTab === 'watchdog' ? 'bg-[#282828] text-white font-semibold shadow-xs' : 'text-neutral-400 hover:text-neutral-200 hover:bg-[#202020]'}"
          >
            <div class="flex items-center gap-2.5">
              <Activity class="w-3.5 h-3.5 {currentTab === 'watchdog' ? 'text-white' : 'text-neutral-400'}" />
              <span>Watchdog</span>
            </div>
            {#if activeMonitorsCount > 0}
              <span class="px-1.5 py-0.2 text-[10px] font-mono rounded bg-emerald-950/60 text-emerald-400 border border-emerald-800/40">
                {activeMonitorsCount}
              </span>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => (currentTab = "batch")}
            class="px-3 py-2 rounded-lg text-xs font-medium flex items-center justify-between transition-colors cursor-pointer {currentTab === 'batch' ? 'bg-[#282828] text-white font-semibold shadow-xs' : 'text-neutral-400 hover:text-neutral-200 hover:bg-[#202020]'}"
          >
            <div class="flex items-center gap-2.5">
              <Layers class="w-3.5 h-3.5 {currentTab === 'batch' ? 'text-white' : 'text-neutral-400'}" />
              <span>Batch Fleet</span>
            </div>
          </button>

          <button
            type="button"
            onclick={() => (currentTab = "shortcuts")}
            class="px-3 py-2 rounded-lg text-xs font-medium flex items-center justify-between transition-colors cursor-pointer {currentTab === 'shortcuts' ? 'bg-[#282828] text-white font-semibold shadow-xs' : 'text-neutral-400 hover:text-neutral-200 hover:bg-[#202020]'}"
          >
            <div class="flex items-center gap-2.5">
              <Keyboard class="w-3.5 h-3.5 {currentTab === 'shortcuts' ? 'text-white' : 'text-neutral-400'}" />
              <span>Shortcuts</span>
            </div>
            <kbd class="text-[10px] font-mono text-neutral-500">⌘K</kbd>
          </button>

          <div class="hidden md:block my-2 border-t border-[#2a2a2a]"></div>

          <button
            type="button"
            onclick={() => (currentTab = "data")}
            class="px-3 py-2 rounded-lg text-xs font-medium flex items-center justify-between transition-colors cursor-pointer {currentTab === 'data' ? 'bg-[#282828] text-white font-semibold shadow-xs' : 'text-neutral-400 hover:text-neutral-200 hover:bg-[#202020]'}"
          >
            <div class="flex items-center gap-2.5">
              <Database class="w-3.5 h-3.5 {currentTab === 'data' ? 'text-white' : 'text-neutral-400'}" />
              <span>Storage & About</span>
            </div>
            {#if historyCount > 0}
              <span class="text-[10px] font-mono text-neutral-500">{historyCount}</span>
            {/if}
          </button>
        </nav>

        <!-- Right Content Body -->
        <div class="flex-1 flex flex-col min-h-0 bg-[#202020]">
          
          <div class="flex-1 overflow-y-auto p-5 md:p-6 space-y-6">

            <!-- TAB 1: Scan & Recon Parameters -->
            {#if currentTab === "params"}
              <div class="space-y-5 animate-fade-in">
                <div>
                  <h3 class="text-sm font-semibold text-white">Scan & Request Parameters</h3>
                  <p class="text-xs text-neutral-400 mt-0.5">Control HTTP timeouts, custom headers, tokens, and subdomain discovery</p>
                </div>

                <!-- Subdomain Recon Option -->
                <div class="p-4 bg-[#191919] border border-[#2e2e2e] rounded-xl flex items-center justify-between">
                  <div>
                    <div class="text-xs font-medium text-neutral-200 flex items-center gap-2">
                      <Globe class="w-3.5 h-3.5 text-neutral-400" />
                      <span>Subdomain Mapping (crt.sh Certificate Logs)</span>
                    </div>
                    <p class="text-[11px] text-neutral-400 mt-0.5">
                      Query Certificate Transparency logs for public subdomains belonging to the target domain.
                    </p>
                  </div>
                  <label class="relative inline-flex items-center cursor-pointer ml-3 flex-shrink-0">
                    <input
                      type="checkbox"
                      bind:checked={includeSubdomains}
                      class="sr-only peer"
                    />
                    <div class="w-9 h-5 bg-[#333333] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-white peer-checked:after:bg-neutral-950"></div>
                  </label>
                </div>

                <!-- HTTP Timeout Selector -->
                <div class="space-y-2">
                  <label for="timeout-select" class="block text-xs font-medium text-neutral-400 uppercase tracking-wider">
                    HTTP Request Socket Timeout
                  </label>
                  <div class="grid grid-cols-4 gap-2" id="timeout-select">
                    {#each [5, 15, 30, 60] as sec}
                      <button
                        type="button"
                        onclick={() => (timeoutSeconds = sec)}
                        class="py-2 text-xs font-mono font-medium rounded-lg border transition-colors cursor-pointer {timeoutSeconds === sec ? 'bg-[#2a2a2a] text-white border-neutral-400 shadow-xs' : 'bg-[#191919] text-neutral-400 border-[#2e2e2e] hover:border-[#3a3a3a]'}"
                      >
                        {sec}s
                      </button>
                    {/each}
                  </div>
                </div>

                <!-- Custom User-Agent Signature -->
                <div class="space-y-1.5">
                  <label
                    for="settings-user-agent"
                    class="block text-xs font-medium text-neutral-400 uppercase tracking-wider"
                  >
                    Custom User-Agent Signature
                  </label>
                  <input
                    id="settings-user-agent"
                    type="text"
                    bind:value={userAgentInput}
                    placeholder="Default: Mozilla/5.0 (Windows NT 10.0; Win64; x64) VulnRadar/1.0"
                    class="w-full px-3 py-2 bg-[#191919] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs font-mono text-neutral-200 placeholder-neutral-500 focus:outline-none"
                  />
                </div>

                <!-- Custom Headers Table -->
                <div class="space-y-2.5">
                  <div class="flex items-center justify-between">
                    <div>
                      <span class="block text-xs font-medium text-neutral-400 uppercase tracking-wider">
                        Custom HTTP Headers & Auth Tokens
                      </span>
                      <p class="text-[11px] text-neutral-500">Injected into all active and passive requests</p>
                    </div>
                    <button
                      type="button"
                      onclick={addHeaderRow}
                      class="px-2.5 py-1 bg-[#252525] hover:bg-[#2f2f2f] text-neutral-300 hover:text-white rounded-lg text-xs flex items-center gap-1 font-medium cursor-pointer transition-colors border border-[#333]"
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
                          placeholder="Header (e.g. Authorization or X-API-Key)"
                          class="flex-1 px-3 py-1.5 bg-[#191919] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs font-mono text-neutral-200 placeholder-neutral-500 focus:outline-none"
                        />
                        <input
                          type="text"
                          bind:value={row.value}
                          placeholder="Value (e.g. Bearer eyJhbGci...)"
                          class="flex-1 px-3 py-1.5 bg-[#191919] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs font-mono text-neutral-200 placeholder-neutral-500 focus:outline-none"
                        />
                        <button
                          type="button"
                          onclick={() => removeHeaderRow(i)}
                          class="p-1.5 text-neutral-500 hover:text-red-400 rounded-lg transition-colors cursor-pointer"
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
                  <h3 class="text-sm font-semibold text-white">Network Port Scanner (Nmap Engine)</h3>
                  <p class="text-xs text-neutral-400 mt-0.5">Asynchronous TCP port discovery, socket banner grabbing, and exposed database inspection</p>
                </div>

                <!-- Main Toggle -->
                <div class="p-4 bg-[#191919] border border-[#2e2e2e] rounded-xl flex items-center justify-between">
                  <div>
                    <div class="text-xs font-medium text-neutral-200 flex items-center gap-2">
                      <Server class="w-3.5 h-3.5 text-neutral-400" />
                      <span>Enable TCP Port Scanner Probes</span>
                    </div>
                    <p class="text-[11px] text-neutral-400 mt-0.5">
                      Probes the resolved host IP address for listening services, sensitive administrative ports, and remote databases.
                    </p>
                  </div>
                  <label class="relative inline-flex items-center cursor-pointer ml-3 flex-shrink-0">
                    <input
                      type="checkbox"
                      bind:checked={enablePortScan}
                      class="sr-only peer"
                    />
                    <div class="w-9 h-5 bg-[#333333] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-white peer-checked:after:bg-neutral-950"></div>
                  </label>
                </div>

                {#if enablePortScan}
                  <div class="space-y-4 pt-1">
                    <!-- Port Profiles -->
                    <div class="space-y-2">
                      <span class="block text-xs font-medium text-neutral-400 uppercase tracking-wider">
                        Port Target Profile
                      </span>
                      <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
                        <button
                          type="button"
                          onclick={() => (portScanProfile = "top20")}
                          class="p-2.5 rounded-lg border text-left transition-colors cursor-pointer {portScanProfile === 'top20' ? 'bg-[#2a2a2a] border-neutral-400 text-white' : 'bg-[#191919] border-[#2e2e2e] text-neutral-400 hover:border-[#3a3a3a]'}"
                        >
                          <div class="text-xs font-medium">Top 20 Ports</div>
                          <div class="text-[10px] text-neutral-400 mt-0.5">Fast web & core infra</div>
                        </button>
                        <button
                          type="button"
                          onclick={() => (portScanProfile = "top100")}
                          class="p-2.5 rounded-lg border text-left transition-colors cursor-pointer {portScanProfile === 'top100' ? 'bg-[#2a2a2a] border-neutral-400 text-white' : 'bg-[#191919] border-[#2e2e2e] text-neutral-400 hover:border-[#3a3a3a]'}"
                        >
                          <div class="text-xs font-medium">Top 100 Ports</div>
                          <div class="text-[10px] text-neutral-400 mt-0.5">Standard Nmap set</div>
                        </button>
                        <button
                          type="button"
                          onclick={() => (portScanProfile = "databases")}
                          class="p-2.5 rounded-lg border text-left transition-colors cursor-pointer {portScanProfile === 'databases' ? 'bg-[#2a2a2a] border-neutral-400 text-white' : 'bg-[#191919] border-[#2e2e2e] text-neutral-400 hover:border-[#3a3a3a]'}"
                        >
                          <div class="text-xs font-medium">Databases</div>
                          <div class="text-[10px] text-neutral-400 mt-0.5">MySQL, Redis, Mongo</div>
                        </button>
                        <button
                          type="button"
                          onclick={() => (portScanProfile = "custom")}
                          class="p-2.5 rounded-lg border text-left transition-colors cursor-pointer {portScanProfile === 'custom' ? 'bg-[#2a2a2a] border-neutral-400 text-white' : 'bg-[#191919] border-[#2e2e2e] text-neutral-400 hover:border-[#3a3a3a]'}"
                        >
                          <div class="text-xs font-medium">Custom List</div>
                          <div class="text-[10px] text-neutral-400 mt-0.5">Specify ranges</div>
                        </button>
                      </div>
                    </div>

                    <!-- Custom Port Input -->
                    {#if portScanProfile === "custom"}
                      <div class="space-y-1.5">
                        <label for="settings-custom-ports" class="block text-xs font-medium text-neutral-400 uppercase tracking-wider">
                          Custom Ports & Ranges (e.g. 21, 22, 80, 443, 3000-3005, 8080)
                        </label>
                        <input
                          id="settings-custom-ports"
                          type="text"
                          bind:value={customPortsInput}
                          placeholder="80, 443, 3000-3005, 8080, 8443"
                          class="w-full px-3 py-2 bg-[#191919] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs font-mono text-neutral-200 placeholder-neutral-500 focus:outline-none"
                        />
                      </div>
                    {/if}

                    <!-- Socket Timeout Per Probe -->
                    <div class="space-y-1.5">
                      <span class="block text-xs font-medium text-neutral-400 uppercase tracking-wider">
                        Port Probe Socket Timeout
                      </span>
                      <div class="grid grid-cols-3 gap-2">
                        {#each [{ label: "Fast (500ms)", ms: 500 }, { label: "Balanced (800ms)", ms: 800 }, { label: "Thorough (1500ms)", ms: 1500 }] as t}
                          <button
                            type="button"
                            onclick={() => (portTimeoutMs = t.ms)}
                            class="py-2 text-xs font-mono rounded-lg border transition-colors cursor-pointer {portTimeoutMs === t.ms ? 'bg-[#2a2a2a] text-white border-neutral-400 font-medium' : 'bg-[#191919] text-neutral-400 border-[#2e2e2e] hover:border-[#3a3a3a]'}"
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
                    <h3 class="text-sm font-semibold text-white">Continuous Security Watchdog</h3>
                    <p class="text-xs text-neutral-400 mt-0.5">Automated background re-scanning with score degradation notifications</p>
                  </div>
                  <span class="px-2 py-0.5 text-xs font-mono bg-[#191919] border border-[#2e2e2e] text-neutral-300 rounded">
                    {monitors.length} Monitored Domains
                  </span>
                </div>

                <!-- Add Watchdog Form -->
                <div class="p-4 bg-[#191919] border border-[#2e2e2e] rounded-xl space-y-3">
                  <div class="flex items-center gap-1.5 text-xs font-medium text-neutral-300">
                    <BellRing class="w-3.5 h-3.5 text-neutral-400" />
                    <span>Schedule New Domain Watchdog</span>
                  </div>
                  <div class="flex flex-col sm:flex-row items-center gap-2">
                    <input
                      type="text"
                      bind:value={newWatchdogUrl}
                      placeholder="https://example.com"
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
                      onclick={handleAddWatchdog}
                      disabled={!newWatchdogUrl.trim() || isAddingWatchdog}
                      class="w-full sm:w-auto px-3.5 py-1.5 bg-white hover:bg-neutral-200 disabled:opacity-50 text-neutral-950 font-semibold rounded-lg text-xs flex items-center justify-center gap-1.5 transition-colors cursor-pointer flex-shrink-0 shadow-sm"
                    >
                      <Plus class="w-3.5 h-3.5" />
                      <span>Add Target</span>
                    </button>
                  </div>
                </div>

                <!-- Configured Domains List -->
                <div class="space-y-2.5">
                  <div class="flex items-center justify-between">
                    <span class="text-xs font-medium text-neutral-400 uppercase tracking-wider">
                      Active Watchdog Fleet ({monitors.length})
                    </span>
                  </div>

                  {#if monitors.length === 0}
                    <div class="py-12 text-center text-neutral-500 bg-[#191919] border border-[#2e2e2e] rounded-xl">
                      <Activity class="w-8 h-8 mx-auto mb-2 opacity-30 text-neutral-400" />
                      <p class="text-xs font-medium text-neutral-300">No continuous monitors configured</p>
                      <p class="text-[11px] text-neutral-500 mt-0.5 max-w-sm mx-auto">
                        Add web properties above to track and alert on score changes and new CVEs.
                      </p>
                    </div>
                  {:else}
                    <div class="space-y-2 max-h-72 overflow-y-auto">
                      {#each monitors as item}
                        <div
                          class="p-3 bg-[#191919] border border-[#2e2e2e] rounded-lg flex items-center justify-between gap-3"
                        >
                          <div class="flex items-center gap-3 min-w-0 flex-1">
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
                                <span>Last: {formatWatchdogDate(item.last_scanned_at)}</span>
                                <span>•</span>
                                <span>Next: {formatWatchdogDate(item.next_scan_at)}</span>
                              </div>
                            </div>
                          </div>

                          <div class="flex items-center gap-1.5 flex-shrink-0">
                            {#if item.last_score !== null && item.last_score !== undefined}
                              <span
                                class="px-2 py-0.5 text-xs font-mono rounded border {getScoreBadge(item.last_score)}"
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
                                class="p-1.5 text-neutral-300 hover:text-white hover:bg-[#282828] rounded-md transition-colors cursor-pointer"
                                title="Run Audit Now"
                              >
                                <RotateCw class="w-3.5 h-3.5" />
                              </button>
                            {/if}

                            {#if onToggleMonitor}
                              <button
                                type="button"
                                onclick={() => onToggleMonitor?.(item.id)}
                                class="p-1.5 text-neutral-400 hover:text-white hover:bg-[#282828] rounded-md transition-colors cursor-pointer"
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
                                class="p-1.5 text-neutral-500 hover:text-red-400 rounded-md transition-colors cursor-pointer"
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
                  <h3 class="text-sm font-semibold text-white">Batch Fleet Security Scanner</h3>
                  <p class="text-xs text-neutral-400 mt-0.5">Audit multiple endpoints in sequence to evaluate entire domain portfolios</p>
                </div>

                {#if !isBatchRunning && batchItems.length === 0}
                  <div class="space-y-3.5">
                    <div>
                      <div class="flex items-center justify-between mb-1.5">
                        <label
                          for="settings-batch-urls"
                          class="block text-xs font-medium text-neutral-400 uppercase tracking-wider"
                        >
                          Target Inventory (One URL Per Line)
                        </label>
                        <span class="text-[11px] text-neutral-500 font-mono">Supports HTTP & HTTPS</span>
                      </div>
                      <textarea
                        id="settings-batch-urls"
                        bind:value={batchRawUrls}
                        rows="5"
                        placeholder="https://example.com&#10;https://api.example.com&#10;https://staging.example.com"
                        class="w-full p-3 bg-[#191919] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs font-mono text-neutral-200 placeholder-neutral-500 focus:outline-none"
                      ></textarea>
                    </div>

                    <div class="p-3.5 bg-[#191919] border border-[#2e2e2e] rounded-lg flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                      <div class="text-xs text-neutral-400">
                        <span class="font-medium text-neutral-200">Sequential Fleet Queue:</span>
                        Targets are scanned sequentially using active audit parameters.
                      </div>
                      <button
                        type="button"
                        onclick={startBatchScan}
                        class="px-4 py-1.5 bg-white hover:bg-neutral-200 text-neutral-950 font-semibold rounded-lg text-xs flex items-center gap-2 transition-colors cursor-pointer shadow-sm flex-shrink-0"
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
                        <h4 class="text-xs font-semibold text-white">
                          {isBatchRunning ? "Auditing Fleet Surface..." : "Fleet Assessment Completed"}
                        </h4>
                        <p class="text-xs text-neutral-400 font-mono mt-0.5">
                          Progress: <strong class="text-white">{batchCompletedCount}</strong> of {batchItems.length} audited
                        </p>
                      </div>

                      {#if !isBatchRunning}
                        <button
                          type="button"
                          onclick={() => (batchItems = [])}
                          class="px-3 py-1 bg-[#262626] hover:bg-[#303030] text-neutral-300 rounded-lg text-xs font-medium cursor-pointer transition-colors"
                        >
                          New Batch
                        </button>
                      {/if}
                    </div>

                    <!-- Progress Bar -->
                    <div class="w-full bg-[#161616] border border-[#2e2e2e] rounded-full h-2 overflow-hidden p-0.5">
                      <div
                        class="bg-blue-500 h-full rounded-full transition-all duration-300"
                        style="width: {batchItems.length > 0 ? (batchCompletedCount / batchItems.length) * 100 : 0}%"
                      ></div>
                    </div>

                    <!-- Results -->
                    <div class="space-y-2 max-h-60 overflow-y-auto">
                      {#each batchItems as item}
                        <div
                          class="p-2.5 bg-[#191919] border border-[#2e2e2e] rounded-lg flex items-center justify-between gap-3"
                        >
                          <div class="flex items-center gap-2.5 min-w-0 flex-1">
                            {#if item.status === "scanning"}
                              <Loader2 class="w-3.5 h-3.5 text-blue-400 animate-spin flex-shrink-0" />
                            {:else if item.status === "completed"}
                              <CheckCircle2 class="w-3.5 h-3.5 text-emerald-400 flex-shrink-0" />
                            {:else}
                              <AlertOctagon class="w-3.5 h-3.5 text-red-400 flex-shrink-0" />
                            {/if}

                            <div class="min-w-0 flex-1">
                              <div class="text-xs font-medium text-neutral-200 truncate font-mono">
                                {item.url}
                              </div>
                              {#if item.error}
                                <div class="text-[10px] text-red-400 font-mono mt-0.5 truncate">
                                  {item.error}
                                </div>
                              {:else if item.report}
                                <div class="text-[10px] text-neutral-400 font-mono mt-0.5 flex items-center gap-2">
                                  <span>{item.report.total_findings} findings</span>
                                  {#if item.report.critical_count > 0}
                                    <span class="text-red-400 font-medium">({item.report.critical_count} critical)</span>
                                  {/if}
                                </div>
                              {/if}
                            </div>
                          </div>

                          {#if item.report}
                            <div class="flex items-center gap-2 flex-shrink-0">
                              <span
                                class="px-2 py-0.5 text-xs font-mono rounded border {getScoreBadge(item.report.security_score)}"
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
                                  class="px-2 py-1 bg-[#262626] hover:bg-[#303030] text-neutral-200 border border-[#383838] rounded text-xs font-medium flex items-center gap-1 cursor-pointer transition-colors"
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
                  <h3 class="text-sm font-semibold text-white">Keyboard Shortcuts & Commands</h3>
                  <p class="text-xs text-neutral-400 mt-0.5">Quick hotkeys for high-velocity navigation and scanning</p>
                </div>

                <div class="p-3.5 bg-[#191919] border border-[#2e2e2e] rounded-xl divide-y divide-[#282828]">
                  {#each shortcutsList as sc}
                    <div class="flex items-center justify-between py-2 first:pt-0 last:pb-0">
                      <span class="text-xs text-neutral-300">{sc.description}</span>
                      <kbd class="px-2 py-0.5 bg-[#141414] border border-[#303030] rounded text-[11px] font-mono text-neutral-200 shadow-xs">
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
                  <h3 class="text-sm font-semibold text-white">Storage, Diagnostics & About</h3>
                  <p class="text-xs text-neutral-400 mt-0.5">Local SQLite database state and system build information</p>
                </div>

                <!-- Database Metrics -->
                <div class="grid grid-cols-2 gap-3">
                  <div class="p-3.5 bg-[#191919] border border-[#2e2e2e] rounded-xl space-y-1">
                    <span class="text-[11px] font-medium text-neutral-400 uppercase tracking-wider">Persisted Scans</span>
                    <div class="text-lg font-bold font-mono text-white flex items-center justify-between">
                      <span>{historyCount} Snapshots</span>
                      {#if onOpenHistory}
                        <button
                          type="button"
                          onclick={() => {
                            onClose();
                            onOpenHistory?.();
                          }}
                          class="text-xs text-neutral-300 hover:text-white font-normal underline cursor-pointer"
                        >
                          View Archive
                        </button>
                      {/if}
                    </div>
                  </div>

                  <div class="p-3.5 bg-[#191919] border border-[#2e2e2e] rounded-xl space-y-1">
                    <span class="text-[11px] font-medium text-neutral-400 uppercase tracking-wider">Watchdog Domains</span>
                    <div class="text-lg font-bold font-mono text-white">
                      {monitors.length} Configured
                    </div>
                  </div>
                </div>

                <!-- Clear History Action -->
                {#if onClearHistory && historyCount > 0}
                  <div class="p-4 bg-red-950/20 border border-red-900/30 rounded-xl flex items-center justify-between gap-4">
                    <div>
                      <div class="text-xs font-semibold text-red-300">Clear Local Scan Archive</div>
                      <p class="text-[11px] text-red-400/80 mt-0.5">Permanently delete all historical audit reports stored in the local SQLite database.</p>
                    </div>
                    <button
                      type="button"
                      onclick={onClearHistory}
                      class="px-3 py-1.5 bg-red-950 hover:bg-red-900 text-red-200 border border-red-800 rounded-lg text-xs font-medium cursor-pointer transition-colors flex-shrink-0"
                    >
                      Clear History
                    </button>
                  </div>
                {/if}

                <!-- About Card -->
                <div class="p-4 bg-[#191919] border border-[#2e2e2e] rounded-xl space-y-2.5">
                  <div class="flex items-center gap-2.5">
                    <div class="w-7 h-7 rounded-lg bg-[#282828] border border-[#383838] flex items-center justify-center text-neutral-300">
                      <ShieldCheck class="w-4 h-4 text-white" />
                    </div>
                    <div>
                      <div class="text-xs font-bold text-white">VulnRadar Desktop v0.6.1</div>
                      <div class="text-[11px] text-neutral-400 font-mono">Tauri v2 • Svelte 5 • Rust 2021 Engine</div>
                    </div>
                  </div>
                  <p class="text-xs text-neutral-400 leading-relaxed">
                    Lightweight passive reconnaissance, HTTP security posture verification, TLS cipher audit, DNS SPF/DMARC alignment, and asynchronous TCP port scanner.
                  </p>
                </div>
              </div>
            {/if}

          </div>

          <!-- Modal Footer (Shown for editable parameter tabs) -->
          {#if currentTab === "params" || currentTab === "ports"}
            <div class="px-5 py-3 border-t border-[#2e2e2e] flex items-center justify-between bg-[#191919]">
              <div class="text-[11px] text-neutral-400">
                Changes apply immediately to subsequent security audits.
              </div>
              <div class="flex items-center gap-2">
                <button
                  type="button"
                  onclick={onClose}
                  class="px-3 py-1.5 bg-[#252525] hover:bg-[#2e2e2e] text-neutral-300 rounded-lg text-xs font-medium cursor-pointer transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="button"
                  onclick={handleSaveParameters}
                  class="px-4 py-1.5 bg-white hover:bg-neutral-200 text-neutral-950 font-semibold rounded-lg text-xs transition-colors cursor-pointer shadow-sm"
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
