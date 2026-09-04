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
  import ScoreGauge from "$lib/components/ScoreGauge.svelte";
  import SeverityBadge from "$lib/components/SeverityBadge.svelte";
  import FindingCard from "$lib/components/FindingCard.svelte";
  import HistoryModal from "$lib/components/HistoryModal.svelte";
  import ExportModal from "$lib/components/ExportModal.svelte";
  import ExecutiveReportModal from "$lib/components/ExecutiveReportModal.svelte";
  import BatchScanModal from "$lib/components/BatchScanModal.svelte";
  import ScanOptionsModal from "$lib/components/ScanOptionsModal.svelte";
  import MonitorModal from "$lib/components/MonitorModal.svelte";
  import ShortcutsModal from "$lib/components/ShortcutsModal.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";
  import Toast from "$lib/components/Toast.svelte";
  import {
    ShieldCheck,
    AlertOctagon,
    AlertTriangle,
    Info,
    Search,
    Globe,
    Cpu,
    ExternalLink,
    Filter,
    Clock,
    Server,
    Sparkles,
    CheckCircle2,
    Mail,
    FileCode,
    Layers,
    Activity,
    Bell,
    Sliders,
    Loader2,
    X,
    Keyboard,
    Terminal,
    ArrowUpDown,
    Check,
    TrendingUp,
    TrendingDown,
    Zap,
    Radio,
    HardDrive,
    ShieldAlert,
    ChevronRight,
    RotateCw,
    Database,
    FileText,
  } from "lucide-svelte";

  let targetUrl = $state("");
  let isScanning = $state(false);
  let scanError = $state<string | null>(null);
  let report = $state<ScanReport | null>(null);
  let history = $state<ScanSummary[]>([]);
  let monitors = $state<MonitorTarget[]>([]);

  // Scan Configuration - Default port scan enabled with lowest setting (Top 20)
  let scanOptions = $state<ScanOptions>({
    timeout_seconds: 15,
    include_subdomains: true,
    enable_port_scan: true,
    port_scan_profile: "top20",
    port_timeout_ms: 600,
  });

  // Active Workspace Navigation View
  let currentWorkspace = $state<"audit" | "ports" | "dns" | "recon" | "batch" | "watchdog" | "history" | "settings">("audit");

  // Modal States
  let isSettingsOpen = $state(false);
  let settingsTab = $state<"params" | "ports" | "watchdog" | "batch" | "shortcuts" | "data">("params");
  let isHistoryOpen = $state(false);
  let isExportOpen = $state(false);
  let isExecutiveReportOpen = $state(false);
  let isBatchOpen = $state(false);
  let isOptionsOpen = $state(false);
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

  // Open Ports Filter State
  let portSearchQuery = $state("");
  let portRiskFilter = $state<"all" | "risky" | "standard">("all");
  let copiedPort = $state<number | null>(null);
  let subdomainSearch = $state("");

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
    } catch {
      // ignore
    }
  }

  async function loadMonitors() {
    try {
      monitors = await invokeTauri<MonitorTarget[]>("get_monitors");
    } catch {
      // ignore
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      isSettingsOpen = false;
      isHistoryOpen = false;
      isExportOpen = false;
      isExecutiveReportOpen = false;
      isBatchOpen = false;
      isOptionsOpen = false;
      isMonitorsOpen = false;
      isShortcutsOpen = false;
      return;
    }

    if ((e.ctrlKey || e.metaKey) && e.key === ",") {
      e.preventDefault();
      currentWorkspace = "settings";
      return;
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      const input = document.querySelector('header input[type="text"]') as HTMLInputElement | null;
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

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "o") {
      e.preventDefault();
      currentWorkspace = "settings";
      return;
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "e") {
      if (report) {
        e.preventDefault();
        openExportModal();
      }
      return;
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "p") {
      if (report) {
        e.preventDefault();
        isExecutiveReportOpen = !isExecutiveReportOpen;
      }
      return;
    }

    if (e.key === "?" && !["INPUT", "TEXTAREA"].includes((e.target as HTMLElement)?.tagName)) {
      e.preventDefault();
      isShortcutsOpen = true;
    }
  }

  onMount(() => {
    loadHistory();
    loadMonitors();

    window.addEventListener("keydown", handleKeydown);

    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlisten = await listen<any>("monitor_alert", (event) => {
          watchdogAlert = event.payload;
          loadMonitors();
          loadHistory();
          showToast(`Watchdog Alert for ${event.payload?.target_url || "monitored target"}`, "error");
        });
      } catch {
        // ignore in non-tauri dev
      }
    })();

    return () => {
      window.removeEventListener("keydown", handleKeydown);
      if (unlisten) unlisten();
    };
  });

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
    } catch (e: any) {
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

  const categories = [
    { id: "all", label: "All Categories" },
    { id: "security_headers", label: "Headers" },
    { id: "cookie_security", label: "Cookies" },
    { id: "port_exposure", label: "Open Ports & Services" },
    { id: "dns_email_security", label: "DNS & Email" },
    { id: "endpoint_exposure", label: "Endpoints / Recon" },
    { id: "vulnerable_dependency", label: "Dependencies (CVEs)" },
    { id: "information_disclosure", label: "Info Leaks" },
    { id: "tls_ssl", label: "TLS / HTTPS" },
    { id: "cors_misconfiguration", label: "CORS" },
    { id: "insecure_form", label: "Forms" },
    { id: "rce_risk", label: "RCE & Injection Risks" },
  ];

  async function copyTargetUrl() {
    if (!report) return;
    try {
      await navigator.clipboard.writeText(report.target_url);
      copiedUrl = true;
      showToast("Target URL copied to clipboard", "success");
      setTimeout(() => (copiedUrl = false), 2000);
    } catch {}
  }

  async function copyCurlCommand() {
    if (!report) return;
    try {
      const curlCmd = `curl -i -s -k -L -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) VulnRadar/1.0" "${report.target_url}"`;
      await navigator.clipboard.writeText(curlCmd);
      copiedCurl = true;
      showToast("cURL command copied to clipboard", "success");
      setTimeout(() => (copiedCurl = false), 2000);
    } catch {}
  }

  async function copyPortAddress(host: string, port: number) {
    try {
      await navigator.clipboard.writeText(`${host}:${port}`);
      copiedPort = port;
      showToast(`Port address ${host}:${port} copied`, "success");
      setTimeout(() => (copiedPort = null), 2000);
    } catch {}
  }

  const filteredOpenPorts = $derived.by(() => {
    if (!report || !report.port_report || !report.port_report.open_ports) return [];
    return report.port_report.open_ports.filter((p) => {
      if (portRiskFilter === "risky" && !p.is_risky) return false;
      if (portRiskFilter === "standard" && p.is_risky) return false;
      if (portSearchQuery.trim()) {
        const q = portSearchQuery.toLowerCase();
        const matchesPort = p.port.toString().includes(q);
        const matchesService = p.service.toLowerCase().includes(q);
        const matchesDesc = p.description.toLowerCase().includes(q);
        const matchesBanner = p.banner?.toLowerCase().includes(q) || false;
        return matchesPort || matchesService || matchesDesc || matchesBanner;
      }
      return true;
    });
  });

  const filteredSubdomains = $derived.by(() => {
    if (!report || !report.subdomains) return [];
    if (!subdomainSearch.trim()) return report.subdomains;
    return report.subdomains.filter((sub) =>
      sub.toLowerCase().includes(subdomainSearch.toLowerCase())
    );
  });
</script>

<svelte:head>
  <title>VulnRadar — Enterprise Web Security Posture & Vulnerability Scanner</title>
</svelte:head>

<!-- Application Window Frame Container -->
<div class="h-screen w-screen flex flex-col overflow-hidden bg-[#0d0e11] text-[#e2e4e9]">
  <!-- Native Desktop Window Header Toolbar -->
  <Navbar
    bind:targetUrl
    {isScanning}
    hasReport={!!report}
    {hasCustomOptions}
    activeMonitorsCount={monitors.filter((m) => m.is_active).length}
    onScan={() => handleScan()}
    onOpenHistory={() => (currentWorkspace = "history")}
    onOpenSettings={(tab) => {
      if (tab) settingsTab = tab;
      currentWorkspace = "settings";
    }}
    onOpenExport={openExportModal}
  />

  <!-- Watchdog Alert Banner -->
  {#if watchdogAlert}
    <div
      class="bg-rose-950/90 backdrop-blur-md border-b border-rose-800/80 px-4 py-2 text-rose-200 text-xs flex items-center justify-between gap-4 animate-fade-in flex-shrink-0 print:hidden"
    >
      <div class="flex items-center gap-2.5 min-w-0">
        <Bell class="w-4 h-4 text-rose-400 animate-bounce flex-shrink-0" />
        <span class="font-bold uppercase tracking-wider font-mono text-[10px]">Watchdog Alert:</span>
        <span class="truncate font-mono font-bold text-white">{watchdogAlert.target_url}</span>
        <span class="text-rose-300 hidden sm:inline text-xs">
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
          class="px-2.5 py-1 bg-rose-500 hover:bg-rose-400 text-slate-950 font-bold rounded text-xs cursor-pointer transition-all shadow-sm"
        >
          View Audit
        </button>
        <button
          type="button"
          onclick={() => (watchdogAlert = null)}
          class="p-1 text-rose-400 hover:text-rose-200 rounded cursor-pointer"
          aria-label="Dismiss alert"
        >
          <X class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  {/if}

  <!-- Desktop Workstation Split Layout (Sidebar + Main Workspace) -->
  <div class="flex-1 flex overflow-hidden">
    <!-- Desktop Activity Sidebar Rail -->
    <aside
      class="w-60 bg-[var(--color-surface)] border-r border-[var(--color-hairline)] flex flex-col justify-between flex-shrink-0 desktop-select-none print:hidden transition-colors"
    >
      <!-- Navigation Workspaces -->
      <div class="p-2 space-y-1 overflow-y-auto">
        <div class="px-3 py-2 text-[10px] font-bold font-mono uppercase tracking-widest text-[var(--color-text-muted)]">
          WORKSPACES
        </div>

        <!-- 1. Audit Workspace -->
        <button
          type="button"
          onclick={() => (currentWorkspace = "audit")}
          class="w-full flex items-center justify-between px-3 py-2 text-xs font-mono transition-colors cursor-pointer {currentWorkspace === 'audit' ? 'bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] border-l-2 border-l-[var(--color-signal-red)] font-bold' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] border-l-2 border-l-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <span class="text-[10px] font-mono opacity-50">01/</span>
            <span class="truncate uppercase tracking-wider font-semibold">POSTURE AUDIT</span>
          </div>
          {#if report}
            <span class="px-1.5 py-0.2 text-[10px] font-mono border border-[var(--color-hairline)] rounded-none bg-[var(--color-canvas)] text-[var(--color-text-headline)]">
              {report.findings.length}
            </span>
          {/if}
        </button>

        <!-- 2. Port Discovery Workspace -->
        <button
          type="button"
          onclick={() => (currentWorkspace = "ports")}
          class="w-full flex items-center justify-between px-3 py-2 text-xs font-mono transition-colors cursor-pointer {currentWorkspace === 'ports' ? 'bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] border-l-2 border-l-[var(--color-signal-red)] font-bold' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] border-l-2 border-l-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <span class="text-[10px] font-mono opacity-50">02/</span>
            <span class="truncate uppercase tracking-wider font-semibold">PORT MATRIX</span>
          </div>
          {#if report?.port_report}
            <span class="px-1.5 py-0.2 text-[10px] font-mono rounded-none border border-[var(--color-hairline)] bg-[var(--color-canvas)] text-[var(--color-text-headline)]">
              {report.port_report.open_ports_count}
            </span>
          {/if}
        </button>

        <!-- 3. DNS & Email Security Workspace -->
        <button
          type="button"
          onclick={() => (currentWorkspace = "dns")}
          class="w-full flex items-center justify-between px-3 py-2 text-xs font-mono transition-colors cursor-pointer {currentWorkspace === 'dns' ? 'bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] border-l-2 border-l-[var(--color-signal-red)] font-bold' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] border-l-2 border-l-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <span class="text-[10px] font-mono opacity-50">03/</span>
            <span class="truncate uppercase tracking-wider font-semibold">DNS & SPOOF</span>
          </div>
          {#if report?.dns_security?.spf_record}
            <span class="w-1.5 h-1.5 rounded-none bg-emerald-500"></span>
          {/if}
        </button>

        <!-- 4. Recon & Surface Workspace -->
        <button
          type="button"
          onclick={() => (currentWorkspace = "recon")}
          class="w-full flex items-center justify-between px-3 py-2 text-xs font-mono transition-colors cursor-pointer {currentWorkspace === 'recon' ? 'bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] border-l-2 border-l-[var(--color-signal-red)] font-bold' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] border-l-2 border-l-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <span class="text-[10px] font-mono opacity-50">04/</span>
            <span class="truncate uppercase tracking-wider font-semibold">SURFACE RECON</span>
          </div>
          {#if report?.subdomains?.length}
            <span class="px-1.5 py-0.2 text-[10px] font-mono rounded-none border border-[var(--color-hairline)] bg-[var(--color-canvas)] text-[var(--color-text-headline)]">
              {report.subdomains.length}
            </span>
          {/if}
        </button>

        <div class="pt-3 pb-1 px-3 text-[10px] font-bold font-mono uppercase tracking-widest text-[var(--color-text-muted)]">
          FLEET TOOLS
        </div>

        <!-- 5. Batch Fleet Scanner -->
        <button
          type="button"
          onclick={() => {
            currentWorkspace = "batch";
            isBatchOpen = true;
          }}
          class="w-full flex items-center justify-between px-3 py-2 text-xs font-mono transition-colors cursor-pointer {currentWorkspace === 'batch' ? 'bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] border-l-2 border-l-[var(--color-signal-red)] font-bold' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] border-l-2 border-l-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <span class="text-[10px] font-mono opacity-50">05/</span>
            <span class="truncate uppercase tracking-wider font-semibold">FLEET BATCH</span>
          </div>
          <span class="text-[9px] font-mono text-[var(--color-text-muted)]">⌘B</span>
        </button>

        <!-- 6. Watchdog Monitor -->
        <button
          type="button"
          onclick={() => (currentWorkspace = "watchdog")}
          class="w-full flex items-center justify-between px-3 py-2 text-xs font-mono transition-colors cursor-pointer {currentWorkspace === 'watchdog' ? 'bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] border-l-2 border-l-[var(--color-signal-red)] font-bold' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] border-l-2 border-l-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <span class="text-[10px] font-mono opacity-50">06/</span>
            <span class="truncate uppercase tracking-wider font-semibold">WATCHDOG</span>
          </div>
          {#if monitors.filter((m) => m.is_active).length > 0}
            <span class="px-1.5 py-0.2 text-[10px] font-mono rounded-none border border-[var(--color-hairline)] bg-[var(--color-canvas)] text-[var(--color-text-headline)] font-bold">
              {monitors.filter((m) => m.is_active).length}
            </span>
          {/if}
        </button>

        <!-- 7. History & Database Logs -->
        <button
          type="button"
          onclick={() => (currentWorkspace = "history")}
          class="w-full flex items-center justify-between px-3 py-2 text-xs font-mono transition-colors cursor-pointer {currentWorkspace === 'history' ? 'bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] border-l-2 border-l-[var(--color-signal-red)] font-bold' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] border-l-2 border-l-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <span class="text-[10px] font-mono opacity-50">07/</span>
            <span class="truncate uppercase tracking-wider font-semibold">SCAN HISTORY</span>
          </div>
          <span class="px-1.5 py-0.2 text-[10px] font-mono rounded-none border border-[var(--color-hairline)] bg-[var(--color-canvas)] text-[var(--color-text-headline)]">
            {history.length}
          </span>
        </button>

        <!-- 8. Preferences & Settings -->
        <button
          type="button"
          onclick={() => (currentWorkspace = "settings")}
          class="w-full flex items-center justify-between px-3 py-2 text-xs font-mono transition-colors cursor-pointer {currentWorkspace === 'settings' ? 'bg-[var(--color-surface-hover)] text-[var(--color-text-headline)] border-l-2 border-l-[var(--color-signal-red)] font-bold' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] border-l-2 border-l-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <span class="text-[10px] font-mono opacity-50">08/</span>
            <span class="truncate uppercase tracking-wider font-semibold">SETTINGS</span>
          </div>
          {#if hasCustomOptions}
            <span class="w-1.5 h-1.5 rounded-none bg-[var(--color-signal-red)]"></span>
          {/if}
        </button>
      </div>

      <!-- Bottom Sidebar System Widget -->
      <div class="p-3 border-t border-[var(--color-hairline)] space-y-2 bg-[var(--color-surface)]">
        <button
          type="button"
          onclick={() => (isShortcutsOpen = true)}
          class="w-full flex items-center justify-between px-2.5 py-1.5 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] rounded-none text-[10px] font-mono uppercase tracking-wider text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] transition-colors cursor-pointer"
        >
          <span class="flex items-center gap-1.5">
            <Keyboard class="w-3.5 h-3.5 text-[var(--color-text-muted)]" />
            SHORTCUTS
          </span>
          <kbd class="px-1 py-0.2 text-[9px] font-mono border border-[var(--color-hairline)] rounded-none">?</kbd>
        </button>

        <div class="flex items-center justify-between text-[10px] font-mono text-[var(--color-text-muted)] px-1 uppercase tracking-wider">
          <span class="flex items-center gap-1">
            <Database class="w-3 h-3 text-[var(--color-text-muted)]" />
            SQLITE WAL
          </span>
          <span class="text-emerald-500 font-bold">READY</span>
        </div>
      </div>
    </aside>

    <!-- Main Desktop Workstation Content Area -->
    <main class="flex-1 overflow-y-auto bg-[var(--color-canvas)] text-[var(--color-text-body)] flex flex-col p-6 transition-colors">
      <!-- 1. SCANNING PROGRESS HUD -->
      {#if isScanning}
        <div class="my-auto max-w-lg mx-auto w-full p-6 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-5 text-center animate-fade-in">
          <div class="w-12 h-12 rounded-none bg-[var(--color-canvas)] border border-[var(--color-hairline)] flex items-center justify-center text-[var(--color-signal-red)] mx-auto">
            <Loader2 class="w-6 h-6 animate-spin" />
          </div>

          <div class="space-y-1.5">
            <div class="inline-flex items-center gap-2 px-2.5 py-0.5 rounded-none bg-[var(--color-canvas)] border border-[var(--color-hairline)] text-[var(--color-text-headline)] text-[11px] font-mono font-bold uppercase">
              <span class="w-1.5 h-1.5 rounded-none bg-[var(--color-signal-red)] animate-pulse"></span>
              <span>RUNNING MULTI-THREADED SECURITY AUDIT</span>
            </div>
            <h2 class="text-lg font-black text-[var(--color-text-headline)] font-mono uppercase tracking-tight">Auditing Target Surface</h2>
            <div class="p-2 bg-[var(--color-canvas)] rounded-none border border-[var(--color-hairline)] text-xs font-mono text-[var(--color-text-headline)] truncate max-w-sm mx-auto">
              {targetUrl}
            </div>
          </div>

          <!-- Active Pipeline Modules Checklist -->
          <div class="grid grid-cols-2 gap-2 text-left text-xs font-mono pt-3 border-t border-[var(--color-hairline)]">
            <div class="flex items-center gap-2 text-[var(--color-text-body)]">
              <CheckCircle2 class="w-3.5 h-3.5 text-emerald-500" />
              <span>HTTP HEADERS</span>
            </div>
            <div class="flex items-center gap-2 text-[var(--color-text-body)]">
              <CheckCircle2 class="w-3.5 h-3.5 text-emerald-500" />
              <span>PORT DISCOVERY</span>
            </div>
            <div class="flex items-center gap-2 text-[var(--color-text-body)]">
              <CheckCircle2 class="w-3.5 h-3.5 text-emerald-500" />
              <span>DOH ANTI-SPOOF</span>
            </div>
            <div class="flex items-center gap-2 text-[var(--color-text-body)]">
              <CheckCircle2 class="w-3.5 h-3.5 text-emerald-500" />
              <span>SCA CVE LIBRARY</span>
            </div>
          </div>
        </div>

      <!-- 2. SCAN ERROR HUD -->
      {:else if scanError}
        <div class="my-auto max-w-xl mx-auto w-full p-6 bg-[var(--color-surface)] border border-red-500/40 rounded-none space-y-4 text-red-600 dark:text-red-400 animate-fade-in">
          <div class="flex items-start gap-3">
            <AlertOctagon class="w-6 h-6 text-red-500 flex-shrink-0 mt-0.5" />
            <div class="space-y-1.5 flex-1">
              <h3 class="text-sm font-bold text-red-600 dark:text-red-400 uppercase font-mono tracking-wider">Audit Execution Failed</h3>
              <p class="text-xs text-[var(--color-text-body)] font-mono bg-[var(--color-canvas)] p-3 rounded-none border border-red-500/20 break-all leading-relaxed">
                {scanError}
              </p>
            </div>
          </div>
          <div class="flex items-center justify-end gap-2 pt-2 border-t border-[var(--color-hairline)]">
            <button
              type="button"
              onclick={() => handleScan()}
              class="px-4 py-1.5 bg-[var(--color-signal-red)] hover:opacity-90 text-white rounded-none text-xs font-mono font-bold uppercase transition-opacity cursor-pointer"
            >
              Retry Audit
            </button>
          </div>
        </div>

      <!-- 3. ACTIVE WORKSPACE RENDERING -->
      {:else if currentWorkspace === "ports" && report}
        <!-- DEDICATED PORT WORKSPACE VIEW -->
        <div class="space-y-4 max-w-6xl w-full mx-auto animate-fade-in">
          <!-- Port Telemetry Bar -->
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
            <div class="p-3.5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-0.5">
              <span class="text-[10px] font-bold uppercase tracking-wider text-[var(--color-text-muted)] font-mono">Host / IP</span>
              <div class="text-sm font-bold font-mono text-[var(--color-text-headline)] truncate">
                {report.port_report?.ip_address || report.port_report?.host || "Resolving IP"}
              </div>
            </div>
            <div class="p-3.5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-0.5">
              <span class="text-[10px] font-bold uppercase tracking-wider text-[var(--color-text-muted)] font-mono">Ports Probed</span>
              <div class="text-sm font-bold font-mono text-[var(--color-text-headline)]">
                {report.port_report?.scanned_ports_count || 0} Ports (Auto Top 20)
              </div>
            </div>
            <div class="p-3.5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-0.5">
              <span class="text-[10px] font-bold uppercase tracking-wider text-[var(--color-text-muted)] font-mono">Open Ports</span>
              <div class="text-sm font-bold font-mono {(report.port_report?.open_ports_count || 0) > 0 ? 'text-emerald-500' : 'text-[var(--color-text-muted)]'}">
                {report.port_report?.open_ports_count || 0} Discovered
              </div>
            </div>
            <div class="p-3.5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-0.5">
              <span class="text-[10px] font-bold uppercase tracking-wider text-[var(--color-text-muted)] font-mono">Duration</span>
              <div class="text-sm font-bold font-mono text-[var(--color-text-headline)]">
                {report.port_report?.scan_duration_ms || 0} ms
              </div>
            </div>
          </div>

          <!-- Port Search & Filter Controls -->
          <div class="p-3 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none flex flex-col md:flex-row items-center justify-between gap-3">
            <div class="relative w-full md:w-80">
              <Search class="w-3.5 h-3.5 text-[var(--color-text-muted)] absolute inset-y-0 left-3 my-auto pointer-events-none" />
              <input
                type="text"
                bind:value={portSearchQuery}
                placeholder="FILTER BY PORT #, SERVICE, BANNER..."
                class="w-full pl-8 pr-3 py-1.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs text-[var(--color-text-headline)] placeholder-[var(--color-text-muted)] font-mono uppercase focus:outline-none"
              />
            </div>

            <div class="flex items-center gap-1.5 w-full md:w-auto">
              <button
                type="button"
                onclick={() => (portRiskFilter = "all")}
                class="px-2.5 py-1 rounded-none text-xs font-mono font-bold uppercase cursor-pointer transition-colors {portRiskFilter === 'all' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)]' : 'bg-[var(--color-canvas)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] border border-[var(--color-hairline)]'}"
              >
                All ({report.port_report?.open_ports.length || 0})
              </button>
              <button
                type="button"
                onclick={() => (portRiskFilter = "risky")}
                class="px-2.5 py-1 rounded-none text-xs font-mono font-bold uppercase cursor-pointer transition-colors {portRiskFilter === 'risky' ? 'bg-red-500 text-white' : 'bg-[var(--color-canvas)] text-red-600 dark:text-red-400 border border-[var(--color-hairline)] hover:border-red-500/40'}"
              >
                Risky ({report.port_report?.open_ports.filter((p) => p.is_risky).length || 0})
              </button>
              <button
                type="button"
                onclick={() => (portRiskFilter = "standard")}
                class="px-2.5 py-1 rounded-none text-xs font-mono font-bold uppercase cursor-pointer transition-colors {portRiskFilter === 'standard' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)]' : 'bg-[var(--color-canvas)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] border border-[var(--color-hairline)]'}"
              >
                Standard ({report.port_report?.open_ports.filter((p) => !p.is_risky).length || 0})
              </button>
            </div>
          </div>

          <!-- Open Ports Cards -->
          {#if filteredOpenPorts.length === 0}
            <div class="py-16 text-center bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-2">
              <CheckCircle2 class="w-10 h-10 text-emerald-500 mx-auto opacity-80" />
              <h3 class="text-sm font-bold text-[var(--color-text-headline)] font-mono uppercase">No Open Ports Matching Criteria</h3>
              <p class="text-xs text-[var(--color-text-muted)] max-w-sm mx-auto font-mono">
                No active listening services were detected on scanned ports.
              </p>
            </div>
          {:else}
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
              {#each filteredOpenPorts as p (p.port)}
                <div
                  class="p-4 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-3 transition-colors {p.is_risky ? 'border-l-4 border-l-red-500' : 'hover:border-[var(--color-hairline-strong)]'}"
                >
                  <div class="flex items-center justify-between gap-2">
                    <div class="flex items-center gap-2">
                      <span class="px-2 py-0.5 text-xs font-mono font-bold rounded-none bg-[var(--color-canvas)] border border-[var(--color-hairline)] text-[var(--color-text-headline)]">
                        PORT {p.port}/{p.protocol.toUpperCase()}
                      </span>
                      <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-none text-[10px] font-mono bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30 font-bold uppercase">
                        <span class="w-1.5 h-1.5 rounded-none bg-emerald-500"></span>
                        OPEN
                      </span>
                    </div>

                    {#if p.is_risky}
                      <span class="px-2 py-0.5 text-[10px] font-mono rounded-none bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/30 flex items-center gap-1 font-bold uppercase">
                        <AlertTriangle class="w-3 h-3 text-red-500" />
                        EXPOSED / RISKY
                      </span>
                    {/if}
                  </div>

                  <div class="space-y-0.5">
                    <div class="text-sm font-bold text-[var(--color-text-headline)] flex items-center gap-1.5 font-mono">
                      <Server class="w-3.5 h-3.5 text-[var(--color-text-muted)]" />
                      <span>{p.service}</span>
                    </div>
                    <p class="text-xs text-[var(--color-text-muted)]">{p.description}</p>
                  </div>

                  {#if p.banner}
                    <div class="p-2.5 bg-[var(--color-canvas)] rounded-none border border-[var(--color-hairline)] space-y-1">
                      <span class="text-[9px] font-mono text-[var(--color-text-muted)] uppercase tracking-wider block font-bold">Service Banner</span>
                      <pre class="text-xs font-mono text-[var(--color-text-headline)] overflow-x-auto whitespace-pre-wrap">{p.banner}</pre>
                    </div>
                  {/if}

                  <div class="pt-2 border-t border-[var(--color-hairline)] flex items-center justify-between">
                    <div class="text-[11px] font-mono text-[var(--color-text-muted)]">
                      {report.port_report?.host}:{p.port}
                    </div>

                    <div class="flex items-center gap-1.5">
                      <button
                        type="button"
                        onclick={() => copyPortAddress(report!.port_report!.host, p.port)}
                        class="px-2 py-0.5 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs font-mono transition-colors cursor-pointer flex items-center gap-1 uppercase"
                      >
                        {#if copiedPort === p.port}
                          <CheckCircle2 class="w-3 h-3 text-emerald-500" />
                          <span class="text-emerald-500">COPIED</span>
                        {:else}
                          <span>COPY ADDRESS</span>
                        {/if}
                      </button>

                      {#if [80, 443, 3000, 5000, 8000, 8080, 8081, 8443, 8888, 9000, 9090].includes(p.port)}
                        <a
                          href={`${p.port === 443 || p.port === 8443 ? "https" : "http"}://${report?.port_report?.host || "localhost"}:${p.port}`}
                          target="_blank"
                          rel="noopener noreferrer"
                          class="p-1 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] border border-[var(--color-hairline)] rounded-none text-xs transition-colors cursor-pointer"
                          title="Open in Browser"
                        >
                          <ExternalLink class="w-3.5 h-3.5" />
                        </a>
                      {/if}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

      <!-- 4. DNS & EMAIL WORKSPACE VIEW -->
      {:else if currentWorkspace === "dns" && report}
        <div class="space-y-4 max-w-6xl w-full mx-auto animate-fade-in">
          {#if report.dns_security}
            <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
              <div class="p-4 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-2">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-bold text-[var(--color-text-headline)] font-mono uppercase">SPF Anti-Spoofing</span>
                  <span class="px-2 py-0.5 text-xs font-mono rounded-none font-bold uppercase {report.dns_security.spf_record ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30' : 'bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/30'}">
                    {report.dns_security.spf_record ? "Configured" : "Missing"}
                  </span>
                </div>
                <p class="text-xs text-[var(--color-text-muted)] font-mono">Validates authorized mail senders via RFC 7208.</p>
                {#if report.dns_security.spf_record}
                  <pre class="bg-[var(--color-canvas)] p-2.5 rounded-none text-xs font-mono text-[var(--color-text-headline)] border border-[var(--color-hairline)] overflow-x-auto whitespace-pre-wrap">{report.dns_security.spf_record}</pre>
                {/if}
              </div>

              <div class="p-4 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-2">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-bold text-[var(--color-text-headline)] font-mono uppercase">DMARC Enforcement</span>
                  <span class="px-2 py-0.5 text-xs font-mono rounded-none font-bold uppercase {report.dns_security.dmarc_policy && report.dns_security.dmarc_policy !== 'none' ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30' : 'bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/30'}">
                    {report.dns_security.dmarc_policy || "None"}
                  </span>
                </div>
                <p class="text-xs text-[var(--color-text-muted)] font-mono">Enforces domain-based email authentication & alignment.</p>
                {#if report.dns_security.dmarc_record}
                  <pre class="bg-[var(--color-canvas)] p-2.5 rounded-none text-xs font-mono text-[var(--color-text-headline)] border border-[var(--color-hairline)] overflow-x-auto whitespace-pre-wrap">{report.dns_security.dmarc_record}</pre>
                {/if}
              </div>

              <div class="p-4 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-2">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-bold text-[var(--color-text-headline)] font-mono uppercase">DNSSEC Validation</span>
                  <span class="px-2 py-0.5 text-xs font-mono rounded-none font-bold uppercase {report.dns_security.dnssec_enabled ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30' : 'bg-[var(--color-canvas)] text-[var(--color-text-muted)] border border-[var(--color-hairline)]'}">
                    {report.dns_security.dnssec_enabled ? "Enabled" : "Disabled"}
                  </span>
                </div>
                <p class="text-xs text-[var(--color-text-muted)] font-mono">Cryptographically authenticates DNS records against poisoning.</p>
              </div>
            </div>
          {/if}
        </div>

      <!-- 5. RECON & SURFACE WORKSPACE VIEW -->
      {:else if currentWorkspace === "recon" && report}
        <div class="space-y-4 max-w-6xl w-full mx-auto animate-fade-in">
          <div class="p-3 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none flex items-center justify-between gap-3">
            <div class="relative w-full max-w-sm">
              <Search class="w-3.5 h-3.5 text-[var(--color-text-muted)] absolute inset-y-0 left-3 my-auto pointer-events-none" />
              <input
                type="text"
                bind:value={subdomainSearch}
                placeholder="FILTER DISCOVERED SUBDOMAINS..."
                class="w-full pl-8 pr-3 py-1.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs text-[var(--color-text-headline)] placeholder-[var(--color-text-muted)] font-mono uppercase focus:outline-none"
              />
            </div>
            <span class="text-xs font-mono text-[var(--color-text-muted)] uppercase font-bold">
              {filteredSubdomains.length} / {report.subdomains?.length || 0} DISCOVERED
            </span>
          </div>

          {#if filteredSubdomains.length > 0}
            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2">
              {#each filteredSubdomains as sub}
                <button
                  type="button"
                  onclick={() => {
                    targetUrl = `https://${sub}`;
                    handleScan(`https://${sub}`);
                  }}
                  class="p-2.5 bg-[var(--color-surface)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] hover:border-[var(--color-hairline-strong)] rounded-none text-left text-xs font-mono text-[var(--color-text-body)] hover:text-[var(--color-text-headline)] transition-colors cursor-pointer truncate flex items-center justify-between group"
                >
                  <span class="truncate">{sub}</span>
                  <ExternalLink class="w-3.5 h-3.5 text-[var(--color-text-muted)] group-hover:text-[var(--color-text-headline)] flex-shrink-0 ml-2" />
                </button>
              {/each}
            </div>
          {/if}
        </div>

      <!-- 6. SCAN LOGS / HISTORY WORKSPACE VIEW -->
      {:else if currentWorkspace === "history"}
        <div class="space-y-4 max-w-6xl w-full mx-auto animate-fade-in">
          <div class="flex items-center justify-between pb-2 border-b border-[var(--color-hairline)]">
            <div class="flex items-center gap-2">
              <HardDrive class="w-4 h-4 text-[var(--color-signal-red)]" />
              <h2 class="text-sm font-black text-[var(--color-text-headline)] font-mono uppercase tracking-tight">Local SQLite Audit Logs ({history.length})</h2>
            </div>
            {#if history.length > 0}
              <button
                type="button"
                onclick={handleClearAllHistory}
                class="px-2.5 py-1 bg-[var(--color-canvas)] hover:bg-[var(--color-surface-hover)] text-red-600 dark:text-red-400 border border-[var(--color-hairline)] rounded-none text-xs font-mono uppercase font-bold transition-colors cursor-pointer"
              >
                Clear History
              </button>
            {/if}
          </div>

          {#if history.length === 0}
            <div class="py-16 text-center bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none text-[var(--color-text-muted)] text-xs font-mono uppercase">
              No historical scan logs recorded yet.
            </div>
          {:else}
            <div class="space-y-2">
              {#each history as item (item.id)}
                <div class="p-3 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none flex items-center justify-between gap-4 hover:border-[var(--color-hairline-strong)] transition-colors">
                  <div class="space-y-1 min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <span class="font-bold text-[var(--color-text-headline)] font-mono text-xs truncate">{item.target_url}</span>
                      <span class="px-1.5 py-0.2 text-[10px] font-mono rounded-none bg-[var(--color-canvas)] border border-[var(--color-hairline)] text-[var(--color-text-muted)]">
                        SCORE: {item.security_score}/100
                      </span>
                    </div>
                    <div class="text-[11px] font-mono text-[var(--color-text-muted)] uppercase">
                      {new Date(item.scanned_at).toLocaleString()} • {item.total_findings} FINDINGS ({item.critical_count} CRITICAL)
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <button
                      type="button"
                      onclick={() => handleSelectHistoryScan(item.id)}
                      class="px-3 py-1 bg-[var(--color-text-headline)] text-[var(--color-canvas)] font-bold rounded-none text-xs font-mono uppercase transition-opacity cursor-pointer hover:opacity-90"
                    >
                      Load
                    </button>
                    <button
                      type="button"
                      onclick={() => handleDeleteScan(item.id)}
                      class="p-1 text-[var(--color-text-muted)] hover:text-red-500 rounded-none cursor-pointer"
                    >
                      <X class="w-4 h-4" />
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

      <!-- 7. WATCHDOG WORKSPACE VIEW -->
      {:else if currentWorkspace === "watchdog"}
        <div class="space-y-4 max-w-6xl w-full mx-auto animate-fade-in">
          <div class="flex items-center justify-between pb-2 border-b border-[var(--color-hairline)]">
            <div class="flex items-center gap-2">
              <Activity class="w-4 h-4 text-[var(--color-signal-red)]" />
              <h2 class="text-sm font-black text-[var(--color-text-headline)] font-mono uppercase tracking-tight">Automated Watchdog Daemon</h2>
            </div>
            <button
              type="button"
              onclick={() => (isMonitorsOpen = true)}
              class="px-3 py-1 bg-[var(--color-text-headline)] text-[var(--color-canvas)] font-bold rounded-none text-xs font-mono uppercase transition-opacity cursor-pointer hover:opacity-90"
            >
              + Add Target
            </button>
          </div>

          {#if monitors.length === 0}
            <div class="py-16 text-center bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-3">
              <Activity class="w-10 h-10 text-[var(--color-signal-red)] mx-auto opacity-70" />
              <h3 class="text-sm font-bold text-[var(--color-text-headline)] font-mono uppercase">No Monitored Targets Active</h3>
              <p class="text-xs text-[var(--color-text-muted)] max-w-sm mx-auto font-mono">
                Schedule targets for continuous re-auditing (1h, 6h, 12h, 24h) with native desktop alerts upon score degradation.
              </p>
            </div>
          {:else}
            <div class="space-y-2">
              {#each monitors as m (m.id)}
                <div class="p-3.5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none flex items-center justify-between gap-4">
                  <div class="space-y-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <span class="font-bold text-[var(--color-text-headline)] font-mono text-xs">{m.target_url}</span>
                      <span class="px-1.5 py-0.2 text-[10px] font-mono rounded-none bg-[var(--color-canvas)] border border-[var(--color-hairline)] text-[var(--color-text-muted)] uppercase">
                        Every {m.interval_hours}h
                      </span>
                    </div>
                    <div class="text-[11px] font-mono text-[var(--color-text-muted)] uppercase">
                      Next Run: {new Date(m.next_scan_at).toLocaleTimeString()} • Last Score: {m.last_score ?? "Pending"}
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <button
                      type="button"
                      onclick={() => handleToggleMonitor(m.id)}
                      class="px-2.5 py-1 rounded-none text-xs font-mono font-bold uppercase transition-colors cursor-pointer {m.is_active ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30' : 'bg-[var(--color-canvas)] text-[var(--color-text-muted)] border border-[var(--color-hairline)]'}"
                    >
                      {m.is_active ? "Active" : "Paused"}
                    </button>
                    <button
                      type="button"
                      onclick={() => handleDeleteMonitor(m.id)}
                      class="p-1 text-[var(--color-text-muted)] hover:text-red-500 rounded-none cursor-pointer"
                    >
                      <X class="w-4 h-4" />
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

      <!-- 8. SETTINGS WORKSPACE VIEW -->
      {:else if currentWorkspace === "settings"}
        <div class="space-y-4 max-w-4xl w-full mx-auto animate-fade-in">
          <div class="p-5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-4">
            <div class="flex items-center justify-between pb-3 border-b border-[var(--color-hairline)]">
              <div>
                <h2 class="text-sm font-black text-[var(--color-text-headline)] font-mono uppercase tracking-tight">Scanner Configuration</h2>
                <p class="text-xs text-[var(--color-text-muted)] font-mono">Configure global scan parameters, HTTP headers, and port probe profiles.</p>
              </div>
              <button
                type="button"
                onclick={() => {
                  settingsTab = "params";
                  isSettingsOpen = true;
                }}
                class="px-3 py-1.5 bg-[var(--color-text-headline)] text-[var(--color-canvas)] font-bold rounded-none text-xs font-mono uppercase transition-opacity cursor-pointer hover:opacity-90"
              >
                Open Full Settings Hub
              </button>
            </div>

            <!-- Fast Port Scanner Preset Selector -->
            <div class="p-3 bg-[var(--color-canvas)] border border-[var(--color-hairline)] rounded-none space-y-2">
              <div class="flex items-center justify-between">
                <span class="text-xs font-bold text-[var(--color-text-headline)] font-mono flex items-center gap-1.5 uppercase">
                  <Server class="w-3.5 h-3.5 text-[var(--color-text-muted)]" />
                  Automatic Port Discovery Profile
                </span>
                <span class="text-[10px] font-mono text-emerald-500 uppercase font-bold">Default: Top 20</span>
              </div>
              <p class="text-xs text-[var(--color-text-muted)] font-mono">
                Ports are scanned concurrently with sub-second probes whenever a target audit is executed.
              </p>
              <div class="grid grid-cols-3 gap-2 pt-1">
                <button
                  type="button"
                  onclick={() => (scanOptions.port_scan_profile = "top20")}
                  class="py-1.5 px-2 rounded-none text-xs font-mono font-bold uppercase border transition-colors cursor-pointer {scanOptions.port_scan_profile === 'top20' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent' : 'bg-[var(--color-surface)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] border-[var(--color-hairline)]'}"
                >
                  Top 20 (Fastest)
                </button>
                <button
                  type="button"
                  onclick={() => (scanOptions.port_scan_profile = "databases")}
                  class="py-1.5 px-2 rounded-none text-xs font-mono font-bold uppercase border transition-colors cursor-pointer {scanOptions.port_scan_profile === 'databases' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent' : 'bg-[var(--color-surface)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] border-[var(--color-hairline)]'}"
                >
                  Databases
                </button>
                <button
                  type="button"
                  onclick={() => (scanOptions.port_scan_profile = "top100")}
                  class="py-1.5 px-2 rounded-none text-xs font-mono font-bold uppercase border transition-colors cursor-pointer {scanOptions.port_scan_profile === 'top100' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)] border-transparent' : 'bg-[var(--color-surface)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] border-[var(--color-hairline)]'}"
                >
                  Top 100
                </button>
              </div>
            </div>
          </div>
        </div>

      <!-- 9. PRIMARY AUDIT WORKSPACE (WHEN REPORT LOADED) -->
      {:else if report}
        <div class="space-y-4 max-w-6xl w-full mx-auto animate-fade-in">
          <!-- Target Summary Card -->
          <div class="p-5 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none flex flex-col lg:flex-row items-start lg:items-center justify-between gap-6 transition-colors">
            <div class="space-y-2 min-w-0 flex-1">
              <div class="flex flex-wrap items-center gap-2">
                <span class="px-2 py-0.5 text-xs font-mono font-bold bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30 rounded-none uppercase tracking-wider">
                  HTTP {report.status_code}
                </span>
                <span class="text-xs text-[var(--color-text-muted)] font-mono">
                  LATENCY: <strong class="text-[var(--color-text-headline)] tabular-nums">{report.response_time_ms} MS</strong>
                </span>
                <span class="text-xs text-[var(--color-hairline-strong)]">•</span>
                <span class="text-xs text-[var(--color-text-muted)] font-mono uppercase">
                  AUDITED AT {new Date(report.scanned_at).toLocaleTimeString()}
                </span>

                {#if previousScan}
                  <span class="text-xs text-[var(--color-hairline-strong)]">•</span>
                  <div class="inline-flex items-center gap-1 px-2 py-0.5 rounded-none text-[11px] font-mono {report.security_score >= previousScan.security_score ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30' : 'bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/30'}">
                    {#if report.security_score > previousScan.security_score}
                      <TrendingUp class="w-3 h-3 text-emerald-500" />
                      <span>+{report.security_score - previousScan.security_score} PTS</span>
                    {:else if report.security_score < previousScan.security_score}
                      <TrendingDown class="w-3 h-3 text-red-500" />
                      <span>-{previousScan.security_score - report.security_score} PTS</span>
                    {/if}
                  </div>
                {/if}
              </div>

              <div class="flex items-center gap-2">
                <h1 class="text-xl font-bold text-[var(--color-text-headline)] tracking-tight truncate flex items-center gap-2 font-mono">
                  <Globe class="w-4 h-4 text-[var(--color-signal-red)] flex-shrink-0" />
                  <span class="truncate">{report.target_url}</span>
                </h1>
                <div class="flex items-center gap-1">
                  <button
                    type="button"
                    onclick={copyTargetUrl}
                    class="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] rounded-none border border-transparent hover:border-[var(--color-hairline)] cursor-pointer"
                    title="Copy URL"
                  >
                    {#if copiedUrl}
                      <CheckCircle2 class="w-3.5 h-3.5 text-emerald-500" />
                    {:else}
                      <Search class="w-3.5 h-3.5" />
                    {/if}
                  </button>
                  <button
                    type="button"
                    onclick={copyCurlCommand}
                    class="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface-hover)] rounded-none border border-transparent hover:border-[var(--color-hairline)] cursor-pointer"
                    title="Copy cURL command"
                  >
                    {#if copiedCurl}
                      <Check class="w-3.5 h-3.5 text-emerald-500" />
                    {:else}
                      <Terminal class="w-3.5 h-3.5" />
                    {/if}
                  </button>
                </div>
              </div>

              <!-- Detected Technologies -->
              {#if report.technologies_detected.length > 0}
                <div class="flex flex-wrap items-center gap-1.5 pt-1">
                  <span class="text-[10px] text-[var(--color-text-muted)] font-mono uppercase tracking-widest flex items-center gap-1 mr-1 font-bold">
                    <Cpu class="w-3 h-3 text-[var(--color-text-muted)]" /> STACK:
                  </span>
                  {#each report.technologies_detected as tech}
                    <span class="px-2 py-0.5 text-[11px] font-mono bg-[var(--color-canvas)] text-[var(--color-text-body)] border border-[var(--color-hairline)] rounded-none">
                      {tech}
                    </span>
                  {/each}
                </div>
              {/if}
            </div>

            <!-- Score Gauge Component -->
            <div class="w-full lg:w-auto flex-shrink-0">
              <ScoreGauge score={report.security_score} />
            </div>
          </div>

          <!-- Severity Distribution & Filter Bar -->
          <div class="p-4 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-3 transition-colors">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span class="text-[11px] font-bold uppercase tracking-wider text-[var(--color-text-headline)] font-mono">
                  SEVERITY DISTRIBUTION
                </span>
                <span class="px-2 py-0.5 text-[10px] font-mono font-bold bg-[var(--color-canvas)] text-[var(--color-text-body)] border border-[var(--color-hairline)] rounded-none">
                  {report.total_findings} FINDINGS
                </span>
              </div>
              {#if selectedSeverity !== "all" || searchQuery.trim() || selectedCategory !== "all"}
                <button
                  type="button"
                  onclick={() => {
                    selectedSeverity = "all";
                    selectedCategory = "all";
                    searchQuery = "";
                  }}
                  class="text-xs font-mono text-[var(--color-signal-red)] hover:underline cursor-pointer uppercase tracking-wider"
                >
                  [Reset Filters]
                </button>
              {/if}
            </div>

            <!-- Proportional Colored Bar -->
            {#if report.total_findings > 0}
              <div class="w-full h-2 bg-[var(--color-hairline)] rounded-none overflow-hidden flex gap-0.5">
                {#if report.critical_count > 0}
                  <div class="bg-red-500 h-full" style="width: {(report.critical_count / report.total_findings) * 100}%"></div>
                {/if}
                {#if report.high_count > 0}
                  <div class="bg-orange-500 h-full" style="width: {(report.high_count / report.total_findings) * 100}%"></div>
                {/if}
                {#if report.medium_count > 0}
                  <div class="bg-amber-500 h-full" style="width: {(report.medium_count / report.total_findings) * 100}%"></div>
                {/if}
                {#if report.low_count > 0}
                  <div class="bg-blue-500 h-full" style="width: {(report.low_count / report.total_findings) * 100}%"></div>
                {/if}
                {#if report.info_count > 0}
                  <div class="bg-zinc-500 h-full" style="width: {(report.info_count / report.total_findings) * 100}%"></div>
                {/if}
              </div>
            {/if}

            <!-- Severity Filter Chips -->
            <div class="flex flex-wrap items-center gap-1.5 pt-1">
              <button
                type="button"
                onclick={() => (selectedSeverity = "all")}
                class="px-2.5 py-1 rounded-none text-xs font-mono font-bold tracking-wider uppercase transition-colors cursor-pointer {selectedSeverity === 'all' ? 'bg-[var(--color-text-headline)] text-[var(--color-canvas)]' : 'bg-[var(--color-canvas)] text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] border border-[var(--color-hairline)]'}"
              >
                ALL ({report.total_findings})
              </button>
              <button
                type="button"
                onclick={() => (selectedSeverity = "critical")}
                class="px-2.5 py-1 rounded-none text-xs font-mono font-bold tracking-wider uppercase transition-colors cursor-pointer {selectedSeverity === 'critical' ? 'bg-red-600 text-white' : 'bg-red-500/10 text-red-600 dark:text-red-400 hover:bg-red-500/20 border border-red-500/30'}"
              >
                CRITICAL ({report.critical_count})
              </button>
              <button
                type="button"
                onclick={() => (selectedSeverity = "high")}
                class="px-2.5 py-1 rounded-none text-xs font-mono font-bold tracking-wider uppercase transition-colors cursor-pointer {selectedSeverity === 'high' ? 'bg-orange-600 text-white' : 'bg-orange-500/10 text-orange-600 dark:text-orange-400 hover:bg-orange-500/20 border border-orange-500/30'}"
              >
                HIGH ({report.high_count})
              </button>
              <button
                type="button"
                onclick={() => (selectedSeverity = "medium")}
                class="px-2.5 py-1 rounded-none text-xs font-mono font-bold tracking-wider uppercase transition-colors cursor-pointer {selectedSeverity === 'medium' ? 'bg-amber-600 text-white' : 'bg-amber-500/10 text-amber-600 dark:text-amber-400 hover:bg-amber-500/20 border border-amber-500/30'}"
              >
                MED ({report.medium_count})
              </button>
              <button
                type="button"
                onclick={() => (selectedSeverity = "low")}
                class="px-2.5 py-1 rounded-none text-xs font-mono font-bold tracking-wider uppercase transition-colors cursor-pointer {selectedSeverity === 'low' ? 'bg-blue-600 text-white' : 'bg-blue-500/10 text-blue-600 dark:text-blue-400 hover:bg-blue-500/20 border border-blue-500/30'}"
              >
                LOW ({report.low_count})
              </button>
              <button
                type="button"
                onclick={() => (selectedSeverity = "info")}
                class="px-2.5 py-1 rounded-none text-xs font-mono font-bold tracking-wider uppercase transition-colors cursor-pointer {selectedSeverity === 'info' ? 'bg-zinc-600 text-white' : 'bg-zinc-500/10 text-zinc-600 dark:text-zinc-400 hover:bg-zinc-500/20 border border-zinc-500/30'}"
              >
                INFO ({report.info_count})
              </button>
            </div>
          </div>

          <!-- Finding Cards Search & Filters -->
          <div class="p-3 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none flex flex-col sm:flex-row items-center justify-between gap-3 transition-colors">
            <div class="relative w-full sm:flex-1">
              <Search class="w-3.5 h-3.5 text-[var(--color-text-muted)] absolute inset-y-0 left-3 my-auto pointer-events-none" />
              <input
                type="text"
                bind:value={searchQuery}
                placeholder="SEARCH FINDINGS, CVES, OWASP CATEGORIES..."
                class="w-full pl-8 pr-3 py-1.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs text-[var(--color-text-body)] placeholder-[var(--color-text-muted)] font-mono focus:outline-none uppercase"
              />
            </div>

            <div class="flex items-center gap-2 w-full sm:w-auto">
              <select
                bind:value={selectedCategory}
                class="w-full sm:w-auto px-3 py-1.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs text-[var(--color-text-body)] font-mono uppercase tracking-wider focus:outline-none cursor-pointer"
              >
                {#each categories as cat}
                  <option value={cat.id}>{cat.label.toUpperCase()}</option>
                {/each}
              </select>

              <div class="flex items-center gap-1">
                <ArrowUpDown class="w-3.5 h-3.5 text-[var(--color-text-muted)] flex-shrink-0" />
                <select
                  bind:value={sortFindingsBy}
                  class="px-3 py-1.5 bg-[var(--color-canvas)] border border-[var(--color-hairline)] focus:border-[var(--color-hairline-strong)] rounded-none text-xs text-[var(--color-text-body)] font-mono uppercase tracking-wider focus:outline-none cursor-pointer"
                >
                  <option value="severity">SORT: SEVERITY</option>
                  <option value="title">SORT: TITLE (A-Z)</option>
                  <option value="category">SORT: CATEGORY</option>
                </select>
              </div>
            </div>
          </div>

          <!-- Findings List -->
          <div class="space-y-3">
            {#if filteredFindings.length === 0}
              <div class="py-16 text-center bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-2">
                <CheckCircle2 class="w-10 h-10 text-emerald-500 mx-auto" />
                <h3 class="text-sm font-bold uppercase tracking-wider font-mono text-[var(--color-text-headline)]">No findings matching active criteria</h3>
              </div>
            {:else}
              {#each filteredFindings as finding (finding.id)}
                <FindingCard {finding} />
              {/each}
            {/if}
          </div>
        </div>

      <!-- 10. EMPTY DASHBOARD / STANDBY WORKSTATION -->
      {:else}
        <div class="my-auto max-w-2xl mx-auto w-full py-12 text-center space-y-6 animate-fade-in">
          <div class="w-14 h-14 rounded-none bg-[var(--color-surface)] border border-[var(--color-hairline)] border-l-4 border-l-[var(--color-signal-red)] flex items-center justify-center text-[var(--color-text-headline)] mx-auto">
            <ShieldCheck class="w-7 h-7" />
          </div>

          <div class="space-y-2">
            <div class="text-[10px] font-mono font-bold tracking-widest text-[var(--color-signal-red)] uppercase">
              00/STANDBY WORKSTATION
            </div>
            <h1 class="text-3xl sm:text-4xl font-black font-sans text-[var(--color-text-headline)] tracking-tight uppercase">
              VulnRadar Security Workstation
            </h1>
            <p class="text-xs text-[var(--color-text-muted)] max-w-md mx-auto leading-relaxed font-mono">
              Multi-threaded passive reconnaissance, automated TCP port discovery, email/DNS anti-spoofing verification, and continuous posture monitoring.
            </p>
          </div>

          <!-- Central Instant Input -->
          <form
            onsubmit={(e) => {
              e.preventDefault();
              handleScan();
            }}
            class="flex items-center gap-2 p-2 bg-[var(--color-surface)] border border-[var(--color-hairline)] focus-within:border-[var(--color-hairline-strong)] rounded-none transition-all max-w-xl mx-auto"
          >
            <div class="pl-2 text-[var(--color-text-muted)]">
              <Search class="w-4 h-4" />
            </div>
            <input
              type="text"
              bind:value={targetUrl}
              placeholder="ENTER TARGET DOMAIN (E.G. EXAMPLE.COM OR HTTP://LOCALHOST:8000)..."
              class="w-full py-2 bg-transparent text-xs font-mono text-[var(--color-text-headline)] placeholder-[var(--color-text-muted)] focus:outline-none uppercase"
            />
            <button
              type="submit"
              disabled={!targetUrl.trim() || isScanning}
              class="px-5 py-2.5 bg-[var(--color-signal-red)] hover:opacity-90 disabled:opacity-40 text-white font-mono font-bold text-xs uppercase tracking-wider rounded-none transition-all cursor-pointer flex-shrink-0"
            >
              [AUDIT TARGET]
            </button>
          </form>

          <!-- Quick Presets -->
          <div class="flex flex-wrap items-center justify-center gap-2 pt-1">
            <span class="text-[10px] text-[var(--color-text-muted)] font-mono uppercase tracking-widest font-bold mr-1">QUICK TARGET:</span>
            {#each ["example.com", "httpbin.org", "testphp.vulnweb.com", "localhost:8000"] as preset}
              <button
                type="button"
                onclick={() => {
                  const url = preset.startsWith("http")
                    ? preset
                    : preset.includes("localhost") || preset.includes("127.0.0.1")
                    ? `http://${preset}`
                    : `https://${preset}`;
                  targetUrl = url;
                  handleScan(url);
                }}
                class="px-3 py-1 bg-[var(--color-surface)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-hairline)] rounded-none text-xs font-mono text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] transition-colors cursor-pointer uppercase"
              >
                {preset}
              </button>
            {/each}
          </div>

          <!-- Native Engine Status Telemetry Widget -->
          <div class="grid grid-cols-3 gap-2 max-w-xl mx-auto pt-4 text-left">
            <div class="p-3 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-1">
              <div class="text-[9px] font-mono uppercase tracking-widest text-[var(--color-text-muted)] font-bold">SCANNER ENGINE</div>
              <div class="text-xs font-bold font-mono text-[var(--color-text-headline)]">RUST CORE v0.7.0</div>
            </div>
            <div class="p-3 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-1">
              <div class="text-[9px] font-mono uppercase tracking-widest text-[var(--color-text-muted)] font-bold">PORT PROBING</div>
              <div class="text-xs font-bold font-mono text-emerald-500">TOP 20 (AUTO)</div>
            </div>
            <div class="p-3 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none space-y-1">
              <div class="text-[9px] font-mono uppercase tracking-widest text-[var(--color-text-muted)] font-bold">LOCAL PERSISTENCE</div>
              <div class="text-xs font-bold font-mono text-[var(--color-text-headline)]">SQLITE WAL</div>
            </div>
          </div>
        </div>
      {/if}
    </main>
  </div>

  <!-- Native Desktop Footbar / Bottom Status Bar -->
  <footer
    class="h-6 bg-[var(--color-surface)] border-t border-[var(--color-hairline)] px-4 flex items-center justify-between text-[10px] font-mono text-[var(--color-text-muted)] desktop-select-none flex-shrink-0 z-30 print:hidden uppercase tracking-wider"
  >
    <div class="flex items-center gap-3">
      <span class="flex items-center gap-1.5 text-[var(--color-text-headline)] font-bold">
        <span class="w-1.5 h-1.5 rounded-none bg-emerald-500"></span>
        VULNRADAR v0.7.0
      </span>
      <span class="text-[var(--color-hairline-strong)]">/</span>
      <span>STORAGE: SQLITE WAL</span>
      <span class="text-[var(--color-hairline-strong)]">/</span>
      <span>PORTS: TOP 20</span>
    </div>

    <div class="flex items-center gap-3">
      {#if report}
        <span class="text-[var(--color-text-body)]">RESPONSE: {report.response_time_ms} MS</span>
        <span class="text-[var(--color-hairline-strong)]">/</span>
      {/if}
      <span>MONITORS: {monitors.filter((m) => m.is_active).length} ACTIVE</span>
      <span class="text-[var(--color-hairline-strong)]">/</span>
      <span>SHORTCUTS: <kbd class="px-1 text-[9px] font-mono bg-[var(--color-canvas)] border border-[var(--color-hairline)] rounded-none text-[var(--color-text-headline)]">?</kbd></span>
    </div>
  </footer>
</div>

<!-- Modals & Drawers -->
<SettingsModal
  isOpen={isSettingsOpen}
  activeTab={settingsTab}
  options={scanOptions}
  {monitors}
  historyCount={history.length}
  onApplyOptions={(newOpts) => {
    scanOptions = newOpts;
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

<HistoryModal
  isOpen={isHistoryOpen}
  {history}
  onSelect={(id: string) => {
    isHistoryOpen = false;
    handleSelectHistoryScan(id);
  }}
  onDelete={handleDeleteScan}
  onClearAll={handleClearAllHistory}
  onClose={() => (isHistoryOpen = false)}
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

<ScanOptionsModal
  isOpen={isOptionsOpen}
  options={scanOptions}
  onApply={(newOpts) => {
    scanOptions = newOpts;
    showToast("Scan parameters applied", "success");
    isOptionsOpen = false;
  }}
  onClose={() => (isOptionsOpen = false)}
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
