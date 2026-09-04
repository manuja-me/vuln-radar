<script lang="ts">
  import type { ScanReport } from "$lib/types";
  import {
    X,
    Printer,
    Shield,
    Globe,
    AlertOctagon,
    AlertTriangle,
    Info,
    CheckCircle2,
    Lock,
    Mail,
    Server,
    ExternalLink,
    Clock,
  } from "lucide-svelte";
  import ScoreGauge from "./ScoreGauge.svelte";
  import SeverityBadge from "./SeverityBadge.svelte";

  let {
    isOpen = false,
    report = null,
    onClose,
  }: {
    isOpen: boolean;
    report: ScanReport | null;
    onClose: () => void;
  } = $props();

  function triggerPrint() {
    window.print();
  }

  function formatDate(iso: string) {
    try {
      return new Date(iso).toLocaleString();
    } catch {
      return iso;
    }
  }
</script>

{#if isOpen && report}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-xs p-4 print:p-0 print:bg-white print:static animate-fade-in overflow-y-auto"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
    onkeydown={(e) => {
      if (e.key === "Escape") onClose();
    }}
    tabindex="-1"
    role="dialog"
    aria-modal="true"
  >
    <div class="bg-[var(--color-surface)] border border-[var(--color-hairline-strong)] print:border-none print:bg-white print:text-black rounded-none w-full max-w-4xl max-h-[90vh] print:max-h-none flex flex-col shadow-2xl overflow-hidden print:overflow-visible my-auto text-[var(--color-text-body)]">
      
      <!-- Top Action Bar (hidden when printing) -->
      <div class="p-3.5 border-b border-[var(--color-hairline)] flex items-center justify-between bg-[var(--color-canvas)] print:hidden">
        <div class="flex items-center gap-2.5">
          <div class="w-7 h-7 rounded-none bg-[var(--color-surface)] border border-[var(--color-hairline)] flex items-center justify-center text-[var(--color-text-headline)]">
            <Shield class="w-3.5 h-3.5" />
          </div>
          <div>
            <div class="text-[10px] font-mono uppercase tracking-widest text-[var(--color-signal-red)]">08 / EXECUTIVE</div>
            <h2 class="text-xs font-mono font-bold text-[var(--color-text-headline)] uppercase tracking-wider">Security Assessment Report</h2>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={triggerPrint}
            class="px-3.5 py-1.5 bg-[var(--color-signal-red)] hover:bg-[var(--color-signal-red-hover)] text-white font-mono uppercase tracking-widest text-xs font-bold rounded-none flex items-center gap-1.5 transition-colors cursor-pointer shadow-sm"
          >
            <Printer class="w-3.5 h-3.5" />
            <span>Print / Save PDF</span>
          </button>
          <button
            type="button"
            onclick={onClose}
            class="p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] hover:bg-[var(--color-surface)] border border-transparent hover:border-[var(--color-hairline)] rounded-none transition-colors cursor-pointer"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Printable Report Container -->
      <div class="flex-1 overflow-y-auto print:overflow-visible p-6 md:p-8 space-y-6 text-[var(--color-text-body)] print:text-neutral-900 print:bg-white">
        
        <!-- Header / Organization Branding -->
        <div class="border-b border-[var(--color-hairline)] print:border-neutral-300 pb-5 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <div>
            <div class="flex items-center gap-2">
              <span class="text-[var(--color-text-headline)] print:text-neutral-900 font-mono font-bold text-lg tracking-tight uppercase">VulnRadar</span>
              <span class="px-1.5 py-0.5 text-[10px] font-mono bg-[var(--color-canvas)] print:bg-neutral-100 text-[var(--color-text-muted)] print:text-neutral-700 rounded-none border border-[var(--color-hairline)] print:border-neutral-300 uppercase tracking-widest">v0.7.0</span>
            </div>
            <h1 class="text-xl font-mono font-bold text-[var(--color-text-headline)] print:text-black mt-1 uppercase tracking-tight">Web Security Posture Audit</h1>
            <p class="text-xs text-[var(--color-text-muted)] print:text-neutral-600 font-mono mt-0.5">TARGET: {report.target_url}</p>
          </div>

          <div class="text-left sm:text-right text-xs font-mono text-[var(--color-text-muted)] print:text-neutral-600 space-y-0.5">
            <div><strong>SCAN ID:</strong> {report.id}</div>
            <div><strong>GENERATED:</strong> {formatDate(report.scanned_at)}</div>
            <div><strong>LATENCY:</strong> {report.response_time_ms} ms</div>
          </div>
        </div>

        <!-- Executive Summary Cards -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
          <!-- Score Gauge Card -->
          <div class="p-4 bg-[var(--color-canvas)] print:bg-neutral-50 border border-[var(--color-hairline)] print:border-neutral-200 rounded-none flex items-center justify-center">
            <ScoreGauge score={report.security_score} />
          </div>

          <!-- Finding Breakdown Stats -->
          <div class="p-4 bg-[var(--color-canvas)] print:bg-neutral-50 border border-[var(--color-hairline)] print:border-neutral-200 rounded-none flex flex-col justify-center space-y-2 col-span-1 md:col-span-2">
            <h3 class="text-[10px] font-mono font-bold uppercase tracking-widest text-[var(--color-text-muted)] print:text-neutral-600">Risk Severity Breakdown</h3>
            <div class="grid grid-cols-5 gap-2 text-center pt-1">
              <div class="p-2 bg-[var(--color-signal-red)]/10 border border-[var(--color-signal-red)]/30 rounded-none">
                <span class="text-xs font-bold text-[var(--color-signal-red)] block font-mono">{report.critical_count}</span>
                <span class="text-[10px] text-[var(--color-signal-red)] font-mono uppercase">Critical</span>
              </div>
              <div class="p-2 bg-[var(--color-signal-orange)]/10 border border-[var(--color-signal-orange)]/30 rounded-none">
                <span class="text-xs font-bold text-[var(--color-signal-orange)] block font-mono">{report.high_count}</span>
                <span class="text-[10px] text-[var(--color-signal-orange)] font-mono uppercase">High</span>
              </div>
              <div class="p-2 bg-[var(--color-signal-amber)]/10 border border-[var(--color-signal-amber)]/30 rounded-none">
                <span class="text-xs font-bold text-[var(--color-signal-amber)] block font-mono">{report.medium_count}</span>
                <span class="text-[10px] text-[var(--color-signal-amber)] font-mono uppercase">Medium</span>
              </div>
              <div class="p-2 bg-[var(--color-signal-blue)]/10 border border-[var(--color-signal-blue)]/30 rounded-none">
                <span class="text-xs font-bold text-[var(--color-signal-blue)] block font-mono">{report.low_count}</span>
                <span class="text-[10px] text-[var(--color-signal-blue)] font-mono uppercase">Low</span>
              </div>
              <div class="p-2 bg-[var(--color-surface)] border border-[var(--color-hairline)] rounded-none">
                <span class="text-xs font-bold text-[var(--color-text-headline)] print:text-neutral-700 block font-mono">{report.info_count}</span>
                <span class="text-[10px] text-[var(--color-text-muted)] print:text-neutral-600 font-mono uppercase">Info</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Reconnaissance Intelligence Section -->
        <div class="space-y-3">
          <h3 class="text-xs font-mono font-bold uppercase tracking-widest text-[var(--color-text-muted)] print:text-neutral-700 border-b border-[var(--color-hairline)] print:border-neutral-200 pb-1.5">
            Surface Posture & Infrastructure
          </h3>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
            <!-- DNS & Email Card -->
            <div class="p-4 bg-[var(--color-canvas)] print:bg-neutral-50 border border-[var(--color-hairline)] print:border-neutral-200 rounded-none space-y-2">
              <div class="flex items-center gap-1.5 font-mono font-semibold text-[var(--color-text-headline)] print:text-neutral-800 uppercase tracking-wider text-[11px]">
                <Mail class="w-3.5 h-3.5 text-[var(--color-text-muted)] print:text-neutral-700" />
                <span>DNS & Email Security</span>
              </div>
              {#if report.dns_security}
                <div class="space-y-1 text-[var(--color-text-body)] print:text-neutral-700 font-mono text-[11px]">
                  <div class="flex justify-between">
                    <span class="text-[var(--color-text-muted)]">SPF RECORD:</span>
                    <span class={report.dns_security.spf_record ? "text-[var(--color-signal-emerald)] print:text-emerald-700 font-bold" : "text-[var(--color-signal-red)] print:text-red-700 font-bold"}>
                      {report.dns_security.spf_record ? "CONFIGURED" : "MISSING"}
                    </span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-[var(--color-text-muted)]">DMARC POLICY:</span>
                    <span class={report.dns_security.dmarc_policy && report.dns_security.dmarc_policy !== "none" ? "text-[var(--color-signal-emerald)] print:text-emerald-700 font-bold" : "text-[var(--color-signal-amber)] print:text-amber-700 font-bold"}>
                      {report.dns_security.dmarc_policy ? report.dns_security.dmarc_policy.toUpperCase() : "NONE"}
                    </span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-[var(--color-text-muted)]">DNSSEC:</span>
                    <span class={report.dns_security.dnssec_enabled ? "text-[var(--color-signal-emerald)] print:text-emerald-700 font-bold" : "text-[var(--color-text-muted)]"}>
                      {report.dns_security.dnssec_enabled ? "ACTIVE" : "DISABLED"}
                    </span>
                  </div>
                </div>
              {:else}
                <p class="text-[var(--color-text-muted)] font-mono text-[11px]">No DNS audit data available.</p>
              {/if}
            </div>

            <!-- Endpoints & Network Card -->
            <div class="p-4 bg-[var(--color-canvas)] print:bg-neutral-50 border border-[var(--color-hairline)] print:border-neutral-200 rounded-none space-y-2">
              <div class="flex items-center gap-1.5 font-mono font-semibold text-[var(--color-text-headline)] print:text-neutral-800 uppercase tracking-wider text-[11px]">
                <Globe class="w-3.5 h-3.5 text-[var(--color-text-muted)] print:text-neutral-700" />
                <span>Surface Assets & Endpoints</span>
              </div>
              <div class="space-y-1 text-[var(--color-text-body)] print:text-neutral-700 font-mono text-[11px]">
                <div class="flex justify-between">
                  <span class="text-[var(--color-text-muted)]">OPEN PORTS:</span>
                  <span class={report.port_report && report.port_report.open_ports_count > 0 ? "text-[var(--color-signal-emerald)] print:text-emerald-700 font-bold" : "text-[var(--color-text-muted)]"}>
                    {report.port_report ? `${report.port_report.open_ports_count} / ${report.port_report.scanned_ports_count}` : "NOT SCANNED"}
                  </span>
                </div>
                <div class="flex justify-between">
                  <span class="text-[var(--color-text-muted)]">SUBDOMAINS:</span>
                  <span class="text-[var(--color-text-headline)] print:text-neutral-800 font-bold">{report.subdomains?.length || 0}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-[var(--color-text-muted)]">ROBOTS.TXT:</span>
                  <span class="font-bold">{report.endpoint_report?.robots_txt_found ? "PRESENT" : "NOT FOUND"}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-[var(--color-text-muted)]">SECURITY.TXT:</span>
                  <span class={report.endpoint_report?.security_txt_found ? "text-[var(--color-signal-emerald)] print:text-emerald-700 font-bold" : "text-[var(--color-text-muted)]"}>
                    {report.endpoint_report?.security_txt_found ? "VERIFIED" : "MISSING"}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Open Ports Discovered Table (if scanned and open ports exist) -->
        {#if report.port_report && report.port_report.open_ports.length > 0}
          <div class="space-y-2">
            <h4 class="text-[10px] font-mono font-bold uppercase tracking-widest text-[var(--color-text-muted)] print:text-neutral-600 flex items-center gap-1.5">
              <Server class="w-3.5 h-3.5 text-[var(--color-text-muted)] print:text-neutral-700" />
              <span>Network Ports & Services ({report.port_report.open_ports.length})</span>
            </h4>
            <div class="border border-[var(--color-hairline)] print:border-neutral-300 rounded-none overflow-hidden text-xs">
              <table class="w-full text-left font-mono">
                <thead class="bg-[var(--color-canvas)] print:bg-neutral-100 text-[var(--color-text-muted)] print:text-neutral-700 border-b border-[var(--color-hairline)] print:border-neutral-300 text-[10px] uppercase">
                  <tr>
                    <th class="p-2.5">Port</th>
                    <th class="p-2.5">Service</th>
                    <th class="p-2.5">Risk Posture</th>
                    <th class="p-2.5">Description / Banner</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-[var(--color-hairline)] print:divide-neutral-200">
                  {#each report.port_report.open_ports as p}
                    <tr class="bg-[var(--color-surface)] print:bg-white text-[var(--color-text-body)] print:text-neutral-800">
                      <td class="p-2.5 font-bold text-[var(--color-text-headline)] print:text-neutral-900">#{p.port}/{p.protocol.toUpperCase()}</td>
                      <td class="p-2.5 font-medium">{p.service}</td>
                      <td class="p-2.5">
                        <span class="px-1.5 py-0.5 rounded-none text-[10px] font-mono font-bold uppercase tracking-widest {p.is_risky ? 'bg-[var(--color-signal-red)]/20 text-[var(--color-signal-red)] print:text-red-700' : 'bg-[var(--color-signal-emerald)]/20 text-[var(--color-signal-emerald)] print:text-emerald-700'}">
                          {p.is_risky ? "Risky" : "Standard"}
                        </span>
                      </td>
                      <td class="p-2.5 truncate max-w-xs text-[11px] text-[var(--color-text-muted)] print:text-neutral-600">
                        {p.banner || p.description}
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {/if}

        <!-- Discovered Subdomains List (if any) -->
        {#if report.subdomains && report.subdomains.length > 0}
          <div class="space-y-2">
            <h4 class="text-[10px] font-mono font-bold uppercase tracking-widest text-[var(--color-text-muted)] print:text-neutral-600">
              Discovered Subdomain Infrastructure ({report.subdomains.length})
            </h4>
            <div class="p-3 bg-[var(--color-canvas)] print:bg-neutral-50 border border-[var(--color-hairline)] print:border-neutral-200 rounded-none max-h-32 overflow-y-auto text-xs font-mono grid grid-cols-2 md:grid-cols-3 gap-1">
              {#each report.subdomains as sub}
                <div class="truncate text-[var(--color-text-body)] print:text-neutral-800">• {sub}</div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Detailed Vulnerability Findings -->
        <div class="space-y-3 pt-2">
          <h3 class="text-xs font-mono font-bold uppercase tracking-widest text-[var(--color-text-muted)] print:text-neutral-700 border-b border-[var(--color-hairline)] print:border-neutral-200 pb-1.5">
            Vulnerability Audit Findings ({report.findings.length})
          </h3>

          {#if report.findings.length === 0}
            <div class="p-5 text-center text-[var(--color-signal-emerald)] font-mono text-xs font-bold uppercase tracking-wider bg-[var(--color-signal-emerald)]/10 border border-[var(--color-signal-emerald)]/30 rounded-none">
              ✓ No security vulnerabilities or configuration weaknesses detected
            </div>
          {:else}
            <div class="space-y-2.5">
              {#each report.findings as finding, idx}
                <div class="p-3.5 bg-[var(--color-canvas)] print:bg-neutral-50 border border-[var(--color-hairline)] print:border-neutral-300 rounded-none space-y-1.5">
                  <div class="flex items-center justify-between gap-2">
                    <div class="flex items-center gap-2 min-w-0">
                      <span class="text-xs text-[var(--color-text-muted)] font-mono">#{idx + 1}</span>
                      <h4 class="text-xs font-mono font-bold text-[var(--color-text-headline)] print:text-neutral-900 truncate uppercase">{finding.title}</h4>
                    </div>
                    <SeverityBadge severity={finding.severity} />
                  </div>

                  <div class="text-[11px] text-[var(--color-text-muted)] print:text-neutral-600 font-mono">
                    <span>OWASP: {finding.owasp_category}</span>
                    {#if finding.cve_id}
                      <span class="ml-2 px-1.5 py-0.5 bg-[var(--color-signal-purple)]/15 text-[var(--color-signal-purple)] border border-[var(--color-signal-purple)]/30 rounded-none font-bold text-[10px]">{finding.cve_id}</span>
                    {/if}
                  </div>

                  <p class="text-xs text-[var(--color-text-body)] print:text-neutral-800 font-mono">{finding.description}</p>

                  <div class="pt-1 text-xs text-[var(--color-text-muted)] print:text-neutral-700 font-mono">
                    <strong class="text-[var(--color-text-headline)] print:text-neutral-900 uppercase">Remediation: </strong>
                    {finding.remediation}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Footer -->
        <div class="border-t border-[var(--color-hairline)] print:border-neutral-300 pt-4 text-center text-[10px] text-[var(--color-text-muted)] font-mono uppercase tracking-widest">
          Report generated by VulnRadar • Web Vulnerability & Security Scanner
        </div>

      </div>
    </div>
  </div>
{/if}
