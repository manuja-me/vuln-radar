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
  let isHistoryOpen = $state(false);
  let isExportOpen = $state(false);
  let isExecutiveReportOpen = $state(false);
  let isBatchOpen = $state(false);
  let isOptionsOpen = $state(false);
  let isMonitorsOpen = $state(false);
  let exportMarkdown = $state("");

  // Watchdog Alert Banner
  let watchdogAlert = $state<{
    target_url: string;
    new_score: number;
    previous_score: number;
    critical_count: number;
  } | null>(null);

  // Filters
  let searchQuery = $state("");
  let selectedSeverity = $state<Severity | "all">("all");
  let selectedCategory = $state<Category | "all">("all");

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

  onMount(() => {
    loadHistory();
    loadMonitors();

    // Listen for background watchdog alerts
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlisten = await listen<any>("monitor_alert", (event) => {
          watchdogAlert = event.payload;
          loadMonitors();
          loadHistory();
        });
      } catch {
        // ignore in non-tauri dev
      }
    })();

    return () => {
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
    } catch (err: any) {
      scanError =
        err?.toString() ||
        "Failed to scan target. Please check the URL and internet connection.";
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
    } catch (err) {
      console.error(err);
    }
  }

  async function handleClearAllHistory() {
    try {
      await invokeTauri("clear_history");
      history = [];
    } catch (err) {
      console.error(err);
    }
  }

  async function handleAddMonitor(url: string, intervalHours: number) {
    try {
      await invokeTauri("add_monitor", { url, intervalHours });
      await loadMonitors();
    } catch (err) {
      console.error("Failed to add monitor:", err);
    }
  }

  async function handleDeleteMonitor(id: string) {
    try {
      await invokeTauri("delete_monitor", { id });
      await loadMonitors();
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

  const filteredFindings = $derived.by(() => {
    if (!report) return [];
    return report.findings.filter((finding) => {
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
      setTimeout(() => (copiedUrl = false), 2000);
    } catch {}
  }

  async function copyPortAddress(host: string, port: number) {
    try {
      await navigator.clipboard.writeText(`${host}:${port}`);
      copiedPort = port;
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
  onScan={() => handleScan()}
  onOpenOptions={() => (isOptionsOpen = true)}
  onOpenBatch={() => (isBatchOpen = true)}
  onOpenMonitors={() => (isMonitorsOpen = true)}
  onOpenHistory={() => (isHistoryOpen = true)}
  onOpenExport={openExportModal}
  onOpenExecutiveReport={() => (isExecutiveReportOpen = true)}
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
          </div>

          <div class="flex items-center gap-2.5">
            <h1 class="text-xl font-bold text-white tracking-tight truncate flex items-center gap-2">
              <Globe class="w-5 h-5 text-neutral-400 flex-shrink-0" />
              <span class="truncate">{report.target_url}</span>
            </h1>
            <button
              type="button"
              onclick={copyTargetUrl}
              class="p-1 text-neutral-400 hover:text-white hover:bg-[#282828] rounded cursor-pointer transition-colors"
              title="Copy URL"
            >
              {#if copiedUrl}
                <CheckCircle2 class="w-3.5 h-3.5 text-emerald-400" />
              {:else}
                <Search class="w-3.5 h-3.5" />
              {/if}
            </button>
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

      <!-- Severity Metrics Grid -->
      <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-2.5">
        <div class="p-3 bg-[#202020] border border-[#2e2e2e] rounded-lg flex flex-col">
          <span class="text-[11px] font-medium text-neutral-400 uppercase tracking-wider">Total Issues</span>
          <span class="text-xl font-bold text-white mt-0.5 font-mono">{report.total_findings}</span>
        </div>
        <div class="p-3 bg-[#202020] border border-[#2e2e2e] rounded-lg flex flex-col">
          <span class="text-[11px] font-medium text-red-400 uppercase tracking-wider">Critical</span>
          <span class="text-xl font-bold text-red-400 mt-0.5 font-mono">{report.critical_count}</span>
        </div>
        <div class="p-3 bg-[#202020] border border-[#2e2e2e] rounded-lg flex flex-col">
          <span class="text-[11px] font-medium text-orange-400 uppercase tracking-wider">High</span>
          <span class="text-xl font-bold text-orange-400 mt-0.5 font-mono">{report.high_count}</span>
        </div>
        <div class="p-3 bg-[#202020] border border-[#2e2e2e] rounded-lg flex flex-col">
          <span class="text-[11px] font-medium text-amber-400 uppercase tracking-wider">Medium</span>
          <span class="text-xl font-bold text-amber-400 mt-0.5 font-mono">{report.medium_count}</span>
        </div>
        <div class="p-3 bg-[#202020] border border-[#2e2e2e] rounded-lg flex flex-col">
          <span class="text-[11px] font-medium text-blue-400 uppercase tracking-wider">Low</span>
          <span class="text-xl font-bold text-blue-400 mt-0.5 font-mono">{report.low_count}</span>
        </div>
        <div class="p-3 bg-[#202020] border border-[#2e2e2e] rounded-lg flex flex-col">
          <span class="text-[11px] font-medium text-neutral-400 uppercase tracking-wider">Info</span>
          <span class="text-xl font-bold text-neutral-300 mt-0.5 font-mono">{report.info_count}</span>
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
        <div class="p-3 bg-[#202020] border border-[#2e2e2e] rounded-xl flex flex-col md:flex-row items-center justify-between gap-3">
          <div class="relative w-full md:w-80">
            <Search class="w-3.5 h-3.5 text-neutral-500 absolute inset-y-0 left-3 my-auto pointer-events-none" />
            <input
              type="text"
              bind:value={searchQuery}
              placeholder="Search findings, CVEs, OWASP..."
              class="w-full pl-8 pr-3 py-1.5 bg-[#191919] border border-[#2e2e2e] focus:border-neutral-500 rounded-lg text-xs text-neutral-200 placeholder-neutral-500 font-mono focus:outline-none"
            />
          </div>

          <div class="flex flex-wrap items-center gap-1 w-full md:w-auto">
            <button
              type="button"
              onclick={() => (selectedSeverity = "all")}
              class="px-2.5 py-1 rounded-md text-xs font-medium cursor-pointer transition-colors {selectedSeverity === 'all' ? 'bg-white text-neutral-950 font-semibold' : 'bg-[#191919] text-neutral-400 hover:text-neutral-200'}"
            >
              All ({report.total_findings})
            </button>
            <button
              type="button"
              onclick={() => (selectedSeverity = "critical")}
              class="px-2.5 py-1 rounded-md text-xs font-medium cursor-pointer transition-colors {selectedSeverity === 'critical' ? 'bg-red-950 text-red-200 border border-red-800' : 'bg-red-950/20 text-red-400 hover:bg-red-950/40'}"
            >
              Critical ({report.critical_count})
            </button>
            <button
              type="button"
              onclick={() => (selectedSeverity = "high")}
              class="px-2.5 py-1 rounded-md text-xs font-medium cursor-pointer transition-colors {selectedSeverity === 'high' ? 'bg-orange-950 text-orange-200 border border-orange-800' : 'bg-orange-950/20 text-orange-400 hover:bg-orange-950/40'}"
            >
              High ({report.high_count})
            </button>
            <button
              type="button"
              onclick={() => (selectedSeverity = "medium")}
              class="px-2.5 py-1 rounded-md text-xs font-medium cursor-pointer transition-colors {selectedSeverity === 'medium' ? 'bg-amber-950 text-amber-200 border border-amber-800' : 'bg-amber-950/20 text-amber-400 hover:bg-amber-950/40'}"
            >
              Med ({report.medium_count})
            </button>
            <button
              type="button"
              onclick={() => (selectedSeverity = "low")}
              class="px-2.5 py-1 rounded-md text-xs font-medium cursor-pointer transition-colors {selectedSeverity === 'low' ? 'bg-blue-950 text-blue-200 border border-blue-800' : 'bg-blue-950/20 text-blue-400 hover:bg-blue-950/40'}"
            >
              Low ({report.low_count})
            </button>
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
                  isOptionsOpen = true;
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
    <div class="my-auto py-12 flex flex-col items-center justify-center text-center max-w-2xl mx-auto animate-fade-in">
      <div class="w-12 h-12 rounded-xl bg-[#262626] border border-[#333333] flex items-center justify-center text-neutral-200 mb-4">
        <ShieldCheck class="w-6 h-6 text-white" />
      </div>

      <div class="inline-flex items-center gap-2 px-2.5 py-0.5 rounded bg-[#252525] border border-[#333] text-neutral-400 text-[11px] font-medium mb-3">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
        <span>Passive Web Security & Reconnaissance</span>
      </div>

      <h1 class="text-3xl font-bold text-white tracking-tight sm:text-4xl leading-tight">
        Web Security Posture Scanner
      </h1>
      <p class="text-xs text-neutral-400 mt-2 max-w-md leading-relaxed">
        Perform real-time passive reconnaissance across HTTP headers, TLS ciphers, cookies, Certificate Transparency subdomains, open TCP ports, and known CVE dependencies.
      </p>

      <!-- Center-Stage Instant Scan Input -->
      <div class="w-full max-w-md mt-6">
        <form
          onsubmit={(e) => {
            e.preventDefault();
            handleScan();
          }}
          class="flex items-center gap-1.5 p-1 bg-[#202020] border border-[#2e2e2e] focus-within:border-neutral-500 rounded-lg shadow-sm"
        >
          <div class="pl-2.5 text-neutral-500">
            <Search class="w-3.5 h-3.5" />
          </div>
          <input
            type="text"
            bind:value={targetUrl}
            placeholder="Enter target domain or URL (e.g. example.com)..."
            class="w-full py-1.5 bg-transparent text-xs font-mono text-white placeholder-neutral-500 focus:outline-none"
          />
          <button
            type="submit"
            disabled={!targetUrl.trim() || isScanning}
            class="px-3.5 py-1.5 bg-white hover:bg-neutral-200 disabled:opacity-50 text-neutral-950 font-semibold text-xs rounded-md transition-colors cursor-pointer disabled:cursor-not-allowed flex-shrink-0"
          >
            Audit
          </button>
        </form>
      </div>

      <!-- Quick presets -->
      <div class="mt-4 flex flex-wrap items-center justify-center gap-1.5">
        <span class="text-xs text-neutral-500 font-medium mr-1">Presets:</span>
        <button
          type="button"
          onclick={() => {
            targetUrl = "https://example.com";
            handleScan("https://example.com");
          }}
          class="px-2.5 py-0.5 bg-[#202020] hover:bg-[#262626] border border-[#2e2e2e] rounded text-xs font-mono text-neutral-300 transition-colors cursor-pointer"
        >
          example.com
        </button>
        <button
          type="button"
          onclick={() => {
            targetUrl = "http://testphp.vulnweb.com";
            handleScan("http://testphp.vulnweb.com");
          }}
          class="px-2.5 py-0.5 bg-[#202020] hover:bg-[#262626] border border-[#2e2e2e] rounded text-xs font-mono text-neutral-300 transition-colors cursor-pointer"
        >
          testphp.vulnweb.com
        </button>
        <button
          type="button"
          onclick={() => {
            targetUrl = "https://httpbin.org";
            handleScan("https://httpbin.org");
          }}
          class="px-2.5 py-0.5 bg-[#202020] hover:bg-[#262626] border border-[#2e2e2e] rounded text-xs font-mono text-neutral-300 transition-colors cursor-pointer"
        >
          httpbin.org
        </button>
      </div>

      <!-- Feature Badges -->
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 mt-10 w-full text-left">
        <div class="p-4 bg-[#202020] border border-[#2e2e2e] rounded-xl">
          <div class="w-7 h-7 rounded-lg bg-[#282828] border border-[#383838] flex items-center justify-center text-neutral-300 mb-2">
            <Globe class="w-3.5 h-3.5" />
          </div>
          <div class="text-white text-xs font-semibold mb-0.5">Recon & Subdomains</div>
          <p class="text-xs text-neutral-400 leading-relaxed">Asset mapping via Certificate Transparency logs and SPF/DMARC anti-spoofing.</p>
        </div>

        <div class="p-4 bg-[#202020] border border-[#2e2e2e] rounded-xl">
          <div class="w-7 h-7 rounded-lg bg-[#282828] border border-[#383838] flex items-center justify-center text-neutral-300 mb-2">
            <Server class="w-3.5 h-3.5" />
          </div>
          <div class="text-white text-xs font-semibold mb-0.5">TCP Port Probing</div>
          <p class="text-xs text-neutral-400 leading-relaxed">Asynchronous port discovery and banner grabbing for exposed risky databases and services.</p>
        </div>

        <div class="p-4 bg-[#202020] border border-[#2e2e2e] rounded-xl">
          <div class="w-7 h-7 rounded-lg bg-[#282828] border border-[#383838] flex items-center justify-center text-neutral-300 mb-2">
            <Activity class="w-3.5 h-3.5" />
          </div>
          <div class="text-white text-xs font-semibold mb-0.5">Watchdog Automation</div>
          <p class="text-xs text-neutral-400 leading-relaxed">Continuous scheduled monitoring with score change alerts and archive snapshots.</p>
        </div>
      </div>
    </div>
  {/if}
</main>

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
  onClose={() => (isExportOpen = false)}
/>

<!-- Executive PDF / Print Report Modal -->
<ExecutiveReportModal
  isOpen={isExecutiveReportOpen}
  {report}
  onClose={() => (isExecutiveReportOpen = false)}
/>

<!-- Batch Fleet Scanner Modal -->
<BatchScanModal
  isOpen={isBatchOpen}
  options={scanOptions}
  onSelectReport={(rep) => {
    report = rep;
    targetUrl = rep.target_url;
  }}
  onClose={() => (isBatchOpen = false)}
/>

<!-- Scan Options & Custom Headers Modal -->
<ScanOptionsModal
  isOpen={isOptionsOpen}
  bind:options={scanOptions}
  onClose={() => (isOptionsOpen = false)}
/>

<!-- Continuous Monitor Modal -->
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

