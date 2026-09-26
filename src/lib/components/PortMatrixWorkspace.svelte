<script lang="ts">
  import type { PortScanReport, OpenPort, ScanOptions } from "$lib/types";
  import {
    Terminal,
    Copy,
    Check,
    Radar,
    ExternalLink,
    ShieldAlert,
    ShieldCheck,
    Loader2,
    AlertOctagon,
  } from "lucide-svelte";

  let {
    targetUrl = "",
    portReport = null,
    options,
    onOptionsChange,
    onUpdateReport,
  }: {
    targetUrl: string;
    portReport: PortScanReport | null;
    options?: ScanOptions;
    onOptionsChange?: (newOptions: ScanOptions) => void;
    onUpdateReport?: (newReport: PortScanReport) => void;
  } = $props();

  let localReport = $state<PortScanReport | null>(null);
  let isScanning = $state(false);
  let scanError = $state<string | null>(null);

  $effect(() => {
    if (portReport) {
      localReport = portReport;
    }
  });

  const activeReport = $derived(localReport || portReport);

  let selectedProfile = $state<"top20" | "top100" | "databases" | "custom">("top20");
  let customPortSpec = $state(
    "1-1024, 3000, 3306, 5432, 6379, 8000, 8080, 8443, 9200, 27017"
  );
  let timeoutMs = $state(600);

  $effect(() => {
    if (options) {
      if (options.port_scan_profile && options.port_scan_profile !== selectedProfile) {
        selectedProfile = options.port_scan_profile as any;
      }
      if (options.custom_ports !== undefined && options.custom_ports !== customPortSpec) {
        customPortSpec = options.custom_ports;
      }
      if (options.port_timeout_ms !== undefined && options.port_timeout_ms !== timeoutMs) {
        timeoutMs = options.port_timeout_ms;
      }
    }
  });

  function notifyOptionsChange(profile: "top20" | "top100" | "databases" | "custom", customPorts: string, timeout: number) {
    if (onOptionsChange) {
      onOptionsChange({
        ...(options || {}),
        enable_port_scan: true,
        port_scan_profile: profile,
        custom_ports: profile === "custom" ? customPorts.trim() : undefined,
        port_timeout_ms: timeout,
      });
    }
  }

  let portFilter = $state<"all" | "critical" | "verified" | "filtered">("all");
  let copiedPayload = $state(false);
  let copiedAddress = $state(false);

  // Derive target hostname
  const defaultHost = $derived.by(() => {
    if (activeReport?.host) return activeReport.host;
    try {
      const parsed = new URL(targetUrl.startsWith("http") ? targetUrl : `https://${targetUrl}`);
      return parsed.hostname;
    } catch {
      return targetUrl.replace(/https?:\/\//, "").split("/")[0] || "localhost";
    }
  });

  let hostInput = $state("");
  $effect(() => {
    if (!hostInput && defaultHost) {
      hostInput = defaultHost;
    }
  });

  const targetHost = $derived(hostInput.trim() || defaultHost);

  // Selected port for the inspector drawer
  let selectedPortNum = $state<number | null>(null);

  const displayedPorts = $derived.by(() => {
    return activeReport?.open_ports || [];
  });

  // Filtered port list
  const filteredPorts = $derived.by(() => {
    return displayedPorts.filter((p) => {
      if (portFilter === "critical" && !p.is_risky) return false;
      if (portFilter === "verified" && (p.is_risky || p.state === "filtered")) return false;
      if (portFilter === "filtered" && p.state !== "filtered") return false;
      return true;
    });
  });

  // Active selected port object
  const activePort = $derived.by(() => {
    if (selectedPortNum !== null) {
      const match = displayedPorts.find((p) => p.port === selectedPortNum);
      if (match) return match;
    }
    return displayedPorts.find((p) => p.is_risky) || displayedPorts[0] || null;
  });

  async function runPortScan() {
    const host = targetHost.trim();
    if (isScanning || !host) return;

    isScanning = true;
    scanError = null;

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const res = await invoke<PortScanReport>("scan_ports", {
        host,
        profile: selectedProfile,
        customPorts: selectedProfile === "custom" ? customPortSpec.trim() : null,
        timeoutMs: Number(timeoutMs) || 600,
      });
      localReport = res;
      if (onUpdateReport) {
        onUpdateReport(res);
      }
      if (res.open_ports.length > 0) {
        selectedPortNum = res.open_ports[0].port;
      }
    } catch (e: any) {
      scanError = typeof e === "string" ? e : (e?.message || "Port scan failed.");
    } finally {
      isScanning = false;
    }
  }

  async function copyPortAddress(host: string, port: number) {
    try {
      await navigator.clipboard.writeText(`${host}:${port}`);
      copiedAddress = true;
      setTimeout(() => (copiedAddress = false), 1800);
    } catch {}
  }

  async function copyRawPayload() {
    if (!activePort) return;
    const payload = activePort.banner
      ? `${targetHost}:${activePort.port} (${activePort.service})\n\nBanner:\n${activePort.banner}`
      : `${targetHost}:${activePort.port} (${activePort.service} - ${activePort.protocol}) state=${activePort.state}`;
    try {
      await navigator.clipboard.writeText(payload);
      copiedPayload = true;
      setTimeout(() => (copiedPayload = false), 1800);
    } catch {}
  }
