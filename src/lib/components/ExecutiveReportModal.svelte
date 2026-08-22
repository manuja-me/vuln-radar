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
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4 print:p-0 print:bg-white print:static animate-fade-in overflow-y-auto">
    <div class="bg-slate-900 border border-slate-800 print:border-none print:bg-white print:text-black rounded-2xl w-full max-w-4xl max-h-[90vh] print:max-h-none flex flex-col shadow-2xl overflow-hidden print:overflow-visible my-auto">
      
      <!-- Top Action Bar (hidden when printing) -->
      <div class="p-4 border-b border-slate-800 flex items-center justify-between bg-slate-950/60 print:hidden">
        <div class="flex items-center gap-2">
          <Shield class="w-5 h-5 text-cyan-400" />
          <h2 class="text-base font-bold text-slate-100">Executive Security Assessment Report</h2>
        </div>
        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={triggerPrint}
            class="px-3.5 py-1.5 bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold rounded-lg text-xs flex items-center gap-1.5 transition-all shadow-lg shadow-cyan-500/20 cursor-pointer"
          >
            <Printer class="w-4 h-4" />
            <span>Print / Save PDF</span>
          </button>
          <button
            type="button"
            onclick={onClose}
            class="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-lg transition-colors cursor-pointer"
          >
            <X class="w-5 h-5" />
          </button>
        </div>
      </div>

      <!-- Printable Report Container -->
      <div class="flex-1 overflow-y-auto print:overflow-visible p-6 md:p-8 space-y-6 text-slate-200 print:text-slate-900 print:bg-white">
        
        <!-- Header / Organization Branding -->
        <div class="border-b border-slate-800 print:border-slate-300 pb-6 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <div>
            <div class="flex items-center gap-2">
              <span class="text-cyan-400 print:text-cyan-700 font-mono font-black text-xl tracking-tight">VulnRadar</span>
              <span class="px-2 py-0.5 text-[10px] font-mono bg-slate-800 print:bg-slate-200 text-slate-300 print:text-slate-800 rounded">v0.1.0</span>
            </div>
            <h1 class="text-xl font-bold text-white print:text-black mt-1">Web Security Posture Audit</h1>
            <p class="text-xs text-slate-400 print:text-slate-600 font-mono mt-0.5">Target: {report.target_url}</p>
          </div>

          <div class="text-left sm:text-right text-xs font-mono text-slate-400 print:text-slate-600 space-y-0.5">
            <div><strong>Scan ID:</strong> {report.id}</div>
            <div><strong>Generated:</strong> {formatDate(report.scanned_at)}</div>
            <div><strong>Response Time:</strong> {report.response_time_ms} ms</div>
          </div>
        </div>

        <!-- Executive Summary Cards -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <!-- Score Gauge Card -->
          <div class="p-5 bg-slate-950/60 print:bg-slate-50 border border-slate-800 print:border-slate-200 rounded-xl flex items-center justify-center">
            <ScoreGauge score={report.security_score} />
          </div>

          <!-- Finding Breakdown Stats -->
          <div class="p-5 bg-slate-950/60 print:bg-slate-50 border border-slate-800 print:border-slate-200 rounded-xl flex flex-col justify-center space-y-2 col-span-1 md:col-span-2">
            <h3 class="text-xs font-bold uppercase tracking-wider text-slate-400 print:text-slate-600">Risk Breakdown</h3>
            <div class="grid grid-cols-5 gap-2 text-center pt-1">
              <div class="p-2 bg-rose-500/10 border border-rose-500/30 rounded-lg">
                <span class="text-xs font-bold text-rose-400 block font-mono">{report.critical_count}</span>
                <span class="text-[10px] text-rose-300 font-medium">Critical</span>
              </div>
              <div class="p-2 bg-orange-500/10 border border-orange-500/30 rounded-lg">
                <span class="text-xs font-bold text-orange-400 block font-mono">{report.high_count}</span>
                <span class="text-[10px] text-orange-300 font-medium">High</span>
              </div>
              <div class="p-2 bg-amber-500/10 border border-amber-500/30 rounded-lg">
                <span class="text-xs font-bold text-amber-400 block font-mono">{report.medium_count}</span>
                <span class="text-[10px] text-amber-300 font-medium">Medium</span>
              </div>
              <div class="p-2 bg-blue-500/10 border border-blue-500/30 rounded-lg">
                <span class="text-xs font-bold text-blue-400 block font-mono">{report.low_count}</span>
                <span class="text-[10px] text-blue-300 font-medium">Low</span>
              </div>
              <div class="p-2 bg-slate-500/10 border border-slate-500/30 rounded-lg">
                <span class="text-xs font-bold text-slate-300 print:text-slate-700 block font-mono">{report.info_count}</span>
                <span class="text-[10px] text-slate-400 print:text-slate-600 font-medium">Info</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Reconnaissance Intelligence Section -->
        <div class="space-y-3">
          <h3 class="text-sm font-bold uppercase tracking-wider text-slate-400 print:text-slate-700 border-b border-slate-800 print:border-slate-200 pb-1">
            Reconnaissance & Surface Posture
          </h3>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
            <!-- DNS & Email Card -->
            <div class="p-4 bg-slate-950/40 print:bg-slate-50 border border-slate-800 print:border-slate-200 rounded-xl space-y-2">
              <div class="flex items-center gap-1.5 font-bold text-slate-200 print:text-slate-800">
                <Mail class="w-4 h-4 text-cyan-400 print:text-cyan-700" />
                <span>DNS & Email Security</span>
              </div>
              {#if report.dns_security}
                <div class="space-y-1 text-slate-300 print:text-slate-700 font-mono">
                  <div class="flex justify-between">
                    <span>SPF Record:</span>
                    <span class={report.dns_security.spf_record ? "text-emerald-400 print:text-emerald-700" : "text-rose-400 print:text-rose-700"}>
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
                    <span class={report.dns_security.dnssec_enabled ? "text-emerald-400 print:text-emerald-700" : "text-slate-500"}>
                      {report.dns_security.dnssec_enabled ? "Active" : "Disabled"}
                    </span>
                  </div>
                </div>
              {:else}
                <p class="text-slate-500">No DNS audit data available.</p>
              {/if}
            </div>

            <!-- Endpoints & Subdomains Card -->
            <div class="p-4 bg-slate-950/40 print:bg-slate-50 border border-slate-800 print:border-slate-200 rounded-xl space-y-2">
              <div class="flex items-center gap-1.5 font-bold text-slate-200 print:text-slate-800">
                <Globe class="w-4 h-4 text-cyan-400 print:text-cyan-700" />
                <span>Surface Assets & Endpoints</span>
              </div>
              <div class="space-y-1 text-slate-300 print:text-slate-700 font-mono">
                <div class="flex justify-between">
                  <span>Discovered Subdomains:</span>
                  <span class="text-cyan-400 print:text-cyan-700 font-bold">{report.subdomains?.length || 0}</span>
                </div>
                <div class="flex justify-between">
                  <span>robots.txt:</span>
                  <span>{report.endpoint_report?.robots_txt_found ? "Present" : "Not Found"}</span>
                </div>
                <div class="flex justify-between">
                  <span>security.txt (RFC 9116):</span>
                  <span class={report.endpoint_report?.security_txt_found ? "text-emerald-400 print:text-emerald-700" : "text-slate-500"}>
                    {report.endpoint_report?.security_txt_found ? "Verified" : "Missing"}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Discovered Subdomains List (if any) -->
        {#if report.subdomains && report.subdomains.length > 0}
          <div class="space-y-2">
            <h4 class="text-xs font-bold uppercase tracking-wider text-slate-400 print:text-slate-600">
              Discovered Subdomain Infrastructure ({report.subdomains.length})
            </h4>
            <div class="p-3 bg-slate-950/60 print:bg-slate-50 border border-slate-800 print:border-slate-200 rounded-xl max-h-32 overflow-y-auto text-xs font-mono grid grid-cols-2 md:grid-cols-3 gap-1">
              {#each report.subdomains as sub}
                <div class="truncate text-slate-300 print:text-slate-800">• {sub}</div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Detailed Vulnerability Findings -->
        <div class="space-y-4 pt-2">
          <h3 class="text-sm font-bold uppercase tracking-wider text-slate-400 print:text-slate-700 border-b border-slate-800 print:border-slate-200 pb-1">
            Vulnerability Audit Findings ({report.findings.length})
          </h3>

          {#if report.findings.length === 0}
            <div class="p-6 text-center text-emerald-400 font-bold bg-emerald-500/10 border border-emerald-500/20 rounded-xl">
              ✓ No security vulnerabilities or configuration weaknesses detected!
            </div>
          {:else}
            <div class="space-y-3">
              {#each report.findings as finding, idx}
                <div class="p-4 bg-slate-950/50 print:bg-slate-50 border border-slate-800 print:border-slate-300 rounded-xl space-y-2">
                  <div class="flex items-center justify-between gap-2">
                    <div class="flex items-center gap-2 min-w-0">
                      <span class="text-xs font-bold text-slate-400 print:text-slate-600 font-mono">#{idx + 1}</span>
                      <h4 class="text-sm font-bold text-slate-100 print:text-slate-900 truncate">{finding.title}</h4>
                    </div>
                    <SeverityBadge severity={finding.severity} />
                  </div>

                  <div class="text-xs text-slate-400 print:text-slate-600 font-mono">
                    <span>OWASP: {finding.owasp_category}</span>
                    {#if finding.cve_id}
                      <span class="ml-2 px-1.5 py-0.5 bg-rose-500/10 text-rose-400 border border-rose-500/20 rounded font-bold">{finding.cve_id}</span>
                    {/if}
                  </div>

                  <p class="text-xs text-slate-300 print:text-slate-800">{finding.description}</p>

                  <div class="pt-1 text-xs text-slate-400 print:text-slate-700">
                    <strong class="text-cyan-400 print:text-cyan-800 font-semibold">Remediation: </strong>
                    {finding.remediation}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Footer -->
        <div class="border-t border-slate-800 print:border-slate-300 pt-4 text-center text-xs text-slate-500 font-mono">
          Report generated by VulnRadar • Passive Web Vulnerability Scanner
        </div>

      </div>
    </div>
  </div>
{/if}
