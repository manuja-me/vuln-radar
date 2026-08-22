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
      scanOptions.include_subdomains === false
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
    { id: "dns_email_security", label: "DNS & Email" },
    { id: "endpoint_exposure", label: "Endpoints / Recon" },
    { id: "vulnerable_dependency", label: "Dependencies (CVEs)" },
    { id: "information_disclosure", label: "Info Leaks" },
    { id: "tls_ssl", label: "TLS / HTTPS" },
    { id: "cors_misconfiguration", label: "CORS" },
    { id: "insecure_form", label: "Forms" },
  ];

  // Active Tab in Report View
  let activeTab = $state<"findings" | "recon" | "dns" | "endpoints">("findings");
  let subdomainSearch = $state("");
  let copiedUrl = $state(false);

  async function copyTargetUrl() {
    if (!report) return;
    try {
      await navigator.clipboard.writeText(report.target_url);
      copiedUrl = true;
      setTimeout(() => (copiedUrl = false), 2000);
    } catch {}
  }

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
      <!-- High-Tech Cyber Radar Spinner -->
      <div class="relative w-24 h-24 mb-8 flex items-center justify-center">
        <div class="absolute inset-0 rounded-full border border-cyan-500/20 animate-ping"></div>
        <div class="absolute inset-2 rounded-full border border-cyan-500/40"></div>
        <div class="absolute inset-4 rounded-full border border-cyan-500/60 animate-pulse"></div>
        <div class="w-12 h-12 rounded-2xl bg-cyan-500/10 border border-cyan-500/50 flex items-center justify-center text-cyan-400 shadow-xl shadow-cyan-500/20">
          <Sparkles class="w-6 h-6 animate-spin" />
        </div>
      </div>

      <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/30 text-cyan-400 text-xs font-mono font-bold mb-3">
        <span class="w-2 h-2 rounded-full bg-cyan-400 animate-ping"></span>
        LIVE SECURITY AUDIT IN PROGRESS
      </div>

      <h2 class="text-2xl font-black text-slate-100 tracking-tight">Auditing Target Surface</h2>
      <p class="text-xs font-mono text-cyan-400 mt-2 max-w-md truncate bg-slate-900/80 px-3 py-1.5 rounded-lg border border-slate-800">
        {targetUrl}
      </p>

      <p class="text-xs text-slate-400 mt-4 max-w-md leading-relaxed">
        Evaluating HTTP security headers, TLS ciphers, cookie flags, Certificate Transparency subdomains, DNS SPF/DMARC anti-spoofing, and front-end CVE dependencies...
      </p>
    </div>

  <!-- Scan Error State -->
  {:else if scanError}
    <div class="my-auto max-w-xl mx-auto p-6 bg-rose-950/20 border border-rose-900/60 rounded-2xl flex items-start gap-4 text-rose-300 shadow-xl">
      <AlertOctagon class="w-7 h-7 flex-shrink-0 text-rose-400 mt-0.5" />
      <div class="space-y-2">
        <h3 class="text-base font-bold text-rose-200">Security Audit Encountered an Error</h3>
        <p class="text-xs text-rose-300/90 font-mono bg-rose-950/50 p-2.5 rounded-lg border border-rose-900/40">{scanError}</p>
        <button
          type="button"
          onclick={() => handleScan()}
          class="mt-2 px-4 py-2 bg-rose-500 hover:bg-rose-400 text-slate-950 rounded-xl text-xs font-bold transition-colors cursor-pointer shadow-lg shadow-rose-500/20"
        >
          Retry Scan
        </button>
      </div>
    </div>

  <!-- Active Report View -->
  {:else if report}
    <div class="space-y-6 animate-fade-in">
      <!-- Target Summary Header Card -->
      <div class="p-6 bg-slate-900/60 border border-slate-800/90 rounded-2xl flex flex-col lg:flex-row items-start lg:items-center justify-between gap-6 shadow-xl backdrop-blur-md">
        <div class="space-y-2.5 min-w-0 flex-1">
          <div class="flex flex-wrap items-center gap-2">
            <span class="px-2.5 py-0.5 text-xs font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 rounded-md">
              HTTP {report.status_code}
            </span>
            <span class="text-xs text-slate-400 font-mono">
              Latency: <strong class="text-slate-200">{report.response_time_ms} ms</strong>
            </span>
            <span class="text-xs text-slate-600">•</span>
            <span class="text-xs text-slate-400 font-mono">
              Audited at {new Date(report.scanned_at).toLocaleTimeString()}
            </span>
          </div>

          <div class="flex items-center gap-3">
            <h1 class="text-2xl font-black text-white tracking-tight truncate flex items-center gap-2.5">
              <Globe class="w-6 h-6 text-cyan-400 flex-shrink-0" />
              <span class="truncate">{report.target_url}</span>
            </h1>
            <button
              type="button"
              onclick={copyTargetUrl}
              class="p-1.5 bg-slate-800/80 hover:bg-slate-800 text-slate-400 hover:text-cyan-400 border border-slate-700/60 rounded-lg cursor-pointer transition-colors"
              title="Copy URL"
            >
              {#if copiedUrl}
                <CheckCircle2 class="w-4 h-4 text-emerald-400" />
              {:else}
                <Search class="w-4 h-4" />
              {/if}
            </button>
          </div>

          <!-- Detected Tech Stack Tags -->
          {#if report.technologies_detected.length > 0}
            <div class="flex flex-wrap items-center gap-1.5 pt-1">
              <span class="text-xs text-slate-400 font-bold uppercase tracking-wider flex items-center gap-1 mr-1">
                <Cpu class="w-3.5 h-3.5 text-cyan-400" /> Fingerprint:
              </span>
              {#each report.technologies_detected as tech}
                <span class="px-2.5 py-0.5 text-xs font-mono bg-slate-950/80 text-slate-300 border border-slate-800 rounded-md">
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
      <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
        <div class="p-4 bg-slate-900/40 border border-slate-800/80 rounded-xl flex flex-col shadow-sm">
          <span class="text-[11px] font-bold text-slate-400 uppercase tracking-wider font-mono">Total Issues</span>
          <span class="text-2xl font-black text-slate-100 mt-1 font-mono">{report.total_findings}</span>
        </div>
        <div class="p-4 bg-rose-500/5 border border-rose-500/20 rounded-xl flex flex-col shadow-sm hover:border-rose-500/40 transition-colors">
          <span class="text-[11px] font-bold text-rose-400 uppercase tracking-wider font-mono">Critical</span>
          <span class="text-2xl font-black text-rose-400 mt-1 font-mono">{report.critical_count}</span>
        </div>
        <div class="p-4 bg-orange-500/5 border border-orange-500/20 rounded-xl flex flex-col shadow-sm hover:border-orange-500/40 transition-colors">
          <span class="text-[11px] font-bold text-orange-400 uppercase tracking-wider font-mono">High</span>
          <span class="text-2xl font-black text-orange-400 mt-1 font-mono">{report.high_count}</span>
        </div>
        <div class="p-4 bg-amber-500/5 border border-amber-500/20 rounded-xl flex flex-col shadow-sm hover:border-amber-500/40 transition-colors">
          <span class="text-[11px] font-bold text-amber-400 uppercase tracking-wider font-mono">Medium</span>
          <span class="text-2xl font-black text-amber-400 mt-1 font-mono">{report.medium_count}</span>
        </div>
        <div class="p-4 bg-blue-500/5 border border-blue-500/20 rounded-xl flex flex-col shadow-sm hover:border-blue-500/40 transition-colors">
          <span class="text-[11px] font-bold text-blue-400 uppercase tracking-wider font-mono">Low</span>
          <span class="text-2xl font-black text-blue-400 mt-1 font-mono">{report.low_count}</span>
        </div>
        <div class="p-4 bg-slate-800/20 border border-slate-700/40 rounded-xl flex flex-col shadow-sm">
          <span class="text-[11px] font-bold text-slate-400 uppercase tracking-wider font-mono">Info</span>
          <span class="text-2xl font-black text-slate-300 mt-1 font-mono">{report.info_count}</span>
        </div>
      </div>

      <!-- Navigation Tabs (Findings, DNS/Email, Endpoints, Subdomain Map) -->
      <div class="border-b border-slate-800 flex items-center gap-2 overflow-x-auto pb-px">
        <button
          type="button"
          onclick={() => (activeTab = "findings")}
          class="px-4 py-2.5 text-xs font-bold uppercase tracking-wider flex items-center gap-2 border-b-2 transition-all cursor-pointer {activeTab === 'findings' ? 'border-cyan-400 text-cyan-400 bg-cyan-500/5' : 'border-transparent text-slate-400 hover:text-slate-200'}"
        >
          <ShieldCheck class="w-4 h-4" />
          <span>Vulnerabilities ({report.findings.length})</span>
        </button>

        <button
          type="button"
          onclick={() => (activeTab = "dns")}
          class="px-4 py-2.5 text-xs font-bold uppercase tracking-wider flex items-center gap-2 border-b-2 transition-all cursor-pointer {activeTab === 'dns' ? 'border-cyan-400 text-cyan-400 bg-cyan-500/5' : 'border-transparent text-slate-400 hover:text-slate-200'}"
        >
          <Mail class="w-4 h-4" />
          <span>DNS & Email Posture</span>
        </button>

        <button
          type="button"
          onclick={() => (activeTab = "endpoints")}
          class="px-4 py-2.5 text-xs font-bold uppercase tracking-wider flex items-center gap-2 border-b-2 transition-all cursor-pointer {activeTab === 'endpoints' ? 'border-cyan-400 text-cyan-400 bg-cyan-500/5' : 'border-transparent text-slate-400 hover:text-slate-200'}"
        >
          <FileCode class="w-4 h-4" />
          <span>Endpoint Policies</span>
        </button>

        <button
          type="button"
          onclick={() => (activeTab = "recon")}
          class="px-4 py-2.5 text-xs font-bold uppercase tracking-wider flex items-center gap-2 border-b-2 transition-all cursor-pointer {activeTab === 'recon' ? 'border-cyan-400 text-cyan-400 bg-cyan-500/5' : 'border-transparent text-slate-400 hover:text-slate-200'}"
        >
          <Globe class="w-4 h-4" />
          <span>Subdomain Map ({report.subdomains?.length || 0})</span>
        </button>
      </div>

      <!-- TAB 1: Findings View -->
      {#if activeTab === "findings"}
        <!-- Search & Filter Controls -->
        <div class="p-4 bg-slate-900/40 border border-slate-800 rounded-2xl flex flex-col md:flex-row items-center justify-between gap-4">
          <div class="relative w-full md:w-80">
            <Search class="w-4 h-4 text-slate-500 absolute inset-y-0 left-3 my-auto pointer-events-none" />
            <input
              type="text"
              bind:value={searchQuery}
              placeholder="Search CVEs, OWASP, or keyword..."
              class="w-full pl-9 pr-3 py-2 bg-slate-950/80 border border-slate-800 focus:border-cyan-500 rounded-xl text-xs text-slate-200 placeholder-slate-500 font-mono shadow-inner"
            />
          </div>

          <div class="flex flex-wrap items-center gap-1.5 w-full md:w-auto">
            <button
              type="button"
              onclick={() => (selectedSeverity = "all")}
              class="px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition-colors {selectedSeverity === 'all' ? 'bg-cyan-500 text-slate-950 font-bold' : 'bg-slate-800 text-slate-400 hover:text-slate-200'}"
            >
              All ({report.total_findings})
            </button>
            <button
              type="button"
              onclick={() => (selectedSeverity = "critical")}
              class="px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition-colors {selectedSeverity === 'critical' ? 'bg-rose-500 text-white font-bold' : 'bg-rose-500/10 text-rose-400 hover:bg-rose-500/20'}"
            >
              Critical ({report.critical_count})
            </button>
            <button
              type="button"
              onclick={() => (selectedSeverity = "high")}
              class="px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition-colors {selectedSeverity === 'high' ? 'bg-orange-500 text-white font-bold' : 'bg-orange-500/10 text-orange-400 hover:bg-orange-500/20'}"
            >
              High ({report.high_count})
            </button>
            <button
              type="button"
              onclick={() => (selectedSeverity = "medium")}
              class="px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition-colors {selectedSeverity === 'medium' ? 'bg-amber-500 text-slate-950 font-bold' : 'bg-amber-500/10 text-amber-400 hover:bg-amber-500/20'}"
            >
              Med ({report.medium_count})
            </button>
            <button
              type="button"
              onclick={() => (selectedSeverity = "low")}
              class="px-2.5 py-1 rounded-lg text-xs font-semibold cursor-pointer transition-colors {selectedSeverity === 'low' ? 'bg-blue-500 text-white font-bold' : 'bg-blue-500/10 text-blue-400 hover:bg-blue-500/20'}"
            >
              Low ({report.low_count})
            </button>
          </div>
        </div>

        <!-- Findings List -->
        <div class="space-y-3">
          <div class="flex items-center justify-between px-1">
            <h2 class="text-xs font-bold uppercase tracking-wider font-mono text-slate-400">
              Security Findings ({filteredFindings.length})
            </h2>
            {#if selectedSeverity !== "all" || searchQuery.trim()}
              <button
                type="button"
                onclick={() => {
                  selectedSeverity = "all";
                  searchQuery = "";
                }}
                class="text-xs text-cyan-400 hover:underline cursor-pointer font-mono"
              >
                Reset Filters
              </button>
            {/if}
          </div>

          {#if filteredFindings.length === 0}
            <div class="py-16 text-center bg-slate-900/30 border border-slate-800 rounded-2xl">
              <CheckCircle2 class="w-12 h-12 text-emerald-400 mx-auto mb-3 opacity-80" />
              <h3 class="text-base font-bold text-slate-200">No matching issues found</h3>
              <p class="text-xs text-slate-400 mt-1">No vulnerabilities match the current filter selection.</p>
            </div>
          {:else}
            {#each filteredFindings as finding (finding.id)}
              <FindingCard {finding} />
            {/each}
          {/if}
        </div>

      <!-- TAB 2: DNS & Email Hardening Posture -->
      {:else if activeTab === "dns"}
        <div class="space-y-4">
          {#if report.dns_security}
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div class="p-5 bg-slate-900/60 border border-slate-800 rounded-2xl space-y-3">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-mono font-bold uppercase tracking-wider text-slate-400">SPF Anti-Spoofing</span>
                  <span class="px-2 py-0.5 text-xs font-mono font-bold rounded {report.dns_security.spf_record ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/30' : 'bg-rose-500/10 text-rose-400 border border-rose-500/30'}">
                    {report.dns_security.spf_record ? "Configured" : "Missing"}
                  </span>
                </div>
                <p class="text-xs text-slate-400">Validates authorized mail senders via Sender Policy Framework.</p>
                {#if report.dns_security.spf_record}
                  <pre class="bg-slate-950 p-3 rounded-lg text-xs font-mono text-cyan-300 border border-slate-800 overflow-x-auto whitespace-pre-wrap">{report.dns_security.spf_record}</pre>
                {/if}
              </div>

              <div class="p-5 bg-slate-900/60 border border-slate-800 rounded-2xl space-y-3">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-mono font-bold uppercase tracking-wider text-slate-400">DMARC Policy</span>
                  <span class="px-2 py-0.5 text-xs font-mono font-bold rounded {report.dns_security.dmarc_policy && report.dns_security.dmarc_policy !== 'none' ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/30' : 'bg-amber-500/10 text-amber-400 border border-amber-500/30'}">
                    {report.dns_security.dmarc_policy || "None"}
                  </span>
                </div>
                <p class="text-xs text-slate-400">Enforces domain-based message authentication and alignment.</p>
                {#if report.dns_security.dmarc_record}
                  <pre class="bg-slate-950 p-3 rounded-lg text-xs font-mono text-cyan-300 border border-slate-800 overflow-x-auto whitespace-pre-wrap">{report.dns_security.dmarc_record}</pre>
                {/if}
              </div>

              <div class="p-5 bg-slate-900/60 border border-slate-800 rounded-2xl space-y-3">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-mono font-bold uppercase tracking-wider text-slate-400">DNSSEC Enforcement</span>
                  <span class="px-2 py-0.5 text-xs font-mono font-bold rounded {report.dns_security.dnssec_enabled ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/30' : 'bg-slate-800 text-slate-400'}">
                    {report.dns_security.dnssec_enabled ? "Enabled" : "Disabled"}
                  </span>
                </div>
                <p class="text-xs text-slate-400">Cryptographically authenticates DNS responses against spoofing.</p>
                <div class="text-xs text-slate-500 font-mono pt-2">
                  Authenticated Data (AD) Flag: {report.dns_security.dnssec_enabled ? "Verified" : "Not Present"}
                </div>
              </div>
            </div>
          {:else}
            <div class="p-8 text-center bg-slate-900/40 border border-slate-800 rounded-2xl text-slate-400 text-xs">
              DNS security was not inspected for this scan.
            </div>
          {/if}
        </div>

      <!-- TAB 3: Endpoint Policies (robots.txt, security.txt) -->
      {:else if activeTab === "endpoints"}
        <div class="space-y-4">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <!-- robots.txt -->
            <div class="p-5 bg-slate-900/60 border border-slate-800 rounded-2xl space-y-3">
              <div class="flex items-center justify-between">
                <span class="text-xs font-mono font-bold uppercase tracking-wider text-slate-300">robots.txt Disallow Rules</span>
                <span class="px-2 py-0.5 text-xs font-mono font-bold rounded bg-slate-800 text-slate-300">
                  {report.endpoint_report?.disallowed_paths.length || 0} paths
                </span>
              </div>
              {#if report.endpoint_report && report.endpoint_report.disallowed_paths.length > 0}
                <div class="max-h-56 overflow-y-auto space-y-1 bg-slate-950 p-3 rounded-lg border border-slate-800 text-xs font-mono">
                  {#each report.endpoint_report.disallowed_paths as path}
                    <div class="text-slate-400">Disallow: {path}</div>
                  {/each}
                </div>
              {:else}
                <p class="text-xs text-slate-500">No robots.txt disallowed rules found.</p>
              {/if}
            </div>

            <!-- Sensitive Exposed Paths -->
            <div class="p-5 bg-slate-900/60 border border-slate-800 rounded-2xl space-y-3">
              <div class="flex items-center justify-between">
                <span class="text-xs font-mono font-bold uppercase tracking-wider text-slate-300">Sensitive Paths Disclosed</span>
                <span class="px-2 py-0.5 text-xs font-mono font-bold rounded {(report.endpoint_report?.sensitive_disallowed_paths.length || 0) > 0 ? 'bg-amber-500/10 text-amber-400 border border-amber-500/30' : 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/30'}">
                  {report.endpoint_report?.sensitive_disallowed_paths.length || 0} exposed
                </span>
              </div>
              {#if report.endpoint_report && report.endpoint_report.sensitive_disallowed_paths.length > 0}
                <div class="max-h-56 overflow-y-auto space-y-1 bg-amber-950/20 p-3 rounded-lg border border-amber-900/40 text-xs font-mono">
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

      <!-- TAB 4: Subdomain Discovery Map -->
      {:else if activeTab === "recon"}
        <div class="space-y-4">
          <div class="p-4 bg-slate-900/40 border border-slate-800 rounded-2xl flex items-center justify-between gap-4">
            <div class="relative w-full max-w-sm">
              <Search class="w-4 h-4 text-slate-500 absolute inset-y-0 left-3 my-auto pointer-events-none" />
              <input
                type="text"
                bind:value={subdomainSearch}
                placeholder="Filter discovered subdomains..."
                class="w-full pl-9 pr-3 py-2 bg-slate-950 border border-slate-800 focus:border-cyan-500 rounded-xl text-xs text-slate-200 placeholder-slate-500 font-mono"
              />
            </div>
            <span class="text-xs font-mono text-cyan-400 font-bold">
              {filteredSubdomains.length} / {report.subdomains?.length || 0} Total
            </span>
          </div>

          {#if filteredSubdomains.length > 0}
            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2.5">
              {#each filteredSubdomains as sub}
                <button
                  type="button"
                  onclick={() => {
                    targetUrl = `https://${sub}`;
                    handleScan(`https://${sub}`);
                  }}
                  class="p-3 bg-slate-900/60 hover:bg-slate-850 hover:border-cyan-500/40 border border-slate-800 rounded-xl text-left text-xs font-mono text-slate-300 hover:text-cyan-300 transition-all cursor-pointer truncate flex items-center justify-between group shadow-sm"
                >
                  <span class="truncate">{sub}</span>
                  <ExternalLink class="w-3.5 h-3.5 text-slate-600 group-hover:text-cyan-400 flex-shrink-0 ml-2" />
                </button>
              {/each}
            </div>
          {:else}
            <div class="p-12 text-center bg-slate-900/30 border border-slate-800 rounded-2xl text-slate-500 text-xs">
              No subdomains found matching current search.
            </div>
          {/if}
        </div>
      {/if}
    </div>

  <!-- Empty Initial Dashboard Landing -->
  {:else}
    <div class="my-auto py-12 flex flex-col items-center justify-center text-center max-w-3xl mx-auto animate-fade-in">
      <!-- High-End Cyber Radar Emblem -->
      <div class="relative w-20 h-20 rounded-2xl bg-gradient-to-tr from-cyan-500/20 via-cyan-500/10 to-blue-600/20 border border-cyan-500/40 flex items-center justify-center mb-6 text-cyan-400 shadow-2xl shadow-cyan-500/20 group">
        <ShieldCheck class="w-10 h-10 text-cyan-400 relative z-10 transition-transform group-hover:scale-110 duration-300" />
        <div class="absolute -inset-1.5 rounded-2xl bg-cyan-400/15 blur-lg pointer-events-none"></div>
      </div>

      <div class="inline-flex items-center gap-2 px-3.5 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/30 text-cyan-400 text-xs font-mono font-bold mb-4 shadow-sm">
        <span class="w-2 h-2 rounded-full bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)] animate-pulse"></span>
        ENTERPRISE WEB SECURITY POSTURE SUITE
      </div>

      <h1 class="text-4xl font-black text-white tracking-tight sm:text-5xl leading-tight">
        Next-Generation Web Security & Posture Scanner
      </h1>
      <p class="text-sm text-slate-400 mt-4 max-w-xl leading-relaxed">
        Perform real-time passive reconnaissance across HTTP security headers, TLS ciphers, cookie directives, Certificate Transparency subdomains, DNS SPF/DMARC anti-spoofing, and front-end CVE dependencies.
      </p>

      <!-- Center-Stage Instant Scan Input -->
      <div class="w-full max-w-xl mt-8">
        <form
          onsubmit={(e) => {
            e.preventDefault();
            handleScan();
          }}
          class="relative flex items-center gap-2 p-1.5 bg-slate-900/90 border border-slate-800 focus-within:border-cyan-500/80 focus-within:ring-2 focus-within:ring-cyan-500/20 rounded-2xl shadow-2xl shadow-cyan-500/5 transition-all"
        >
          <div class="pl-3.5 text-slate-500">
            <Search class="w-4 h-4" />
          </div>
          <input
            type="text"
            bind:value={targetUrl}
            placeholder="Enter target domain or URL (e.g. example.com)..."
            class="w-full py-2 bg-transparent text-xs font-mono text-white placeholder-slate-500 focus:outline-none"
          />
          <button
            type="submit"
            disabled={!targetUrl.trim() || isScanning}
            class="px-5 py-2.5 bg-gradient-to-r from-cyan-500 to-cyan-400 hover:from-cyan-400 hover:to-cyan-300 disabled:opacity-50 text-slate-950 font-bold text-xs rounded-xl flex items-center gap-1.5 transition-all shadow-lg shadow-cyan-500/20 cursor-pointer disabled:cursor-not-allowed flex-shrink-0"
          >
            <Sparkles class="w-3.5 h-3.5" />
            <span>Launch Audit</span>
          </button>
        </form>
      </div>

      <!-- Quick presets -->
      <div class="mt-6 flex flex-wrap items-center justify-center gap-2">
        <span class="text-xs text-slate-500 font-bold uppercase tracking-wider font-mono mr-1">Preset Targets:</span>
        <button
          type="button"
          onclick={() => {
            targetUrl = "https://example.com";
            handleScan("https://example.com");
          }}
          class="px-3 py-1 bg-slate-900/90 hover:bg-slate-800 border border-slate-800 hover:border-cyan-500/40 rounded-lg text-xs font-mono text-cyan-400 transition-all cursor-pointer shadow-sm"
        >
          example.com
        </button>
        <button
          type="button"
          onclick={() => {
            targetUrl = "http://testphp.vulnweb.com";
            handleScan("http://testphp.vulnweb.com");
          }}
          class="px-3 py-1 bg-slate-900/90 hover:bg-slate-800 border border-slate-800 hover:border-cyan-500/40 rounded-lg text-xs font-mono text-cyan-400 transition-all cursor-pointer shadow-sm"
        >
          testphp.vulnweb.com
        </button>
        <button
          type="button"
          onclick={() => {
            targetUrl = "https://httpbin.org";
            handleScan("https://httpbin.org");
          }}
          class="px-3 py-1 bg-slate-900/90 hover:bg-slate-800 border border-slate-800 hover:border-cyan-500/40 rounded-lg text-xs font-mono text-cyan-400 transition-all cursor-pointer shadow-sm"
        >
          httpbin.org
        </button>
      </div>

      <!-- Feature Badges -->
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mt-12 w-full text-left">
        <div class="p-5 bg-slate-900/40 hover:bg-slate-900/70 border border-slate-800 hover:border-cyan-500/30 rounded-2xl transition-all shadow-sm group">
          <div class="w-9 h-9 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 flex items-center justify-center mb-3 group-hover:scale-105 transition-transform">
            <Globe class="w-4 h-4" />
          </div>
          <div class="text-slate-200 text-xs font-bold uppercase tracking-wider mb-1">Recon & Subdomains</div>
          <p class="text-xs text-slate-400 leading-relaxed">Passive asset mapping via Certificate Transparency logs and SPF/DMARC anti-spoofing checks.</p>
        </div>

        <div class="p-5 bg-slate-900/40 hover:bg-slate-900/70 border border-slate-800 hover:border-cyan-500/30 rounded-2xl transition-all shadow-sm group">
          <div class="w-9 h-9 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 flex items-center justify-center mb-3 group-hover:scale-105 transition-transform">
            <Cpu class="w-4 h-4" />
          </div>
          <div class="text-slate-200 text-xs font-bold uppercase tracking-wider mb-1">Software Composition (SCA)</div>
          <p class="text-xs text-slate-400 leading-relaxed">Identifies outdated front-end JavaScript libraries and maps them to known public CVEs.</p>
        </div>

        <div class="p-5 bg-slate-900/40 hover:bg-slate-900/70 border border-slate-800 hover:border-cyan-500/30 rounded-2xl transition-all shadow-sm group">
          <div class="w-9 h-9 rounded-xl bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 flex items-center justify-center mb-3 group-hover:scale-105 transition-transform">
            <Activity class="w-4 h-4" />
          </div>
          <div class="text-slate-200 text-xs font-bold uppercase tracking-wider mb-1">Continuous Watchdog</div>
          <p class="text-xs text-slate-400 leading-relaxed">Automated scheduled background scanning with desktop alert triggers when risk scores drop.</p>
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

