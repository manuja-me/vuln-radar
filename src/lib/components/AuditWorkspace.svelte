<script lang="ts">
  import type {
    Category,
    ScanOptions,
    ScanReport,
    Severity,
  } from "$lib/types";
  import FindingCard from "./FindingCard.svelte";
  import {
    Check,
    CheckCircle2,
    Database,
    FileDown,
    Search,
    ShieldAlert,
    Sliders,
    Terminal,
    X,
  } from "lucide-svelte";

  let {
    targetUrl = $bindable(""),
    report = null,
    isScanning = false,
    scanOptions = $bindable(),
    onScan,
    onOpenExecutiveReport,
    onOpenExport,
    onOpenSettings,
    onResetBuffer,
  }: {
    targetUrl: string;
    report: ScanReport | null;
    isScanning: boolean;
    scanOptions: ScanOptions;
    onScan: (url?: string) => void;
    onOpenExecutiveReport?: () => void;
    onOpenExport?: () => void;
    onOpenSettings?: () => void;
    onResetBuffer?: () => void;
  } = $props();

  let searchQuery = $state("");
  let selectedSeverity = $state<Severity | "all">("all");
  let selectedCategory = $state<Category | "all">("all");
  let sortFindingsBy = $state<"severity" | "title" | "category">("severity");
  let copiedCurl = $state(false);

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

  async function copyCurlCommand() {
    if (!targetUrl) return;
    const cmd = `curl -I -s -L "${targetUrl}"`;
    try {
      await navigator.clipboard.writeText(cmd);
      copiedCurl = true;
      setTimeout(() => (copiedCurl = false), 1500);
    } catch {}
  }
</script>

<div class="flex flex-col w-full gap-6 max-w-7xl mx-auto pb-14 animate-fade-in flex-1">
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
              if (e.key === "Enter") onScan();
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
          onclick={() => onScan()}
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
          {#if onOpenExecutiveReport}
            <button
              type="button"
              onclick={onOpenExecutiveReport}
              class="w-8 h-8 flex items-center justify-center text-on-surface-variant hover:text-on-surface hover:bg-surface-container rounded transition-colors cursor-pointer"
              title="Executive Audit Report (PDF)"
            >
              <FileDown class="w-3.5 h-3.5" />
            </button>
          {/if}
          {#if onOpenExport}
            <button
              type="button"
              onclick={onOpenExport}
              class="w-8 h-8 flex items-center justify-center text-on-surface-variant hover:text-on-surface hover:bg-surface-container rounded transition-colors cursor-pointer"
              title="Raw JSON Findings Export"
            >
              <Database class="w-3.5 h-3.5" />
            </button>
          {/if}
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
          {#if onResetBuffer}
            <button
              type="button"
              onclick={onResetBuffer}
              class="w-8 h-8 flex items-center justify-center text-on-surface-variant hover:text-error hover:bg-surface-container rounded transition-colors cursor-pointer"
              title="Reset Audit Buffer"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          {/if}
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
            onScan(preset.url);
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
      {#if onOpenSettings}
        <button
          type="button"
          onclick={onOpenSettings}
          class="flex items-center gap-1.5 text-outline hover:text-on-surface transition-colors cursor-pointer text-xs"
        >
          <Sliders class="w-3.5 h-3.5" />
          <span>Advanced Config (⌘O)</span>
        </button>
      {/if}
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
          <div class="p-12 text-center bg-surface-container-low rounded border border-surface-container-high text-outline font-mono">
            <CheckCircle2 class="w-8 h-8 text-tertiary mx-auto mb-2 opacity-80" />
            <p class="font-bold text-on-surface">No findings match active filter.</p>
            <p class="text-xs mt-1 text-outline">Adjust severity or category selectors to view other security telemetry.</p>
          </div>
        {:else}
          {#each filteredFindings as finding (finding.id)}
            <FindingCard {finding} />
          {/each}
        {/if}
      {:else}
        <div class="p-12 text-center bg-surface-container-low rounded border border-surface-container-high text-outline font-mono">
          <ShieldAlert class="w-10 h-10 text-outline mx-auto mb-3 opacity-60" />
          <p class="font-bold text-on-surface">No Active Target Audited</p>
          <p class="text-xs mt-1 text-outline">Enter a target URL above and initiate an audit to view live security findings.</p>
        </div>
      {/if}
    </div>
  </section>
</div>
