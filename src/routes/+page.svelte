<script lang="ts">
  import { onMount } from "svelte";
  import type {
    ScanReport,
    ScanSummary,
    ScanOptions,
    MonitorTarget,
  } from "$lib/types";
  import AppHeader from "$lib/components/AppHeader.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import AuditWorkspace from "$lib/components/AuditWorkspace.svelte";
  import PortMatrixWorkspace from "$lib/components/PortMatrixWorkspace.svelte";
  import DnsPostureWorkspace from "$lib/components/DnsPostureWorkspace.svelte";
  import PathAnalysisWorkspace from "$lib/components/PathAnalysisWorkspace.svelte";
  import FleetWorkspace from "$lib/components/FleetWorkspace.svelte";
  import WatchdogWorkspace from "$lib/components/WatchdogWorkspace.svelte";
  import HistoryWorkspace from "$lib/components/HistoryWorkspace.svelte";
  import SettingsWorkspace from "$lib/components/SettingsWorkspace.svelte";
  import ExportModal from "$lib/components/ExportModal.svelte";
  import ExecutiveReportModal from "$lib/components/ExecutiveReportModal.svelte";
  import ShortcutsModal from "$lib/components/ShortcutsModal.svelte";
  import UpdateModal from "$lib/components/UpdateModal.svelte";
  import Toast from "$lib/components/Toast.svelte";
  import {
    AlertOctagon,
    Bell,
    CheckCircle2,
    Loader2,
    X,
  } from "lucide-svelte";

  let targetUrl = $state("https://example.com");
  let isScanning = $state(false);
  let scanError = $state<string | null>(null);
  let report = $state<ScanReport | null>(null);
  let history = $state<ScanSummary[]>([]);
  let monitors = $state<MonitorTarget[]>([]);

  // Scan Configuration - Default port scan enabled with Top 20
  let scanOptions = $state<ScanOptions>({
    timeout_seconds: 15,
    include_subdomains: true,
    enable_port_scan: true,
    port_scan_profile: "top20",
    port_timeout_ms: 600,
  });

  // Active Workspace Navigation View
  let currentWorkspace = $state<"audit" | "ports" | "dns" | "paths" | "batch" | "watchdog" | "history" | "settings">("audit");

  // Modal States
  let settingsTab = $state<"params" | "ports" | "watchdog" | "wordlists" | "shortcuts" | "data" | "updates">("params");
  let isExportOpen = $state(false);
  let isExecutiveReportOpen = $state(false);
  let isShortcutsOpen = $state(false);
  let isUpdateOpen = $state(false);
  let hasUpdateAvailable = $state(false);
  let updateVersion = $state("");
  let updateModalRef: any = $state(null);
  let exportMarkdown = $state("");
  const currentAppVersion = "1.5.2";

  function handleCheckUpdates() {
    isUpdateOpen = true;
    if (updateModalRef) {
      updateModalRef.checkForUpdates(true);
    }
  }

  // Toast Notification System
  let toastMessage = $state("");
  let toastType = $state<"success" | "error" | "info">("info");
  let toastVisible = $state(false);
  let toastTimer: any = null;

  function showToast(msg: string, type: "success" | "error" | "info" = "info") {
    toastMessage = msg;
    toastType = type;
    toastVisible = true;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toastVisible = false;
    }, 2800);
  }

  // Watchdog Alert Banner
  let watchdogAlert = $state<{
    target_url: string;
    new_score: number;
    previous_score: number;
    critical_count: number;
  } | null>(null);

  async function invokeTauri<T>(
    cmd: string,
    args: Record<string, unknown> = {}
  ): Promise<T> {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      return await invoke<T>(cmd, args);
    } catch (e: any) {
      console.warn("Tauri invoke fallback / error:", cmd, e);
      throw e;
    }
  }

  async function loadHistory() {
    try {
      history = await invokeTauri<ScanSummary[]>("get_history");
    } catch {}
  }

  async function loadMonitors() {
    try {
      monitors = await invokeTauri<MonitorTarget[]>("get_monitors");
    } catch {}
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      isExportOpen = false;
      isExecutiveReportOpen = false;
      isShortcutsOpen = false;
      if (currentWorkspace === "settings" || currentWorkspace === "batch") {
        currentWorkspace = "audit";
      }
      return;
    }

    if (e.key === "/" && (document.activeElement?.tagName !== "INPUT" && document.activeElement?.tagName !== "TEXTAREA")) {
      e.preventDefault();
      const input = document.getElementById("finding-search") as HTMLInputElement | null;
      input?.focus();
      return;
    }

    if ((e.ctrlKey || e.metaKey) && e.key === ",") {
      e.preventDefault();
      currentWorkspace = "settings";
      return;
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      const input = document.getElementById("target-url-input") as HTMLInputElement | null;
      if (input) {
        input.focus();
        input.select();
      }
      return;
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "b") {
      e.preventDefault();
      currentWorkspace = "batch";
      return;
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "h") {
      e.preventDefault();
      currentWorkspace = "history";
      return;
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "m") {
      e.preventDefault();
      currentWorkspace = "watchdog";
      return;
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "e") {
      e.preventDefault();
      openExportModal();
      return;
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "p") {
      e.preventDefault();
      if (report) {
        isExecutiveReportOpen = true;
      }
      return;
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "o") {
      e.preventDefault();
      settingsTab = "params";
      currentWorkspace = "settings";
      return;
    }

    if (e.key === "?" && document.activeElement?.tagName !== "INPUT" && document.activeElement?.tagName !== "TEXTAREA") {
      e.preventDefault();
      isShortcutsOpen = true;
      return;
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
    loadHistory();
    loadMonitors();

    let unlistenWatchdog: (() => void) | null = null;
    (async () => {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlistenWatchdog = await listen<any>("watchdog_alert", (event) => {
          watchdogAlert = event.payload;
          loadMonitors();
          loadHistory();
          showToast(`Watchdog: Security alert on ${event.payload.target_url}`, "error");
        });
      } catch {}
    })();

    return () => {
      window.removeEventListener("keydown", handleKeydown);
      if (unlistenWatchdog) unlistenWatchdog();
      if (toastTimer) clearTimeout(toastTimer);
    };
  });

  async function handleScan(overrideUrl?: string) {
    const url = (overrideUrl || targetUrl).trim();
    if (!url) {
      showToast("Please enter a valid target URL or hostname", "error");
      return;
    }

    isScanning = true;
    scanError = null;

    try {
      const res = await invokeTauri<ScanReport>("scan_target", {
        targetUrl: url,
        url,
        options: scanOptions,
      });

      report = res;
      targetUrl = res.target_url;
      await loadHistory();

      if (res.findings.length === 0) {
        showToast("Audit completed: 0 security findings", "success");
      } else {
        const crit = res.findings.filter((f) => f.severity === "critical").length;
        const msg = `Audit completed: ${res.findings.length} findings (${crit} critical)`;
        showToast(msg, crit > 0 ? "error" : "success");
      }
    } catch (e: any) {
      const errStr = typeof e === "string" ? e : (e?.message || "Audit failed to execute.");
      scanError = errStr;
      showToast(errStr, "error");
    } finally {
      isScanning = false;
    }
  }

  async function handleSelectHistoryScan(scanId: string) {
    try {
      const fullReport = await invokeTauri<ScanReport>("get_scan_report", { id: scanId });
      report = fullReport;
      targetUrl = fullReport.target_url;
      currentWorkspace = "audit";
      showToast(`Loaded audit snapshot for ${fullReport.target_url}`, "success");
    } catch {
      showToast("Failed to load historical scan report", "error");
    }
  }

  async function handleDeleteScan(scanId: string) {
    try {
      await invokeTauri("delete_scan", { id: scanId });
      await loadHistory();
      if (report?.id === scanId) {
        report = null;
      }
      showToast("Scan report removed", "info");
    } catch {
      showToast("Failed to delete scan", "error");
    }
  }

  async function handleClearAllHistory() {
    try {
      await invokeTauri("clear_history");
      history = [];
      showToast("Scan history cleared", "info");
    } catch {
      showToast("Failed to clear history", "error");
    }
  }

  async function handleAddMonitor(url: string, intervalHours: number) {
    try {
      await invokeTauri("add_monitor", { url, intervalHours });
      await loadMonitors();
      showToast(`Added ${url} to continuous monitoring (${intervalHours}h schedule)`, "success");
    } catch {
      showToast("Failed to add target to watchdog", "error");
    }
  }

  async function handleDeleteMonitor(id: string) {
    try {
      await invokeTauri("delete_monitor", { id });
      await loadMonitors();
      showToast("Target removed from watchdog", "info");
    } catch {
      showToast("Failed to delete monitor", "error");
    }
  }

  async function handleToggleMonitor(id: string) {
    try {
      await invokeTauri("toggle_monitor", { id });
      await loadMonitors();
      showToast("Monitor status updated", "info");
    } catch {
      showToast("Failed to toggle monitor", "error");
    }
  }

  async function openExportModal() {
    if (!report) {
      showToast("Execute an audit before exporting findings", "info");
      return;
    }

    try {
      exportMarkdown = await invokeTauri<string>("export_report_markdown", {
        report,
      });
    } catch {
      exportMarkdown = "";
    }
    isExportOpen = true;
  }
