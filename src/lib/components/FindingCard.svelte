<script lang="ts">
  import type { Finding } from "$lib/types";
  import SeverityBadge from "./SeverityBadge.svelte";
  import {
    ChevronDown,
    ChevronUp,
    Shield,
    ExternalLink,
    Terminal,
    Sparkles,
    Copy,
    Check,
  } from "lucide-svelte";

  let { finding }: { finding: Finding } = $props();
  let expanded = $state(false);
  let copiedEvidence = $state(false);
  let copiedRemediationText = $state(false);
  let copiedAiPrompt = $state(false);

  function toggle() {
    expanded = !expanded;
  }

  async function copyEvidence(e: MouseEvent) {
    e.stopPropagation();
    if (!finding.evidence) return;
    try {
      await navigator.clipboard.writeText(finding.evidence);
      copiedEvidence = true;
      setTimeout(() => (copiedEvidence = false), 2000);
    } catch {}
  }

  async function copyRemediation(e: MouseEvent) {
    e.stopPropagation();
    try {
      await navigator.clipboard.writeText(finding.remediation);
      copiedRemediationText = true;
      setTimeout(() => (copiedRemediationText = false), 2000);
    } catch {}
  }

  async function copyAiPrompt(e: MouseEvent) {
    e.stopPropagation();
    const promptText = `Fix this vulnerability in my application:\n\nTitle: ${finding.title}\nSeverity: ${finding.severity.toUpperCase()}\nOWASP: ${finding.owasp_category}\n${finding.cve_id ? `CVE: ${finding.cve_id}\n` : ""}\nDescription:\n${finding.description}\n\nEvidence:\n${finding.evidence || "Not available"}\n\nRecommended Remediation:\n${finding.remediation}\n\nPlease generate the exact configuration or code patch required to resolve this security issue.`;
    try {
      await navigator.clipboard.writeText(promptText);
      copiedAiPrompt = true;
      setTimeout(() => (copiedAiPrompt = false), 2000);
    } catch {}
  }

  const categoryLabels: Record<string, string> = {
    security_headers: "SECURITY HEADERS",
    cookie_security: "COOKIE POLICY",
    vulnerable_dependency: "DEPENDENCY VULNERABILITY",
    information_disclosure: "INFO EXPOSURE",
    tls_ssl: "TLS / CIPHERS",
    cors_misconfiguration: "CORS CONFIG",
    insecure_form: "FORM HYGIENE",
    dom_security: "DOM INTEGRITY",
    dns_email_security: "DNS & DMARC",
    endpoint_exposure: "ENDPOINT DISCLOSURE",
    port_exposure: "NETWORK PORT",
    rce_risk: "RCE HEURISTIC",
  };

  const severityBorders: Record<string, string> = {
    critical: "border-l-error",
    high: "border-l-primary-container",
    medium: "border-l-primary",
    low: "border-l-secondary",
    info: "border-l-outline",
  };

  const cvssScores: Record<string, string> = {
    critical: "CVSS 9.8",
    high: "CVSS 7.5",
    medium: "CVSS 5.3",
    low: "CVSS 3.7",
    info: "INFO 0.0",
  };
</script>

<article
  class="bg-surface-container-low border border-surface-container-high border-l-4 {severityBorders[finding.severity] || 'border-l-surface-container-high'} rounded-lg transition-colors overflow-hidden shadow-2xs"
