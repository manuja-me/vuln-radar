<script lang="ts">
  import { onMount } from "svelte";
  import type {
    ScanReport,
    ScanSummary,
    Severity,
    Category,
    ScanOptions,
    MonitorTarget,
  } from "$lib/types";
  import Navbar from "$lib/components/Navbar.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import FindingCard from "$lib/components/FindingCard.svelte";
  import PortMatrixWorkspace from "$lib/components/PortMatrixWorkspace.svelte";
  import DnsPostureWorkspace from "$lib/components/DnsPostureWorkspace.svelte";
  import PathAnalysisWorkspace from "$lib/components/PathAnalysisWorkspace.svelte";
  import ExportModal from "$lib/components/ExportModal.svelte";
  import ExecutiveReportModal from "$lib/components/ExecutiveReportModal.svelte";
  import BatchScanModal from "$lib/components/BatchScanModal.svelte";
  import MonitorModal from "$lib/components/MonitorModal.svelte";
  import ShortcutsModal from "$lib/components/ShortcutsModal.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";
  import Toast from "$lib/components/Toast.svelte";
  import {
    AlertOctagon,
    AlertTriangle,
    Search,
    Globe,
    ExternalLink,
    Filter,
    CheckCircle2,
    Activity,
    Bell,
    Loader2,
    X,
    Terminal,
    Check,
    TrendingUp,
    TrendingDown,
    Zap,
    HardDrive,
    ShieldAlert,
    ShieldCheck,
    FileText,
    FileDown,
    Database,
    Sliders,
  } from "lucide-svelte";
  import { WORDLIST_PRESETS } from "$lib/wordlists";

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
  let isSettingsOpen = $state(false);
  let settingsTab = $state<"params" | "ports" | "watchdog" | "batch" | "wordlists" | "shortcuts" | "data">("params");
  let isExportOpen = $state(false);
  let isExecutiveReportOpen = $state(false);
  let isBatchOpen = $state(false);
  let isMonitorsOpen = $state(false);
  let isShortcutsOpen = $state(false);
  let exportMarkdown = $state("");

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

  // Filters & Sorting
  let searchQuery = $state("");
  let selectedSeverity = $state<Severity | "all">("all");
  let selectedCategory = $state<Category | "all">("all");
  let sortFindingsBy = $state<"severity" | "title" | "category">("severity");
  let copiedCurl = $state(false);
  let copiedUrl = $state(false);

  const hasCustomOptions = $derived(
    !!(
      (scanOptions.custom_headers && scanOptions.custom_headers.length > 0) ||
      scanOptions.user_agent ||
      (scanOptions.timeout_seconds && scanOptions.timeout_seconds !== 15) ||
      scanOptions.include_subdomains === false ||
      scanOptions.enable_port_scan === false ||
      (scanOptions.port_scan_profile && scanOptions.port_scan_profile !== "top20") ||
      (scanOptions.custom_ports && scanOptions.custom_ports.trim().length > 0)
    )
  );

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
      isSettingsOpen = false;
      isExportOpen = false;
      isExecutiveReportOpen = false;
      isBatchOpen = false;
      isMonitorsOpen = false;
      isShortcutsOpen = false;
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
      isSettingsOpen = true;
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
      isBatchOpen = true;
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
      isSettingsOpen = true;
      return;
    }

    if (e.key === "?" && document.activeElement?.tagName !== "INPUT" && document.activeElement?.tagName !== "TEXTAREA") {
      e.preventDefault();
      isShortcutsOpen = true;
      return;
    }
  }

  onMount(() => {
    loadHistory();
    loadMonitors();

    window.addEventListener("keydown", handleKeydown);

    let unlistenWatchdog: (() => void) | undefined;
    let unlistenMonitor: (() => void) | undefined;
    import("@tauri-apps/api/event")
      .then(({ listen }) => {
        const onAlert = (event: { payload: { target_url: string; new_score: number; previous_score: number; critical_count: number } }) => {
          watchdogAlert = event.payload;
          loadMonitors();
          loadHistory();
        };
        listen<{
          target_url: string;
          new_score: number;
          previous_score: number;
          critical_count: number;
        }>("watchdog_alert", onAlert).then((unsub) => {
          unlistenWatchdog = unsub;
        });
        listen<{
          target_url: string;
          new_score: number;
          previous_score: number;
          critical_count: number;
        }>("monitor_alert", onAlert).then((unsub) => {
          unlistenMonitor = unsub;
        });
      })
      .catch(() => {});

    return () => {
      window.removeEventListener("keydown", handleKeydown);
      if (unlistenWatchdog) unlistenWatchdog();
      if (unlistenMonitor) unlistenMonitor();
    };
  });

  function ensureReport(url?: string): ScanReport {
    if (!report) {
      report = {
        id: "temp-" + Date.now(),
        target_url: url || targetUrl,
        scanned_at: new Date().toISOString(),
        status_code: 200,
        response_time_ms: 0,
        security_score: 100,
        total_findings: 0,
        critical_count: 0,
        high_count: 0,
        medium_count: 0,
        low_count: 0,
        info_count: 0,
        findings: [],
        technologies_detected: [],
        response_headers: [],
        port_report: null,
        dns_security: null,
        endpoint_report: null,
      };
    }
    return report;
  }

  async function handleScan(urlToScan?: string) {
    const url = (urlToScan || targetUrl).trim();
    if (!url || isScanning) return;

    isScanning = true;
    scanError = null;
    currentWorkspace = "audit";

    try {
      const res = await invokeTauri<ScanReport>("scan_target", {
        url,
        options: scanOptions,
      });

      report = res;
      targetUrl = res.target_url;
      await loadHistory();
      showToast(`Audit completed for ${res.target_url}`, "success");
    } catch (e: any) {
      scanError = typeof e === "string" ? e : (e?.message || "Scan failed unexpectedly.");
      showToast("Security audit failed. Check target URL and connection.", "error");
    } finally {
      isScanning = false;
    }
  }

  async function handleSelectHistoryScan(scanId: string) {
    try {
      const res = await invokeTauri<ScanReport | null>("get_scan_report", {
        id: scanId,
      });
      if (res) {
        report = res;
        targetUrl = res.target_url;
        currentWorkspace = "audit";
        showToast(`Loaded audit report for ${res.target_url}`, "info");
      }
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
    if (!report) return;
    try {
      exportMarkdown = await invokeTauri<string>("export_report_markdown", {
        report,
      });
    } catch {
      exportMarkdown = `# Security Audit Report for ${report.target_url}\nScore: ${report.security_score}/100`;
    }
    isExportOpen = true;
  }

  async function copyCurlCommand() {
    const url = report?.target_url || targetUrl;
    try {
      const curlCmd = `curl -i -s -k -L -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) VulnRadar/2.0" "${url}"`;
      await navigator.clipboard.writeText(curlCmd);
      copiedCurl = true;
      showToast("cURL probe copied to clipboard", "success");
      setTimeout(() => (copiedCurl = false), 2000);
    } catch {}
  }

  const previousScan = $derived.by(() => {
    if (!report || history.length === 0) return null;
    const cleanCurrent = report.target_url.replace(/\/$/, "").toLowerCase();
    return history.find((h) => h.id !== report?.id && h.target_url.replace(/\/$/, "").toLowerCase() === cleanCurrent) || null;
  });

  const severityWeights: Record<Severity, number> = {
    critical: 5,
    high: 4,
    medium: 3,
    low: 2,
    info: 1,
  };

  const filteredFindings = $derived.by(() => {
    if (!report) return [];
    let list = report.findings.filter((finding) => {
      if (selectedSeverity !== "all" && finding.severity !== selectedSeverity) {
        return false;
      }
      if (
        selectedCategory !== "all" &&
        finding.category !== selectedCategory
      ) {
        return false;
      }
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        const matchesTitle = finding.title.toLowerCase().includes(q);
        const matchesDesc = finding.description.toLowerCase().includes(q);
        const matchesOwasp = finding.owasp_category.toLowerCase().includes(q);
        const matchesCve = finding.cve_id?.toLowerCase().includes(q) || false;
        return matchesTitle || matchesDesc || matchesOwasp || matchesCve;
      }
      return true;
    });

    if (sortFindingsBy === "severity") {
      list.sort((a, b) => severityWeights[b.severity] - severityWeights[a.severity]);
    } else if (sortFindingsBy === "title") {
      list.sort((a, b) => a.title.localeCompare(b.title));
    } else if (sortFindingsBy === "category") {
      list.sort((a, b) => a.category.localeCompare(b.category));
    }
    return list;
  });

  // Posture Score & Metrics derivation
  const postureScore = $derived(report ? report.security_score : null);

  const scoreGrade = $derived.by(() => {
    if (postureScore === null) return "";
    if (postureScore >= 95) return "A+";
    if (postureScore >= 90) return "A";
    if (postureScore >= 80) return "B+";
    if (postureScore >= 70) return "B";
    if (postureScore >= 60) return "C";
    if (postureScore >= 50) return "D";
    return "F";
  });

  const criticalCount = $derived(report ? report.findings.filter((f) => f.severity === "critical").length : 0);
  const highCount = $derived(report ? report.findings.filter((f) => f.severity === "high").length : 0);
  const medCount = $derived(report ? report.findings.filter((f) => f.severity === "medium").length : 0);
  const lowCount = $derived(report ? report.findings.filter((f) => f.severity === "low").length : 0);
  const infoCount = $derived(report ? report.findings.filter((f) => f.severity === "info").length : 0);
