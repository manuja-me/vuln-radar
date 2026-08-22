<script lang="ts">
  import type { Finding } from "$lib/types";
  import SeverityBadge from "./SeverityBadge.svelte";
  import {
    ChevronDown,
    ChevronUp,
    Shield,
    ExternalLink,
    Terminal,
    AlertCircle,
    CheckCircle2,
    Copy,
    Check,
  } from "lucide-svelte";

  let { finding }: { finding: Finding } = $props();
  let expanded = $state(false);
  let copied = $state(false);

  function toggle() {
    expanded = !expanded;
  }

  async function copyEvidence(e: MouseEvent) {
    e.stopPropagation();
    if (!finding.evidence) return;
    try {
      await navigator.clipboard.writeText(finding.evidence);
      copied = true;
      setTimeout(() => (copied = false), 2000);
    } catch {
      // ignore
    }
  }

  const categoryLabels: Record<string, string> = {
    security_headers: "Security Headers",
    cookie_security: "Cookie Policy",
    vulnerable_dependency: "Vulnerable Dependency",
    information_disclosure: "Information Leakage",
    tls_ssl: "TLS / HTTPS",
    cors_misconfiguration: "CORS Policy",
    insecure_form: "Form Security",
    dom_security: "DOM Security",
    dns_email_security: "DNS & Email",
    endpoint_exposure: "Endpoint / Recon",
  };

  const severityBorders: Record<string, string> = {
    critical: "border-l-rose-500 hover:border-slate-700",
    high: "border-l-orange-500 hover:border-slate-700",
    medium: "border-l-amber-500 hover:border-slate-700",
    low: "border-l-blue-500 hover:border-slate-700",
    info: "border-l-slate-600 hover:border-slate-700",
  };
</script>

<div
  class="border border-slate-800/90 border-l-4 {severityBorders[finding.severity] || 'border-l-slate-700'} bg-slate-900/40 hover:bg-slate-900/70 rounded-xl overflow-hidden transition-all duration-200 shadow-sm"
>
  <!-- Header row / trigger -->
  <button
    type="button"
    class="w-full text-left p-4 flex items-center justify-between gap-4 cursor-pointer select-none focus:outline-none focus:bg-slate-900/90"
    onclick={toggle}
  >
    <div class="flex items-center gap-3 min-w-0 flex-1">
      <SeverityBadge severity={finding.severity} />
      <span class="font-bold text-slate-100 text-sm truncate">{finding.title}</span>
    </div>

    <div class="flex items-center gap-2.5 flex-shrink-0">
      {#if finding.cve_id}
        <span
          class="px-2 py-0.5 text-[11px] font-mono font-bold bg-purple-500/15 text-purple-400 border border-purple-500/30 rounded-md"
        >
          {finding.cve_id}
        </span>
      {/if}
      <span
        class="text-xs text-slate-400 font-mono hidden md:inline-block px-2 py-0.5 bg-slate-950/60 rounded border border-slate-800"
      >
        {categoryLabels[finding.category] || finding.category}
      </span>
      <div class="text-slate-400 p-1 rounded-md hover:bg-slate-800/80 transition-colors">
        {#if expanded}
          <ChevronUp class="w-4 h-4 text-cyan-400" />
        {:else}
          <ChevronDown class="w-4 h-4" />
        {/if}
      </div>
    </div>
  </button>

  <!-- Expandable Content -->
  {#if expanded}
    <div
      class="p-5 pt-1 border-t border-slate-800/80 space-y-4 text-sm bg-slate-950/50 animate-fade-in"
    >
      <!-- Classification tags -->
      <div class="flex flex-wrap items-center gap-2 pt-2">
        <span
          class="px-2.5 py-1 text-xs bg-slate-900 border border-slate-800 text-slate-300 rounded-lg font-mono flex items-center gap-1.5"
        >
          <Shield class="w-3.5 h-3.5 text-cyan-400" />
          {finding.owasp_category}
        </span>
      </div>

      <!-- Description -->
      <div class="space-y-1.5">
        <div class="text-[11px] font-mono font-bold uppercase tracking-wider text-slate-400">
          Vulnerability Overview
        </div>
        <p class="text-slate-300 text-xs leading-relaxed">{finding.description}</p>
      </div>

      <!-- Impact & Remediation Grid -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3 pt-1">
        <!-- Impact -->
        <div
          class="space-y-1.5 bg-rose-950/20 border border-rose-900/40 p-3.5 rounded-xl"
        >
          <div
            class="text-[11px] font-mono font-bold uppercase tracking-wider text-rose-400 flex items-center gap-1.5"
          >
            <AlertCircle class="w-3.5 h-3.5" />
            Security Impact
          </div>
          <p class="text-rose-200/90 text-xs leading-relaxed">{finding.impact}</p>
        </div>

        <!-- Remediation -->
        <div
          class="space-y-1.5 bg-emerald-950/20 border border-emerald-900/40 p-3.5 rounded-xl"
        >
          <div
            class="text-[11px] font-mono font-bold uppercase tracking-wider text-emerald-400 flex items-center gap-1.5"
          >
            <CheckCircle2 class="w-3.5 h-3.5" />
            Remediation Guidance
          </div>
          <p class="text-emerald-200/90 text-xs leading-relaxed">
            {finding.remediation}
          </p>
        </div>
      </div>

      <!-- Evidence -->
      {#if finding.evidence}
        <div class="space-y-1.5">
          <div class="flex items-center justify-between">
            <div
              class="text-[11px] font-mono font-bold uppercase tracking-wider text-slate-400 flex items-center gap-1.5"
            >
              <Terminal class="w-3.5 h-3.5 text-cyan-400" />
              Detected Evidence & Trigger
            </div>
            <button
              type="button"
              onclick={copyEvidence}
              class="px-2 py-1 text-[11px] font-mono text-slate-400 hover:text-cyan-300 flex items-center gap-1 bg-slate-900 border border-slate-800 hover:border-slate-700 rounded-md cursor-pointer transition-colors"
            >
              {#if copied}
                <Check class="w-3 h-3 text-emerald-400" />
                <span class="text-emerald-400 font-bold">Copied</span>
              {:else}
                <Copy class="w-3 h-3" />
                <span>Copy</span>
              {/if}
            </button>
          </div>
          <pre
            class="bg-slate-950 border border-slate-800/90 p-3 rounded-xl text-xs font-mono text-cyan-300 overflow-x-auto whitespace-pre-wrap leading-relaxed shadow-inner"
          >{finding.evidence}</pre>
        </div>
      {/if}

      <!-- References -->
      {#if finding.references.length > 0}
        <div class="space-y-1.5 pt-1">
          <div class="text-[11px] font-mono font-bold uppercase tracking-wider text-slate-400">
            Advisories & Standards
          </div>
          <ul class="space-y-1">
            {#each finding.references as reference}
              <li>
                <a
                  href={reference}
                  target="_blank"
                  rel="noreferrer"
                  class="text-cyan-400 hover:text-cyan-300 text-xs flex items-center gap-1.5 hover:underline truncate"
                >
                  <ExternalLink class="w-3 h-3 flex-shrink-0" />
                  <span class="truncate">{reference}</span>
                </a>
              </li>
            {/each}
          </ul>
        </div>
      {/if}
    </div>
  {/if}
</div>

