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
  } from "lucide-svelte";

  let targetUrl = $state("");
  let isScanning = $state(false);
  let scanError = $state<string | null>(null);
  let report = $state<ScanReport | null>(null);
  let history = $state<ScanSummary[]>([]);
  let monitors = $state<MonitorTarget[]>([]);

  // Scan Configuration
  let scanOptions = $state<ScanOptions>({
    timeout_seconds: 15,
    include_subdomains: true,
  });

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

  const hasCustomOptions = $derived(
    !!(
      (scanOptions.custom_headers && scanOptions.custom_headers.length > 0) ||
      scanOptions.user_agent ||
      (scanOptions.timeout_seconds && scanOptions.timeout_seconds !== 15) ||
      scanOptions.include_subdomains === false ||
      scanOptions.enable_port_scan === true
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
      isSettingsOpen = !isSettingsOpen;
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
      settingsTab = "batch";
      isSettingsOpen = true;
      return;
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "h") {
      e.preventDefault();
      isHistoryOpen = !isHistoryOpen;
      return;
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "m") {
      e.preventDefault();
      settingsTab = "watchdog";
      isSettingsOpen = true;
      return;
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "o") {
      e.preventDefault();
      settingsTab = "params";
      isSettingsOpen = true;
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
      settingsTab = "shortcuts";
      isSettingsOpen = true;
    }
  }

  onMount(() => {
    loadHistory();
    loadMonitors();

    window.addEventListener("keydown", handleKeydown);

    // Listen for background watchdog alerts
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
    report = null;

    try {
      const res = await invokeTauri<ScanReport>("scan_target", {
        url,
        options: scanOptions,
      });
      report = res;
      await loadHistory();
      showToast(`Scan complete: Score ${res.security_score}/100 with ${res.total_findings} findings`, "success");
    } catch (err: any) {
      scanError =
        err?.toString() ||
        "Failed to scan target. Please check the URL and internet connection.";
      showToast("Security audit failed. Check target URL and network.", "error");
    } finally {
      isScanning = false;
    }
  }

  async function handleSelectFromHistory(scanId: string) {
    isHistoryOpen = false;
    isScanning = true;
    scanError = null;

    try {
      const res = await invokeTauri<ScanReport | null>("get_scan_report", {
        id: scanId,
      });
      if (res) {
        report = res;
        targetUrl = res.target_url;
        showToast(`Loaded historical audit from ${new Date(res.scanned_at).toLocaleDateString()}`, "info");
      }
    } catch (err: any) {
      scanError = err?.toString() || "Failed to load past scan report.";
    } finally {
      isScanning = false;
    }
  }

  async function handleDeleteFromHistory(scanId: string) {
    try {
      await invokeTauri("delete_scan", { id: scanId });
      await loadHistory();
      showToast("Scan removed from history archive", "info");
    } catch (err) {
      console.error(err);
    }
  }

  async function handleClearAllHistory() {
    try {
      await invokeTauri("clear_history");
      history = [];
      showToast("History archive cleared", "info");
    } catch (err) {
      console.error(err);
    }
  }

  async function handleAddMonitor(url: string, intervalHours: number) {
    try {
      await invokeTauri("add_monitor", { url, intervalHours });
      await loadMonitors();
      showToast(`Added ${url} to continuous watchdog (${intervalHours}h)`, "success");
    } catch (err) {
      console.error("Failed to add monitor:", err);
    }
  }

  async function handleDeleteMonitor(id: string) {
    try {
      await invokeTauri("delete_monitor", { id });
      await loadMonitors();
      showToast("Monitor removed", "info");
    } catch (err) {
      console.error("Failed to delete monitor:", err);
    }
  }

  async function handleToggleMonitor(id: string) {
    try {
      await invokeTauri("toggle_monitor", { id });
      await loadMonitors();
    } catch (err) {
      console.error("Failed to toggle monitor:", err);
    }
  }

  async function openExportModal() {
    if (!report) return;
    try {
      exportMarkdown = await invokeTauri<string>("export_report_markdown", {
        report,
      });
    } catch {
      exportMarkdown = `# Security Report: ${report.target_url}\nScore: ${report.security_score}/100`;
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
  ];

  // Active Tab in Report View
  let activeTab = $state<"findings" | "ports" | "recon" | "dns" | "endpoints">("findings");
  let subdomainSearch = $state("");
  let copiedUrl = $state(false);

  // Open Ports Filter State
  let portSearchQuery = $state("");
  let portRiskFilter = $state<"all" | "risky" | "standard">("all");
  let copiedPort = $state<number | null>(null);

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

<!-- Top Navigation & Scan Bar -->
<Navbar
  bind:targetUrl
  {isScanning}
  hasReport={!!report}
  {hasCustomOptions}
  activeMonitorsCount={monitors.filter((m) => m.is_active).length}
  onScan={() => handleScan()}
  onOpenHistory={() => (isHistoryOpen = true)}
  onOpenSettings={(tab) => {
    if (tab) settingsTab = tab;
    isSettingsOpen = true;
  }}
  onOpenExport={openExportModal}
/>

<!-- Watchdog Alert Banner -->
{#if watchdogAlert}
  <div
    class="bg-rose-950/80 backdrop-blur-md border-b border-rose-800/80 px-6 py-3 text-rose-200 text-xs flex items-center justify-between gap-4 animate-fade-in print:hidden"
  >
    <div class="flex items-center gap-2.5 min-w-0">
      <Bell class="w-4 h-4 text-rose-400 animate-bounce flex-shrink-0" />
      <span class="font-bold uppercase tracking-wider font-mono text-[11px]">Watchdog Trigger:</span>
      <span class="truncate font-mono font-bold text-white">{watchdogAlert.target_url}</span>
      <span class="text-rose-300 hidden sm:inline">
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
        class="px-3 py-1 bg-rose-500 hover:bg-rose-400 text-slate-950 font-bold rounded-lg text-xs cursor-pointer transition-all shadow-md shadow-rose-500/20"
      >
        View Audit
      </button>
      <button
        type="button"
        onclick={() => (watchdogAlert = null)}
        class="p-1 text-rose-400 hover:text-rose-200 rounded cursor-pointer"
        aria-label="Dismiss alert"
      >
        <X class="w-4 h-4" />
      </button>
    </div>
  </div>
{/if}

<!-- Main Content Area -->
<main class="max-w-7xl w-full mx-auto px-6 py-8 flex-1 flex flex-col">
  <!-- Scanning State -->
  {#if isScanning}
    <div class="my-auto py-24 flex flex-col items-center justify-center text-center">
      <div class="w-12 h-12 rounded-xl bg-[#252525] border border-[#333333] flex items-center justify-center text-white mb-6">
        <Loader2 class="w-6 h-6 animate-spin text-neutral-300" />
      </div>

      <div class="inline-flex items-center gap-2 px-2.5 py-1 rounded-md bg-[#252525] border border-[#333] text-neutral-300 text-xs font-mono mb-3">
        <span class="w-2 h-2 rounded-full bg-blue-400"></span>
        <span>Audit in progress</span>
      </div>

      <h2 class="text-xl font-semibold text-white tracking-tight">Auditing Target Surface</h2>
      <p class="text-xs font-mono text-neutral-300 mt-2 max-w-md truncate bg-[#202020] px-3 py-1.5 rounded-lg border border-[#2e2e2e]">
        {targetUrl}
      </p>

      <p class="text-xs text-neutral-400 mt-3 max-w-md leading-relaxed">
        Evaluating HTTP security headers, TLS ciphers, cookie flags, Certificate Transparency subdomains, DNS SPF/DMARC anti-spoofing, and front-end CVE dependencies...
      </p>
    </div>

  <!-- Scan Error State -->
  {:else if scanError}
    <div class="my-auto max-w-xl mx-auto p-5 bg-red-950/20 border border-red-900/40 rounded-xl flex items-start gap-4 text-red-300">
      <AlertOctagon class="w-6 h-6 flex-shrink-0 text-red-400 mt-0.5" />
      <div class="space-y-2">
        <h3 class="text-sm font-semibold text-red-200">Security Audit Encountered an Error</h3>
        <p class="text-xs text-red-300/90 font-mono bg-[#161616] p-2.5 rounded-lg border border-red-900/30">{scanError}</p>
        <button
          type="button"
          onclick={() => handleScan()}
          class="mt-1 px-3 py-1.5 bg-white hover:bg-neutral-200 text-neutral-950 rounded-lg text-xs font-semibold transition-colors cursor-pointer shadow-sm"
        >
          Retry Scan
        </button>
      </div>
    </div>

  <!-- Active Report View -->
  {:else if report}
    <div class="space-y-6 animate-fade-in">
      <!-- Target Summary Header Card -->
      <div class="p-5 bg-[#202020] border border-[#2e2e2e] rounded-xl flex flex-col lg:flex-row items-start lg:items-center justify-between gap-5">
        <div class="space-y-2 min-w-0 flex-1">
          <div class="flex flex-wrap items-center gap-2">
            <span class="px-2 py-0.5 text-xs font-mono font-medium bg-emerald-950/40 text-emerald-300 border border-emerald-800/50 rounded">
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
              <div class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[11px] font-mono {report.security_score >= previousScan.security_score ? 'bg-emerald-950/30 text-emerald-300 border border-emerald-800/30' : 'bg-rose-950/30 text-rose-300 border border-rose-800/30'}">
                {#if report.security_score > previousScan.security_score}
                  <TrendingUp class="w-3 h-3 text-emerald-400" />
                  <span>+{report.security_score - previousScan.security_score} pts vs previous</span>
                {:else if report.security_score < previousScan.security_score}
                  <TrendingDown class="w-3 h-3 text-rose-400" />
                  <span>-{previousScan.security_score - report.security_score} pts vs previous</span>
                {:else}
                  <span>Score unchanged</span>
                {/if}
              </div>
            {/if}
          </div>

          <div class="flex items-center gap-2.5">
            <h1 class="text-xl font-bold text-white tracking-tight truncate flex items-center gap-2">
              <Globe class="w-5 h-5 text-neutral-400 flex-shrink-0" />
              <span class="truncate">{report.target_url}</span>
            </h1>
            <div class="flex items-center gap-1">
              <button
                type="button"
                onclick={copyTargetUrl}
                class="p-1.5 text-neutral-400 hover:text-white hover:bg-[#282828] rounded cursor-pointer transition-colors"
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
                class="p-1.5 text-neutral-400 hover:text-white hover:bg-[#282828] rounded cursor-pointer transition-colors"
                title="Copy cURL reproduction command"
              >
                {#if copiedCurl}
                  <Check class="w-3.5 h-3.5 text-emerald-400" />
                {:else}
                  <Terminal class="w-3.5 h-3.5" />
                {/if}
              </button>
            </div>
          </div>

          <!-- Detected Tech Stack Tags -->
          {#if report.technologies_detected.length > 0}
            <div class="flex flex-wrap items-center gap-1.5 pt-0.5">
              <span class="text-xs text-neutral-400 font-medium flex items-center gap-1 mr-1">
                <Cpu class="w-3.5 h-3.5 text-neutral-400" /> Fingerprint:
              </span>
              {#each report.technologies_detected as tech}
                <span class="px-2 py-0.5 text-xs font-mono bg-[#191919] text-neutral-300 border border-[#2e2e2e] rounded">
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

      <!-- Compact Interactive Severity Distribution & Telemetry Bar -->
      <div class="p-3.5 bg-[#202020] border border-[#2e2e2e] rounded-xl space-y-2.5">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="text-[11px] font-semibold uppercase tracking-wider text-neutral-400">
              Severity Distribution
            </span>
            <span class="px-1.5 py-0.2 text-[10px] font-mono bg-[#191919] text-neutral-300 border border-[#2e2e2e] rounded">
              {report.total_findings} Total
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
              class="text-[11px] text-neutral-400 hover:text-white underline cursor-pointer"
            >
              Reset Filters
            </button>
          {/if}
        </div>

        <!-- Proportional Colored Bar -->
        {#if report.total_findings > 0}
          <div class="w-full h-2 bg-[#161616] rounded-full overflow-hidden flex gap-0.5 p-0.5 border border-[#2a2a2a]">
            {#if report.critical_count > 0}
              <div
                class="bg-rose-500 h-full rounded-full transition-all duration-300"
                style="width: {(report.critical_count / report.total_findings) * 100}%"
                title="Critical: {report.critical_count}"
              ></div>
            {/if}
            {#if report.high_count > 0}
              <div
                class="bg-orange-500 h-full rounded-full transition-all duration-300"
                style="width: {(report.high_count / report.total_findings) * 100}%"
                title="High: {report.high_count}"
              ></div>
            {/if}
            {#if report.medium_count > 0}
              <div
                class="bg-amber-500 h-full rounded-full transition-all duration-300"
                style="width: {(report.medium_count / report.total_findings) * 100}%"
                title="Medium: {report.medium_count}"
              ></div>
            {/if}
            {#if report.low_count > 0}
              <div
                class="bg-blue-500 h-full rounded-full transition-all duration-300"
                style="width: {(report.low_count / report.total_findings) * 100}%"
                title="Low: {report.low_count}"
              ></div>
            {/if}
            {#if report.info_count > 0}
              <div
                class="bg-neutral-500 h-full rounded-full transition-all duration-300"
                style="width: {(report.info_count / report.total_findings) * 100}%"
                title="Info: {report.info_count}"
              ></div>
            {/if}
          </div>
        {/if}

        <!-- Interactive Filter Chips -->
        <div class="flex flex-wrap items-center gap-1.5 pt-0.5">
          <button
            type="button"
            onclick={() => (selectedSeverity = "all")}
            class="px-2.5 py-1 rounded-lg text-xs font-mono transition-colors cursor-pointer {selectedSeverity === 'all' ? 'bg-white text-neutral-950 font-bold shadow-xs' : 'bg-[#191919] text-neutral-400 hover:text-white border border-[#2e2e2e]'}"
          >
            All ({report.total_findings})
          </button>
          <button
            type="button"
            onclick={() => (selectedSeverity = "critical")}
            class="px-2.5 py-1 rounded-lg text-xs font-mono transition-colors cursor-pointer {selectedSeverity === 'critical' ? 'bg-rose-950 text-rose-200 border border-rose-700 font-bold' : 'bg-rose-950/20 text-rose-400 hover:bg-rose-950/40 border border-rose-900/30'}"
          >
            Critical ({report.critical_count})
          </button>
          <button
            type="button"
            onclick={() => (selectedSeverity = "high")}
            class="px-2.5 py-1 rounded-lg text-xs font-mono transition-colors cursor-pointer {selectedSeverity === 'high' ? 'bg-orange-950 text-orange-200 border border-orange-700 font-bold' : 'bg-orange-950/20 text-orange-400 hover:bg-orange-950/40 border border-orange-900/30'}"
          >
            High ({report.high_count})
          </button>
          <button
            type="button"
            onclick={() => (selectedSeverity = "medium")}
            class="px-2.5 py-1 rounded-lg text-xs font-mono transition-colors cursor-pointer {selectedSeverity === 'medium' ? 'bg-amber-950 text-amber-200 border border-amber-700 font-bold' : 'bg-amber-950/20 text-amber-400 hover:bg-amber-950/40 border border-amber-900/30'}"
          >
            Med ({report.medium_count})
          </button>
          <button
            type="button"
            onclick={() => (selectedSeverity = "low")}
            class="px-2.5 py-1 rounded-lg text-xs font-mono transition-colors cursor-pointer {selectedSeverity === 'low' ? 'bg-blue-950 text-blue-200 border border-blue-700 font-bold' : 'bg-blue-950/20 text-blue-400 hover:bg-blue-950/40 border border-blue-900/30'}"
          >
            Low ({report.low_count})
          </button>
          <button
            type="button"
            onclick={() => (selectedSeverity = "info")}
            class="px-2.5 py-1 rounded-lg text-xs font-mono transition-colors cursor-pointer {selectedSeverity === 'info' ? 'bg-neutral-700 text-white font-bold' : 'bg-[#191919] text-neutral-400 hover:text-white border border-[#2e2e2e]'}"
          >
            Info ({report.info_count})
          </button>
        </div>
      </div>

      <!-- Navigation Tabs (Findings, Ports, DNS/Email, Endpoints, Subdomain Map) -->
      <div class="border-b border-[#2e2e2e] flex items-center gap-1 overflow-x-auto pb-px">
        <button
          type="button"
          onclick={() => (activeTab = "findings")}
          class="px-3.5 py-2 text-xs font-medium flex items-center gap-2 border-b-2 transition-colors cursor-pointer {activeTab === 'findings' ? 'border-white text-white font-semibold' : 'border-transparent text-neutral-400 hover:text-neutral-200'}"
        >
          <ShieldCheck class="w-3.5 h-3.5" />
          <span>Findings ({report.findings.length})</span>
        </button>

        <button
          type="button"
          onclick={() => (activeTab = "ports")}
          class="px-3.5 py-2 text-xs font-medium flex items-center gap-2 border-b-2 transition-colors cursor-pointer {activeTab === 'ports' ? 'border-white text-white font-semibold' : 'border-transparent text-neutral-400 hover:text-neutral-200'}"
        >
          <Server class="w-3.5 h-3.5" />
          <span>Open Ports ({report.port_report?.open_ports_count || 0})</span>
        </button>

        <button
          type="button"
          onclick={() => (activeTab = "dns")}
          class="px-3.5 py-2 text-xs font-medium flex items-center gap-2 border-b-2 transition-colors cursor-pointer {activeTab === 'dns' ? 'border-white text-white font-semibold' : 'border-transparent text-neutral-400 hover:text-neutral-200'}"
        >
          <Mail class="w-3.5 h-3.5" />
          <span>DNS & Email</span>
        </button>

        <button
          type="button"
          onclick={() => (activeTab = "endpoints")}
          class="px-3.5 py-2 text-xs font-medium flex items-center gap-2 border-b-2 transition-colors cursor-pointer {activeTab === 'endpoints' ? 'border-white text-white font-semibold' : 'border-transparent text-neutral-400 hover:text-neutral-200'}"
        >
          <FileCode class="w-3.5 h-3.5" />
          <span>Endpoints</span>
        </button>

        <button
          type="button"
          onclick={() => (activeTab = "recon")}
          class="px-3.5 py-2 text-xs font-medium flex items-center gap-2 border-b-2 transition-colors cursor-pointer {activeTab === 'recon' ? 'border-white text-white font-semibold' : 'border-transparent text-neutral-400 hover:text-neutral-200'}"
        >
          <Globe class="w-3.5 h-3.5" />
          <span>Subdomains ({report.subdomains?.length || 0})</span>
        </button>
      </div>

      <!-- TAB 1: Findings View -->
      {#if activeTab === "findings"}
        <!-- Search & Filter Controls -->
        <div class="p-3 bg-[#202020] border border-[#2e2e2e] rounded-xl flex flex-col sm:flex-row items-center justify-between gap-2.5">
          <div class="relative w-full sm:flex-1">
            <Search class="w-3.5 h-3.5 text-neutral-500 absolute inset-y-0 left-3 my-auto pointer-events-none" />
            <input
              type="text"
              bind:value={searchQuery}
              placeholder="Search findings, CVEs, OWASP categories..."
              class="w-full pl-8 pr-3 py-1.5 bg-[#191919] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs text-neutral-200 placeholder-neutral-500 font-mono focus:outline-none"
            />
          </div>

          <div class="flex items-center gap-2 w-full sm:w-auto">
            <!-- Category Filter Dropdown -->
            <select
              bind:value={selectedCategory}
              class="w-full sm:w-auto px-2.5 py-1.5 bg-[#191919] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs text-neutral-300 font-mono focus:outline-none cursor-pointer"
            >
              {#each categories as cat}
                <option value={cat.id}>{cat.label}</option>
              {/each}
            </select>

            <!-- Sort By Dropdown -->
            <div class="flex items-center gap-1">
              <ArrowUpDown class="w-3.5 h-3.5 text-neutral-400 flex-shrink-0" />
              <select
                bind:value={sortFindingsBy}
                class="px-2 py-1.5 bg-[#191919] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs text-neutral-300 font-mono focus:outline-none cursor-pointer"
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
          <div class="flex items-center justify-between px-1">
            <h2 class="text-xs font-medium uppercase tracking-wider text-neutral-400">
              Security Findings ({filteredFindings.length})
            </h2>
            {#if selectedSeverity !== "all" || searchQuery.trim()}
              <button
                type="button"
                onclick={() => {
                  selectedSeverity = "all";
                  searchQuery = "";
                }}
                class="text-xs text-neutral-300 hover:text-white underline cursor-pointer"
              >
                Reset Filters
              </button>
            {/if}
          </div>

          {#if filteredFindings.length === 0}
            <div class="py-16 text-center bg-[#202020] border border-[#2e2e2e] rounded-xl">
              <CheckCircle2 class="w-10 h-10 text-emerald-400 mx-auto mb-2 opacity-80" />
              <h3 class="text-sm font-semibold text-neutral-200">No matching issues found</h3>
              <p class="text-xs text-neutral-400 mt-0.5">No vulnerabilities match the current filter selection.</p>
            </div>
          {:else}
            {#each filteredFindings as finding (finding.id)}
              <FindingCard {finding} />
            {/each}
          {/if}
        </div>

      <!-- TAB 2: Open Ports & Network Surface (Nmap-Style) -->
      {:else if activeTab === "ports"}
        <div class="space-y-5">
          {#if report.port_report}
            <!-- Port Telemetry Bar -->
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-2.5">
              <div class="p-3 bg-[#202020] border border-[#2e2e2e] rounded-lg space-y-0.5">
                <span class="text-[11px] font-medium uppercase tracking-wider text-neutral-400">Target IP</span>
                <div class="text-sm font-bold font-mono text-neutral-200 truncate">
                  {report.port_report.ip_address || "Resolved Domain"}
                </div>
              </div>
              <div class="p-3 bg-[#202020] border border-[#2e2e2e] rounded-lg space-y-0.5">
                <span class="text-[11px] font-medium uppercase tracking-wider text-neutral-400">Scanned Ports</span>
                <div class="text-sm font-bold font-mono text-neutral-200">
                  {report.port_report.scanned_ports_count} Ports
                </div>
              </div>
              <div class="p-3 bg-[#202020] border border-[#2e2e2e] rounded-lg space-y-0.5">
                <span class="text-[11px] font-medium uppercase tracking-wider text-neutral-400">Open Ports</span>
                <div class="text-sm font-bold font-mono {report.port_report.open_ports_count > 0 ? 'text-emerald-400' : 'text-neutral-400'}">
                  {report.port_report.open_ports_count} Discovered
                </div>
              </div>
              <div class="p-3 bg-[#202020] border border-[#2e2e2e] rounded-lg space-y-0.5">
                <span class="text-[11px] font-medium uppercase tracking-wider text-neutral-400">Scan Duration</span>
                <div class="text-sm font-bold font-mono text-neutral-300">
                  {report.port_report.scan_duration_ms} ms
                </div>
              </div>
            </div>

            <!-- Search and Filter Controls -->
            <div class="p-3 bg-[#202020] border border-[#2e2e2e] rounded-xl flex flex-col md:flex-row items-center justify-between gap-3">
              <div class="relative w-full md:w-80">
                <Search class="w-3.5 h-3.5 text-neutral-500 absolute inset-y-0 left-3 my-auto pointer-events-none" />
                <input
                  type="text"
                  bind:value={portSearchQuery}
                  placeholder="Filter by port #, service, banner..."
                  class="w-full pl-8 pr-3 py-1.5 bg-[#191919] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs text-neutral-200 placeholder-neutral-500 font-mono focus:outline-none"
                />
              </div>

              <div class="flex flex-wrap items-center gap-1.5 w-full md:w-auto">
                <button
                  type="button"
                  onclick={() => (portRiskFilter = "all")}
                  class="px-2.5 py-1 rounded-md text-xs font-medium cursor-pointer transition-colors {portRiskFilter === 'all' ? 'bg-white text-neutral-950 font-semibold' : 'bg-[#191919] text-neutral-400 hover:text-neutral-200'}"
                >
                  All ({report.port_report.open_ports.length})
                </button>
                <button
                  type="button"
                  onclick={() => (portRiskFilter = "risky")}
                  class="px-2.5 py-1 rounded-md text-xs font-medium cursor-pointer transition-colors {portRiskFilter === 'risky' ? 'bg-red-950 text-red-200 border border-red-800' : 'bg-red-950/20 text-red-400 hover:bg-red-950/40'}"
                >
                  Risky ({report.port_report.open_ports.filter((p) => p.is_risky).length})
                </button>
                <button
                  type="button"
                  onclick={() => (portRiskFilter = "standard")}
                  class="px-2.5 py-1 rounded-md text-xs font-medium cursor-pointer transition-colors {portRiskFilter === 'standard' ? 'bg-[#2a2a2a] text-white border border-neutral-600' : 'bg-[#191919] text-neutral-400 hover:text-neutral-200'}"
                >
                  Standard ({report.port_report.open_ports.filter((p) => !p.is_risky).length})
                </button>
              </div>
            </div>

            <!-- Open Ports Grid -->
            {#if filteredOpenPorts.length === 0}
              <div class="py-16 text-center bg-[#202020] border border-[#2e2e2e] rounded-xl space-y-2">
                <CheckCircle2 class="w-10 h-10 text-emerald-400 mx-auto opacity-80" />
                <h3 class="text-sm font-semibold text-neutral-200">No Open Ports Matching Criteria</h3>
                <p class="text-xs text-neutral-400 max-w-sm mx-auto">
                  {report.port_report.open_ports.length === 0
                    ? `Scanned ${report.port_report.scanned_ports_count} ports on ${report.port_report.host} — no listening services were found.`
                    : "No ports matched your current search or risk filter."}
                </p>
              </div>
            {:else}
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                {#each filteredOpenPorts as p (p.port)}
                  <div
                    class="p-4 bg-[#202020] border rounded-xl space-y-3 transition-colors {p.is_risky ? 'border-red-800/40 bg-red-950/10' : 'border-[#2e2e2e] hover:border-[#383838]'}"
                  >
                    <!-- Port Card Header -->
                    <div class="flex items-center justify-between gap-2">
                      <div class="flex items-center gap-2">
                        <span class="px-2 py-0.5 text-xs font-mono font-medium rounded bg-[#191919] border border-[#2e2e2e] text-neutral-200">
                          PORT {p.port}/{p.protocol.toUpperCase()}
                        </span>
                        <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[10px] font-mono bg-emerald-950/40 text-emerald-300 border border-emerald-800/50">
                          <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                          OPEN
                        </span>
                      </div>

                      {#if p.is_risky}
                        <span class="px-2 py-0.5 text-[10px] font-mono rounded bg-red-950/40 text-red-300 border border-red-800/50 flex items-center gap-1">
                          <AlertTriangle class="w-3 h-3 text-red-400" />
                          RISKY SERVICE
                        </span>
                      {/if}
                    </div>

                    <!-- Service Description -->
                    <div class="space-y-0.5">
                      <div class="text-sm font-semibold text-white flex items-center gap-1.5">
                        <Server class="w-3.5 h-3.5 text-neutral-400" />
                        <span>{p.service}</span>
                      </div>
                      <p class="text-xs text-neutral-400">{p.description}</p>
                    </div>

                    <!-- Banner Details if available -->
                    {#if p.banner}
                      <div class="p-2.5 bg-[#161616] rounded-lg border border-[#2a2a2a] space-y-1">
                        <span class="text-[10px] font-mono text-neutral-400 uppercase tracking-wider block">
                          Service Banner
                        </span>
                        <pre class="text-xs font-mono text-neutral-200 overflow-x-auto whitespace-pre-wrap">{p.banner}</pre>
                      </div>
                    {/if}

                    <!-- Port Actions -->
                    <div class="pt-2 border-t border-[#2a2a2a] flex items-center justify-between">
                      <div class="text-[11px] font-mono text-neutral-500">
                        {report.port_report.host}:{p.port}
                      </div>

                      <div class="flex items-center gap-1.5">
                        <button
                          type="button"
                          onclick={() => copyPortAddress(report!.port_report!.host, p.port)}
                          class="px-2 py-0.5 bg-[#262626] hover:bg-[#303030] text-neutral-300 hover:text-white rounded text-xs font-mono transition-colors cursor-pointer flex items-center gap-1"
                          title="Copy host and port"
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
                            href={`${p.port === 443 || p.port === 8443 ? "https" : "http"}://${report.port_report.host}:${p.port}`}
                            target="_blank"
                            rel="noopener noreferrer"
                            class="p-1 bg-[#262626] hover:bg-[#303030] text-neutral-400 hover:text-white rounded text-xs transition-colors cursor-pointer"
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
          {:else}
            <!-- Port scan not enabled for this scan -->
            <div class="p-8 text-center bg-[#202020] border border-[#2e2e2e] rounded-xl space-y-3 max-w-lg mx-auto">
              <div class="w-10 h-10 rounded-xl bg-[#262626] border border-[#333] flex items-center justify-center text-neutral-300 mx-auto">
                <Server class="w-5 h-5" />
              </div>
              <div class="space-y-1">
                <h3 class="text-sm font-semibold text-white">Port Scanning Was Not Enabled</h3>
                <p class="text-xs text-neutral-400 leading-relaxed">
                  Port scanning is an active probe feature and is configured per audit. You can enable open port discovery in Scan Parameters.
                </p>
              </div>
              <button
                type="button"
                onclick={() => {
                  scanOptions.enable_port_scan = true;
                  settingsTab = "ports";
                  isSettingsOpen = true;
                }}
                class="px-3.5 py-1.5 bg-white hover:bg-neutral-200 text-neutral-950 font-semibold rounded-lg text-xs transition-colors cursor-pointer inline-flex items-center gap-2 shadow-sm"
              >
                <Sliders class="w-3.5 h-3.5" />
                <span>Configure & Enable Port Scanner</span>
              </button>
            </div>
          {/if}
        </div>

      <!-- TAB 3: DNS & Email Hardening Posture -->
      {:else if activeTab === "dns"}
        <div class="space-y-4">
          {#if report.dns_security}
            <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
              <div class="p-4 bg-[#202020] border border-[#2e2e2e] rounded-xl space-y-2">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-medium text-neutral-300">SPF Anti-Spoofing</span>
                  <span class="px-2 py-0.5 text-xs font-mono rounded {report.dns_security.spf_record ? 'bg-emerald-950/40 text-emerald-300 border border-emerald-800/50' : 'bg-red-950/40 text-red-300 border border-red-800/50'}">
                    {report.dns_security.spf_record ? "Configured" : "Missing"}
                  </span>
                </div>
                <p class="text-xs text-neutral-400">Validates authorized mail senders via Sender Policy Framework.</p>
                {#if report.dns_security.spf_record}
                  <pre class="bg-[#161616] p-2.5 rounded-lg text-xs font-mono text-neutral-200 border border-[#2a2a2a] overflow-x-auto whitespace-pre-wrap">{report.dns_security.spf_record}</pre>
                {/if}
              </div>

              <div class="p-4 bg-[#202020] border border-[#2e2e2e] rounded-xl space-y-2">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-medium text-neutral-300">DMARC Policy</span>
                  <span class="px-2 py-0.5 text-xs font-mono rounded {report.dns_security.dmarc_policy && report.dns_security.dmarc_policy !== 'none' ? 'bg-emerald-950/40 text-emerald-300 border border-emerald-800/50' : 'bg-amber-950/40 text-amber-300 border border-amber-800/50'}">
                    {report.dns_security.dmarc_policy || "None"}
                  </span>
                </div>
                <p class="text-xs text-neutral-400">Enforces domain-based message authentication and alignment.</p>
                {#if report.dns_security.dmarc_record}
                  <pre class="bg-[#161616] p-2.5 rounded-lg text-xs font-mono text-neutral-200 border border-[#2a2a2a] overflow-x-auto whitespace-pre-wrap">{report.dns_security.dmarc_record}</pre>
                {/if}
              </div>

              <div class="p-4 bg-[#202020] border border-[#2e2e2e] rounded-xl space-y-2">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-medium text-neutral-300">DNSSEC Enforcement</span>
                  <span class="px-2 py-0.5 text-xs font-mono rounded {report.dns_security.dnssec_enabled ? 'bg-emerald-950/40 text-emerald-300 border border-emerald-800/50' : 'bg-[#282828] text-neutral-400'}">
                    {report.dns_security.dnssec_enabled ? "Enabled" : "Disabled"}
                  </span>
                </div>
                <p class="text-xs text-neutral-400">Cryptographically authenticates DNS responses against spoofing.</p>
                <div class="text-xs text-neutral-500 font-mono pt-1">
                  Authenticated Data (AD) Flag: {report.dns_security.dnssec_enabled ? "Verified" : "Not Present"}
                </div>
              </div>
            </div>
          {:else}
            <div class="p-6 text-center bg-[#202020] border border-[#2e2e2e] rounded-xl text-neutral-400 text-xs">
              DNS security was not inspected for this scan.
            </div>
          {/if}
        </div>

      <!-- TAB 4: Endpoint Policies (robots.txt, security.txt) -->
      {:else if activeTab === "endpoints"}
        <div class="space-y-4">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
            <!-- robots.txt -->
            <div class="p-4 bg-[#202020] border border-[#2e2e2e] rounded-xl space-y-2">
              <div class="flex items-center justify-between">
                <span class="text-xs font-medium text-neutral-300">robots.txt Disallow Rules</span>
                <span class="px-2 py-0.5 text-xs font-mono rounded bg-[#191919] text-neutral-300 border border-[#2e2e2e]">
                  {report.endpoint_report?.disallowed_paths.length || 0} paths
                </span>
              </div>
              {#if report.endpoint_report && report.endpoint_report.disallowed_paths.length > 0}
                <div class="max-h-56 overflow-y-auto space-y-1 bg-[#161616] p-3 rounded-lg border border-[#2a2a2a] text-xs font-mono">
                  {#each report.endpoint_report.disallowed_paths as path}
                    <div class="text-neutral-400">Disallow: {path}</div>
                  {/each}
                </div>
              {:else}
                <p class="text-xs text-neutral-500">No robots.txt disallowed rules found.</p>
              {/if}
            </div>

            <!-- Sensitive Exposed Paths -->
            <div class="p-4 bg-[#202020] border border-[#2e2e2e] rounded-xl space-y-2">
              <div class="flex items-center justify-between">
                <span class="text-xs font-medium text-neutral-300">Sensitive Paths Disclosed</span>
                <span class="px-2 py-0.5 text-xs font-mono rounded {(report.endpoint_report?.sensitive_disallowed_paths.length || 0) > 0 ? 'bg-amber-950/40 text-amber-300 border border-amber-800/50' : 'bg-emerald-950/40 text-emerald-300 border border-emerald-800/50'}">
                  {report.endpoint_report?.sensitive_disallowed_paths.length || 0} exposed
                </span>
              </div>
              {#if report.endpoint_report && report.endpoint_report.sensitive_disallowed_paths.length > 0}
                <div class="max-h-56 overflow-y-auto space-y-1 bg-amber-950/20 p-3 rounded-lg border border-amber-900/30 text-xs font-mono">
                  {#each report.endpoint_report.sensitive_disallowed_paths as sp}
                    <div class="text-amber-300">⚠️ {sp}</div>
                  {/each}
                </div>
              {:else}
                <p class="text-xs text-emerald-400">No sensitive administration endpoints disclosed in robots.txt.</p>
              {/if}
            </div>
          </div>
        </div>

      <!-- TAB 5: Subdomain Discovery Map -->
      {:else if activeTab === "recon"}
        <div class="space-y-4">
          <div class="p-3 bg-[#202020] border border-[#2e2e2e] rounded-xl flex items-center justify-between gap-3">
            <div class="relative w-full max-w-sm">
              <Search class="w-3.5 h-3.5 text-neutral-500 absolute inset-y-0 left-3 my-auto pointer-events-none" />
              <input
                type="text"
                bind:value={subdomainSearch}
                placeholder="Filter discovered subdomains..."
                class="w-full pl-8 pr-3 py-1.5 bg-[#191919] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs text-neutral-200 placeholder-neutral-500 font-mono focus:outline-none"
              />
            </div>
            <span class="text-xs font-mono text-neutral-400">
              {filteredSubdomains.length} / {report.subdomains?.length || 0} Total
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
                  class="p-2.5 bg-[#202020] hover:bg-[#262626] border border-[#2e2e2e] hover:border-[#383838] rounded-lg text-left text-xs font-mono text-neutral-300 hover:text-white transition-colors cursor-pointer truncate flex items-center justify-between group"
                >
                  <span class="truncate">{sub}</span>
                  <ExternalLink class="w-3.5 h-3.5 text-neutral-500 group-hover:text-neutral-300 flex-shrink-0 ml-2" />
                </button>
              {/each}
            </div>
          {:else}
            <div class="p-8 text-center bg-[#202020] border border-[#2e2e2e] rounded-xl text-neutral-500 text-xs">
              No subdomains found matching current search.
            </div>
          {/if}
        </div>
      {/if}
    </div>

  <!-- Empty Initial Dashboard Landing -->
  {:else}
    <div class="my-auto py-16 flex flex-col items-center justify-center text-center max-w-xl mx-auto animate-fade-in">
      <div class="w-12 h-12 rounded-2xl bg-[#222222] border border-[#333333] flex items-center justify-center text-white mb-5 shadow-lg">
        <ShieldCheck class="w-6 h-6 text-white" />
      </div>

      <h1 class="text-2xl sm:text-3xl font-bold text-white tracking-tight">
        Web Security Posture Scanner
      </h1>
      <p class="text-xs text-neutral-400 mt-2 max-w-md leading-relaxed">
        Passive reconnaissance, HTTP headers, TLS ciphers, cookie flags, and port discovery.
      </p>

      <!-- Center-Stage Instant Scan Input -->
      <div class="w-full max-w-md mt-6">
        <form
          onsubmit={(e) => {
            e.preventDefault();
            handleScan();
          }}
          class="flex items-center gap-1.5 p-1 bg-[#202020] border border-[#2e2e2e] focus-within:border-neutral-400 rounded-xl shadow-md transition-all"
        >
          <div class="pl-3 text-neutral-500">
            <Search class="w-3.5 h-3.5" />
          </div>
          <input
            type="text"
            bind:value={targetUrl}
            placeholder="Enter target domain (e.g. example.com)..."
            class="w-full py-2 bg-transparent text-xs font-mono text-white placeholder-neutral-500 focus:outline-none"
          />
          <button
            type="submit"
            disabled={!targetUrl.trim() || isScanning}
            class="px-4 py-1.5 bg-white hover:bg-neutral-200 disabled:opacity-50 text-neutral-950 font-semibold text-xs rounded-lg transition-colors cursor-pointer disabled:cursor-not-allowed flex-shrink-0 shadow-sm"
          >
            Audit
          </button>
        </form>
      </div>

      <!-- Quick presets -->
      <div class="mt-4 flex flex-wrap items-center justify-center gap-1.5">
        <span class="text-[11px] text-neutral-500 font-mono mr-1">Presets:</span>
        {#each ["example.com", "httpbin.org", "testphp.vulnweb.com"] as preset}
          <button
            type="button"
            onclick={() => {
              const url = preset.startsWith("http") ? preset : `https://${preset}`;
              targetUrl = url;
              handleScan(url);
            }}
            class="px-2.5 py-0.5 bg-[#1e1e1e] hover:bg-[#282828] border border-[#2e2e2e] rounded-md text-xs font-mono text-neutral-400 hover:text-white transition-colors cursor-pointer"
          >
            {preset}
          </button>
        {/each}
      </div>

      <!-- Subtle Keyboard Hints -->
      <div class="mt-12 flex flex-wrap items-center justify-center gap-4 text-[11px] text-neutral-500 font-mono">
        <div class="flex items-center gap-1">
          <kbd class="px-1.5 py-0.5 bg-[#1b1b1b] border border-[#2e2e2e] rounded text-neutral-300">⌘K</kbd>
          <span>Focus input</span>
        </div>
        <div class="flex items-center gap-1">
          <kbd class="px-1.5 py-0.5 bg-[#1b1b1b] border border-[#2e2e2e] rounded text-neutral-300">⌘,</kbd>
          <span>Settings</span>
        </div>
        <div class="flex items-center gap-1">
          <kbd class="px-1.5 py-0.5 bg-[#1b1b1b] border border-[#2e2e2e] rounded text-neutral-300">⌘H</kbd>
          <span>History</span>
        </div>
      </div>
    </div>
  {/if}
</main>

<!-- Unified Settings Hub Modal -->
<SettingsModal
  isOpen={isSettingsOpen}
  activeTab={settingsTab}
  options={scanOptions}
  {monitors}
  historyCount={history.length}
  onApplyOptions={(newOpts) => {
    scanOptions = newOpts;
    showToast("Audit parameters applied", "success");
  }}
  onAddMonitor={handleAddMonitor}
  onDeleteMonitor={handleDeleteMonitor}
  onToggleMonitor={handleToggleMonitor}
  onScanTarget={(url) => {
    targetUrl = url;
    handleScan(url);
  }}
  onClearHistory={handleClearAllHistory}
  onOpenHistory={() => {
    isSettingsOpen = false;
    isHistoryOpen = true;
  }}
  onSelectBatchReport={(rep) => {
    report = rep;
    targetUrl = rep.target_url;
  }}
  onClose={() => (isSettingsOpen = false)}
/>

<!-- History Drawer Modal -->
<HistoryModal
  isOpen={isHistoryOpen}
  {history}
  onSelect={handleSelectFromHistory}
  onDelete={handleDeleteFromHistory}
  onClearAll={handleClearAllHistory}
  onClose={() => (isHistoryOpen = false)}
/>

<!-- Export Report Modal -->
<ExportModal
  isOpen={isExportOpen}
  {report}
  markdownContent={exportMarkdown}
  onOpenExecutiveReport={() => (isExecutiveReportOpen = true)}
  onClose={() => (isExportOpen = false)}
/>

<!-- Executive PDF / Print Report Modal -->
<ExecutiveReportModal
  isOpen={isExecutiveReportOpen}
  {report}
  onClose={() => (isExecutiveReportOpen = false)}
/>

<!-- Batch Fleet Scanner Modal (Standalone shortcut target) -->
<BatchScanModal
  isOpen={isBatchOpen}
  options={scanOptions}
  onSelectReport={(rep) => {
    report = rep;
    targetUrl = rep.target_url;
  }}
  onClose={() => (isBatchOpen = false)}
/>

<!-- Scan Options Modal (Standalone shortcut target) -->
<ScanOptionsModal
  isOpen={isOptionsOpen}
  options={scanOptions}
  onApply={(newOpts) => {
    scanOptions = newOpts;
    showToast("Audit parameters applied", "success");
  }}
  onClose={() => (isOptionsOpen = false)}
/>

<!-- Continuous Monitor Modal (Standalone shortcut target) -->
<MonitorModal
  isOpen={isMonitorsOpen}
  {monitors}
  onAddMonitor={handleAddMonitor}
  onDeleteMonitor={handleDeleteMonitor}
  onToggleMonitor={handleToggleMonitor}
  onScanNow={(url) => {
    targetUrl = url;
    handleScan(url);
  }}
  onClose={() => (isMonitorsOpen = false)}
/>

<!-- Keyboard Shortcuts Helper Modal (Standalone shortcut target) -->
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

