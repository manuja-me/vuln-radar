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
      class="w-56 bg-[#111216] border-r border-white/[0.08] flex flex-col justify-between flex-shrink-0 desktop-select-none print:hidden"
    >
      <!-- Navigation Workspaces -->
      <div class="p-2 space-y-1 overflow-y-auto">
        <div class="px-2.5 py-1.5 text-[10px] font-bold font-mono uppercase tracking-wider text-neutral-400">
          Workspaces
        </div>

        <!-- 1. Audit Workspace -->
        <button
          type="button"
          onclick={() => (currentWorkspace = "audit")}
          class="w-full flex items-center justify-between px-2.5 py-2 rounded-md text-xs font-medium transition-colors cursor-pointer {currentWorkspace === 'audit' ? 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/20 font-semibold' : 'text-neutral-300 hover:bg-white/[0.04] hover:text-white border border-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <ShieldCheck class="w-4 h-4 {currentWorkspace === 'audit' ? 'text-cyan-400' : 'text-neutral-400'}" />
            <span class="truncate">Posture Audit</span>
          </div>
          {#if report}
            <span class="px-1.5 py-0.5 text-[10px] font-mono rounded bg-white/[0.06] text-neutral-300">
              {report.findings.length}
            </span>
          {/if}
        </button>

        <!-- 2. Port Discovery Workspace -->
        <button
          type="button"
          onclick={() => (currentWorkspace = "ports")}
          class="w-full flex items-center justify-between px-2.5 py-2 rounded-md text-xs font-medium transition-colors cursor-pointer {currentWorkspace === 'ports' ? 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/20 font-semibold' : 'text-neutral-300 hover:bg-white/[0.04] hover:text-white border border-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <Server class="w-4 h-4 {currentWorkspace === 'ports' ? 'text-cyan-400' : 'text-neutral-400'}" />
            <span class="truncate">Port Matrix</span>
          </div>
          {#if report?.port_report}
            <span class="px-1.5 py-0.5 text-[10px] font-mono rounded {report.port_report.open_ports_count > 0 ? 'bg-emerald-950/40 text-emerald-300 border border-emerald-800/40' : 'bg-white/[0.06] text-neutral-400'}">
              {report.port_report.open_ports_count}
            </span>
          {/if}
        </button>

        <!-- 3. DNS & Email Security Workspace -->
        <button
          type="button"
          onclick={() => (currentWorkspace = "dns")}
          class="w-full flex items-center justify-between px-2.5 py-2 rounded-md text-xs font-medium transition-colors cursor-pointer {currentWorkspace === 'dns' ? 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/20 font-semibold' : 'text-neutral-300 hover:bg-white/[0.04] hover:text-white border border-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <Mail class="w-4 h-4 {currentWorkspace === 'dns' ? 'text-cyan-400' : 'text-neutral-400'}" />
            <span class="truncate">DNS & Anti-Spoof</span>
          </div>
          {#if report?.dns_security?.spf_record}
            <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
          {/if}
        </button>

        <!-- 4. Recon & Surface Workspace -->
        <button
          type="button"
          onclick={() => (currentWorkspace = "recon")}
          class="w-full flex items-center justify-between px-2.5 py-2 rounded-md text-xs font-medium transition-colors cursor-pointer {currentWorkspace === 'recon' ? 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/20 font-semibold' : 'text-neutral-300 hover:bg-white/[0.04] hover:text-white border border-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <Globe class="w-4 h-4 {currentWorkspace === 'recon' ? 'text-cyan-400' : 'text-neutral-400'}" />
            <span class="truncate">Surface Recon</span>
          </div>
          {#if report?.subdomains?.length}
            <span class="px-1.5 py-0.5 text-[10px] font-mono rounded bg-white/[0.06] text-neutral-300">
              {report.subdomains.length}
            </span>
          {/if}
        </button>

        <div class="pt-3 pb-1 px-2.5 text-[10px] font-bold font-mono uppercase tracking-wider text-neutral-400">
          Fleet Tools
        </div>

        <!-- 5. Batch Fleet Scanner -->
        <button
          type="button"
          onclick={() => {
            currentWorkspace = "batch";
            isBatchOpen = true;
          }}
          class="w-full flex items-center justify-between px-2.5 py-2 rounded-md text-xs font-medium transition-colors cursor-pointer {currentWorkspace === 'batch' ? 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/20 font-semibold' : 'text-neutral-300 hover:bg-white/[0.04] hover:text-white border border-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <Layers class="w-4 h-4 {currentWorkspace === 'batch' ? 'text-cyan-400' : 'text-neutral-400'}" />
            <span class="truncate">Fleet Batch</span>
          </div>
          <span class="text-[10px] font-mono text-neutral-400">⌘B</span>
        </button>

        <!-- 6. Watchdog Monitor -->
        <button
          type="button"
          onclick={() => (currentWorkspace = "watchdog")}
          class="w-full flex items-center justify-between px-2.5 py-2 rounded-md text-xs font-medium transition-colors cursor-pointer {currentWorkspace === 'watchdog' ? 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/20 font-semibold' : 'text-neutral-300 hover:bg-white/[0.04] hover:text-white border border-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <Activity class="w-4 h-4 {currentWorkspace === 'watchdog' ? 'text-cyan-400' : 'text-neutral-400'}" />
            <span class="truncate">Watchdog</span>
          </div>
          {#if monitors.filter((m) => m.is_active).length > 0}
            <span class="px-1.5 py-0.5 text-[10px] font-mono rounded bg-cyan-950/40 text-cyan-300 border border-cyan-800/40 font-bold">
              {monitors.filter((m) => m.is_active).length}
            </span>
          {/if}
        </button>

        <!-- 7. History & Database Logs -->
        <button
          type="button"
          onclick={() => (currentWorkspace = "history")}
          class="w-full flex items-center justify-between px-2.5 py-2 rounded-md text-xs font-medium transition-colors cursor-pointer {currentWorkspace === 'history' ? 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/20 font-semibold' : 'text-neutral-300 hover:bg-white/[0.04] hover:text-white border border-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <HardDrive class="w-4 h-4 {currentWorkspace === 'history' ? 'text-cyan-400' : 'text-neutral-400'}" />
            <span class="truncate">Scan Logs</span>
          </div>
          <span class="px-1.5 py-0.5 text-[10px] font-mono rounded bg-white/[0.06] text-neutral-400">
            {history.length}
          </span>
        </button>

        <!-- 8. Preferences & Settings -->
        <button
          type="button"
          onclick={() => (currentWorkspace = "settings")}
          class="w-full flex items-center justify-between px-2.5 py-2 rounded-md text-xs font-medium transition-colors cursor-pointer {currentWorkspace === 'settings' ? 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/20 font-semibold' : 'text-neutral-300 hover:bg-white/[0.04] hover:text-white border border-transparent'}"
        >
          <div class="flex items-center gap-2.5 truncate">
            <Sliders class="w-4 h-4 {currentWorkspace === 'settings' ? 'text-cyan-400' : 'text-neutral-400'}" />
            <span class="truncate">Settings</span>
          </div>
          {#if hasCustomOptions}
            <span class="w-2 h-2 rounded-full bg-cyan-400"></span>
          {/if}
        </button>
      </div>

      <!-- Bottom Sidebar System Widget -->
      <div class="p-3 border-t border-white/[0.08] space-y-2 bg-[#0e0f13]">
        <button
          type="button"
          onclick={() => (isShortcutsOpen = true)}
          class="w-full flex items-center justify-between px-2.5 py-1.5 rounded bg-white/[0.03] hover:bg-white/[0.06] border border-white/[0.06] text-[11px] text-neutral-400 hover:text-neutral-200 transition-colors cursor-pointer"
        >
          <span class="flex items-center gap-1.5">
            <Keyboard class="w-3.5 h-3.5 text-neutral-400" />
            Shortcuts
          </span>
          <kbd class="px-1 py-0.2 text-[9px] font-mono bg-[#181920] border border-white/[0.08] rounded text-neutral-400">?</kbd>
        </button>

        <div class="flex items-center justify-between text-[10px] font-mono text-neutral-400 px-1">
          <span class="flex items-center gap-1">
            <Database class="w-3 h-3 text-neutral-400" />
            SQLite WAL
          </span>
          <span class="text-emerald-400">Connected</span>
        </div>
      </div>
    </aside>

    <!-- Main Desktop Workstation Content Area -->
    <main class="flex-1 overflow-y-auto bg-[#0d0e11] flex flex-col p-5">
      <!-- 1. SCANNING PROGRESS HUD -->
      {#if isScanning}
        <div class="my-auto max-w-lg mx-auto w-full p-6 bg-[#13141a] border border-white/[0.08] rounded-xl shadow-2xl space-y-5 text-center animate-fade-in">
          <div class="w-12 h-12 rounded-xl bg-cyan-500/10 border border-cyan-500/20 flex items-center justify-center text-cyan-400 mx-auto">
            <Loader2 class="w-6 h-6 animate-spin" />
          </div>

          <div class="space-y-1.5">
            <div class="inline-flex items-center gap-2 px-2.5 py-0.5 rounded-full bg-cyan-950/40 border border-cyan-800/40 text-cyan-300 text-[11px] font-mono font-medium">
              <span class="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-pulse"></span>
              <span>Running Multi-Threaded Security Audit</span>
            </div>
            <h2 class="text-lg font-bold text-white tracking-tight">Auditing Target Surface</h2>
            <div class="p-2 bg-[#1a1b22] rounded-lg border border-white/[0.06] text-xs font-mono text-cyan-300 truncate max-w-sm mx-auto">
              {targetUrl}
            </div>
          </div>

          <!-- Active Pipeline Modules Checklist -->
          <div class="grid grid-cols-2 gap-2 text-left text-xs font-mono pt-2 border-t border-white/[0.06]">
            <div class="flex items-center gap-2 text-neutral-300">
              <CheckCircle2 class="w-3.5 h-3.5 text-cyan-400" />
              <span>HTTP Headers</span>
            </div>
            <div class="flex items-center gap-2 text-neutral-300">
              <CheckCircle2 class="w-3.5 h-3.5 text-cyan-400" />
              <span>Port Discovery (Auto)</span>
            </div>
            <div class="flex items-center gap-2 text-neutral-300">
              <CheckCircle2 class="w-3.5 h-3.5 text-cyan-400" />
              <span>DoH Anti-Spoof</span>
            </div>
            <div class="flex items-center gap-2 text-neutral-300">
              <CheckCircle2 class="w-3.5 h-3.5 text-cyan-400" />
              <span>SCA CVE Library</span>
            </div>
          </div>
        </div>

      <!-- 2. SCAN ERROR HUD -->
      {:else if scanError}
        <div class="my-auto max-w-xl mx-auto w-full p-6 bg-red-950/20 border border-red-900/40 rounded-xl space-y-4 text-red-300 animate-fade-in shadow-xl">
          <div class="flex items-start gap-3">
            <AlertOctagon class="w-6 h-6 text-red-400 flex-shrink-0 mt-0.5" />
            <div class="space-y-1.5 flex-1">
              <h3 class="text-sm font-bold text-red-200 uppercase font-mono tracking-wider">Audit Execution Failed</h3>
              <p class="text-xs text-red-300/90 font-mono bg-[#15161c] p-3 rounded-lg border border-red-900/30 break-all leading-relaxed">
                {scanError}
              </p>
            </div>
          </div>
          <div class="flex items-center justify-end gap-2 pt-2 border-t border-red-900/30">
            <button
              type="button"
              onclick={() => handleScan()}
              class="px-4 py-1.5 bg-red-500 hover:bg-red-400 text-slate-950 rounded-md text-xs font-bold transition-colors cursor-pointer shadow-sm"
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
            <div class="p-3.5 bg-[#14151b] border border-white/[0.08] rounded-xl space-y-0.5">
              <span class="text-[10px] font-bold uppercase tracking-wider text-neutral-400 font-mono">Host / IP</span>
              <div class="text-sm font-bold font-mono text-neutral-100 truncate">
                {report.port_report?.ip_address || report.port_report?.host || "Resolving IP"}
              </div>
            </div>
            <div class="p-3.5 bg-[#14151b] border border-white/[0.08] rounded-xl space-y-0.5">
              <span class="text-[10px] font-bold uppercase tracking-wider text-neutral-400 font-mono">Ports Probed</span>
              <div class="text-sm font-bold font-mono text-neutral-100">
                {report.port_report?.scanned_ports_count || 0} Ports (Auto Top 20)
              </div>
            </div>
            <div class="p-3.5 bg-[#14151b] border border-white/[0.08] rounded-xl space-y-0.5">
              <span class="text-[10px] font-bold uppercase tracking-wider text-neutral-400 font-mono">Open Ports</span>
              <div class="text-sm font-bold font-mono {(report.port_report?.open_ports_count || 0) > 0 ? 'text-emerald-400' : 'text-neutral-400'}">
                {report.port_report?.open_ports_count || 0} Discovered
              </div>
            </div>
            <div class="p-3.5 bg-[#14151b] border border-white/[0.08] rounded-xl space-y-0.5">
              <span class="text-[10px] font-bold uppercase tracking-wider text-neutral-400 font-mono">Duration</span>
              <div class="text-sm font-bold font-mono text-cyan-300">
                {report.port_report?.scan_duration_ms || 0} ms
              </div>
            </div>
          </div>

          <!-- Port Search & Filter Controls -->
          <div class="p-3 bg-[#14151b] border border-white/[0.08] rounded-xl flex flex-col md:flex-row items-center justify-between gap-3">
            <div class="relative w-full md:w-80">
              <Search class="w-3.5 h-3.5 text-neutral-500 absolute inset-y-0 left-3 my-auto pointer-events-none" />
              <input
                type="text"
                bind:value={portSearchQuery}
                placeholder="Filter by port #, service, banner..."
                class="w-full pl-8 pr-3 py-1.5 bg-[#1b1c24] border border-white/[0.08] focus:border-cyan-500/80 rounded-md text-xs text-neutral-200 placeholder-neutral-500 font-mono focus:outline-none"
              />
            </div>

            <div class="flex items-center gap-1.5 w-full md:w-auto">
              <button
                type="button"
                onclick={() => (portRiskFilter = "all")}
                class="px-2.5 py-1 rounded-md text-xs font-medium cursor-pointer transition-colors {portRiskFilter === 'all' ? 'bg-white text-slate-950 font-bold' : 'bg-[#1b1c24] text-neutral-400 hover:text-neutral-200'}"
              >
                All ({report.port_report?.open_ports.length || 0})
              </button>
              <button
                type="button"
                onclick={() => (portRiskFilter = "risky")}
                class="px-2.5 py-1 rounded-md text-xs font-medium cursor-pointer transition-colors {portRiskFilter === 'risky' ? 'bg-red-950 text-red-200 border border-red-800' : 'bg-red-950/20 text-red-400 hover:bg-red-950/40'}"
              >
                Risky ({report.port_report?.open_ports.filter((p) => p.is_risky).length || 0})
              </button>
              <button
                type="button"
                onclick={() => (portRiskFilter = "standard")}
                class="px-2.5 py-1 rounded-md text-xs font-medium cursor-pointer transition-colors {portRiskFilter === 'standard' ? 'bg-[#22232c] text-white border border-white/[0.1]' : 'bg-[#1b1c24] text-neutral-400 hover:text-neutral-200'}"
              >
                Standard ({report.port_report?.open_ports.filter((p) => !p.is_risky).length || 0})
              </button>
            </div>
          </div>

          <!-- Open Ports Cards -->
          {#if filteredOpenPorts.length === 0}
            <div class="py-16 text-center bg-[#14151b] border border-white/[0.08] rounded-xl space-y-2">
              <CheckCircle2 class="w-10 h-10 text-emerald-400 mx-auto opacity-80" />
              <h3 class="text-sm font-semibold text-neutral-200">No Open Ports Matching Criteria</h3>
              <p class="text-xs text-neutral-400 max-w-sm mx-auto">
                No active listening services were detected on scanned ports.
              </p>
            </div>
          {:else}
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
              {#each filteredOpenPorts as p (p.port)}
                <div
                  class="p-4 bg-[#14151b] border rounded-xl space-y-3 transition-colors {p.is_risky ? 'border-red-800/40 bg-red-950/10' : 'border-white/[0.08] hover:border-white/[0.15]'}"
                >
                  <div class="flex items-center justify-between gap-2">
                    <div class="flex items-center gap-2">
                      <span class="px-2 py-0.5 text-xs font-mono font-bold rounded bg-[#1a1b23] border border-white/[0.08] text-neutral-100">
                        PORT {p.port}/{p.protocol.toUpperCase()}
                      </span>
                      <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[10px] font-mono bg-emerald-950/40 text-emerald-300 border border-emerald-800/50 font-bold">
                        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                        OPEN
                      </span>
                    </div>

                    {#if p.is_risky}
                      <span class="px-2 py-0.5 text-[10px] font-mono rounded bg-red-950/40 text-red-300 border border-red-800/50 flex items-center gap-1 font-bold">
                        <AlertTriangle class="w-3 h-3 text-red-400" />
                        EXPOSED / RISKY
                      </span>
                    {/if}
                  </div>

                  <div class="space-y-0.5">
                    <div class="text-sm font-bold text-white flex items-center gap-1.5">
                      <Server class="w-3.5 h-3.5 text-neutral-400" />
                      <span>{p.service}</span>
                    </div>
                    <p class="text-xs text-neutral-400">{p.description}</p>
                  </div>

                  {#if p.banner}
                    <div class="p-2.5 bg-[#111216] rounded-lg border border-white/[0.06] space-y-1">
                      <span class="text-[9px] font-mono text-neutral-400 uppercase tracking-wider block">Service Banner</span>
                      <pre class="text-xs font-mono text-neutral-200 overflow-x-auto whitespace-pre-wrap">{p.banner}</pre>
                    </div>
                  {/if}

                  <div class="pt-2 border-t border-white/[0.06] flex items-center justify-between">
                    <div class="text-[11px] font-mono text-neutral-400">
                      {report.port_report?.host}:{p.port}
                    </div>

                    <div class="flex items-center gap-1.5">
                      <button
                        type="button"
                        onclick={() => copyPortAddress(report!.port_report!.host, p.port)}
                        class="px-2 py-0.5 bg-[#1f2029] hover:bg-[#282a36] text-neutral-300 hover:text-white rounded text-xs font-mono transition-colors cursor-pointer flex items-center gap-1"
                      >
                        {#if copiedPort === p.port}
                          <CheckCircle2 class="w-3 h-3 text-emerald-400" />
                          <span class="text-emerald-400">Copied</span>
                        {:else}
                          <span>Copy Address</span>
                        {/if}
                      </button>

                      {#if [80, 443, 3000, 5000, 8000, 8080, 8081, 8443, 8888, 9000, 9090].includes(p.port)}
                        <a
                          href={`${p.port === 443 || p.port === 8443 ? "https" : "http"}://${report?.port_report?.host || "localhost"}:${p.port}`}
                          target="_blank"
                          rel="noopener noreferrer"
                          class="p-1 bg-[#1f2029] hover:bg-[#282a36] text-neutral-400 hover:text-white rounded text-xs transition-colors cursor-pointer"
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
              <div class="p-4 bg-[#14151b] border border-white/[0.08] rounded-xl space-y-2">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-bold text-neutral-200 font-mono">SPF Anti-Spoofing</span>
                  <span class="px-2 py-0.5 text-xs font-mono rounded font-bold {report.dns_security.spf_record ? 'bg-emerald-950/40 text-emerald-300 border border-emerald-800/50' : 'bg-red-950/40 text-red-300 border border-red-800/50'}">
                    {report.dns_security.spf_record ? "Configured" : "Missing"}
                  </span>
                </div>
                <p class="text-xs text-neutral-400">Validates authorized mail senders via RFC 7208.</p>
                {#if report.dns_security.spf_record}
                  <pre class="bg-[#101115] p-2.5 rounded-lg text-xs font-mono text-cyan-300 border border-white/[0.06] overflow-x-auto whitespace-pre-wrap">{report.dns_security.spf_record}</pre>
                {/if}
              </div>

              <div class="p-4 bg-[#14151b] border border-white/[0.08] rounded-xl space-y-2">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-bold text-neutral-200 font-mono">DMARC Enforcement</span>
                  <span class="px-2 py-0.5 text-xs font-mono rounded font-bold {report.dns_security.dmarc_policy && report.dns_security.dmarc_policy !== 'none' ? 'bg-emerald-950/40 text-emerald-300 border border-emerald-800/50' : 'bg-amber-950/40 text-amber-300 border border-amber-800/50'}">
                    {report.dns_security.dmarc_policy || "None"}
                  </span>
                </div>
                <p class="text-xs text-neutral-400">Enforces domain-based email authentication & alignment.</p>
                {#if report.dns_security.dmarc_record}
                  <pre class="bg-[#101115] p-2.5 rounded-lg text-xs font-mono text-cyan-300 border border-white/[0.06] overflow-x-auto whitespace-pre-wrap">{report.dns_security.dmarc_record}</pre>
                {/if}
              </div>

              <div class="p-4 bg-[#14151b] border border-white/[0.08] rounded-xl space-y-2">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-bold text-neutral-200 font-mono">DNSSEC Validation</span>
                  <span class="px-2 py-0.5 text-xs font-mono rounded font-bold {report.dns_security.dnssec_enabled ? 'bg-emerald-950/40 text-emerald-300 border border-emerald-800/50' : 'bg-[#1e1f28] text-neutral-400'}">
                    {report.dns_security.dnssec_enabled ? "Enabled" : "Disabled"}
                  </span>
                </div>
                <p class="text-xs text-neutral-400">Cryptographically authenticates DNS records against poisoning.</p>
              </div>
            </div>
          {/if}
        </div>

      <!-- 5. RECON & SURFACE WORKSPACE VIEW -->
      {:else if currentWorkspace === "recon" && report}
        <div class="space-y-4 max-w-6xl w-full mx-auto animate-fade-in">
          <div class="p-3 bg-[#14151b] border border-white/[0.08] rounded-xl flex items-center justify-between gap-3">
            <div class="relative w-full max-w-sm">
              <Search class="w-3.5 h-3.5 text-neutral-500 absolute inset-y-0 left-3 my-auto pointer-events-none" />
              <input
                type="text"
                bind:value={subdomainSearch}
                placeholder="Filter discovered subdomains..."
                class="w-full pl-8 pr-3 py-1.5 bg-[#1b1c24] border border-white/[0.08] focus:border-cyan-500/80 rounded-md text-xs text-neutral-200 placeholder-neutral-500 font-mono focus:outline-none"
              />
            </div>
            <span class="text-xs font-mono text-neutral-400">
              {filteredSubdomains.length} / {report.subdomains?.length || 0} Discovered
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
                  class="p-2.5 bg-[#14151b] hover:bg-[#1a1c24] border border-white/[0.08] hover:border-cyan-500/40 rounded-lg text-left text-xs font-mono text-neutral-300 hover:text-white transition-colors cursor-pointer truncate flex items-center justify-between group"
                >
                  <span class="truncate">{sub}</span>
                  <ExternalLink class="w-3.5 h-3.5 text-neutral-500 group-hover:text-cyan-400 flex-shrink-0 ml-2" />
                </button>
              {/each}
            </div>
          {/if}
        </div>

      <!-- 6. SCAN LOGS / HISTORY WORKSPACE VIEW -->
      {:else if currentWorkspace === "history"}
        <div class="space-y-4 max-w-6xl w-full mx-auto animate-fade-in">
          <div class="flex items-center justify-between pb-2 border-b border-white/[0.08]">
            <div class="flex items-center gap-2">
              <HardDrive class="w-4 h-4 text-cyan-400" />
              <h2 class="text-sm font-bold text-white font-mono uppercase">Local SQLite Audit Logs ({history.length})</h2>
            </div>
            {#if history.length > 0}
              <button
                type="button"
                onclick={handleClearAllHistory}
                class="px-2.5 py-1 bg-red-950/40 hover:bg-red-900/60 text-red-300 border border-red-800/50 rounded text-xs font-mono transition-colors cursor-pointer"
              >
                Clear History
              </button>
            {/if}
          </div>

          {#if history.length === 0}
            <div class="py-16 text-center bg-[#14151b] border border-white/[0.08] rounded-xl text-neutral-400 text-xs">
              No historical scan logs recorded yet.
            </div>
          {:else}
            <div class="space-y-2">
              {#each history as item (item.id)}
                <div class="p-3 bg-[#14151b] border border-white/[0.08] rounded-xl flex items-center justify-between gap-4 hover:border-white/[0.15] transition-colors">
                  <div class="space-y-1 min-w-0 flex-1">
                    <div class="flex items-center gap-2">
                      <span class="font-bold text-white font-mono text-xs truncate">{item.target_url}</span>
                      <span class="px-1.5 py-0.2 text-[10px] font-mono rounded bg-white/[0.06] text-neutral-400">
                        Score: {item.security_score}/100
                      </span>
                    </div>
                    <div class="text-[11px] font-mono text-neutral-400">
                      {new Date(item.scanned_at).toLocaleString()} • {item.total_findings} findings ({item.critical_count} critical)
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <button
                      type="button"
                      onclick={() => handleSelectHistoryScan(item.id)}
                      class="px-3 py-1 bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold rounded text-xs transition-colors cursor-pointer"
                    >
                      Load
                    </button>
                    <button
                      type="button"
                      onclick={() => handleDeleteScan(item.id)}
                      class="p-1 text-neutral-500 hover:text-red-400 rounded cursor-pointer"
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
          <div class="flex items-center justify-between pb-2 border-b border-white/[0.08]">
            <div class="flex items-center gap-2">
              <Activity class="w-4 h-4 text-cyan-400" />
              <h2 class="text-sm font-bold text-white font-mono uppercase">Automated Watchdog Daemon</h2>
            </div>
            <button
              type="button"
              onclick={() => (isMonitorsOpen = true)}
              class="px-3 py-1 bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold rounded text-xs transition-colors cursor-pointer"
            >
              + Add Target
            </button>
          </div>

          {#if monitors.length === 0}
            <div class="py-16 text-center bg-[#14151b] border border-white/[0.08] rounded-xl space-y-3">
              <Activity class="w-10 h-10 text-cyan-400 mx-auto opacity-70" />
              <h3 class="text-sm font-bold text-white">No Monitored Targets Active</h3>
              <p class="text-xs text-neutral-400 max-w-sm mx-auto">
                Schedule targets for continuous re-auditing (1h, 6h, 12h, 24h) with native desktop alerts upon score degradation.
              </p>
            </div>
          {:else}
            <div class="space-y-2">
              {#each monitors as m (m.id)}
                <div class="p-3.5 bg-[#14151b] border border-white/[0.08] rounded-xl flex items-center justify-between gap-4">
                  <div class="space-y-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <span class="font-bold text-white font-mono text-xs">{m.target_url}</span>
                      <span class="px-1.5 py-0.2 text-[10px] font-mono rounded bg-white/[0.06] text-neutral-300">
                        Every {m.interval_hours}h
                      </span>
                    </div>
                    <div class="text-[11px] font-mono text-neutral-400">
                      Next Run: {new Date(m.next_scan_at).toLocaleTimeString()} • Last Score: {m.last_score ?? "Pending"}
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <button
                      type="button"
                      onclick={() => handleToggleMonitor(m.id)}
                      class="px-2.5 py-1 rounded text-xs font-mono font-bold transition-colors cursor-pointer {m.is_active ? 'bg-emerald-950/40 text-emerald-300 border border-emerald-800/50' : 'bg-[#1b1c24] text-neutral-400'}"
                    >
                      {m.is_active ? "Active" : "Paused"}
                    </button>
                    <button
                      type="button"
                      onclick={() => handleDeleteMonitor(m.id)}
                      class="p-1 text-neutral-500 hover:text-red-400 rounded cursor-pointer"
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
          <div class="p-5 bg-[#14151b] border border-white/[0.08] rounded-xl space-y-4">
            <div class="flex items-center justify-between pb-3 border-b border-white/[0.08]">
              <div>
                <h2 class="text-sm font-bold text-white font-mono uppercase">Scanner Configuration</h2>
                <p class="text-xs text-neutral-400">Configure global scan parameters, HTTP headers, and port probe profiles.</p>
              </div>
              <button
                type="button"
                onclick={() => {
                  settingsTab = "params";
                  isSettingsOpen = true;
                }}
                class="px-3 py-1.5 bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold rounded text-xs transition-colors cursor-pointer"
              >
                Open Full Settings Hub
              </button>
            </div>

            <!-- Fast Port Scanner Preset Selector -->
            <div class="p-3 bg-[#111216] border border-white/[0.06] rounded-lg space-y-2">
              <div class="flex items-center justify-between">
                <span class="text-xs font-bold text-white font-mono flex items-center gap-1.5">
                  <Server class="w-3.5 h-3.5 text-cyan-400" />
                  Automatic Port Discovery Profile
                </span>
                <span class="text-[10px] font-mono text-emerald-400">Default: Top 20</span>
              </div>
              <p class="text-xs text-neutral-400">
                Ports are scanned concurrently with sub-second probes whenever a target audit is executed.
              </p>
              <div class="grid grid-cols-3 gap-2 pt-1">
                <button
                  type="button"
                  onclick={() => (scanOptions.port_scan_profile = "top20")}
                  class="py-1.5 px-2 rounded text-xs font-mono font-medium border transition-colors cursor-pointer {scanOptions.port_scan_profile === 'top20' ? 'bg-cyan-500/20 text-cyan-300 border-cyan-500/50 font-bold' : 'bg-[#171820] text-neutral-400 border-white/[0.06]'}"
                >
                  Top 20 (Fastest)
                </button>
                <button
                  type="button"
                  onclick={() => (scanOptions.port_scan_profile = "databases")}
                  class="py-1.5 px-2 rounded text-xs font-mono font-medium border transition-colors cursor-pointer {scanOptions.port_scan_profile === 'databases' ? 'bg-cyan-500/20 text-cyan-300 border-cyan-500/50 font-bold' : 'bg-[#171820] text-neutral-400 border-white/[0.06]'}"
                >
                  Databases
                </button>
                <button
                  type="button"
                  onclick={() => (scanOptions.port_scan_profile = "top100")}
                  class="py-1.5 px-2 rounded text-xs font-mono font-medium border transition-colors cursor-pointer {scanOptions.port_scan_profile === 'top100' ? 'bg-cyan-500/20 text-cyan-300 border-cyan-500/50 font-bold' : 'bg-[#171820] text-neutral-400 border-white/[0.06]'}"
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
          <div class="p-4 bg-[#14151b] border border-white/[0.08] rounded-xl flex flex-col lg:flex-row items-start lg:items-center justify-between gap-4 shadow-lg">
            <div class="space-y-1.5 min-w-0 flex-1">
              <div class="flex flex-wrap items-center gap-2">
                <span class="px-2 py-0.5 text-xs font-mono font-bold bg-emerald-950/40 text-emerald-300 border border-emerald-800/50 rounded">
                  HTTP {report.status_code}
                </span>
                <span class="text-xs text-neutral-400 font-mono">
                  Latency: <strong class="text-neutral-200">{report.response_time_ms} ms</strong>
                </span>
                <span class="text-xs text-neutral-600">•</span>
                <span class="text-xs text-neutral-400 font-mono">
                  Audited at {new Date(report.scanned_at).toLocaleTimeString()}
                </span>

                {#if previousScan}
                  <span class="text-xs text-neutral-600">•</span>
                  <div class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-mono {report.security_score >= previousScan.security_score ? 'bg-emerald-950/30 text-emerald-300 border border-emerald-800/30' : 'bg-rose-950/30 text-rose-300 border border-rose-800/30'}">
                    {#if report.security_score > previousScan.security_score}
                      <TrendingUp class="w-3 h-3 text-emerald-400" />
                      <span>+{report.security_score - previousScan.security_score} pts vs previous</span>
                    {:else if report.security_score < previousScan.security_score}
                      <TrendingDown class="w-3 h-3 text-rose-400" />
                      <span>-{previousScan.security_score - report.security_score} pts vs previous</span>
                    {/if}
                  </div>
                {/if}
              </div>

              <div class="flex items-center gap-2">
                <h1 class="text-lg font-bold text-white tracking-tight truncate flex items-center gap-2 font-mono">
                  <Globe class="w-4 h-4 text-cyan-400 flex-shrink-0" />
                  <span class="truncate">{report.target_url}</span>
                </h1>
                <div class="flex items-center gap-1">
                  <button
                    type="button"
                    onclick={copyTargetUrl}
                    class="p-1 text-neutral-400 hover:text-white hover:bg-white/[0.06] rounded cursor-pointer"
                    title="Copy URL"
                  >
                    {#if copiedUrl}
                      <CheckCircle2 class="w-3.5 h-3.5 text-emerald-400" />
                    {:else}
                      <Search class="w-3.5 h-3.5" />
                    {/if}
                  </button>
                  <button
                    type="button"
                    onclick={copyCurlCommand}
                    class="p-1 text-neutral-400 hover:text-white hover:bg-white/[0.06] rounded cursor-pointer"
                    title="Copy cURL command"
                  >
                    {#if copiedCurl}
                      <Check class="w-3.5 h-3.5 text-emerald-400" />
                    {:else}
                      <Terminal class="w-3.5 h-3.5" />
                    {/if}
                  </button>
                </div>
              </div>

              <!-- Detected Technologies -->
              {#if report.technologies_detected.length > 0}
                <div class="flex flex-wrap items-center gap-1.5 pt-0.5">
                  <span class="text-[11px] text-neutral-400 font-mono flex items-center gap-1 mr-1">
                    <Cpu class="w-3 h-3 text-cyan-400" /> Stack:
                  </span>
                  {#each report.technologies_detected as tech}
                    <span class="px-2 py-0.2 text-[11px] font-mono bg-[#1b1c24] text-neutral-300 border border-white/[0.08] rounded">
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
          <div class="p-3 bg-[#14151b] border border-white/[0.08] rounded-xl space-y-2">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span class="text-[10px] font-bold uppercase tracking-wider text-neutral-400 font-mono">
                  Severity Breakdown
                </span>
                <span class="px-1.5 py-0.2 text-[10px] font-mono bg-white/[0.06] text-neutral-300 rounded">
                  {report.total_findings} Findings
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
                  class="text-[11px] text-cyan-400 hover:underline cursor-pointer"
                >
                  Reset Filters
                </button>
              {/if}
            </div>

            <!-- Proportional Colored Bar -->
            {#if report.total_findings > 0}
              <div class="w-full h-1.5 bg-[#111216] rounded-full overflow-hidden flex gap-0.5">
                {#if report.critical_count > 0}
                  <div class="bg-rose-500 h-full rounded-full" style="width: {(report.critical_count / report.total_findings) * 100}%"></div>
                {/if}
                {#if report.high_count > 0}
                  <div class="bg-orange-500 h-full rounded-full" style="width: {(report.high_count / report.total_findings) * 100}%"></div>
                {/if}
                {#if report.medium_count > 0}
                  <div class="bg-amber-500 h-full rounded-full" style="width: {(report.medium_count / report.total_findings) * 100}%"></div>
                {/if}
                {#if report.low_count > 0}
                  <div class="bg-blue-500 h-full rounded-full" style="width: {(report.low_count / report.total_findings) * 100}%"></div>
                {/if}
                {#if report.info_count > 0}
                  <div class="bg-neutral-500 h-full rounded-full" style="width: {(report.info_count / report.total_findings) * 100}%"></div>
                {/if}
              </div>
            {/if}

            <!-- Severity Filter Chips -->
            <div class="flex flex-wrap items-center gap-1.5 pt-0.5">
              <button
                type="button"
                onclick={() => (selectedSeverity = "all")}
                class="px-2 py-0.5 rounded text-xs font-mono transition-colors cursor-pointer {selectedSeverity === 'all' ? 'bg-white text-slate-950 font-bold' : 'bg-[#1a1b23] text-neutral-400 hover:text-white border border-white/[0.06]'}"
              >
                All ({report.total_findings})
              </button>
              <button
                type="button"
                onclick={() => (selectedSeverity = "critical")}
                class="px-2 py-0.5 rounded text-xs font-mono transition-colors cursor-pointer {selectedSeverity === 'critical' ? 'bg-rose-950 text-rose-200 border border-rose-700 font-bold' : 'bg-rose-950/20 text-rose-400 hover:bg-rose-950/40 border border-rose-900/30'}"
              >
                Critical ({report.critical_count})
              </button>
              <button
                type="button"
                onclick={() => (selectedSeverity = "high")}
                class="px-2 py-0.5 rounded text-xs font-mono transition-colors cursor-pointer {selectedSeverity === 'high' ? 'bg-orange-950 text-orange-200 border border-orange-700 font-bold' : 'bg-orange-950/20 text-orange-400 hover:bg-orange-950/40 border border-orange-900/30'}"
              >
                High ({report.high_count})
              </button>
              <button
                type="button"
                onclick={() => (selectedSeverity = "medium")}
                class="px-2 py-0.5 rounded text-xs font-mono transition-colors cursor-pointer {selectedSeverity === 'medium' ? 'bg-amber-950 text-amber-200 border border-amber-700 font-bold' : 'bg-amber-950/20 text-amber-400 hover:bg-amber-950/40 border border-amber-900/30'}"
              >
                Med ({report.medium_count})
              </button>
              <button
                type="button"
                onclick={() => (selectedSeverity = "low")}
                class="px-2 py-0.5 rounded text-xs font-mono transition-colors cursor-pointer {selectedSeverity === 'low' ? 'bg-blue-950 text-blue-200 border border-blue-700 font-bold' : 'bg-blue-950/20 text-blue-400 hover:bg-blue-950/40 border border-blue-900/30'}"
              >
                Low ({report.low_count})
              </button>
              <button
                type="button"
                onclick={() => (selectedSeverity = "info")}
                class="px-2 py-0.5 rounded text-xs font-mono transition-colors cursor-pointer {selectedSeverity === 'info' ? 'bg-neutral-700 text-white font-bold' : 'bg-[#1a1b23] text-neutral-400 hover:text-white border border-white/[0.06]'}"
              >
                Info ({report.info_count})
              </button>
            </div>
          </div>

          <!-- Finding Cards Search & Filters -->
          <div class="p-3 bg-[#14151b] border border-white/[0.08] rounded-xl flex flex-col sm:flex-row items-center justify-between gap-2.5">
            <div class="relative w-full sm:flex-1">
              <Search class="w-3.5 h-3.5 text-neutral-500 absolute inset-y-0 left-3 my-auto pointer-events-none" />
              <input
                type="text"
                bind:value={searchQuery}
                placeholder="Search findings, CVEs, OWASP categories..."
                class="w-full pl-8 pr-3 py-1.5 bg-[#1b1c24] border border-white/[0.08] focus:border-cyan-500/80 rounded-md text-xs text-neutral-200 placeholder-neutral-500 font-mono focus:outline-none"
              />
            </div>

            <div class="flex items-center gap-2 w-full sm:w-auto">
              <select
                bind:value={selectedCategory}
                class="w-full sm:w-auto px-2.5 py-1.5 bg-[#1b1c24] border border-white/[0.08] focus:border-cyan-500/80 rounded-md text-xs text-neutral-300 font-mono focus:outline-none cursor-pointer"
              >
                {#each categories as cat}
                  <option value={cat.id}>{cat.label}</option>
                {/each}
              </select>

              <div class="flex items-center gap-1">
                <ArrowUpDown class="w-3.5 h-3.5 text-neutral-400 flex-shrink-0" />
                <select
                  bind:value={sortFindingsBy}
                  class="px-2 py-1.5 bg-[#1b1c24] border border-white/[0.08] focus:border-cyan-500/80 rounded-md text-xs text-neutral-300 font-mono focus:outline-none cursor-pointer"
                >
                  <option value="severity">Severity</option>
                  <option value="title">Title (A-Z)</option>
                  <option value="category">Category</option>
                </select>
              </div>
            </div>
          </div>

          <!-- Findings List -->
          <div class="space-y-2.5">
            {#if filteredFindings.length === 0}
              <div class="py-16 text-center bg-[#14151b] border border-white/[0.08] rounded-xl space-y-2">
                <CheckCircle2 class="w-10 h-10 text-emerald-400 mx-auto opacity-80" />
                <h3 class="text-sm font-semibold text-neutral-200">No issues found matching current criteria</h3>
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
        <div class="my-auto max-w-xl mx-auto w-full py-8 text-center space-y-5 animate-fade-in">
          <div class="w-14 h-14 rounded-2xl bg-gradient-to-br from-neutral-800 to-neutral-900 border border-white/[0.12] flex items-center justify-center text-cyan-400 mx-auto shadow-2xl">
            <ShieldCheck class="w-7 h-7" />
          </div>

          <div class="space-y-1.5">
            <h1 class="text-2xl font-bold text-white tracking-tight font-mono">
              VulnRadar Security Workstation
            </h1>
            <p class="text-xs text-neutral-400 max-w-md mx-auto leading-relaxed">
              Multi-threaded passive reconnaissance, automated TCP port discovery, email/DNS anti-spoofing verification, and continuous posture monitoring.
            </p>
          </div>

          <!-- Central Instant Input -->
          <form
            onsubmit={(e) => {
              e.preventDefault();
              handleScan();
            }}
            class="flex items-center gap-1.5 p-1.5 bg-[#14151b] border border-white/[0.1] focus-within:border-cyan-500/80 rounded-xl shadow-xl transition-all max-w-md mx-auto"
          >
            <div class="pl-3 text-neutral-500">
              <Search class="w-4 h-4" />
            </div>
            <input
              type="text"
              bind:value={targetUrl}
              placeholder="Enter domain or backend URL (e.g. example.com or http://localhost:8000)..."
              class="w-full py-2 bg-transparent text-xs font-mono text-white placeholder-neutral-500 focus:outline-none"
            />
            <button
              type="submit"
              disabled={!targetUrl.trim() || isScanning}
              class="px-4 py-2 bg-cyan-500 hover:bg-cyan-400 disabled:opacity-40 text-slate-950 font-bold text-xs rounded-lg transition-colors cursor-pointer flex-shrink-0 shadow-sm"
            >
              Audit Target
            </button>
          </form>

          <!-- Quick Presets -->
          <div class="flex flex-wrap items-center justify-center gap-1.5 pt-1">
            <span class="text-[11px] text-neutral-500 font-mono mr-1">Quick Target:</span>
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
                class="px-2.5 py-1 bg-[#14151b] hover:bg-[#1b1c24] border border-white/[0.08] hover:border-cyan-500/40 rounded-md text-xs font-mono text-neutral-400 hover:text-white transition-colors cursor-pointer"
              >
                {preset}
              </button>
            {/each}
          </div>

          <!-- Native Engine Status Telemetry Widget -->
          <div class="grid grid-cols-3 gap-2.5 max-w-md mx-auto pt-4 text-left">
            <div class="p-3 bg-[#13141a] border border-white/[0.06] rounded-lg space-y-1">
              <div class="text-[10px] font-mono uppercase text-neutral-400">Scanner Engine</div>
              <div class="text-xs font-bold font-mono text-cyan-300">Rust Core v0.7.0</div>
            </div>
            <div class="p-3 bg-[#13141a] border border-white/[0.06] rounded-lg space-y-1">
              <div class="text-[10px] font-mono uppercase text-neutral-400">Port Probing</div>
              <div class="text-xs font-bold font-mono text-emerald-400">Top 20 (Auto)</div>
            </div>
            <div class="p-3 bg-[#13141a] border border-white/[0.06] rounded-lg space-y-1">
              <div class="text-[10px] font-mono uppercase text-neutral-400">Local Storage</div>
              <div class="text-xs font-bold font-mono text-neutral-200">SQLite WAL</div>
            </div>
          </div>
        </div>
      {/if}
    </main>
  </div>

  <!-- Native Desktop Footbar / Bottom Status Bar -->
  <footer
    class="h-6 bg-[#0e0f13] border-t border-white/[0.08] px-4 flex items-center justify-between text-[10px] font-mono text-neutral-400 desktop-select-none flex-shrink-0 z-30 print:hidden"
  >
    <div class="flex items-center gap-3">
      <span class="flex items-center gap-1.5 text-cyan-300">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
        VulnRadar v0.7.0 (Native Core)
      </span>
      <span class="text-neutral-600">|</span>
      <span>SQLite: WAL Mode</span>
      <span class="text-neutral-600">|</span>
      <span>Default Ports: Top 20 (Auto)</span>
    </div>

    <div class="flex items-center gap-3">
      {#if report}
        <span class="text-neutral-300">Response: {report.response_time_ms} ms</span>
        <span class="text-neutral-600">|</span>
      {/if}
      <span>Monitors: {monitors.filter((m) => m.is_active).length} active</span>
      <span class="text-neutral-600">|</span>
      <span class="text-neutral-400">Shortcuts: Press <kbd class="px-1 text-[9px] bg-[#1a1b22] border border-white/[0.1] rounded text-neutral-300">?</kbd></span>
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