</script>

<svelte:head>
  <title>VulnRadar — Enterprise Web Security Posture & Vulnerability Scanner</title>
</svelte:head>

<!-- Application Window Frame Container -->
<div class="h-screen w-screen flex flex-col overflow-hidden bg-surface-container-lowest text-on-surface">
  <!-- Native Desktop Window Header Toolbar -->
  <Navbar
    {isScanning}
    hasReport={!!report}
    {hasCustomOptions}
    activeMonitorsCount={monitors.filter((m) => m.is_active).length}
    onOpenSettings={(tab) => {
      if (tab) {
        settingsTab = tab;
      }
      isSettingsOpen = true;
    }}
    onOpenExport={openExportModal}
    onOpenShortcuts={() => (isShortcutsOpen = true)}
  />

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
      onSelectWorkspace={(ws) => {
        if (ws === "settings") {
          isSettingsOpen = true;
        } else if (ws === "batch") {
          isBatchOpen = true;
        } else {
          currentWorkspace = ws as any;
        }
      }}
    />

    <!-- Main Workspace Area -->
    <main class="flex-1 w-full overflow-y-auto p-5 md:p-6 lg:p-8 bg-surface-container-lowest flex flex-col focus:outline-none">
    <!-- 1. ACTIVE SCAN HUD (WHEN SCANNING) -->
      {#if isScanning}
        <div class="my-auto max-w-lg mx-auto w-full p-8 bg-surface-container-low border border-surface-container-high rounded text-center space-y-6 shadow-xl animate-fade-in">
          <div class="w-14 h-14 mx-auto rounded-full bg-surface-container-highest flex items-center justify-center text-primary">
            <Loader2 class="w-7 h-7 animate-spin" />
          </div>

          <div class="space-y-2">
            <div class="inline-flex items-center gap-2 px-2.5 py-0.5 rounded bg-surface-container text-on-surface font-label-sm uppercase font-semibold">
              <span class="w-1.5 h-1.5 rounded-full bg-primary animate-pulse"></span>
              <span>RUNNING MULTI-THREADED SECURITY AUDIT</span>
            </div>
            <h2 class="text-xl font-bold text-on-surface tracking-tight">Auditing Target Surface</h2>
            <div class="p-2 bg-surface-container-lowest rounded border border-surface-container-high font-code-inline text-xs text-secondary truncate max-w-sm mx-auto">
              {targetUrl}
            </div>
          </div>

          <!-- Active Pipeline Modules Checklist -->
          <div class="grid grid-cols-2 gap-2 text-left font-label-sm pt-3 border-t border-surface-container-high">
            <div class="flex items-center gap-2 text-on-surface-variant">
              <CheckCircle2 class="w-3.5 h-3.5 text-tertiary" />
              <span>HTTP HEADERS</span>
            </div>
            <div class="flex items-center gap-2 text-on-surface-variant">
              <CheckCircle2 class="w-3.5 h-3.5 text-tertiary" />
              <span>PORT DISCOVERY</span>
            </div>
            <div class="flex items-center gap-2 text-on-surface-variant">
              <CheckCircle2 class="w-3.5 h-3.5 text-tertiary" />
              <span>DOH ANTI-SPOOF</span>
            </div>
            <div class="flex items-center gap-2 text-on-surface-variant">
              <CheckCircle2 class="w-3.5 h-3.5 text-tertiary" />
              <span>SCA CVE HEURISTICS</span>
            </div>
          </div>
        </div>

      <!-- 2. SCAN ERROR HUD -->
      {:else if scanError}
        <div class="my-auto max-w-xl mx-auto w-full p-6 bg-surface-container-low border border-error/40 rounded space-y-4 text-error animate-fade-in">
          <div class="flex items-start gap-3">
            <AlertOctagon class="w-6 h-6 text-error flex-shrink-0 mt-0.5" />
            <div class="space-y-1.5 flex-1">
              <h3 class="text-sm font-bold uppercase font-mono tracking-wider">Audit Execution Failed</h3>
              <p class="text-xs text-on-surface-variant font-code-inline bg-surface-container-lowest p-3 rounded border border-error/20 break-all leading-relaxed">
                {scanError}
              </p>
            </div>
          </div>
          <div class="flex items-center justify-end gap-2 pt-2 border-t border-surface-container-high">
            <button
              type="button"
              onclick={() => handleScan()}
              class="px-4 py-1.5 bg-primary text-on-primary rounded text-xs font-mono font-bold uppercase transition-opacity cursor-pointer hover:opacity-90"
            >
              Retry Audit
            </button>
          </div>
        </div>

      <!-- 3. WORKSPACE 01: POSTURE AUDIT DASHBOARD -->
      {:else if currentWorkspace === "audit"}
        <div class="flex flex-col w-full gap-6 max-w-7xl mx-auto pb-14">
          <!-- TOP COMMAND & TARGETING BAR -->
          <section class="bg-surface-container-low p-5 sm:p-6 rounded-lg flex flex-col gap-4 border border-surface-container-high shadow-sm">
            <div class="flex flex-wrap items-center justify-between gap-3 sm:gap-4">
              <!-- Target Input Group -->
              <div class="flex items-center flex-1 min-w-[200px] sm:min-w-[280px] h-10 bg-surface-container-lowest px-3.5 rounded border border-surface-container-high focus-within:border-outline transition-colors">
                <span class="px-2 py-0.5 rounded font-mono text-[10px] bg-tertiary/10 text-tertiary tracking-wider font-bold mr-2.5">
                  HTTPS
                </span>
                <div class="flex items-center flex-1 min-w-0 font-mono text-xs text-on-surface">
                  <input
                    id="target-url-input"
                    class="bg-transparent border-0 outline-none w-full text-on-surface focus:text-secondary selection:bg-surface-container-highest"
                    spellcheck="false"
                    type="text"
                    placeholder="Enter target URL or IP..."
                    bind:value={targetUrl}
                    onkeydown={(e) => {
                      if (e.key === "Enter") handleScan();
                    }}
                  />
                </div>
                <div class="flex items-center gap-1.5 pl-2 text-outline">
                  <span class="w-1.5 h-1.5 rounded-full {report ? 'bg-tertiary' : 'bg-outline'}"></span>
                  <span class="font-mono text-xs text-on-surface-variant font-medium">
                    {report ? `${report.response_time_ms}ms` : "—"}
                  </span>
                </div>
              </div>

              <!-- Action Exec Group -->
              <div class="flex items-center gap-2.5">
                <!-- Main Audit Action Button -->
                <button
                  id="btn-audit"
                  type="button"
                  disabled={isScanning}
                  onclick={() => handleScan()}
                  class="h-10 flex items-center gap-2.5 bg-on-surface text-surface-container-lowest px-5 rounded font-mono text-xs uppercase tracking-wider font-bold hover:bg-surface-bright hover:text-on-surface transition-all active:scale-[0.98] cursor-pointer disabled:opacity-50 whitespace-nowrap shadow-sm"
                >
                  <span class="relative flex h-2 w-2">
                    <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary-container opacity-75"></span>
                    <span class="relative inline-flex rounded-full h-2 w-2 bg-primary"></span>
                  </span>
                  <span>AUDIT TARGET</span>
                  {#if report}
                    <span class="font-mono text-[11px] bg-surface-container-highest text-on-surface px-1.5 py-0.5 rounded font-medium ml-1">
                      {(report.response_time_ms / 1000).toFixed(2)}s
                    </span>
                  {/if}
                </button>

                <!-- Command Utility Icons -->
                <div class="h-10 flex items-center bg-surface-container-lowest rounded border border-surface-container-high px-1 gap-1">
                  <button
                    type="button"
                    onclick={() => (isExecutiveReportOpen = true)}
                    class="w-8 h-8 flex items-center justify-center text-on-surface-variant hover:text-on-surface hover:bg-surface-container rounded transition-colors cursor-pointer"
                    title="Executive Audit Report (PDF)"
                  >
                    <FileDown class="w-3.5 h-3.5" />
                  </button>
                  <button
                    type="button"
                    onclick={openExportModal}
                    class="w-8 h-8 flex items-center justify-center text-on-surface-variant hover:text-on-surface hover:bg-surface-container rounded transition-colors cursor-pointer"
                    title="Raw JSON Findings Export"
                  >
                    <Database class="w-3.5 h-3.5" />
                  </button>
                  <button
                    type="button"
                    onclick={copyCurlCommand}
                    class="w-8 h-8 flex items-center justify-center text-on-surface-variant hover:text-secondary hover:bg-surface-container rounded transition-colors cursor-pointer"
                    title="Copy Target cURL probe"
                  >
                    {#if copiedCurl}
                      <Check class="w-3.5 h-3.5 text-tertiary" />
                    {:else}
                      <Terminal class="w-3.5 h-3.5" />
                    {/if}
                  </button>
                  <button
                    type="button"
                    onclick={() => {
                      report = null;
                      scanError = null;
                      showToast("Audit buffer reset", "info");
                    }}
                    class="w-8 h-8 flex items-center justify-center text-on-surface-variant hover:text-error hover:bg-surface-container rounded transition-colors cursor-pointer"
                    title="Reset Audit Buffer"
                  >
                    <X class="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            </div>

            <!-- Quick Target Scope Selector Pills -->
            <div class="flex items-center gap-2 flex-wrap font-mono text-[11px] pt-1">
              <span class="uppercase tracking-wider text-outline text-[10px] mr-1">Scope Presets:</span>
              {#each [
                { host: "example.com", url: "https://example.com" },
                { host: "httpbin.org", url: "https://httpbin.org" },
                { host: "testphp.vulnweb.com", url: "http://testphp.vulnweb.com" },
                { host: "localhost:8000", url: "http://localhost:8000" }
              ] as preset}
                <button
                  type="button"
                  class="px-2.5 py-1 bg-surface-container-lowest hover:bg-surface-container text-on-surface-variant rounded transition-colors text-[11px] font-mono border border-surface-container-high cursor-pointer {targetUrl === preset.url ? 'border-primary text-primary font-medium' : ''}"
                  onclick={() => {
                    targetUrl = preset.url;
                    handleScan(preset.url);
                  }}
                >
                  {preset.host}
                </button>
              {/each}
            </div>

            <!-- Quick Options Bar -->
            <div class="flex items-center justify-between gap-4 pt-3.5 border-t border-surface-container-high/60 flex-wrap text-xs font-mono">
              <div class="flex items-center gap-5 sm:gap-6 flex-wrap">
                <!-- Subdomains -->
                <label class="flex items-center gap-2 cursor-pointer text-on-surface-variant hover:text-on-surface select-none group">
                  <input
                    type="checkbox"
                    bind:checked={scanOptions.include_subdomains}
                    class="sr-only"
                  />
                  <div class="w-4 h-4 rounded flex items-center justify-center border transition-colors {scanOptions.include_subdomains ? 'bg-primary border-primary text-on-primary' : 'bg-surface-container-lowest border-surface-container-high group-hover:border-outline'}">
                    {#if scanOptions.include_subdomains}
                      <Check class="w-2.5 h-2.5 stroke-[3]" />
                    {/if}
                  </div>
                  <span>Subdomains</span>
                </label>

                <!-- Port Scan Toggle & Profile -->
                <div class="flex items-center gap-2">
                  <label class="flex items-center gap-2 cursor-pointer text-on-surface-variant hover:text-on-surface select-none group">
                    <input
                      type="checkbox"
                      bind:checked={scanOptions.enable_port_scan}
                      class="sr-only"
                    />
                    <div class="w-4 h-4 rounded flex items-center justify-center border transition-colors {scanOptions.enable_port_scan ? 'bg-primary border-primary text-on-primary' : 'bg-surface-container-lowest border-surface-container-high group-hover:border-outline'}">
                      {#if scanOptions.enable_port_scan}
                        <Check class="w-2.5 h-2.5 stroke-[3]" />
                      {/if}
                    </div>
                    <span>Port Recon</span>
                  </label>
                  {#if scanOptions.enable_port_scan}
                    <select
                      bind:value={scanOptions.port_scan_profile}
                      class="bg-surface-container-lowest text-on-surface border border-surface-container-high rounded px-2 py-1 text-xs outline-none ml-1 cursor-pointer"
                    >
                      <option value="top20">Top 20</option>
                      <option value="top100">Top 100</option>
                      <option value="databases">Databases</option>
                      <option value="custom">Custom</option>
                    </select>
                  {/if}
                </div>

                <!-- Timeout -->
                <div class="flex items-center gap-1.5 text-on-surface-variant">
                  <span class="text-outline">Timeout:</span>
                  <select
                    bind:value={scanOptions.timeout_seconds}
                    class="bg-surface-container-lowest text-on-surface border border-surface-container-high rounded px-2 py-1 text-xs outline-none cursor-pointer"
                  >
                    <option value={5}>5s</option>
                    <option value={10}>10s</option>
                    <option value={15}>15s</option>
                    <option value={30}>30s</option>
                  </select>
                </div>
              </div>

              <!-- Open Settings -->
              <button
                type="button"
                onclick={() => {
                  settingsTab = "params";
                  isSettingsOpen = true;
                }}
                class="flex items-center gap-1.5 text-outline hover:text-on-surface transition-colors cursor-pointer text-xs"
              >
                <Sliders class="w-3.5 h-3.5" />
                <span>Advanced Config (⌘O)</span>
              </button>
            </div>
          </section>

          <!-- POSTURE SCORE & SEVERITY GAUGES SECTION -->
          <section class="grid grid-cols-1 lg:grid-cols-12 gap-4 lg:gap-5 items-stretch">
            <!-- Hero Score Indicator Card -->
            <div class="lg:col-span-4 bg-surface-container-low p-5 sm:p-6 rounded-lg flex flex-col justify-between relative overflow-hidden border border-surface-container-high shadow-xs">
              <div class="absolute -right-10 -bottom-10 w-44 h-44 bg-tertiary/5 rounded-full pointer-events-none blur-2xl"></div>
              <div class="flex items-start justify-between">
                <span class="font-mono text-xs uppercase tracking-wider text-outline font-semibold">Security Posture Score</span>
                {#if postureScore !== null && scoreGrade}
                  <div class="px-2.5 py-0.5 rounded font-mono text-xs font-bold {postureScore >= 80 ? 'bg-tertiary-container/30 text-tertiary' : postureScore >= 60 ? 'bg-primary/20 text-primary' : 'bg-error/20 text-error'}">
                    Grade {scoreGrade}
                  </div>
                {/if}
              </div>

              <!-- Center Metric Hero -->
              <div class="my-4 sm:my-5 flex items-baseline gap-1.5">
                {#if postureScore !== null}
                  <span class="text-4xl sm:text-5xl leading-none font-extrabold text-on-surface tracking-tight font-mono">
                    {postureScore}
                  </span>
                {:else}
                  <span class="text-3xl sm:text-4xl leading-none font-normal text-on-surface-variant font-mono">
                    —
                  </span>
                {/if}
                <span class="font-code-inline text-base text-outline">/100</span>
              </div>

              <!-- Segmented Calibrated Bar Metric (10 ticks) -->
              <div class="flex flex-col gap-2 w-full">
                <div class="flex items-center justify-between font-code-inline text-[11px] text-outline">
                  <span>0</span>
                  <span>50</span>
                  <span class="text-tertiary">100</span>
                </div>
                <div class="grid grid-cols-10 gap-1.5 w-full h-2.5">
                  {#each [10, 20, 30, 40, 50, 60, 70, 80, 90, 100] as tick, i}
                    {@const active = postureScore !== null && postureScore >= tick}
                    {@const col = i < 2 ? "bg-error/80" : i < 4 ? "bg-primary-container/80" : i < 6 ? "bg-primary/80" : i < 8 ? "bg-secondary/80" : "bg-tertiary"}
                    <div class="{active ? col : 'bg-surface-container-highest'} rounded-xs"></div>
                  {/each}
                </div>
              </div>
            </div>

            <!-- Severity Deduction Metric Cards (Responsive 5-column grid) -->
            <div class="lg:col-span-8 grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-5 gap-3 sm:gap-3.5 items-stretch">
              <!-- Critical -->
              <div class="bg-surface-container-low p-4 rounded-lg flex flex-col justify-between border border-surface-container-high shadow-xs">
                <div class="flex items-center justify-between font-label-sm text-outline">
                  <span class="font-semibold tracking-wider text-[11px]">CRITICAL</span>
                  <span class="font-code-inline text-[11px] {criticalCount > 0 ? 'text-error' : 'text-tertiary'}">
                    {criticalCount > 0 ? `-${criticalCount * 25} PTS` : "0 PTS"}
                  </span>
                </div>
                <div class="my-3">
                  <span class="text-2xl font-bold font-mono {criticalCount > 0 ? 'text-error' : 'text-on-surface'}">
                    {criticalCount}
                  </span>
                </div>
                <div class="text-[11px] font-label-sm flex items-center gap-1.5 {criticalCount > 0 ? 'text-error' : 'text-outline'}">
                  <span class="w-1.5 h-1.5 rounded-full {criticalCount > 0 ? 'bg-error animate-ping' : 'bg-outline'}"></span>
                  <span>{criticalCount > 0 ? "Action Required" : "All Clear"}</span>
                </div>
              </div>

              <!-- High -->
              <div class="bg-surface-container-low p-4 rounded-lg flex flex-col justify-between relative overflow-hidden border border-surface-container-high shadow-xs">
                <div class="absolute top-0 left-0 w-full h-0.5 bg-primary-container"></div>
                <div class="flex items-center justify-between font-label-sm text-primary-container">
                  <span class="font-semibold tracking-wider text-[11px]">HIGH</span>
                  <span class="font-code-inline text-[11px] font-medium">{highCount > 0 ? `-${highCount * 8} PTS` : "0 PTS"}</span>
                </div>
                <div class="my-3">
                  <span class="text-2xl font-bold text-primary-container font-mono">{highCount}</span>
                </div>
                <div class="text-[11px] font-label-sm text-primary-container flex items-center gap-1.5 font-medium">
                  <span class="w-1.5 h-1.5 rounded-full {highCount > 0 ? 'bg-primary-container animate-pulse' : 'bg-outline'}"></span>
                  <span>{highCount > 0 ? "Action Required" : "All Clear"}</span>
                </div>
              </div>

              <!-- Medium -->
              <div class="bg-surface-container-low p-4 rounded-lg flex flex-col justify-between relative overflow-hidden border border-surface-container-high shadow-xs">
                <div class="absolute top-0 left-0 w-full h-0.5 bg-primary"></div>
                <div class="flex items-center justify-between font-label-sm text-primary">
                  <span class="font-semibold tracking-wider text-[11px]">MEDIUM</span>
                  <span class="font-code-inline text-[11px] font-medium">{medCount > 0 ? `-${medCount * 4} PTS` : "0 PTS"}</span>
                </div>
                <div class="my-3">
                  <span class="text-2xl font-bold text-primary font-mono">{medCount}</span>
                </div>
                <div class="text-[11px] font-label-sm text-primary flex items-center gap-1.5">
                  <span class="w-1.5 h-1.5 rounded-full {medCount > 0 ? 'bg-primary' : 'bg-outline'}"></span>
                  <span>{medCount > 0 ? "Attention" : "All Clear"}</span>
                </div>
              </div>

              <!-- Low -->
              <div class="bg-surface-container-low p-4 rounded-lg flex flex-col justify-between relative overflow-hidden border border-surface-container-high shadow-xs">
                <div class="absolute top-0 left-0 w-full h-0.5 bg-secondary"></div>
                <div class="flex items-center justify-between font-label-sm text-secondary">
                  <span class="font-semibold tracking-wider text-[11px]">LOW</span>
                  <span class="font-code-inline text-[11px] font-medium">{lowCount > 0 ? `-${lowCount * 1} PTS` : "0 PTS"}</span>
                </div>
                <div class="my-3">
                  <span class="text-2xl font-bold text-secondary font-mono">{lowCount}</span>
                </div>
                <div class="text-[11px] font-label-sm text-secondary flex items-center gap-1.5">
                  <span class="w-1.5 h-1.5 rounded-full {lowCount > 0 ? 'bg-secondary' : 'bg-outline'}"></span>
                  <span>{lowCount > 0 ? "Info" : "All Clear"}</span>
                </div>
              </div>

              <!-- Info -->
              <div class="bg-surface-container-low p-4 rounded-lg flex flex-col justify-between border border-surface-container-high shadow-xs">
                <div class="flex items-center justify-between font-label-sm text-outline">
                  <span class="font-semibold tracking-wider text-[11px]">INFO</span>
                  <span class="font-code-inline text-[11px]">+0 PTS</span>
                </div>
                <div class="my-3">
                  <span class="text-2xl font-bold text-on-surface-variant font-mono">{infoCount}</span>
                </div>
                <div class="text-[11px] font-label-sm text-outline flex items-center gap-1.5">
                  <span class="w-1.5 h-1.5 rounded-full bg-outline"></span>
                  <span>Telemetry</span>
                </div>
              </div>
            </div>
          </section>

          <!-- INTERACTIVE AUDIT FINDINGS INSPECTOR -->
          <section class="flex flex-col gap-4">
            <!-- Filter Toolbar Controls -->
            <div class="bg-surface-container-low p-3 sm:p-3.5 rounded-lg flex flex-wrap items-center justify-between gap-3 border border-surface-container-high shadow-xs">
              <!-- Monospace Search Query Input -->
              <div class="h-9.5 flex items-center gap-2.5 bg-surface-container-lowest px-3.5 rounded min-w-[240px] flex-1 border border-surface-container-high focus-within:border-outline transition-colors">
                <Search class="w-3.5 h-3.5 text-outline flex-shrink-0" />
                <input
                  id="finding-search"
                  bind:value={searchQuery}
                  placeholder="Filter by CVE, header, endpoint, vector..."
                  type="text"
                  class="bg-transparent border-0 outline-none font-mono text-xs text-on-surface w-full placeholder:text-outline"
                />
                <span class="font-mono text-[10px] text-outline bg-surface-container px-1.5 py-0.5 rounded">/</span>
              </div>

              <!-- Severity Filter Buttons -->
              <div class="flex items-center gap-1.5 font-mono text-xs flex-wrap">
                <button
                  type="button"
                  onclick={() => (selectedSeverity = "all")}
                  class="h-9 px-3.5 rounded transition-colors cursor-pointer {selectedSeverity === 'all' ? 'bg-surface-container-highest text-on-surface font-bold' : 'bg-surface-container-lowest hover:bg-surface-container text-outline'}"
                >
                  All ({report?.findings.length || 0})
                </button>
                <button
                  type="button"
                  onclick={() => (selectedSeverity = "high")}
                  class="h-9 px-3.5 rounded transition-colors cursor-pointer {selectedSeverity === 'high' ? 'bg-primary-container text-on-primary font-bold' : 'bg-surface-container-lowest hover:bg-surface-container text-primary-container'}"
                >
                  High ({highCount})
                </button>
                <button
                  type="button"
                  onclick={() => (selectedSeverity = "medium")}
                  class="h-9 px-3.5 rounded transition-colors cursor-pointer {selectedSeverity === 'medium' ? 'bg-primary text-on-primary font-bold' : 'bg-surface-container-lowest hover:bg-surface-container text-primary'}"
                >
                  Med ({medCount})
                </button>
                <button
                  type="button"
                  onclick={() => (selectedSeverity = "low")}
                  class="h-9 px-3.5 rounded transition-colors cursor-pointer {selectedSeverity === 'low' ? 'bg-secondary text-surface-container-lowest font-bold' : 'bg-surface-container-lowest hover:bg-surface-container text-secondary'}"
                >
                  Low ({lowCount})
                </button>
                <button
                  type="button"
                  onclick={() => (selectedSeverity = "info")}
                  class="h-9 px-3.5 rounded transition-colors cursor-pointer {selectedSeverity === 'info' ? 'bg-surface-container-highest text-on-surface font-bold' : 'bg-surface-container-lowest hover:bg-surface-container text-outline'}"
                >
                  Info ({infoCount})
                </button>
              </div>

              <!-- OWASP Classification & Sort Dropdowns -->
              <div class="flex items-center gap-2.5">
                <div class="h-9 bg-surface-container-lowest px-3 rounded flex items-center gap-2 font-mono text-xs text-on-surface border border-surface-container-high">
                  <span class="text-outline">Category:</span>
                  <select
                    bind:value={selectedCategory}
                    class="bg-transparent border-0 outline-none text-on-surface font-mono text-xs cursor-pointer"
                  >
                    <option value="all">All</option>
                    <option value="security_headers">Security Headers</option>
                    <option value="cors_misconfiguration">CORS</option>
                    <option value="tls_ssl">TLS / SSL</option>
                    <option value="cookie_security">Cookies</option>
                    <option value="information_disclosure">Info Leaks</option>
                  </select>
                </div>
                <div class="h-9 bg-surface-container-lowest px-3 rounded flex items-center gap-2 font-mono text-xs text-on-surface border border-surface-container-high">
                  <span class="text-outline">Sort:</span>
                  <select
                    bind:value={sortFindingsBy}
                    class="bg-transparent border-0 outline-none text-on-surface font-mono text-xs cursor-pointer"
                  >
                    <option value="severity">Severity</option>
                    <option value="title">Title</option>
                    <option value="category">Category</option>
                  </select>
                </div>
              </div>
            </div>

            <!-- Findings Card List -->
            <div class="flex flex-col gap-3">
              {#if report}
                {#if filteredFindings.length === 0}
                  <div class="p-12 text-center bg-surface-container-low rounded border border-surface-container-high text-outline">
                    <CheckCircle2 class="w-8 h-8 text-tertiary mx-auto mb-2 opacity-80" />
                    <p class="font-bold text-on-surface font-mono">No findings match active filter.</p>
                    <p class="text-xs mt-1">Adjust severity or category selectors to view other security telemetry.</p>
                  </div>
                {:else}
                  {#each filteredFindings as finding (finding.id)}
                    <FindingCard {finding} />
                  {/each}
                {/if}
              {:else}
                <div class="p-12 text-center bg-surface-container-low rounded border border-surface-container-high text-outline">
                  <ShieldAlert class="w-10 h-10 text-outline mx-auto mb-3 opacity-60" />
                  <p class="font-bold text-on-surface font-mono">No Active Target Audited</p>
                  <p class="text-xs mt-1">Enter a target URL above and initiate an audit to view live security findings.</p>
                </div>
              {/if}
            </div>
          </section>
        </div>

      <!-- 4. WORKSPACE 02: PORT MATRIX & RECONNAISSANCE -->
      {:else if currentWorkspace === "ports"}
        <PortMatrixWorkspace
          {targetUrl}
          portReport={report?.port_report || null}
          options={scanOptions}
          onOptionsChange={(opts) => (scanOptions = opts)}
          onUpdateReport={(rep) => {
            ensureReport(targetUrl);
            if (report) {
              report = { ...report, port_report: rep };
            }
          }}
        />

      <!-- 5. WORKSPACE 03: DNS & POSTURE INSPECTOR -->
      {:else if currentWorkspace === "dns"}
        <DnsPostureWorkspace
          {targetUrl}
          dnsReport={report?.dns_security || null}
          onUpdateReport={(dns) => {
            ensureReport(targetUrl);
            if (report) {
              report = { ...report, dns_security: dns };
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

      <!-- 7. WORKSPACE 06: WATCHDOG CONTINUOUS MONITORING -->
      {:else if currentWorkspace === "watchdog"}
        <div class="space-y-4 max-w-6xl w-full mx-auto animate-fade-in pb-10">
          <div class="flex items-center justify-between pb-2 border-b border-surface-container-high">
            <div class="flex items-center gap-2">
              <Activity class="w-4 h-4 text-primary" />
              <h2 class="text-sm font-bold text-on-surface font-mono uppercase tracking-tight">Automated Watchdog Daemon</h2>
            </div>
            <button
              type="button"
              onclick={() => (isMonitorsOpen = true)}
              class="px-3 py-1 bg-primary text-on-primary font-bold rounded text-xs font-mono uppercase transition-opacity cursor-pointer hover:opacity-90"
            >
              + Add Target
            </button>
          </div>

          {#if monitors.length === 0}
            <div class="py-16 text-center bg-surface-container-low border border-surface-container-high rounded space-y-3">
              <Activity class="w-10 h-10 text-primary mx-auto opacity-70" />
              <h3 class="text-sm font-bold text-on-surface font-mono uppercase">No Monitored Targets Active</h3>
              <p class="text-xs text-outline max-w-sm mx-auto font-mono">
                Schedule targets for continuous re-auditing (1h, 6h, 12h, 24h) with native desktop alerts upon score degradation.
              </p>
            </div>
          {:else}
            <div class="space-y-2">
              {#each monitors as m (m.id)}
                <div class="p-3.5 bg-surface-container-low border border-surface-container-high rounded flex items-center justify-between gap-4">
                  <div class="space-y-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <span class="font-bold text-on-surface font-mono text-xs">{m.target_url}</span>
                      <span class="px-1.5 py-0.2 text-[10px] font-mono rounded bg-surface-container text-outline uppercase">
                        Every {m.interval_hours}h
                      </span>
                    </div>
                    <div class="text-[11px] font-mono text-outline uppercase">
                      Next Run: {new Date(m.next_scan_at).toLocaleTimeString()} • Last Score: {m.last_score ?? "Pending"}
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <button
                      type="button"
                      onclick={() => {
                        targetUrl = m.target_url;
                        currentWorkspace = "audit";
                        handleScan(m.target_url);
                      }}
                      class="px-2.5 py-1 bg-surface-container hover:bg-surface-bright text-on-surface border border-surface-container-high rounded text-xs font-mono font-bold uppercase transition-colors cursor-pointer"
                      title="Trigger immediate scan now"
                    >
                      Scan Now
                    </button>
                    <button
                      type="button"
                      onclick={() => handleToggleMonitor(m.id)}
                      class="px-2.5 py-1 rounded text-xs font-mono font-bold uppercase transition-colors cursor-pointer {m.is_active ? 'bg-tertiary/20 text-tertiary border border-tertiary/30' : 'bg-surface-container text-outline border border-surface-container-high'}"
                    >
                      {m.is_active ? "Active" : "Paused"}
                    </button>
                    <button
                      type="button"
                      onclick={() => handleDeleteMonitor(m.id)}
                      class="p-1 text-outline hover:text-error rounded cursor-pointer"
                    >
                      <X class="w-4 h-4" />
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

      <!-- 8. WORKSPACE 07: LOCAL SQLITE LOGS / HISTORY -->
      {:else if currentWorkspace === "history"}
        <div class="space-y-4 max-w-6xl w-full mx-auto animate-fade-in pb-10">
          <div class="flex items-center justify-between pb-2 border-b border-surface-container-high">
            <div class="flex items-center gap-2">
              <HardDrive class="w-4 h-4 text-primary" />
              <h2 class="text-sm font-bold text-on-surface font-mono uppercase tracking-tight">Local SQLite Audit Logs ({history.length})</h2>
            </div>
            {#if history.length > 0}
              <button
                type="button"
                onclick={handleClearAllHistory}
                class="px-2.5 py-1 bg-surface-container-low hover:bg-surface-container text-error border border-error/30 rounded text-xs font-mono uppercase font-bold transition-colors cursor-pointer"
              >
                Clear History
              </button>
            {/if}
          </div>

          {#if history.length === 0}
            <div class="py-16 text-center bg-surface-container-low border border-surface-container-high rounded text-outline text-xs font-mono uppercase">
              No historical scan logs recorded yet.
            </div>
          {:else}
            <div class="space-y-2">
              {#each history as item (item.id)}
                <div class="p-3 bg-surface-container-low border border-surface-container-high rounded flex items-center justify-between gap-4 hover:border-surface-container-highest transition-colors">
                  <div class="space-y-1 min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <span class="font-bold text-on-surface font-mono text-xs truncate">{item.target_url}</span>
                      <span class="px-1.5 py-0.2 text-[10px] font-mono rounded bg-surface-container text-outline">
                        SCORE: {item.security_score}/100
                      </span>
                    </div>
                    <div class="text-[11px] font-mono text-outline uppercase">
                      {new Date(item.scanned_at).toLocaleString()} • {item.total_findings} FINDINGS ({item.critical_count} CRITICAL)
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <button
                      type="button"
                      onclick={() => handleSelectHistoryScan(item.id)}
                      class="px-3 py-1 bg-on-surface text-surface-container-lowest font-bold rounded text-xs font-mono uppercase transition-opacity cursor-pointer hover:opacity-90"
                    >
                      Load
                    </button>
                    <button
                      type="button"
                      onclick={() => handleDeleteScan(item.id)}
                      class="p-1 text-outline hover:text-error rounded cursor-pointer"
                    >
                      <X class="w-4 h-4" />
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
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
<SettingsModal
  isOpen={isSettingsOpen}
  activeTab={settingsTab}
  options={scanOptions}
  {monitors}
  historyCount={history.length}
  onApplyOptions={(opts: ScanOptions) => {
    scanOptions = opts;
    showToast("Scan configuration updated", "success");
    isSettingsOpen = false;
  }}
  onAddMonitor={handleAddMonitor}
  onDeleteMonitor={handleDeleteMonitor}
  onToggleMonitor={handleToggleMonitor}
  onScanTarget={(url) => {
    isSettingsOpen = false;
    targetUrl = url;
    handleScan(url);
  }}
  onClearHistory={handleClearAllHistory}
  onOpenHistory={() => {
    isSettingsOpen = false;
    currentWorkspace = "history";
  }}
  onSelectBatchReport={(batchReport) => {
    isSettingsOpen = false;
    report = batchReport;
    targetUrl = batchReport.target_url;
    currentWorkspace = "audit";
  }}
  onClose={() => (isSettingsOpen = false)}
/>

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

<BatchScanModal
  isOpen={isBatchOpen}
  options={scanOptions}
  onSelectReport={(batchReport) => {
    isBatchOpen = false;
    report = batchReport;
    targetUrl = batchReport.target_url;
    currentWorkspace = "audit";
  }}
  onClose={() => (isBatchOpen = false)}
/>

<MonitorModal
  isOpen={isMonitorsOpen}
  {monitors}
  onAddMonitor={handleAddMonitor}
  onDeleteMonitor={handleDeleteMonitor}
  onToggleMonitor={handleToggleMonitor}
  onScanNow={(url: string) => {
    isMonitorsOpen = false;
    targetUrl = url;
    handleScan(url);
  }}
  onClose={() => (isMonitorsOpen = false)}
/>

<ShortcutsModal
  isOpen={isShortcutsOpen}
  onClose={() => (isShortcutsOpen = false)}
/>

<!-- Toast Notifications -->
<Toast
  message={toastMessage}
  type={toastType}
  visible={toastVisible}
  onDismiss={() => (toastVisible = false)}
/>
