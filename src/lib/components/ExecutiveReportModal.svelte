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
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 print:p-0 print:bg-white print:static animate-fade-in overflow-y-auto"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
    role="dialog"
    aria-modal="true"
  >
    <div class="bg-[#202020] border border-[#333333] print:border-none print:bg-white print:text-black rounded-xl w-full max-w-4xl max-h-[90vh] print:max-h-none flex flex-col shadow-xl overflow-hidden print:overflow-visible my-auto text-[#e3e2e0]">
      
      <!-- Top Action Bar (hidden when printing) -->
      <div class="p-3.5 border-b border-[#2e2e2e] flex items-center justify-between bg-[#191919] print:hidden">
        <div class="flex items-center gap-2">
          <Shield class="w-4 h-4 text-neutral-300" />
          <h2 class="text-sm font-semibold text-white">Executive Security Assessment Report</h2>
        </div>
        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={triggerPrint}
            class="px-3.5 py-1.5 bg-white hover:bg-neutral-200 text-neutral-950 font-semibold rounded-lg text-xs flex items-center gap-1.5 transition-colors cursor-pointer shadow-sm"
          >
            <Printer class="w-3.5 h-3.5" />
            <span>Print / Save PDF</span>
          </button>
          <button
            type="button"
            onclick={onClose}
            class="p-1.5 text-neutral-400 hover:text-white hover:bg-[#282828] rounded-lg transition-colors cursor-pointer"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Printable Report Container -->
      <div class="flex-1 overflow-y-auto print:overflow-visible p-6 md:p-8 space-y-6 text-[#e3e2e0] print:text-neutral-900 print:bg-white">
        
        <!-- Header / Organization Branding -->
        <div class="border-b border-[#2e2e2e] print:border-neutral-300 pb-5 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <div>
            <div class="flex items-center gap-2">
              <span class="text-white print:text-neutral-900 font-bold text-lg tracking-tight">VulnRadar</span>
              <span class="px-1.5 py-0.2 text-[10px] font-mono bg-[#282828] print:bg-neutral-100 text-neutral-400 print:text-neutral-700 rounded border border-[#383838] print:border-neutral-300">v0.4.0</span>
            </div>
            <h1 class="text-xl font-bold text-white print:text-black mt-1">Web Security Posture Audit</h1>
            <p class="text-xs text-neutral-400 print:text-neutral-600 font-mono mt-0.5">Target: {report.target_url}</p>
          </div>

          <div class="text-left sm:text-right text-xs font-mono text-neutral-400 print:text-neutral-600 space-y-0.5">
            <div><strong>Scan ID:</strong> {report.id}</div>
            <div><strong>Generated:</strong> {formatDate(report.scanned_at)}</div>
            <div><strong>Response Time:</strong> {report.response_time_ms} ms</div>
          </div>
        </div>

        <!-- Executive Summary Cards -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
          <!-- Score Gauge Card -->
          <div class="p-4 bg-[#191919] print:bg-neutral-50 border border-[#2e2e2e] print:border-neutral-200 rounded-xl flex items-center justify-center">
            <ScoreGauge score={report.security_score} />
          </div>

          <!-- Finding Breakdown Stats -->
          <div class="p-4 bg-[#191919] print:bg-neutral-50 border border-[#2e2e2e] print:border-neutral-200 rounded-xl flex flex-col justify-center space-y-2 col-span-1 md:col-span-2">
            <h3 class="text-xs font-medium uppercase tracking-wider text-neutral-400 print:text-neutral-600">Risk Breakdown</h3>
            <div class="grid grid-cols-5 gap-2 text-center pt-1">
              <div class="p-2 bg-red-950/30 border border-red-800/40 rounded-lg">
                <span class="text-xs font-bold text-red-400 block font-mono">{report.critical_count}</span>
                <span class="text-[10px] text-red-300 font-medium">Critical</span>
              </div>
              <div class="p-2 bg-orange-950/30 border border-orange-800/40 rounded-lg">
                <span class="text-xs font-bold text-orange-400 block font-mono">{report.high_count}</span>
                <span class="text-[10px] text-orange-300 font-medium">High</span>
              </div>
              <div class="p-2 bg-amber-950/30 border border-amber-800/40 rounded-lg">
                <span class="text-xs font-bold text-amber-400 block font-mono">{report.medium_count}</span>
                <span class="text-[10px] text-amber-300 font-medium">Medium</span>
              </div>
              <div class="p-2 bg-blue-950/30 border border-blue-800/40 rounded-lg">
                <span class="text-xs font-bold text-blue-400 block font-mono">{report.low_count}</span>
                <span class="text-[10px] text-blue-300 font-medium">Low</span>
              </div>
              <div class="p-2 bg-[#252525] border border-[#383838] rounded-lg">
                <span class="text-xs font-bold text-neutral-300 print:text-neutral-700 block font-mono">{report.info_count}</span>
                <span class="text-[10px] text-neutral-400 print:text-neutral-600 font-medium">Info</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Reconnaissance Intelligence Section -->
        <div class="space-y-3">
          <h3 class="text-sm font-semibold uppercase tracking-wider text-neutral-400 print:text-neutral-700 border-b border-[#2e2e2e] print:border-neutral-200 pb-1">
            Reconnaissance & Surface Posture
          </h3>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
            <!-- DNS & Email Card -->
            <div class="p-4 bg-[#191919] print:bg-neutral-50 border border-[#2e2e2e] print:border-neutral-200 rounded-xl space-y-2">
              <div class="flex items-center gap-1.5 font-medium text-neutral-200 print:text-neutral-800">
                <Mail class="w-4 h-4 text-neutral-400 print:text-neutral-700" />
                <span>DNS & Email Security</span>
              </div>
              {#if report.dns_security}
                <div class="space-y-1 text-neutral-300 print:text-neutral-700 font-mono">
                  <div class="flex justify-between">
                    <span>SPF Record:</span>
                    <span class={report.dns_security.spf_record ? "text-emerald-400 print:text-emerald-700" : "text-red-400 print:text-red-700"}>
                      {report.dns_security.spf_record ? "Configured" : "Missing"}
                    </span>
                  </div>
                  <div class="flex justify-between">
                    <span>DMARC Policy:</span>
                    <span class={report.dns_security.dmarc_policy && report.dns_security.dmarc_policy !== "none" ? "text-emerald-400 print:text-emerald-700" : "text-amber-400 print:text-amber-700"}>
                      {report.dns_security.dmarc_policy || "None"}
                    </span>
                  </div>
                  <div class="flex justify-between">
                    <span>DNSSEC:</span>
                    <span class={report.dns_security.dnssec_enabled ? "text-emerald-400 print:text-emerald-700" : "text-neutral-500"}>
                      {report.dns_security.dnssec_enabled ? "Active" : "Disabled"}
                    </span>
                  </div>
                </div>
              {:else}
                <p class="text-neutral-500">No DNS audit data available.</p>
              {/if}
            </div>

            <!-- Endpoints & Network Card -->
            <div class="p-4 bg-[#191919] print:bg-neutral-50 border border-[#2e2e2e] print:border-neutral-200 rounded-xl space-y-2">
              <div class="flex items-center gap-1.5 font-medium text-neutral-200 print:text-neutral-800">
                <Globe class="w-4 h-4 text-neutral-400 print:text-neutral-700" />
                <span>Surface Assets & Open Ports</span>
              </div>
              <div class="space-y-1 text-neutral-300 print:text-neutral-700 font-mono">
                <div class="flex justify-between">
                  <span>Discovered Open Ports:</span>
                  <span class={report.port_report && report.port_report.open_ports_count > 0 ? "text-emerald-400 print:text-emerald-700 font-bold" : "text-neutral-400"}>
                    {report.port_report ? `${report.port_report.open_ports_count} of ${report.port_report.scanned_ports_count}` : "Not Scanned"}
                  </span>
                </div>
                <div class="flex justify-between">
                  <span>Discovered Subdomains:</span>
                  <span class="text-white print:text-neutral-800 font-bold">{report.subdomains?.length || 0}</span>
                </div>
                <div class="flex justify-between">
                  <span>robots.txt:</span>
                  <span>{report.endpoint_report?.robots_txt_found ? "Present" : "Not Found"}</span>
                </div>
                <div class="flex justify-between">
                  <span>security.txt (RFC 9116):</span>
                  <span class={report.endpoint_report?.security_txt_found ? "text-emerald-400 print:text-emerald-700" : "text-neutral-500"}>
                    {report.endpoint_report?.security_txt_found ? "Verified" : "Missing"}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Open Ports Discovered Table (if scanned and open ports exist) -->
        {#if report.port_report && report.port_report.open_ports.length > 0}
          <div class="space-y-2">
            <h4 class="text-xs font-medium uppercase tracking-wider text-neutral-400 print:text-neutral-600 flex items-center gap-1.5">
              <Server class="w-3.5 h-3.5 text-neutral-400 print:text-neutral-700" />
              <span>Discovered Network Ports & Services ({report.port_report.open_ports.length})</span>
            </h4>
            <div class="border border-[#2e2e2e] print:border-neutral-300 rounded-lg overflow-hidden text-xs">
              <table class="w-full text-left font-mono">
                <thead class="bg-[#191919] print:bg-neutral-100 text-neutral-400 print:text-neutral-700 border-b border-[#2e2e2e] print:border-neutral-300 text-[10px] uppercase">
                  <tr>
                    <th class="p-2.5">Port</th>
                    <th class="p-2.5">Service</th>
                    <th class="p-2.5">Risk Posture</th>
                    <th class="p-2.5">Description / Banner</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-[#282828] print:divide-neutral-200">
                  {#each report.port_report.open_ports as p}
                    <tr class="bg-[#202020] print:bg-white text-neutral-300 print:text-neutral-800">
                      <td class="p-2.5 font-bold text-white print:text-neutral-900">#{p.port}/{p.protocol.toUpperCase()}</td>
                      <td class="p-2.5 font-medium">{p.service}</td>
                      <td class="p-2.5">
                        <span class="px-1.5 py-0.5 rounded text-[10px] font-medium {p.is_risky ? 'bg-red-950/40 text-red-300 print:text-red-700' : 'bg-emerald-950/40 text-emerald-300 print:text-emerald-700'}">
                          {p.is_risky ? "Risky" : "Standard"}
                        </span>
                      </td>
                      <td class="p-2.5 truncate max-w-xs text-[11px] text-neutral-400 print:text-neutral-600">
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
            <h4 class="text-xs font-medium uppercase tracking-wider text-neutral-400 print:text-neutral-600">
              Discovered Subdomain Infrastructure ({report.subdomains.length})
            </h4>
            <div class="p-3 bg-[#191919] print:bg-neutral-50 border border-[#2e2e2e] print:border-neutral-200 rounded-lg max-h-32 overflow-y-auto text-xs font-mono grid grid-cols-2 md:grid-cols-3 gap-1">
              {#each report.subdomains as sub}
                <div class="truncate text-neutral-300 print:text-neutral-800">• {sub}</div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Detailed Vulnerability Findings -->
        <div class="space-y-3 pt-2">
          <h3 class="text-sm font-semibold uppercase tracking-wider text-neutral-400 print:text-neutral-700 border-b border-[#2e2e2e] print:border-neutral-200 pb-1">
            Vulnerability Audit Findings ({report.findings.length})
          </h3>

          {#if report.findings.length === 0}
            <div class="p-5 text-center text-emerald-400 font-medium bg-emerald-950/20 border border-emerald-900/40 rounded-lg">
              ✓ No security vulnerabilities or configuration weaknesses detected!
            </div>
          {:else}
            <div class="space-y-2.5">
              {#each report.findings as finding, idx}
                <div class="p-3.5 bg-[#191919] print:bg-neutral-50 border border-[#2e2e2e] print:border-neutral-300 rounded-lg space-y-1.5">
                  <div class="flex items-center justify-between gap-2">
                    <div class="flex items-center gap-2 min-w-0">
                      <span class="text-xs text-neutral-500 font-mono">#{idx + 1}</span>
                      <h4 class="text-sm font-medium text-white print:text-neutral-900 truncate">{finding.title}</h4>
                    </div>
                    <SeverityBadge severity={finding.severity} />
                  </div>

                  <div class="text-xs text-neutral-400 print:text-neutral-600 font-mono">
                    <span>OWASP: {finding.owasp_category}</span>
                    {#if finding.cve_id}
                      <span class="ml-2 px-1.5 py-0.2 bg-purple-950/40 text-purple-300 border border-purple-800/50 rounded font-bold">{finding.cve_id}</span>
                    {/if}
                  </div>

                  <p class="text-xs text-neutral-300 print:text-neutral-800">{finding.description}</p>

                  <div class="pt-1 text-xs text-neutral-400 print:text-neutral-700">
                    <strong class="text-neutral-200 print:text-neutral-900 font-medium">Remediation: </strong>
                    {finding.remediation}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Footer -->
        <div class="border-t border-[#2e2e2e] print:border-neutral-300 pt-4 text-center text-xs text-neutral-500 font-mono">
          Report generated by VulnRadar • Web Vulnerability & Security Scanner
        </div>

      </div>
    </div>
  </div>
{/if}