>
  <!-- Card Header Row -->
  <button
    type="button"
    class="w-full text-left px-4 py-3 flex items-center justify-between gap-3 cursor-pointer select-none focus:outline-none hover:bg-surface-container/60 transition-colors"
    onclick={toggle}
  >
    <div class="flex items-center gap-2.5 min-w-0 flex-1">
      <SeverityBadge severity={finding.severity} />
      <span class="font-bold text-on-surface text-xs md:text-sm font-mono tracking-tight truncate">
        {finding.title}
      </span>
      {#if finding.cve_id}
        <span
          class="px-2 py-0.5 text-[10px] font-mono font-bold bg-secondary/10 text-secondary border border-secondary/30 rounded uppercase shrink-0"
        >
          {finding.cve_id}
        </span>
      {/if}
    </div>

    <div class="flex items-center gap-3 font-mono text-xs flex-shrink-0">
      <span class="font-bold text-primary text-xs">
        {cvssScores[finding.severity] || "CVSS 5.0"}
      </span>
      <span class="text-[10px] text-outline font-semibold hidden sm:inline-block px-2 py-0.5 bg-surface-container-lowest border border-surface-container-high rounded uppercase tracking-wider">
        {finding.owasp_category}
      </span>
      <div class="text-outline p-0.5 hover:text-on-surface transition-colors">
        {#if expanded}
          <ChevronUp class="w-4 h-4 text-on-surface" />
        {:else}
          <ChevronDown class="w-4 h-4" />
        {/if}
      </div>
    </div>
  </button>

  <!-- Expanded Section -->
  {#if expanded}
    <div class="p-4 sm:p-5 border-t border-surface-container-high space-y-4 bg-surface-container-lowest">
      <p class="text-xs text-on-surface-variant leading-relaxed font-sans">
        {finding.description}
      </p>

      <div class="grid grid-cols-1 lg:grid-cols-12 gap-4 pt-1">
        <!-- Left Column: Proof of Concept Probe & Evidence -->
        <div class="lg:col-span-6 flex flex-col gap-2.5">
          <div class="flex items-center justify-between text-[11px] font-mono text-outline">
            <span class="uppercase tracking-wider font-bold">Proof of Concept Probe</span>
            {#if finding.evidence}
              <button
                type="button"
                onclick={copyEvidence}
                class="text-secondary hover:underline flex items-center gap-1.5 font-mono text-[10px] font-bold uppercase cursor-pointer"
              >
                {#if copiedEvidence}
                  <Check class="w-3 h-3 text-tertiary" />
                  <span>COPIED REPRO</span>
                {:else}
                  <Copy class="w-3 h-3" />
                  <span>COPY REPRO cURL</span>
                {/if}
              </button>
            {/if}
          </div>

          <div class="bg-surface-container-low border border-surface-container-high rounded p-3 font-mono text-xs text-on-surface overflow-x-auto leading-relaxed select-text">
            <span class="text-outline"># Diagnostic Security Assessment</span>
            <div class="text-on-surface mt-1 break-all">
              curl -i -s -k -X GET "{finding.references?.[0] || 'https://target.endpoint'}"
            </div>
          </div>

          <span class="text-[10px] font-mono text-outline uppercase tracking-wider font-bold mt-1">
            Server Response Evidence
          </span>
          <div class="bg-surface-container-low border border-surface-container-high rounded p-3 font-mono text-xs text-error overflow-x-auto leading-relaxed select-text max-h-48">
            {#if finding.evidence}
              <pre class="whitespace-pre-wrap break-all font-mono"><code>{finding.evidence}</code></pre>
            {:else}
              <span class="text-outline">No explicit payload evidence string recorded for this finding.</span>
            {/if}
          </div>
        </div>

        <!-- Right Column: Remediation Blueprint & Configuration Patch -->
        <div class="lg:col-span-6 flex flex-col justify-between gap-3 bg-surface-container-low p-4 rounded-lg border border-surface-container-high">
          <div class="flex flex-col gap-2.5">
            <div class="flex items-center justify-between">
              <span class="text-[11px] font-mono text-on-surface font-bold uppercase tracking-wider">
                Remediation Blueprint
              </span>
              <span class="px-2 py-0.5 text-[9px] font-mono bg-tertiary/10 text-tertiary border border-tertiary/30 rounded uppercase font-bold">
                Safe Configuration Patch
              </span>
            </div>

            <!-- Impact Summary -->
            <div class="text-[11px] font-sans text-on-surface-variant leading-relaxed bg-error/5 border border-error/20 rounded p-2.5">
              <strong class="font-mono text-[10px] uppercase block mb-1 text-error">Security Impact:</strong>
              {finding.impact}
            </div>

            <!-- Remediation Directive Code Block -->
            <div class="flex flex-col gap-1.5 mt-1">
              <span class="font-mono text-[10px] text-outline uppercase font-bold">
                Security Remediation Directive
              </span>
              <div class="bg-surface-container-lowest border border-surface-container-high rounded p-3 font-mono text-xs text-tertiary overflow-x-auto leading-relaxed select-text max-h-40">
                <pre class="whitespace-pre-wrap break-all"><code>{finding.remediation}</code></pre>
              </div>
            </div>
          </div>

          <!-- Bottom Action Buttons in Blueprint -->
          <div class="flex items-center justify-between pt-2.5 border-t border-surface-container-high">
            <button
              type="button"
              onclick={copyAiPrompt}
              class="px-3 py-1.5 bg-secondary/10 hover:bg-secondary/20 text-secondary border border-secondary/30 rounded text-[10px] font-mono font-bold uppercase flex items-center gap-1.5 cursor-pointer transition-colors"
            >
              <Sparkles class="w-3 h-3 text-secondary" />
              <span>{copiedAiPrompt ? "Copied AI Prompt!" : "Generate AI Fix Prompt"}</span>
            </button>

            <button
              type="button"
              onclick={copyRemediation}
              class="text-outline hover:text-on-surface text-[10px] font-mono uppercase font-bold cursor-pointer transition-colors"
            >
              {copiedRemediationText ? "Copied Patch!" : "Copy Directive"}
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}
</article>