</script>

<div class="flex flex-col w-full gap-4 max-w-7xl mx-auto pb-10">
  <!-- Ribbon Controls Header -->
  <section class="flex flex-col bg-surface-container rounded-lg p-3 sm:p-4 gap-3 border border-surface-container-high shadow-sm">
    <div class="flex flex-wrap items-center gap-2">
      <!-- Target Host Input -->
      <div class="h-9 flex items-center bg-surface-container-lowest px-3 rounded border border-surface-container-high flex-1 min-w-[200px]">
        <input
          type="text"
          bind:value={hostInput}
          placeholder="Target Hostname or IP"
          onkeydown={(e) => {
            if (e.key === "Enter") runPortScan();
          }}
          class="bg-transparent font-mono text-xs text-on-surface outline-none w-full"
        />
      </div>

      {#if activeReport?.ip_address}
        <span class="h-9 flex items-center px-2.5 rounded font-mono text-xs bg-surface-container-lowest text-secondary font-medium border border-surface-container-high">
          {activeReport.ip_address}
        </span>
      {/if}

      <!-- Profile Selector Buttons -->
      <div class="flex items-center gap-1 bg-surface-container-lowest p-0.5 rounded border border-surface-container-high">
        {#each [
          { id: "top20", label: "Top 20" },
          { id: "top100", label: "Top 100" },
          { id: "databases", label: "Databases" },
          { id: "custom", label: "Custom" }
        ] as prof}
          <button
            type="button"
            onclick={() => {
              selectedProfile = prof.id as any;
              notifyOptionsChange(prof.id as any, customPortSpec, timeoutMs);
            }}
            class="h-8 px-2.5 font-mono text-xs rounded transition-colors cursor-pointer {selectedProfile === prof.id ? 'bg-primary text-on-primary font-bold' : 'text-on-surface-variant hover:text-on-surface'}"
          >
            {prof.label}
          </button>
        {/each}
      </div>

      <!-- Timeout Selector -->
      <div class="h-9 flex items-center gap-1.5 px-2.5 bg-surface-container-lowest rounded border border-surface-container-high font-mono text-xs text-on-surface-variant">
        <span class="text-outline">Timeout:</span>
        <select
          bind:value={timeoutMs}
          onchange={() => notifyOptionsChange(selectedProfile, customPortSpec, Number(timeoutMs) || 600)}
          class="bg-transparent font-mono text-xs text-on-surface outline-none cursor-pointer"
        >
          <option value={300}>300ms</option>
          <option value={600}>600ms</option>
          <option value={1000}>1000ms</option>
          <option value={2000}>2000ms</option>
        </select>
      </div>

      <!-- Scan Action Button -->
      <button
        type="button"
        onclick={runPortScan}
        disabled={isScanning || !targetHost}
        class="h-9 flex items-center gap-2 px-4 bg-primary text-on-primary font-mono text-xs font-bold rounded shadow-sm hover:opacity-90 active:scale-95 transition-all cursor-pointer disabled:opacity-50 whitespace-nowrap"
      >
        {#if isScanning}
          <Loader2 class="w-3.5 h-3.5 animate-spin" />
          <span>Scanning...</span>
        {:else}
          <Radar class="w-3.5 h-3.5" />
          <span>Scan Ports</span>
        {/if}
      </button>
    </div>

    <!-- Custom Port Spec Field (Only shown when custom profile is active) -->
    {#if selectedProfile === "custom"}
      <div class="h-9 flex items-center bg-surface-container-lowest px-3 rounded gap-2 border border-surface-container-high">
        <span class="font-mono text-xs text-outline uppercase font-semibold whitespace-nowrap">Ports:</span>
        <input
          class="bg-transparent font-mono text-xs text-on-surface w-full focus:outline-none"
          type="text"
          placeholder="e.g. 80, 443, 8080-8090, 3000, 5432"
          bind:value={customPortSpec}
          oninput={() => notifyOptionsChange(selectedProfile, customPortSpec, timeoutMs)}
        />
      </div>
    {/if}

    <!-- Error Banner -->
    {#if scanError}
      <div class="p-3 bg-surface-container-lowest border border-error/40 rounded flex items-center justify-between text-xs text-error font-mono">
        <div class="flex items-center gap-2">
          <AlertOctagon class="w-4 h-4 flex-shrink-0" />
          <span>{scanError}</span>
        </div>
        <button
          type="button"
          onclick={runPortScan}
          class="px-2 py-0.5 bg-error text-surface-container-lowest font-bold rounded cursor-pointer uppercase text-[10px]"
        >
          Retry
        </button>
      </div>
    {/if}

    <!-- Live Scanned Metrics Bar (Only shown when scanning or results available) -->
    {#if isScanning || activeReport}
      <div class="flex flex-wrap items-center justify-between gap-3 pt-2 border-t border-surface-container-high font-mono text-xs">
        <div class="flex items-center gap-4 flex-wrap">
          <span class="text-on-surface-variant">Scanned: <strong class="text-on-surface">{activeReport?.scanned_ports_count ?? 0} ports</strong></span>
          <span class="text-tertiary">Open: <strong>{displayedPorts.filter(p => p.state === 'open').length}</strong></span>
          <span class="text-secondary">Filtered: <strong>{displayedPorts.filter(p => p.state === 'filtered').length}</strong></span>
          <span class="text-outline">Closed: <strong>{Math.max(0, (activeReport?.scanned_ports_count ?? 0) - displayedPorts.length)}</strong></span>
          {#if activeReport?.scan_duration_ms != null}
            <span class="text-outline">Duration: <strong class="text-on-surface-variant">{activeReport.scan_duration_ms}ms</strong></span>
          {/if}
        </div>
        {#if activeReport && activeReport.scanned_ports_count > 0}
          {@const openPct = Math.round((displayedPorts.filter(p => p.state === 'open').length / activeReport.scanned_ports_count) * 100)}
          {@const filteredPct = Math.round((displayedPorts.filter(p => p.state === 'filtered').length / activeReport.scanned_ports_count) * 100)}
          <div class="flex items-center gap-2 w-full sm:w-64">
            <div class="h-1.5 w-full bg-surface-container-high rounded-full overflow-hidden flex">
              <div class="h-full bg-error" style="width: {openPct}%;"></div>
              <div class="h-full bg-secondary" style="width: {filteredPct}%;"></div>
              <div class="h-full bg-outline" style="width: {Math.max(0, 100 - openPct - filteredPct)}%;"></div>
            </div>
            <span class="text-[11px] text-tertiary whitespace-nowrap">100%</span>
          </div>
        {/if}
      </div>
    {/if}
  </section>

  <!-- Split Grid: Port Results & Inspector -->
  <div class="grid grid-cols-1 lg:grid-cols-12 gap-4">
    <!-- Left: Port Cards (8 cols) -->
    <section class="lg:col-span-8 flex flex-col gap-3">
      <!-- Section Header & Filter Toolbar -->
      <div class="flex items-center justify-between px-1 gap-2">
        <h2 class="text-sm font-semibold text-on-surface">Port Matrix</h2>
        {#if displayedPorts.length > 0}
          <div class="flex items-center gap-1 font-mono text-xs">
            <button
              type="button"
              onclick={() => (portFilter = "all")}
              class="px-2 py-0.5 rounded cursor-pointer transition-colors {portFilter === 'all' ? 'bg-surface-container-high text-on-surface font-bold' : 'text-outline hover:text-on-surface'}"
            >
              All ({displayedPorts.length})
            </button>
            <button
              type="button"
              onclick={() => (portFilter = "critical")}
              class="px-2 py-0.5 rounded cursor-pointer transition-colors flex items-center gap-1 {portFilter === 'critical' ? 'bg-error/20 text-error font-bold' : 'text-outline hover:text-error'}"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-error"></span> Critical ({displayedPorts.filter(p => p.is_risky).length})
            </button>
            <button
              type="button"
              onclick={() => (portFilter = "verified")}
              class="px-2 py-0.5 rounded cursor-pointer transition-colors flex items-center gap-1 {portFilter === 'verified' ? 'bg-tertiary/20 text-tertiary font-bold' : 'text-outline hover:text-tertiary'}"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-tertiary"></span> Verified ({displayedPorts.filter(p => !p.is_risky && p.state !== 'filtered').length})
            </button>
            <button
              type="button"
              onclick={() => (portFilter = "filtered")}
              class="px-2 py-0.5 rounded cursor-pointer transition-colors flex items-center gap-1 {portFilter === 'filtered' ? 'bg-secondary/20 text-secondary font-bold' : 'text-outline hover:text-secondary'}"
            >
              <span class="w-1.5 h-1.5 rounded-full bg-secondary"></span> Filtered ({displayedPorts.filter(p => p.state === 'filtered').length})
            </button>
          </div>
        {/if}
      </div>

      <!-- Empty / Standby / Results Grid -->
      {#if filteredPorts.length === 0}
        <div class="p-10 text-center bg-surface-container rounded-lg border border-surface-container-high text-outline flex flex-col items-center justify-center gap-2">
          {#if isScanning}
            <Loader2 class="w-6 h-6 text-primary animate-spin" />
            <span class="font-medium text-sm text-on-surface">Probing TCP ports on {targetHost}...</span>
          {:else if activeReport}
            <ShieldCheck class="w-7 h-7 text-tertiary opacity-80" />
            <span class="font-medium text-sm text-on-surface">No open ports detected</span>
            <span class="text-xs">All {activeReport.scanned_ports_count} probed ports returned closed or filtered.</span>
          {:else}
            <Radar class="w-7 h-7 opacity-30 mb-0.5" />
            <span class="font-medium text-sm text-on-surface">No ports scanned yet</span>
            <span class="text-xs">Click "Scan Ports" to inspect active TCP services on {targetHost}.</span>
          {/if}
        </div>
      {:else}
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          {#each filteredPorts as port (port.port)}
            <div
              role="button"
              tabindex="0"
              onclick={() => (selectedPortNum = port.port)}
              onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") selectedPortNum = port.port; }}
              class="flex flex-col justify-between p-3.5 bg-surface-container rounded-lg hover:bg-surface-bright transition-colors cursor-pointer border border-surface-container-high {activePort?.port === port.port ? 'ring-1 ring-primary' : ''}"
            >
              <div class="flex items-start justify-between gap-2">
                <div class="flex flex-col">
                  <div class="flex items-center gap-2">
                    <span class="font-mono text-sm font-semibold {port.is_risky ? 'text-error' : 'text-secondary'}">
                      {port.service}:{port.port}
                    </span>
                    <span class="font-mono text-[10px] bg-surface-container-lowest text-outline px-1.5 py-0.5 rounded uppercase">
                      {port.protocol}
                    </span>
                  </div>
                  {#if port.banner || port.description}
                    <span class="text-xs text-on-surface-variant mt-1 line-clamp-1">
                      {port.banner || port.description}
                    </span>
                  {/if}
                </div>
                <span class="px-1.5 py-0.5 text-[10px] rounded uppercase font-semibold font-mono flex items-center gap-1 {port.is_risky ? 'bg-error/20 text-error' : 'bg-tertiary/20 text-tertiary'}">
                  {#if port.is_risky}
                    <ShieldAlert class="w-3 h-3" />
                    <span>RISK</span>
                  {:else}
                    <ShieldCheck class="w-3 h-3" />
                    <span>OPEN</span>
                  {/if}
                </span>
              </div>

              <div class="flex items-center justify-between mt-3 pt-2 border-t border-surface-container-high text-xs">
                <span class="font-mono text-[11px] text-outline uppercase">
                  State: {port.state}
                </span>
                <span class="text-[11px] text-primary hover:underline font-medium">
                  Inspect &rarr;
                </span>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- Right: Port Detail Drawer (4 cols) -->
    <section class="lg:col-span-4 flex flex-col gap-3">
      <div class="flex items-center justify-between px-1">
        <div class="flex items-center gap-2">
          <Terminal class="w-4 h-4 text-secondary" />
          <h2 class="text-sm font-semibold text-on-surface">
            Port Inspector {activePort ? `[${activePort.port}]` : ""}
          </h2>
        </div>
        {#if activePort}
          <span class="font-mono text-[10px] px-1.5 py-0.5 rounded font-medium {activePort.is_risky ? 'text-error bg-error/10' : 'text-tertiary bg-tertiary/10'}">
            {activePort.is_risky ? "EXPOSED" : "NORMAL"}
          </span>
        {/if}
      </div>

      {#if !activePort}
        <div class="bg-surface-container rounded-lg p-8 text-center text-outline border border-surface-container-high flex flex-col items-center justify-center gap-2">
          <Terminal class="w-7 h-7 opacity-30 mb-0.5" />
          <span class="font-medium text-sm text-on-surface">No Port Selected</span>
          <span class="text-xs">Select any discovered port to view its service details and response banner.</span>
        </div>
      {:else}
        <div class="bg-surface-container rounded-lg p-3.5 flex flex-col gap-3 border border-surface-container-high shadow-sm">
          <!-- Port Meta -->
          <div class="bg-surface-container-low p-3 rounded flex flex-col gap-1 border border-surface-container-high">
            <span class="text-[10px] text-outline uppercase font-mono font-semibold">Service Details</span>
            <div class="font-mono text-sm text-on-surface font-bold">
              {activePort.service} (Port {activePort.port} / {activePort.protocol})
            </div>
            <div class="flex items-center gap-3 font-mono text-xs text-on-surface-variant">
              <span>State: {activePort.state.toUpperCase()}</span>
              <span class="{activePort.is_risky ? 'text-error font-medium' : 'text-tertiary'}">
                {activePort.is_risky ? 'Review exposure' : 'Standard service'}
              </span>
            </div>
          </div>

          <!-- Description -->
          {#if activePort.description}
            <div class="bg-surface-container-lowest p-3 rounded flex flex-col gap-1 border border-surface-container-high">
              <span class="text-[10px] text-outline uppercase font-mono font-semibold">Description</span>
              <p class="text-xs text-on-surface-variant leading-relaxed">
                {activePort.description}
              </p>
            </div>
          {/if}

          <!-- Banner / Response -->
          <div class="flex flex-col gap-1.5">
            <div class="flex items-center justify-between">
              <span class="text-[10px] text-outline uppercase font-mono font-semibold">Response Banner</span>
              <button
                type="button"
                onclick={copyRawPayload}
                class="font-mono text-[11px] text-secondary hover:underline cursor-pointer flex items-center gap-1"
              >
                {#if copiedPayload}
                  <Check class="w-3 h-3 text-tertiary" />
                  <span class="text-tertiary">Copied</span>
                {:else}
                  <Copy class="w-3 h-3" />
                  <span>Copy</span>
                {/if}
              </button>
            </div>
            <div class="bg-surface-container-lowest p-3 rounded font-mono text-xs text-on-surface overflow-x-auto max-h-48 border border-surface-container-high">
              {#if activePort.banner}
                <div class="whitespace-pre-wrap break-all leading-relaxed">
                  {activePort.banner}
                </div>
              {:else}
                <span class="text-outline italic">No application banner emitted during handshake.</span>
              {/if}
            </div>
          </div>

          <!-- Actions -->
          <div class="flex items-center gap-2 pt-1">
            <button
              type="button"
              onclick={() => copyPortAddress(targetHost, activePort.port)}
              class="h-9 flex-1 bg-surface-container-high hover:bg-surface-bright text-on-surface font-mono text-xs rounded transition-colors cursor-pointer flex items-center justify-center gap-1.5 border border-surface-container-high"
            >
              {#if copiedAddress}
                <Check class="w-3.5 h-3.5 text-tertiary" />
                <span>Copied</span>
              {:else}
                <Copy class="w-3.5 h-3.5" />
                <span>Copy Socket</span>
              {/if}
            </button>
            {#if [80, 443, 3000, 5000, 8000, 8080, 8443].includes(activePort.port)}
              <a
                href={`${activePort.port === 443 || activePort.port === 8443 ? "https" : "http"}://${targetHost}:${activePort.port}`}
                target="_blank"
                rel="noopener noreferrer"
                class="h-9 px-3 bg-secondary text-surface-container-lowest font-mono text-xs rounded font-medium hover:opacity-90 flex items-center gap-1.5 whitespace-nowrap"
              >
                <ExternalLink class="w-3.5 h-3.5" />
                <span>Open URL</span>
              </a>
            {/if}
          </div>
        </div>
      {/if}
    </section>
  </div>
</div>
