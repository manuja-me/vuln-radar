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
    critical: "border-l-red-500",
    high: "border-l-orange-500",
    medium: "border-l-amber-500",
    low: "border-l-blue-500",
    info: "border-l-zinc-500",
  };
</script>

<div
  class="border border-[var(--color-hairline)] border-l-4 {severityBorders[finding.severity] || 'border-l-[var(--color-hairline)]'} bg-[var(--color-surface)] hover:bg-[var(--color-surface-hover)] rounded-none overflow-hidden transition-colors"
>
  <!-- Header row / trigger -->
  <button
    type="button"
    class="w-full text-left p-4 flex items-center justify-between gap-4 cursor-pointer select-none focus:outline-none"
    onclick={toggle}
  >
    <div class="flex items-center gap-3 min-w-0 flex-1">
      <SeverityBadge severity={finding.severity} />
      <span class="font-bold text-[var(--color-text-headline)] text-sm tracking-tight truncate">{finding.title}</span>
    </div>

    <div class="flex items-center gap-2 flex-shrink-0">
      {#if finding.cve_id}
        <span
          class="px-2 py-0.5 text-[10px] font-mono font-bold bg-purple-500/10 text-purple-600 dark:text-purple-400 border border-purple-500/30 rounded-none uppercase tracking-wider"
        >
          {finding.cve_id}
        </span>
      {/if}
      <span
        class="text-[10px] text-[var(--color-text-muted)] font-mono font-semibold hidden md:inline-block px-2 py-0.5 bg-[var(--color-canvas)] rounded-none border border-[var(--color-hairline)] uppercase tracking-wider"
      >
        {categoryLabels[finding.category] || finding.category}
      </span>
      <div class="text-[var(--color-text-muted)] p-1 rounded-none hover:text-[var(--color-text-headline)] transition-colors">
        {#if expanded}
          <ChevronUp class="w-4 h-4 text-[var(--color-text-headline)]" />
        {:else}
          <ChevronDown class="w-4 h-4" />
        {/if}
      </div>
    </div>
  </button>

  <!-- Expandable Content -->
  {#if expanded}
    <div
      class="p-5 pt-3 border-t border-[var(--color-hairline)] space-y-4 text-sm bg-[var(--color-canvas)]"
    >
      <!-- Classification tags -->
      <div class="flex flex-wrap items-center gap-2">
        <span
          class="px-2 py-0.5 text-[10px] bg-[var(--color-surface)] border border-[var(--color-hairline)] text-[var(--color-text-muted)] rounded-none font-mono font-bold uppercase tracking-wider flex items-center gap-1.5"
        >
          <Shield class="w-3.5 h-3.5 text-[var(--color-text-muted)]" />
          {finding.owasp_category}
        </span>
      </div>

      <!-- Description -->
      <div class="space-y-1">
        <div class="text-[10px] font-mono font-bold text-[var(--color-text-muted)] uppercase tracking-widest">
          OVERVIEW
        </div>
        <p class="text-[var(--color-text-body)] text-xs leading-relaxed font-sans">{finding.description}</p>
      </div>

      <!-- Impact & Remediation Grid -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3 pt-1">
        <!-- Impact -->
        <div
          class="space-y-1.5 bg-red-500/5 border border-red-500/20 p-3.5 rounded-none"
        >
          <div
            class="text-[10px] font-mono font-bold text-red-600 dark:text-red-400 uppercase tracking-widest flex items-center gap-1.5"
          >
            <AlertCircle class="w-3.5 h-3.5" />
            <span>SECURITY IMPACT</span>
          </div>
          <p class="text-[var(--color-text-body)] text-xs leading-relaxed font-sans">{finding.impact}</p>
        </div>

        <!-- Remediation -->
        <div
          class="space-y-1.5 bg-emerald-500/5 border border-emerald-500/20 p-3.5 rounded-none flex flex-col justify-between"
        >
          <div>
            <div
              class="text-[10px] font-mono font-bold text-emerald-600 dark:text-emerald-400 uppercase tracking-widest flex items-center justify-between gap-1.5 mb-1"
            >
              <div class="flex items-center gap-1.5">
                <CheckCircle2 class="w-3.5 h-3.5" />
                <span>REMEDIATION</span>
              </div>
              <button
                type="button"
                onclick={copyRemediation}
                class="px-2 py-0.5 text-[10px] font-mono font-bold text-emerald-600 dark:text-emerald-400 hover:text-emerald-700 dark:hover:text-emerald-300 flex items-center gap-1 bg-emerald-500/10 border border-emerald-500/30 rounded-none cursor-pointer transition-colors uppercase tracking-wider"
                title="Copy remediation guidance"
              >
                {#if copiedRemediationText}
                  <Check class="w-2.5 h-2.5 text-emerald-500" />
                  <span>COPIED</span>
                {:else}
                  <Copy class="w-2.5 h-2.5" />
                  <span>COPY</span>
                {/if}
              </button>
            </div>
            <p class="text-[var(--color-text-body)] text-xs leading-relaxed font-sans">
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
              class="text-[10px] font-mono font-bold text-[var(--color-text-muted)] uppercase tracking-widest flex items-center gap-1.5"
            >
              <Terminal class="w-3.5 h-3.5 text-[var(--color-text-muted)]" />
              <span>TECHNICAL EVIDENCE & TRIGGER</span>
            </div>
            <button
              type="button"
              onclick={copyEvidence}
              class="px-2 py-0.5 text-[10px] font-mono font-bold text-[var(--color-text-muted)] hover:text-[var(--color-text-headline)] flex items-center gap-1 bg-[var(--color-surface)] border border-[var(--color-hairline)] hover:border-[var(--color-hairline-strong)] rounded-none cursor-pointer transition-colors uppercase tracking-wider"
            >
              {#if copied}
                <Check class="w-3 h-3 text-emerald-500" />
                <span class="text-emerald-500">COPIED</span>
              {:else}
                <Copy class="w-3 h-3" />
                <span>COPY</span>
              {/if}
            </button>
          </div>
          <pre
            class="bg-[var(--color-surface)] border border-[var(--color-hairline)] p-3 rounded-none text-xs font-mono text-[var(--color-text-body)] overflow-x-auto whitespace-pre-wrap leading-relaxed"
          >{finding.evidence}</pre>
        </div>
      {/if}

      <!-- References -->
      {#if finding.references.length > 0}
        <div class="space-y-1.5 pt-1">
          <div class="text-[10px] font-mono font-bold text-[var(--color-text-muted)] uppercase tracking-widest">
            ADVISORIES & REFERENCES
          </div>
          <ul class="space-y-1">
            {#each finding.references as reference}
              <li>
                <a
                  href={reference}
                  target="_blank"
                  rel="noreferrer"
                  class="text-[var(--color-signal-blue)] hover:underline text-xs flex items-center gap-1.5 truncate font-mono"
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
