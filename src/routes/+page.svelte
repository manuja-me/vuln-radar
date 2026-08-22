<script lang="ts">
  import { onMount } from "svelte";
  import type { ScanReport, ScanSummary, Severity, Category } from "$lib/types";
  import Navbar from "$lib/components/Navbar.svelte";
  import ScoreGauge from "$lib/components/ScoreGauge.svelte";
  import SeverityBadge from "$lib/components/SeverityBadge.svelte";
  import FindingCard from "$lib/components/FindingCard.svelte";
  import HistoryModal from "$lib/components/HistoryModal.svelte";
  import ExportModal from "$lib/components/ExportModal.svelte";
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
  } from "lucide-svelte";

  let targetUrl = $state("");
  let isScanning = $state(false);
  let scanError = $state<string | null>(null);
  let report = $state<ScanReport | null>(null);
  let history = $state<ScanSummary[]>([]);
  let isHistoryOpen = $state(false);
  let isExportOpen = $state(false);
  let exportMarkdown = $state("");

  // Filters
  let searchQuery = $state("");
  let selectedSeverity = $state<Severity | "all">("all");
  let selectedCategory = $state<Category | "all">("all");

  async function invokeTauri<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      return await invoke<T>(cmd, args);
    } catch (e: any) {
      // In browser preview / fallback dev environment
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

  onMount(() => {
    loadHistory();
  });

  async function handleScan(urlToScan?: string) {
    const url = (urlToScan || targetUrl).trim();
    if (!url || isScanning) return;

    isScanning = true;
    scanError = null;
    report = null;

    try {
      const res = await invokeTauri<ScanReport>("scan_target", { url });
      report = res;
      await loadHistory();
    } catch (err: any) {
      scanError = err?.toString() || "Failed to scan target. Please check the URL and internet connection.";
    } finally {
      isScanning = false;
    }
  }

  async function handleSelectFromHistory(scanId: string) {
    isHistoryOpen = false;
    isScanning = true;
    scanError = null;

    try {
      const res = await invokeTauri<ScanReport | null>("get_scan_report", { id: scanId });
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

  async function openExportModal() {
    if (!report) return;
    try {
      exportMarkdown = await invokeTauri<string>("export_report_markdown", { report });
    } catch {
      exportMarkdown = `# Security Report: ${report.target_url}\nScore: ${report.security_score}/100`;
    }
    isExportOpen = true;
  }

  const filteredFindings = $derived.by(() => {
    if (!report) return [];
    return report.findings.filter((finding) => {
      // Filter severity
      if (selectedSeverity !== "all" && finding.severity !== selectedSeverity) {
        return false;
      }
      // Filter category
      if (selectedCategory !== "all" && finding.category !== selectedCategory) {
        return false;
      }
      // Filter query
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
    { id: "vulnerable_dependency", label: "Dependencies (CVEs)" },
    { id: "information_disclosure", label: "Info Leaks" },
    { id: "tls_ssl", label: "TLS / HTTPS" },
    { id: "cors_misconfiguration", label: "CORS" },
    { id: "insecure_form", label: "Forms" },
  ];
</script>

<svelte:head>
  <title>VulnRadar - Web Vulnerability & Security Scanner</title>
</svelte:head>

<!-- Top Navigation & Scan Bar -->
<Navbar
  bind:targetUrl
  {isScanning}
  hasReport={!!report}
  onScan={() => handleScan()}
  onOpenHistory={() => (isHistoryOpen = true)}
  onOpenExport={openExportModal}
/>

<!-- Main Content Area -->
<main class="max-w-7xl w-full mx-auto px-6 py-8 flex-1 flex flex-col">
  <!-- Scanning State -->
  {#if isScanning}
    <div class="my-auto py-24 flex flex-col items-center justify-center text-center animate-pulse">
      <div class="w-16 h-16 rounded-2xl bg-cyan-500/10 border border-cyan-500/30 flex items-center justify-center mb-4 text-cyan-400">
        <Sparkles class="w-8 h-8 animate-spin" />
      </div>
      <h2 class="text-xl font-extrabold text-slate-100">Scanning Target Security Posture</h2>
      <p class="text-xs font-mono text-cyan-400 mt-2 max-w-md truncate">{targetUrl}</p>
      <p class="text-xs text-slate-400 mt-2 max-w-md">
        Inspecting HTTP response headers, cookie flags, client-side JavaScript dependencies, and DOM security indicators...
      </p>
    </div>

  <!-- Scan Error State -->
  {:else if scanError}
    <div class="my-10 p-6 bg-rose-950/30 border border-rose-900/50 rounded-2xl flex items-start gap-4 text-rose-300">
      <AlertOctagon class="w-6 h-6 flex-shrink-0 text-rose-400 mt-0.5" />
      <div>
        <h3 class="text-base font-bold text-rose-200">Scan Failed</h3>
        <p class="text-xs text-rose-300/90 mt-1 font-mono">{scanError}</p>
        <button
          type="button"
          onclick={() => handleScan()}
          class="mt-4 px-4 py-2 bg-rose-500/20 hover:bg-rose-500/30 text-rose-200 border border-rose-500/40 rounded-xl text-xs font-bold transition-colors cursor-pointer"
        >
          Try Again
        </button>
      </div>
    </div>

  <!-- Active Report View -->
  {:else if report}
    <div class="space-y-6">
      <!-- Target Summary Header Card -->
      <div class="p-6 bg-slate-900/60 border border-slate-800 rounded-2xl flex flex-col lg:flex-row items-start lg:items-center justify-between gap-6">
        <div class="space-y-2 min-w-0">
          <div class="flex items-center gap-2">
            <span class="px-2.5 py-0.5 text-xs font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 rounded-md">
              HTTP {report.status_code}
            </span>
            <span class="text-xs text-slate-400 font-mono">
              Latency: {report.response_time_ms} ms
            </span>
            <span class="text-xs text-slate-500">•</span>
            <span class="text-xs text-slate-400 font-mono">
              {new Date(report.scanned_at).toLocaleTimeString()}
            </span>
          </div>

          <h1 class="text-2xl font-black text-slate-100 truncate flex items-center gap-2">
            <Globe class="w-6 h-6 text-cyan-400 flex-shrink-0" />
            <span class="truncate">{report.target_url}</span>
          </h1>

          <!-- Detected Tech Stack Tags -->
          {#if report.technologies_detected.length > 0}
            <div class="flex flex-wrap items-center gap-1.5 pt-1">
              <span class="text-xs text-slate-400 font-semibold flex items-center gap-1 mr-1">
                <Cpu class="w-3.5 h-3.5 text-slate-500" /> Stack:
              </span>
              {#each report.technologies_detected as tech}
                <span class="px-2 py-0.5 text-xs font-mono bg-slate-800/80 text-slate-300 border border-slate-700/60 rounded">
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

      <!-- Finding Metrics Grid -->
      <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
        <div class="p-4 bg-slate-900/40 border border-slate-800 rounded-xl flex flex-col">
          <span class="text-xs font-bold text-slate-400 uppercase tracking-wider">Total Findings</span>
          <span class="text-2xl font-black text-slate-100 mt-1 font-mono">{report.total_findings}</span>
        </div>
        <div class="p-4 bg-rose-950/20 border border-rose-900/30 rounded-xl flex flex-col">
          <span class="text-xs font-bold text-rose-400 uppercase tracking-wider">Critical</span>
          <span class="text-2xl font-black text-rose-400 mt-1 font-mono">{report.critical_count}</span>
        </div>
        <div class="p-4 bg-orange-950/20 border border-orange-900/30 rounded-xl flex flex-col">
          <span class="text-xs font-bold text-orange-400 uppercase tracking-wider">High</span>
          <span class="text-2xl font-black text-orange-400 mt-1 font-mono">{report.high_count}</span>
        </div>
        <div class="p-4 bg-amber-950/20 border border-amber-900/30 rounded-xl flex flex-col">
          <span class="text-xs font-bold text-amber-400 uppercase tracking-wider">Medium</span>
          <span class="text-2xl font-black text-amber-400 mt-1 font-mono">{report.medium_count}</span>
        </div>
        <div class="p-4 bg-blue-950/20 border border-blue-900/30 rounded-xl flex flex-col">
          <span class="text-xs font-bold text-blue-400 uppercase tracking-wider">Low</span>
          <span class="text-2xl font-black text-blue-400 mt-1 font-mono">{report.low_count}</span>
        </div>
        <div class="p-4 bg-slate-900/40 border border-slate-800 rounded-xl flex flex-col">
          <span class="text-xs font-bold text-slate-400 uppercase tracking-wider">Info</span>
          <span class="text-2xl font-black text-slate-400 mt-1 font-mono">{report.info_count}</span>
        </div>
      </div>

      <!-- Findings Filter & Search Bar -->
      <div class="p-4 bg-slate-900/40 border border-slate-800 rounded-2xl flex flex-col md:flex-row items-center justify-between gap-4">
        <!-- Search -->
        <div class="relative w-full md:w-80">
          <Search class="w-4 h-4 text-slate-500 absolute inset-y-0 left-3 my-auto pointer-events-none" />
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="Search findings by keyword, CVE, or OWASP..."
            class="w-full pl-9 pr-3 py-2 bg-slate-950/80 border border-slate-800 focus:border-cyan-500 rounded-xl text-xs text-slate-200 placeholder-slate-500"
          />
        </div>

        <!-- Severity Filter Buttons -->
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
          <h2 class="text-sm font-bold uppercase tracking-wider text-slate-400">
            Detected Vulnerabilities & Weaknesses ({filteredFindings.length})
          </h2>
          {#if selectedSeverity !== "all" || searchQuery.trim()}
            <button
              type="button"
              onclick={() => {
                selectedSeverity = "all";
                searchQuery = "";
              }}
              class="text-xs text-cyan-400 hover:underline cursor-pointer"
            >
              Reset Filters
            </button>
          {/if}
        </div>

        {#if filteredFindings.length === 0}
          <div class="py-16 text-center bg-slate-900/30 border border-slate-800 rounded-2xl">
            <CheckCircle2 class="w-12 h-12 text-emerald-400 mx-auto mb-3 opacity-80" />
            <h3 class="text-base font-bold text-slate-200">No matching findings</h3>
            <p class="text-xs text-slate-400 mt-1">No vulnerabilities match the current search or severity filter.</p>
          </div>
        {:else}
          {#each filteredFindings as finding (finding.id)}
            <FindingCard {finding} />
          {/each}
        {/if}
      </div>
    </div>

  <!-- Empty Initial Dashboard Landing -->
  {:else}
    <div class="my-auto py-16 flex flex-col items-center justify-center text-center max-w-2xl mx-auto">
      <div class="w-16 h-16 rounded-2xl bg-cyan-500/10 border border-cyan-500/30 flex items-center justify-center mb-6 text-cyan-400 shadow-xl shadow-cyan-500/10">
        <ShieldCheck class="w-8 h-8" />
      </div>

      <h1 class="text-3xl font-black text-white tracking-tight">
        Instant Web Vulnerability Scanner
      </h1>
      <p class="text-sm text-slate-400 mt-3 max-w-lg leading-relaxed">
        Enter any website URL above to inspect its live HTTP headers, SSL/TLS configuration, cookie policies, exposed sensitive data, and client-side CVEs.
      </p>

      <!-- Quick targets -->
      <div class="mt-8 flex flex-wrap items-center justify-center gap-2">
        <span class="text-xs text-slate-500 font-semibold mr-1">Quick Presets:</span>
        <button
          type="button"
          onclick={() => {
            targetUrl = "https://example.com";
            handleScan("https://example.com");
          }}
          class="px-3 py-1.5 bg-slate-900 hover:bg-slate-800 border border-slate-800 hover:border-cyan-500/30 rounded-xl text-xs font-mono text-cyan-400 transition-colors cursor-pointer"
        >
          example.com
        </button>
        <button
          type="button"
          onclick={() => {
            targetUrl = "http://testphp.vulnweb.com";
            handleScan("http://testphp.vulnweb.com");
          }}
          class="px-3 py-1.5 bg-slate-900 hover:bg-slate-800 border border-slate-800 hover:border-cyan-500/30 rounded-xl text-xs font-mono text-cyan-400 transition-colors cursor-pointer"
        >
          testphp.vulnweb.com
        </button>
        <button
          type="button"
          onclick={() => {
            targetUrl = "https://httpbin.org";
            handleScan("https://httpbin.org");
          }}
          class="px-3 py-1.5 bg-slate-900 hover:bg-slate-800 border border-slate-800 hover:border-cyan-500/30 rounded-xl text-xs font-mono text-cyan-400 transition-colors cursor-pointer"
        >
          httpbin.org
        </button>
      </div>

      <!-- Feature Badges -->
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mt-12 w-full text-left">
        <div class="p-4 bg-slate-900/40 border border-slate-800/80 rounded-xl">
          <div class="text-cyan-400 text-xs font-bold uppercase tracking-wider mb-1">Passive & Safe</div>
          <p class="text-xs text-slate-400">Non-intrusive header, cookie, and DOM inspection without triggering intrusion alarms.</p>
        </div>
        <div class="p-4 bg-slate-900/40 border border-slate-800/80 rounded-xl">
          <div class="text-cyan-400 text-xs font-bold uppercase tracking-wider mb-1">CVE Audit (SCA)</div>
          <p class="text-xs text-slate-400">Detects outdated JavaScript libraries (jQuery, Angular, Lodash, Bootstrap) with known CVEs.</p>
        </div>
        <div class="p-4 bg-slate-900/40 border border-slate-800/80 rounded-xl">
          <div class="text-cyan-400 text-xs font-bold uppercase tracking-wider mb-1">OWASP Mapped</div>
          <p class="text-xs text-slate-400">Aligns findings with OWASP Top 10 classifications and actionable remediation steps.</p>
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
