<script lang="ts">
  import type { ScanReport, Finding } from "$lib/types";
  import {
    Sparkles,
    Copy,
    Check,
    X,
    ShieldAlert,
    Terminal,
    Cpu,
    Bot,
  } from "lucide-svelte";

  let {
    isOpen = false,
    report = null,
    onClose,
  }: {
    isOpen: boolean;
    report: ScanReport | null;
    onClose: () => void;
  } = $props();

  let targetFramework = $state<"auto" | "express" | "nextjs" | "fastapi" | "django" | "nginx" | "sveltekit">("auto");
  let copied = $state(false);

  const frameworks = [
    { id: "auto", label: "Auto-Detect Framework" },
    { id: "express", label: "Node / Express" },
    { id: "nextjs", label: "Next.js / React" },
    { id: "fastapi", label: "Python / FastAPI" },
    { id: "django", label: "Python / Django" },
    { id: "nginx", label: "Nginx / Reverse Proxy" },
    { id: "sveltekit", label: "SvelteKit" },
  ];

  const generatedPrompt = $derived.by(() => {
    if (!report || !report.findings || report.findings.length === 0) return "";

    const stackHint = targetFramework === "auto"
      ? (report.technologies_detected.length > 0 ? `Detected Stack: ${report.technologies_detected.join(", ")}` : "Identify the web server or framework in the codebase.")
      : `Target Framework / Environment: ${frameworks.find(f => f.id === targetFramework)?.label || targetFramework}`;

    let prompt = `You are an expert Application Security and DevSecOps Engineer working with me in Antigravity to secure our codebase.
I ran an automated security posture audit on our application (${report.target_url}) using VulnRadar and discovered ${report.total_findings} security issues (${report.critical_count} Critical, ${report.high_count} High, ${report.medium_count} Medium, ${report.low_count} Low, ${report.info_count} Info).

Please review the security findings below, inspect our project repository, and implement the exact code changes, middleware configurations, header rules, and cookie policies to resolve all issues with zero regressions.

---
### 🌐 Target Application Context:
- **Target URL**: ${report.target_url}
- **Security Health Score**: ${report.security_score}/100
- **HTTP Status Code**: ${report.status_code}
- **Response Latency**: ${report.response_time_ms} ms
- **Tech Stack Context**: ${stackHint}
${report.server_info ? `- **Server Header Disclosed**: ${report.server_info}` : ""}

---
### 🛡️ Discovered Security Findings to Fix:
`;

    report.findings.forEach((finding: Finding, index: number) => {
      prompt += `
#### ${index + 1}. [${finding.severity.toUpperCase()}] ${finding.title}
- **Category**: ${finding.category}
- **OWASP Classification**: ${finding.owasp_category}
${finding.cve_id ? `- **Associated CVE**: ${finding.cve_id}` : ""}
- **Description**: ${finding.description}
- **Security Impact**: ${finding.impact}
- **Recommended Remediation**: ${finding.remediation}
${finding.evidence ? `- **Evidence / Trigger**:\n\`\`\`\n${finding.evidence}\n\`\`\`` : ""}
${finding.references && finding.references.length > 0 ? `- **Advisory References**: ${finding.references.join(", ")}` : ""}
`;
    });

    prompt += `
---
### 🛠️ Implementation Instructions for Antigravity:
1. **Locate Server / Middleware Files**: Search our repository for the HTTP response pipeline, server entry points (e.g. server.js, app.py, main.go, next.config.js, hooks.server.ts, or nginx.conf).
2. **Inject Missing Security Headers**:
   - Content-Security-Policy (CSP without 'unsafe-inline' if possible, or with nonces/hashes).
   - Strict-Transport-Security (HSTS: max-age=31536000; includeSubDomains; preload).
   - X-Content-Type-Options: nosniff.
   - Anti-Clickjacking: X-Frame-Options: DENY / CSP frame-ancestors 'none'.
   - Referrer-Policy: strict-origin-when-cross-origin.
   - Permissions-Policy: camera=(), microphone=(), geolocation=().
3. **Harden Cookie Policies**: Ensure all authentication, session, and CSRF cookies have \`HttpOnly; Secure; SameSite=Lax\` (or \`SameSite=Strict\`).
4. **Sanitize Server Fingerprints**: Strip \`Server:\` and \`X-Powered-By:\` headers from responses.
5. **Preserve Application Functionality**: Ensure styles, fonts, and legitimate API calls continue to function properly.
6. **Provide Clean Diffs**: Output the exact file paths and updated code blocks so they can be immediately committed.
`;

    return prompt;
  });

  async function handleCopy() {
    if (!generatedPrompt) return;
    try {
      await navigator.clipboard.writeText(generatedPrompt);
      copied = true;
      setTimeout(() => (copied = false), 2500);
    } catch {}
  }
</script>

{#if isOpen && report}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4 animate-fade-in print:hidden"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div
      class="bg-[#14151b] border border-white/[0.12] rounded-2xl w-full max-w-2xl max-h-[85vh] flex flex-col shadow-2xl overflow-hidden text-neutral-100"
    >
      <!-- Header -->
      <div
        class="p-4 border-b border-white/[0.08] flex items-center justify-between bg-[#111216] flex-shrink-0"
      >
        <div class="flex items-center gap-2.5">
          <div
            class="w-8 h-8 rounded-lg bg-cyan-500/10 border border-cyan-500/30 flex items-center justify-center text-cyan-400 shadow-sm"
          >
            <Sparkles class="w-4 h-4" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-sm font-bold text-white tracking-tight font-mono">
                Fix Findings with AI
              </h2>
              <span
                class="px-1.5 py-0.2 text-[10px] font-mono bg-cyan-950/40 text-cyan-300 border border-cyan-800/50 rounded font-bold"
              >
                Antigravity Ready
              </span>
            </div>
            <p class="text-[11px] text-neutral-400 font-mono">
              Generates a tailored prompt for Antigravity or any AI coding assistant to remediate all {report.findings.length} findings.
            </p>
          </div>
        </div>

        <button
          type="button"
          onclick={onClose}
          class="p-1 text-neutral-400 hover:text-white hover:bg-white/[0.06] rounded-md transition-colors cursor-pointer"
          aria-label="Close modal"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Body / Framework selector + Prompt Preview -->
      <div class="p-4 overflow-y-auto space-y-3.5 flex-1 bg-[#0f1014]">
        <!-- Framework Selection Bar -->
        <div class="space-y-1.5">
          <label
            for="framework-select"
            class="text-[10px] font-bold uppercase tracking-wider text-neutral-400 font-mono flex items-center gap-1.5"
          >
            <Cpu class="w-3 h-3 text-cyan-400" />
            Target Web Framework / Backend Stack:
          </label>
          <div class="flex flex-wrap items-center gap-1.5" id="framework-select">
            {#each frameworks as fw}
              <button
                type="button"
                onclick={() => (targetFramework = fw.id as any)}
                class="px-2.5 py-1 rounded-md text-xs font-mono transition-colors cursor-pointer {targetFramework === fw.id ? 'bg-cyan-500 text-slate-950 font-bold shadow-sm' : 'bg-[#181921] hover:bg-[#20222c] text-neutral-400 hover:text-neutral-200 border border-white/[0.06]'}"
              >
                {fw.label}
              </button>
            {/each}
          </div>
        </div>

        <!-- Prompt Preview Box -->
        <div class="space-y-1.5">
          <div class="flex items-center justify-between">
            <span class="text-[10px] font-bold uppercase tracking-wider text-neutral-400 font-mono flex items-center gap-1.5">
              <Bot class="w-3 h-3 text-cyan-400" />
              Generated Remediation Prompt ({report.findings.length} Findings Included):
            </span>
            <span class="text-[10px] font-mono text-neutral-400">Markdown Format</span>
          </div>

          <div class="relative group">
            <textarea
              readonly
              value={generatedPrompt}
              rows="12"
              class="w-full p-3 bg-[#13141a] border border-white/[0.08] rounded-xl text-xs font-mono text-neutral-300 leading-relaxed resize-none focus:outline-none select-all"
            ></textarea>
          </div>
        </div>
      </div>

      <!-- Footer Action -->
      <div
        class="p-3.5 border-t border-white/[0.08] flex items-center justify-between gap-3 bg-[#111216] flex-shrink-0"
      >
        <div class="text-[11px] font-mono text-neutral-400 hidden sm:block">
          Paste this prompt directly into Antigravity to apply code diffs.
        </div>

        <div class="flex items-center gap-2 w-full sm:w-auto justify-end">
          <button
            type="button"
            onclick={onClose}
            class="px-3 py-1.5 bg-[#1a1b22] hover:bg-[#22242e] text-neutral-300 rounded-md text-xs font-medium transition-colors cursor-pointer"
          >
            Cancel
          </button>

          <button
            type="button"
            onclick={handleCopy}
            class="px-4 py-1.5 bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-bold rounded-md text-xs transition-all cursor-pointer flex items-center gap-1.5 shadow-sm shadow-cyan-500/20 active:scale-95 flex-shrink-0"
          >
            {#if copied}
              <Check class="w-3.5 h-3.5 text-slate-950" />
              <span>Prompt Copied to Clipboard!</span>
            {:else}
              <Copy class="w-3.5 h-3.5" />
              <span>Copy Prompt for AI</span>
            {/if}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