</script>

<svelte:head>
  <title>VulnRadar — Enterprise Web Security Posture & Vulnerability Scanner</title>
</svelte:head>

<!-- Application Window Frame Container -->
<div class="h-screen w-screen flex flex-col overflow-hidden bg-surface-container-lowest text-on-surface">
  <AppHeader {currentWorkspace} currentVersion={currentAppVersion} />

  <!-- Watchdog Alert Banner -->
  {#if watchdogAlert}
    <div
      class="bg-error-container text-on-error-container border-b border-error/40 px-4 py-2 text-xs flex items-center justify-between gap-4 animate-fade-in flex-shrink-0 print:hidden"
    >
      <div class="flex items-center gap-2.5 min-w-0">
        <Bell class="w-4 h-4 text-error animate-bounce flex-shrink-0" />
        <span class="font-bold uppercase tracking-wider font-mono text-[10px]">Watchdog Alert:</span>
        <span class="truncate font-mono font-bold text-on-surface">{watchdogAlert.target_url}</span>
        <span class="hidden sm:inline text-xs">
          Score dropped from {watchdogAlert.previous_score} to {watchdogAlert.new_score} ({watchdogAlert.critical_count} critical issues detected)
        </span>
      </div>
      <div class="flex items-center gap-2 flex-shrink-0">
        <button
          type="button"
          onclick={() => {
            targetUrl = watchdogAlert!.target_url;
            handleScan(watchdogAlert!.target_url);
            watchdogAlert = null;
          }}
          class="px-2.5 py-1 bg-primary text-on-primary font-bold rounded text-xs cursor-pointer transition-all shadow-sm"
        >
          View Audit
        </button>
        <button
          type="button"
          onclick={() => (watchdogAlert = null)}
          class="p-1 text-outline hover:text-on-surface rounded cursor-pointer"
          aria-label="Dismiss alert"
        >
          <X class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  {/if}

  <!-- Main Body Layout: Vertical Sidebar + Active Workspace View -->
  <div class="flex-1 flex overflow-hidden">
    <Sidebar
      {currentWorkspace}
      activeMonitorsCount={monitors.filter((m) => m.is_active).length}
      historyCount={history.length}
      currentVersion={currentAppVersion}
      {hasUpdateAvailable}
      {updateVersion}
      {isScanning}
      onCheckUpdates={handleCheckUpdates}
      onSelectWorkspace={(ws) => {
        currentWorkspace = ws as any;
      }}
    />

    <!-- Main Workspace Area -->
    <main class="flex-1 w-full overflow-y-auto p-5 md:p-6 lg:p-8 bg-surface-container-lowest flex flex-col focus:outline-none">
      <!-- 1. ACTIVE SCAN HUD (WHEN SCANNING) -->
      {#if isScanning}
        <div class="my-auto max-w-lg mx-auto w-full p-8 bg-surface-container-low border border-surface-container-high rounded text-center space-y-6 shadow-xl animate-fade-in font-mono">
          <div class="w-14 h-14 mx-auto rounded-full bg-surface-container-highest flex items-center justify-center text-primary">
            <Loader2 class="w-7 h-7 animate-spin" />
          </div>

          <div class="space-y-2">
            <div class="inline-flex items-center gap-2 px-2.5 py-0.5 rounded bg-surface-container text-on-surface text-[10px] uppercase font-bold tracking-wider">
              <span class="w-1.5 h-1.5 rounded-full bg-primary animate-pulse"></span>
              <span>RUNNING MULTI-THREADED SECURITY AUDIT</span>
            </div>
            <h2 class="text-xl font-bold text-on-surface tracking-tight">Auditing Target Surface</h2>
            <div class="p-2 bg-surface-container-lowest rounded border border-surface-container-high text-xs text-secondary truncate max-w-sm mx-auto">
              {targetUrl}
            </div>
          </div>

          <!-- Active Pipeline Modules Checklist -->
          <div class="grid grid-cols-2 gap-2 text-left text-xs pt-3 border-t border-surface-container-high text-on-surface-variant">
            <div class="flex items-center gap-2">
              <CheckCircle2 class="w-3.5 h-3.5 text-tertiary" />
              <span>HTTP HEADERS</span>
            </div>
            <div class="flex items-center gap-2">
              <CheckCircle2 class="w-3.5 h-3.5 text-tertiary" />
              <span>PORT DISCOVERY</span>
            </div>
            <div class="flex items-center gap-2">
              <CheckCircle2 class="w-3.5 h-3.5 text-tertiary" />
              <span>DOH ANTI-SPOOF</span>
            </div>
            <div class="flex items-center gap-2">
              <CheckCircle2 class="w-3.5 h-3.5 text-tertiary" />
              <span>SCA CVE HEURISTICS</span>
            </div>
          </div>
        </div>

      <!-- 2. SCAN ERROR HUD -->
      {:else if scanError}
        <div class="my-auto max-w-xl mx-auto w-full p-6 bg-surface-container-low border border-error/40 rounded space-y-4 text-error animate-fade-in font-mono">
          <div class="flex items-start gap-3">
            <AlertOctagon class="w-6 h-6 text-error flex-shrink-0 mt-0.5" />
            <div class="space-y-1.5 flex-1">
              <h3 class="text-sm font-bold uppercase tracking-wider">Audit Execution Failed</h3>
              <p class="text-xs text-on-surface-variant bg-surface-container-lowest p-3 rounded border border-error/20 break-all leading-relaxed">
                {scanError}
              </p>
            </div>
          </div>
          <div class="flex items-center justify-end gap-2 pt-2 border-t border-surface-container-high">
            <button
              type="button"
              onclick={() => handleScan()}
              class="px-4 py-1.5 bg-primary text-on-primary rounded text-xs font-bold uppercase transition-opacity cursor-pointer hover:opacity-90"
            >
              Retry Audit
            </button>
          </div>
        </div>

      <!-- 3. WORKSPACE 01: POSTURE AUDIT DASHBOARD -->
      {:else if currentWorkspace === "audit"}
        <AuditWorkspace
          bind:targetUrl
          {report}
          {isScanning}
          bind:scanOptions
          onScan={(url) => {
            if (url) targetUrl = url;
            handleScan(url);
          }}
          onOpenExecutiveReport={() => (isExecutiveReportOpen = true)}
          onOpenExport={openExportModal}
          onOpenSettings={() => {
            settingsTab = "params";
            currentWorkspace = "settings";
          }}
          onResetBuffer={() => {
            report = null;
            scanError = null;
            showToast("Audit buffer reset", "info");
          }}
        />

      <!-- 4. WORKSPACE 02: PORT MATRIX & RECONNAISSANCE -->
      {:else if currentWorkspace === "ports"}
        <PortMatrixWorkspace
          {targetUrl}
          portReport={report?.port_report || null}
          options={scanOptions}
          onOptionsChange={(newOpts) => (scanOptions = newOpts)}
          onUpdateReport={(updatedPortReport) => {
            if (report) {
              report.port_report = updatedPortReport;
            }
          }}
        />

      <!-- 5. WORKSPACE 03: DNS SECURITY & ARCHITECTURE -->
      {:else if currentWorkspace === "dns"}
        <DnsPostureWorkspace
          {targetUrl}
          dnsReport={report?.dns_security || null}
          onUpdateReport={(updatedDnsReport) => {
            if (report) {
              report.dns_security = updatedDnsReport;
            }
          }}
        />

      <!-- 6. WORKSPACE 04: PATH RADAR & CONTENT DISCOVERY -->
      {:else if currentWorkspace === "paths"}
        <PathAnalysisWorkspace
          defaultUrl={targetUrl}
          onScanReport={(url) => {
            targetUrl = url;
            handleScan(url);
          }}
        />

      <!-- 7. WORKSPACE 05: FLEET RECONNAISSANCE & BATCH SCANNER -->
      {:else if currentWorkspace === "batch"}
        <FleetWorkspace
          options={scanOptions}
          onSelectReport={(batchReport) => {
            report = batchReport;
            targetUrl = batchReport.target_url;
            currentWorkspace = "audit";
          }}
          onClose={() => (currentWorkspace = "audit")}
        />

      <!-- 8. WORKSPACE 06: WATCHDOG CONTINUOUS MONITORING -->
      {:else if currentWorkspace === "watchdog"}
        <WatchdogWorkspace
          {monitors}
          onAddMonitor={handleAddMonitor}
          onDeleteMonitor={handleDeleteMonitor}
          onToggleMonitor={handleToggleMonitor}
          onScanTarget={(url) => {
            targetUrl = url;
            currentWorkspace = "audit";
            handleScan(url);
          }}
        />

      <!-- 9. WORKSPACE 07: LOCAL SQLITE LOGS / HISTORY -->
      {:else if currentWorkspace === "history"}
        <HistoryWorkspace
          {history}
          onSelectScan={handleSelectHistoryScan}
          onDeleteScan={handleDeleteScan}
          onClearHistory={handleClearAllHistory}
        />

      <!-- 10. WORKSPACE 08: SETTINGS & ENGINE CONFIGURATION -->
      {:else if currentWorkspace === "settings"}
        <SettingsWorkspace
          activeTab={settingsTab}
          options={scanOptions}
          {monitors}
          historyCount={history.length}
          {hasUpdateAvailable}
          {updateVersion}
          currentVersion={currentAppVersion}
          onCheckUpdates={handleCheckUpdates}
          onApplyOptions={(opts) => {
            scanOptions = opts;
            showToast("Scan configuration updated", "success");
          }}
          onAddMonitor={handleAddMonitor}
          onDeleteMonitor={handleDeleteMonitor}
          onToggleMonitor={handleToggleMonitor}
          onScanTarget={(url) => {
            targetUrl = url;
            currentWorkspace = "audit";
            handleScan(url);
          }}
          onClearHistory={handleClearAllHistory}
          onOpenHistory={() => {
            currentWorkspace = "history";
          }}
          onClose={() => (currentWorkspace = "audit")}
        />
      {/if}
    </main>
  </div>

  <!-- Telemetry Footer Status Bar -->
  <footer
    class="h-6 bg-surface-container-lowest border-t border-surface-container-high z-50 px-4 flex items-center justify-between font-mono text-xs text-on-surface-variant flex-shrink-0 select-none"
  >
    <div class="flex items-center gap-3">
      <span class="flex items-center gap-1.5 text-tertiary font-medium">
        <span class="w-1.5 h-1.5 rounded-full {isScanning ? 'bg-primary animate-ping' : 'bg-tertiary'}"></span>
        {isScanning ? "Scanning..." : "Ready"}
      </span>
      {#if report?.target_url || targetUrl}
        <span class="text-outline">/</span>
        <span class="truncate max-w-xs text-on-surface">
          {report?.target_url || targetUrl}
        </span>
      {/if}
      {#if report}
        <span class="text-outline">/</span>
        <span class="text-outline">{report.findings.length} findings</span>
        <span class="text-outline">/</span>
        <span class="text-outline">{report.response_time_ms}ms</span>
      {/if}
    </div>
    <div class="flex items-center gap-3 text-outline text-[11px]">
      <span>⌘K Target</span>
      <span>⌘B Fleet</span>
      <span>⌘O Options</span>
      <span>⌘, Settings</span>
      <button
        type="button"
        onclick={() => (isShortcutsOpen = true)}
        class="hover:text-on-surface cursor-pointer"
      >
        ? Help
      </button>
    </div>
  </footer>
</div>

<!-- Global Modals System -->
<ExportModal
  isOpen={isExportOpen}
  {report}
  markdownContent={exportMarkdown}
  onOpenExecutiveReport={() => {
    isExportOpen = false;
    isExecutiveReportOpen = true;
  }}
  onClose={() => (isExportOpen = false)}
/>

<ExecutiveReportModal
  isOpen={isExecutiveReportOpen}
  {report}
  onClose={() => (isExecutiveReportOpen = false)}
/>

<ShortcutsModal
  isOpen={isShortcutsOpen}
  onClose={() => (isShortcutsOpen = false)}
/>

<UpdateModal
  bind:this={updateModalRef}
  bind:isOpen={isUpdateOpen}
  currentVersion={currentAppVersion}
  onUpdateStatusChange={(hasUp, ver) => {
    hasUpdateAvailable = hasUp;
    if (ver) updateVersion = ver;
  }}
  onClose={() => (isUpdateOpen = false)}
/>

<!-- Toast Notifications -->
<Toast
  message={toastMessage}
  type={toastType}
  visible={toastVisible}
  onDismiss={() => (toastVisible = false)}
/>
