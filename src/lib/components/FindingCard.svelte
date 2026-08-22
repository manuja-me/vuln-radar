<script lang="ts">
  import type { Finding } from "$lib/types";
  import SeverityBadge from "./SeverityBadge.svelte";
  import { ChevronDown, ChevronUp, Shield, ExternalLink, Terminal, AlertCircle, CheckCircle2 } from "lucide-svelte";

  let { finding }: { finding: Finding } = $props();
  let expanded = $state(false);

  function toggle() {
    expanded = !expanded;
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
  };
</script>

<div class="border border-slate-800 hover:border-slate-700 bg-slate-900/40 rounded-xl overflow-hidden transition-all duration-200">
  <!-- Header row / trigger -->
  <button
    type="button"
    class="w-full text-left p-4 flex items-center justify-between gap-4 cursor-pointer select-none focus:outline-none focus:bg-slate-900/80"
    onclick={toggle}
  >
    <div class="flex items-center gap-3 min-w-0 flex-1">
      <SeverityBadge severity={finding.severity} />
      <span class="font-semibold text-slate-100 text-sm truncate">{finding.title}</span>
    </div>

    <div class="flex items-center gap-3 flex-shrink-0">
      {#if finding.cve_id}
        <span class="px-2 py-0.5 text-xs font-mono font-bold bg-purple-500/15 text-purple-400 border border-purple-500/30 rounded">
          {finding.cve_id}
        </span>
      {/if}
      <span class="text-xs text-slate-400 font-mono hidden md:inline-block">
        {categoryLabels[finding.category] || finding.category}
      </span>
      <div class="text-slate-400 p-1">
        {#if expanded}
          <ChevronUp class="w-4 h-4" />
        {:else}
          <ChevronDown class="w-4 h-4" />
        {/if}
      </div>
    </div>
  </button>

  <!-- Expandable Content -->
  {#if expanded}
    <div class="p-4 pt-0 border-t border-slate-800/80 space-y-4 text-sm bg-slate-950/40">
      <!-- Classification tags -->
      <div class="flex flex-wrap items-center gap-2 pt-3">
        <span class="px-2.5 py-1 text-xs bg-slate-800 text-slate-300 rounded-md font-mono flex items-center gap-1.5">
          <Shield class="w-3.5 h-3.5 text-cyan-400" />
          {finding.owasp_category}
        </span>
      </div>

      <!-- Description -->
      <div class="space-y-1">
        <div class="text-xs font-bold uppercase tracking-wider text-slate-400">Description</div>
        <p class="text-slate-300 leading-relaxed">{finding.description}</p>
      </div>

      <!-- Impact -->
      <div class="space-y-1 bg-rose-950/20 border border-rose-900/30 p-3 rounded-lg">
        <div class="text-xs font-bold uppercase tracking-wider text-rose-400 flex items-center gap-1.5">
          <AlertCircle class="w-3.5 h-3.5" />
          Security Impact
        </div>
        <p class="text-rose-200 text-xs leading-relaxed">{finding.impact}</p>
      </div>

      <!-- Remediation -->
      <div class="space-y-1 bg-emerald-950/20 border border-emerald-900/30 p-3 rounded-lg">
        <div class="text-xs font-bold uppercase tracking-wider text-emerald-400 flex items-center gap-1.5">
          <CheckCircle2 class="w-3.5 h-3.5" />
          Remediation Guidance
        </div>
        <p class="text-emerald-200 text-xs leading-relaxed">{finding.remediation}</p>
      </div>

      <!-- Evidence -->
      {#if finding.evidence}
        <div class="space-y-1">
          <div class="text-xs font-bold uppercase tracking-wider text-slate-400 flex items-center gap-1.5">
            <Terminal class="w-3.5 h-3.5 text-cyan-400" />
            Detected Evidence / Trigger
          </div>
          <pre class="bg-slate-950 border border-slate-800 p-3 rounded-lg text-xs font-mono text-cyan-300 overflow-x-auto whitespace-pre-wrap">{finding.evidence}</pre>
        </div>
      {/if}

      <!-- References -->
      {#if finding.references.length > 0}
        <div class="space-y-1 pt-1">
          <div class="text-xs font-bold uppercase tracking-wider text-slate-400">References & Standards</div>
          <ul class="space-y-1">
            {#each finding.references as reference}
              <li>
                <a
                  href={reference}
                  target="_blank"
                  rel="noreferrer"
                  class="text-cyan-400 hover:text-cyan-300 text-xs flex items-center gap-1 hover:underline truncate"
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
