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
  let copiedRemediationText = $state(false);

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

  async function copyRemediation(e: MouseEvent) {
    e.stopPropagation();
    try {
      await navigator.clipboard.writeText(finding.remediation);
      copiedRemediationText = true;
      setTimeout(() => (copiedRemediationText = false), 2000);
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
    port_exposure: "Port / Network",
    rce_risk: "RCE & Injection Risk",
  };

  const severityBorders: Record<string, string> = {
    critical: "border-l-red-500",
    high: "border-l-orange-500",
    medium: "border-l-amber-500",
    low: "border-l-blue-500",
    info: "border-l-neutral-600",
  };
</script>

<div
  class="border border-[#2e2e2e] border-l-4 {severityBorders[finding.severity] || 'border-l-neutral-700'} bg-[#202020] hover:bg-[#242424] rounded-xl overflow-hidden transition-colors"
>
  <!-- Header row / trigger -->
  <button
    type="button"
    class="w-full text-left p-3.5 flex items-center justify-between gap-4 cursor-pointer select-none focus:outline-none"
    onclick={toggle}
  >
    <div class="flex items-center gap-3 min-w-0 flex-1">
      <SeverityBadge severity={finding.severity} />
      <span class="font-medium text-[#e3e2e0] text-sm truncate">{finding.title}</span>
    </div>

    <div class="flex items-center gap-2 flex-shrink-0">
      {#if finding.cve_id}
        <span
          class="px-2 py-0.5 text-[11px] font-mono bg-purple-950/40 text-purple-300 border border-purple-800/50 rounded"
        >
          {finding.cve_id}
        </span>
      {/if}
      <span
        class="text-xs text-neutral-400 font-mono hidden md:inline-block px-2 py-0.5 bg-[#191919] rounded border border-[#2e2e2e]"
      >
        {categoryLabels[finding.category] || finding.category}
      </span>
      <div class="text-neutral-400 p-1 rounded hover:bg-[#2e2e2e] transition-colors">
        {#if expanded}
          <ChevronUp class="w-4 h-4 text-neutral-200" />
        {:else}
          <ChevronDown class="w-4 h-4 text-neutral-400" />
        {/if}
      </div>
    </div>
  </button>

  <!-- Expandable Content -->
  {#if expanded}
    <div
      class="p-4 pt-1 border-t border-[#2e2e2e] space-y-3.5 text-sm bg-[#1a1a1a]"
    >
      <!-- Classification tags -->
      <div class="flex flex-wrap items-center gap-2 pt-2">
        <span
          class="px-2 py-0.5 text-xs bg-[#242424] border border-[#333] text-neutral-300 rounded font-mono flex items-center gap-1.5"
        >
          <Shield class="w-3.5 h-3.5 text-neutral-400" />
          {finding.owasp_category}
        </span>
      </div>

      <!-- Description -->
      <div class="space-y-1">
        <div class="text-[11px] font-medium text-neutral-400 uppercase tracking-wider">
          Overview
        </div>
        <p class="text-neutral-300 text-xs leading-relaxed">{finding.description}</p>
      </div>

      <!-- Impact & Remediation Grid -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3 pt-1">
        <!-- Impact -->
        <div
          class="space-y-1 bg-red-950/20 border border-red-900/30 p-3 rounded-lg"
        >
          <div
            class="text-[11px] font-medium text-red-300 flex items-center gap-1.5"
          >
            <AlertCircle class="w-3.5 h-3.5" />
            <span>Security Impact</span>
          </div>
          <p class="text-red-200/90 text-xs leading-relaxed">{finding.impact}</p>
        </div>

        <!-- Remediation -->
        <div
          class="space-y-1 bg-emerald-950/20 border border-emerald-900/30 p-3 rounded-lg flex flex-col justify-between"
        >
          <div>
            <div
              class="text-[11px] font-medium text-emerald-300 flex items-center justify-between gap-1.5 mb-1"
            >
              <div class="flex items-center gap-1.5">
                <CheckCircle2 class="w-3.5 h-3.5" />
                <span>Remediation</span>
              </div>
              <button
                type="button"
                onclick={copyRemediation}
                class="px-1.5 py-0.5 text-[10px] font-mono text-emerald-400/80 hover:text-emerald-200 flex items-center gap-1 bg-emerald-950/40 border border-emerald-800/40 rounded cursor-pointer transition-colors"
                title="Copy remediation guidance"
              >
                {#if copiedRemediationText}
                  <Check class="w-2.5 h-2.5 text-emerald-400" />
                  <span>Copied</span>
                {:else}
                  <Copy class="w-2.5 h-2.5" />
                  <span>Copy</span>
                {/if}
              </button>
            </div>
            <p class="text-emerald-200/90 text-xs leading-relaxed">
              {finding.remediation}
            </p>
          </div>
        </div>
      </div>

      <!-- Evidence -->
      {#if finding.evidence}
        <div class="space-y-1.5">
          <div class="flex items-center justify-between">
            <div
              class="text-[11px] font-medium text-neutral-400 flex items-center gap-1.5"
            >
              <Terminal class="w-3.5 h-3.5 text-neutral-400" />
              <span>Evidence & Trigger</span>
            </div>
            <button
              type="button"
              onclick={copyEvidence}
              class="px-2 py-0.5 text-[11px] font-mono text-neutral-400 hover:text-white flex items-center gap-1 bg-[#242424] border border-[#333] hover:border-[#444] rounded cursor-pointer transition-colors"
            >
              {#if copied}
                <Check class="w-3 h-3 text-emerald-400" />
                <span class="text-emerald-400">Copied</span>
              {:else}
                <Copy class="w-3 h-3" />
                <span>Copy</span>
              {/if}
            </button>
          </div>
          <pre
            class="bg-[#141414] border border-[#2a2a2a] p-3 rounded-lg text-xs font-mono text-neutral-200 overflow-x-auto whitespace-pre-wrap leading-relaxed"
          >{finding.evidence}</pre>
        </div>
      {/if}

      <!-- References -->
      {#if finding.references.length > 0}
        <div class="space-y-1 pt-1">
          <div class="text-[11px] font-medium text-neutral-400 uppercase tracking-wider">
            Advisories & References
          </div>
          <ul class="space-y-0.5">
            {#each finding.references as reference}
              <li>
                <a
                  href={reference}
                  target="_blank"
                  rel="noreferrer"
                  class="text-blue-400 hover:text-blue-300 text-xs flex items-center gap-1.5 hover:underline truncate"
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

